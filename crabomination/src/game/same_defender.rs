//! `SelectionRequirement::IsAttackingSameAsSource` — "another target creature
//! attacking the same player or planeswalker" (Kitesail Skirmisher, CR 506.2):
//! both are attacking and their declared attack targets are equal.

use super::GameState;
use crate::card::CardId;

impl GameState {
    pub(crate) fn attacking_same_as_source(&self, id: CardId, source: Option<CardId>) -> bool {
        let Some(src) = source else { return false };
        let target_of = |c: CardId| self.attacking.iter().find(|a| a.attacker == c).map(|a| a.target);
        target_of(id).is_some_and(|t| target_of(src) == Some(t))
    }
}
