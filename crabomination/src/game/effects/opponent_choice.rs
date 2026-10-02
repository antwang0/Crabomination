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
    #[allow(clippy::too_many_arguments)]
    pub(super) fn opponent_chooses_permanent_then(
        &mut self,
        filter: &SelectionRequirement,
        body: &Effect,
        chooser: Option<&crate::effect::PlayerRef>,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use crate::game::types::Target;
        let me = ctx.controller;
        let asks = |g: &GameState, seat: usize| {
            g.seat_prompts(seat) || !matches!(g.decider.kind(), crate::decision::DeciderKind::Auto)
        };
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let chooser = match chooser {
            Some(who) => match self.resolve_player(who, ctx) {
                Some(p) => p,
                None => return Ok(()),
            },
            None => {
                let Some(hostile) = self.default_hostile_opponent(me) else { return Ok(()) };
                let opps: Vec<Target> = std::iter::once(hostile)
                    .chain(self.opponents_of(me).into_iter().filter(|&q| q != hostile))
                    .map(Target::Player)
                    .collect();
                if opps.len() > 1 && asks(self, me) {
                    match self.ask_seat_target_logged(&mut cursor, me, "Choose an opponent to choose".into(), source, opps, effect) {
                        None => return Ok(()),
                        Some(Target::Player(q)) => q,
                        Some(_) => hostile,
                    }
                } else {
                    hostile
                }
            }
        };
        let candidates: Vec<&CardInstance> = self
            .battlefield
            .iter()
            .filter(|c| self.evaluate_requirement_on_card(filter, c, me))
            .collect();
        if candidates.is_empty() {
            self.clear_answer_log();
            return Ok(());
        }
        // Headless: the chooser gives up someone else's priciest, else its
        // own cheapest.
        let auto = candidates
            .iter()
            .max_by_key(|c| {
                let mv = c.definition.cost.cmc() as i64;
                if c.controller != chooser { (1, mv, c.id.0) } else { (0, -mv, c.id.0) }
            })
            .map(|c| c.id);
        let named: Vec<(CardId, String)> = candidates.iter().map(|c| (c.id, c.definition.name.to_string())).collect();
        let Some(ids) = self.ask_seat_cards_logged(
            &mut cursor,
            chooser,
            "Choose a permanent for your opponent's spell".into(),
            source,
            named,
            1,
            1,
            PickValue::Cost,
            effect,
            auto.into_iter().collect(),
        ) else {
            return Ok(());
        };
        self.clear_answer_log();
        let Some(pick) = ids.first().copied().or(auto) else { return Ok(()) };
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
        let seats = self.apnap_sort(self.living_seats().collect());
        let mut exiled: Vec<(usize, Vec<CardId>)> = Vec::new();
        for &s in &seats {
            let ids: Vec<CardId> = self.players[s]
                .graveyard
                .iter()
                .filter(|c| self.computed_has_card_type(c, crate::card::CardType::Artifact))
                .map(|c| c.id)
                .collect();
            for &id in &ids {
                if let Some(card) = Self::take_card(&mut self.players[s].graveyard, id) {
                    self.exile.push(card);
                    self.note_exiled_from_graveyard(s, id, events);
                }
            }
            exiled.push((s, ids));
        }
        for &s in &seats {
            let mine: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| c.controller == s && self.computed_has_card_type(c, crate::card::CardType::Artifact))
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
