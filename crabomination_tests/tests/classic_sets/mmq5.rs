//! Mercadian Masques (MMQ) gap closure, fifth wave.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

/// Install a scripted decider that answers with `answers`, then AutoDecider.
fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
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

/// Seat 0's `attacker` attacks seat 1 and is blocked by seat 1's `blocker`.
fn attack_and_block(g: &mut GameState, attacker: CardId, blocker: CardId) {
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    drain_stack(g);
}

/// Statecraft blanks combat damage in both directions for its controller.
#[test]
fn statecraft_seals_combat_damage_both_ways() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::statecraft());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    attack_and_block(&mut g, mine, theirs);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 0, "incoming sealed");
    assert_eq!(g.battlefield_find(theirs).unwrap().damage, 0, "outgoing sealed");
}

/// Insubordination punishes a host that stayed home, and spares one that swung.
#[test]
fn insubordination_bites_a_creature_that_didnt_attack() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::insubordination());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);

    g.clear_sickness(host);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: host, target: AttackTarget::Player(0) }])
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "it attacked, so no bite");
}

/// Barbed Wire's {2} shield soaks its own upkeep ping.
#[test]
fn barbed_wire_can_buy_off_its_own_damage() {
    let mut g = two_player_game();
    let wire = g.add_card_to_battlefield(0, catalog::barbed_wire());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wire,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the shield ate the ping");
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "shield spent");
}

/// Battle Squadron counts your creatures, itself included.
#[test]
fn battle_squadron_sizes_to_your_board() {
    let mut g = two_player_game();
    let squad = g.add_card_to_battlefield(0, catalog::battle_squadron());
    assert_eq!(g.computed_permanent(squad).unwrap().power, 1);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(squad).unwrap().power, 2);
}

/// Bribery pulls a creature out of the opponent's library onto your side.
#[test]
fn bribery_steals_from_the_opponents_library() {
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::grizzly_bears());
    let bear = g.players[1].library.last().map(|c| c.id).expect("seeded");
    let bribery = g.add_card_to_hand(0, catalog::bribery());
    script(&mut g, vec![DecisionAnswer::Search(Some(bear))]);
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bribery,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Grizzly Bears"),
        "stolen under your control"
    );
}

/// Renounce trades permanents for 2 life each.
#[test]
fn renounce_pays_two_life_per_sacrifice() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let renounce = g.add_card_to_hand(0, catalog::renounce());
    script(&mut g, vec![DecisionAnswer::Amount(2)]);
    cast(&mut g, 0, renounce, None);
    assert_eq!(g.players[0].life, 24);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 0);
}

/// Invigorate's alt cost is free — an opponent just gains 3.
#[test]
fn invigorate_casts_free_by_gifting_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let inv = g.add_card_to_hand(0, catalog::invigorate());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: inv,
        pitch_card: None,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 23);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6);
}

/// Orim's Cure taps a creature instead of paying mana.
#[test]
fn orims_cure_taps_a_creature_as_its_alt_cost() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::plains());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cure = g.add_card_to_hand(0, catalog::orims_cure());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: cure,
        pitch_card: None,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "tapped as a cost");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    cast(&mut g, 1, bolt, Some(Target::Player(0)));
    assert_eq!(g.players[0].life, 20, "shield soaked all 3");
}

/// Ferocity grows its host when it meets a blocker.
#[test]
fn ferocity_counters_up_on_a_block() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::ferocity());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    attack_and_block(&mut g, attacker, host);
    assert_eq!(
        g.battlefield_find(host).and_then(|c| c.counters.get(&CounterType::PlusOnePlusOne)),
        Some(&1)
    );
}

/// Volcanic Wind's damage total is the creature count on resolution.
#[test]
fn volcanic_wind_scales_with_the_board() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wind = g.add_card_to_hand(0, catalog::volcanic_wind());
    script(&mut g, vec![DecisionAnswer::DamageDivision(vec![2, 0])]);
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: wind,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none(), "took both points");
    assert!(g.battlefield_find(b).is_some());
}

/// Puppet's Verdict sweeps the small creatures on heads.
#[test]
fn puppets_verdict_kills_by_power_on_the_flip() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    let verdict = g.add_card_to_hand(0, catalog::puppets_verdict());
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    cast(&mut g, 0, verdict, None);
    assert!(g.battlefield_find(small).is_none());
    assert!(g.battlefield_find(big).is_some());
}

/// Nether Spirit climbs back while it's the graveyard's only creature card.
#[test]
fn nether_spirit_returns_when_it_is_alone() {
    let mut g = two_player_game();
    let spirit = g.add_card_to_graveyard(0, catalog::nether_spirit());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(spirit).is_some());
}
