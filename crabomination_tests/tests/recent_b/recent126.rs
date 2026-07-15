//! Functionality tests for `catalog::sets::decks::recent126`.

use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Mine Raider makes a Treasure only while you control another outlaw.
#[test]
fn mine_raider_treasure_with_another_outlaw() {
    // No other outlaw → no Treasure.
    let mut g = two_player_game();
    let raider = g.add_card_to_battlefield(0, catalog::mine_raider());
    g.fire_self_etb_triggers(raider, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 0);

    // With a Rogue ally (an outlaw) → a Treasure.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::deadeye_duelist()); // Human Assassin = outlaw
    let raider = g.add_card_to_battlefield(0, catalog::mine_raider());
    g.fire_self_etb_triggers(raider, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1,
        "another outlaw → Treasure");
}

/// Scorching Shot deals 5 to a creature.
#[test]
fn scorching_shot_deals_five() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let wall = g.add_card_to_battlefield(1, catalog::gigantosaurus()); // 10/10
    let spell = g.add_card_to_hand(0, catalog::scorching_shot());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(wall)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Scorching Shot");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wall).unwrap().damage, 5, "5 damage marked");
}

/// Peerless Ropemaster bounces a tapped creature on entry.
#[test]
fn peerless_ropemaster_bounces_tapped() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let rope = g.add_card_to_battlefield(0, catalog::peerless_ropemaster());
    g.fire_self_etb_triggers(rope, 0);
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "tapped creature returned to hand");
}

/// Spring Splasher weakens a defender's creature when it attacks.
#[test]
fn spring_splasher_attack_debuff() {
    let mut g = two_player_game();
    let splasher = g.add_card_to_battlefield(0, catalog::spring_splasher());
    let blocker = g.add_card_to_battlefield(1, catalog::gigantosaurus()); // 10/10
    g.clear_sickness(splasher);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: splasher, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(blocker).unwrap().power, 7, "-3/-0 on the defender's creature");
}

/// Raven of Fell Omens drains 1 when you commit a crime, once per turn.
#[test]
fn raven_of_fell_omens_crime_drain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::raven_of_fell_omens());
    let opp = g.players[1].life;
    let me = g.players[0].life;
    g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "opponent loses 1");
    assert_eq!(g.players[0].life, me + 1, "you gain 1");
    // Second crime same turn does nothing (once each turn).
    g.dispatch_triggers_for_events(&[GameEvent::CommittedCrime { player: 0 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "only once per turn");
}

/// Stagecoach Security pumps the team +1/+1 and grants vigilance on entry.
#[test]
fn stagecoach_security_team_pump() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sec = g.add_card_to_battlefield(0, catalog::stagecoach_security());
    g.fire_self_etb_triggers(sec, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "bear 2/2 → 3/3");
    assert!(cp.keywords.contains(&crabomination::card::Keyword::Vigilance), "and vigilance");
}
