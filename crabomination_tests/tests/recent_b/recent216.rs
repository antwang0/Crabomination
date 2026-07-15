//! Functionality tests for `catalog::sets::decks::recent216`.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Teapot Slinger deals 2 to each opponent when you expend 4.
#[test]
fn teapot_slinger_expend_pings() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::teapot_slinger());
    let moose = g.add_card_to_hand(0, catalog::galewind_moose()); // {4}{G}{G} crosses expend 4
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast 6-mana spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "expend 4 → 2 damage to the opponent");
}

/// Byway Barterer discards your hand and draws two on expend 4.
#[test]
fn byway_barterer_expend_wheels_hand() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::byway_barterer());
    let moose = g.add_card_to_hand(0, catalog::galewind_moose());
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest()); // hand cards to discard
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast 6-mana spell");
    drain_stack(&mut g);
    // The 2 forests in hand were discarded (moose left on cast), then drew two.
    assert_eq!(g.players[0].hand.len(), 2, "discarded whole hand, drew two");
    assert!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Forest").count() >= 2,
        "discarded forests hit the graveyard");
}

/// Wick's Patrol mills three, then hands a -X/-X to an opponent's creature where
/// X is the greatest mana value among cards in your graveyard.
#[test]
fn wicks_patrol_debuffs_by_greatest_gy_mv() {
    let mut g = two_player_game();
    // Seed the graveyard: a 6-mana card ({4}{G}{G}) sets X = 6.
    g.add_card_to_graveyard(0, catalog::galewind_moose()); // MV 6
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let patrol = g.add_card_to_hand(0, catalog::wicks_patrol());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: patrol, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Wick's Patrol");
    drain_stack(&mut g);
    // Moose MV 6 is the greatest → -6/-6 kills the 2/2.
    assert!(g.battlefield_find(target).is_none(), "2/2 dies to -6/-6");
}

/// Maha sets opponents' creatures to base toughness 1 (power untouched, and
/// +1/+1 counters stack on top per CR 613); your own creatures are unaffected.
#[test]
fn maha_sets_opponent_base_toughness_to_one() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::maha_its_feathers_night());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, yours
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 1), "opponent's 2/2 → 2/1");
    let cm = g.computed_permanent(mine).unwrap();
    assert_eq!((cm.power, cm.toughness), (2, 2), "your own creature is unaffected");
    // A +1/+1 counter stacks on the reduced base (1 → 2 toughness, 2 → 3 power).
    g.battlefield_find_mut(foe).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let cp2 = g.computed_permanent(foe).unwrap();
    assert_eq!((cp2.power, cp2.toughness), (3, 2), "counter stacks on base toughness 1");
}
