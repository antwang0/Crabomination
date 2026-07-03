//! Functionality tests for the `catalog::sets::decks::recent98` Kamigawa: Neon
//! Dynasty batch 4.

use crate::catalog;
use crate::card::{CounterType, Keyword};
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;

fn pass_through_combat(g: &mut GameState) {
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(g);
}

/// Nezumi Bladeblesser gains deathtouch/menace from artifacts/enchantments.
#[test]
fn nezumi_bladeblesser_conditional_keywords() {
    let mut g = two_player_game();
    let nezumi = g.add_card_to_battlefield(0, catalog::nezumi_bladeblesser());
    assert!(!g.computed_permanent(nezumi).unwrap().keywords.contains(&Keyword::Deathtouch));
    g.add_card_to_battlefield(0, catalog::bonesplitter()); // artifact
    assert!(g.computed_permanent(nezumi).unwrap().keywords.contains(&Keyword::Deathtouch));
    assert!(!g.computed_permanent(nezumi).unwrap().keywords.contains(&Keyword::Menace));
    g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); // enchantment
    assert!(g.computed_permanent(nezumi).unwrap().keywords.contains(&Keyword::Menace));
}

/// Iron Apprentice enters as a 1/1 and moves its counter on death.
#[test]
fn iron_apprentice_moves_counter_on_death() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let iron = g.move_card_to_battlefield_for_test(0, catalog::iron_apprentice());
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(iron).unwrap().power, 1, "0/0 + counter = 1/1");
    g.remove_to_graveyard_with_triggers(iron);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "counter moved to the bear"
    );
}

/// Circuit Mender gains 2 life on entry and draws when it leaves.
#[test]
fn circuit_mender_etb_and_ltb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.players[0].life = 20;
    let mender = g.add_card_to_battlefield(0, catalog::circuit_mender());
    g.fire_self_etb_triggers(mender, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22, "ETB gained 2 life");
    let hand_before = g.players[0].hand.len();
    g.remove_to_graveyard_with_triggers(mender);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "LTB drew a card");
}

/// Dragonfly Suit is a flying Vehicle with Crew 1.
#[test]
fn dragonfly_suit_is_a_crewable_flyer() {
    let def = catalog::dragonfly_suit();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Crew(1)));
}

/// Moon-Circuit Hacker attacks and deals combat damage; helper returns the
/// hand-size delta. `entered_this_turn` toggles the "discard unless it entered
/// this turn" rider.
fn moon_circuit_hacker_hand_delta(entered_this_turn: bool) -> i64 {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard, if forced
    let hacker = g.add_card_to_battlefield(0, catalog::moon_circuit_hacker());
    if entered_this_turn {
        g.battlefield_find_mut(hacker).unwrap().entered_turn = Some(g.turn_number);
    }
    g.clear_sickness(hacker);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand_before = g.players[0].hand.len() as i64;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: hacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("hacker attacks");
    drain_stack(&mut g);
    pass_through_combat(&mut g);
    g.players[0].hand.len() as i64 - hand_before
}

/// Draws (net +1) when it entered this turn — the discard rider is skipped.
#[test]
fn moon_circuit_hacker_no_discard_when_fresh() {
    assert_eq!(moon_circuit_hacker_hand_delta(true), 1);
}

/// Draws then discards (net 0) when it's been around since a prior turn.
#[test]
fn moon_circuit_hacker_discards_when_established() {
    assert_eq!(moon_circuit_hacker_hand_delta(false), 0);
}

/// Kaito's Pursuit makes the opponent discard two and gives your Ninjas menace.
#[test]
fn kaitos_pursuit_discards_and_grants_menace() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    let ninja = g.add_card_to_battlefield(0, catalog::dokuchi_shadow_walker());
    let spell = g.add_card_to_hand(0, catalog::kaitos_pursuit());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Kaito's Pursuit");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "opponent discarded two");
    assert!(
        g.computed_permanent(ninja).unwrap().keywords.contains(&Keyword::Menace),
        "your Ninja gained menace"
    );
}

/// Bearer of Memory pumps a target enchantment creature.
#[test]
fn bearer_of_memory_counters_enchantment_creature() {
    let mut g = two_player_game();
    let bearer = g.add_card_to_battlefield(0, catalog::bearer_of_memory());
    let target = g.add_card_to_battlefield(0, catalog::golden_tail_disciple()); // enchantment creature
    for _ in 0..5 { g.players[0].mana_pool.add_colorless(1); }
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bearer,
        ability_index: 0,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate Bearer of Memory");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "enchantment creature got a +1/+1 counter"
    );
    assert!(g.computed_permanent(target).unwrap().keywords.contains(&Keyword::Trample));
}

/// Dokuchi Shadow-Walker is a 5/5 with Ninjutsu.
#[test]
fn dokuchi_shadow_walker_stats() {
    let def = catalog::dokuchi_shadow_walker();
    assert_eq!((def.power, def.toughness), (5, 5));
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Ninjutsu(_))));
}

/// Reito Sentinel mills three on entry.
#[test]
fn reito_sentinel_mills_on_etb() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(1, catalog::island()); }
    let sentinel = g.add_card_to_battlefield(0, catalog::reito_sentinel());
    let gy_before = g.players[1].graveyard.len();
    g.fire_self_etb_triggers(sentinel, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy_before + 3, "opponent milled three");
}

/// Akki Ronin loots on a lone Samurai attack.
#[test]
fn akki_ronin_loots_on_solo_attack() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears()); // to discard
    g.add_card_to_library(0, catalog::island()); // to draw
    let ronin = g.add_card_to_battlefield(0, catalog::akki_ronin());
    g.clear_sickness(ronin);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ronin,
        target: AttackTarget::Player(1),
    }]))
    .expect("ronin attacks alone");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "looted: discard one, draw one");
}
