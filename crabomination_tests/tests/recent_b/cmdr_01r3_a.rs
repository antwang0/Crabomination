//! Commander: the 2026-10-21 (`01R3xh4r`) one-gap pair closers
//! (`decks::cmdr_01r3_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// CR 702.160 + 707.12 — Arcane Proxy's ETB reads its own power: cast as a
/// 2/1 prototype it can't take a mana value 3 sorcery, cast as a 4/3 it exiles
/// Divination and casts a copy (two cards drawn), the card staying in exile.
#[test]
fn cr_702_160_arcane_proxy_target_is_capped_by_its_power() {
    let mut g = pod(2);
    let div = g.add_card_to_graveyard(0, catalog::divination());
    let proxy = g.add_card_to_hand(0, catalog::arcane_proxy());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastPrototype {
        card_id: proxy,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("prototype cast");
    let hand = g.players[0].hand.len();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "a 2/1 can't reach mana value 3");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == div));

    let mut g = pod(2);
    let div = g.add_card_to_graveyard(0, catalog::divination());
    let proxy = g.add_card_to_hand(0, catalog::arcane_proxy());
    let hand = g.players[0].hand.len() - 1;
    cast(&mut g, proxy, None);
    assert_eq!(g.players[0].hand.len(), hand + 2, "the copy of Divination resolved");
    assert!(g.exile.iter().any(|c| c.id == div), "the card stays exiled");
}

/// Worldpurge: every permanent goes to its owner's hand, each seat keeps at
/// most seven (APNAP, CR 101.4) and shuffles the rest in, and the caster's
/// floating mana is gone (an effect, not the step-end empty — CR 500.4).
#[test]
fn worldpurge_bounces_then_trims_each_hand_to_seven() {
    let mut g = pod(3);
    let purge = g.add_card_to_hand(0, catalog::worldpurge());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..8 {
        g.add_card_to_hand(1, catalog::island());
    }
    g.add_card_to_hand(2, catalog::island());
    let lib1 = g.players[1].library.len();
    cast(&mut g, purge, None);
    assert!(g.battlefield.is_empty());
    assert_eq!(g.players[1].hand.len(), 7, "nine in hand, seven kept");
    assert_eq!(g.players[1].library.len(), lib1 + 2);
    assert!(g.players[1].hand.iter().any(|c| c.id == bears), "the headless seat keeps its spell");
    assert_eq!(g.players[2].hand.len(), 1, "a short hand keeps everything");
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

fn activate(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<Vec<GameEvent>, GameError> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target, additional_targets: vec![], x_value: None, mode: None })
}

/// CR 611.2c — Braided Net's lock lasts "for as long as it remains tapped":
/// the tapped Shivan Dragon can't pump, and once it untaps the lock is gone.
/// The net spends a counter per use.
#[test]
fn cr_611_2c_braided_net_locks_only_while_the_target_stays_tapped() {
    let mut g = pod(3);
    let net = g.add_card_to_battlefield(0, catalog::braided_net());
    g.battlefield_find_mut(net).unwrap().add_counters(CounterType::Net, 3);
    let dragon = g.add_card_to_battlefield(2, catalog::shivan_dragon());
    activate(&mut g, 0, net, Some(Target::Permanent(dragon))).expect("net");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).unwrap().tapped);
    assert_eq!(g.battlefield_find(net).unwrap().counter_count(CounterType::Net), 2);
    assert!(activate(&mut g, 2, dragon, None).is_err(), "locked while tapped");
    g.battlefield_find_mut(dragon).unwrap().tapped = false;
    g.check_state_based_actions();
    activate(&mut g, 2, dragon, None).expect("untapped, the lock has ended");
}

/// CR 702.26 — Disciple of Caelus Nin: each seat keeps five permanents
/// (turn order from the caster, CR 101.4), the rest phase out and stay out
/// while it is on the battlefield; once it leaves they phase in on their
/// controller's untap step.
#[test]
fn cr_702_26_disciple_of_caelus_nin_holds_the_board_phased_out() {
    let mut g = pod(3);
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..7 {
        g.add_card_to_battlefield(2, catalog::island());
    }
    let disciple = g.add_card_to_hand(0, catalog::disciple_of_caelus_nin());
    cast(&mut g, disciple, None);
    let count = |g: &GameState, seat| g.battlefield.iter().filter(|c| c.controller == seat).count();
    assert_eq!((count(&g, 0), count(&g, 1), count(&g, 2)), (6, 1, 5), "five kept and the Disciple");
    assert_eq!(g.phased_out.len(), 3);
    g.active_player_idx = 2;
    g.do_phasing();
    assert_eq!(g.phased_out.len(), 3, "permanents can't phase in");
    let pos = g.battlefield.iter().position(|c| c.id == disciple).unwrap();
    g.battlefield.remove(pos);
    g.do_phasing();
    assert_eq!(count(&g, 2), 7, "the lock is gone");
}

/// CR 702.29c — Deem Worthy cycled: the "may" trigger deals 2 to a creature.
#[test]
fn cr_702_29c_deem_worthy_cycling_deals_two() {
    let mut g = pod(2);
    let ogre = g.add_card_to_battlefield(1, catalog::hill_giant());
    let deem = g.add_card_to_hand(0, catalog::deem_worthy());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::Cycle { card_id: deem, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ogre).unwrap().damage, 2);
}

/// Dire Wolves bands only while its controller has a Plains (CR 702.22).
#[test]
fn cr_702_22_dire_wolves_band_with_a_plains() {
    let mut g = pod(2);
    let wolves = g.add_card_to_battlefield(0, catalog::dire_wolves());
    let bands = |g: &GameState| g.computed_permanent(wolves).unwrap().keywords().contains(&Keyword::Banding);
    assert!(!bands(&g));
    g.add_card_to_battlefield(1, catalog::plains());
    assert!(!bands(&g), "an opponent's Plains doesn't count");
    g.add_card_to_battlefield(0, catalog::plains());
    assert!(bands(&g));
}
