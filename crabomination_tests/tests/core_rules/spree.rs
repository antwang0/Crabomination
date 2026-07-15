//! Functionality tests for the Spree spells (CR 702.172) in
//! `catalog::sets::decks::spree`, exercising `GameAction::CastSpellSpree`.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

fn spree(
    card_id: CardId,
    modes: Vec<u8>,
    target: Option<Target>,
    additional: Vec<Target>,
) -> GameAction {
    GameAction::CastSpellSpree {
        card_id,
        spree_modes: modes,
        target,
        additional_targets: additional,
        x_value: None,
    }
}

#[test]
fn explosive_derailment_both_modes() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let artifact = g.add_card_to_battlefield(1, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::explosive_derailment());
    // {R} base + {2} + {2} = {R} and {4} generic.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(spree(
        id,
        vec![0, 1],
        Some(Target::Permanent(creature)),
        vec![Target::Permanent(artifact)],
    ))
    .expect("cast both modes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(creature).is_none(), "4 damage killed the 4/4");
    assert!(g.battlefield_find(artifact).is_none(), "artifact destroyed");
}

#[test]
fn explosive_derailment_single_mode_pays_only_that_cost() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::explosive_derailment());
    // Only mode 0: {R} + {2}. Two colorless is enough; leftover none.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(spree(id, vec![0], Some(Target::Permanent(creature)), vec![]))
        .expect("cast one mode");
    drain_stack(&mut g);
    assert!(g.battlefield_find(creature).is_none(), "4 damage killed it");
}

#[test]
fn spree_requires_at_least_one_mode() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::explosive_derailment());
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(g.perform_action(spree(id, vec![], None, vec![])).is_err());
}

#[test]
fn spree_insufficient_mana_for_chosen_modes_fails() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(1, catalog::serra_angel());
    let artifact = g.add_card_to_battlefield(1, catalog::the_everflowing_well());
    let id = g.add_card_to_hand(0, catalog::explosive_derailment());
    // Only {R}{2} available but both modes need {R}{4}.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g
        .perform_action(spree(
            id,
            vec![0, 1],
            Some(Target::Permanent(creature)),
            vec![Target::Permanent(artifact)],
        ))
        .is_err());
    // The spell stayed in hand (atomic rollback).
    assert!(g.players[0].hand.iter().any(|c| c.id == id));
}

#[test]
fn insatiable_avarice_draw_and_lose_life() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::serra_angel());
    }
    let id = g.add_card_to_hand(0, catalog::insatiable_avarice());
    // Mode 1 only: {B} base + {B}{B} = three black.
    g.players[0].mana_pool.add(Color::Black, 3);
    let (life, hand) = (g.players[1].life, g.players[1].hand.len());
    g.perform_action(spree(id, vec![1], Some(Target::Player(1)), vec![]))
        .expect("cast draw-3-lose-3 mode");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand + 3, "target drew three");
    assert_eq!(g.players[1].life, life - 3, "target lost 3 life");
}

#[test]
fn rustler_rampage_untaps_target_players_creatures() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.battlefield_find_mut(c).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::rustler_rampage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(spree(id, vec![0], Some(Target::Player(0)), vec![]))
        .expect("cast untap mode");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(c).unwrap().tapped, "creature untapped");
}

#[test]
fn rustler_rampage_grants_double_strike() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::rustler_rampage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(spree(id, vec![1], Some(Target::Permanent(c)), vec![]))
        .expect("cast double strike mode");
    drain_stack(&mut g);
    assert!(g
        .computed_permanent(c)
        .unwrap()
        .keywords
        .contains(&crabomination::card::Keyword::DoubleStrike));
}

#[test]
fn requisition_raid_counters_each_creature_of_target_player() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::serra_angel());
    let b = g.add_card_to_battlefield(1, catalog::barony_vampire());
    let id = g.add_card_to_hand(0, catalog::requisition_raid());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(spree(id, vec![2], Some(Target::Player(1)), vec![]))
        .expect("cast counter mode");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn caught_in_the_crossfire_hits_only_outlaws() {
    let mut g = two_player_game();
    let outlaw = g.add_card_to_battlefield(1, catalog::guul_draz_vampire()); // Vampire Rogue = outlaw
    let plain = g.add_card_to_battlefield(1, catalog::barony_vampire()); // Vampire = not outlaw
    let id = g.add_card_to_hand(0, catalog::caught_in_the_crossfire());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(spree(id, vec![0], None, vec![])).expect("outlaw mode");
    drain_stack(&mut g);
    assert!(g.battlefield_find(outlaw).is_none(), "outlaw took 2 and died (1/1)");
    assert_eq!(g.battlefield_find(plain).unwrap().damage, 0, "non-outlaw untouched");
}

#[test]
fn rush_of_dread_loses_half_life() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    let id = g.add_card_to_hand(0, catalog::rush_of_dread());
    // Base {1}{B}{B} + mode 2 {2}.
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(spree(id, vec![2], Some(Target::Player(1)), vec![]))
        .expect("cast lose-half mode");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 10, "lost half of 20, rounded up");
}

#[test]
fn phantom_interference_makes_a_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::phantom_interference());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(spree(id, vec![0], None, vec![])).expect("token mode");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Spirit").collect();
    assert_eq!(spirits.len(), 1);
    assert_eq!((spirits[0].definition.power, spirits[0].definition.toughness), (2, 2));
}

#[test]
fn three_steps_ahead_copies_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::three_steps_ahead());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(spree(id, vec![1], Some(Target::Permanent(bear)), vec![]))
        .expect("cast copy mode");
    drain_stack(&mut g);
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "original + token copy");
}

#[test]
fn three_steps_ahead_draws_and_discards() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::three_steps_ahead());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib = g.players[0].library.len();
    g.perform_action(spree(id, vec![2], None, vec![])).expect("cast loot mode");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 2, "drew two");
}

#[test]
fn spreeable_affordance_surfaces_castable_spree_card() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::explosive_derailment());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // No mana → not offered (can't afford even the cheapest mode).
    assert!(!g.compute_hand_affordances(0).spreeable.contains(&id));
    // {R} + a mode's {2} → offered.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.compute_hand_affordances(0).spreeable.contains(&id));
}

#[test]
fn dance_of_the_tumbleweeds_token_scales_with_lands() {
    let mut g = two_player_game();
    // Give the caster three lands so the Elemental is 3/3.
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::dance_of_the_tumbleweeds());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(spree(id, vec![1], None, vec![])).expect("token mode");
    drain_stack(&mut g);
    let elem = g.battlefield.iter().find(|c| c.definition.name == "Elemental").expect("token");
    let cp = g.compute_battlefield();
    let cp = cp.iter().find(|c| c.id == elem.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "X = lands you control");
}

#[test]
fn final_showdown_wrath_mode_destroys_all_creatures() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::final_showdown());
    g.players[0].mana_pool.add(Color::White, 3); // {W} base + {W}{W} in the mode
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(spree(id, vec![2], None, vec![])).expect("wrath mode");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "all creatures destroyed");
}

#[test]
fn jailbreak_scheme_counter_and_unblockable() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::jailbreak_scheme());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(spree(id, vec![0], Some(Target::Permanent(bear)), vec![]))
        .expect("counter/unblockable mode");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "+1/+1 counter placed",
    );
    assert!(
        g.compute_battlefield().iter().find(|c| c.id == bear).unwrap()
            .keywords.contains(&crabomination::card::Keyword::Unblockable),
        "granted unblockable",
    );
}
