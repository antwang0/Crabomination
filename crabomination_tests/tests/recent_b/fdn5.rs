//! Fifth Dawn gap batch 1 (`decks::recent322`).

use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Beacon of Creation mints an Insect per Forest and shuffles itself back.
#[test]
fn beacon_of_creation_counts_forests_then_reshuffles() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let beacon = g.add_card_to_hand(0, catalog::beacon_of_creation());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: beacon, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Insect").count(), 3);
    assert!(g.players[0].library.iter().any(|c| c.id == beacon), "shuffled back in");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == beacon));
}

/// Beacon of Unrest reanimates out of any graveyard.
#[test]
fn beacon_of_unrest_reanimates_from_any_graveyard() {
    let mut g = main_phase();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let corpse = g.players[1].graveyard[0].id;
    let beacon = g.add_card_to_hand(0, catalog::beacon_of_unrest());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: beacon, target: Some(Target::Permanent(corpse)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(corpse).unwrap().controller, 0);
}

/// Beacon of Tomorrows hands an extra turn to its target.
#[test]
fn beacon_of_tomorrows_grants_an_extra_turn() {
    let mut g = main_phase();
    let beacon = g.add_card_to_hand(0, catalog::beacon_of_tomorrows());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: beacon, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].extra_turns, 1);
}

/// Clock of Omens taps two artifacts to untap a third.
#[test]
fn clock_of_omens_untaps_with_two_helpers() {
    let mut g = main_phase();
    let clock = g.add_card_to_battlefield(0, catalog::clock_of_omens());
    let a = g.add_card_to_battlefield(0, catalog::tanglebloom());
    let b = g.add_card_to_battlefield(0, catalog::tanglebloom());
    let target = g.add_card_to_battlefield(0, catalog::tanglebloom());
    g.battlefield_find_mut(target).unwrap().tapped = true;
    for id in [a, b] {
        g.clear_sickness(id);
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: clock, ability_index: 0, target: Some(Target::Permanent(target)),
        additional_targets: vec![], x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(target).unwrap().tapped);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
}

/// Gemstone Array banks generic mana and pays it back in any colour.
#[test]
fn gemstone_array_banks_and_returns_colored_mana() {
    let mut g = main_phase();
    let array = g.add_card_to_battlefield(0, catalog::gemstone_array());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: array, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("bank");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(array).unwrap().counter_count(CounterType::Charge), 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: array, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("spend");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Blind Creeper shrinks on every spell anyone casts.
#[test]
fn blind_creeper_shrinks_on_each_cast() {
    let mut g = main_phase();
    let creeper = g.add_card_to_battlefield(0, catalog::blind_creeper());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(creeper).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Desecration Elemental eats a creature on every cast.
#[test]
fn desecration_elemental_eats_a_creature_per_cast() {
    let mut g = main_phase();
    let elem = g.add_card_to_battlefield(0, catalog::desecration_elemental());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the cheapest creature went");
    assert!(g.battlefield_find(elem).is_some());
}

/// Goblin Brawler refuses every Equipment (CR 702.6c).
#[test]
fn goblin_brawler_cant_be_equipped() {
    let mut g = main_phase();
    let brawler = g.add_card_to_battlefield(0, catalog::goblin_brawler());
    let blade = g.add_card_to_battlefield(0, catalog::banshees_blade());
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::Equip { equipment: blade, target: brawler }).is_err());
}

/// Armed Response scales with the Equipment you control.
#[test]
fn armed_response_counts_equipment() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::banshees_blade());
    g.add_card_to_battlefield(0, catalog::worldslayer());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    g.priority.player_with_priority = 0;
    let bolt = g.add_card_to_hand(0, catalog::armed_response());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(attacker)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "2 Equipment kills a 2/2");
}

/// Devour in Shadow costs you the creature's toughness in life.
#[test]
fn devour_in_shadow_charges_toughness_in_life() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::arachnoid());
    let spell = g.add_card_to_hand(0, catalog::devour_in_shadow());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(victim)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(g.players[0].life, 14, "a 2/6 costs six");
}

/// Composite Golem cashes itself in for one of each colour.
#[test]
fn composite_golem_makes_five_colors() {
    let mut g = main_phase();
    let golem = g.add_card_to_battlefield(0, catalog::composite_golem());
    g.perform_action(GameAction::ActivateAbility {
        card_id: golem, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sac");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 5);
    assert!(g.battlefield_find(golem).is_none());
}

/// Cosmic Larva eats two lands each upkeep, or itself.
#[test]
fn cosmic_larva_demands_lands() {
    let mut g = main_phase();
    let larva = g.add_card_to_battlefield(0, catalog::cosmic_larva());
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert!(g.battlefield_find(larva).is_none(), "no lands to feed it");
}

/// Fleshgrafter pitches an artifact for +2/+2.
#[test]
fn fleshgrafter_discards_an_artifact_to_pump() {
    let mut g = main_phase();
    let grafter = g.add_card_to_battlefield(0, catalog::fleshgrafter());
    g.add_card_to_hand(0, catalog::tanglebloom());
    g.perform_action(GameAction::ActivateAbility {
        card_id: grafter, ability_index: 0, target: None, additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(grafter).unwrap().power, 4);
    assert!(g.players[0].hand.is_empty());
}

/// Dawn's Reflection adds two extra mana whenever the land taps.
#[test]
fn dawns_reflection_triples_the_land() {
    let mut g = main_phase();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let aura = g.add_card_to_hand(0, catalog::dawns_reflection());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(forest)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    g.players[0].mana_pool.empty();
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// Blinkmoth Infusion's affinity makes it castable, and it untaps everything.
#[test]
fn blinkmoth_infusion_untaps_all_artifacts() {
    let mut g = main_phase();
    let rock = g.add_card_to_battlefield(0, catalog::tanglebloom());
    g.battlefield_find_mut(rock).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::blinkmoth_infusion());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(11);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("affinity for one artifact");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(rock).unwrap().tapped);
}

/// Ferocious Charge pumps and digs.
#[test]
fn ferocious_charge_pumps_and_scries() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::ferocious_charge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6);
    assert_eq!(g.players[0].library.len(), 3, "scry doesn't draw");
}

/// Ferropede strips a counter when it connects.
#[test]
fn ferropede_strips_a_counter_on_connect() {
    let mut g = main_phase();
    let pede = g.add_card_to_battlefield(0, catalog::ferropede());
    let target = g.add_card_to_battlefield(1, catalog::clockwork_dragon());
    g.battlefield_find_mut(target).unwrap().add_counters(CounterType::PlusOnePlusOne, 6);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.clear_sickness(pede);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: pede, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 5);
}

/// Eyes of the Watcher offers a {1} scry on each instant or sorcery.
#[test]
fn eyes_of_the_watcher_scries_for_one() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::eyes_of_the_watcher());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0, "the extra mana was paid");
}

/// Baton of Courage enters with sunburst counters and spends them as pumps.
#[test]
fn baton_of_courage_spends_sunburst_counters() {
    let mut g = main_phase();
    let baton = g.add_card_to_hand(0, catalog::baton_of_courage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: baton, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(baton).unwrap().counter_count(CounterType::Charge), 3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: baton, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    })
    .expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    let _ = (CardType::Artifact, Keyword::Flash);
}
