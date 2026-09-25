//! Token copies that enter combat: blocking (Mirror Match) or attacking
//! (Gyrus, Waker of Corpses).

use super::EffectContext;
use crate::effect::{AttackingTokenCleanup, Selector};
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

    /// Gyrus's copy: a token copy of the card `source` names, put into the
    /// current combat tapped and attacking (CR 508.4 — never declared), and
    /// exiled at end of combat.
    pub(super) fn token_copy_attacking_until_end_of_combat(
        &mut self,
        source: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        if self.attacking.is_empty() {
            return Ok(());
        }
        let me = ctx.controller;
        let Some(def) = self
            .resolve_selector(source, ctx)
            .into_iter()
            .find_map(|e| e.as_card_id())
            .and_then(|id| self.find_card_anywhere(id))
            .map(|c| c.definition.arc())
        else {
            return Ok(());
        };
        let target = ctx
            .source
            .and_then(|src| self.attacking.iter().find(|a| a.attacker == src))
            .map(|a| a.target)
            .or_else(|| self.default_hostile_opponent(me).map(AttackTarget::Player));
        let Some(target) = target else { return Ok(()) };
        let token = self.mint_token_onto_battlefield(def, me, true, events);
        if self.put_into_combat_attacking(token, target) {
            self.attacking_token_cleanup.push((token, AttackingTokenCleanup::ExileAtEndOfCombat));
        }
        Ok(())
    }

    /// `Effect::TokenCopyTappedAttacking` — the token stays after combat and
    /// is `last_created_token` for a chained rider (Satya).
    pub(super) fn token_copy_tapped_attacking(
        &mut self,
        source: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        if self.attacking.is_empty() {
            return Ok(());
        }
        let me = ctx.controller;
        let Some(def) = self
            .resolve_selector(source, ctx)
            .into_iter()
            .find_map(|e| e.as_card_id())
            .and_then(|id| self.find_card_anywhere(id))
            .map(|c| c.definition.arc())
        else {
            return Ok(());
        };
        let target = ctx
            .source
            .and_then(|src| self.attacking.iter().find(|a| a.attacker == src))
            .map(|a| a.target)
            .or_else(|| self.default_hostile_opponent(me).map(AttackTarget::Player));
        let Some(target) = target else { return Ok(()) };
        let token = self.mint_token_onto_battlefield(def, me, true, events);
        self.put_into_combat_attacking(token, target);
        self.last_created_token = Some(token);
        Ok(())
    }
}
