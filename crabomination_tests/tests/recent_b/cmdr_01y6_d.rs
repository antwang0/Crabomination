//! Commander: the 2026-10-25 (`01Y6ZRFx`) Molly Hayes, Runaway closers
//! (`decks::cmdr_01y6_d`).

use crabomination::card::{CardId, Keyword, Supertype};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

fn to_upkeep(g: &mut GameState, seat: usize) {
    g.active_player_idx = seat;
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = seat;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

/// Declare `attackers` at seat 1, no blocks, deal the damage.
fn swing(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    g.step = TurnStep::CombatDamage;
    g.priority.player_with_priority = 0;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

/// Trip Noose taps a creature; Ruinous Gremlin trades itself for an artifact.
#[test]
fn trip_noose_and_ruinous_gremlin() {
    let mut g = pod(2);
    let noose = ready(&mut g, 0, catalog::trip_noose());
    let gremlin = ready(&mut g, 0, catalog::ruinous_gremlin());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rock = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, noose, Some(Target::Permanent(bear))).expect("noose");
    assert!(g.battlefield_find(bear).unwrap().tapped);
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Red, 1);
    activate(&mut g, gremlin, Some(Target::Permanent(rock))).expect("gremlin");
    assert!(g.battlefield_find(rock).is_none() && g.battlefield_find(gremlin).is_none());
}

/// Mage-Ring Responder's attack burns a creature of the defending player for 7.
#[test]
fn mage_ring_responder_burns_on_attack() {
    let mut g = pod(3);
    let golem = ready(&mut g, 0, catalog::mage_ring_responder());
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: golem, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(wurm).is_none(), "7 damage to the defending player's Wurm");
}

/// Squawkroaster's power counts the colours among your permanents.
#[test]
fn squawkroaster_counts_your_colors() {
    let mut g = pod(2);
    let s = g.add_card_to_battlefield(0, catalog::squawkroaster());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::savannah_lions());
    let c = g.computed_permanent(s).unwrap();
    assert_eq!((c.power, c.toughness), (2, 4), "red and green");
    assert!(g.permanent_has_keyword(s, &Keyword::DoubleStrike));
}

/// Bloodshed Fever: its host must attack.
#[test]
fn bloodshed_fever_forces_attacks() {
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let fever = g.add_card_to_hand(0, catalog::bloodshed_fever());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: fever,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.permanent_has_keyword(bear, &Keyword::MustAttack));
}

/// Chaos Moon: an odd permanent count pumps red creatures and doubles a
/// Mountain's {R}; an even one shrinks them and turns the Mountain colorless.
#[test]
fn chaos_moon_reads_the_permanent_count() {
    for odd in [true, false] {
        let mut g = pod(2);
        g.add_card_to_battlefield(1, catalog::chaos_moon());
        let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
        let mountain = g.add_card_to_battlefield(0, catalog::mountain());
        if !odd {
            g.add_card_to_battlefield(1, catalog::plains());
        }
        // Chaos Moon, the Giant and the Mountain are three; a Plains makes four.
        to_upkeep(&mut g, 0);
        assert_eq!(g.computed_permanent(giant).unwrap().power, if odd { 4 } else { 2 });
        g.battlefield_find_mut(mountain).unwrap().tapped = false;
        activate(&mut g, mountain, None).expect("tap for mana");
        let pool = &g.players[0].mana_pool;
        if odd {
            assert_eq!(pool.amount(Color::Red), 2);
        } else {
            assert_eq!((pool.amount(Color::Red), pool.total()), (0, 1), "one colorless");
        }
    }
}

/// CR 205.4 — Melting: a snow land is no longer snow.
#[test]
fn melting_strips_snow_from_lands() {
    let mut g = pod(2);
    let island = g.add_card_to_battlefield(1, catalog::snow_covered_island());
    assert!(g.computed_permanent(island).unwrap().supertypes().contains(&Supertype::Snow));
    g.add_card_to_battlefield(0, catalog::melting());
    assert!(!g.computed_permanent(island).unwrap().supertypes().contains(&Supertype::Snow));
}

/// Total War: attacking destroys the attacker's untapped stay-at-homes, but
/// not a Wall or a creature that came in this turn.
#[test]
fn total_war_punishes_the_creatures_that_stay_home() {
    let mut g = pod(2);
    g.add_card_to_battlefield(1, catalog::total_war());
    let attacker = ready(&mut g, 0, catalog::grizzly_bears());
    let home = ready(&mut g, 0, catalog::savannah_lions());
    let wall = ready(&mut g, 0, catalog::wall_of_stone());
    let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(fresh).unwrap().entered_turn = Some(g.turn_number);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(home).is_none(), "stayed home: destroyed");
    assert!(g.battlefield_find(attacker).is_some());
    assert!(g.battlefield_find(wall).is_some(), "a Wall is spared");
    assert!(g.battlefield_find(fresh).is_some(), "came in this turn: spared");
}

/// Thunderblade Charge comes back from the graveyard for {2}{R}{R}{R} when
/// your creatures connect.
#[test]
fn thunderblade_charge_recasts_from_the_graveyard() {
    let mut g = pod(2);
    let charge = g.add_card_to_graveyard(0, catalog::thunderblade_charge());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    swing(&mut g, &[bear]);
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == charge) || g.stack.is_empty());
    assert_eq!(g.players[1].life, life - 2 - 3, "2 combat + 3 from the free Charge");
}
