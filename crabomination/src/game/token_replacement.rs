//! CR 614.1a — "If you would create one or more tokens, you may instead
//! create that many [option] tokens" (Jinnie Fay, Jetmir's Second). Applied
//! per token as it is minted, so "that many" holds.

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
}
