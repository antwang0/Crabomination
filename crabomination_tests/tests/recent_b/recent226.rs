//! Functionality tests for `catalog::sets::decks::recent226`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::two_player_game;

/// Doorkeeper Thrull suppresses an entering artifact's ETB trigger.
#[test]
fn doorkeeper_thrull_suppresses_artifact_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::doorkeeper_thrull());
    // Sandstorm Salvager's ETB (make a Golem) must not fire under the suppressor.
    let sal = g.add_card_to_battlefield(0, catalog::sandstorm_salvager());
    let etb_fired = crabomination::game::actions::etb_trigger_multiplier(&g, 0, Some(sal));
    assert_eq!(etb_fired, 0, "artifact/creature ETB triggers suppressed");
}

/// Sanctuary Wall taps and stuns a target and itself.
#[test]
fn sanctuary_wall_taps_and_stuns() {
    let mut g = two_player_game();
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wall = g.add_card_to_battlefield(0, catalog::sanctuary_wall());
    let effect = catalog::sanctuary_wall().activated_abilities[0].effect.clone();
    let ctx = EffectContext { targets: vec![Target::Permanent(enemy)], ..EffectContext::for_ability(wall, 0, None) };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield_find(enemy).unwrap().tapped, "target tapped");
    assert!(g.battlefield_find(enemy).unwrap().counter_count(crabomination::card::CounterType::Stun) > 0);
    assert!(g.battlefield_find(wall).unwrap().counter_count(crabomination::card::CounterType::Stun) > 0);
}

/// All-Out Assault buffs the team with +1/+1 and deathtouch.
#[test]
fn all_out_assault_anthem_and_deathtouch() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::all_out_assault());
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "+1/+1");
    assert!(cp.keywords.contains(&Keyword::Deathtouch));
}

/// Homicide Investigator investigates when a creature you control dies.
#[test]
fn homicide_investigator_investigates_on_death() {
    let mut g = two_player_game();
    let inv = g.add_card_to_battlefield(0, catalog::homicide_investigator());
    let effect = catalog::homicide_investigator().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(inv, 0, None, 0)).unwrap();
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Clue").count(),
        1,
    );
}

/// Lead Pipe grants +2/+0 to the creature it equips.
#[test]
fn lead_pipe_buffs_equipped() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pipe = g.add_card_to_battlefield(0, catalog::lead_pipe());
    g.battlefield_find_mut(pipe).unwrap().attached_to = Some(bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "2 + 2 = 4");
}

/// Karlov Watchdog's battalion pumps the team.
#[test]
fn karlov_watchdog_battalion_pumps() {
    let mut g = two_player_game();
    let dog = g.add_card_to_battlefield(0, catalog::karlov_watchdog());
    let effect = catalog::karlov_watchdog().triggered_abilities[0].effect.clone();
    g.resolve_effect(&effect, &EffectContext::for_trigger(dog, 0, None, 0)).unwrap();
    assert_eq!(g.computed_permanent(dog).unwrap().power, 4, "3 + 1 = 4");
}

/// No Witnesses wraths the board and investigates.
#[test]
fn no_witnesses_destroys_all_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.resolve_effect(&catalog::no_witnesses().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    crabomination::game::drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == a || c.id == b), "all creatures destroyed");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Clue").count(), 1);
}
