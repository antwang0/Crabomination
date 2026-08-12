//! Puzzle benchmark: fixed positions with machine-certified answers.
//!
//! Why this exists. Until now a bot change was measured two ways, and
//! neither one localises anything. The paired ladder costs minutes of
//! wall clock to produce a win rate with a ±2-point interval and says
//! nothing about *which* decisions moved it — round 31 spent a whole
//! round establishing "combat declarations need the sims' precision",
//! which a combat puzzle set would have shown in seconds. Holdout AUC
//! measures prediction on positions nobody chose, which is a different
//! question from whether the bot plays them well.
//!
//! A puzzle is a fixed position, a seat, and a goal. Scoring is
//! deterministic (no sampling interval), fast (no games to simulate),
//! and diagnostic (tagged by mechanic and difficulty).
//!
//! # The uniqueness problem, and why it mostly dissolves
//!
//! The obvious objection to puzzle benchmarks is that most Magic
//! positions have no single objectively best play, so the answer key is
//! a matter of opinion. That is true of *move*-checked puzzles. It is
//! not true of **outcome**-checked ones: "the opponent is at 5, win this
//! turn" has an unambiguous pass condition that every winning line
//! satisfies, however it gets there. There is nothing to disagree about
//! because no specific move is ever asserted.
//!
//! Better still, the answer key is *derived*, not authored.
//! [`solve`] brute-forces the position with the engine as its legality
//! oracle, which certifies three things at authoring time: that a
//! solution exists at all, that the greedy line does not stumble into
//! it, and the minimum search depth that finds it. That last number is
//! an objective difficulty tier rather than an author's guess.
//!
//! # The action space
//!
//! [`legal_actions`] enumerates candidates and then validates every one
//! through [`GameState::would_accept`]. Nothing here re-implements an
//! engine rule: eligibility to attack, block legality, sorcery timing,
//! targeting restrictions, and mana payment are all decided by actually
//! attempting the action on a clone. That matters for certification —
//! a solver built on the *bot's* candidate enumeration could only ever
//! find lines the bot already considers, so it could never certify a
//! puzzle whose whole point is that the bot's enumeration misses the
//! answer.

use crate::game::types::TurnStep;
use crate::card::CardId;
use crate::game::{Attack, AttackTarget, GameAction, GameState, Target};

use super::bot::{Bot, HeuristicBot};

/// Combinatorial guards. A puzzle position is small by construction, but
/// `legal_actions` is called at every node of a depth-first search and
/// combat subsets are exponential, so both are capped rather than left
/// to the position's good manners. A cap that binds is reported by
/// [`Enumeration::truncated`] rather than silently shrinking the space —
/// a benchmark that quietly stops looking reads as "no solution exists",
/// which is exactly the wrong answer to record.
const MAX_COMBAT_CREATURES: usize = 8;
/// Ceiling on actions returned from one node.
const MAX_ACTIONS: usize = 256;

/// What a puzzle asks the seat to achieve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Goal {
    /// End the game this turn with `seat` as the winner. The canonical
    /// "find lethal" puzzle: any line that gets there passes.
    WinThisTurn,
    /// Reach the next turn alive. The defensive mirror — blocks, removal,
    /// and tricks that stop a lethal swing.
    SurviveTurn,
    /// Finish the turn with the opponent's board empty of creatures.
    ClearOpposingCreatures,
    /// Finish the turn having taken no damage at all.
    TakeNoDamage,
}

/// A goal's verdict on a position, evaluated after a line has been played
/// out to the end of the acting seat's window.
fn goal_met(before: &GameState, after: &GameState, seat: usize, goal: Goal) -> bool {
    let opp = 1 - seat;
    match goal {
        Goal::WinThisTurn => after.game_over == Some(Some(seat)),
        // A loss is a loss however it arrives; "survived" also requires the
        // game not to have ended in a draw underneath us.
        Goal::SurviveTurn => after.game_over.is_none() && after.players[seat].is_alive(),
        Goal::ClearOpposingCreatures => {
            after.game_over != Some(Some(opp))
                && !after
                    .battlefield
                    .iter()
                    .any(|c| c.controller == opp && c.definition.is_creature())
        }
        Goal::TakeNoDamage => {
            after.game_over.is_none() && after.players[seat].life >= before.players[seat].life
        }
    }
}

/// The result of enumerating one node's action space.
pub struct Enumeration {
    pub actions: Vec<GameAction>,
    /// Set when a cap bound the enumeration, so a caller can tell "no
    /// solution in this space" apart from "we stopped looking".
    pub truncated: bool,
}

/// Every action `seat` can legally take in `state` right now, each one
/// verified by an engine dry-run.
///
/// Deliberately *not* the bot's candidate list: this is the certification
/// oracle, so it has to be able to find plays the bot never considers.
pub fn legal_actions(state: &GameState, seat: usize) -> Enumeration {
    let mut out: Vec<GameAction> = Vec::new();
    let mut truncated = false;

    if state.is_game_over() {
        return Enumeration { actions: out, truncated };
    }

    // A pending decision is answered, not chosen among: the engine routes
    // it through `submit_decision` and `would_accept` explicitly cannot
    // probe it (see `affordances.rs`). Puzzles that turn on a decision
    // policy are a separate class and are not enumerated here.
    if state.pending_decision.is_some() {
        return Enumeration { actions: out, truncated };
    }

    let accepts = |a: &GameAction| state.would_accept(a.clone());

    // ── Combat declarations ────────────────────────────────────────────
    // Both are enumerated as *subsets*, and each full subset is validated
    // by the engine, which is what enforces "must attack if able",
    // Propaganda-style costs, menace, and every other constraint that a
    // per-creature filter would get wrong.
    if state.step == TurnStep::DeclareAttackers
        && state.attack_declarer() == seat
        && state.attacking().is_empty()
    {
        let target = AttackTarget::Player(state.next_alive_seat(seat));
        // Individually-legal attackers first: cheap, and it shrinks the
        // subset base from "my whole board" to "creatures that can swing".
        let eligible: Vec<CardId> = state
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && c.definition.is_creature())
            .map(|c| c.id)
            .filter(|&id| {
                accepts(&GameAction::DeclareAttackers(vec![Attack { attacker: id, target }]))
            })
            .collect();
        let (eligible, cut) = cap(eligible, MAX_COMBAT_CREATURES);
        truncated |= cut;
        for mask in 0u32..(1u32 << eligible.len()) {
            let atks: Vec<Attack> = eligible
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, &attacker)| Attack { attacker, target })
                .collect();
            let a = GameAction::DeclareAttackers(atks);
            if accepts(&a) {
                out.push(a);
            }
        }
    }

    if state.step == TurnStep::DeclareBlockers
        && state.may_declare_blocks(seat)
        && !state.attacking().is_empty()
        && !state.blockers_declared()
    {
        // For each of my untapped creatures, which attackers can it
        // legally block? Answered by dry-running the single pair, so
        // flying / menace / protection / "can't be blocked by" all come
        // from the engine.
        let mine: Vec<CardId> = state
            .battlefield
            .iter()
            .filter(|c| c.controller == seat && c.definition.is_creature() && !c.tapped)
            .map(|c| c.id)
            .collect();
        let (mine, cut) = cap(mine, MAX_COMBAT_CREATURES);
        truncated |= cut;
        let attackers: Vec<CardId> = state.attacking().iter().map(|a| a.attacker).collect();
        let options: Vec<Vec<Option<CardId>>> = mine
            .iter()
            .map(|&b| {
                let mut v: Vec<Option<CardId>> = vec![None];
                v.extend(
                    attackers
                        .iter()
                        .filter(|&&a| accepts(&GameAction::DeclareBlockers(vec![(b, a)])))
                        .map(|&a| Some(a)),
                );
                v
            })
            .collect();
        // Cartesian product over "which attacker does each blocker take,
        // or none". Bounded by MAX_ACTIONS so a wide board can't explode.
        let mut assignments: Vec<Vec<(CardId, CardId)>> = vec![Vec::new()];
        for (bi, opts) in options.iter().enumerate() {
            let mut next = Vec::new();
            for base in &assignments {
                for opt in opts {
                    let mut a = base.clone();
                    if let Some(atk) = opt {
                        a.push((mine[bi], *atk));
                    }
                    next.push(a);
                }
            }
            if next.len() > MAX_ACTIONS {
                next.truncate(MAX_ACTIONS);
                truncated = true;
            }
            assignments = next;
        }
        for blocks in assignments {
            let a = GameAction::DeclareBlockers(blocks);
            if accepts(&a) {
                out.push(a);
            }
        }
    }

    // ── Priority actions ───────────────────────────────────────────────
    if state.player_with_priority() == seat {
        // Lands: one per hand card, engine-gated (sorcery timing, the
        // per-turn limit, Exploration-style extra drops).
        for id in hand_ids(state, seat) {
            let a = GameAction::PlayLand(id);
            if accepts(&a) {
                out.push(a);
            }
        }

        // Casts: untargeted first, then one candidate per legal target.
        for id in hand_ids(state, seat) {
            let untargeted = GameAction::CastSpell {
                card_id: id,
                target: None,
                additional_targets: Vec::new(),
                x_value: None,
                mode: None,
            };
            if accepts(&untargeted) {
                out.push(untargeted);
            }
            for t in cast_targets(state, seat, id) {
                let a = GameAction::CastSpell {
                    card_id: id,
                    target: Some(t),
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                if accepts(&a) {
                    out.push(a);
                }
            }
        }

        // Activated abilities of permanents we control, including mana
        // abilities — a puzzle can turn on floating the right colour.
        for c in state.battlefield.iter().filter(|c| c.controller == seat) {
            for idx in 0..c.definition.activated_abilities.len() {
                let untargeted = GameAction::ActivateAbility {
                    card_id: c.id,
                    ability_index: idx,
                    target: None,
                    additional_targets: Vec::new(),
                    x_value: None,
                    mode: None,
                };
                if accepts(&untargeted) {
                    out.push(untargeted);
                }
                for t in ability_targets(state, seat, c.id, idx) {
                    let a = GameAction::ActivateAbility {
                        card_id: c.id,
                        ability_index: idx,
                        target: Some(t),
                        additional_targets: Vec::new(),
                        x_value: None,
                        mode: None,
                    };
                    if accepts(&a) {
                        out.push(a);
                    }
                }
            }
        }

        out.push(GameAction::PassPriority);
    }

    if out.len() > MAX_ACTIONS {
        out.truncate(MAX_ACTIONS);
        truncated = true;
    }
    Enumeration { actions: out, truncated }
}

fn cap<T>(mut v: Vec<T>, n: usize) -> (Vec<T>, bool) {
    let cut = v.len() > n;
    v.truncate(n);
    (v, cut)
}

fn hand_ids(state: &GameState, seat: usize) -> Vec<CardId> {
    state.players[seat].hand.iter().map(|c| c.id).collect()
}

/// Legal targets for a hand card's spell effect.
fn cast_targets(state: &GameState, seat: usize, id: CardId) -> Vec<Target> {
    let Some(card) = state.players[seat].hand.iter().find(|c| c.id == id) else {
        return Vec::new();
    };
    state.enumerate_legal_targets_with_source(&card.definition.effect, seat, Some(id))
}

/// Legal targets for an activated ability's effect.
fn ability_targets(state: &GameState, seat: usize, id: CardId, idx: usize) -> Vec<Target> {
    let Some(card) = state.battlefield_find(id) else {
        return Vec::new();
    };
    let Some(ability) = card.definition.activated_abilities.get(idx) else {
        return Vec::new();
    };
    state.enumerate_legal_targets_with_source(&ability.effect, seat, Some(id))
}

/// Outcome of certifying a puzzle.
#[derive(Debug, Clone)]
pub struct Certificate {
    /// The shallowest solution found, as the actions to take in order.
    pub line: Vec<GameAction>,
    /// Number of `seat` actions in that line — the difficulty tier.
    pub depth: usize,
    /// Set when a cap bound the search, so "unsolved" is not overclaimed.
    pub truncated: bool,
}

/// Depth-first search for a line achieving `goal`, shallowest first.
///
/// Iterative deepening rather than plain DFS, because the *depth* of the
/// shallowest solution is the number a puzzle records as its difficulty:
/// a lethal that needs one attack declaration is a different puzzle from
/// one that needs a pump, a second pump, and then the right attack.
pub fn solve(state: &GameState, seat: usize, goal: Goal, max_depth: usize) -> Option<Certificate> {
    for depth in 1..=max_depth {
        let mut truncated = false;
        if let Some(line) = search(state, state, seat, goal, depth, &mut truncated) {
            let d = line.len();
            return Some(Certificate { line, depth: d, truncated });
        }
    }
    None
}

/// Fresh [`HeuristicBot`]s for a playout, synced to combat declarations
/// that have already happened.
///
/// The bot latches "I have declared attackers/blockers this combat" on
/// itself so the match actor can poll it every tick without it
/// re-submitting. A bot constructed mid-combat has those latches clear,
/// so it will happily declare blocks a second time — which silently
/// undid the premise of every combat puzzle: a position built by
/// declining blocks had them re-declared underneath it, and a lethal
/// swing read as survived. Sync the latches to the position instead.
fn playout_bots(g: &GameState) -> [HeuristicBot; 2] {
    let mut bots = [HeuristicBot::new(), HeuristicBot::new()];
    for b in &mut bots {
        if !g.attacking().is_empty() {
            b.note_external_declaration(g, true);
        }
        if g.blockers_declared() {
            b.note_external_declaration(g, false);
        }
    }
    bots
}

/// Play `state` forward to the end of the turn with *both* seats on the
/// default heuristic, and return where it lands.
///
/// Every goal is judged here rather than at the node itself, because most
/// of them are only meaningful once the turn has resolved: "survive this
/// turn" is trivially true the instant before blockers are declared, and
/// an earlier version of this file duly certified an empty line as the
/// answer to a lethal-attack-incoming puzzle. Judging after a playout
/// asks the question that actually matters — *if I stop choosing here and
/// let the default policy finish, does the goal hold?*
///
/// It also gives depth 0 a precise meaning: the default policy solves
/// this position unaided, i.e. the puzzle is trivial. That is exactly the
/// "greedy doesn't stumble into it" criterion a puzzle has to fail to be
/// worth keeping.
fn resolve(state: &GameState, seat: usize) -> GameState {
    let mut g = state.clone();
    let mut bots = playout_bots(&g);
    let start_turn = g.turn_number;
    let mut fuel = 400u32;
    while fuel > 0 && !g.is_game_over() && g.turn_number == start_turn {
        fuel -= 1;
        let acted = (0..2).any(|s| {
            let order = if s == 0 { seat } else { 1 - seat };
            bots[order]
                .next_action(&g, order)
                .is_some_and(|a| g.perform_action(a).is_ok())
        });
        if acted {
            continue;
        }
        if g.perform_action(GameAction::PassPriority).is_err() {
            break;
        }
    }
    g
}

fn search(
    origin: &GameState,
    state: &GameState,
    seat: usize,
    goal: Goal,
    budget: usize,
    truncated: &mut bool,
) -> Option<Vec<GameAction>> {
    if goal_met(origin, &resolve(state, seat), seat, goal) {
        return Some(Vec::new());
    }
    if budget == 0 || state.is_game_over() {
        return None;
    }
    let e = legal_actions(state, seat);
    *truncated |= e.truncated;
    for a in e.actions {
        let mut next = state.clone();
        if next.perform_action(a.clone()).is_err() {
            continue;
        }
        // Let the opponent and the engine respond, so the line is judged
        // on what actually happens rather than on the instant it resolves.
        settle(&mut next, seat);
        if let Some(mut rest) = search(origin, &next, seat, goal, budget - 1, truncated) {
            let mut line = vec![a];
            line.append(&mut rest);
            return Some(line);
        }
    }
    None
}

/// Advance the game until `seat` faces a *substantive* choice, the turn
/// ends, or the game does.
///
/// The subtlety this exists to handle: declaring attackers does not deal
/// damage. Between the declaration and the damage step sit several
/// priority windows where passing is the only thing the seat can do, and
/// an earlier version of this function handed control back at the first
/// of them. The search then spent its depth budget on `PassPriority`
/// chains and never reached the damage that decides a lethal puzzle.
///
/// So `seat` auto-passes through windows where passing is its only
/// option, and only stops where it has a real decision. That also gives
/// [`Certificate::depth`] its meaning: the number of substantive choices
/// in the line, not the number of engine ticks.
///
/// The opponent is piloted by the default heuristic throughout, which is
/// what makes a puzzle's answer robust rather than a scripted line —
/// "win this turn" has to survive the opponent blocking as well as it can.
fn settle(g: &mut GameState, seat: usize) {
    let opp = 1 - seat;
    let mut bots = playout_bots(g);
    let start_turn = g.turn_number;
    let mut fuel = 400u32;
    while fuel > 0 && !g.is_game_over() && g.turn_number == start_turn {
        fuel -= 1;
        // Pending decisions are answered by policy for whichever seat owns
        // them — including `seat`. A puzzle whose answer is a decision
        // policy is a different class and is out of the enumerated action
        // space by construction (see `legal_actions`).
        if let Some(p) = &g.pending_decision {
            let owner = p.acting_player();
            if let Some(a) = bots[owner].next_action(g, owner)
                && g.perform_action(a).is_ok()
            {
                continue;
            }
            return; // an unanswerable decision would spin the loop
        }
        if let Some(a) = bots[opp].next_action(g, opp)
            && g.perform_action(a).is_ok()
        {
            continue;
        }
        if has_substantive_choice(g, seat) {
            return;
        }
        if g.perform_action(GameAction::PassPriority).is_err() {
            return;
        }
    }
}

/// Whether `seat` has anything to decide here beyond passing.
fn has_substantive_choice(g: &GameState, seat: usize) -> bool {
    legal_actions(g, seat)
        .actions
        .iter()
        .any(|a| !matches!(a, GameAction::PassPriority))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardDefinition, CardType};
    use crate::game::two_player_game;

    fn vanilla(name: &'static str, p: i32, t: i32) -> CardDefinition {
        CardDefinition {
            name,
            card_types: vec![CardType::Creature],
            power: p,
            toughness: t,
            ..Default::default()
        }
    }

    fn advance_to(g: &mut GameState, step: TurnStep) {
        while g.step != step {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
    }

    /// `two_player_game` ships empty libraries, so any test that crosses a
    /// draw step loses to decking (CR 104.3c) before the puzzle is even
    /// asked. Stock both.
    fn with_libraries(g: &mut GameState) {
        for seat in 0..2 {
            for i in 0..20 {
                let def = vanilla("Filler", 1, 1);
                let id = g.add_card_to_hand(seat, def);
                let card = g.players[seat]
                    .hand
                    .iter()
                    .position(|c| c.id == id)
                    .map(|p| g.players[seat].hand.remove(p))
                    .expect("just added");
                let _ = i;
                g.players[seat].library.push(card);
            }
        }
    }

    /// Hand the turn to `seat`, stopping if the game ends underneath us.
    fn give_turn_to(g: &mut GameState, seat: usize) {
        let mut fuel = 200;
        while g.active_player_idx != seat && fuel > 0 && !g.is_game_over() {
            fuel -= 1;
            if g.perform_action(GameAction::PassPriority).is_err() {
                break;
            }
        }
        assert!(!g.is_game_over(), "game ended while handing over the turn");
    }

    /// The enumerator must offer every attack *subset*, not just the
    /// greedy all-in — a puzzle whose answer is "hold one back" is
    /// uncertifiable otherwise, and that is precisely the class round 31
    /// showed the sims care about.
    #[test]
    fn attack_enumeration_offers_every_subset() {
        let mut g = two_player_game();
        for (n, p, t) in [("A", 2, 2), ("B", 3, 3)] {
            let id = g.add_card_to_battlefield(0, vanilla(n, p, t));
            g.clear_sickness(id);
        }
        advance_to(&mut g, TurnStep::DeclareAttackers);
        let acts = legal_actions(&g, 0).actions;
        let mut sizes: Vec<usize> = acts
            .iter()
            .filter_map(|a| match a {
                GameAction::DeclareAttackers(v) => Some(v.len()),
                _ => None,
            })
            .collect();
        sizes.sort_unstable();
        assert_eq!(sizes, vec![0, 1, 1, 2], "want none/each/both: {sizes:?}");
    }

    /// A creature that cannot legally attack must not appear in any
    /// subset. Summoning sickness is the cheapest such rule, and the
    /// point is that the enumerator learns it from the engine rather
    /// than checking a field.
    #[test]
    fn enumeration_excludes_illegal_attackers() {
        let mut g = two_player_game();
        let ready = g.add_card_to_battlefield(0, vanilla("Ready", 2, 2));
        g.clear_sickness(ready);
        // Second body stays summoning-sick.
        g.add_card_to_battlefield(0, vanilla("Sick", 5, 5));
        advance_to(&mut g, TurnStep::DeclareAttackers);
        let acts = legal_actions(&g, 0).actions;
        for a in &acts {
            if let GameAction::DeclareAttackers(v) = a {
                assert!(
                    v.iter().all(|atk| atk.attacker == ready),
                    "sick creature enumerated as an attacker"
                );
            }
        }
        // ... and the legal one is still offered.
        assert!(acts.iter().any(|a| matches!(a, GameAction::DeclareAttackers(v) if v.len() == 1)));
    }

    /// An obvious lethal certifies at **depth 0**, and that is the design
    /// rather than a weakness: depth counts the choices that must be
    /// forced *before* the default policy can finish the job, so zero
    /// means the heuristic already finds it unaided. A puzzle certifying
    /// at depth 0 is a trivial puzzle and should not be kept.
    #[test]
    fn an_obvious_lethal_is_certified_trivial() {
        let mut g = two_player_game();
        with_libraries(&mut g);
        g.players[1].life = 4;
        let id = g.add_card_to_battlefield(0, vanilla("Beater", 5, 5));
        g.clear_sickness(id);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        let cert = solve(&g, 0, Goal::WinThisTurn, 3).expect("lethal exists");
        assert_eq!(cert.depth, 0, "the heuristic finds this one: {:?}", cert.line);
    }

    /// The contract that gives `depth` its meaning, asserted directly:
    /// depth 0 is exactly the case where playing the position out on
    /// autopilot already meets the goal. Every difficulty tier the
    /// benchmark reports is measured from this baseline.
    #[test]
    fn depth_zero_is_exactly_what_the_default_policy_already_solves() {
        let mut g = two_player_game();
        with_libraries(&mut g);
        g.players[1].life = 4;
        let id = g.add_card_to_battlefield(0, vanilla("Beater", 5, 5));
        g.clear_sickness(id);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        let solved_by_default = goal_met(&g, &resolve(&g, 0), 0, Goal::WinThisTurn);
        let depth = solve(&g, 0, Goal::WinThisTurn, 3).map(|c| c.depth);
        assert_eq!(solved_by_default, depth == Some(0));
    }

    /// And it must NOT hallucinate a line: with the opponent out of range
    /// there is no winning play, and `solve` has to say so rather than
    /// returning a plausible attack.
    #[test]
    fn solve_reports_no_line_when_there_is_none() {
        let mut g = two_player_game();
        with_libraries(&mut g);
        g.players[1].life = 20;
        let id = g.add_card_to_battlefield(0, vanilla("Beater", 2, 2));
        g.clear_sickness(id);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        assert!(solve(&g, 0, Goal::WinThisTurn, 2).is_none());
    }

    /// Survival is judged *after* damage. This is the bug the goal was
    /// born with: before blockers are declared, "still alive" is trivially
    /// true, and the solver duly certified an empty line as the answer to
    /// an incoming lethal swing. Declining the block at 3 life against a
    /// 3/3 must not read as survival.
    #[test]
    fn survival_is_judged_after_damage_not_before() {
        let mut g = two_player_game();
        with_libraries(&mut g);
        g.players[0].life = 3;
        let atk = g.add_card_to_battlefield(1, vanilla("Ogre", 3, 3));
        let blk = g.add_card_to_battlefield(0, vanilla("Wall", 0, 4));
        g.clear_sickness(blk);
        give_turn_to(&mut g, 1);
        g.clear_sickness(atk);
        advance_to(&mut g, TurnStep::DeclareAttackers);
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: atk,
            target: AttackTarget::Player(0),
        }]))
        .expect("attack");
        crate::game::drain_stack(&mut g);
        advance_to(&mut g, TurnStep::DeclareBlockers);

        let mut unblocked = g.clone();
        unblocked
            .perform_action(GameAction::DeclareBlockers(Vec::new()))
            .expect("decline blocks");
        assert!(
            !goal_met(&g, &resolve(&unblocked, 0), 0, Goal::SurviveTurn),
            "taking 3 at 3 life must not count as surviving"
        );
    }
}

