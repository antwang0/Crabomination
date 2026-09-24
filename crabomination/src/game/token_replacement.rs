//! CR 614.1a — token replacements applied per token as it is minted, so
//! "that many" holds: "If you would create one or more tokens, you may
//! instead create that many [option] tokens" (Jinnie Fay, Jetmir's Second),
//! and "If you would create a [name] token, create [another] instead"
//! (Fisher's Talent).

use super::GameState;
use crate::card::{CardDefinition, Keyword};
use std::sync::Arc;

impl GameState {
    /// The token `ctrl` gets instead of a creature token `def`, if a
    /// `TokensMayBecome` static they control offers a bigger one. The "may" is
    /// the engine's: it takes the option with the greatest power plus
    /// toughness (haste breaking a tie), and only when that beats `def`;
    /// non-creature tokens (Treasure, Clue) are kept.
    pub(crate) fn token_replacement_for(&self, ctrl: usize, def: &CardDefinition) -> Option<Arc<CardDefinition>> {
        if !def.is_creature() {
            return None;
        }
        let base = def.power + def.toughness;
        self.battlefield
            .iter()
            .filter(|c| c.controller == ctrl)
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match &sa.effect {
                crate::effect::StaticEffect::TokensMayBecome { options } => Some(options),
                _ => None,
            })
            .flatten()
            .filter(|o| o.power + o.toughness > base)
            .max_by_key(|o| (o.power + o.toughness, o.keywords.contains(&Keyword::Haste)))
            .map(crabomination_base::tokens::token_card_arc)
    }

    /// The token `ctrl` gets instead of `def` under a `TokenNamedBecomes`
    /// static they control (a Class's gated level counts only once reached).
    /// Chained, since a Shark made in place of a Fish is then a Shark being
    /// created (CR 614.5 lets each replacement apply once): at most one pass
    /// per such static.
    pub(crate) fn named_token_replacement(&self, ctrl: usize, def: &CardDefinition) -> Option<Arc<CardDefinition>> {
        use crate::effect::StaticEffect;
        let rules: Vec<(&str, &crate::card::TokenDefinition)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == ctrl)
            .flat_map(|c| c.definition.static_abilities.iter().map(move |sa| (c, sa)))
            .filter_map(|(c, sa)| {
                let eff = match &sa.effect {
                    StaticEffect::WhileClassLevelAtLeast { n, inner } if c.class_level >= *n => inner.as_ref(),
                    other => other,
                };
                match eff {
                    StaticEffect::TokenNamedBecomes { name, into } => Some((name.as_str(), into)),
                    _ => None,
                }
            })
            .collect();
        if rules.is_empty() {
            return None;
        }
        let mut used = vec![false; rules.len()];
        let mut name: String = def.name.to_string();
        let mut out: Option<&crate::card::TokenDefinition> = None;
        while let Some(i) = (0..rules.len()).find(|&i| !used[i] && rules[i].0 == name) {
            used[i] = true;
            out = Some(rules[i].1);
            name = rules[i].1.name.clone();
        }
        out.map(crabomination_base::tokens::token_card_arc)
    }
}
