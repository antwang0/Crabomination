//! Functionality tests for `catalog::sets::decks::recent106` — modern
//! archetype gaps (Eggs/Stations, Melira combo, Iona, Reshape).

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::types::{Target, TurnStep};
use crate::game::*;

/// Grinding Station mills three off a sacrificed artifact and untaps on an
/// artifact ETB.
#[test]
fn grinding_station_mills_and_untaps() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let station = g.add_card_to_battlefield(0, catalog::grinding_station());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: station, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("grind");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 3, "milled three");
    assert!(g.battlefield_find(station).unwrap().tapped);
    // A new artifact untaps it (may → scripted yes).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let evs = vec![GameEvent::PermanentEntered {
        card_id: g.add_card_to_battlefield(0, catalog::mind_stone()),
    }];
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(station).unwrap().tapped, "untapped off the ETB");
}

/// Anafenza bolsters when another nontoken creature enters.
#[test]
fn anafenza_bolsters_on_ally_etb() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let ana = g.add_card_to_battlefield(0, catalog::anafenza_kin_tree_spirit());
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the elf");
    drain_stack(&mut g);
    // Bolster 1: the 1/1 entrant is the least-tough creature.
    assert_eq!(g.battlefield_find(elf).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(ana).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Slitherhead scavenges from the graveyard for {0}.
#[test]
fn slitherhead_scavenges_for_free() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::slitherhead());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: dead, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("scavenge");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.exile.iter().any(|c| c.id == dead), "scavenged card exiled");
}

/// Iona names a color; opponents can't cast spells of it.
#[test]
fn iona_locks_the_chosen_color() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Color(crate::mana::Color::Red),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::iona_shield_of_emeria());
    drain_stack(&mut g);
    // Opponent can't cast a red spell…
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "red spell locked");
    // …but a nonred one resolves.
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("green spell fine");
}

/// Thopter Assembly bounces itself for five Thopters at a lonely upkeep.
#[test]
fn thopter_assembly_disassembles() {
    let mut g = two_player_game();
    let asm = g.add_card_to_battlefield(0, catalog::thopter_assembly());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == asm), "bounced to hand");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count(), 5);
}

/// Reshape sacrifices an artifact and fetches one with MV ≤ X.
#[test]
fn reshape_swaps_an_artifact() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::sol_ring());
    let target = g.add_card_to_library(0, catalog::mind_stone()); // MV 2
    let re = g.add_card_to_hand(0, catalog::reshape());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: re, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("Reshape for X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "artifact sacrificed");
    assert!(g.battlefield_find(target).is_some(), "fetched onto the battlefield");
}

/// Wild Cantor sacrifices for one mana of any color.
#[test]
fn wild_cantor_ramps() {
    let mut g = two_player_game();
    let cantor = g.add_card_to_battlefield(0, catalog::wild_cantor());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cantor, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for mana");
    assert!(g.battlefield_find(cantor).is_none(), "sacrificed");
    assert!(g.players[0].mana_pool.total() >= 1, "one mana added");
}

/// Melira: no poison for you, no -1/-1 counters on your creatures, and
/// opponents' creatures lose infect.
#[test]
fn melira_shuts_off_infect() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::melira_sylvok_outcast());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Poison lock.
    let mut evs = Vec::new();
    g.add_poison(0, 3, &mut evs);
    assert_eq!(g.players[0].poison_counters, 0, "no poison counters for you");
    // -1/-1 lock.
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 1, None);
    g.resolve_effect(&crate::effect::Effect::AddCounter {
        what: crate::effect::Selector::EachPermanent(
            crate::card::SelectionRequirement::Creature,
        ),
        kind: CounterType::MinusOneMinusOne,
        amount: crate::effect::Value::Const(2),
    }, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 0,
        "your creature dodges the -1/-1 counters");
    // Opponent's infect creature loses the keyword.
    let carrier = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(carrier) {
        let def = std::sync::Arc::make_mut(&mut c.definition);
        def.keywords.push(Keyword::Infect);
    }
    assert!(!g.computed_permanent(carrier).unwrap().keywords.contains(&Keyword::Infect),
        "opponent's creature loses infect");
}
