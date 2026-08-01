//! Mercadian Masques (MMQ) gap closure, fourth wave.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: idx,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
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

/// Cho-Manno shrugs off both combat and noncombat damage.
#[test]
fn cho_manno_takes_no_damage_at_all() {
    let mut g = two_player_game();
    let cho = g.add_card_to_battlefield(0, catalog::cho_manno_revolutionary()); // 2/2
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(cho)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cho).is_some());
    assert_eq!(g.battlefield_find(cho).unwrap().damage, 0);
}

/// Drake Hatchling's pump is once each turn.
#[test]
fn drake_hatchling_pumps_only_once_a_turn() {
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(0, catalog::drake_hatchling());
    mana(&mut g, 0);
    activate(&mut g, drake, 0, None);
    assert_eq!(g.computed_permanent(drake).unwrap().power, 2);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: drake,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "already used this turn"
    );
}

/// Pious Warrior turns the combat damage it takes into life.
#[test]
fn pious_warrior_converts_damage_into_life() {
    let mut g = two_player_game();
    let warrior = g.add_card_to_battlefield(0, catalog::pious_warrior()); // 2/3
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let life = g.players[0].life;
    attack_and_block(&mut g, warrior, blocker);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2);
}

/// Quagmire Lamprey drops a -1/-1 counter on whatever blocks it.
#[test]
fn quagmire_lamprey_shrinks_its_blocker() {
    let mut g = two_player_game();
    let lamprey = g.add_card_to_battlefield(0, catalog::quagmire_lamprey());
    let blocker = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    attack_and_block(&mut g, lamprey, blocker);
    assert_eq!(
        g.battlefield_find(blocker).unwrap().counter_count(CounterType::MinusOneMinusOne),
        1
    );
}

/// Saber Ants spawns an Insect for every point of damage it soaks.
#[test]
fn saber_ants_spawns_an_insect_per_damage() {
    let mut g = two_player_game();
    let ants = g.add_card_to_battlefield(0, catalog::saber_ants()); // 2/3
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(ants)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let insects = g.battlefield.iter().filter(|c| c.definition.name == "Insect").count();
    assert_eq!(insects, 3, "one per point of damage");
}

/// Pangosaur bounces itself whenever *anyone* plays a land.
#[test]
fn pangosaur_bounces_on_any_land_drop() {
    let mut g = two_player_game();
    let saur = g.add_card_to_battlefield(0, catalog::pangosaur());
    let land = g.add_card_to_hand(1, catalog::forest());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    let evs = g.perform_action(GameAction::PlayLand(land)).expect("play land");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(saur).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == saur));
}

/// Sustenance eats a land for a pump.
#[test]
fn sustenance_trades_a_land_for_a_pump() {
    let mut g = two_player_game();
    let sus = g.add_card_to_battlefield(0, catalog::sustenance());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    activate(&mut g, sus, 0, Some(Target::Permanent(bears)));
    assert!(g.battlefield_find(forest).is_none(), "the land was the cost");
    assert_eq!(g.computed_permanent(bears).unwrap().power, 3);
}

/// Pretender's Claim taps the defender's whole mana base when its host is
/// blocked.
#[test]
fn pretenders_claim_taps_the_defenders_lands() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::hill_giant());
    let their_land = g.add_card_to_battlefield(1, catalog::forest());
    let your_land = g.add_card_to_battlefield(0, catalog::forest());
    let claim = g.add_card_to_hand(0, catalog::pretenders_claim());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: claim,
        target: Some(Target::Permanent(host)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    attack_and_block(&mut g, host, blocker);
    assert!(g.battlefield_find(their_land).unwrap().tapped);
    assert!(!g.battlefield_find(your_land).unwrap().tapped);
}

/// Puffer Extract's pump comes with a death sentence at the next end step.
#[test]
fn puffer_extract_kills_what_it_pumps() {
    let mut g = two_player_game();
    let extract = g.add_card_to_battlefield(0, catalog::puffer_extract());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: extract,
        ability_index: 0,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 5);
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
}

/// Territorial Dispute locks lands down for *both* players.
#[test]
fn territorial_dispute_locks_everyones_land_drops() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::territorial_dispute());
    for seat in [0, 1] {
        let land = g.add_card_to_hand(seat, catalog::forest());
        g.active_player_idx = seat;
        g.priority.player_with_priority = seat;
        g.step = TurnStep::PreCombatMain;
        assert!(g.perform_action(GameAction::PlayLand(land)).is_err(), "seat {seat} is locked");
    }
}

/// Territorial Dispute eats a land each upkeep, then itself.
#[test]
fn territorial_dispute_eats_a_land_each_upkeep() {
    let mut g = two_player_game();
    let dispute = g.add_card_to_battlefield(0, catalog::territorial_dispute());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest).is_none());
    assert!(g.battlefield_find(dispute).is_some());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(dispute).is_none(), "nothing left to feed it");
}

/// Sand Squid pins a creature down for as long as the Squid stays tapped.
#[test]
fn sand_squid_locks_a_creature_while_tapped() {
    let mut g = two_player_game();
    let squid = g.add_card_to_battlefield(0, catalog::sand_squid());
    g.clear_sickness(squid);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, squid, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).unwrap().tapped);
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(victim).unwrap().tapped, "still locked");
}

/// Righteous Indignation pumps a creature that blocks a black or red one.
#[test]
fn righteous_indignation_pumps_the_blocker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::righteous_indignation());
    let attacker = g.add_card_to_battlefield(0, catalog::deathgazer()); // black
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    attack_and_block(&mut g, attacker, blocker);
    assert_eq!(g.computed_permanent(blocker).unwrap().power, 3);
    let _ = attacker;
}

/// Saprazzan Breaker slips through when its mill hits a land.
#[test]
fn saprazzan_breaker_is_unblockable_after_milling_a_land() {
    let mut g = two_player_game();
    let breaker = g.add_card_to_battlefield(0, catalog::saprazzan_breaker());
    g.add_card_to_library(0, catalog::forest());
    mana(&mut g, 0);
    activate(&mut g, breaker, 0, None);
    assert!(
        g.computed_permanent(breaker).unwrap().keywords.contains(&Keyword::Unblockable),
        "a land was milled"
    );
}
