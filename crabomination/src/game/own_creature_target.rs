//! Professor Hojo — "the first activated ability you activate during your
//! turn that targets a creature you control costs {2} less to activate".
//! `Player::own_creature_ability_this_turn` spends the discount; equip counts.

use super::GameState;
use super::types::Target;
use crate::effect::StaticEffect;

impl GameState {
    /// Whether any of `targets` is a creature `p` controls.
    pub(crate) fn targets_own_creature<'a>(&self, p: usize, mut targets: impl Iterator<Item = &'a Target>) -> bool {
        targets.any(|t| {
            matches!(t, Target::Permanent(id)
                if self.battlefield_find(*id).is_some_and(|c| c.controller == p)
                    && self.permanent_is_creature(*id))
        })
    }

    /// True while `p`'s first own-creature-targeting activation of the turn
    /// is still unspent (only `p`'s own turn counts).
    pub(crate) fn own_creature_ability_unspent(&self, p: usize) -> bool {
        self.active_player_idx == p && !self.players.get(p).is_none_or(|pl| pl.own_creature_ability_this_turn)
    }

    /// The summed `FirstOwnCreatureTargetingAbilityCostsLess` discount for an
    /// activation by `p` that targets one of `p`'s creatures — 0 once spent.
    pub(crate) fn own_creature_target_discount(&self, p: usize) -> u32 {
        if !self.own_creature_ability_unspent(p) {
            return 0;
        }
        self.battlefield
            .iter()
            .filter(|c| c.controller == p)
            .flat_map(|c| c.definition.static_abilities.iter())
            .map(|sa| match sa.effect {
                StaticEffect::FirstOwnCreatureTargetingAbilityCostsLess { amount } => amount,
                _ => 0,
            })
            .sum()
    }
}
