//! Turtle Power! (TMC, Heroes in a Half Shell) primitives: Myriad's copies
//! with their own terms (Shredder, Shadow Master — non-legendary, sacrificed,
//! only off an attack on a player).

use super::EffectContext;
use crate::effect::AttackingTokenCleanup;
use crate::game::GameState;
use crate::game::types::{AttackTarget, GameEvent};

impl GameState {
    /// CR 702.116a's shape: for each opponent of the source's controller
    /// other than the one it attacks, a token copy of the source enters
    /// tapped and attacking that opponent, cleaned up by `cleanup`.
    /// `non_legendary` strips Legendary (CR 707.2e); `player_only` makes a
    /// source attacking a planeswalker or battle do nothing ("attacks a
    /// player"). No-op when the source isn't attacking.
    pub(super) fn copies_attack_each_other_opponent(
        &mut self,
        ctx: &EffectContext,
        non_legendary: bool,
        cleanup: AttackingTokenCleanup,
        player_only: bool,
        events: &mut Vec<GameEvent>,
    ) {
        let Some(src) = ctx.source else { return };
        let Some(src_attack) = self.attacking.iter().find(|a| a.attacker == src) else { return };
        let defending = match src_attack.target {
            AttackTarget::Player(p) => p,
            _ if player_only => return,
            AttackTarget::Planeswalker(pw) => self.battlefield_find(pw).map(|c| c.controller).unwrap_or(usize::MAX),
            AttackTarget::Battle(b) => self.battlefield_find(b).and_then(|c| c.protected_by).unwrap_or(usize::MAX),
        };
        let Some((ctrl, def)) = self.battlefield_find(src).map(|c| (c.controller, c.definition.arc())) else {
            return;
        };
        let def = if non_legendary {
            let mut d = (*def).clone();
            d.supertypes.retain(|s| *s != crate::card::Supertype::Legendary);
            std::sync::Arc::new(d)
        } else {
            def
        };
        // A seat that has left the game is no opponent (CR 800.4a).
        let mut opps = self.opponents_of(ctrl);
        opps.retain(|&q| q != defending);
        for opp in opps {
            let id = self.mint_token_onto_battlefield(def.clone(), ctrl, true, events);
            if self.put_into_combat_attacking(id, AttackTarget::Player(opp)) && cleanup != AttackingTokenCleanup::None {
                self.attacking_token_cleanup.push((id, cleanup));
            }
        }
    }
}
