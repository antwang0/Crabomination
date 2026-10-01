//! Take-backs (TODO "Engine — Rollback / Undo system", step 1): a match's
//! recent undo points, one kept just before each deliberate action of a
//! human seat, and what each is called.
//!
//! The history lives on the match thread beside the `GameState`
//! (`run_match_inner`), never inside it, so a snapshot can't hold the
//! history. A point is a whole-state clone — about a 1.6 KB copy plus
//! refcount bumps (cards are `Arc`-shared and the big groups copy-on-write,
//! PERF `(-143)`), so the ring costs roughly what the game has changed since
//! its oldest point. Automatic actions (`ClientMsg::SubmitAuto`: auto-pass,
//! auto-answered prompts) keep none; bots keep none.
//!
//! Rewinding to a point discards it and every later point, every seat's: the
//! branch played after it is gone, and the next take-back goes one further.
//!
//! A cast that stops for its player to tap mana (`ManualTapRequired`) is paid
//! over the messages after it: the taps, then the client's re-submit. Those
//! taps are part of the cast and keep no points of their own, so one
//! take-back undoes the whole cast, mid-payment or after it, lands and all.

use std::collections::VecDeque;

use crate::game::{GameAction, GameState, TurnStep};
use crate::net::UndoPointView;

/// The most points a match keeps.
pub const MAX_POINTS: usize = 64;

/// Turns of points a match keeps: the current one and the one before.
const TURNS_KEPT: u32 = 2;

/// Where a seat can take back to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndoPoint {
    pub id: u64,
    pub seat: usize,
    pub label: String,
    pub turn: u32,
    pub step: TurnStep,
}

impl UndoPoint {
    pub fn view(&self) -> UndoPointView {
        UndoPointView { id: self.id, label: self.label.clone(), turn: self.turn, step: self.step }
    }
}

/// A match's undo points, oldest first, each with the state just before its
/// action.
#[derive(Default)]
pub struct UndoHistory {
    next_id: u64,
    points: VecDeque<(UndoPoint, GameState)>,
}

impl UndoHistory {
    /// Keep a point for `seat`'s action `label`, taken in `before` (the state
    /// the action is about to change); returns its id. The oldest go past
    /// [`MAX_POINTS`].
    pub fn record(&mut self, seat: usize, label: String, before: GameState) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        let point = UndoPoint { id, seat, label, turn: before.turn_number, step: before.step };
        self.points.push_back((point, before));
        while self.points.len() > MAX_POINTS {
            self.points.pop_front();
        }
        id
    }

    /// Drop point `id` — its action was rejected and changed nothing.
    pub fn discard(&mut self, id: u64) {
        self.points.retain(|(p, _)| p.id != id);
    }

    /// Drop the points older than the turn before `turn`; whether any went.
    pub fn evict_before(&mut self, turn: u32) -> bool {
        let oldest = turn.saturating_sub(TURNS_KEPT - 1);
        let before = self.points.len();
        self.points.retain(|(p, _)| p.turn >= oldest);
        self.points.len() != before
    }

    /// `seat`'s points, oldest first.
    pub fn points_for(&self, seat: usize) -> Vec<UndoPointView> {
        self.points.iter().filter(|(p, _)| p.seat == seat).map(|(p, _)| p.view()).collect()
    }

    /// Take `seat` back to its point `to` (its latest for `None`): the point
    /// and its state, with that point and every later one — any seat's —
    /// removed. `None` if `seat` has no such point.
    pub fn take(&mut self, seat: usize, to: Option<u64>) -> Option<(UndoPoint, GameState)> {
        let at = self
            .points
            .iter()
            .rposition(|(p, _)| p.seat == seat && to.is_none_or(|id| p.id == id))?;
        let mut later = self.points.split_off(at);
        later.pop_front()
    }
}

/// Whether `action` taps a source for mana: the activation of a mana
/// ability, printed or granted.
pub fn is_mana_activation(state: &GameState, action: &GameAction) -> bool {
    let GameAction::ActivateAbility { card_id, ability_index, .. } = action else { return false };
    let Some(card) = state.find_card_anywhere(*card_id) else { return false };
    let printed = &card.definition.activated_abilities;
    match printed.get(*ability_index) {
        Some(ability) => crate::game::actions::is_mana_ability_public(&ability.effect),
        None => state
            .granted_abilities_for(*card_id)
            .get(ability_index - printed.len())
            .is_some_and(|ability| crate::game::actions::is_mana_ability_public(&ability.effect)),
    }
}

/// What `action`, about to be taken in `state`, is called in a take-back:
/// "cast Lightning Bolt", "declared blockers", "passed priority".
pub fn action_label(state: &GameState, action: &GameAction) -> String {
    // Every action is externally tagged; most name their card in one field.
    let json = serde_json::to_value(action).unwrap_or_default();
    let (variant, fields) = match &json {
        serde_json::Value::String(v) => (v.as_str(), None),
        serde_json::Value::Object(m) => match m.iter().next() {
            Some((v, f)) => (v.as_str(), Some(f)),
            None => ("", None),
        },
        _ => ("", None),
    };
    // A tuple variant carries the id bare (`PlayLand(id)`), a struct variant
    // in a field.
    const CARD_FIELDS: [&str; 7] = ["card_id", "creature_id", "source", "card", "equipment", "vehicle", "permanent"];
    let card = fields
        .and_then(|f| f.as_u64().or_else(|| CARD_FIELDS.iter().find_map(|k| f.get(k)?.as_u64())))
        .and_then(|id| state.find_card_anywhere(crate::card::CardId(id as u32)))
        .map(|c| c.definition.name);
    let verb = match variant {
        "PassPriority" => return "passed priority".to_string(),
        "DeclareAttackers" | "DeclareAttackersBanded" | "DeclareAttackersExerting" => {
            return "declared attackers".to_string();
        }
        "DeclareBlockers" => return "declared blockers".to_string(),
        "SubmitDecision" => return "answered a choice".to_string(),
        "Concede" => return "conceded".to_string(),
        "RollPlanarDie" => return "rolled the planar die".to_string(),
        v if v.starts_with("PlayLand") => "played",
        v if v.starts_with("Activate") => "activated",
        "Suspend" => "suspended",
        "Foretell" => "foretold",
        "Plot" => "plotted",
        "Cycle" | "Landcycle" => "cycled",
        "Equip" => "equipped with",
        "Reconfigure" => "reconfigured",
        "Crew" => "crewed",
        "Saddle" => "saddled",
        "Ninjutsu" => "ninjutsu'd",
        "Reinforce" => "reinforced with",
        "UnlockRoomDoor" => "unlocked a door of",
        "CompanionToHand" => "put into hand",
        "RevealConspiracy" => "revealed",
        v if v.starts_with("TurnFaceUp") => "turned face up",
        v if v.starts_with("Cast") => "cast",
        _ => "acted with",
    };
    match card {
        Some(name) => format!("{verb} {name}"),
        None => verb.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::game::two_player_game;

    /// A take-back goes to the seat's latest point and drops it and every
    /// later point, any seat's; the next goes one further back.
    #[test]
    fn taking_back_drops_the_point_and_everything_after() {
        let g = two_player_game();
        let mut h = UndoHistory::default();
        let a = h.record(0, "a".into(), g.clone());
        let b = h.record(1, "b".into(), g.clone());
        let c = h.record(0, "c".into(), g.clone());
        let d = h.record(1, "d".into(), g.clone());
        let (p, _) = h.take(0, None).unwrap();
        assert_eq!(p.id, c);
        assert_eq!(h.points_for(1).iter().map(|p| p.id).collect::<Vec<_>>(), [b], "d went with c");
        let (p, _) = h.take(0, None).unwrap();
        assert_eq!(p.id, a);
        assert!(h.take(0, None).is_none());
        assert!(h.points_for(1).is_empty(), "b came after a");
        let _ = d;
    }

    /// A rejected action's point goes; the history keeps two turns and at
    /// most [`MAX_POINTS`].
    #[test]
    fn the_history_is_bounded() {
        let mut g = two_player_game();
        let mut h = UndoHistory::default();
        let rejected = h.record(0, "x".into(), g.clone());
        h.discard(rejected);
        assert!(h.points_for(0).is_empty());
        for turn in 1..=4 {
            g.turn_number = turn;
            h.record(0, format!("t{turn}"), g.clone());
        }
        assert!(h.evict_before(4));
        let turns: Vec<u32> = h.points_for(0).iter().map(|p| p.turn).collect();
        assert_eq!(turns, [3, 4]);
        for _ in 0..(MAX_POINTS + 5) {
            h.record(0, "y".into(), g.clone());
        }
        assert_eq!(h.points_for(0).len(), MAX_POINTS);
    }

    /// A rewind restores the state exactly — serialized, it is the snapshot
    /// — and its randomness: a shuffle after the rewind comes out as the
    /// same shuffle did after the snapshot.
    #[test]
    fn a_rewind_restores_the_state_and_its_randomness() {
        use rand::seq::SliceRandom;
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        for _ in 0..12 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let forest = g.add_card_to_hand(0, catalog::forest());
        let before = g.clone();
        let json = serde_json::to_string(&before).unwrap();
        let mut probe = before.clone();
        probe.players[0].library.shuffle(&mut probe.rng.draw());
        let shuffled: Vec<_> = probe.players[0].library.iter().map(|c| c.id).collect();

        g.perform_action(GameAction::PlayLand(forest)).expect("play the Forest");
        g.players[0].library.shuffle(&mut g.rng.draw());
        g.rewind_to(before);
        assert_eq!(serde_json::to_string(&g).unwrap(), json, "the snapshot, exactly");
        g.players[0].library.shuffle(&mut g.rng.draw());
        let again: Vec<_> = g.players[0].library.iter().map(|c| c.id).collect();
        assert_eq!(again, shuffled, "the same shuffle");
    }

    /// Taking back a decision's answer puts the decision back.
    #[test]
    fn a_rewind_reposes_an_answered_decision() {
        use crate::decision::DecisionAnswer;
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].wants_ui = true;
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        let opt = g.add_card_to_hand(0, catalog::opt());
        g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
        g.perform_action(GameAction::CastSpell { card_id: opt, target: None, additional_targets: vec![], mode: None, x_value: None })
            .expect("cast Opt");
        crate::game::drain_stack(&mut g);
        let asked = g.pending_decision.clone().expect("Opt's scry asks");
        let before = g.clone();
        let top = g.players[0].library.iter().next().map(|c| c.id).into_iter().collect();
        g.perform_action(GameAction::SubmitDecision(DecisionAnswer::ScryOrder { kept_top: top, bottom: vec![] }))
            .expect("answer the scry");
        assert!(g.pending_decision.is_none());
        g.rewind_to(before);
        assert_eq!(
            format!("{:?}", g.pending_decision.as_ref().map(|p| &p.decision)),
            format!("{:?}", Some(&asked.decision)),
            "the scry is asked again"
        );
    }

    /// A cast stopped for the player to tap mana has already tapped its
    /// forced sources (`ManualTapRequired`); rewinding to before it gives
    /// them back.
    #[test]
    fn a_rewind_untaps_a_half_paid_cast() {
        let mut g = two_player_game();
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.players[0].wants_ui = true;
        g.players[0].manual_mana = true;
        let forests = [g.add_card_to_battlefield(0, catalog::forest()), g.add_card_to_battlefield(0, catalog::forest())];
        // Five sources for the generic {4}: a choice, so the cast stops for
        // the player to make it.
        for _ in 0..4 {
            g.add_card_to_battlefield(0, catalog::mountain());
        }
        g.add_card_to_battlefield(0, catalog::island());
        let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
        let before = g.clone();
        let cast = GameAction::CastSpell { card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None };
        assert!(matches!(g.perform_action(cast), Err(crate::game::GameError::ManualTapRequired { .. })));
        assert!(forests.iter().all(|f| g.battlefield_find(*f).unwrap().tapped), "forced green tapped");
        g.rewind_to(before);
        assert!(forests.iter().all(|f| !g.battlefield_find(*f).unwrap().tapped), "and untapped again");
        assert_eq!(g.players[0].mana_pool.total(), 0);
    }

    /// Actions read as what they did, by the card they did it with.
    #[test]
    fn actions_are_labelled_by_their_card() {
        let mut g = two_player_game();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let forest = g.add_card_to_hand(0, catalog::forest());
        let cast = GameAction::CastSpell { card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None };
        assert_eq!(action_label(&g, &cast), "cast Lightning Bolt");
        assert_eq!(action_label(&g, &GameAction::PlayLand(forest)), "played Forest");
        assert_eq!(action_label(&g, &GameAction::PassPriority), "passed priority");
        assert_eq!(action_label(&g, &GameAction::DeclareBlockers(vec![])), "declared blockers");
    }
}
