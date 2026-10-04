//! "Only the chosen creatures can attack during that combat phase" (CR
//! 508.1c) — Last Night Together's extra combat.

use crate::card::{CardId, Keyword};
use crate::effect::{Duration, Effect};
use crate::game::GameState;
use crate::game::effects::EffectContext;
use crate::game::layers::Modification;
use crate::game::types::{DelayedKind, DelayedTrigger, Target};

impl GameState {
    /// Registers the restriction for the combat the same spell just banked
    /// (CR 500.8 — not the turn's scheduled one), keyed on its targets.
    pub(crate) fn only_targets_attack_next_combat(&mut self, ctx: &EffectContext) {
        let except: Vec<CardId> = ctx
            .targets
            .iter()
            .filter_map(|t| match t {
                Target::Permanent(id) => Some(*id),
                Target::Player(_) => None,
            })
            .collect();
        self.delayed_triggers.push(DelayedTrigger {
            controller: ctx.controller,
            source: ctx.source.unwrap_or(CardId(0)),
            kind: DelayedKind::CombatNumberThisTurn(self.last_added_combat_number()),
            effect: Effect::CantAttackThisCombatExcept { except },
            target: None,
            bound_token: None,
            bound_subject: None,
            fires_once: true,
            expires_after_turn: None,
        });
    }

    /// Each creature `ctx.controller` controls outside `except` can't attack
    /// until end of combat.
    pub(crate) fn cant_attack_this_combat_except(&mut self, except: &[CardId], ctx: &EffectContext) {
        let barred: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == ctx.controller && !except.contains(&c.id) && self.computed_is_creature(c))
            .map(|c| c.id)
            .collect();
        let source = ctx.source.unwrap_or(CardId(0));
        for id in barred {
            self.keyword_layer_effect(
                id,
                Modification::AddKeyword(Keyword::CantAttack),
                Duration::EndOfCombat,
                ctx.controller,
                source,
            );
        }
    }
}
