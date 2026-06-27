//! Functionality tests for `catalog::sets::decks::tla` — TLA staples.

use crate::catalog;
use crate::card::Keyword;
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;
use crate::TurnStep;

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == controller && c.definition.name == name)
        .count()
}

fn attack_with(g: &mut GameState, atk: CardId) {
    g.clear_sickness(atk);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
}

/// Cat-Gator deals damage equal to the Swamps you control on ETB.
#[test]
fn cat_gator_pings_for_swamps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::swamp());
    let cg = g.add_card_to_battlefield(0, catalog::cat_gator());
    let before = g.players[1].life;
    g.fire_self_etb_triggers(cg, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 2, "2 damage = 2 Swamps");
}

/// Cat-Owl untaps a target permanent when it attacks.
#[test]
fn cat_owl_untaps_on_attack() {
    let mut g = two_player_game();
    let owl = g.add_card_to_battlefield(0, catalog::cat_owl());
    let mana_rock = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mana_rock).unwrap().tapped = true;
    attack_with(&mut g, owl);
    assert!(!g.battlefield_find(mana_rock).unwrap().tapped, "target untapped");
}

/// Kyoshi Warriors makes a 1/1 Ally on ETB.
#[test]
fn kyoshi_warriors_makes_ally() {
    let mut g = two_player_game();
    let kw = g.add_card_to_battlefield(0, catalog::kyoshi_warriors());
    g.fire_self_etb_triggers(kw, 0);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Ally"), 1);
}

/// The Walls of Ba Sing Se grant indestructible to your other permanents.
#[test]
fn walls_grant_indestructible() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::walls_of_ba_sing_se());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Indestructible),
        "other permanent gains indestructible"
    );
}

/// Wandering Musicians pump the team +1/+0 on attack.
#[test]
fn wandering_musicians_team_pump() {
    let mut g = two_player_game();
    let wm = g.add_card_to_battlefield(0, catalog::wandering_musicians());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    attack_with(&mut g, wm);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+0 → 3 power");
}
