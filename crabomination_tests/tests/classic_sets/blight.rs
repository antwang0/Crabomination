//! Blight (CR 701.68) + Ward—Blight + saddle sorcery-speed conformance.

use crabomination::catalog;
use crabomination::card::CounterType;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;
use crabomination::TurnStep;

/// CR 701.68 / 702.21 — Ward—Blight 2: a spell targeting Auntie forces the
/// caster to put two -1/-1 counters on a creature they control. Auntie's payoff
/// then fires (counters on a creature you don't control → its controller loses
/// 1 life).
#[test]
fn cr_701_68_ward_blight_paid_with_own_creature() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let auntie = g.add_card_to_battlefield(0, catalog::auntie_ool_cursewretch());
    let wall = g.add_card_to_battlefield(1, catalog::caelorna_coral_tyrant()); // 0/8, survives blight
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(auntie)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt at Auntie");
    drain_stack(&mut g);
    let _ = p1_life;
    // Ward paid by blighting the caster's wall (0/8 → 0/6, survives).
    let wc = g.battlefield_find(wall).expect("wall survives blight");
    assert_eq!(wc.counters.get(&CounterType::MinusOneMinusOne).copied().unwrap_or(0), 2, "two -1/-1 counters");
    assert!(g.battlefield_find(auntie).is_some(), "Auntie survives the 3-damage bolt");
    // Auntie's payoff fires off the ward-payment counters: the counters land on
    // a creature P0 doesn't control → its controller (P1) loses 1 life.
    assert_eq!(g.players[1].life, p1_life - 1, "ward-payment counters fired Auntie's payoff");
}

/// Ward—Blight is unpayable with no creature → the spell is countered.
#[test]
fn cr_701_68_ward_blight_unpayable_counters_spell() {
    let mut g = two_player_game();
    let auntie = g.add_card_to_battlefield(0, catalog::auntie_ool_cursewretch());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(auntie)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt at Auntie");
    drain_stack(&mut g);
    // No creature to blight → ward unpaid → bolt countered, Auntie untouched.
    let c = g.battlefield_find(auntie).expect("Auntie alive");
    assert_eq!(c.damage, 0, "bolt was countered, no damage");
}

/// CR 702.171a — Saddle is sorcery-speed only; rejected with the stack non-empty.
#[test]
fn cr_702_171a_saddle_is_sorcery_speed_only() {
    let mut g = two_player_game();
    let ghoda = g.add_card_to_battlefield(0, catalog::gilded_ghoda());
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(helper);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Put something on the stack so it isn't a clean sorcery-speed window.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    let res = g.perform_action(GameAction::Saddle { mount: ghoda, creatures: vec![helper] });
    assert!(matches!(res, Err(GameError::SorcerySpeedOnly)), "saddle rejected at instant speed");
}

/// Auntie Ool's payoff: -1/-1 counters on a creature you control → you draw.
/// Exercises the AnyPlayer `CounterAdded(-1/-1)` trigger via Blighted
/// Blackthorn's own blight (CR 701.68).
#[test]
fn cr_701_68_auntie_draws_when_you_blight_your_own() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    g.add_card_to_battlefield(0, catalog::auntie_ool_cursewretch());
    let bt = g.add_card_to_battlefield(0, catalog::blighted_blackthorn()); // 3/7
    g.clear_sickness(bt);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true), // opt into blight 2
    ]));
    let hand = g.players[0].hand.len();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: bt, target: crabomination::game::types::AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // Blackthorn's own draw (1) + Auntie's payoff draw (1) = +2.
    assert_eq!(g.players[0].hand.len(), hand + 2, "Blackthorn + Auntie both drew");
}
