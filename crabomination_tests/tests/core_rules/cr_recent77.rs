//! CR conformance: 726 (the initiative), 905 (Conspiracy Draft) and the
//! 903 commander rules the engine already enforces.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
    g
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

// ── CR 726 — The Initiative ─────────────────────────────────────────────────

/// CR 726.2 — "Whenever you take the initiative … venture into Undercity."
/// The designation and the first room land together.
#[test]
fn cr_726_2_taking_the_initiative_ventures_into_undercity() {
    let mut g = main_phase();
    let id = g.add_card_to_hand(0, catalog::aarakocra_sneak());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.initiative, Some(0));
    assert_eq!(
        g.players[0].dungeon.as_ref().map(|(d, r)| (d.as_str(), *r)),
        Some(("Undercity", 0)),
        "Secret Entrance"
    );
}

/// CR 726.3 — only one player has the initiative; taking it strips the
/// previous holder. CR 726.2's combat clause is the handover.
#[test]
fn cr_726_3_combat_damage_moves_the_initiative() {
    let mut g = main_phase();
    let mut events = Vec::new();
    g.take_initiative(1, &mut events);
    drain_stack(&mut g);
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.initiative, Some(0), "the attacker's controller took it");
}

/// CR 726.2 — "At the beginning of the upkeep of the player who has the
/// initiative, that player ventures into Undercity."
#[test]
fn cr_726_2_upkeep_ventures_again() {
    let mut g = main_phase();
    let mut events = Vec::new();
    g.take_initiative(0, &mut events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].dungeon.as_ref().map(|(_, r)| *r), Some(0));
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert!(
        g.players[0].dungeon.as_ref().is_some_and(|(_, r)| *r > 0),
        "advanced past Secret Entrance"
    );
}

/// CR 726.5 — re-taking the initiative you already have still ventures, and
/// doesn't create a second designation.
#[test]
fn cr_726_5_retaking_ventures_without_duplicating() {
    let mut g = main_phase();
    let mut events = Vec::new();
    g.take_initiative(0, &mut events);
    drain_stack(&mut g);
    g.take_initiative(0, &mut events);
    drain_stack(&mut g);
    assert_eq!(g.initiative, Some(0));
    assert!(g.players[0].dungeon.as_ref().is_some_and(|(_, r)| *r > 0), "ventured twice");
}

/// A player mid-way through a different dungeon can't be dropped into
/// Undercity by "venture into Undercity".
#[test]
fn venture_into_undercity_skips_a_player_in_another_dungeon() {
    let mut g = main_phase();
    g.players[0].dungeon = Some(("Tomb of Annihilation".into(), 0));
    let mut events = Vec::new();
    g.take_initiative(0, &mut events);
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].dungeon.as_ref().map(|(d, r)| (d.as_str(), *r)),
        Some(("Tomb of Annihilation", 0)),
        "unchanged"
    );
}

/// Passageway Seer's end-step rider reads the live initiative.
#[test]
fn passageway_seer_grows_while_you_hold_the_initiative() {
    let mut g = main_phase();
    let seer = g.add_card_to_battlefield(0, catalog::passageway_seer());
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(seer).expect("seer").counter_count(CounterType::PlusOnePlusOne), 0);
    let mut events = Vec::new();
    g.take_initiative(0, &mut events);
    drain_stack(&mut g);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(seer).expect("seer").counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Undermountain Adventurer's mana ability scales off a completed dungeon.
#[test]
fn undermountain_adventurer_pays_six_after_a_dungeon() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(0, catalog::undermountain_adventurer());
    g.clear_sickness(giant);
    g.perform_action(GameAction::ActivateAbility {
        card_id: giant, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 2);
    g.players[0].mana_pool = Default::default();
    g.players[0].dungeons_completed = 1;
    g.battlefield_find_mut(giant).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: giant, ability_index: 0, target: None,
        additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 6);
}

/// CR 726.4 — when the initiative's holder leaves the game, the active player
/// takes it.
#[test]
fn cr_726_4_the_initiative_passes_when_its_holder_leaves() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    let mut events = Vec::new();
    g.take_initiative(1, &mut events);
    drain_stack(&mut g);
    g.players[1].life = 0;
    g.check_state_based_actions();
    assert_eq!(g.initiative, Some(0), "the active player picks it up");
}

// ── CR 905 — Conspiracy Draft ───────────────────────────────────────────────

/// CR 905.4 / 905.5 — a conspiracy starts in the command zone, owned and
/// controlled by the player who put it there, and never enters the battlefield.
#[test]
fn cr_905_4_conspiracies_start_in_the_command_zone() {
    let mut g = main_phase();
    let id = g.seat_conspiracy(0, catalog::weight_advantage(), None);
    assert!(g.battlefield_find(id).is_none(), "not a permanent");
    assert!(
        g.players[0].command.iter().any(|c| c.id == id && c.controller == 0),
        "controlled from the command zone by its owner"
    );
}

/// CR 905.4a — a hidden-agenda conspiracy starts face down and its abilities
/// do nothing until its controller turns it face up.
#[test]
fn cr_905_4a_hidden_agendas_start_face_down() {
    let mut g = main_phase();
    let id = g.seat_conspiracy(0, catalog::immediate_action(), Some("Grizzly Bears"));
    assert!(
        g.players[0].command.iter().any(|c| c.id == id && c.face_down),
        "face down until revealed"
    );
    assert!(g.reveal_hidden_agenda(0, id));
    assert!(g.players[0].command.iter().any(|c| c.id == id && !c.face_down));
}

/// CR 905.6 — each player starts a Conspiracy Draft game at 20 life.
#[test]
fn cr_905_6_players_start_at_twenty() {
    let g = main_phase();
    assert!(g.players.iter().all(|p| p.life == 20));
}

// ── CR 903 — Commander ──────────────────────────────────────────────────────

/// CR 903.8 — each previous cast from the command zone taxes the next by {2}.
#[test]
fn cr_903_8_commander_tax_is_two_generic_per_prior_cast() {
    let mut g = main_phase();
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    let cast = |g: &mut GameState| {
        mana(g, 0);
        let before = g.players[0].mana_pool.total();
        g.perform_action(GameAction::CastFromCommandZone {
            card_id: cmd, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .expect("cast");
        drain_stack(g);
        before - g.players[0].mana_pool.total()
    };
    assert_eq!(cast(&mut g), 2, "printed cost only");
    let mut events = Vec::new();
    g.destroy_permanent(cmd, false, &mut events);
    assert_eq!(cast(&mut g), 4, "printed cost plus the tax");
}

/// CR 903.10a — 21 combat damage from one commander eliminates the victim.
#[test]
fn cr_903_10a_twenty_one_commander_damage_eliminates() {
    let mut g = main_phase();
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    g.record_commander_damage(1, cmd, 21);
    g.check_state_based_actions();
    assert!(g.players[1].eliminated, "CR 903.10a");
}

/// A scripted ballot still reaches the Undercity's branching rooms.
#[test]
fn undercity_branches_are_offered_at_secret_entrance() {
    let mut g = main_phase();
    // Secret Entrance's search finds nothing; the branch pick then takes Forge.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(None),
        DecisionAnswer::Mode(0),
    ]));
    let mut events = Vec::new();
    g.take_initiative(0, &mut events);
    drain_stack(&mut g);
    g.take_initiative(0, &mut events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].dungeon.as_ref().map(|(_, r)| *r), Some(1), "Forge");
}
