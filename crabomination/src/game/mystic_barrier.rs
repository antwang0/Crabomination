//! Mystic Barrier (CR 508.1a): "When this enchantment enters and at the
//! beginning of your upkeep, choose left or right. Each player may attack only
//! the nearest opponent in the last chosen direction and planeswalkers
//! controlled by that opponent." The restriction rides the CR 803 attack-
//! left / attack-right walk (`attack_left_right_defender`), so the engine's
//! declaration gate and the bot's `attackable_players_for` read one answer.
//! Unlike the variant rule it skips dead seats and teammates: the *nearest*
//! opponent, not the adjacent seat.

use crate::effect::StaticEffect;
use crate::game::GameState;
use crate::game::effects::EffectContext;

impl GameState {
    /// The step of the most recently entered Barrier's choice: +1 (left, the
    /// next seat) or -1 (right). `None` at two seats, where the nearest
    /// opponent in either direction is the only one.
    fn barrier_step(&self) -> Option<isize> {
        if self.players.len() <= 2 {
            return None;
        }
        self.battlefield
            .iter()
            .rev()
            .find(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::AttackOnlyNearestOpponentInChosenDirection))
            })
            .map(|c| if c.modes_chosen.first() == Some(&1) { -1 } else { 1 })
    }

    /// The only seat the active player may attack under a Barrier:
    /// `Some(Some(seat))`, `Some(None)` when no opponent is left, `None` with
    /// no Barrier out.
    pub(crate) fn barrier_defender(&self) -> Option<Option<usize>> {
        let step = self.barrier_step()?;
        let n = self.players.len() as isize;
        let me = self.active_player_idx;
        Some((1..n).map(|k| (me as isize + step * k).rem_euclid(n) as usize).find(|&s| {
            self.players[s].is_alive() && !self.same_team(me, s)
        }))
    }

    /// `Effect::ChooseAttackDirection` — the controller picks left (mode 0)
    /// or right (mode 1); the pick is stamped on the source.
    pub(super) fn choose_attack_direction(&mut self, ctx: &EffectContext) {
        use crate::decision::{Decision, DecisionAnswer};
        let Some(source) = ctx.source else { return };
        let answer = self.decider.decide(&Decision::ChooseMode {
            source,
            num_modes: 2,
            mode_texts: vec!["Left".into(), "Right".into()],
        });
        let pick = match answer {
            DecisionAnswer::Mode(1) => 1u8,
            _ => 0u8,
        };
        if let Some(c) = self.battlefield_find_mut(source) {
            c.modes_chosen = vec![pick];
        }
    }
}
