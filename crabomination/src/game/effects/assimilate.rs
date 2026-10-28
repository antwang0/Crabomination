//! Assimilate (Star Trek Commander keyword action): "put it onto the
//! battlefield under your control with a +1/+1 counter on it. It's a Borg
//! artifact creature and loses all other creature types." A permanent is
//! gained control of instead of moved. Each one is tagged
//! `assimilated_by = source` for `Selector::AssimilatedBySource` (The Best of
//! Both Worlds' "the Borg assimilated with this Saga"), and the Borg Queen's
//! reveal-until-a-creature that feeds it.

use super::EffectContext;
use crate::card::{CardType, CounterType, CreatureType};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    pub(super) fn assimilate(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let ids: Vec<_> = self.resolve_selector(what, ctx).into_iter().filter_map(|e| e.as_card_id()).collect();
        self.assimilate_ids(&ids, ctx, events)
    }

    /// The Borg Queen / Gift of Perfection — each player `who` names reveals
    /// from the top of their library until a creature card; that card is
    /// assimilated (with `one_of_all`, only the best one among every player's
    /// hit — headless: greatest power, then mana value); the rest of each
    /// reveal goes to the bottom in a random order (CR 401.4). The Queen's
    /// "exiles" is read as a reveal: the cards end on the bottom either way.
    pub(super) fn reveal_until_creature_assimilate(
        &mut self,
        who: &PlayerRef,
        one_of_all: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let seats = self.apnap_sort(self.resolve_players(who, ctx));
        let mut hits: Vec<(usize, crate::card::CardId, i32, u32)> = Vec::new();
        let mut reveals: Vec<(usize, Vec<crate::card::CardId>)> = Vec::new();
        for seat in seats {
            if !self.players.get(seat).is_some_and(|p| p.is_alive()) {
                continue;
            }
            let lib = &self.players[seat].library;
            let stop = lib
                .iter()
                .position(|c| crate::game::layers::requirement_matches_card(&crate::card::SelectionRequirement::Creature, c, seat));
            let shown: Vec<_> = lib.iter().take(stop.map_or(lib.len(), |i| i + 1)).map(|c| c.id).collect();
            for c in lib.iter().take(shown.len()) {
                events.push(GameEvent::TopCardRevealed {
                    player: seat,
                    card_name: c.definition.name,
                    is_land: c.definition.is_land(),
                });
            }
            if let Some(i) = stop {
                let c = &lib[i];
                hits.push((seat, c.id, c.definition.power, c.definition.cost.cmc()));
            }
            reveals.push((seat, shown));
        }
        if one_of_all && hits.len() > 1 {
            let best = hits.iter().copied().max_by_key(|&(_, _, p, mv)| (p, mv));
            hits = best.into_iter().collect();
        }
        let ids: Vec<_> = hits.iter().map(|&(_, id, _, _)| id).collect();
        self.assimilate_ids(&ids, ctx, events)?;
        for (seat, shown) in reveals {
            self.bottom_in_random_order(seat, &shown);
        }
        Ok(())
    }

    fn assimilate_ids(
        &mut self,
        ids: &[crate::card::CardId],
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        for &id in ids {
            let one = EffectContext { targets: vec![Target::Permanent(id)], ..ctx.clone() };
            if self.battlefield_find(id).is_some() {
                self.run_effect(
                    &Effect::GainControl { what: Selector::Target(0), to: None, duration: Duration::Permanent },
                    &one,
                    events,
                )?;
            } else {
                self.move_card_to(
                    id,
                    &ZoneDest::Battlefield { controller: PlayerRef::Seat(me), tapped: false },
                    &one,
                    events,
                );
                if self.battlefield_find(id).is_none() {
                    continue;
                }
                self.scratch.last_moved_cards.push(id);
            }
            for e in [
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::AddCardTypeIndefinitely { what: Selector::Target(0), card_type: CardType::Artifact, until_eot: false },
                Effect::AddCardTypeIndefinitely { what: Selector::Target(0), card_type: CardType::Creature, until_eot: false },
                Effect::BecomeCreatureType {
                    what: Selector::Target(0),
                    creature_types: vec![CreatureType::Borg],
                    duration: Duration::Permanent,
                },
            ] {
                self.run_effect(&e, &one, events)?;
            }
            if let (Some(src), Some(c)) = (ctx.source, self.battlefield_find_mut(id)) {
                c.assimilated_by = Some(src);
            }
        }
        Ok(())
    }
}
