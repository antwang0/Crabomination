//! Legends (LEG) wave 8 — the set's last nine cards (`catalog::sets::leg7`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
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

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Run to the start of the next turn so untap/upkeep roll-overs really fire.
fn end_turn(g: &mut GameState) {
    let started = g.turn_number;
    while g.turn_number == started {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

// ── All Hallow's Eve ───────────────────────────────────────────────────────

#[test]
fn all_hallows_eve_exiles_itself_with_two_scream_counters() {
    let mut g = main_phase();
    let eve = g.add_card_to_hand(0, catalog::all_hallows_eve());
    cast(&mut g, 0, eve, None);
    let card = g.exile.iter().find(|c| c.id == eve).expect("exiled");
    assert_eq!(card.counter_count(CounterType::Scream), 2);
}

#[test]
fn all_hallows_eve_reanimates_every_graveyard_when_the_fuse_burns_out() {
    let mut g = main_phase();
    let eve = g.add_card_to_hand(0, catalog::all_hallows_eve());
    let mine = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    cast(&mut g, 0, eve, None);
    // Two of seat 0's upkeeps take both counters off.
    for _ in 0..4 {
        end_turn(&mut g);
    }
    assert!(g.battlefield_find(mine).is_some());
    assert!(g.battlefield_find(theirs).is_some());
    assert!(g.players[0].graveyard.iter().any(|c| c.id == eve), "the fuse ends in the graveyard");
}

// ── Arboria ────────────────────────────────────────────────────────────────

#[test]
fn arboria_locks_out_attacks_on_an_idle_player() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::arboria());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    // Seat 1 has done nothing on a turn of their own.
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
            .is_err()
    );
}

#[test]
fn arboria_lets_you_hit_a_player_who_cast_a_spell_on_their_turn() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::arboria());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Seat 1's turn: they cast something.
    end_turn(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    // Back to seat 0.
    end_turn(&mut g);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
            .is_ok()
    );
}

// ── Backdraft ──────────────────────────────────────────────────────────────

#[test]
fn backdraft_deals_half_a_sorcerys_damage_back_at_its_caster() {
    let mut g = main_phase();
    // Seat 1 casts a 5-damage sorcery at seat 0 on their own turn.
    let axe = g.add_card_to_hand(1, catalog::lava_axe());
    g.active_player_idx = 1;
    cast(&mut g, 1, axe, Some(Target::Player(0)));
    g.active_player_idx = 0;
    let before = g.players[1].life;
    let back = g.add_card_to_hand(0, catalog::backdraft());
    cast(&mut g, 0, back, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, before - 2, "half of Lava Axe's 5, rounded down");
}

#[test]
fn backdraft_cant_target_a_player_who_cast_no_sorcery() {
    let mut g = main_phase();
    let back = g.add_card_to_hand(0, catalog::backdraft());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: back,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err()
    );
}

// ── Chains of Mephistopheles ───────────────────────────────────────────────

#[test]
fn chains_turns_an_extra_draw_into_discard_then_draw() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::chains_of_mephistopheles());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    let mut events = Vec::new();
    assert!(g.draw_one(0, &mut events));
    assert_eq!(g.players[0].hand.len(), hand, "one discarded, one drawn");
    assert_eq!(g.players[0].graveyard.len(), 1);
}

#[test]
fn chains_mills_when_the_hand_is_empty() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::chains_of_mephistopheles());
    g.players[0].hand.clear();
    let lib = g.players[0].library.len();
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    assert!(g.players[0].hand.is_empty());
    assert_eq!(g.players[0].library.len(), lib - 1);
    assert_eq!(g.players[0].graveyard.len(), 1);
}

#[test]
fn chains_leaves_the_turn_based_draw_step_draw_alone() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::chains_of_mephistopheles());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    end_turn(&mut g);
    end_turn(&mut g);
    while g.step != TurnStep::PreCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1, "the draw step's first draw is exempt");
    assert!(g.players[0].graveyard.is_empty());
}

// ── Equinox ────────────────────────────────────────────────────────────────

#[test]
fn equinox_counters_a_spell_aimed_at_one_of_your_lands() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let aura = g.add_card_to_hand(0, catalog::equinox());
    cast(&mut g, 0, aura, Some(Target::Permanent(land)));
    let stone = g.add_card_to_hand(1, catalog::stone_rain());
    g.active_player_idx = 1;
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: stone,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Stone Rain");
    // The enchanted land's granted ability sits after its printed ones.
    let index = g.battlefield_find(land).expect("land").definition.activated_abilities.len();
    activate(&mut g, 0, land, index, Some(Target::Permanent(stone)));
    assert!(g.battlefield_find(land).is_some(), "Stone Rain never resolved");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == stone));
}

// ── Knowledge Vault ────────────────────────────────────────────────────────

#[test]
fn knowledge_vault_trades_your_hand_for_the_stash() {
    let mut g = main_phase();
    let vault = g.add_card_to_battlefield(0, catalog::knowledge_vault());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..3 {
        activate(&mut g, 0, vault, 0, None);
        if let Some(c) = g.battlefield_find_mut(vault) {
            c.tapped = false;
        }
    }
    assert_eq!(g.exile.iter().filter(|c| c.exiled_with == Some(vault)).count(), 3);
    activate(&mut g, 0, vault, 1, None);
    assert_eq!(g.players[0].hand.len(), 3, "the stash arrives, the old hand is gone");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

#[test]
fn knowledge_vault_burns_the_stash_if_it_leaves_another_way() {
    let mut g = main_phase();
    let vault = g.add_card_to_battlefield(0, catalog::knowledge_vault());
    activate(&mut g, 0, vault, 0, None);
    let stashed = g.exile.iter().find(|c| c.exiled_with == Some(vault)).expect("stash").id;
    let bolt = g.add_card_to_hand(1, catalog::naturalize());
    cast(&mut g, 1, bolt, Some(Target::Permanent(vault)));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == stashed));
}

// ── Land Equilibrium ───────────────────────────────────────────────────────

#[test]
fn land_equilibrium_taxes_a_caught_up_opponent() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::land_equilibrium());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(1, catalog::forest());
    let played = g.add_card_to_hand(1, catalog::mountain());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::PlayLand(played)).expect("land");
    drain_stack(&mut g);
    let lands = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_land()).count();
    assert_eq!(lands, 1, "the new land came with a sacrifice");
}

#[test]
fn land_equilibrium_spares_an_opponent_who_is_behind() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::land_equilibrium());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let played = g.add_card_to_hand(1, catalog::mountain());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::PlayLand(played)).expect("land");
    drain_stack(&mut g);
    let lands = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_land()).count();
    assert_eq!(lands, 1, "1 < 3, so nothing is sacrificed");
}

// ── Reverberation ──────────────────────────────────────────────────────────

#[test]
fn reverberation_turns_a_sorcery_on_its_caster() {
    let mut g = main_phase();
    let axe = g.add_card_to_hand(1, catalog::lava_axe());
    g.active_player_idx = 1;
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: axe,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Lava Axe");
    let rev = g.add_card_to_hand(0, catalog::reverberation());
    cast(&mut g, 0, rev, Some(Target::Permanent(axe)));
    assert_eq!(g.players[0].life, 20, "the damage never reached seat 0");
    assert_eq!(g.players[1].life, 20 - 5, "it landed on the caster instead");
}

// ── Wall of Caltrops ───────────────────────────────────────────────────────

#[test]
fn wall_of_caltrops_bands_with_another_wall() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let caltrops = g.add_card_to_battlefield(0, catalog::wall_of_caltrops());
    let other = g.add_card_to_battlefield(0, catalog::wall_of_stone());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(caltrops, attacker), (other, attacker)]))
        .expect("block");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(caltrops).expect("wall").keywords.contains(&Keyword::Banding),
        "two Walls and nothing else blocking grants banding"
    );
}

#[test]
fn wall_of_caltrops_stays_plain_when_a_non_wall_joins_the_block() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let caltrops = g.add_card_to_battlefield(0, catalog::wall_of_caltrops());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(caltrops, attacker), (bear, attacker)]))
        .expect("block");
    drain_stack(&mut g);
    assert!(
        !g.computed_permanent(caltrops).expect("wall").keywords.contains(&Keyword::Banding)
    );
}
