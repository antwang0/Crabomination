//! Functionality tests for `catalog::sets::decks::recent218`.

use crate::card::CounterType;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::effects::EffectContext;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Baylen taps three tokens to draw a card.
#[test]
fn baylen_taps_tokens_to_draw() {
    let mut g = two_player_game();
    let baylen = g.add_card_to_battlefield(0, catalog::baylen_the_haymaker());
    let tokens: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    for &t in &tokens { g.battlefield_find_mut(t).unwrap().is_token = true; }
    g.add_card_to_library(0, catalog::forest());
    g.clear_sickness(baylen);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: baylen, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap three tokens: draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert_eq!(tokens.iter().filter(|&&t| g.battlefield_find(t).unwrap().tapped).count(), 3, "three tokens tapped");
}

/// Haazda Vigilante puts a +1/+1 counter on a small creature on ETB.
#[test]
fn haazda_vigilante_boosts_on_etb() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    let haazda = catalog::haazda_vigilante();
    let effect = haazda.triggered_abilities[0].effect.clone();
    let hz = g.add_card_to_battlefield(0, catalog::haazda_vigilante());
    let ctx = EffectContext { targets: vec![Target::Permanent(small)], ..EffectContext::for_trigger(hz, 0, None, 0) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(small).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Neighborhood Guardian pumps a creature when a small creature enters.
#[test]
fn neighborhood_guardian_pumps_on_small_entrant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::neighborhood_guardian());
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let small = g.add_card_to_hand(0, catalog::grizzly_bears()); // power 2 ≤ 2 entrant
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Target the existing buddy with the pump.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(buddy))]));
    g.perform_action(GameAction::CastSpell {
        card_id: small, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast small creature");
    drain_stack(&mut g);
    let v = g.computed_permanent(buddy).unwrap();
    assert_eq!((v.power, v.toughness), (3, 3), "buddy pumped +1/+1");
}

/// Griffnaut Tracker exiles up to two cards from a single graveyard on ETB.
#[test]
fn griffnaut_tracker_exiles_graveyard() {
    let mut g = two_player_game();
    let ids: Vec<_> = (0..3).map(|_| g.add_card_to_graveyard(1, catalog::grizzly_bears())).collect();
    let tracker = g.add_card_to_battlefield(0, catalog::griffnaut_tracker());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![ids[0], ids[1]])]));
    let effect = catalog::griffnaut_tracker().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_trigger(tracker, 0, None, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.players[1].graveyard.len(), 1, "two of three graveyard cards exiled");
}

/// Rubblebelt Braggart suspects itself when it attacks.
#[test]
fn rubblebelt_braggart_suspects_on_attack() {
    let mut g = two_player_game();
    let brag = g.add_card_to_battlefield(0, catalog::rubblebelt_braggart());
    g.clear_sickness(brag);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.declare_attackers(vec![Attack { attacker: brag, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(brag).unwrap().suspected, "suspected itself");
}

/// Gearbane Orangutan destroys an artifact via mode 0.
#[test]
fn gearbane_orangutan_destroys_artifact() {
    let mut g = two_player_game();
    let clue = g.add_card_to_battlefield(1, catalog::mazemind_tome()); // an artifact target
    let ape = g.add_card_to_battlefield(0, catalog::gearbane_orangutan());
    let effect = catalog::gearbane_orangutan().triggered_abilities[0].effect.clone();
    let ctx = EffectContext {
        mode: 0,
        targets: vec![Target::Permanent(clue)],
        ..EffectContext::for_trigger(ape, 0, None, 0)
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield_find(clue).is_none(), "artifact destroyed");
}
