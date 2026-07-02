//! Functionality tests for `catalog::sets::decks::recent86`.

use crate::card::CreatureType;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::two_player_game;
use crate::game::types::Target;
use crate::game::*;

fn enter_choosing(g: &mut GameState, id: CardId, ct: CreatureType) {
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(ct)]));
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
}

#[test]
fn urzas_incubator_reduces_chosen_type_creature_cost() {
    let mut g = two_player_game();
    let inc = g.add_card_to_battlefield(0, catalog::urzas_incubator());
    enter_choosing(&mut g, inc, CreatureType::Bear);
    // Grizzly Bears is {1}{G}; with Urza's Incubator naming Bear, it costs {G}.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast for the reduced cost (generic {1} waived)");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "Bear resolved for one green");
}

#[test]
fn heralds_horn_reduces_by_one_only() {
    let mut g = two_player_game();
    let horn = g.add_card_to_battlefield(0, catalog::heralds_horn());
    enter_choosing(&mut g, horn, CreatureType::Elf);
    // A non-Elf creature spell is NOT reduced: Grizzly Bears still needs {1}{G}.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "off-type spell isn't reduced, so one green alone is insufficient");
}

/// CR 601.2f / 117.7c — a cost reduction only removes generic mana; colored
/// pips survive. Urza's Incubator naming Elf can't waive Llanowar Elves' {G}.
#[test]
fn cr_601_2f_reduction_is_generic_only() {
    let mut g = two_player_game();
    let inc = g.add_card_to_battlefield(0, catalog::urzas_incubator());
    enter_choosing(&mut g, inc, CreatureType::Elf);
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves()); // {G}, an Elf
    g.players[0].mana_pool.add_colorless(5); // only generic mana
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "the {{G}} pip can't be paid with generic even under a {{2}} reduction");
}

#[test]
fn seismic_assault_discards_land_for_two_damage() {
    let mut g = two_player_game();
    let sa = g.add_card_to_battlefield(0, catalog::seismic_assault());
    g.add_card_to_hand(0, catalog::mountain()); // a land to discard
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sa, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    })
    .expect("discard a land, deal 2");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 damage to the opponent");
    assert!(g.players[0].hand.iter().all(|c| c.definition.name != "Mountain"),
        "the land was discarded as a cost");
}
