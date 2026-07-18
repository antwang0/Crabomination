//! Functionality tests for `catalog::sets::decks::recent258` (MKM split cards).

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameState};

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Fuss (the left half) grows only your attacking creatures.
#[test]
fn fuss_pumps_attackers() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let idle = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attacker");
    drain_stack(&mut g);
    let def = catalog::fuss_bother();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&def.effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert_eq!(
        g.battlefield_find(attacker).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "the attacker got a +1/+1 counter",
    );
    assert_eq!(
        g.battlefield_find(idle).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
        "the non-attacker was untouched",
    );
}

/// Bother (the right half) makes three Thopters and surveils.
#[test]
fn bother_makes_thopters() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let right = catalog::fuss_bother().split.unwrap().right.effect.clone();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&right, &ctx).unwrap();
    drain_stack(&mut g);
    let thopters = g.battlefield.iter().filter(|c| c.definition.name == "Thopter" && c.controller == 0).count();
    assert_eq!(thopters, 3, "created three Thopters");
}

/// Cease (the left half) exiles graveyard cards and gives the target player 2
/// life plus a card.
#[test]
fn cease_exiles_and_refills() {
    let mut g = two_player_game();
    let gy1 = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let gy2 = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::mountain());
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    // Choose both graveyard cards to exile.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Cards(vec![gy1, gy2]),
    ]));
    let ctx = EffectContext::for_spell(0, Some(Target::Player(0)), 0, 0);
    g.resolve_effect(&catalog::cease_desist().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 0, "up to two graveyard cards exiled");
    assert_eq!(g.players[0].life, life + 2, "target player gained 2 life");
    assert_eq!(g.players[0].hand.len(), hand + 1, "target player drew a card");
}

/// Desist (the right half) destroys all artifacts and enchantments.
#[test]
fn desist_wipes_artifacts_and_enchantments() {
    let mut g = two_player_game();
    let clue = g.add_card_to_battlefield(0, catalog::insidious_roots()); // enchantment
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // creature — spared
    let artifact = g.add_card_to_battlefield(1, catalog::assemble_the_players()); // enchantment
    let right = catalog::cease_desist().split.unwrap().right.effect.clone();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&right, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    g.check_state_based_actions();
    assert!(g.battlefield_find(clue).is_none(), "enchantment destroyed");
    assert!(g.battlefield_find(artifact).is_none(), "opponent enchantment destroyed");
    assert!(g.battlefield_find(bear).is_some(), "creature spared");
}
