//! Commander: the 2026-10-21 (`01R3xh4r`) one-gap pair closers
//! (`decks::cmdr_01r3_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Selector, Value};
use crabomination::game::effects::EffectContext;
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

/// Geistflame Reservoir banks a charge counter per instant or sorcery cast,
/// then spends any number of them as damage (the X removed, CR 107.3).
#[test]
fn geistflame_reservoir_banks_spells_and_fires_x() {
    let mut g = pod(2);
    let res = g.add_card_to_battlefield(0, catalog::geistflame_reservoir());
    for _ in 0..3 {
        let opt = g.add_card_to_hand(0, catalog::opt());
        cast(&mut g, opt, None);
    }
    assert_eq!(g.battlefield_find(res).unwrap().counter_count(CounterType::Charge), 3);
    let life = g.players[1].life;
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: res,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("fire two");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
    assert_eq!(g.battlefield_find(res).unwrap().counter_count(CounterType::Charge), 1);
}

/// Invasion of Arcavios fetches an instant or sorcery from the graveyard as
/// well as the library (CR 903.11: nothing waits outside the game).
#[test]
fn invasion_of_arcavios_finds_a_graveyard_sorcery() {
    let mut g = pod(2);
    let div = g.add_card_to_graveyard(0, catalog::divination());
    let inv = g.add_card_to_hand(0, catalog::invasion_of_arcavios());
    cast(&mut g, inv, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == div));
}

/// Fungal Behemoth's P/T counts +1/+1 counters on every creature its
/// controller has — not an opponent's.
#[test]
fn fungal_behemoth_counts_counters_on_your_creatures() {
    let mut g = pod(2);
    let fb = g.add_card_to_battlefield(0, catalog::fungal_behemoth());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(theirs).unwrap().add_counters(CounterType::PlusOnePlusOne, 5);
    g.battlefield_find_mut(fb).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let cp = g.computed_permanent(fb).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "three counters, plus its own +1/+1");
}

/// Feral Encounter exiles a creature from the top five face up, castable this
/// turn; the other four go to the bottom.
#[test]
fn feral_encounter_exiles_a_creature_castable_this_turn() {
    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let fe = g.add_card_to_hand(0, catalog::feral_encounter());
    cast(&mut g, fe, None);
    let ex = g.exile.iter().find(|c| c.id == bears).expect("the creature is exiled");
    assert!(!ex.face_down && ex.may_play_until.is_some());
    assert_eq!(g.players[0].library.len(), 7);
}

/// CR 706.6 — Vedalken Squirrel-Whacker enters as the two rolls (5/6), and a
/// d6 its controller later rolls as a 1 is exchanged with the larger base stat:
/// the roll reads 6 and the creature becomes 5/1.
#[test]
fn cr_706_6_squirrel_whacker_rolls_its_body_and_trades_a_low_roll() {
    let mut g = pod(2);
    let whacker = g.add_card_to_hand(0, catalog::vedalken_squirrel_whacker());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(5), DecisionAnswer::DieRoll(6)]));
    cast(&mut g, whacker, None);
    let cp = g.computed_permanent(whacker).expect("a */* survives its as-enters roll");
    assert_eq!((cp.power, cp.toughness), (5, 6));
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(1)]));
    let roll = Effect::RollDie {
        sides: 6,
        count: Value::ONE,
        modifier: Value::Const(0),
        reroll_at_most: 0,
        ignore_lowest: 0,
        results: vec![(6, 6, Effect::Draw { who: Selector::You, amount: Value::ONE })],
        on_doubles: None,
    };
    g.resolve_effect(&roll, &EffectContext::for_ability(whacker, 0, None)).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "the 1 became a 6");
    let cp = g.computed_permanent(whacker).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 1));
}

/// CR 614.1a — Soul Echo: once the opponent chooses, damage to its controller
/// removes echo counters instead, until that player's next turn; life at 0
/// doesn't lose the game.
#[test]
fn cr_614_1a_soul_echo_turns_damage_into_counters_until_your_next_upkeep() {
    let mut g = pod(3);
    let echo = g.add_card_to_hand(0, catalog::soul_echo());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: echo, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("cast X=3");
    drain_stack(&mut g);
    let count = |g: &GameState| g.battlefield_find(echo).unwrap().counter_count(CounterType::Echo);
    assert_eq!(count(&g), 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let stamp = Effect::PlayerMayStampSourceTurn { who: Selector::Player(PlayerRef::Seat(1)) };
    g.resolve_effect(&stamp, &EffectContext::for_ability(echo, 0, None)).unwrap();
    let hit = |g: &mut GameState, n| {
        let dmg = Effect::DealDamage { to: Selector::Player(PlayerRef::Seat(0)), amount: Value::Const(n) };
        g.resolve_effect(&dmg, &EffectContext::for_ability(echo, 1, None)).unwrap();
    };
    let life = g.players[0].life;
    hit(&mut g, 2);
    assert_eq!((g.players[0].life, count(&g)), (life, 1));
    g.turn_number += 1;
    g.active_player_idx = 1;
    hit(&mut g, 5);
    assert_eq!((g.players[0].life, count(&g)), (life, 0), "still replaced, with nothing left to remove");
    g.turn_number += 1;
    g.active_player_idx = 0;
    hit(&mut g, life);
    assert_eq!(g.players[0].life, 0, "the controller's own turn ends the replacement");
    g.check_state_based_actions();
    assert!(g.players[0].is_alive(), "0 life doesn't lose");
}
