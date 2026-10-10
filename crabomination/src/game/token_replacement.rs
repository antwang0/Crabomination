//! CR 614.1a — token replacements applied per token as it is minted, so
//! "that many" holds: "If you would create one or more tokens, you may
//! instead create that many [option] tokens" (Jinnie Fay, Jetmir's Second),
//! and "If you would create a [name] token, create [another] instead"
//! (Fisher's Talent).

use super::GameState;
use crate::card::{CardDefinition, Keyword, TokenDefinition};
use std::sync::Arc;

impl GameState {
    /// The token `ctrl` gets instead of `def`. Divine Visitation's
    /// `CreatureTokensBecome` is not a choice: the first one applies to a
    /// creature token. A `TokensMayBecome` static (Jinnie Fay) is: "you may
    /// instead create that many" of one option, any token kind — asked once
    /// per creation event and reused for the rest of its tokens
    /// (`token_replacement_pick`). The ballot leads with the headless pick:
    /// the biggest option (haste breaking a tie) when it beats a creature
    /// token's own body, else the token as printed.
    pub(crate) fn token_replacement_for(&mut self, ctrl: usize, def: &CardDefinition) -> Option<Arc<CardDefinition>> {
        if def.is_creature() {
            let mandatory = self
                .battlefield
                .iter()
                .filter(|c| c.controller == ctrl)
                .flat_map(|c| c.definition.static_abilities.iter())
                .find_map(|sa| match &sa.effect {
                    crate::effect::StaticEffect::CreatureTokensBecome { into } => Some(into),
                    _ => None,
                });
            if let Some(into) = mandatory {
                return Some(crabomination_base::tokens::token_card_arc(into));
            }
        }
        let offers = |c: &crate::card::CardInstance| {
            c.controller == ctrl
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, crate::effect::StaticEffect::TokensMayBecome { .. }))
        };
        let source = self.battlefield.iter().find(|c| offers(c))?.id;
        let options: Vec<TokenDefinition> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == ctrl)
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match &sa.effect {
                crate::effect::StaticEffect::TokensMayBecome { options } => Some(options),
                _ => None,
            })
            .flatten()
            .cloned()
            .collect();
        // One creation event, one answer: the batch shares `def`'s allocation.
        let key = (std::ptr::from_ref(def) as usize, ctrl);
        let pick = match self.scratch.token_replacement_pick {
            Some((p, c, i)) if (p, c) == key => i as usize,
            _ => {
                let i = self.ask_token_replacement(ctrl, source, def, &options);
                self.scratch.token_replacement_pick = Some((key.0, key.1, i as u8));
                i
            }
        };
        pick.checked_sub(1).and_then(|k| options.get(k)).map(crabomination_base::tokens::token_card_arc)
    }

    /// Jinnie Fay's ballot: 0 keeps `def`, `k` takes `options[k - 1]`. The
    /// headless pick leads; a headless seat is not asked.
    fn ask_token_replacement(
        &mut self,
        ctrl: usize,
        source: crate::card::CardId,
        def: &CardDefinition,
        options: &[TokenDefinition],
    ) -> usize {
        let base = def.power + def.toughness;
        let best = options
            .iter()
            .enumerate()
            .filter(|(_, o)| def.is_creature() && o.power + o.toughness > base)
            .max_by_key(|(_, o)| (o.power + o.toughness, o.keywords.contains(&Keyword::Haste)))
            .map_or(0, |(k, _)| k + 1);
        let mut order = vec![best];
        order.extend((0..=options.len()).filter(|&i| i != best));
        if !self.seat_prompts(ctrl) && matches!(self.decider.kind(), crate::decision::DeciderKind::Auto) {
            return best;
        }
        let label = |i: usize| match i {
            0 => format!("Keep {}", def.name),
            k => {
                let o = &options[k - 1];
                format!("{} {}/{}", o.name, o.power, o.toughness)
            }
        };
        let decision = crate::decision::Decision::ChooseOption {
            source,
            prompt: "Create these tokens instead?".to_string(),
            options: order.iter().map(|&i| label(i)).collect(),
        };
        match self.ask_entering(ctrl, &decision) {
            crate::decision::DecisionAnswer::Amount(n) => order.get(n as usize).copied().unwrap_or(best),
            _ => best,
        }
    }

    /// CR 614.1a — the extra tokens a `name` mint also creates under `ctrl`
    /// (Bilbo, Fellow Conspirator's Treasure beside each Food). Empty, and
    /// unallocated, when nothing applies.
    pub(crate) fn named_token_extras(&self, ctrl: usize, name: &str) -> Vec<crate::card::TokenDefinition> {
        self.battlefield
            .iter()
            .filter(|c| c.controller == ctrl)
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match &sa.effect {
                crate::effect::StaticEffect::TokenNamedAlsoMints { name: n, also } if n == name => Some(also.clone()),
                _ => None,
            })
            .collect()
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
