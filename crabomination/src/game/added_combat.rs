//! "At the beginning of that combat" (CR 500.8) — a delayed trigger bound to
//! one combat banked by `AdditionalCombatPhaseAfterMain` (Moraug, Last Night
//! Together), not to whichever combat begins next.

use crate::card::CardId;
use crate::effect::Effect;
use crate::game::GameState;
use crate::game::effects::EffectContext;
use crate::game::types::{DelayedKind, DelayedTrigger, TurnStep};

impl GameState {
    /// The turn's combat number of the most recently banked post-main combat.
    /// A precombat-main bank runs after the turn's scheduled combat (unless
    /// that one is skipped), and combats banked "after this combat" run first.
    pub(crate) fn last_added_combat_number(&self) -> u32 {
        let before_combat = matches!(
            self.step,
            TurnStep::Untap | TurnStep::Upkeep | TurnStep::Draw | TurnStep::PreCombatMain
        );
        let scheduled = (before_combat
            && self.players.get(self.active_player_idx).is_none_or(|p| p.skip_next_combat == 0))
            as u32;
        self.combat_phases_this_turn
            .saturating_add(scheduled)
            .saturating_add(self.additional_combat_phases)
            .saturating_add(self.additional_post_main_combats)
    }

    /// Runs `body` once, at the beginning of the combat just banked.
    pub(crate) fn at_the_added_combat(&mut self, body: Effect, ctx: &EffectContext) {
        let n = self.last_added_combat_number();
        self.delayed_triggers.push(DelayedTrigger {
            controller: ctx.controller,
            source: ctx.source.unwrap_or(CardId(0)),
            kind: DelayedKind::CombatNumberThisTurn(n),
            effect: body,
            target: None,
            bound_token: None,
            bound_subject: None,
            fires_once: true,
            expires_after_turn: None,
        });
    }
}
