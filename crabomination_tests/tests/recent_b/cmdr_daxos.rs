//! Commander: the Call the Spirits precon (C15, Daxos, `decks::cmdr_daxos`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn declare(g: &mut GameState, attacker: CardId, defender: usize) -> Result<(), String> {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(defender),
    }]))
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

#[test]
fn dawnglare_invoker_taps_every_creature_of_the_target_player() {
    let mut g = main_phase(3);
    let inv = g.add_card_to_battlefield(0, catalog::dawnglare_invoker());
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    activate(&mut g, inv, 0, Some(Target::Player(1))).expect("activate");
    for id in [a, b] {
        assert!(g.battlefield_find(id).unwrap().tapped);
    }
    assert!(!g.battlefield_find(other).unwrap().tapped);
}

/// Constellation: another enchantment entering animates it until end of turn.
#[test]
fn daxoss_torment_becomes_a_hasty_flying_demon() {
    let mut g = main_phase(2);
    let t = g.add_card_to_battlefield(0, catalog::daxoss_torment());
    assert!(!g.computed_permanent(t).is_some_and(|c| c.card_types().contains(&crabomination::card::CardType::Creature)));
    let peril = g.add_card_to_hand(0, catalog::grave_peril());
    cast(&mut g, peril, &[]);
    assert_eq!(pt(&g, t), (5, 5));
    assert!(g.permanent_has_keyword(t, &Keyword::Flying));
    assert!(g.permanent_has_keyword(t, &Keyword::Haste));
}

/// Each player loses one life per creature they controlled that was
/// destroyed — tokens count too, although they are gone by then.
#[test]
fn deadly_tempest_charges_each_player_for_their_own_creatures() {
    let mut g = main_phase(3);
    let conf = g.add_card_to_hand(0, catalog::righteous_confluence());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: conf, spree_modes: vec![0, 0, 0], target: None, additional_targets: vec![], x_value: None,
    })
    .expect("CR 700.2d — the Knight mode three times");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Knight"), 3);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = [g.players[0].life, g.players[1].life, g.players[2].life];
    let t = g.add_card_to_hand(0, catalog::deadly_tempest());
    cast(&mut g, t, &[]);
    assert!(g.battlefield.iter().all(|c| !c.definition.is_creature()));
    assert_eq!(g.players[0].life, life[0] - 3);
    assert_eq!(g.players[1].life, life[1] - 1);
    assert_eq!(g.players[2].life, life[2]);
}

#[test]
fn fallen_ideal_grants_flight_and_a_sacrifice_pump_then_comes_home() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let food = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ideal = g.add_card_to_hand(0, catalog::fallen_ideal());
    cast(&mut g, ideal, &[Target::Permanent(bear)]);
    assert!(g.permanent_has_keyword(bear, &Keyword::Flying));
    activate(&mut g, bear, 0, None).expect("granted sacrifice ability");
    assert!(g.battlefield_find(food).is_none());
    assert_eq!(pt(&g, bear), (4, 3));
    let kill = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, kill, &[Target::Permanent(bear)]);
    assert!(g.players[0].hand.iter().any(|c| c.id == ideal), "the Aura returns to hand");
}

/// A nonblack creature entering costs Grave Peril and that creature; a black
/// one (Karlov is white and black) does not trigger it.
#[test]
fn grave_peril_trades_itself_for_the_first_nonblack_creature() {
    let mut g = main_phase(2);
    let peril = g.add_card_to_battlefield(0, catalog::grave_peril());
    let karlov = g.add_card_to_hand(0, catalog::karlov_of_the_ghost_council());
    cast(&mut g, karlov, &[]);
    assert!(g.battlefield_find(peril).is_some() && g.battlefield_find(karlov).is_some());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]);
    assert!(g.battlefield_find(peril).is_none());
    assert!(g.battlefield_find(bear).is_none());
}

/// CR 702.116a — myriad copies attack each other opponent.
#[test]
fn herald_of_the_host_attacks_every_opponent() {
    let mut g = main_phase(3);
    let h = g.add_card_to_battlefield(0, catalog::herald_of_the_host());
    declare(&mut g, h, 1).expect("attack");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Herald of the Host"), 2);
}

#[test]
fn karlov_grows_on_life_gain_and_spends_six_counters_to_exile() {
    let mut g = main_phase(2);
    let k = g.add_card_to_battlefield(0, catalog::karlov_of_the_ghost_council());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
        let r = g.add_card_to_hand(0, catalog::revitalize());
        cast(&mut g, r, &[]);
    }
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 6);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, k, 0, Some(Target::Permanent(victim))).expect("exile");
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(g.battlefield_find(k).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

#[test]
fn necromancers_covenant_turns_a_graveyards_creatures_into_lifelinking_zombies() {
    let mut g = main_phase(2);
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::swamp());
    let c = g.add_card_to_hand(0, catalog::necromancers_covenant());
    cast(&mut g, c, &[]);
    assert_eq!(count_named(&g, 0, "Zombie"), 2);
    assert_eq!(g.players[1].graveyard.len(), 1, "the land stays");
    let z = g.battlefield.iter().find(|c| c.definition.name == "Zombie").unwrap().id;
    assert!(g.permanent_has_keyword(z, &Keyword::Lifelink));
}

#[test]
fn new_benalia_enters_tapped() {
    let mut g = main_phase(2);
    let land = g.add_card_to_hand(0, catalog::new_benalia());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped);
}

/// X counts every player with more lands than you: one of two opponents here.
#[test]
fn oreskos_explorer_finds_a_plains_per_player_ahead_on_lands() {
    let mut g = main_phase(3);
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(1, catalog::swamp());
    g.add_card_to_battlefield(1, catalog::swamp());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::plains());
    }
    let e = g.add_card_to_hand(0, catalog::oreskos_explorer());
    cast(&mut g, e, &[]);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Plains").count(), 1);
}

#[test]
fn sandstone_oracle_draws_up_to_the_fullest_opponent_hand() {
    let mut g = main_phase(3);
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::swamp());
        g.add_card_to_library(0, catalog::plains());
    }
    g.add_card_to_hand(2, catalog::swamp());
    let o = g.add_card_to_hand(0, catalog::sandstone_oracle());
    cast(&mut g, o, &[]);
    assert_eq!(g.players[0].hand.len(), 4);
}

#[test]
fn vivid_meadow_enters_tapped_with_two_charge_counters() {
    let mut g = main_phase(2);
    let land = g.add_card_to_hand(0, catalog::vivid_meadow());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    let c = g.battlefield_find(land).unwrap();
    assert!(c.tapped);
    assert_eq!(c.counter_count(CounterType::Charge), 2);
}

/// CR 508.1a — a Vow on an opponent's creature keeps it off you (and your
/// planeswalkers) but not off anyone else.
#[test]
fn a_vow_keeps_the_creature_off_its_aura_controller_only() {
    for vow in [catalog::vow_of_duty as fn() -> _, catalog::vow_of_malice] {
        let mut g = main_phase(3);
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let v = g.add_card_to_hand(0, vow());
        cast(&mut g, v, &[Target::Permanent(bear)]);
        assert_eq!(pt(&g, bear), (4, 4));
        g.active_player_idx = 1;
        assert!(declare(&mut g, bear, 0).is_err(), "can't attack the Vow's controller");
        declare(&mut g, bear, 2).expect("another opponent is fine");
    }
}
