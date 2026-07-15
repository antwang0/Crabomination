//! Functionality tests for `catalog::sets::decks::recent235` (DSK Rooms +
//! the manifest-dread `LastMoved` rider).

use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::Effect;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};

/// A Room's back-door unlock trigger effect, by door name.
fn door_effect(def: &crabomination::card::CardDefinition, right: bool) -> Effect {
    let room = def.room.as_ref().expect("room card");
    let door = if right { &room.right } else { &room.left };
    door.triggered_abilities[0].effect.clone()
}

/// Surgical Suite's unlock reanimates a creature MV≤3 from the graveyard.
#[test]
fn surgical_suite_reanimates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let def = catalog::surgical_suite_hospital_room();
    let src = g.add_card_to_battlefield(0, catalog::surgical_suite_hospital_room());
    let ctx = EffectContext {
        targets: vec![Target::Permanent(bear)],
        ..EffectContext::for_trigger(src, 0, None, 0)
    };
    g.resolve_effect(&door_effect(&def, false), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear && c.controller == 0), "bear reanimated");
}

/// Slimy Aquarium manifests a face-down 2/2 and puts a +1/+1 counter on it,
/// exercising the manifest-dread `LastMoved` rider.
#[test]
fn slimy_aquarium_manifests_and_counters() {
    let mut g = two_player_game();
    // Ensure the library has cards to manifest.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let def = catalog::underwater_tunnel_slimy_aquarium();
    let src = g.add_card_to_battlefield(0, catalog::underwater_tunnel_slimy_aquarium());
    let ctx = EffectContext::for_trigger(src, 0, None, 0);
    g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
    // A face-down 2/2 with a +1/+1 counter now sits on the battlefield.
    let manifested = g
        .battlefield
        .iter()
        .find(|c| c.controller == 0 && c.face_down && c.id != src)
        .expect("a manifested creature");
    assert_eq!(manifested.counter_count(CounterType::PlusOnePlusOne), 1, "counter placed");
}

/// Moldering Gym searches a basic land onto the battlefield tapped.
#[test]
fn moldering_gym_fetches_basic() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let def = catalog::moldering_gym_weight_room();
    let src = g.add_card_to_battlefield(0, catalog::moldering_gym_weight_room());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let ctx = EffectContext::for_trigger(src, 0, None, 0);
    g.resolve_effect(&door_effect(&def, false), &ctx).unwrap();
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.is_land() && c.tapped),
        "a basic land entered tapped",
    );
}

/// Greenhouse grants every land you control a "{T}: Add any color" ability.
#[test]
fn greenhouse_grants_land_mana_ability() {
    let def = catalog::greenhouse_rickety_gazebo();
    let room = def.room.as_ref().unwrap();
    assert!(
        !room.left.static_abilities.is_empty(),
        "Greenhouse has the land-grant static",
    );
}

/// Rickety Gazebo mills four, then returns up to two permanent cards to hand.
#[test]
fn rickety_gazebo_mill_then_return() {
    let mut g = two_player_game();
    let ids: Vec<_> = (0..4).map(|_| g.add_card_to_library(0, catalog::grizzly_bears())).collect();
    let def = catalog::greenhouse_rickety_gazebo();
    let src = g.add_card_to_battlefield(0, catalog::greenhouse_rickety_gazebo());
    // Pick two of the four milled creatures to return.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![ids[0], ids[1]])]));
    let ctx = EffectContext::for_trigger(src, 0, None, 0);
    g.resolve_effect(&door_effect(&def, true), &ctx).unwrap();
    let in_hand = g.players[0].hand.iter().filter(|c| ids.contains(&c.id)).count();
    let in_gy = g.players[0].graveyard.iter().filter(|c| ids.contains(&c.id)).count();
    assert_eq!(in_hand, 2, "two returned to hand");
    assert_eq!(in_gy, 2, "the other two stayed milled");
}

/// Walk-In Closet grants "play lands from your graveyard".
#[test]
fn walk_in_closet_static() {
    let def = catalog::walk_in_closet_forgotten_cellar();
    let room = def.room.as_ref().unwrap();
    assert!(!room.left.static_abilities.is_empty(), "Walk-In Closet carries a static");
}

/// Orphans of the Wheat taps chosen creatures on attack and pumps per tap.
#[test]
fn orphans_taps_and_pumps() {
    let mut g = two_player_game();
    let orphans = g.add_card_to_battlefield(0, catalog::orphans_of_the_wheat()); // 2/1
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b])]));
    let effect = catalog::orphans_of_the_wheat().triggered_abilities[0].effect.clone();
    let ctx = EffectContext::for_trigger(orphans, 0, None, 0);
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped, "both tapped");
    assert_eq!(g.computed_permanent(orphans).unwrap().power, 4, "2 + 2 tapped = 4");
}
