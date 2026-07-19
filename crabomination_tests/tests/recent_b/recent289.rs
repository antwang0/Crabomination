//! Functionality tests for `catalog::sets::decks::recent289` (OTJ Vehicles +
//! Wylie Duke).

use crabomination::catalog;
use crabomination::card::CreatureType;
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::{drain_stack, two_player_game, GameAction, GameEvent, GameState};
use crabomination::TurnStep;

fn ready(g: &mut GameState) {
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
}

fn attack_with(g: &mut GameState, attacker: crabomination::card::CardId) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(g);
}

/// Luxurious Locomotive makes a Treasure for each creature that crewed it.
#[test]
fn luxurious_locomotive_treasures_per_crewer() {
    let mut g = two_player_game();
    let loco = g.add_card_to_battlefield(0, catalog::luxurious_locomotive());
    let c1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(c1);
    g.clear_sickness(c2);
    ready(&mut g);
    g.perform_action(GameAction::Crew { vehicle: loco, crew_creatures: vec![c1, c2] }).expect("crew");
    attack_with(&mut g, loco);
    let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 2, "one Treasure per crewer");
}

/// Mobile Homestead has haste only while you control a Mount, and deploys a
/// revealed top land when it attacks.
#[test]
fn mobile_homestead_haste_and_land_deploy() {
    let mut g = two_player_game();
    let home = g.add_card_to_battlefield(0, catalog::mobile_homestead());
    let has_haste = |g: &GameState| {
        g.computed_permanent(home).unwrap().keywords.contains(&crabomination::card::Keyword::Haste)
    };
    assert!(!has_haste(&g), "no Mount → no haste");
    // Add a Mount → haste.
    let mut mount = catalog::grizzly_bears();
    mount.subtypes.creature_types = vec![CreatureType::Mount];
    g.add_card_to_battlefield(0, mount);
    assert!(has_haste(&g), "controlling a Mount grants haste");
    // Attack with a Forest on top → it enters tapped.
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::forest());
    ready(&mut g);
    g.clear_sickness(home);
    // Crew the vehicle so it can attack as a creature.
    let crew = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(crew);
    g.perform_action(GameAction::Crew { vehicle: home, crew_creatures: vec![crew] }).expect("crew");
    attack_with(&mut g, home);
    let forest = g.battlefield.iter().find(|c| c.definition.name == "Forest").expect("land deployed");
    assert!(forest.tapped, "the deployed land enters tapped");
}

/// Wylie Duke draws and gains life whenever it becomes tapped.
#[test]
fn wylie_duke_draws_on_tap() {
    let mut g = two_player_game();
    let wylie = g.add_card_to_battlefield(0, catalog::wylie_duke_atiin_hero());
    g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: wylie, actor: None }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    assert_eq!(g.players[0].life, life_before + 1, "gained 1 life");
}
