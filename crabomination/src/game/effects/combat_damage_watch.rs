//! CR 603.7d — a delayed trigger watching a creature's combat damage until
//! its controller's next turn (Tamiyo, Field Researcher's +1): the trigger is
//! the effect's controller's, whoever controls the creature.

use super::EffectContext;
use crate::card::CardId;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::{DelayedKind, DelayedTrigger, TriggerPush};

impl GameState {
    /// `Effect::WatchCombatDamageUntilYourNextTurn` — one watcher per permanent.
    pub(super) fn watch_combat_damage_until_your_next_turn(
        &mut self,
        what: &Selector,
        body: &Effect,
        to_player: bool,
        ctx: &EffectContext,
    ) {
        let source = ctx.source.unwrap_or(CardId(0));
        for ent in self.resolve_selector(what, ctx) {
            let Some(cid) = ent.as_permanent_id() else { continue };
            self.delayed_triggers.push(DelayedTrigger {
                controller: ctx.controller,
                source,
                kind: if to_player {
                    DelayedKind::SourceDealsCombatDamageToPlayerUntilYourNextTurn(cid)
                } else {
                    DelayedKind::SourceDealsCombatDamageUntilYourNextTurn(cid)
                },
                effect: body.clone(),
                target: None,
                bound_token: None,
                bound_subject: None,
                fires_once: false,
                expires_after_turn: None,
            });
        }
    }

    /// Fire the watchers on `source` for one combat-damage assignment; the
    /// player-only ones when `to_player`.
    pub(crate) fn fire_combat_damage_watchers(&mut self, source: CardId, amount: u32, to_player: bool) {
        if amount == 0 || self.delayed_triggers.is_empty() {
            return;
        }
        let watchers: Vec<DelayedTrigger> = self
            .delayed_triggers
            .iter()
            .filter(|dt| match dt.kind {
                DelayedKind::SourceDealsCombatDamageUntilYourNextTurn(id) => id == source,
                DelayedKind::SourceDealsCombatDamageToPlayerUntilYourNextTurn(id) => to_player && id == source,
                _ => false,
            })
            .cloned()
            .collect();
        for dt in watchers {
            self.push_stack(
                TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                    .trigger_source(Some(crate::game::effects::EntityRef::Permanent(source)))
                    .event_amount(amount)
                    .build(),
            );
        }
    }
}
