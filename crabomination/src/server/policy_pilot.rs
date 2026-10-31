//! The policy pilot: every decision made by the net's policy head over the
//! complete move list — phase 2 of "the net makes every decision"
//! (`ML_NOTES` 2026-10-31).
//!
//! A decision is a set of candidate moves from [`decision_menu::menu`], each
//! scored by the policy head over the state it leads to plus the decision's
//! kind row (`PlayNet::forward_policy_kind`). The pilot plays the best (or
//! samples at a temperature). Nothing else is consulted: no heuristic
//! picker, no sims, no search.
//!
//! # Combat, one creature at a time
//!
//! An attack or block declaration is a set, and the sets are exponential
//! in the board (the move list caps them at 256). So a declaration is built
//! by steps: from the set so far, the candidates are "declare it as is"
//! and "add one more" (one more attacker at one target; one more blocker on
//! one attacker, or a pair where only a pair is legal — menace). Every
//! candidate's successor is the real engine state of declaring that set, so
//! the head compares "these three attack" against "these three and that
//! one" — no hypothetical encoding, no new features. A step's kind is the
//! declaration's.
//!
//! # Imitation rows
//!
//! [`record`] turns any pilot's move at a decision into the same rows the
//! policy pilot scores: one row over the move list for a plain decision,
//! and one row per step for a declaration — the pilot's set reached by
//! adding its members one at a time, each step's target every member still
//! to add (a uniform soft target: the order is ours, not the pilot's), the
//! last step's target "stop". Rows go through `decision_capture`'s buffer
//! with their kind, so `selfplay_train` takes them as it takes every other
//! decision row. Candidate lists are sampled down to [`ROW_CANDIDATES`]
//! with the target kept, which the trainer's window then never cuts.

use rand::SeedableRng;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;

use crate::card::CardId;
use crate::game::types::TurnStep;
use crate::game::{Attack, AttackTarget, GameAction, GameState};

use super::bot::{Bot, EvalWeights};
use super::decision_capture::{self, Provenance};
use super::decision_menu::{self, Kind};

/// Most candidates one recorded row carries (the trainer's
/// `POLICY_MAX_CANDIDATES`).
pub const ROW_CANDIDATES: usize = 8;

impl Kind {
    /// The kind's row in the policy head's kind table (below
    /// `crabomination_nn::POLICY_KINDS`). Stable: a trained table is read
    /// by these numbers.
    pub fn index(self) -> u8 {
        match self {
            Kind::Priority => 0,
            Kind::Attack => 1,
            Kind::Block => 2,
            Kind::Prompt(p) => match p {
                "ChooseTarget" => 3,
                "ChooseMode" => 4,
                "ChooseColor" => 5,
                "Scry" => 6,
                "Discard" => 7,
                "SearchLibrary" => 8,
                "OptionalTrigger" => 9,
                "PutOnLibrary" => 10,
                "Mulligan" => 11,
                "ChooseCreatureType" => 12,
                "ChooseCreatureTypePair" => 13,
                "NameCard" => 14,
                "CoinFlip" => 15,
                "DieRoll" => 16,
                "CommanderRedirect" => 17,
                "CombatDamageOrder" => 18,
                "ChooseModes" => 19,
                "Learn" => 20,
                "OrderTriggers" => 21,
                "DivideDamage" => 22,
                "AssignCombatDamage" => 23,
                "ChooseLegendToKeep" => 24,
                "ChooseAmount" => 25,
                "ChooseOption" => 26,
                "ChooseCards" => 27,
                _ => 31,
            },
        }
    }
}

/// One member of a declaration being built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Member {
    Attack(Attack),
    Block(CardId, CardId),
}

fn declaration(kind: Kind, set: &[Member]) -> GameAction {
    if kind == Kind::Attack {
        GameAction::DeclareAttackers(
            set.iter().filter_map(|m| if let Member::Attack(a) = m { Some(*a) } else { None }).collect(),
        )
    } else {
        GameAction::DeclareBlockers(
            set.iter().filter_map(|m| if let Member::Block(b, a) = m { Some((*b, *a)) } else { None }).collect(),
        )
    }
}

/// The engine state after `a`, or `None` if it refuses.
fn successor(g: &GameState, a: &GameAction) -> Option<GameState> {
    let mut next = g.clone();
    next.perform_action(a.clone()).ok().map(|_| next)
}

/// One step of building a declaration from `set`: the additions (each a
/// group of members — one, or a menace pair) whose set the engine accepts,
/// and whether `set` itself may be declared as is.
fn combat_step(g: &GameState, seat: usize, kind: Kind, set: &[Member]) -> (Vec<Vec<Member>>, bool) {
    let ok = |s: &[Member]| g.would_accept(declaration(kind, s));
    let mut adds: Vec<Vec<Member>> = Vec::new();
    if kind == Kind::Attack {
        let used: Vec<CardId> =
            set.iter().filter_map(|m| if let Member::Attack(a) = m { Some(a.attacker) } else { None }).collect();
        let mut targets = vec![AttackTarget::Player(g.next_alive_seat(seat))];
        for c in g.battlefield.iter().filter(|c| c.controller != seat) {
            if c.definition.is_planeswalker() {
                targets.push(AttackTarget::Planeswalker(c.id));
            }
            if c.definition.card_types.contains(&crate::card::CardType::Battle) {
                targets.push(AttackTarget::Battle(c.id));
            }
        }
        for c in g.battlefield.iter().filter(|c| c.controller == seat && c.definition.is_creature() && !used.contains(&c.id)) {
            for &target in &targets {
                let m = Member::Attack(Attack { attacker: c.id, target });
                let mut s = set.to_vec();
                s.push(m);
                if ok(&s) {
                    adds.push(vec![m]);
                }
            }
        }
    } else {
        let used: Vec<CardId> = set.iter().filter_map(|m| if let Member::Block(b, _) = m { Some(*b) } else { None }).collect();
        let mine: Vec<CardId> = g
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && c.definition.is_creature() && !c.tapped && !used.contains(&c.id))
            .map(|c| c.id)
            .collect();
        let attackers: Vec<CardId> = g.attacking().iter().map(|a| a.attacker).collect();
        for (i, &b) in mine.iter().enumerate() {
            for &a in &attackers {
                let mut s = set.to_vec();
                s.push(Member::Block(b, a));
                if ok(&s) {
                    adds.push(vec![Member::Block(b, a)]);
                    continue;
                }
                // Only legal in company (menace): offer the pair, once.
                for &b2 in &mine[i + 1..] {
                    let mut s2 = s.clone();
                    s2.push(Member::Block(b2, a));
                    if ok(&s2) {
                        adds.push(vec![Member::Block(b, a), Member::Block(b2, a)]);
                    }
                }
            }
        }
    }
    let stop = ok(set) || (set.is_empty() && g.would_accept(GameAction::PassPriority));
    (adds, stop)
}

/// Scores successor states for `seat` at decision kind `kind` with the
/// slot's policy head; `None` when the slot has no policy head.
fn policy_scores(states: &[GameState], seat: usize, slot: u8, kind: Kind) -> Option<Vec<f32>> {
    states.iter().map(|s| super::net_eval::policy_logit_kind(s, seat, slot, kind.index())).collect()
}

/// Index of the pick: the best score at `temp` 0, else a softmax sample.
fn pick(scores: &[f32], temp: f32, rng: &mut StdRng) -> usize {
    use rand::RngExt;
    if temp <= 0.0 {
        return (0..scores.len()).max_by(|&a, &b| scores[a].partial_cmp(&scores[b]).unwrap_or(std::cmp::Ordering::Equal)).unwrap_or(0);
    }
    let m = scores.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let w: Vec<f32> = scores.iter().map(|s| ((s - m) / temp).exp()).collect();
    let total: f32 = w.iter().sum();
    let mut r = rng.random::<f32>() * total;
    for (i, x) in w.iter().enumerate() {
        r -= x;
        if r <= 0.0 {
            return i;
        }
    }
    scores.len() - 1
}

/// The policy pilot. Makes every decision with the policy head of the
/// net in `weights.net_slot`; a slot without a policy head is a
/// configuration error the constructor reports.
pub struct PolicyBot {
    weights: EvalWeights,
    /// Softmax temperature over a decision's logits; 0 plays the best.
    pub temp: f32,
    rng: StdRng,
    /// The (turn, step) this pilot last declared in: a declaration —
    /// above all an empty one — leaves the step where it was, and without
    /// the latch the same declaration would be offered again forever.
    declared: Option<(u32, TurnStep)>,
    /// Decisions made, and how many fell to the forced-move shortcut.
    pub decisions: u64,
    pub forced: u64,
}

impl PolicyBot {
    pub fn new(weights: EvalWeights, temp: f32, seed: u64) -> Self {
        assert!(
            super::net_eval::slot_has_policy(weights.net_slot),
            "the policy pilot needs a net with a policy head in slot {}",
            weights.net_slot
        );
        Self { weights, temp, rng: StdRng::seed_from_u64(seed), declared: None, decisions: 0, forced: 0 }
    }

    fn decide(&mut self, g: &GameState, seat: usize) -> Option<GameAction> {
        let mut m = decision_menu::menu(g, seat, &self.weights)?;
        let latched = self.declared == Some((g.turn_number, g.step));
        if latched && matches!(m.kind, Kind::Attack | Kind::Block) {
            m.moves.retain(|a| !matches!(a, GameAction::DeclareAttackers(_) | GameAction::DeclareBlockers(_)));
            m.kind = Kind::Priority;
        }
        self.decisions += 1;
        if matches!(m.kind, Kind::Attack | Kind::Block) {
            let a = self.build_declaration(g, seat, m.kind).or_else(|| m.moves.first().cloned());
            self.declared = Some((g.turn_number, g.step));
            return a;
        }
        if m.moves.len() <= 1 {
            self.forced += 1;
            return m.moves.into_iter().next();
        }
        let (moves, states): (Vec<GameAction>, Vec<GameState>) =
            m.moves.into_iter().filter_map(|a| successor(g, &a).map(|s| (a, s))).unzip();
        if moves.is_empty() {
            return None;
        }
        let scores = policy_scores(&states, seat, self.weights.net_slot, m.kind)?;
        Some(moves[pick(&scores, self.temp, &mut self.rng)].clone())
    }

    /// Build an attack or block declaration one member at a time.
    fn build_declaration(&mut self, g: &GameState, seat: usize, kind: Kind) -> Option<GameAction> {
        let mut set: Vec<Member> = Vec::new();
        loop {
            let (adds, stop) = combat_step(g, seat, kind, &set);
            if adds.is_empty() {
                return stop.then(|| declaration(kind, &set));
            }
            let mut options: Vec<Option<Vec<Member>>> = Vec::with_capacity(adds.len() + 1);
            if stop {
                options.push(None);
            }
            options.extend(adds.into_iter().map(Some));
            let states: Vec<GameState> = options
                .iter()
                .filter_map(|o| {
                    let mut s = set.clone();
                    s.extend(o.iter().flatten().copied());
                    successor(g, &declaration(kind, &s))
                })
                .collect();
            if states.len() != options.len() {
                return stop.then(|| declaration(kind, &set));
            }
            let scores = policy_scores(&states, seat, self.weights.net_slot, kind)?;
            match &options[pick(&scores, self.temp, &mut self.rng)] {
                None => return Some(declaration(kind, &set)),
                Some(add) => set.extend(add.iter().copied()),
            }
        }
    }
}

impl Bot for PolicyBot {
    fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction> {
        self.decide(state, seat)
    }
}

// ── Imitation rows ─────────────────────────────────────────────────────

/// Record `chosen` — any pilot's move at `seat`'s decision in `g` — as the
/// policy rows the policy pilot would score (see the module docs). A no-op
/// unless decision capture is on, when `seat` has nothing to decide, or
/// when `chosen` is not on the move list (0.01 % of the client pilot's
/// moves, census 2026-10-31).
pub fn record(g: &GameState, seat: usize, chosen: &GameAction, w: &EvalWeights, rng: &mut StdRng) {
    if !decision_capture::enabled() {
        return;
    }
    let Some(m) = decision_menu::menu(g, seat, w) else { return };
    if matches!(m.kind, Kind::Attack | Kind::Block)
        && let Some(target) = members_of(chosen)
    {
        record_declaration(g, seat, m.kind, &target, rng);
        return;
    }
    let key = decision_menu::key(g, seat, chosen);
    let Some(ci) = m.moves.iter().position(|a| decision_menu::key(g, seat, a) == key) else { return };
    let mut idx: Vec<usize> = (0..m.moves.len()).filter(|&i| i != ci).collect();
    idx.shuffle(rng);
    idx.truncate(ROW_CANDIDATES - 1);
    idx.push(ci);
    idx.shuffle(rng);
    let chosen_at = idx.iter().position(|&i| i == ci).expect("kept");
    let successors: Vec<Option<GameState>> = idx.iter().map(|&i| successor(g, &m.moves[i])).collect();
    decision_capture::maybe_successors(g, seat, successors, chosen_at, Provenance { kind: m.kind.index(), ..Default::default() });
}

fn members_of(a: &GameAction) -> Option<Vec<Member>> {
    match a {
        GameAction::DeclareAttackers(v) => Some(v.iter().map(|x| Member::Attack(*x)).collect()),
        GameAction::DeclareBlockers(v) => Some(v.iter().map(|(b, a)| Member::Block(*b, *a)).collect()),
        // Passing at a declaration is declaring nothing.
        GameAction::PassPriority => Some(Vec::new()),
        _ => None,
    }
}

/// The declaration `target` as policy steps: from the empty set, each
/// step's candidates are the additions (and "stop" when legal), the target
/// every addition still inside `target` (uniform, as logits 0 vs −30), and
/// "stop" once `target` is reached.
fn record_declaration(g: &GameState, seat: usize, kind: Kind, target: &[Member], rng: &mut StdRng) {
    let mut set: Vec<Member> = Vec::new();
    for _ in 0..=target.len() {
        let (adds, stop) = combat_step(g, seat, kind, &set);
        let done = target.iter().all(|m| set.contains(m));
        let good = |o: &Option<Vec<Member>>| match o {
            None => done,
            Some(add) => add.iter().all(|m| target.contains(m)),
        };
        let mut options: Vec<Option<Vec<Member>>> = Vec::new();
        if stop {
            options.push(None);
        }
        options.extend(adds.into_iter().map(Some));
        let goods: Vec<usize> = (0..options.len()).filter(|&i| good(&options[i])).collect();
        let Some(&first_good) = goods.first() else { return };
        // Keep every good option, fill with the rest up to the cap.
        let mut idx: Vec<usize> = goods.clone();
        let mut rest: Vec<usize> = (0..options.len()).filter(|i| !goods.contains(i)).collect();
        rest.shuffle(rng);
        idx.extend(rest.into_iter().take(ROW_CANDIDATES.saturating_sub(goods.len())));
        idx.truncate(ROW_CANDIDATES.max(1));
        if idx.len() >= 2 {
            let successors: Vec<Option<GameState>> = idx
                .iter()
                .map(|&i| {
                    let mut s = set.clone();
                    s.extend(options[i].iter().flatten().copied());
                    successor(g, &declaration(kind, &s))
                })
                .collect();
            let logits: Vec<f32> = idx.iter().map(|i| if goods.contains(i) { 0.0 } else { -30.0 }).collect();
            let chosen_at = idx.iter().position(|&i| i == first_good).expect("kept");
            decision_capture::maybe_successors(
                g,
                seat,
                successors,
                chosen_at,
                Provenance { kind: kind.index(), values: Some(&logits), values_are_logits: true, ..Default::default() },
            );
        }
        if done {
            return;
        }
        // Advance by the next addition inside the target, in our order.
        match options.into_iter().flatten().find(|add| add.iter().all(|m| target.contains(m))) {
            Some(add) => set.extend(add),
            None => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_indices_are_distinct_and_in_range() {
        let names = [
            "ChooseTarget", "ChooseMode", "ChooseColor", "Scry", "Discard", "SearchLibrary", "OptionalTrigger",
            "PutOnLibrary", "Mulligan", "ChooseCreatureType", "ChooseCreatureTypePair", "NameCard", "CoinFlip",
            "DieRoll", "CommanderRedirect", "CombatDamageOrder", "ChooseModes", "Learn", "OrderTriggers",
            "DivideDamage", "AssignCombatDamage", "ChooseLegendToKeep", "ChooseAmount", "ChooseOption", "ChooseCards",
        ];
        let mut seen: Vec<u8> = vec![Kind::Priority.index(), Kind::Attack.index(), Kind::Block.index()];
        seen.extend(names.iter().map(|n| Kind::Prompt(n).index()));
        let mut sorted = seen.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), seen.len(), "two kinds share a row");
        assert!(seen.iter().all(|&k| (k as usize) < crabomination_nn::POLICY_KINDS));
    }

    /// An arbitrary but deterministic policy: a hash of the encoded state,
    /// shifted by the kind — enough to drive every code path without a
    /// trained net.
    struct HashPolicy;
    impl crabomination_nn::NetEvaluator for HashPolicy {
        fn eval(&self, _s: crabomination_nn::EncodedState) -> f32 {
            0.5
        }
        fn eval_policy(&self, s: crabomination_nn::EncodedState) -> Option<f32> {
            self.eval_policy_kind(s, 0)
        }
        fn eval_policy_kind(&self, s: crabomination_nn::EncodedState, kind: u8) -> Option<f32> {
            let g: f32 = s.global.iter().enumerate().map(|(i, x)| x * (i as f32 + 1.3)).sum();
            let o: f32 = s.objects().iter().map(|o| o.card as f32 * 0.37 + o.feats[0]).sum();
            Some((g * 0.71 + o * 0.13 + kind as f32).sin())
        }
        fn has_policy(&self) -> bool {
            true
        }
    }

    /// The net slots are process-global; these tests use the FRA slot,
    /// which no other test installs into, and serialize among themselves.
    static SLOT: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn sealed_template(seed: u64) -> GameState {
        let build = |k: u64| crate::selfplay::heuristic_sealed_build(&crate::selfplay::sealed_pool(seed ^ k), seed ^ (k << 4));
        crate::selfplay::sealed_game_template(&build(1), &build(2))
    }

    /// The pilot plays whole sealed games on its own — every decision from
    /// the move list, combat built a member at a time — and the engine
    /// accepts every move it makes.
    #[test]
    fn the_policy_pilot_plays_whole_games_on_its_own() {
        let _g = SLOT.lock().unwrap_or_else(|e| e.into_inner());
        let slot = super::super::net_eval::SLOT_FRA;
        super::super::net_eval::set_slot(slot, Some(std::sync::Arc::new(HashPolicy)));
        let w = EvalWeights { net_slot: slot, ..EvalWeights::default() };
        let mut combat = 0u64;
        let mut finished = 0;
        for seed in [3u64] {
            let mut g = super::super::action_gap::start_game(&sealed_template(seed), &w, seed);
            let mut bots = [PolicyBot::new(w, 0.0, seed), PolicyBot::new(w, 0.5, seed + 1)];
            let mut stalled = 0;
            let mut fuel = 8000;
            while fuel > 0 && !g.is_game_over() && g.turn_number <= 16 && stalled < 50 {
                fuel -= 1;
                let mut acted = false;
                for (seat, bot) in bots.iter_mut().enumerate() {
                    let Some(a) = bot.next_action(&g, seat) else { continue };
                    if matches!(a, GameAction::DeclareAttackers(ref v) if !v.is_empty())
                        || matches!(a, GameAction::DeclareBlockers(ref v) if !v.is_empty())
                    {
                        combat += 1;
                    }
                    let r = g.perform_action(a.clone());
                    assert!(r.is_ok(), "seed {seed}: the engine refused the pilot's {a:?}: {r:?}");
                    acted = true;
                    break;
                }
                if acted {
                    stalled = 0;
                } else {
                    stalled += 1;
                    let _ = g.perform_action(GameAction::PassPriority);
                }
            }
            assert!(stalled < 50, "seed {seed}: no seat had a move for 50 ticks at {:?}", g.step);
            assert!(g.is_game_over() || g.turn_number > 16, "seed {seed}: ran out of fuel on turn {}", g.turn_number);
            finished += 1;
            assert!(bots.iter().map(|b| b.decisions).sum::<u64>() > 100, "seed {seed}: too few decisions");
        }
        super::super::net_eval::set_slot(slot, None);
        assert_eq!(finished, 1);
        assert!(combat > 0, "never declared a combat");
    }

    /// The recorder turns a heuristic game into kind-tagged rows: plain
    /// decisions over the move list with a one-hot pick, and combat steps
    /// with a soft target over the members still to add.
    #[test]
    fn the_recorder_turns_a_game_into_kind_tagged_rows() {
        let _serial = decision_capture::TEST_SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        decision_capture::set_enabled(true);
        decision_capture::set_menu_only(true);
        let _ = decision_capture::drain();
        let vocab = super::super::net_eval::vocab();
        let _ = crate::selfplay::play_recorded_game(&sealed_template(5), [EvalWeights::default(); 2], 5, 300, vocab);
        let rows = decision_capture::drain();
        decision_capture::set_menu_only(false);
        decision_capture::set_enabled(false);
        assert!(rows.len() > 20, "only {} rows", rows.len());
        for r in &rows {
            assert!(r.successors.len() >= 2 && r.successors.len() <= ROW_CANDIDATES);
            assert!(r.chosen < r.successors.len());
        }
        let kinds = |k: Kind| rows.iter().filter(|r| r.kind == k.index()).count();
        assert!(kinds(Kind::Priority) > 0 && kinds(Kind::Attack) > 0, "kinds: {:?}", rows.iter().map(|r| r.kind).collect::<Vec<_>>());
        let steps: Vec<_> = rows.iter().filter(|r| r.kind == Kind::Attack.index() || r.kind == Kind::Block.index()).collect();
        assert!(steps.iter().all(|r| r.values_are_logits && r.values.as_ref().is_some_and(|v| v[r.chosen] == 0.0)));
        assert!(rows.iter().filter(|r| r.kind == Kind::Priority.index()).all(|r| r.values.is_none()));
    }
}
