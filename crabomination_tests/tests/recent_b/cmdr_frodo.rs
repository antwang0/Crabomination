//! Commander: the Food and Fellowship precon (LTC, Frodo + Sam,
//! `decks::cmdr_frodo`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    stock_libraries(&mut g, 10);
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
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

fn tokens(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.is_token && c.definition.name == name).map(|c| c.id).collect()
}

fn fire(g: &mut GameState, step: TurnStep) {
    g.step = step;
    g.fire_step_triggers(step);
    drain_stack(g);
}

/// Sam makes a Food at the beginning of combat, and Foods' abilities cost
/// {1} less: one floating mana cracks one.
#[test]
fn sam_makes_food_and_discounts_it() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::sam_loyal_attendant());
    fire(&mut g, TurnStep::BeginCombat);
    let food = tokens(&g, 0, "Food");
    assert_eq!(food.len(), 1);
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, food[0], 0, None).expect("{1} pays for the Food");
    assert_eq!(g.players[0].life, 23);
}

/// CR 701.54 — Frodo's attack after gaining 3 life tempts you; on the second
/// temptation, with Frodo as Ring-bearer, it draws.
#[test]
fn frodo_tempts_and_draws_on_the_second_temptation() {
    for (gained, before, tempted, draws) in [(0, 0, 0, 0), (3, 0, 1, 0), (3, 1, 2, 1)] {
        let mut g = main_phase(2);
        let frodo = g.add_card_to_battlefield(0, catalog::frodo_adventurous_hobbit());
        g.clear_sickness(frodo);
        g.players[0].life_gained_this_turn = gained;
        g.players[0].ring_temptations = before;
        let hand = g.players[0].hand.len();
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
            attacker: frodo,
            target: crabomination::game::types::AttackTarget::Player(1),
        }]))
        .expect("attack");
        drain_stack(&mut g);
        assert_eq!(g.players[0].ring_temptations, tempted, "gained {gained}, tempted {before} before");
        assert_eq!(g.players[0].hand.len(), hand + draws, "gained {gained}, tempted {before} before");
    }
}

/// Prize Pig banks ribbon counters; at three it sheds them and untaps.
#[test]
fn prize_pig_untaps_on_three_ribbons() {
    let mut g = main_phase(2);
    let pig = g.add_card_to_battlefield(0, catalog::prize_pig());
    g.battlefield_find_mut(pig).unwrap().tapped = true;
    let food = g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, food, 0, None).expect("eat");
    let p = g.battlefield_find(pig).unwrap();
    assert!(!p.tapped);
    assert_eq!(p.counter_count(CounterType::Ribbon), 0);
}

/// Call for Unity: revolt at your end step adds a unity counter, and each one
/// is +1/+1 for your creatures.
#[test]
fn call_for_unity_grows_on_revolt() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::call_for_unity());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    fire(&mut g, TurnStep::End);
    assert_eq!(g.computed_permanent(bear).map(|c| c.power), Some(2), "no revolt yet");
    let food = g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, food, 0, None).expect("eat");
    fire(&mut g, TurnStep::End);
    assert_eq!(g.computed_permanent(bear).map(|c| c.power), Some(3));
}

/// Assemble the Entmoot's Treefolk are X/X for the life gained this turn,
/// tapped, with a reach counter.
#[test]
fn entmoot_treefolk_are_sized_by_life_gained() {
    let mut g = main_phase(2);
    let moot = g.add_card_to_battlefield(0, catalog::assemble_the_entmoot());
    g.players[0].life_gained_this_turn = 4;
    activate(&mut g, moot, 0, None).expect("sacrifice");
    let trees = tokens(&g, 0, "Treefolk");
    assert_eq!(trees.len(), 3);
    for t in trees {
        let c = g.computed_permanent(t).unwrap();
        assert_eq!((c.power, c.toughness), (4, 4));
        assert!(c.keywords().contains(&Keyword::Reach));
        assert!(g.battlefield_find(t).unwrap().tapped);
    }
}

/// Banquet Guests enters with twice X +1/+1 counters.
#[test]
fn banquet_guests_enters_with_twice_x() {
    let mut g = main_phase(2);
    let guests = g.add_card_to_hand(0, catalog::banquet_guests());
    cast(&mut g, guests, None, Some(3)).expect("Guests");
    assert_eq!(g.battlefield_find(guests).map(|c| c.counter_count(CounterType::PlusOnePlusOne)), Some(6));
}

/// Bilbo adds 1 to each lifegain.
#[test]
fn bilbo_adds_one_to_lifegain() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::bilbo_birthday_celebrant());
    let food = g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, food, 0, None).expect("eat");
    assert_eq!(g.players[0].life, 24);
}

/// Hithlain Rope passes to the player on your right after use, and can't be
/// sacrificed.
#[test]
fn hithlain_rope_passes_right() {
    let mut g = main_phase(4);
    let rope = g.add_card_to_battlefield(0, catalog::hithlain_rope());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, rope, 1, None).expect("draw");
    let owner_right = g.battlefield_find(rope).map(|c| c.controller);
    assert!(owner_right.is_some_and(|c| c != 0), "it changed hands");
    assert!(g.battlefield_find(rope).is_some_and(|c| c.has_keyword(&Keyword::CantBeSacrificed)));
}

/// Feasting Hobbit devours Foods for three counters each.
#[test]
fn feasting_hobbit_devours_food() {
    let mut g = main_phase(2);
    for _ in 0..2 {
        g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    }
    let hobbit = g.add_card_to_hand(0, catalog::feasting_hobbit());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    cast(&mut g, hobbit, None, None).expect("Hobbit");
    assert_eq!(g.battlefield_find(hobbit).map(|c| c.counter_count(CounterType::PlusOnePlusOne)), Some(6));
    assert!(tokens(&g, 0, "Food").is_empty());
}

/// Butterbur makes a Food at your end step only when you have none.
#[test]
fn butterbur_refills_an_empty_pantry() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::butterbur_bree_innkeeper());
    fire(&mut g, TurnStep::End);
    fire(&mut g, TurnStep::End);
    assert_eq!(tokens(&g, 0, "Food").len(), 1);
}

/// Farmer Cotton: X Halflings and X Foods.
#[test]
fn farmer_cotton_makes_x_of_each() {
    let mut g = main_phase(2);
    let cotton = g.add_card_to_hand(0, catalog::farmer_cotton());
    cast(&mut g, cotton, None, Some(2)).expect("Cotton");
    assert_eq!((tokens(&g, 0, "Halfling").len(), tokens(&g, 0, "Food").len()), (2, 2));
}

/// Lobelia exiles each opponent's top card; its first mode plays one free.
#[test]
fn lobelia_exiles_each_opponents_top_card() {
    let mut g = main_phase(3);
    let lobelia = g.add_card_to_hand(0, catalog::lobelia_defender_of_bag_end());
    let tops: Vec<CardId> = (1..3).map(|p| g.players[p].library[0].id).collect();
    cast(&mut g, lobelia, None, None).expect("Lobelia");
    assert!(tops.iter().all(|t| g.exile.iter().any(|c| c.id == *t && c.exiled_with == Some(lobelia))));
}

/// Rapacious Guest leaving drains an opponent for its power; a Food
/// sacrificed grows it first.
#[test]
fn rapacious_guest_leaves_for_its_power() {
    let mut g = main_phase(2);
    let guest = g.add_card_to_battlefield(0, catalog::rapacious_guest());
    let food = g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, food, 0, None).expect("eat");
    assert_eq!(g.battlefield_find(guest).map(|c| c.counter_count(CounterType::PlusOnePlusOne)), Some(1));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(guest)), None).expect("Bolt");
    assert_eq!(g.players[1].life, 17, "3 power when it left");
}

/// Treebeard puts the life gained as counters on a Halfling or Treefolk.
#[test]
fn treebeard_turns_lifegain_into_counters() {
    let mut g = main_phase(2);
    let tree = g.add_card_to_hand(0, catalog::treebeard_gracious_host());
    cast(&mut g, tree, None, None).expect("Treebeard");
    let food = tokens(&g, 0, "Food")[0];
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, food, 0, None).expect("eat");
    assert_eq!(g.battlefield_find(tree).map(|c| c.counter_count(CounterType::PlusOnePlusOne)), Some(3));
}

/// A prompting seat's as-enters ask on a creature a mass reanimation returns
/// (Feasting Hobbit's devour under Living Death) is answered on the spot: the
/// enclosing resolution can't replay, so a parked ask used to leak its answer
/// and the devour never happened (a strict 6-seat pod, seed 10481).
#[test]
fn an_as_enters_ask_inside_living_death_is_answered_in_place() {
    use crabomination::decision::Decision;
    let mut g = main_phase(2);
    g.players[0].wants_ui = true;
    g.add_card_to_graveyard(0, catalog::feasting_hobbit());
    g.add_token_to_battlefield(0, &crabomination::game::effects::food_token());
    let ld = g.add_card_to_hand(0, catalog::living_death());
    cast(&mut g, ld, None, None).expect("Living Death");
    for _ in 0..10 {
        let Some(pending) = g.pending_decision.as_ref() else { break };
        let answer = match &pending.decision {
            Decision::ChooseAmount { max, .. } => DecisionAnswer::Amount(*max),
            _ => DecisionAnswer::Bool(true),
        };
        g.submit_decision(answer).expect("answer");
        drain_stack(&mut g);
    }
    assert!(g.pending_decision.is_none());
    let hobbit = g.battlefield.iter().find(|c| c.definition.name == "Feasting Hobbit").expect("returned");
    assert_eq!(hobbit.counter_count(CounterType::PlusOnePlusOne), 3, "it devoured the Food");
}
