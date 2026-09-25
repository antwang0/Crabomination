//! Three Faceless Menace effects that reach past the battlefield:
//!
//! - Kadena's Silencer: "counter all abilities your opponents control" — every
//!   triggered or activated ability on the stack an opponent controls.
//! - Leadership Vacuum: "target player returns each commander they control
//!   from the battlefield to the command zone" (CR 903.9 — a move, not a
//!   death, so no dies trigger).
//! - Sudden Substitution: "exchange control of target noncreature spell and
//!   target creature. Then the spell's controller may choose new targets."

use crate::card::{CardId, Zone};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::{GameEvent, StackItem, Target};
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::CounterAllAbilitiesOf`.
    pub(super) fn counter_all_abilities_of(
        &mut self,
        who: &PlayerRef,
        ctx: &EffectContext,
    ) -> Result<(), GameError> {
        let seats = self.resolve_players(who, ctx);
        self.stack.retain(|si| !matches!(si, StackItem::Trigger { controller, .. } if seats.contains(controller)));
        Ok(())
    }

    /// `Effect::ReturnCommandersToCommandZone`.
    pub(super) fn return_commanders_to_command_zone(
        &mut self,
        who: &PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(seat) = self.resolve_player(who, ctx) else { return Ok(()) };
        let commanders: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && self.players.iter().any(|pl| pl.commanders.contains(&c.id)))
            .map(|c| c.id)
            .collect();
        for id in commanders {
            self.move_card_to(id, &ZoneDest::Command, ctx, events);
        }
        Ok(())
    }

    /// `Effect::ExchangeSpellAndCreatureControl`.
    pub(super) fn exchange_spell_and_creature_control(
        &mut self,
        a: &Selector,
        b: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let first = |g: &Self, s: &Selector| g.resolve_selector(s, ctx).into_iter().find_map(|e| e.as_card_id());
        let (Some(spell), Some(creature)) = (first(self, a), first(self, b)) else {
            return Ok(());
        };
        let Some(spell_ctrl) = self.stack.iter().find_map(|si| match si {
            StackItem::Spell { card, caster, .. } if card.id == spell => Some(*caster),
            _ => None,
        }) else {
            return Ok(());
        };
        let Some(creature_ctrl) = self.battlefield_find(creature).map(|c| c.controller) else {
            return Ok(());
        };
        if spell_ctrl == creature_ctrl {
            return Ok(());
        }
        for si in self.stack.iter_mut() {
            if let StackItem::Spell { card, caster, .. } = si
                && card.id == spell
            {
                *caster = creature_ctrl;
                card.controller = creature_ctrl;
            }
        }
        let c = EffectContext { targets: vec![Target::Permanent(creature)], ..ctx.clone() };
        self.run_effect(
            &Effect::GainControl {
                what: Selector::Target(0),
                to: Some(PlayerRef::Seat(spell_ctrl)),
                duration: Duration::Permanent,
            },
            &c,
            events,
        )?;
        // The spell's new controller may choose new targets (CR 115.7d).
        let c = EffectContext { targets: vec![Target::Permanent(spell)], controller: creature_ctrl, ..ctx.clone() };
        if self.find_card_zone(spell) == Some(Zone::Stack) {
            self.run_effect(&Effect::ChooseNewTargetsForSpell { what: Selector::Target(0) }, &c, events)?;
        }
        Ok(())
    }
}
