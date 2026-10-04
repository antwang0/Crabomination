//! The planar die outside the special action (CR 901.9): Fractured
//! Powerstone's "{T}: Roll the planar die", and Ichor Elixir's extra die,
//! which also reaches the special action.

use crate::effect::StaticEffect;
use crate::game::types::{GameEvent, PlanarFace};
use crate::game::GameState;

impl GameState {
    /// One planar-die result for `seat`. With an Ichor Elixir under `seat`'s
    /// control, one extra die is rolled per Elixir and all but one ignored:
    /// the roller keeps chaos over planeswalking over a blank.
    pub(crate) fn planar_die_face(&mut self, seat: usize) -> PlanarFace {
        let extra = self
            .battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter(|sa| matches!(sa.effect, StaticEffect::ExtraPlanarDie))
            .count();
        let rank = |f: &PlanarFace| match f {
            PlanarFace::Chaos => 2,
            PlanarFace::Planeswalker => 1,
            PlanarFace::Blank => 0,
        };
        (0..=extra)
            .map(|_| self.roll_one_planar_die(seat))
            .max_by_key(rank)
            .unwrap_or(PlanarFace::Blank)
    }

    /// Susan Foreman's replacement (`StaticEffect::PlaneswalkSeesTopTwo`):
    /// before `seat` planeswalks, it keeps one of its planar deck's top two
    /// on top and puts the other on the bottom. Asked like an as-enters
    /// choice: a prompting seat's policy answers (option 0 keeps the top).
    pub(crate) fn order_top_two_planes(&mut self, seat: usize) {
        let has = self.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::PlaneswalkSeesTopTwo))
        });
        let deck = &self.players[seat].planar_deck;
        if !has || deck.len() < 2 {
            return;
        }
        let source = self
            .battlefield
            .iter()
            .find(|c| {
                c.controller == seat
                    && c.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::PlaneswalkSeesTopTwo))
            })
            .map_or(deck[0].id, |c| c.id);
        let decision = crate::decision::Decision::ChooseOption {
            source,
            prompt: "Which plane stays on top?".into(),
            options: vec![deck[0].definition.name.to_string(), deck[1].definition.name.to_string()],
        };
        let answer = if self.seat_prompts(seat) {
            crate::server::bot::decide_pending_policy(
                self,
                seat,
                &crate::server::bot::EvalWeights::default(),
                &decision,
                false,
            )
        } else {
            self.decider.decide(&decision)
        };
        let keep_second = matches!(answer, crate::decision::DecisionAnswer::Amount(1));
        let deck = &mut self.players[seat].planar_deck;
        let bottom = deck.remove(usize::from(!keep_second));
        deck.push(bottom);
    }

    fn roll_one_planar_die(&mut self, seat: usize) -> PlanarFace {
        match self.roll_one_die(seat, 6) {
            1 => PlanarFace::Planeswalker,
            2 => PlanarFace::Chaos,
            _ => PlanarFace::Blank,
        }
    }

    /// `Effect::RollPlanarDie` — CR 901.9d: the roll reports no numeric
    /// result, then chaos ensues or `seat` planeswalks.
    pub(super) fn roll_planar_die_for_effect(&mut self, seat: usize, events: &mut Vec<GameEvent>) {
        let face = self.planar_die_face(seat);
        events.push(GameEvent::DiceRolled { player: seat, count: 1, high: 0 });
        match face {
            PlanarFace::Blank => {}
            PlanarFace::Chaos => self.chaos_ensues(seat),
            PlanarFace::Planeswalker => {
                self.planeswalk(seat);
            }
        }
    }
}
