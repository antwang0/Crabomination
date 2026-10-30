//! CR 603.6a / 608.2d — "choose a card with mana value 1, then do the same
//! for 2 and 3; return those cards" (`Effect::ChoosePerManaValue`): every
//! pick is made first and the cards enter together, so their enter triggers
//! see each other.

use crabomination::card::{CardDefinition, CardId};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::GameAction;
use crabomination::game::*;
use crabomination::mana::{cost, generic};
use crabomination::TurnStep;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

/// A three-mana Soul Warden: it hears the other two only if they enter with it.
fn big_warden() -> CardDefinition {
    CardDefinition { name: "Big Warden", cost: cost(&[generic(3)]), ..catalog::soul_warden() }
}

fn activate_kayla(g: &mut GameState, kayla: CardId) {
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kayla, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Queen Kayla returns her mana value 1, 2 and 3 discards together: the
/// three-drop warden gains life for the other two (one after another, it
/// entered last and heard nothing).
#[test]
fn cr_603_6a_queen_kayla_returns_her_picks_together() {
    let mut g = main_phase();
    let kayla = g.add_card_to_battlefield(0, catalog::queen_kayla_bin_kroog());
    g.clear_sickness(kayla);
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    let warden = g.add_card_to_hand(0, big_warden());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let life = g.players[0].life;
    activate_kayla(&mut g, kayla);
    for id in [vanguard, bears, warden] {
        assert!(g.battlefield_find(id).is_some(), "each mana value's pick returns");
    }
    assert_eq!(g.players[0].life, life + 2, "the warden saw the other two enter with it");
}

/// "You may choose": a passed mana value stays in the graveyard.
#[test]
fn cr_608_2d_queen_kayla_may_pass_a_mana_value() {
    let mut g = main_phase();
    let kayla = g.add_card_to_battlefield(0, catalog::queen_kayla_bin_kroog());
    g.clear_sickness(kayla);
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![]), DecisionAnswer::Cards(vec![bears])]));
    activate_kayla(&mut g, kayla);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == vanguard), "the passed one-drop stays");
    assert!(g.battlefield_find(bears).is_some());
}

/// Gruesome Menagerie's picks are mandatory and one per value; a second
/// one-drop stays behind.
#[test]
fn cr_608_2d_gruesome_menagerie_takes_one_of_each_value() {
    let mut g = main_phase();
    let a = g.add_card_to_graveyard(0, catalog::elite_vanguard());
    let b = g.add_card_to_graveyard(0, catalog::elite_vanguard());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::centaur_courser());
    let spell = g.add_card_to_hand(0, catalog::gruesome_menagerie());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some() && g.battlefield_find(giant).is_some());
    let ones = [a, b].iter().filter(|id| g.battlefield_find(**id).is_some()).count();
    assert_eq!(ones, 1, "one creature of mana value 1");
}
