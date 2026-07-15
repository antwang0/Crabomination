//! Functionality tests for `catalog::sets::decks::recent51`.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::two_player_game;
use crabomination::game::*;
use crabomination::mana::Color;

#[test]
fn standard_bearer_draws_per_creature_that_died() {
    let mut g = two_player_game();
    g.players[0].creatures_died_this_turn = 2;
    for _ in 0..3 { g.add_card_to_library(0, catalog::swamp()); }
    let lib0 = g.players[0].library.len();
    let sb = g.add_card_to_battlefield(0, catalog::lilianas_standard_bearer());
    g.fire_self_etb_triggers(sb, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib0 - 2, "drew two for two dead creatures");
}

#[test]
fn skullport_merchant_makes_a_treasure_and_loots() {
    let mut g = two_player_game();
    let sm = g.add_card_to_battlefield(0, catalog::skullport_merchant());
    g.fire_self_etb_triggers(sm, 0);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(),
        1,
        "ETB Treasure"
    );
    // Sac the Treasure to draw.
    g.add_card_to_library(0, catalog::swamp());
    let lib0 = g.players[0].library.len();
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sm, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("loot ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib0 - 1, "drew off the sacrifice");
}

#[test]
fn bone_picker_is_cheap_after_a_death() {
    let mut g = two_player_game();
    g.players[0].creatures_died_this_turn = 1;
    let bp = g.add_card_to_hand(0, catalog::bone_picker());
    g.players[0].mana_pool.add(Color::Black, 1); // {3}{B} - {3} = {B}
    g.perform_action(GameAction::CastSpell {
        card_id: bp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bone Picker castable for {B} after a death");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Bone Picker"));
}

#[test]
fn driver_of_the_dead_reanimates_a_small_creature() {
    let mut g = two_player_game();
    let small = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let driver = g.add_card_to_battlefield(0, catalog::driver_of_the_dead());
    let evs = g.remove_to_graveyard_with_triggers(driver);
    let mut evs = evs;
    evs.push(GameEvent::CreatureDied { card_id: driver });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_some(), "small creature back on the battlefield");
}

#[test]
fn gixian_infiltrator_grows_on_sacrifice() {
    let mut g = two_player_game();
    let gix = g.add_card_to_battlefield(0, catalog::gixian_infiltrator());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.sacrifice_one(fodder, 0, &mut vec![]);
    let evs = vec![GameEvent::PermanentSacrificed { card_id: fodder, who: 0 }];
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(gix).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "a +1/+1 counter for the sacrifice"
    );
}

#[test]
fn hunger_of_the_howlpack_is_morbid() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // No death yet → +1/+1.
    let ctx = EffectContext::for_spell(0, Some(Target::Permanent(bear)), 0, 0);
    g.resolve_effect(&catalog::hunger_of_the_howlpack().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counters[&CounterType::PlusOnePlusOne], 1);
    // After a death → +3 more (4 total).
    g.players[0].creatures_died_this_turn = 1;
    g.resolve_effect(&catalog::hunger_of_the_howlpack().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counters[&CounterType::PlusOnePlusOne], 4);
}
