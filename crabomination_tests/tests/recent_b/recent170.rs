//! Functionality tests for `catalog::sets::decks::recent170` — the Roads land
//! cycle, exhaust/speed/anthem Vehicles.

use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::*;

/// A Roads land enters tapped when you control no Mount or Vehicle, untapped
/// when you do.
#[test]
fn roads_land_enters_tapped_unless_vehicle() {
    // No Vehicle → enters tapped.
    let mut g = two_player_game();
    let l1 = g.move_card_to_battlefield_for_test(0, catalog::foul_roads());
    drain_stack(&mut g);
    assert!(g.battlefield_find(l1).unwrap().tapped, "enters tapped with no Mount/Vehicle");

    // Control a Vehicle → enters untapped.
    let mut g2 = two_player_game();
    g2.add_card_to_battlefield(0, catalog::skybox_ferry());
    let l2 = g2.move_card_to_battlefield_for_test(0, catalog::rocky_roads());
    drain_stack(&mut g2);
    assert!(!g2.battlefield_find(l2).unwrap().tapped, "enters untapped with a Vehicle out");
}

/// A Roads land sacrifices for a Pilot token that crews/saddles with a +2 power
/// bonus.
#[test]
fn roads_land_sacrifices_for_boosted_pilot() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::reef_roads());
    g.clear_sickness(land);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("sac for a Pilot");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land sacrificed");
    let pilot = g.battlefield.iter().find(|c| c.definition.name == "Pilot").expect("made a Pilot");
    assert_eq!(g.crew_saddle_power_bonus(pilot.id), 2, "Pilot crews as though +2 power");
}

/// Rangers' Aetherhive mints a Thopter whenever you activate an exhaust ability.
#[test]
fn rangers_aetherhive_thopter_on_exhaust() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rangers_aetherhive());
    let refueler = g.add_card_to_battlefield(0, catalog::rangers_refueler());
    g.add_card_to_library(0, catalog::forest()); // refueler's own draw-on-exhaust
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: refueler, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate an exhaust ability");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Thopter"),
        "the Aetherhive made a Thopter off the exhaust event");
}

/// Racers' Scoreboard draws two and discards one on ETB, and cuts spell costs
/// by {1} at max speed.
#[test]
fn racers_scoreboard_etb_and_max_speed_discount() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let c1 = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![c1])]));
    let hand_before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::racers_scoreboard());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew two, discarded one");

    // Max speed → a {1}{G} creature costs just {G}.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].speed = 4;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(bear, None, vec![], None, None).expect("max-speed {1} discount pays {1}{G} with {G}");
}

/// Salvation Engine buffs other artifact creatures and reanimates an artifact on
/// attack.
#[test]
fn salvation_engine_anthem_and_attack_reanimate() {
    let mut g = two_player_game();
    let engine = g.add_card_to_battlefield(0, catalog::salvation_engine());
    let ornithopter = g.add_card_to_battlefield(0, catalog::ornithopter()); // 0/2 artifact creature
    g.add_card_to_graveyard(0, catalog::sol_ring()); // artifact in gy
    // Anthem: other artifact creatures get +2/+2.
    let cp = g.computed_permanent(ornithopter).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4), "Ornithopter buffed to 2/4");
    // Animate + attack to fire the reanimation.
    let ts = g.next_timestamp();
    g.add_continuous_effect(crabomination::game::layers::ContinuousEffect {
        timestamp: ts,
        source: engine,
        affected: crabomination::game::layers::AffectedPermanents::Specific(vec![engine]),
        layer: crabomination::game::layers::Layer::L4Type,
        sublayer: None,
        duration: crabomination::game::layers::EffectDuration::UntilEndOfTurn,
        modification: crabomination::game::layers::Modification::AddCardType(crabomination::card::CardType::Creature),
    });
    g.clear_sickness(engine);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: engine, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Sol Ring"),
        "reanimated the artifact from the graveyard");
}
