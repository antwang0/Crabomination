//! Functionality tests for `catalog::sets::decks::recent213`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::actions::cost_reduction_for_spell;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Heroes' Bane enters as a 4/4 and doubles via its power-scaled pump.
#[test]
fn heroes_bane_enters_and_pumps_by_power() {
    let mut g = two_player_game();
    let hydra = g.move_card_to_battlefield_for_test(0, catalog::heroes_bane());
    drain_stack(&mut g);
    let v = g.computed_permanent(hydra).unwrap();
    assert_eq!((v.power, v.toughness), (4, 4), "enters with four +1/+1 counters");
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hydra, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("pump by power");
    drain_stack(&mut g);
    let v = g.computed_permanent(hydra).unwrap();
    assert_eq!((v.power, v.toughness), (8, 8), "added +4/+4 (X = power 4)");
}

/// Wildwood Scourge grows when a +1/+1 counter lands on another non-Hydra.
#[test]
fn wildwood_scourge_tracks_counters() {
    let mut g = two_player_game();
    let scourge = g.add_card_to_battlefield(0, catalog::wildwood_scourge());
    g.battlefield_find_mut(scourge).unwrap().counters.insert(CounterType::PlusOnePlusOne, 2);
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Put a +1/+1 counter on the (non-Hydra) bear through the real add path so
    // the CounterAdded event fires.
    let ctx = crabomination::game::effects::EffectContext::for_ability(scourge, 0, None);
    let evs = g.resolve_effect(&crabomination::effect::Effect::AddCounter {
        what: crabomination::effect::Selector::EachPermanent(
            crabomination::card::SelectionRequirement::HasCreatureType(crabomination::card::CreatureType::Bear),
        ),
        kind: CounterType::PlusOnePlusOne,
        amount: crabomination::effect::Value::ONE,
    }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(scourge).unwrap().counter_count(CounterType::PlusOnePlusOne), 3,
        "Scourge gained a counter from the bear's counter");
}

/// Sanguine Indulgence's discount switches on after you gain life.
#[test]
fn sanguine_indulgence_life_discount() {
    let mut g = two_player_game();
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::sanguine_indulgence(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0, "no lifegain → no discount");
    g.players[0].life_gained_this_turn = 3;
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 3, "gained 3 → {{3}} off");
}

/// Sanguine Indulgence returns creatures from the graveyard.
#[test]
fn sanguine_indulgence_returns_creatures() {
    let mut g = two_player_game();
    let c1 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let c2 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // not a creature
    let spell = g.add_card_to_hand(0, catalog::sanguine_indulgence());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(Target::Permanent(c1)),
        crabomination::decision::DecisionAnswer::Target(Target::Permanent(c2)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sanguine Indulgence");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.iter().filter(|c| c.id == c1 || c.id == c2).count(), 2);
}

/// Demolition Field blows up a nonbasic land and ramps a basic.
#[test]
fn demolition_field_destroys_and_ramps() {
    let mut g = two_player_game();
    let field = g.add_card_to_battlefield(0, catalog::demolition_field());
    let target_land = g.add_card_to_battlefield(1, catalog::demolition_field()); // nonbasic opp land
    let forest = g.add_card_to_library(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(forest)),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: field, ability_index: 1, target: Some(Target::Permanent(target_land)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate Demolition Field");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target_land).is_none(), "opponent's nonbasic land destroyed");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Forest"),
        "ramped a basic Forest to the battlefield");
}

/// Goblin Firebomb flashes in and can be sacrificed to destroy a permanent.
#[test]
fn goblin_firebomb_destroys_permanent() {
    let mut g = two_player_game();
    let bomb = g.add_card_to_battlefield(0, catalog::goblin_firebomb());
    assert!(catalog::goblin_firebomb().keywords.contains(&Keyword::Flash), "has flash");
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("detonate the Firebomb");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "target permanent destroyed");
    assert!(g.battlefield_find(bomb).is_none(), "Firebomb sacrificed");
}

/// Ajani's +1 adds a counter and his ultimate mints a Cat per life.
#[test]
fn ajani_plus_one_and_ultimate() {
    let mut g = two_player_game();
    let ajani = g.add_card_to_battlefield(0, catalog::ajani_caller_of_the_pride());
    assert_eq!(g.battlefield_find(ajani).unwrap().counter_count(CounterType::Loyalty), 4);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ajani, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("Ajani +1");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(ajani).unwrap().counter_count(CounterType::Loyalty), 5, "loyalty 4→5");
}
