//! Functionality tests for `catalog::sets::decks::recent217`.

use crate::card::CounterType;
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Serra Redeemer puts two +1/+1 counters on a small creature that enters.
#[test]
fn serra_redeemer_boosts_small_entrants() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::serra_redeemer());
    let small = g.add_card_to_hand(0, catalog::grizzly_bears()); // power 2 ≤ 2
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: small, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(small).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "small entrant gets two +1/+1 counters");
}

/// Wandertale Mentor grows on expend 4 and taps for red or green.
#[test]
fn wandertale_mentor_expend_and_mana() {
    let mut g = two_player_game();
    let mentor = g.add_card_to_battlefield(0, catalog::wandertale_mentor());
    let moose = g.add_card_to_hand(0, catalog::galewind_moose()); // {4}{G}{G} crosses expend 4
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast 6-mana spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mentor).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "expend 4 → +1/+1 counter");
    // Mana ability (index 0 = red) produces {R}.
    g.clear_sickness(mentor);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mentor, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for red");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "added one red mana");
}

/// Starseer Mentor punishes an opponent who can't dodge (no permanent to sac,
/// empty hand) for 3 life. Drives the trigger effect directly.
#[test]
fn starseer_mentor_drains_when_no_dodge() {
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let mentor = g.add_card_to_battlefield(0, catalog::starseer_mentor());
    g.players[1].hand.clear(); // no card to discard, no permanent to sacrifice
    let foe_life = g.players[1].life;
    let effect = catalog::starseer_mentor().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_trigger(mentor, 0, None, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[1].life, foe_life - 3, "no dodge available → opponent loses 3");
}
