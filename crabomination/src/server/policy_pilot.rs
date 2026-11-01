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
//! # Candidate states
//!
//! Every candidate is scored from one determinized copy of the position
//! per decision (hidden zones redealt — no peeking at the next card), a
//! mulligan as the hand put back, a block settled through combat damage;
//! see [`candidate_state`]. The recorder builds its rows the same way, so
//! training and play read the same states.
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

/// The state a decision's candidates are scored from: `g` with every
/// hidden zone redealt (both libraries, the opponent's hand — the search's
/// `determinize_hidden`) and the engine's random stream reseeded. One base
/// per decision, shared by its candidates, so they differ by the move and
/// not by the sample. Without it a successor read the real next card: the
/// state after "take a mulligan" showed the next seven, after "may draw"
/// the card drawn.
fn hidden_safe(g: &GameState, seat: usize, salt: u64) -> GameState {
    let mut h = g.clone();
    super::bot::determinize_hidden(&mut h, seat, salt);
    h.rng.reseed(salt ^ 0x9E37_79B9_7F4A_7C15);
    h
}

/// A candidate's successor, from a [`hidden_safe`] base: the engine state
/// after the move, with two exceptions.
///
/// * **A mulligan** is the base with the hand put back in the library — a
///   fixed stand-in, so the head learns "is this hand worth keeping" rather
///   than scoring one random redraw.
/// * **A block** is settled through combat damage with both players
///   passing: the block's worth is who dies, and at declaration nothing
///   has. No choice of the seat's is made in the settle (prompts it raises
///   take the engine's default answer), so the net still decides.
/// * **A cast or an activation** in a priority window is settled until the
///   item it put on the stack resolves, the opponent passing: a spell's
///   worth is what it does, and on the stack it has done nothing (a 25 %
///   imitation pilot acted a quarter of the times its teacher passed). The
///   settle stops at any prompt the resolution raises, so the seat's own
///   choices stay the head's.
/// * **An attack** is settled the same way after the defender's blocks,
///   which come from the unscored heuristic — the opponent model the
///   search's playouts use — because an attack's worth is how it is
///   blocked, and at declaration nobody has.
fn candidate_state(base: &GameState, seat: usize, kind: Kind, a: &GameAction) -> Option<GameState> {
    use crate::decision::DecisionAnswer;
    if kind == Kind::Prompt("Mulligan")
        && matches!(a, GameAction::SubmitDecision(DecisionAnswer::TakeMulligan | DecisionAnswer::SerumPowder(_)))
    {
        let mut h = base.clone();
        while !h.players[seat].hand.is_empty() {
            let c = h.players[seat].hand.remove(0);
            h.players[seat].library.push(c);
        }
        return Some(h);
    }
    let mut next = base.clone();
    let depth = next.stack.len();
    next.perform_action(a.clone()).ok()?;
    if kind == Kind::Priority && next.stack.len() > depth {
        settle_stack(&mut next, depth);
    }
    if kind == Kind::Block {
        settle_combat(&mut next);
    }
    // Every declaration, the empty one too, so "no attack" is compared at
    // the same point of the turn as the attacks are.
    if kind == Kind::Attack && matches!(a, GameAction::DeclareAttackers(_) | GameAction::PassPriority) {
        settle_attack(&mut next, seat);
    }
    Some(next)
}

/// Through the defender's block declaration (the unscored heuristic's) and
/// then combat damage.
fn settle_attack(g: &mut GameState, seat: usize) {
    let defender = g.next_alive_seat(seat);
    let mut opp = super::bot::HeuristicBot::uniform_baseline();
    opp.note_external_declaration(g, true);
    let mut fuel = 64;
    while fuel > 0 && !g.is_game_over() && g.step == TurnStep::DeclareAttackers {
        fuel -= 1;
        if g.perform_action(GameAction::PassPriority).is_err() {
            return;
        }
    }
    if g.step == TurnStep::DeclareBlockers && !g.blockers_declared() {
        let blocks = opp.next_action(g, defender);
        let declared = blocks.is_some_and(|b| matches!(b, GameAction::DeclareBlockers(_)) && g.perform_action(b).is_ok());
        if !declared {
            let _ = g.perform_action(GameAction::DeclareBlockers(Vec::new()));
        }
    }
    settle_combat(g);
}

/// Pass until the stack is back to `depth` items — what was put on it has
/// resolved — stopping at any prompt, or at game over.
fn settle_stack(g: &mut GameState, depth: usize) {
    let mut fuel = 64;
    while fuel > 0 && !g.is_game_over() && g.pending_decision.is_none() && g.stack.len() > depth {
        fuel -= 1;
        if g.perform_action(GameAction::PassPriority).is_err() {
            return;
        }
    }
}

/// Pass (and default-answer) through the rest of combat damage.
fn settle_combat(g: &mut GameState) {
    use crate::decision::{AutoDecider, Decider};
    let mut fuel = 64;
    while fuel > 0 && !g.is_game_over() {
        fuel -= 1;
        if let Some(p) = &g.pending_decision {
            let ans = AutoDecider.decide(&p.decision);
            if g.perform_action(GameAction::SubmitDecision(ans)).is_err() {
                return;
            }
            continue;
        }
        if !matches!(g.step, TurnStep::DeclareBlockers | TurnStep::FirstStrikeDamage | TurnStep::CombatDamage) {
            return;
        }
        if g.perform_action(GameAction::PassPriority).is_err() {
            return;
        }
    }
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
    /// The ablation control (`CRAB_POLICY_DELEGATE=Mulligan,block,...`,
    /// kind names as `Kind::name`): these kinds go to the heuristic instead
    /// of the head — how much each kind costs the pilot, read off the
    /// ladder. Empty by default.
    delegate: Vec<String>,
    fallback: super::bot::HeuristicBot,
    /// Score each decision from a determinized copy (`hidden_safe`). Off
    /// inside a search rollout, whose state the search has already redealt.
    redeal: bool,
}

impl PolicyBot {
    pub fn new(weights: EvalWeights, temp: f32, seed: u64) -> Self {
        assert!(
            super::net_eval::slot_has_policy(weights.net_slot),
            "the policy pilot needs a net with a policy head in slot {}",
            weights.net_slot
        );
        let delegate = std::env::var("CRAB_POLICY_DELEGATE")
            .map(|v| v.split(',').map(|k| k.trim().to_string()).filter(|k| !k.is_empty()).collect())
            .unwrap_or_default();
        Self {
            weights,
            temp,
            rng: StdRng::seed_from_u64(seed),
            declared: None,
            decisions: 0,
            forced: 0,
            delegate,
            fallback: super::bot::HeuristicBot::with_weights(weights),
            redeal: true,
        }
    }

    /// The pilot as a search rollout's policy: no per-decision redeal (the
    /// rollout's state is already determinized), no delegation.
    pub fn for_rollout(weights: EvalWeights, temp: f32, seed: u64) -> Self {
        Self { redeal: false, delegate: Vec::new(), ..Self::new(weights, temp, seed) }
    }

    fn base(&mut self, g: &GameState, seat: usize) -> GameState {
        use rand::RngExt;
        if self.redeal { hidden_safe(g, seat, self.rng.random()) } else { g.clone() }
    }

    fn decide(&mut self, g: &GameState, seat: usize) -> Option<GameAction> {
        let mut m = decision_menu::menu(g, seat, &self.weights)?;
        if !self.delegate.is_empty() && self.delegate.iter().any(|k| k == m.kind.name()) {
            return self.fallback.next_action(g, seat);
        }
        let latched = self.declared == Some((g.turn_number, g.step));
        if latched && matches!(m.kind, Kind::Attack | Kind::Block) {
            m.moves.retain(|a| !matches!(a, GameAction::DeclareAttackers(_) | GameAction::DeclareBlockers(_)));
            m.kind = Kind::Priority;
        }
        self.decisions += 1;
        if matches!(m.kind, Kind::Attack | Kind::Block) {
            self.declared = Some((g.turn_number, g.step));
        }
        self.choose_from(g, seat, m)
    }

    /// The pilot's pick from the list `m` (built for `seat` in `g`), with
    /// no latch bookkeeping — what [`agreement_game`] asks at a decision
    /// another pilot is making.
    pub fn choose_from(&mut self, g: &GameState, seat: usize, m: decision_menu::Menu) -> Option<GameAction> {
        if matches!(m.kind, Kind::Attack | Kind::Block) {
            return self.build_declaration(g, seat, m.kind).or_else(|| m.moves.first().cloned());
        }
        if m.moves.len() <= 1 {
            self.forced += 1;
            return m.moves.into_iter().next();
        }
        let base = self.base(g, seat);
        let kind = m.kind;
        let (moves, states): (Vec<GameAction>, Vec<GameState>) =
            m.moves.into_iter().filter_map(|a| candidate_state(&base, seat, kind, &a).map(|s| (a, s))).unzip();
        if moves.is_empty() {
            return None;
        }
        let scores = policy_scores(&states, seat, self.weights.net_slot, m.kind)?;
        Some(moves[pick(&scores, self.temp, &mut self.rng)].clone())
    }

    /// Build an attack or block declaration one member at a time.
    fn build_declaration(&mut self, g: &GameState, seat: usize, kind: Kind) -> Option<GameAction> {
        let base = self.base(g, seat);
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
                    candidate_state(&base, seat, kind, &declaration(kind, &s))
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

// ── The search's arms (phase 4) ────────────────────────────────────────

/// The search's arms at a menu decision: up to `k` moves from `m`, ranked
/// best-first by the policy head in `slot` over their candidate states
/// (the same states the policy pilot scores), with their logits. A
/// declaration list is ranked as whole sets — every set on the list scored
/// once, settled. With no policy (`slot` 0 or a headless net) the arms are
/// the list's first `k` at equal logits.
pub fn ranked_arms(g: &GameState, seat: usize, m: &decision_menu::Menu, slot: u8, k: usize) -> (Vec<GameAction>, Vec<f64>) {
    let k = k.max(1);
    if slot == 0 || !super::net_eval::slot_has_policy(slot) {
        let arms: Vec<GameAction> = m.moves.iter().take(k).cloned().collect();
        let n = arms.len();
        return (arms, vec![0.0; n]);
    }
    let mut scored = scored_moves(g, seat, m, slot);
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(k);
    scored.into_iter().map(|(a, l, _)| (a, l)).unzip()
}

/// Every move on `m` the policy in `slot` can score, in list order, with
/// its logit and the candidate state the logit was read from — the
/// unranked form of [`ranked_arms`], for a caller that samples its arms
/// or records the states (phase 5's search targets). Empty with no policy.
pub fn scored_moves(g: &GameState, seat: usize, m: &decision_menu::Menu, slot: u8) -> Vec<(GameAction, f64, GameState)> {
    if slot == 0 || !super::net_eval::slot_has_policy(slot) {
        return Vec::new();
    }
    let base = hidden_safe(g, seat, (g.turn_number as u64) << 20 ^ (seat as u64) << 8 ^ g.step as u64);
    m.moves
        .iter()
        .filter_map(|a| {
            let st = candidate_state(&base, seat, m.kind, a)?;
            let l = super::net_eval::policy_logit_kind(&st, seat, slot, m.kind.index())?;
            Some((a.clone(), l as f64, st))
        })
        .collect()
}

// ── Agreement with a teacher ───────────────────────────────────────────

/// One decision of a teacher's game, against the policy pilot's pick.
#[derive(Debug, Clone)]
pub struct AgreeRow {
    pub kind: Kind,
    pub moves: usize,
    /// The teacher passed (or declared nothing).
    pub teacher_passed: bool,
    pub policy_passed: bool,
    pub agree: bool,
}

/// Play a game with `teachers` on both seats and ask `policy`, at every
/// decision with two or more moves, what it would have played — the
/// imitation run's on-policy read, split the way its failures are: by kind,
/// and by whether the teacher acted or passed.
pub fn agreement_game(
    template: &GameState,
    teachers: &mut [Box<dyn Bot>; 2],
    policy: &mut PolicyBot,
    w: &EvalWeights,
    seed: u64,
) -> Vec<AgreeRow> {
    let mut g = super::action_gap::start_game(template, w, seed);
    let mut out = Vec::new();
    let mut declared: [Option<(u32, TurnStep)>; 2] = [None, None];
    let passed = |a: &GameAction| match a {
        GameAction::PassPriority => true,
        GameAction::DeclareAttackers(v) => v.is_empty(),
        GameAction::DeclareBlockers(v) => v.is_empty(),
        _ => false,
    };
    let mut fuel = 6000u32;
    while fuel > 0 && !g.is_game_over() && g.turn_number <= 30 {
        fuel -= 1;
        let mut acted = false;
        for seat in 0..2 {
            let mut m = decision_menu::menu(&g, seat, w);
            if let Some(mm) = m.as_mut()
                && declared[seat] == Some((g.turn_number, g.step))
                && matches!(mm.kind, Kind::Attack | Kind::Block)
            {
                mm.moves.retain(|a| !matches!(a, GameAction::DeclareAttackers(_) | GameAction::DeclareBlockers(_)));
                mm.kind = Kind::Priority;
            }
            let Some(a) = teachers[seat].next_action(&g, seat) else { continue };
            let before = g.clone();
            if g.perform_action(a.clone()).is_err() {
                continue;
            }
            acted = true;
            if let Some(m) = m
                && m.moves.len() >= 2
            {
                let (kind, moves) = (m.kind, m.moves.len());
                if matches!(kind, Kind::Attack | Kind::Block) {
                    declared[seat] = Some((before.turn_number, before.step));
                }
                if let Some(p) = policy.choose_from(&before, seat, m) {
                    let canon = |x: &GameAction| match x {
                        // Passing at a declaration is declaring nothing.
                        GameAction::PassPriority if kind == Kind::Attack => GameAction::DeclareAttackers(Vec::new()),
                        GameAction::PassPriority if kind == Kind::Block => GameAction::DeclareBlockers(Vec::new()),
                        _ => x.clone(),
                    };
                    out.push(AgreeRow {
                        kind,
                        moves,
                        teacher_passed: passed(&a),
                        policy_passed: passed(&p),
                        agree: decision_menu::key(&before, seat, &canon(&a)) == decision_menu::key(&before, seat, &canon(&p)),
                    });
                }
            }
            break;
        }
        if !acted && g.perform_action(GameAction::PassPriority).is_err() {
            break;
        }
    }
    out
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
    let base = hidden_safe(g, seat, rand::RngExt::random(rng));
    let successors: Vec<Option<GameState>> = idx.iter().map(|&i| candidate_state(&base, seat, m.kind, &m.moves[i])).collect();
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
    let base = hidden_safe(g, seat, rand::RngExt::random(rng));
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
                    candidate_state(&base, seat, kind, &declaration(kind, &s))
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

    /// "Take a mulligan" is the hand put back — never the real next seven
    /// — and a block is read after the damage it leads to.
    #[test]
    fn a_mulligan_is_the_hand_put_back_and_a_block_is_read_after_damage() {
        use crate::card::{CardDefinition, CardType};
        use crate::decision::DecisionAnswer;
        let mut g = crate::game::two_player_game();
        for seat in 0..2 {
            for _ in 0..10 {
                g.add_card_to_library(seat, crate::catalog::island());
            }
        }
        for _ in 0..7 {
            g.add_card_to_hand(0, crate::catalog::forest());
        }
        let base = hidden_safe(&g, 0, 9);
        let mull = candidate_state(&base, 0, Kind::Prompt("Mulligan"), &GameAction::SubmitDecision(DecisionAnswer::TakeMulligan))
            .expect("stand-in");
        assert!(mull.players[0].hand.is_empty());
        assert_eq!(mull.players[0].library.len(), base.players[0].library.len() + 7);

        // A 1/1 blocks a 3/3: read after damage, the 1/1 is gone.
        let body = |name: &'static str, p: i32, t: i32| CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            power: p,
            toughness: t,
            ..Default::default()
        };
        let chump = g.add_card_to_battlefield(0, body("Chump", 1, 1));
        let ogre = g.add_card_to_battlefield(1, body("Ogre", 3, 3));
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.clear_sickness(ogre);
        while g.step != TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: ogre, target: AttackTarget::Player(0) }]))
            .expect("attack");
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        let base = hidden_safe(&g, 0, 9);
        let blocked = candidate_state(&base, 0, Kind::Block, &GameAction::DeclareBlockers(vec![(chump, ogre)])).expect("block");
        assert!(blocked.battlefield_find(chump).is_none(), "the chump died in the settle");
        let open = candidate_state(&base, 0, Kind::Block, &GameAction::DeclareBlockers(Vec::new())).expect("no block");
        assert_eq!(open.players[0].life, base.players[0].life - 3, "unblocked, the 3 went through");
    }

    /// A cast is read after it resolves: a Lightning Bolt's successor has the
    /// creature dead and the Bolt in the graveyard, not the Bolt on the stack.
    #[test]
    fn a_cast_is_read_after_it_resolves() {
        use crate::card::{CardDefinition, CardType};
        let mut g = crate::game::two_player_game();
        for seat in 0..2 {
            for _ in 0..5 {
                g.add_card_to_library(seat, crate::catalog::island());
            }
        }
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let m = g.add_card_to_battlefield(0, crate::catalog::mountain());
        g.clear_sickness(m);
        let bolt = g.add_card_to_hand(0, crate::catalog::lightning_bolt());
        let bear = g.add_card_to_battlefield(
            1,
            CardDefinition { name: "Bear", card_types: vec![CardType::Creature], power: 2, toughness: 2, ..Default::default() },
        );
        let cast = GameAction::CastSpell {
            card_id: bolt,
            target: Some(crate::game::Target::Permanent(bear)),
            additional_targets: Vec::new(),
            mode: None,
            x_value: None,
        };
        let base = hidden_safe(&g, 0, 3);
        let after = candidate_state(&base, 0, Kind::Priority, &cast).expect("cast");
        assert!(after.stack.is_empty(), "resolved");
        assert!(after.battlefield_find(bear).is_none(), "the bear died");
        assert_eq!(after.player_with_priority(), 0, "and it's our priority again");
    }

    /// Phase 4's search over every window plays a legal game: every move
    /// it makes, in every window, is one the engine takes.
    #[test]
    fn the_search_over_every_window_plays_a_legal_game() {
        search_plays_a_legal_game(false);
    }

    /// ...and with the policy pilot playing its rollouts.
    #[test]
    fn the_search_with_policy_rollouts_plays_a_legal_game() {
        search_plays_a_legal_game(true);
    }

    /// Phase 5: with capture on, a searched menu decision records the
    /// search's improved policy over its arms — logits, the rollouts behind
    /// each, the decision's kind — and the arms are drawn by Gumbel-top-k.
    #[test]
    fn the_sampled_search_records_its_improved_policy() {
        let _serial = decision_capture::TEST_SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        decision_capture::set_enabled(true);
        decision_capture::set_menu_only(true);
        let _ = decision_capture::drain();
        let before = decision_capture::pushed();
        search_game(false, true, 4);
        let rows = decision_capture::drain();
        decision_capture::set_menu_only(false);
        decision_capture::set_enabled(false);
        assert!(!rows.is_empty(), "no searched decision recorded");
        assert_eq!(decision_capture::pushed() - before, rows.len() as u64);
        for r in &rows {
            let v = r.values.as_ref().expect("a target");
            assert!(r.values_are_logits);
            assert_eq!(v.len(), r.successors.len());
            assert!(r.visits.as_ref().is_some_and(|n| n.len() == v.len() && n.iter().any(|&x| x > 0)));
            assert!(r.chosen < v.len() && v.iter().all(|x| x.is_finite()));
            assert!(r.successors.len() <= 4, "at most menu_arms arms");
        }
        assert!(rows.iter().any(|r| r.kind == Kind::Priority.index()), "a main-phase row");
    }

    fn search_plays_a_legal_game(policy_rollouts: bool) {
        search_game(policy_rollouts, false, if policy_rollouts { 5 } else { 8 });
    }

    fn search_game(policy_rollouts: bool, menu_sample: bool, last_turn: u32) {
        use crate::server::mcts::{MENU_COMBAT, MENU_INSTANT, MENU_MAIN, MENU_PROMPT, MctsBot, MctsConfig};
        let _g = SLOT.lock().unwrap_or_else(|e| e.into_inner());
        let slot = super::super::net_eval::SLOT_FRA;
        super::super::net_eval::set_slot(slot, Some(std::sync::Arc::new(HashPolicy)));
        let cfg = MctsConfig {
            iterations: if policy_rollouts { 3 } else { 6 },
            horizon_turns: 1,
            weights: EvalWeights::default(),
            menu_windows: MENU_MAIN | MENU_INSTANT | MENU_COMBAT | MENU_PROMPT,
            policy_slot: slot,
            menu_arms: 4,
            policy_rollouts,
            menu_sample,
            ..MctsConfig::default()
        };
        let seed = 7u64;
        let mut g = super::super::action_gap::start_game(&sealed_template(seed), &cfg.weights, seed);
        let mut bots = [MctsBot::new(cfg), MctsBot::new(cfg)];
        let (mut fuel, mut stalled, mut declared) = (4000, 0, 0);
        while fuel > 0 && !g.is_game_over() && g.turn_number <= last_turn && stalled < 50 {
            fuel -= 1;
            let mut acted = false;
            for (seat, bot) in bots.iter_mut().enumerate() {
                let Some(a) = bot.next_action(&g, seat) else { continue };
                if matches!(a, GameAction::DeclareAttackers(ref v) if !v.is_empty()) {
                    declared += 1;
                }
                let r = g.perform_action(a.clone());
                assert!(r.is_ok(), "the engine refused the search's {a:?}: {r:?}");
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
        super::super::net_eval::set_slot(slot, None);
        assert!(stalled < 50, "stalled at {:?}", g.step);
        assert!(g.turn_number > 3, "the game advanced");
        let _ = declared;
    }
}
