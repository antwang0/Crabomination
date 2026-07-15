//! Functionality tests for `catalog::sets::decks::recent220`.

use crate::card::{CardType, CounterType, Keyword};
use crate::catalog;
use crate::effect::{Effect, Selector, Value};
use crate::game::effects::EffectContext;
use crate::game::types::{GameAction, Target, TurnStep};
use crate::game::{drain_stack, two_player_game};

/// Stocking the Pantry gains a supply counter when you counter-up a creature,
/// and spends it to draw.
#[test]
fn stocking_the_pantry_banks_and_draws() {
    let mut g = two_player_game();
    let pantry = g.add_card_to_battlefield(0, catalog::stocking_the_pantry());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    // Putting a +1/+1 counter on my creature banks a supply counter.
    let add = Effect::AddCounter {
        what: Selector::Target(0),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_trigger(bear, 0, None, 0) };
    let evs = g.resolve_effect(&add, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(pantry).unwrap().counter_count(CounterType::Supply), 1, "banked a supply counter");

    // Spend it: {2}, remove a supply counter: draw.
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: pantry, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("remove a supply counter to draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(g.battlefield_find(pantry).unwrap().counter_count(CounterType::Supply), 0, "supply counter spent");
}

/// War Squeak's ETB makes an opponent's creature unable to block.
#[test]
fn war_squeak_grants_cant_block() {
    let mut g = two_player_game();
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let squeak = g.add_card_to_battlefield(0, catalog::war_squeak());
    let effect = catalog::war_squeak().triggered_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(blocker)], ..EffectContext::for_trigger(squeak, 0, None, 0) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.computed_permanent(blocker).unwrap().keywords.contains(&Keyword::CantBlock), "opponent's creature can't block");
}

/// Tangle Tumbler animates itself by tapping two tokens.
#[test]
fn tangle_tumbler_animates_via_tokens() {
    let mut g = two_player_game();
    let tumbler = g.add_card_to_battlefield(0, catalog::tangle_tumbler());
    let tokens: Vec<_> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    for &t in &tokens { g.battlefield_find_mut(t).unwrap().is_token = true; }
    g.clear_sickness(tumbler);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(!g.computed_permanent(tumbler).unwrap().card_types.contains(&CardType::Creature), "starts as a non-creature Vehicle");
    g.perform_action(GameAction::ActivateAbility {
        card_id: tumbler, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap two tokens to animate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(tumbler).unwrap().card_types.contains(&CardType::Creature), "now an artifact creature");
    assert_eq!(tokens.iter().filter(|&&t| g.battlefield_find(t).unwrap().tapped).count(), 2, "two tokens tapped");
}

/// Bonecache Overseer only draws once three cards have left your graveyard.
#[test]
fn bonecache_overseer_gated_on_graveyard_departures() {
    let mut g = two_player_game();
    let overseer = g.add_card_to_battlefield(0, catalog::bonecache_overseer());
    g.add_card_to_library(0, catalog::forest());
    g.clear_sickness(overseer);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Not yet: fewer than three cards have left the graveyard.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: overseer, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "gated until three cards leave the graveyard");
    g.players[0].cards_left_graveyard_this_turn = 3;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: overseer, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("now activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}
