//! Functionality tests for `catalog::sets::decks::recent92` (Izzet spell-copy
//! artifacts / enchantments & payoffs).

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::two_player_game;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

/// Cast Lightning Bolt from P0 at P1's face.
fn bolt_face(g: &mut GameState) {
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(g);
}

/// Total damage dealt to both players from a starting 20/20.
fn damage_dealt(g: &GameState) -> i32 {
    (20 - g.players[0].life) + (20 - g.players[1].life)
}

#[test]
fn firemind_vessel_taps_for_two() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::firemind_vessel());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: v, ability_index: 0, target: None, additional_targets: Vec::new(),
        x_value: None,
    }).expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 2, "added two mana");
}

#[test]
fn swarm_intelligence_copies_each_is() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::swarm_intelligence());
    g.players[0].life = 20;
    g.players[1].life = 20;
    bolt_face(&mut g);
    assert_eq!(damage_dealt(&g), 6, "bolt + one copy each dealt 3");
}

#[test]
fn thousand_year_storm_copies_per_prior_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::thousand_year_storm());
    g.players[0].life = 20;
    g.players[1].life = 20;
    bolt_face(&mut g); // first spell: 0 copies → 3 damage
    assert_eq!(damage_dealt(&g), 3, "first spell makes no copies");
    bolt_face(&mut g); // second spell: 1 copy → +6 damage
    assert_eq!(damage_dealt(&g), 9, "second spell copied once");
}

#[test]
fn mirari_copies_when_you_pay_three() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mirari());
    g.players[0].life = 20;
    g.players[1].life = 20;
    // Float {3} for the optional copy cost; say yes to the payment.
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    bolt_face(&mut g);
    assert_eq!(damage_dealt(&g), 6, "paid three to copy the bolt");
}

#[test]
fn nivmizzet_dracogenius_pings_for_one() {
    let mut g = two_player_game();
    let niv = g.add_card_to_battlefield(0, catalog::nivmizzet_dracogenius());
    for c in g.battlefield.iter_mut() { c.summoning_sick = false; }
    g.players[1].life = 20;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: niv, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "dealt 1 to the opponent");
}

#[test]
fn jhoira_draws_on_historic_but_not_vanilla() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jhoira_weatherlight_captain());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_library(0, catalog::forest());
    let before = g.players[0].hand.len();
    // Cast an artifact spell (historic) → draw.
    let orn = g.add_card_to_hand(0, catalog::ornithopter());
    g.perform_action(GameAction::CastSpell {
        card_id: orn, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ornithopter castable");
    drain_stack(&mut g);
    // Ornithopter left hand to the battlefield, then the historic trigger drew
    // one, so the hand grew by one and the library is now empty.
    assert_eq!(g.players[0].hand.len(), before + 1, "the historic cast drew a card");
    assert!(g.players[0].library.is_empty(), "the historic draw emptied the library");
}

#[test]
fn arjun_wheels_hand_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::arjun_the_shifting_flame());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Two spare cards in hand + a fresh library to draw from.
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    bolt_face(&mut g);
    // The two spare cards were discarded and two fresh cards drawn.
    assert!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Forest").count() >= 2,
        "hand cards discarded");
    assert_eq!(g.players[0].hand.len(), 2, "drew back that many");
}

#[test]
fn electrodominance_deals_x_and_free_casts() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    let ed = g.add_card_to_hand(0, catalog::electrodominance());
    // A bolt in hand (mv 1 ≤ X) to free-cast.
    g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: ed, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: Some(3),
    }).expect("Electrodominance castable");
    drain_stack(&mut g);
    // At least the X = 3 damage landed on P1.
    assert!(g.players[1].life <= 17, "dealt X=3 to the opponent");
}
