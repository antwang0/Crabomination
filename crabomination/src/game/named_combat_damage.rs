//! "Choose a card name. Whenever a creature with the chosen name deals combat
//! damage to a player this turn, [body]" (The Clone Saga III,
//! `Effect::NameCardThenNamedCreatureCombatDamageThisTurn`). The controller
//! names a card on resolution (CR 608.2d — a choice, not a target); the name
//! arms a turn-scoped delayed trigger (CR 603.7) keyed on
//! `DelayedKind::NamedCreatureDealsCombatDamageToPlayerThisTurn`, which fires
//! once per such creature per combat-damage event whoever controls it.

use super::GameState;
use super::effects::{EffectContext, EntityRef};
use super::types::{DelayedKind, DelayedTrigger, GameError, GameEvent, PendingEffectState, TriggerPush};
use crate::card::CardId;
use crate::decision::{Decision, DecisionAnswer};
use crate::effect::Effect;

impl GameState {
    /// Resolve the naming: ask (or suspend on) the `NameCard` decision. The
    /// headless pick is the controller's own strongest creature — the one most
    /// likely to connect this turn.
    pub(crate) fn resolve_name_card_then_named_combat_damage(
        &mut self,
        body: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let controller = ctx.controller;
        let mut own: Vec<(&'static str, i32)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == controller && self.permanent_is_creature(c.id))
            .map(|c| (c.definition.name, self.computed_permanent(c.id).map(|cp| cp.power).unwrap_or(0)))
            .collect();
        own.sort_by_key(|&(_, p)| std::cmp::Reverse(p));
        let mut suggestions: Vec<String> = Vec::new();
        for (n, _) in own {
            if !suggestions.iter().any(|s| s == n) {
                suggestions.push(n.to_string());
            }
        }
        let source = ctx.source.unwrap_or(CardId(0));
        let decision = Decision::NameCard {
            source,
            source_name: ctx.source_name.unwrap_or_default().to_string(),
            suggestions,
            restriction: None,
        };
        let pending =
            PendingEffectState::NamedCreatureCombatDamagePending { controller, source, body: Box::new(body.clone()) };
        if self.seat_prompts(controller) {
            self.suspend_signal = Some(Box::new((decision, pending, Effect::Noop)));
            return Ok(());
        }
        let answer = self.decider.decide(&decision);
        let mut applied = self.apply_pending_effect_answer(pending, &answer)?;
        events.append(&mut applied);
        Ok(())
    }

    /// The named card arms the watcher. An empty name arms nothing.
    pub(crate) fn apply_named_creature_combat_damage(
        &mut self,
        controller: usize,
        source: CardId,
        body: Effect,
        answer: &DecisionAnswer,
    ) -> Result<Vec<GameEvent>, GameError> {
        let DecisionAnswer::NamedCard(name) = answer else {
            return Err(GameError::DecisionAnswerMismatch);
        };
        if !name.is_empty() {
            self.delayed_triggers.push(DelayedTrigger {
                controller,
                source,
                kind: DelayedKind::NamedCreatureDealsCombatDamageToPlayerThisTurn(name.clone()),
                effect: body,
                target: None,
                bound_token: None,
                bound_subject: None,
                fires_once: false,
                expires_after_turn: None,
            });
        }
        Ok(Vec::new())
    }

    /// Combat damage from `source` to `damaged_player`: fire every armed
    /// watcher whose name is `source`'s (any controller's creature).
    pub(crate) fn fire_named_creature_combat_damage_watchers(
        &mut self,
        source: CardId,
        damaged_player: usize,
        damage_amount: u32,
    ) {
        if !self
            .delayed_triggers
            .iter()
            .any(|dt| matches!(dt.kind, DelayedKind::NamedCreatureDealsCombatDamageToPlayerThisTurn(_)))
        {
            return;
        }
        let Some(name) = self.battlefield_find(source).map(|c| c.definition.name) else { return };
        let watchers: Vec<DelayedTrigger> = self
            .delayed_triggers
            .iter()
            .filter(|dt| matches!(&dt.kind, DelayedKind::NamedCreatureDealsCombatDamageToPlayerThisTurn(n) if n == name))
            .cloned()
            .collect();
        for dt in watchers {
            self.push_stack(
                TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                    .trigger_source(Some(EntityRef::Permanent(source)))
                    .trigger_player(Some(damaged_player))
                    .event_amount(damage_amount)
                    .build(),
            );
        }
    }
}
