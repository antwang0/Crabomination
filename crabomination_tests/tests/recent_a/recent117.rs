//! Functionality tests for `catalog::sets::decks::recent117`.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Bigfin Bouncer's ETB returns an opponent's creature to hand.
#[test]
fn bigfin_bouncer_bounces_on_etb() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::bigfin_bouncer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bigfin Bouncer");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != victim), "opponent's creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "back in owner's hand");
}

/// Alania's Pathmaker exiles the top card and makes it playable.
#[test]
fn alanias_pathmaker_impulses_top() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::alanias_pathmaker());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Alania's Pathmaker");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == top), "top card exiled and playable");
}

/// Apothecary Stomper's ETB (mode 0) puts two +1/+1 counters on a creature.
#[test]
fn apothecary_stomper_etb_counters() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::apothecary_stomper());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Apothecary Stomper");
    drain_stack(&mut g);
    let counters: u32 = g.battlefield.iter().filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne)).sum();
    assert_eq!(counters, 2, "two +1/+1 counters placed");
}

/// Armasaur Guide adds a counter when you attack with three or more creatures.
#[test]
fn armasaur_guide_counter_on_three_attackers() {
    let mut g = two_player_game();
    let guide = g.add_card_to_battlefield(0, catalog::armasaur_guide());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [guide, a, b] { g.clear_sickness(id); }
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![
        Attack { attacker: guide, target: AttackTarget::Player(1) },
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]).expect("declare three attackers");
    drain_stack(&mut g);
    let counters: u32 = g.battlefield.iter().filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne)).sum();
    assert_eq!(counters, 1, "one +1/+1 counter on a three-creature attack");
}

/// Battlesong Berserker pumps a creature and grants menace when you attack.
#[test]
fn battlesong_berserker_pumps_on_attack() {
    let mut g = two_player_game();
    let zerk = g.add_card_to_battlefield(0, catalog::battlesong_berserker());
    g.clear_sickness(zerk);
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: zerk, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    // The berserker is the only creature; it targets itself.
    let cp = g.computed_permanent(zerk).unwrap();
    assert_eq!(cp.power, 4, "3/4 pumped to 4/4");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Menace), "gains menace");
}

/// Billowing Shriekmass mills three on entry and grows under threshold.
#[test]
fn billowing_shriekmass_mills_and_threshold() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    for _ in 0..10 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::billowing_shriekmass());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Billowing Shriekmass");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy_before + 3, "milled three");
    let mass = g.battlefield.iter().find(|c| c.definition.name == "Billowing Shriekmass").unwrap().id;
    assert_eq!(g.computed_permanent(mass).unwrap().power, 2, "no threshold yet");
    for _ in 0..7 { g.add_card_to_graveyard(0, catalog::grizzly_bears()); }
    assert_eq!(g.computed_permanent(mass).unwrap().power, 4, "threshold → +2/+1");
}

/// Bulk Up doubles a creature's power.
#[test]
fn bulk_up_doubles_power() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let spell = g.add_card_to_hand(0, catalog::bulk_up());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bulk Up");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6, "3 doubled to 6");
}
