//! Functionality tests for `catalog::sets::decks::recent34` — the Zendikar
//! quest-counter cycle plus a few long-missing staples.

use crabomination::card::{CardType, CounterType, CreatureType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::mana::Color;
use crabomination::game::two_player_game;
use crabomination::game::*;

#[test]
fn quest_for_the_goblin_lord_anthem_at_five_counters() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::quest_for_the_goblin_lord());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    // Below threshold: no bonus.
    g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 4);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "no anthem under five counters");
    g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 1);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0 at five counters");
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 2, "toughness unchanged");
}

#[test]
fn quest_for_the_goblin_lord_counts_dying_creature_is_not_it() {
    // Sanity: the accrual trigger is keyed on Goblin ETB, so an unrelated
    // death leaves the counter count at zero.
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::quest_for_the_goblin_lord());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(quest).unwrap().counters.get(&CounterType::Quest).copied().unwrap_or(0),
        0
    );
}

#[test]
fn quest_for_the_gravelord_accrues_on_death_and_makes_zombie() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::quest_for_the_gravelord());
    // A creature dies → a quest counter.
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(quest).unwrap().counters.get(&CounterType::Quest).copied().unwrap_or(0),
        1,
        "quest counter on a creature dying"
    );
    // Top up to three and activate the sacrifice ability.
    g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: quest, ability_index: 0, target: None, additional_targets: Vec::new(),
        x_value: None,
    }).expect("remove 3 quest counters + sacrifice");
    drain_stack(&mut g);
    assert!(g.battlefield_find(quest).is_none(), "quest sacrificed");
    let zombie = g.battlefield.iter().find(|c| c.definition.name == "Zombie Giant")
        .expect("5/5 Zombie Giant minted");
    assert_eq!((zombie.power(), zombie.toughness()), (5, 5));
    assert!(zombie.definition.subtypes.creature_types.contains(&CreatureType::Zombie));
}

#[test]
fn quest_for_the_gemblades_drops_four_counters_on_target() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::quest_for_the_gemblades());
    g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 1);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: quest, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("remove a quest counter + sacrifice");
    drain_stack(&mut g);
    assert!(g.battlefield_find(quest).is_none(), "quest sacrificed");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6, "four +1/+1 counters → 6/6");
}

#[test]
fn quest_for_ancient_secrets_shuffles_graveyard() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::quest_for_ancient_secrets());
    g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 5);
    for _ in 0..4 { g.add_card_to_graveyard(0, catalog::island()); }
    let gy_before = g.players[0].graveyard.len();
    assert!(gy_before >= 4);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: quest, ability_index: 0, target: Some(Target::Player(0)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("remove 5 quest counters + sacrifice");
    drain_stack(&mut g);
    // The quest itself is sacrificed (so it's now in the graveyard), but the
    // four islands shuffled into the library.
    assert!(g.players[0].library.len() >= 4, "graveyard shuffled into library");
}

#[test]
fn quest_for_the_holy_relic_tutors_an_equipment_to_play() {
    let mut g = two_player_game();
    let quest = g.add_card_to_battlefield(0, catalog::quest_for_the_holy_relic());
    g.battlefield_find_mut(quest).unwrap().add_counters(CounterType::Quest, 5);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let equip = g.add_card_to_library(0, catalog::bonesplitter()); // an Equipment to find
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(equip))]));
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: quest, ability_index: 0, target: None, additional_targets: Vec::new(),
        x_value: None,
    }).expect("remove 5 quest counters + sacrifice");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(equip).expect("Equipment fetched onto the battlefield").attached_to,
        Some(bear),
        "fetched Equipment enters attached to your creature"
    );
}

#[test]
fn magebane_lizard_burns_the_caster_per_noncreature_spell() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.add_card_to_battlefield(1, catalog::magebane_lizard());
    let opt = g.add_card_to_hand(0, catalog::opt());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: opt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Opt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "1 noncreature spell this turn → 1 damage to caster");
}

#[test]
fn atog_sacrifices_an_artifact_for_plus_two() {
    let mut g = two_player_game();
    let atog = g.add_card_to_battlefield(0, catalog::atog());
    let art = g.add_card_to_battlefield(0, catalog::ornithopter()); // an artifact
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: atog, ability_index: 0, target: None, additional_targets: Vec::new(),
        x_value: None,
    }).expect("Sacrifice an artifact: +2/+2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact sacrificed");
    assert_eq!(g.computed_permanent(atog).unwrap().power, 3, "1 + 2 = 3");
}

#[test]
fn origin_spellbomb_sacrifices_for_a_myr() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::origin_spellbomb());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb, ability_index: 0, target: None, additional_targets: Vec::new(),
        x_value: None,
    }).expect("{1}, {T}, Sacrifice: make a Myr");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bomb).is_none(), "spellbomb sacrificed");
    let myr = g.battlefield.iter().find(|c| c.definition.name == "Myr").expect("Myr minted");
    assert!(myr.definition.card_types.contains(&CardType::Artifact));
}

#[test]
fn land_tax_fetches_basics_when_behind_on_lands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::land_tax());
    // Opponent controls more lands than us (we control none).
    for _ in 0..2 { g.add_card_to_battlefield(1, catalog::island()); }
    let mut basics = Vec::new();
    for _ in 0..3 { basics.push(g.add_card_to_library(0, catalog::plains())); }
    g.decider = Box::new(ScriptedDecider::new(
        basics.iter().map(|&id| DecisionAnswer::Search(Some(id))).collect::<Vec<_>>(),
    ));
    let hand_before = g.players[0].hand.len();
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 3, "tutored three basics to hand");
}
