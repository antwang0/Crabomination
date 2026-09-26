//! Choices an opponent makes on your spell's behalf ("of an opponent's
//! choice"), and Scrap Mastery's table-wide artifact recycle.

use super::EffectContext;
use crate::card::{CardId, CardInstance, SelectionRequirement};
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// The opponent is the caster's most hostile one; a bot opponent hands
    /// over the costliest match it doesn't control itself, else its own
    /// cheapest. A prompting opponent picks (CR 800.4g routes a departed
    /// seat's pick).
    pub(super) fn opponent_chooses_permanent_then(
        &mut self,
        filter: &SelectionRequirement,
        body: &Effect,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let me = ctx.controller;
        let Some(chooser) = self.default_hostile_opponent(me) else { return Ok(()) };
        let candidates: Vec<&CardInstance> = self
            .battlefield
            .iter()
            .filter(|c| self.evaluate_requirement_on_card(filter, c, me))
            .collect();
        if candidates.is_empty() {
            return Ok(());
        }
        let pick = if self.seat_prompts(chooser)
            || !matches!(self.decider.kind(), crate::decision::DeciderKind::Auto)
        {
            let named: Vec<(CardId, String)> =
                candidates.iter().map(|c| (c.id, c.definition.name.to_string())).collect();
            let Some(ids) = self.ask_seat_cards(
                chooser,
                "Choose a permanent for your opponent's spell".into(),
                ctx.source.unwrap_or(CardId(0)),
                named,
                1,
                1,
                PickValue::Cost,
                effect,
            ) else {
                return Ok(());
            };
            ids.first().copied()
        } else {
            candidates
                .iter()
                .max_by_key(|c| {
                    let mv = c.definition.cost.cmc() as i64;
                    if c.controller != chooser { (1, mv, c.id.0) } else { (0, -mv, c.id.0) }
                })
                .map(|c| c.id)
        };
        let Some(pick) = pick else { return Ok(()) };
        self.run_effect(&Effect::BindTargetObjects { ids: vec![pick], body: Box::new(body.clone()) }, ctx, events)
    }

    /// Three passes over the table (CR 101.4, APNAP): exile each player's
    /// artifact cards, sacrifice each player's artifacts, return what each
    /// exiled. A sacrificed artifact that a replacement sends to exile is
    /// not "exiled this way" and stays there.
    pub(super) fn each_player_recycles_artifacts(
        &mut self,
        _ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let seats = self.apnap_sort((0..self.players.len()).collect());
        let seats: Vec<usize> = seats.into_iter().filter(|&s| self.players[s].is_alive()).collect();
        let mut exiled: Vec<(usize, Vec<CardId>)> = Vec::new();
        for &s in &seats {
            let ids: Vec<CardId> = self.players[s]
                .graveyard
                .iter()
                .filter(|c| c.definition.is_artifact())
                .map(|c| c.id)
                .collect();
            for &id in &ids {
                if let Some(card) = Self::take_card(&mut self.players[s].graveyard, id) {
                    self.exile.push(card);
                }
            }
            exiled.push((s, ids));
        }
        for &s in &seats {
            let mine: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == s && c.definition.is_artifact())
                .map(|c| c.id)
                .collect();
            for id in mine {
                self.sacrifice_one(id, s, events);
            }
        }
        for (s, ids) in exiled {
            for id in ids {
                if let Some(card) = Self::take_card(&mut self.exile, id) {
                    let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(s), tapped: false };
                    self.place_card_in_dest(card, s, &dest, events);
                }
            }
        }
        self.check_state_based_actions_mid_resolution(events);
        Ok(())
    }
}
