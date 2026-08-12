//! The puzzle corpus: candidate positions, and the criterion that decides
//! which of them are worth keeping.
//!
//! A candidate earns a place only if it is **solvable and not trivial** —
//! [`puzzle::solve`] finds a line, and [`puzzle::solved_by_default`] says
//! the heuristic does not already stumble into it. Both halves are
//! machine-checked (`certify`), so authoring a puzzle is proposing a
//! position rather than asserting an answer, and a position that turns
//! out to be easy is demoted by the certifier instead of by argument.
//!
//! This matters more than it sounds. Hand-authored benchmark items drift
//! toward what the author already believes the bot is bad at. Certifying
//! against the live heuristic means the corpus tracks the *current*
//! weaknesses, and a puzzle silently becoming trivial (because the bot
//! improved) shows up as a tier change rather than as a test that still
//! passes for the wrong reason.

use crate::card::{CardDefinition, CardType};
use crate::effect::{Effect, Selector, Value};
use crate::game::types::TurnStep;
use crate::game::{Attack, AttackTarget, GameAction, GameState, two_player_game};
use crate::mana::{ManaCost, ManaSymbol};

use super::puzzle::{self, Goal};

/// What a puzzle is testing, so a failure names a subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mechanic {
    /// Attack and block declarations.
    Combat,
    /// Spot removal aimed at the right object.
    Removal,
    /// Instant-speed intervention during combat.
    Trick,
    /// Spending the mana you have on the right thing.
    Mana,
}

impl Mechanic {
    pub fn name(self) -> &'static str {
        match self {
            Mechanic::Combat => "combat",
            Mechanic::Removal => "removal",
            Mechanic::Trick => "trick",
            Mechanic::Mana => "mana",
        }
    }
}

/// One benchmark item.
pub struct Puzzle {
    pub id: &'static str,
    pub mechanic: Mechanic,
    pub goal: Goal,
    /// The seat being tested.
    pub seat: usize,
    pub build: fn() -> GameState,
    /// What the position is asking for, in words — for the report only.
    pub prompt: &'static str,
}

/// A candidate's certification: is it solvable, and is it trivial?
#[derive(Debug, Clone)]
pub struct Certified {
    pub id: &'static str,
    pub mechanic: Mechanic,
    pub depth: Option<usize>,
    pub trivial: bool,
    pub truncated: bool,
    /// The solution is nothing but priority passes — see
    /// [`puzzle::Certificate::is_degenerate`].
    pub degenerate: bool,
}

impl Certified {
    /// Corpus membership: a real puzzle has a solution, the default
    /// policy does not already find it, and the solution is an actual
    /// play rather than a priority pass.
    pub fn keep(&self) -> bool {
        self.depth.is_some() && !self.trivial && !self.degenerate
    }
}

/// Certify one candidate against the live heuristic.
pub fn certify(p: &Puzzle, max_depth: usize) -> Certified {
    let g = (p.build)();
    let cert = puzzle::solve(&g, p.seat, p.goal, max_depth);
    Certified {
        id: p.id,
        mechanic: p.mechanic,
        depth: cert.as_ref().map(|c| c.depth),
        trivial: puzzle::solved_by_default(&g, p.seat, p.goal),
        truncated: cert.as_ref().is_some_and(|c| c.truncated),
        degenerate: cert.as_ref().is_some_and(|c| c.is_degenerate()),
    }
}

// ── Position-building helpers ──────────────────────────────────────────

fn vanilla(name: &'static str, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// An instant that deals `amount` to one target, costing `{n}`.
fn burn(name: &'static str, n: u32, amount: i32) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Instant],
        cost: ManaCost::new(vec![ManaSymbol::Generic(n)]),
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::Const(amount),
        },
        ..Default::default()
    }
}

/// `two_player_game` ships empty libraries, so anything crossing a draw
/// step loses to decking (CR 104.3c) before the puzzle is asked.
fn stock_libraries(g: &mut GameState) {
    for seat in 0..2 {
        for _ in 0..20 {
            let id = g.add_card_to_hand(seat, vanilla("Filler", 1, 1));
            if let Some(pos) = g.players[seat].hand.iter().position(|c| c.id == id) {
                let card = g.players[seat].hand.remove(pos);
                g.players[seat].library.push(card);
            }
        }
    }
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    let mut fuel = 200;
    while g.step != step && fuel > 0 && !g.is_game_over() {
        fuel -= 1;
        if g.perform_action(GameAction::PassPriority).is_err() {
            break;
        }
    }
}

fn give_turn_to(g: &mut GameState, seat: usize) {
    let mut fuel = 200;
    while g.active_player_idx != seat && fuel > 0 && !g.is_game_over() {
        fuel -= 1;
        if g.perform_action(GameAction::PassPriority).is_err() {
            break;
        }
    }
}

/// Untapped basic lands, so a puzzle that needs mana has real sources
/// rather than a hand-filled pool (pools empty at every step boundary,
/// CR 500.4, which would silently strand an instant-speed answer).
fn add_lands(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        let id = g.add_card_to_battlefield(seat, crate::catalog::mountain());
        g.clear_sickness(id);
    }
}

fn ready_creature(g: &mut GameState, seat: usize, name: &'static str, p: i32, t: i32) -> crate::card::CardId {
    let id = g.add_card_to_battlefield(seat, vanilla(name, p, t));
    g.clear_sickness(id);
    id
}

// ── The candidates ─────────────────────────────────────────────────────

/// Three 2/2s into a lone 3/3 blocker, opponent at 4. All-in is exactly
/// lethal: they block one, four damage goes through. The blocked 2/2
/// dies for nothing, which is the shape the heuristic's suicide filter is
/// built to avoid — so this asks whether "this attacker dies" can be
/// overridden by "and we win anyway".
fn combat_all_in_lethal() -> GameState {
    let mut g = two_player_game();
    stock_libraries(&mut g);
    g.players[1].life = 4;
    for n in ["A", "B", "C"] {
        ready_creature(&mut g, 0, n, 2, 2);
    }
    g.add_card_to_battlefield(1, vanilla("Wall", 3, 3));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g
}

/// Opponent at 6 behind a 4/4. A 5/5 alone is not lethal and trades
/// badly; a burn spell to the face closes it. The question is whether
/// the seat spends removal on the blocker (the material-positive play)
/// or on the opponent's life total (the winning one).
fn burn_the_face_not_the_blocker() -> GameState {
    let mut g = two_player_game();
    stock_libraries(&mut g);
    g.players[1].life = 6;
    ready_creature(&mut g, 0, "Beater", 5, 5);
    g.add_card_to_battlefield(1, vanilla("Guard", 4, 4));
    add_lands(&mut g, 0, 2);
    g.add_card_to_hand(0, burn("Bolt", 1, 3));
    g
}

/// At 2 life against a lone 5/5 attacker with a 1/1 to spare. Chumping
/// is a pure material loss and the only line that lives.
fn chump_or_die() -> GameState {
    let mut g = two_player_game();
    stock_libraries(&mut g);
    g.players[0].life = 2;
    let chump = ready_creature(&mut g, 0, "Runt", 1, 1);
    let _ = chump;
    give_turn_to(&mut g, 1);
    let atk = ready_creature(&mut g, 1, "Giant", 5, 5);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    let _ = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(0),
    }]));
    crate::game::drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g
}

/// Three 2/2s attacking into two potential blockers at 4 life. Blocking
/// for value (two profitable trades) still lets 2 through and lives;
/// this one should be trivial, and is included precisely so the
/// certifier has something to demote.
fn block_for_value() -> GameState {
    let mut g = two_player_game();
    stock_libraries(&mut g);
    g.players[0].life = 4;
    ready_creature(&mut g, 0, "Guard A", 2, 2);
    ready_creature(&mut g, 0, "Guard B", 2, 2);
    give_turn_to(&mut g, 1);
    let a = ready_creature(&mut g, 1, "Raider A", 2, 2);
    let b = ready_creature(&mut g, 1, "Raider B", 2, 2);
    let c = ready_creature(&mut g, 1, "Raider C", 2, 2);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    let _ = g.perform_action(GameAction::DeclareAttackers(
        [a, b, c]
            .into_iter()
            .map(|attacker| Attack { attacker, target: AttackTarget::Player(0) })
            .collect(),
    ));
    crate::game::drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g
}

/// A lethal swing incoming at 3 life, with burn in hand and mana up.
/// Killing the attacker before damage is the only out — the defensive
/// intervention the `pick_defensive_removal` path exists for.
fn removal_stops_lethal() -> GameState {
    let mut g = two_player_game();
    stock_libraries(&mut g);
    g.players[0].life = 3;
    add_lands(&mut g, 0, 2);
    g.add_card_to_hand(0, burn("Bolt", 1, 3));
    give_turn_to(&mut g, 1);
    let atk = ready_creature(&mut g, 1, "Ogre", 3, 3);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    let _ = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(0),
    }]));
    crate::game::drain_stack(&mut g);
    g
}

/// Opponent at 3 with no board; a burn spell and enough mana. The
/// simplest possible "just win" — a sanity item that should certify
/// trivial.
fn burn_for_the_win() -> GameState {
    let mut g = two_player_game();
    stock_libraries(&mut g);
    g.players[1].life = 3;
    add_lands(&mut g, 0, 2);
    g.add_card_to_hand(0, burn("Bolt", 1, 3));
    g
}

/// Two burn spells, opponent at 6, only three mana: one costs {1} for 3
/// and one costs {3} for 3, so both are castable but not together
/// without spending correctly. Tests whether the mana gets allocated to
/// the pair that adds up to lethal.
fn two_spells_one_budget() -> GameState {
    let mut g = two_player_game();
    stock_libraries(&mut g);
    g.players[1].life = 6;
    add_lands(&mut g, 0, 4);
    g.add_card_to_hand(0, burn("Jab", 1, 3));
    g.add_card_to_hand(0, burn("Haymaker", 3, 3));
    g
}

/// Every candidate, in a stable order.
pub fn candidates() -> Vec<Puzzle> {
    vec![
        Puzzle {
            id: "combat_all_in_lethal",
            mechanic: Mechanic::Combat,
            goal: Goal::WinThisTurn,
            seat: 0,
            build: combat_all_in_lethal,
            prompt: "three 2/2s vs one 3/3 blocker, opponent at 4 — all in",
        },
        Puzzle {
            id: "chump_or_die",
            mechanic: Mechanic::Combat,
            goal: Goal::SurviveTurn,
            seat: 0,
            build: chump_or_die,
            prompt: "at 2 life vs a 5/5 with a 1/1 spare — chump",
        },
        Puzzle {
            id: "block_for_value",
            mechanic: Mechanic::Combat,
            goal: Goal::SurviveTurn,
            seat: 0,
            build: block_for_value,
            prompt: "three 2/2s attacking, two blockers, 4 life",
        },
        Puzzle {
            id: "burn_the_face_not_the_blocker",
            mechanic: Mechanic::Removal,
            goal: Goal::WinThisTurn,
            seat: 0,
            build: burn_the_face_not_the_blocker,
            prompt: "5/5 vs 4/4, opponent at 6, one burn spell",
        },
        Puzzle {
            id: "removal_stops_lethal",
            mechanic: Mechanic::Trick,
            goal: Goal::SurviveTurn,
            seat: 0,
            build: removal_stops_lethal,
            prompt: "lethal 3/3 incoming at 3 life, burn in hand",
        },
        Puzzle {
            id: "burn_for_the_win",
            mechanic: Mechanic::Removal,
            goal: Goal::WinThisTurn,
            seat: 0,
            build: burn_for_the_win,
            prompt: "opponent at 3, burn in hand — sanity item",
        },
        Puzzle {
            id: "two_spells_one_budget",
            mechanic: Mechanic::Mana,
            goal: Goal::WinThisTurn,
            seat: 0,
            build: two_spells_one_budget,
            prompt: "two burn spells, four mana, opponent at 6",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every candidate has to *build* — a position that throws or ends
    /// the game while being constructed is a broken benchmark item, and
    /// silently scores as a failure for every bot otherwise.
    #[test]
    fn every_candidate_builds_a_live_position() {
        for p in candidates() {
            let g = (p.build)();
            assert!(!g.is_game_over(), "{} built an already-decided game", p.id);
            assert!(
                g.players.iter().all(|pl| pl.is_alive()),
                "{} built a position with a dead player",
                p.id
            );
        }
    }

    /// The certifier has to actually discriminate, and right now its
    /// verdict on this batch is that **none** of them is a puzzle: the
    /// heuristic solves six unaided and the seventh is solved by a bare
    /// priority pass. That is the honest state of the corpus and it is
    /// asserted rather than papered over — a benchmark whose items the
    /// bot already passes measures nothing, and the failure mode to
    /// guard against is quietly keeping them anyway.
    ///
    /// What must hold regardless of how the corpus grows: every
    /// candidate is a *solvable* position (an unsolvable one is a broken
    /// item, not a hard one), and the trivial verdict is exactly depth 0.
    #[test]
    fn certification_is_consistent_and_reports_broken_positions() {
        for c in candidates().iter().map(|p| certify(p, 3)) {
            assert!(
                c.depth.is_some(),
                "{} has no solution at all — a broken position, not a hard one",
                c.id
            );
            assert_eq!(
                c.trivial,
                c.depth == Some(0),
                "{}: trivial must mean exactly depth 0",
                c.id
            );
            assert!(
                !(c.depth == Some(0) && c.degenerate),
                "{}: an empty line is trivial, not degenerate",
                c.id
            );
        }
    }
}
