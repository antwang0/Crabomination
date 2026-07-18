//! Functionality tests for `catalog::sets::decks::recent254`
//! (Melek + variable collect-evidence dragon).

use crabomination::card::CardInstance;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::actions::cost_reduction_for_spell;
use crabomination::game::{drain_stack, two_player_game};

/// Melek's power/toughness is twice the instant and sorcery cards in his
/// controller's graveyard.
#[test]
fn melek_pt_scales_with_instants_and_sorceries() {
    let mut g = two_player_game();
    let melek = g.add_card_to_battlefield(0, catalog::melek_reforged_researcher());
    assert_eq!(g.computed_permanent(melek).map(|c| (c.power, c.toughness)), Some((0, 0)));
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
    g.add_card_to_graveyard(0, catalog::divination()); // sorcery
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature — ignored
    assert_eq!(
        g.computed_permanent(melek).map(|c| (c.power, c.toughness)),
        Some((4, 4)),
        "2 I/S cards × 2",
    );
}

/// Melek makes the first instant/sorcery spell each turn cost {3} less, but not
/// creature spells or the second I/S spell.
#[test]
fn melek_discounts_first_instant_or_sorcery() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::melek_reforged_researcher());
    let bolt = CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    let bear = CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 3, "first I/S → {{3}} off");
    assert_eq!(cost_reduction_for_spell(&g, 0, &bear, None), 0, "creature spell unaffected");
    g.players[0].instants_or_sorceries_cast_this_turn = 1;
    assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 0, "second I/S → no discount");
}

/// Incinerator of the Guilty collects evidence on combat damage and deals X to
/// each creature/planeswalker the damaged player controls, where X is the total
/// mana value exiled.
#[test]
fn incinerator_collects_evidence_and_burns_the_board() {
    let mut g = two_player_game();
    let incinerator = g.add_card_to_battlefield(0, catalog::incinerator_of_the_guilty());
    // Fuel: two MV-2 cards → X = 4 when the bot exiles the whole graveyard.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    // Opponent's board: a 2/2 and a 5/5 (survives X=4).
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    // Opt into the "collect evidence X" reflexive (bot path exiles the whole gy).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_combat_damage_to_player_triggers(incinerator, 1, 6);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2/2 took 4 and died");
    assert!(g.battlefield_find(big).is_some(), "6/6 survived 4 damage");
    assert_eq!(g.players[0].graveyard.len(), 0, "evidence exiled the graveyard");
}
