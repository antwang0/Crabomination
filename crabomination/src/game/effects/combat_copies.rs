//! Token copies that enter combat blocking (Mirror Match).

use super::EffectContext;
use crate::effect::AttackingTokenCleanup;
use crate::game::GameState;
use crate::game::types::{AttackTarget, GameError, GameEvent};

impl GameState {
    /// Each copy is put onto the battlefield blocking its original; it was
    /// never declared a blocker, so no `BlockerDeclared` and no "whenever ~
    /// blocks" trigger (the blocking twin of CR 508.4).
    /// A creature attacking a teammate or another opponent is not "attacking
    /// you" (CR 506.3).
    pub(super) fn copy_attackers_as_blockers(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let attackers: Vec<_> = self
            .attacking
            .iter()
            .filter(|a| match a.target {
                AttackTarget::Player(p) => p == me,
                AttackTarget::Planeswalker(pw) => self.battlefield_find(pw).is_some_and(|c| c.controller == me),
                AttackTarget::Battle(_) => false,
            })
            .map(|a| a.attacker)
            .collect();
        for attacker in attackers {
            let Some(def) = self.battlefield_find(attacker).map(|c| c.definition.arc()) else { continue };
            let token = self.mint_token_onto_battlefield(def, me, false, events);
            self.add_block(token, attacker);
            if !self.blocked_attackers.contains(&attacker) {
                self.blocked_attackers.push(attacker);
            }
            self.attacking_token_cleanup.push((token, AttackingTokenCleanup::ExileAtEndOfCombat));
        }
        Ok(())
    }
}
