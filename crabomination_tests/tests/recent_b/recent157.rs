//! Functionality tests for `catalog::sets::decks::recent157` (BLB gaps).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// Advance to the active player's declare-attackers step and swing with `id`.
fn attack_with(g: &mut GameState, id: CardId) {
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
}

fn team_power(g: &GameState, seat: usize) -> i32 {
    g.battlefield
        .iter()
        .filter(|c| c.controller == seat && c.definition.is_creature())
        .filter_map(|c| g.computed_permanent(c.id).map(|p| p.power))
        .sum()
}

/// Darkstar Augur's upkeep draws the top card and loses life equal to its MV.
#[test]
fn darkstar_augur_draws_and_loses_life() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    let cid = g.next_id();
    g.players[0].add_to_library_top(cid, catalog::serra_angel()); // 5 MV
    let augur = g.add_card_to_battlefield(0, catalog::darkstar_augur());
    g.clear_sickness(augur);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew the top card");
    assert_eq!(life - g.players[0].life, 5, "lost life equal to Serra Angel's MV");
}

/// Honored Dreyleader enters with a +1/+1 counter per other Squirrel/Food.
#[test]
fn honored_dreyleader_enters_scaled_by_food() {
    let mut g = two_player_game();
    g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token());
    g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token());
    g.move_card_to_battlefield_for_test(0, catalog::honored_dreyleader());
    drain_stack(&mut g);
    let dl = g.battlefield.iter().find(|c| c.definition.name == "Honored Dreyleader").unwrap().id;
    assert_eq!(g.computed_permanent(dl).unwrap().power, 3, "1/1 + two counters from two Food");
}

/// Fecund Greenshell's +2/+2 anthem switches on at ten lands.
#[test]
fn fecund_greenshell_anthem_at_ten_lands() {
    let mut g = two_player_game();
    let shell = g.add_card_to_battlefield(0, catalog::fecund_greenshell());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..9 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "only nine lands → no anthem");
    g.add_card_to_battlefield(0, catalog::forest());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "ten lands → +2/+2");
    assert_eq!(g.computed_permanent(shell).unwrap().power, 6, "anthem hits itself too");
}

/// Hazardroot Herbalist's attack trigger pumps a creature you control.
#[test]
fn hazardroot_herbalist_pumps_on_attack() {
    let mut g = two_player_game();
    let herb = g.add_card_to_battlefield(0, catalog::hazardroot_herbalist());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let before = team_power(&g, 0);
    attack_with(&mut g, herb);
    assert_eq!(team_power(&g, 0), before + 1, "+1/+0 to a creature you control");
}

/// Rust-Shield Rampager can't be blocked by power 2 or less.
#[test]
fn rust_shield_rampager_evasion() {
    let mut g = two_player_game();
    let r = g.add_card_to_battlefield(0, catalog::rust_shield_rampager());
    assert!(g
        .computed_permanent(r)
        .unwrap()
        .keywords
        .contains(&Keyword::CantBeBlockedByPowerAtMost(2)));
}

/// Seedpod Squire's attack pumps a non-flying creature you control.
#[test]
fn seedpod_squire_pumps_grounded_ally() {
    let mut g = two_player_game();
    let squire = g.add_card_to_battlefield(0, catalog::seedpod_squire());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    attack_with(&mut g, squire);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "grounded bear got +1/+1");
}

/// Steampath Charger's death deals 1 to a player.
#[test]
fn steampath_charger_death_pings() {
    let mut g = two_player_game();
    let charger = g.add_card_to_battlefield(0, catalog::steampath_charger());
    let life = g.players[1].life;
    let mut evs = g.remove_to_graveyard_with_triggers(charger);
    evs.push(GameEvent::CreatureDied { card_id: charger });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "death dealt 1 damage");
}

/// Treeguard Duo's ETB grants +X/+X where X is creatures you control.
#[test]
fn treeguard_duo_pumps_by_creature_count() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let before = team_power(&g, 0);
    g.move_card_to_battlefield_for_test(0, catalog::treeguard_duo());
    drain_stack(&mut g);
    // Two creatures you control at resolution → +2/+2 to one of them.
    assert_eq!(team_power(&g, 0), before + 3 /* Duo's own 3 power */ + 2);
}

/// Junkblade Bruiser grows when you expend 4.
#[test]
fn junkblade_bruiser_expend_pumps() {
    let mut g = two_player_game();
    let bruiser = g.add_card_to_battlefield(0, catalog::junkblade_bruiser());
    let moose = g.add_card_to_hand(0, catalog::galewind_moose()); // {4}{G}{G}
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast a 6-mana spell (crosses expend 4)");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bruiser).unwrap().power, 6, "expend 4 → +2/+1");
}

/// Waterspout Warden gains flying when another creature entered this turn.
#[test]
fn waterspout_warden_conditional_flying() {
    let mut g = two_player_game();
    let warden = g.add_card_to_battlefield(0, catalog::waterspout_warden());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].creatures_entered_this_turn.push(ally);
    attack_with(&mut g, warden);
    assert!(
        g.computed_permanent(warden).unwrap().keywords.contains(&Keyword::Flying),
        "gained flying after another creature entered this turn"
    );
}
