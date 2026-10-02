//! Take-backs (TODO "Engine — Rollback / Undo system", steps 1–5): a match's
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
//!
//! **No fishing** (step 2). A take-back mustn't be a way to try again for a
//! better roll of the dice:
//! - the game's own randomness comes back with the state (its stream
//!   position is part of the clone), so a shuffle, a draw or a coin flip
//!   replays the same;
//! - each bot decision is pinned to a seed from the state it is made in
//!   ([`decision_seed`], [`pin_bot`]), so the same position always gets the
//!   same reply — its blocks, its counterspell;
//! - what the undone stretch *showed* can't be taken back, so the rewind
//!   says what it was ([`seen_since`]): "1 draw", "a coin flip", "Bot's
//!   hand".
//!
//! **Consent** (step 4). With other people at the table a take-back is a
//! request ([`Asked`]): the match pauses — no actions, no bot moves, the
//! rope and the chess clock stopped — while each other human still at the
//! table allows or declines it, for [`ASK_FOR`] at most. Bots allow. Every
//! allow applies it; a decline, the deadline, a player leaving or a
//! concession ends it, and play goes on.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::game::{GameAction, GameState, TurnStep};
use crate::net::{GameEventWire, ServerMsg, UndoPointView};
use crate::server::mcts::PinnedStreams;

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
    /// How much of the match's [`Seen`] journal came before it.
    seen_at: usize,
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
        let point = UndoPoint { id, seat, label, turn: before.turn_number, step: before.step, seen_at: seen_len() };
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

    /// `seat`'s point `to` (its latest for `None`) and the state it keeps,
    /// left in place.
    pub fn peek(&self, seat: usize, to: Option<u64>) -> Option<(&UndoPoint, &GameState)> {
        self.find(seat, to).map(|at| (&self.points[at].0, &self.points[at].1))
    }

    /// Take `seat` back to its point `to` (its latest for `None`): the point
    /// and its state, with that point and every later one — any seat's —
    /// removed. `None` if `seat` has no such point. What the undone stretch
    /// showed is forgotten: it is no longer the game's.
    pub fn take(&mut self, seat: usize, to: Option<u64>) -> Option<(UndoPoint, GameState)> {
        let at = self.find(seat, to)?;
        let mut later = self.points.split_off(at);
        let taken = later.pop_front()?;
        SEEN.with(|s| {
            if let Some(journal) = s.borrow_mut().as_mut() {
                journal.truncate(taken.0.seen_at);
            }
        });
        Some(taken)
    }

    fn find(&self, seat: usize, to: Option<u64>) -> Option<usize> {
        self.points.iter().rposition(|(p, _)| p.seat == seat && to.is_none_or(|id| p.id == id))
    }
}

/// How long the other players have to answer a take-back request. Two
/// seconds in this crate's tests, so the deadline test waits that and no
/// more.
pub const ASK_FOR: Duration = if cfg!(test) { Duration::from_secs(2) } else { Duration::from_secs(30) };

/// A take-back waiting on the table's consent: `by`'s point `to`.
#[derive(Clone, Debug)]
pub struct Asked {
    pub by: usize,
    pub to: u64,
    pub label: String,
    /// What it would undo that was seen ([`seen_since`]).
    pub saw: Vec<String>,
    /// The human seats still to answer.
    pub waiting: Vec<usize>,
    pub deadline: Instant,
}

impl Asked {
    /// The request as the table hears it, with the time left.
    pub fn message(&self) -> ServerMsg {
        ServerMsg::UndoRequested {
            by: self.by,
            label: self.label.clone(),
            saw: self.saw.clone(),
            seconds: self.deadline.saturating_duration_since(Instant::now()).as_secs_f32().ceil() as u32,
            waiting: self.waiting.clone(),
        }
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

// ── No fishing ──────────────────────────────────────────────────────────

/// The seed a bot's decision as `seat` in `state` is pinned to: a function
/// of the state alone, so a rewound position, played into the same way,
/// gets the same reply. Mixed from the game's stream position (secret, and
/// unique to the match) and the counters that move with play. Two different
/// positions sharing a seed would cost nothing: each still gets one reply.
pub fn decision_seed(state: &GameState, seat: usize) -> u64 {
    use std::hash::{Hash, Hasher};
    // Fixed keys: the same state hashes the same for the life of the
    // process, which is all a rewind needs.
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (state.rng.position(), state.turn_number, state.step, state.active_player_idx, seat).hash(&mut h);
    (state.player_with_priority(), state.stack.len(), state.battlefield.len()).hash(&mut h);
    (state.next_effect_timestamp, state.pending_decision.is_some()).hash(&mut h);
    for p in &state.players {
        (p.life, p.hand.len(), p.library.len(), p.graveyard.len()).hash(&mut h);
    }
    h.finish()
}

/// Pin a bot's randomness for its decision as `seat` in `state`
/// ([`decision_seed`]) until the guard drops. A thread that pinned its own
/// stream for the whole match (the server's seeded bot-vs-bot sweeps) keeps
/// it: those replay from their seed already, and their games stay the ones
/// they were chosen for.
pub fn pin_bot(state: &GameState, seat: usize) -> Option<PinnedStreams> {
    (!super::bot::jitter_pinned()).then(|| PinnedStreams::install(decision_seed(state, seat)))
}

/// Something play showed that a rewind can't take back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Seen {
    /// `seat` drew a card.
    Drew(usize),
    Scried(usize),
    Surveiled(usize),
    Searched(usize),
    /// A card was revealed to the table.
    Revealed,
    CoinFlip,
    DieRoll,
    /// `seat` was shown `of`'s hand by a choice it was asked (Thoughtseize).
    Hand { seat: usize, of: usize },
}

thread_local! {
    /// The match's journal of what play has shown, fed by every broadcast
    /// batch ([`note_seen`]). Per thread, armed for the match by
    /// [`SeenJournal`], like the replay sink: each match runs its loop on
    /// one thread.
    static SEEN: RefCell<Option<Vec<Seen>>> = const { RefCell::new(None) };
}

/// Arms this thread's [`Seen`] journal for one match; dropping it disarms.
pub(crate) struct SeenJournal;

impl SeenJournal {
    pub(crate) fn arm() -> Self {
        SEEN.with(|s| *s.borrow_mut() = Some(Vec::new()));
        Self
    }
}

impl Drop for SeenJournal {
    fn drop(&mut self) {
        SEEN.with(|s| *s.borrow_mut() = None);
    }
}

fn seen_len() -> usize {
    SEEN.with(|s| s.borrow().as_ref().map_or(0, Vec::len))
}

/// Journal what a broadcast batch showed: its `events`, and a choice it
/// left `state` waiting on that lists another player's hand.
pub(crate) fn note_seen(state: &GameState, events: &[GameEventWire]) {
    SEEN.with(|s| {
        let mut s = s.borrow_mut();
        let Some(journal) = s.as_mut() else { return };
        for ev in events {
            match ev {
                GameEventWire::CardDrawn { player, .. } => journal.push(Seen::Drew(*player)),
                GameEventWire::ScryPerformed { player, looked_at, .. } if *looked_at > 0 => {
                    journal.push(Seen::Scried(*player))
                }
                GameEventWire::SurveilPerformed { player, looked_at, .. } if *looked_at > 0 => {
                    journal.push(Seen::Surveiled(*player))
                }
                GameEventWire::PlayerSearchedLibrary { player } => journal.push(Seen::Searched(*player)),
                GameEventWire::TopCardRevealed { .. } => journal.push(Seen::Revealed),
                GameEventWire::CoinFlipWon { .. } | GameEventWire::CoinFlipLost { .. } => {
                    journal.push(Seen::CoinFlip)
                }
                GameEventWire::DiceRolled { count, .. } => {
                    journal.extend(std::iter::repeat_n(Seen::DieRoll, (*count).max(1) as usize))
                }
                _ => {}
            }
        }
        if let Some(pending) = &state.pending_decision {
            use crate::decision::Decision;
            let cards = match &pending.decision {
                Decision::Discard { hand, .. } => hand,
                Decision::ChooseCards { candidates, .. } => candidates,
                _ => return,
            };
            let seat = pending.acting_player();
            for (of, p) in state.players.iter().enumerate() {
                if of != seat && p.hand.iter().any(|c| cards.iter().any(|(id, _)| *id == c.id)) {
                    journal.push(Seen::Hand { seat, of });
                }
            }
        }
    });
}

/// What `point`'s seat saw between keeping it and `now` that rewinding to
/// `before` (its state) can't take back, in words: "2 draws", "a coin
/// flip", "Bot's hand".
pub fn seen_since(point: &UndoPoint, before: &GameState, now: &GameState) -> Vec<String> {
    let seat = point.seat;
    let seen: Vec<Seen> = SEEN.with(|s| {
        s.borrow().as_ref().map_or_else(Vec::new, |journal| journal.get(point.seen_at..).unwrap_or_default().to_vec())
    });
    let count = |f: &dyn Fn(&Seen) -> bool| seen.iter().filter(|x| f(x)).count();
    let mut out = Vec::new();
    for (n, one, many) in [
        (count(&|x| *x == Seen::Drew(seat)), "1 draw", "draws"),
        (count(&|x| *x == Seen::Scried(seat)), "a scry", "scries"),
        (count(&|x| *x == Seen::Surveiled(seat)), "a surveil", "surveils"),
        (count(&|x| *x == Seen::Searched(seat)), "a library search", "library searches"),
        (count(&|x| *x == Seen::Revealed), "a revealed card", "revealed cards"),
        (count(&|x| *x == Seen::CoinFlip), "a coin flip", "coin flips"),
        (count(&|x| *x == Seen::DieRoll), "a die roll", "die rolls"),
    ] {
        match n {
            0 => {}
            1 => out.push(one.to_string()),
            n => out.push(format!("{n} {many}")),
        }
    }
    // A hand: shown by a choice, or looked at for good (`hands_revealed_to`).
    let mut hands: Vec<usize> = seen
        .iter()
        .filter_map(|x| match x {
            Seen::Hand { seat: s, of } if *s == seat => Some(*of),
            _ => None,
        })
        .chain(
            now.hands_revealed_to
                .iter()
                .filter(|pair| pair.0 == seat && !before.hands_revealed_to.contains(pair))
                .map(|pair| pair.1),
        )
        .collect();
    hands.sort_unstable();
    hands.dedup();
    for of in hands {
        let name = now.players.get(of).map_or_else(|| format!("P{of}"), |p| p.name.clone());
        out.push(format!("{name}'s hand"));
    }
    out
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

    /// Undo and redo flips the same coin: the flip comes off the game's own
    /// stream, which the rewind puts back. Across matches it still lands
    /// both ways.
    #[test]
    fn a_redo_flips_the_same_coin() {
        use crate::game::GameEvent;
        let mut heads_seen = Vec::new();
        for seed in 0..16u64 {
            let mut g = two_player_game();
            g.rng.reseed(seed);
            let stitch = g.add_card_to_hand(0, catalog::stitch_in_time());
            g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
            g.players[0].mana_pool.add(crate::mana::Color::Red, 2);
            let before = g.clone();
            let flip = |g: &mut GameState| {
                crate::game::cast(g, stitch).iter().any(|e| matches!(e, GameEvent::CoinFlipWon { .. }))
            };
            let heads = flip(&mut g);
            g.rewind_to(before);
            assert_eq!(flip(&mut g), heads, "seed {seed}: the redo flips the same coin");
            heads_seen.push(heads);
        }
        assert!(heads_seen.contains(&true) && heads_seen.contains(&false), "{heads_seen:?}");
    }

    /// A bot decision's seed is a function of the position: a clone (a
    /// rewound state) gets the same one, a move on the board another, and
    /// each seat its own. A thread that pinned its own stream keeps it.
    #[test]
    fn a_bot_decision_is_seeded_by_its_position() {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        let seed = decision_seed(&g, 1);
        assert_eq!(decision_seed(&g.clone(), 1), seed);
        assert_ne!(decision_seed(&g, 0), seed);
        let forest = g.add_card_to_hand(0, catalog::forest());
        g.perform_action(GameAction::PlayLand(forest)).expect("play the Forest");
        assert_ne!(decision_seed(&g, 1), seed);

        assert!(pin_bot(&g, 1).is_some());
        crate::server::bot::set_jitter_seed(Some(9));
        assert!(pin_bot(&g, 1).is_none(), "a pinned thread keeps its stream");
        crate::server::bot::set_jitter_seed(None);
    }

    /// A take-back names what the undone stretch showed its seat: its own
    /// draws and looks, every coin and die, and a hand a choice laid open
    /// (Thoughtseize) — not the opponent's draws. The stretch is then
    /// forgotten.
    #[test]
    fn a_take_back_names_what_it_showed() {
        let _journal = SeenJournal::arm();
        let mut g = two_player_game();
        g.players[0].wants_ui = true;
        g.add_card_to_hand(1, catalog::grizzly_bears());
        let seize = g.add_card_to_hand(0, catalog::thoughtseize());
        g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
        let mut h = UndoHistory::default();
        h.record(0, "cast Thoughtseize".into(), g.clone());
        note_seen(&g, &[GameEventWire::CardDrawn { player: 0, card_id: crate::card::CardId(90) }]);
        g.perform_action(GameAction::CastSpell {
            card_id: seize,
            target: Some(crate::game::Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast Thoughtseize");
        crate::game::drain_stack(&mut g);
        assert!(g.pending_decision.is_some(), "the caster picks from the hand it sees");
        note_seen(
            &g,
            &[
                GameEventWire::CardDrawn { player: 0, card_id: crate::card::CardId(91) },
                GameEventWire::CardDrawn { player: 1, card_id: crate::card::CardId(92) },
                GameEventWire::CoinFlipLost { player: 1 },
                GameEventWire::ScryPerformed { player: 0, looked_at: 2, bottomed: 1 },
            ],
        );
        let (point, before) = h.peek(0, None).unwrap();
        assert_eq!(seen_since(point, before, &g), ["2 draws", "a scry", "a coin flip", "P1's hand"]);
        let (point, before) = h.take(0, None).unwrap();
        assert!(seen_since(&point, &before, &before).is_empty(), "taken back, the stretch is forgotten");
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
