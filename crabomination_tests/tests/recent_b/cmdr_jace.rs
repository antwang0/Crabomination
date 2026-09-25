//! Commander: the Multiverse Reforged precon (FRC, Jace, Multiverse
//! Architect, `decks::cmdr_jace`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..seats {
        for _ in 0..8 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target], mode: Option<usize>) -> Result<(), String> {
    flood(g, 0);
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode,
        x_value: None,
    })
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// CR 306.5b — Empower Jace: the first empower mints a blue Jace token with
/// that much loyalty, the next grows the same token, and its −1 surveils.
#[test]
fn cr_306_5b_empower_jace_mints_then_grows_a_jace_token() {
    let mut g = main_phase(2);
    let charm = g.add_card_to_hand(0, catalog::fatehold_charm());
    cast(&mut g, charm, &[], Some(0)).expect("draw + empower 2");
    let jace = named(&g, 0, "Jace");
    assert_eq!(jace.len(), 1);
    assert_eq!(g.battlefield_find(jace[0]).unwrap().counter_count(CounterType::Loyalty), 2);
    let again = g.add_card_to_hand(0, catalog::fatehold_charm());
    cast(&mut g, again, &[], Some(0)).expect("empower 2 again");
    assert_eq!(named(&g, 0, "Jace"), jace, "the same token grew");
    assert_eq!(g.battlefield_find(jace[0]).unwrap().counter_count(CounterType::Loyalty), 4);
    act(&mut g, GameAction::ActivateLoyaltyAbility { card_id: jace[0], ability_index: 0, target: None, x_value: None })
        .expect("−1: surveil 1");
    assert_eq!(g.battlefield_find(jace[0]).unwrap().counter_count(CounterType::Loyalty), 3);
}

/// CR 508.1 — Jace, Multiverse Architect: an opponent who declines the {2}
/// at their beginning of combat can't attack your Jaces that turn, but can
/// still attack you.
#[test]
fn cr_508_1_jace_architect_taxes_attacks_on_jaces() {
    let mut g = main_phase(2);
    let jace = g.add_card_to_battlefield(0, catalog::jace_multiverse_architect());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear,
            target: AttackTarget::Planeswalker(jace),
        }]))
        .is_err(),
        "can't attack a Jace this turn",
    );
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("the player is still attackable");
}

/// CR 702.26 / 119.7 — Teferi's Reproach: the target opponent's nonland
/// permanents phase out and their life total can't change until their turn.
#[test]
fn cr_702_26_teferis_reproach_phases_out_and_locks_life() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let reproach = g.add_card_to_hand(0, catalog::teferis_reproach());
    cast(&mut g, reproach, &[Target::Player(1)], None).expect("cast");
    assert!(g.battlefield_find(bear).is_none(), "phased out");
    assert!(g.battlefield_find(forest).is_some(), "lands stay");
    assert!(g.exile.iter().any(|c| c.id == reproach), "exiled on resolution");
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let _ = cast(&mut g, bolt, &[Target::Player(1)], None);
    assert_eq!(g.players[1].life, life, "life can't change");
}

/// Dack Fayden, Helping Hand — at four seats it reveals three creatures,
/// goads each and hands one to every opponent.
#[test]
fn dack_fayden_hands_out_goaded_creatures() {
    let mut g = main_phase(4);
    g.players[0].library.clear();
    let bears: Vec<CardId> = (0..3).map(|_| g.add_card_to_library(0, catalog::grizzly_bears())).collect();
    let dack = g.add_card_to_hand(0, catalog::dack_fayden_helping_hand());
    cast(&mut g, dack, &[], None).expect("cast");
    let mut owners: Vec<usize> = bears.iter().map(|b| g.battlefield_find(*b).expect("in play").controller).collect();
    owners.sort();
    assert_eq!(owners, vec![1, 2, 3], "one to each opponent");
    assert!(bears.iter().all(|b| g.is_goaded(g.battlefield_find(*b).unwrap())));
}

/// Omnath, Locus of the Void grows with unspent mana; its landfall adds
/// {C}{C}.
#[test]
fn omnath_locus_of_the_void_counts_unspent_mana() {
    let mut g = main_phase(2);
    let omnath = g.add_card_to_battlefield(0, catalog::omnath_locus_of_the_void());
    assert_eq!(g.computed_permanent(omnath).unwrap().power, 6);
    let land = g.add_card_to_hand(0, catalog::forest());
    act(&mut g, GameAction::PlayLand(land)).expect("land");
    assert_eq!(g.players[0].mana_pool.total(), 2);
    let cp = g.computed_permanent(omnath).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8));
}

/// The Ur-Sphinx — a Sphinx attacking mills each player one and offers one
/// free cast from each player's milled cards.
#[test]
fn the_ur_sphinx_mills_and_casts() {
    let mut g = main_phase(2);
    let sphinx = g.add_card_to_battlefield(0, catalog::the_ur_sphinx());
    g.clear_sickness(sphinx);
    g.players[1].library.clear();
    let theirs = g.add_card_to_library(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: sphinx, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let bear = g.battlefield_find(theirs).expect("their milled Bear was cast");
    assert_eq!(bear.controller, 0);
}

/// Jhoira, Weatherlight Corsair steals the first historic permanent card of
/// the target opponent's library and costs you its mana value in life.
#[test]
fn jhoira_steals_a_historic_permanent() {
    let mut g = main_phase(2);
    g.players[1].library.clear();
    g.add_card_to_library(1, catalog::grizzly_bears());
    let ring = g.add_card_to_library(1, catalog::sol_ring());
    let jhoira = g.add_card_to_hand(0, catalog::jhoira_weatherlight_corsair());
    cast(&mut g, jhoira, &[Target::Player(1)], None).expect("cast");
    assert_eq!(g.battlefield_find(ring).expect("stolen").controller, 0);
    assert_eq!(g.players[0].life, 19, "lost the Sol Ring's mana value");
}
