//! Commander: two-card gaps of the top-2000 commanders (`decks::cmdr_rank2k_m`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
}

/// Hans Eriksson drops a revealed creature into the attack, where it fights
/// him; a noncreature card goes to hand.
#[test]
fn hans_eriksson_reveals_into_the_fight() {
    for creature in [true, false] {
        let mut g = pod(2);
        let top = if creature {
            g.add_card_to_library(0, catalog::grizzly_bears())
        } else {
            g.add_card_to_library(0, catalog::island())
        };
        let hans = ready(&mut g, 0, catalog::hans_eriksson());
        g.step = TurnStep::DeclareAttackers;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: hans, target: AttackTarget::Player(1) }]))
            .expect("attack");
        drain_stack(&mut g);
        if creature {
            assert!(g.attacking_ids().contains(&top), "joined the attack");
            assert_eq!(g.battlefield_find(hans).unwrap().damage, 2, "the Bears fought Hans");
        } else {
            assert!(g.players[0].hand.iter().any(|c| c.id == top));
        }
    }
}

/// Mwonvuli Beast Tracker stacks a trampler on top.
#[test]
fn mwonvuli_beast_tracker_tops_a_trampler() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let liege = g.add_card_to_library(0, catalog::liege_of_the_tangle());
    g.add_card_to_library(0, catalog::island());
    let tracker = g.add_card_to_hand(0, catalog::mwonvuli_beast_tracker());
    cast(&mut g, tracker, None);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(liege));
}

/// Treacherous Urge borrows a creature from an opponent's hand, hasty.
#[test]
fn treacherous_urge_borrows_a_creature() {
    let mut g = pod(2);
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    let urge = g.add_card_to_hand(0, catalog::treacherous_urge());
    cast(&mut g, urge, Some(Target::Player(1)));
    let c = g.battlefield_find(bear).expect("on the battlefield");
    assert_eq!(c.controller, 0);
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&crabomination::card::Keyword::Haste));
}

fn convoke(g: &mut GameState, id: CardId, helpers: &[CardId]) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellConvoke {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        convoke_creatures: helpers.to_vec(),
    })
    .expect("convoke cast");
    drain_stack(g);
}

/// Keensight Mentor hands out vigilance, then counters the vigilant.
#[test]
fn keensight_mentor_rewards_vigilance() {
    let mut g = pod(2);
    let elf = ready(&mut g, 0, catalog::elvish_warrior());
    let mentor = g.add_card_to_hand(0, catalog::keensight_mentor());
    cast(&mut g, mentor, Some(Target::Permanent(elf)));
    assert!(g.computed_permanent(elf).unwrap().keywords().contains(&Keyword::Vigilance));
    g.clear_sickness(mentor);
    act(&mut g, GameAction::ActivateAbility {
        card_id: mentor,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert_eq!(g.battlefield_find(elf).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Zephyr Singer gives each convoker a flying counter.
#[test]
fn zephyr_singer_lifts_its_convokers() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let singer = g.add_card_to_hand(0, catalog::zephyr_singer());
    convoke(&mut g, singer, &[a]);
    assert!(g.computed_permanent(a).unwrap().keywords().contains(&Keyword::Flying));
    assert!(!g.computed_permanent(b).unwrap().keywords().contains(&Keyword::Flying));
}

/// Knight-Errant of Eos takes creatures up to its convoker count.
#[test]
fn knight_errant_of_eos_digs_by_convokers() {
    let mut g = pod(2);
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let angel = g.add_card_to_library(0, catalog::serra_angel());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let knight = g.add_card_to_hand(0, catalog::knight_errant_of_eos());
    convoke(&mut g, knight, &[a, b]);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "mana value 2, two convokers");
    assert!(!g.players[0].hand.iter().any(|c| c.id == angel), "mana value 5 is too big");
}
