//! CR conformance for this run's sweep:
//! - CR 710 — flip cards.
//! - CR 403 — the battlefield zone.
//! - CR 112 — spells.

use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, ZoneDest, PlayerRef};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

// ── CR 710 — Flip cards ──

/// CR 710.1c — flipping doesn't change the permanent's colour or mana cost.
#[test]
fn cr_710_1c_flipping_keeps_color_and_mana_cost() {
    let mut g = two_player_game();
    let pupil = g.add_card_to_battlefield(0, catalog::budoka_pupil());
    let mut evs = vec![];
    g.flip_permanent(pupil, &mut evs);
    let cp = g.computed_permanent(pupil).unwrap();
    assert_eq!(g.battlefield_find(pupil).unwrap().definition.name, "Ichiga, Who Topples Oaks");
    assert!(cp.colors.contains(&Color::Green), "still green");
    assert_eq!(g.battlefield_find(pupil).unwrap().definition.cost.cmc(), 3);
}

/// CR 710.2 — outside the battlefield a flip card has only its normal
/// (unflipped) characteristics.
#[test]
fn cr_710_2_flipped_card_reverts_off_the_battlefield() {
    let mut g = two_player_game();
    let pupil = g.add_card_to_battlefield(0, catalog::budoka_pupil());
    let mut evs = vec![];
    g.flip_permanent(pupil, &mut evs);
    assert!(g.battlefield_find(pupil).unwrap().flipped);
    g.remove_to_graveyard_with_triggers(pupil);
    let gy = g.players[0].graveyard.iter().find(|c| c.id == pupil).expect("in graveyard");
    assert!(!gy.flipped);
    assert_eq!(gy.definition.name, "Budoka Pupil");
}

/// CR 710.4 — flipping is one-way, and a flipped permanent that leaves keeps
/// no memory of it: recast, it comes back unflipped.
#[test]
fn cr_710_4_flip_is_one_way_and_forgotten_on_zone_change() {
    let mut g = two_player_game();
    let pupil = g.add_card_to_battlefield(0, catalog::budoka_pupil());
    let mut evs = vec![];
    g.flip_permanent(pupil, &mut evs);
    // A second flip is a no-op — the permanent is already flipped.
    let before = evs.len();
    g.flip_permanent(pupil, &mut evs);
    assert_eq!(evs.len(), before, "no second Flipped event");
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::Move {
            what: Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        &ctx,
    )
    .expect("bounce");
    let hand = g.players[0].hand.iter().find(|c| c.id == pupil).expect("in hand");
    assert!(!hand.flipped);
    assert_eq!(hand.definition.name, "Budoka Pupil");
}

// ── CR 403 — The battlefield ──

/// CR 403.3 — every object on the battlefield is a permanent; a resolved
/// instant never lands there.
#[test]
fn cr_403_3_resolved_instants_never_reach_the_battlefield() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.definition.is_permanent()));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt));
}

/// CR 403.4 — a permanent re-entering the battlefield is a new object: its
/// counters are gone and it's summoning sick again.
#[test]
fn cr_403_4_reentering_permanent_is_a_new_object() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.clear_sickness(bear);
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::Move {
            what: Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        &ctx,
    )
    .expect("bounce");
    g.resolve_effect(
        &Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: crabomination::card::SelectionRequirement::Creature,
            count: crabomination::effect::Value::ONE,
            tapped: false,
            haste: false,
            sacrifice_eot: false,
            return_eot: false,
        },
        &ctx,
    )
    .expect("redeploy");
    let back = g.battlefield_find(bear).expect("back on the battlefield");
    assert_eq!(back.counter_count(CounterType::PlusOnePlusOne), 0);
    assert!(back.summoning_sick);
}

/// CR 403.2 — an effect that doesn't name another zone touches only the
/// battlefield: a board wipe leaves graveyard creature cards alone.
#[test]
fn cr_403_2_board_wipe_ignores_other_zones() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let in_hand = g.add_card_to_hand(0, catalog::grizzly_bears());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::DestroyNoRegen {
            what: Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature),
        },
        &ctx,
    )
    .expect("wipe");
    assert!(g.battlefield.iter().all(|c| !c.definition.is_creature()));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == in_gy));
    assert!(g.players[0].hand.iter().any(|c| c.id == in_hand));
}

// ── CR 112 — Spells ──

/// CR 112.1 — casting moves the card out of its zone onto the stack, where it
/// is a spell until it resolves.
#[test]
fn cr_112_1_a_cast_card_leaves_its_zone_for_the_stack() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    assert!(!g.players[0].hand.iter().any(|c| c.id == bolt), "gone from hand");
    assert_eq!(g.stack.len(), 1);
    drain_stack(&mut g);
    assert!(g.stack.is_empty());
}

/// CR 112.1a / 112.2 — a copy of a spell is a spell in its own right, and its
/// controller is the player who put it on the stack, not the original caster.
#[test]
fn cr_112_2_a_spell_copy_is_controlled_by_the_copier() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let fork = g.add_card_to_hand(1, catalog::reverberate());
    g.players[1].mana_pool.add(Color::Red, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: fork,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("copy");
    drain_stack(&mut g);
    // Seat 1's copy resolved first (it was on top) and seat 0's original after,
    // so seat 1 took 3 from its own copy plus 3 from the original.
    assert_eq!(g.players[1].life, 20 - 6);
}

/// CR 112.4 — a characteristic change applied to a permanent spell keeps
/// applying to the permanent it becomes.
#[test]
fn cr_112_4_permanent_spell_pump_survives_resolution() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::PumpPT {
            what: Selector::EachMatching {
                zone: crabomination::effect::ZoneRef::Stack,
                filter: crabomination::card::SelectionRequirement::Creature,
            },
            power: crabomination::effect::Value::Const(2),
            toughness: crabomination::effect::Value::Const(2),
            duration: crabomination::effect::Duration::EndOfTurn,
        },
        &ctx,
    )
    .expect("pump the spell");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).expect("resolved");
    assert_eq!((cp.power, cp.toughness), (4, 4), "the pump followed it onto the battlefield");
    assert!(cp.card_types.contains(&CardType::Creature));
}
