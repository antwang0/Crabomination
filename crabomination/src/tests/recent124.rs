//! Functionality tests for `catalog::sets::decks::recent124`.

use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Armored Armadillo's pump adds its toughness to power.
#[test]
fn armored_armadillo_pumps_by_toughness() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let dillo = g.add_card_to_battlefield(0, catalog::armored_armadillo()); // 0/4
    g.clear_sickness(dillo);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dillo, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(dillo).unwrap().power, 4, "+X/+0 where X = toughness 4");
}

/// Ambush Gigapede shrinks an opponent's creature on entry.
#[test]
fn ambush_gigapede_minus_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pede = g.add_card_to_battlefield(0, catalog::ambush_gigapede());
    g.fire_self_etb_triggers(pede, 0);
    drain_stack(&mut g);
    // 2/2 → 0/0, dies as a state-based action.
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "-2/-2 kills the 2/2");
}

/// Desperate Bloodseeker mills the targeted player two and has lifelink.
#[test]
fn desperate_bloodseeker_mills_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let gy_before = g.players[1].graveyard.len();
    let seeker = g.add_card_to_battlefield(0, catalog::desperate_bloodseeker());
    assert!(g.computed_permanent(seeker).unwrap().keywords.contains(&crate::card::Keyword::Lifelink));
    g.fire_self_etb_triggers(seeker, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy_before + 2, "target player milled two");
}

/// Deadeye Duelist pings an opponent for 1.
#[test]
fn deadeye_duelist_pings() {
    let mut g = two_player_game();
    let duelist = g.add_card_to_battlefield(0, catalog::deadeye_duelist());
    g.clear_sickness(duelist);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: duelist, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "1 damage to the opponent");
}

/// Eriette's Lullaby destroys a tapped creature and gains 2 life.
#[test]
fn eriettes_lullaby_kills_tapped() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::eriettes_lullaby());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Lullaby");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "tapped creature destroyed");
    assert_eq!(g.players[0].life, life + 2, "gained 2 life");
}

/// Geyser Drake discounts your spells during the opponent's turn only.
#[test]
fn geyser_drake_off_turn_discount() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::geyser_drake());
    // On the opponent's turn, a {1}{U} flash creature costs just {U}.
    g.active_player_idx = 1;
    let spell = g.add_card_to_hand(0, catalog::ambush_gigapede()); // {4}{B}{B} flash
    // {3}{B}{B} after the {1} off-turn discount.
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    let r = g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(r.is_ok(), "cast at the off-turn discount: {r:?}");
}

/// Bristlepack Sentry can attack only while you control a 4-power creature.
#[test]
fn bristlepack_sentry_conditional_attack() {
    let mut g = two_player_game();
    let sentry = g.add_card_to_battlefield(0, catalog::bristlepack_sentry());
    g.clear_sickness(sentry);
    g.step = TurnStep::DeclareAttackers;
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sentry, target: AttackTarget::Player(1),
    }])).is_err(), "defender can't attack without a big creature");

    let big = g.add_card_to_battlefield(0, catalog::gigantosaurus()); // 10/10
    g.clear_sickness(big);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sentry, target: AttackTarget::Player(1),
    }])).expect("attacks with a 4-power ally in play");
}
