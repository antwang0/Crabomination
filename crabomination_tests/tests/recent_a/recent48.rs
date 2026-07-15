//! Functionality tests for `catalog::sets::decks::recent48`.

use crabomination::card::{CounterType, Keyword, TokenDefinition};
use crabomination::catalog;
use crabomination::game::effects::{EffectContext, EntityRef};
use crabomination::game::two_player_game;
use crabomination::game::*;
use crabomination::mana::Color;

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id)
        .and_then(|c| c.counters.get(&CounterType::PlusOnePlusOne).copied())
        .unwrap_or(0)
}

fn flyer() -> TokenDefinition {
    TokenDefinition {
        name: "Bird".into(),
        power: 2,
        toughness: 2,
        card_types: vec![crabomination::card::CardType::Creature],
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

#[test]
fn predator_ooze_grows_on_attack() {
    let mut g = two_player_game();
    let ooze = g.add_card_to_battlefield(0, catalog::predator_ooze());
    let ctx = EffectContext::for_trigger(ooze, 0, None, 0);
    let trig = catalog::predator_ooze().triggered_abilities[0].effect.clone();
    g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!(counters(&g, ooze), 1, "attack adds a +1/+1 counter");
}

#[test]
fn predator_ooze_grows_when_its_victim_dies() {
    let mut g = two_player_game();
    let ooze = g.add_card_to_battlefield(0, catalog::predator_ooze());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Record that the Ooze damaged the victim this turn.
    g.battlefield_find_mut(victim).unwrap().damaged_by_this_turn.push(ooze);
    let evs = g.remove_to_graveyard_with_triggers(victim);
    let mut evs = evs;
    evs.push(GameEvent::CreatureDied { card_id: victim });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(counters(&g, ooze), 1, "a damaged creature dying adds a counter");
}

#[test]
fn hornet_nest_spawns_insects_when_damaged() {
    let mut g = two_player_game();
    let nest = g.add_card_to_battlefield(0, catalog::hornet_nest());
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(nest), 1, None, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let insects = g.battlefield.iter().filter(|c| c.definition.name == "Insect" && c.controller == 0).count();
    assert_eq!(insects, 1, "one Insect per point of damage");
}

#[test]
fn aerie_ouphes_sacs_to_shoot_a_flier() {
    let mut g = two_player_game();
    let ouphe = g.add_card_to_battlefield(0, catalog::aerie_ouphes());
    let bird = g.add_token_to_battlefield(1, &flyer());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ouphe, ability_index: 0, target: Some(Target::Permanent(bird)),
        additional_targets: vec![], x_value: None,
    }).expect("sac ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bird).is_none(), "the flier took 3 and died");
}

#[test]
fn walking_atlas_drops_a_land() {
    let mut g = two_player_game();
    let atlas = g.add_card_to_battlefield(0, catalog::walking_atlas());
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.clear_sickness(atlas);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Cards(vec![forest]),
    ]));
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: atlas, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for a land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest).is_some(), "land entered from hand");
}

#[test]
fn rishkar_counters_two_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rish = g.add_card_to_battlefield(0, catalog::rishkar_peema_renegade());
    g.fire_self_etb_triggers(rish, 0);
    drain_stack(&mut g);
    assert_eq!(counters(&g, a) + counters(&g, b), 2, "two counters distributed");
}

#[test]
fn rishkar_grants_mana_to_counter_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    g.add_card_to_battlefield(0, catalog::rishkar_peema_renegade());
    g.clear_sickness(bear);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("granted mana ability");
    assert!(g.players[0].mana_pool.amount(Color::Green) >= 1, "tapped the counter-bearing bear for green");
}

#[test]
fn gnarlid_colony_kicked_enters_with_counters() {
    let mut g = two_player_game();
    let gn = g.add_card_to_hand(0, catalog::gnarlid_colony());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: gn, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert_eq!(counters(&g, gn), 2, "kicked → two +1/+1 counters");
    let cp = g.computed_permanent(gn).unwrap();
    assert!(cp.keywords.contains(&Keyword::Trample), "counter-bearing creature has trample");
}
