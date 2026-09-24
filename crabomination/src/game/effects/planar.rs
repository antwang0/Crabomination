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

    fn roll_one_planar_die(&mut self, seat: usize) -> PlanarFace {
        match self.decider.decide(&crate::decision::Decision::DieRoll { player: seat, sides: 6 }) {
            crate::decision::DecisionAnswer::DieRoll(1) => PlanarFace::Planeswalker,
            crate::decision::DecisionAnswer::DieRoll(2) => PlanarFace::Chaos,
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
