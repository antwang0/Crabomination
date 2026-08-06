//! Stronghold (STH) — `catalog::sets::sth`.

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), ()> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
    .map_err(|_| ())
}

/// Flowstone Mauler's {R} trades toughness for power, repeatedly.
#[test]
fn flowstone_mauler_pumps_plus_one_minus_one() {
    let mut g = two_player_game();
    let mauler = g.add_card_to_battlefield(0, catalog::flowstone_mauler());
    g.clear_sickness(mauler);
    g.step = TurnStep::PreCombatMain;
    for _ in 0..2 {
        g.players[0].mana_pool.add(Color::Red, 1);
        activate(&mut g, mauler, 0, None).expect("pump");
        drain_stack(&mut g);
    }
    let cp = g.computed_permanent(mauler).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 3));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Revenant's power and toughness track the creature cards in your graveyard.
#[test]
fn revenant_scales_with_the_graveyard() {
    let mut g = two_player_game();
    let rev = g.add_card_to_battlefield(0, catalog::revenant());
    assert_eq!(g.computed_permanent(rev).unwrap().power, 0);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // not a creature
    let cp = g.computed_permanent(rev).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Stronghold Taskmaster shrinks other black creatures but not itself.
#[test]
fn stronghold_taskmaster_spares_itself() {
    let mut g = two_player_game();
    let boss = g.add_card_to_battlefield(0, catalog::stronghold_taskmaster());
    let other = g.add_card_to_battlefield(1, catalog::dungeon_shade()); // 1/1 black
    let green = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(boss).unwrap().power, 4);
    assert_eq!(g.computed_permanent(other).unwrap().toughness, 0);
    assert_eq!(g.computed_permanent(green).unwrap().power, 2);
}

/// A Spike moves its counters onto another creature.
#[test]
fn spike_soldier_transfers_a_counter() {
    let mut g = two_player_game();
    let spike = g.move_card_to_battlefield_for_test(0, catalog::spike_soldier());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(spike).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3
    );
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(spike);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, spike, 0, Some(Target::Permanent(bear))).expect("transfer");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(spike).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2
    );
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Mogg Maniac throws the damage it takes back at an opponent.
#[test]
fn mogg_maniac_reflects_damage() {
    let mut g = two_player_game();
    let maniac = g.add_card_to_battlefield(0, catalog::mogg_maniac());
    let mut events = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(maniac),
        3,
        None,
        &mut events,
    );
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Wall of Souls mirrors the combat damage it takes; Wall of Essence banks
/// it as life.
#[test]
fn the_walls_convert_the_combat_damage_they_take() {
    let mut g = two_player_game();
    let souls = g.add_card_to_battlefield(0, catalog::wall_of_souls());
    let essence = g.add_card_to_battlefield(0, catalog::wall_of_essence());
    let attackers: Vec<CardId> = (0..2)
        .map(|_| {
            let id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
            g.clear_sickness(id);
            id
        })
        .collect();
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(
        attackers
            .iter()
            .map(|&a| Attack { attacker: a, target: AttackTarget::Player(0) })
            .collect(),
    ))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![
        (souls, attackers[0]),
        (essence, attackers[1]),
    ]))
    .expect("block");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "Wall of Souls sent 2 back");
    assert_eq!(g.players[0].life, 22, "Wall of Essence gained 2");
}

/// Ruination leaves basics alone.
#[test]
fn ruination_spares_basic_lands() {
    let mut g = two_player_game();
    let basic = g.add_card_to_battlefield(0, catalog::forest());
    let nonbasic = g.add_card_to_battlefield(1, catalog::volcanic_island());
    let ctx = EffectContext::for_ability(CardId(0), 0, None);
    g.resolve_effect(&catalog::ruination().effect, &ctx).expect("ruination");
    assert!(g.battlefield_find(basic).is_some());
    assert!(g.battlefield_find(nonbasic).is_none());
}

/// Constant Mists fogs, and its buyback is a land sacrifice rather than mana.
#[test]
fn constant_mists_buys_back_for_a_land() {
    let mut g = two_player_game();
    let mists = g.add_card_to_hand(0, catalog::constant_mists());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let lands = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellBuyback {
        card_id: mists,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("buyback cast");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == mists), "bought back");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.is_land()).count(),
        lands - 1,
        "a land paid the buyback"
    );
}

/// Horn of Greed cantrips off every player's land drop.
#[test]
fn horn_of_greed_draws_for_whoever_played_the_land() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::horn_of_greed());
    let land = g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let hand = g.players[1].hand.len();
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::PlayLand(land)).expect("land drop");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand, "played one, drew one");
}

/// Tortured Existence swaps a creature card in hand for one in the graveyard.
#[test]
fn tortured_existence_swaps_creature_cards() {
    let mut g = two_player_game();
    let engine = g.add_card_to_battlefield(0, catalog::tortured_existence());
    let pitched = g.add_card_to_hand(0, catalog::grizzly_bears());
    let buried = g.add_card_to_graveyard(0, catalog::serra_angel());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    activate(&mut g, engine, 0, Some(Target::Permanent(buried))).expect("swap");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == buried));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pitched));
}

/// Rolling Stones lets a Wall attack.
#[test]
fn rolling_stones_lifts_defender_off_walls() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_razors());
    assert!(g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::Defender));
    g.add_card_to_battlefield(0, catalog::rolling_stones());
    assert!(!g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::Defender));
}

/// Mortuary sends your dead creatures back to the top of your library.
#[test]
fn mortuary_recycles_the_dead_onto_your_library() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mortuary());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut events = vec![];
    g.destroy_permanent(bear, false, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bear));
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear));
}

/// Contemplation gains a life per spell you cast.
#[test]
fn contemplation_gains_life_per_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::contemplation());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21);
}

/// Dream Prowler is unblockable only while it attacks alone.
#[test]
fn dream_prowler_is_unblockable_alone() {
    let mut g = two_player_game();
    let prowler = g.add_card_to_battlefield(0, catalog::dream_prowler());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(prowler).unwrap().keywords.contains(&Keyword::Unblockable));

    g.clear_sickness(prowler);
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: prowler,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack alone");
    drain_stack(&mut g);
    assert!(g.computed_permanent(prowler).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Mogg Bombers goes off the moment another creature lands.
#[test]
fn mogg_bombers_detonates_on_the_next_creature() {
    let mut g = two_player_game();
    let bombers = g.add_card_to_battlefield(0, catalog::mogg_bombers());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast a creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bombers).is_none(), "it sacrificed itself");
    assert_eq!(g.players[1].life, 17);
}

/// Hermit Druid digs to a basic and bins everything above it.
#[test]
fn hermit_druid_mills_to_the_first_basic() {
    let mut g = two_player_game();
    let druid = g.add_card_to_battlefield(0, catalog::hermit_druid());
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let basic = g.add_card_to_library(0, catalog::forest());
    g.clear_sickness(druid);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, druid, 0, None).expect("dig");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == basic));
    assert_eq!(g.players[0].graveyard.len(), 3);
}
