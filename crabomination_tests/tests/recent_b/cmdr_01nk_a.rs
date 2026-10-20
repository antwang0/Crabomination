//! Commander: the 2026-10-20 (`01NkAE5B`) one-gap pair closers
//! (`decks::cmdr_01nk_a`) and the per-object locks they need.

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, Value};
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

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>, x: Option<u32>, mode: Option<usize>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: more, mode, x_value: x })
        .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    cast_with(g, id, target, vec![], None, None);
}

fn counter_on_each_creature(g: &mut GameState) {
    let each = Effect::AddCounter {
        what: Selector::EachPermanent(crabomination::card::SelectionRequirement::Creature),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    g.resolve_effect(&each, &EffectContext::for_spell(0, None, 0, 0)).unwrap();
}

/// CR 122.1 / 614.17 — Melira's Keepers' "this creature can't have counters
/// put on it" locks only the Keepers (it was encoded as Solemnity's global
/// lock, which stopped counters on every permanent), and proliferate skips it.
#[test]
fn cr_614_17_cant_have_counters_is_per_object() {
    let mut g = pod(2);
    let keepers = g.add_card_to_battlefield(0, catalog::meliras_keepers());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    counter_on_each_creature(&mut g);
    assert_eq!(g.battlefield_find(keepers).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.resolve_effect(&Effect::Proliferate, &EffectContext::for_spell(1, None, 1, 0)).unwrap();
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(keepers).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// CR 701.26b / 614.17 — Blossombind taps its creature, which then neither
/// untaps in its controller's untap step nor by an effect, and takes no
/// counters; another creature still does.
#[test]
fn cr_701_26b_blossombind_locks_untap_and_counters() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::blossombind());
    cast(&mut g, aura, Some(Target::Permanent(bears)));
    assert!(g.battlefield_find(bears).unwrap().tapped, "entering, it taps the creature");
    counter_on_each_creature(&mut g);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let untap = Effect::Untap { what: Selector::Target(0), up_to: None };
    g.resolve_effect(&untap, &EffectContext::for_spell(1, Some(Target::Permanent(bears)), 1, 0)).unwrap();
    assert!(g.battlefield_find(bears).unwrap().tapped, "an effect can't untap it");
    // Seat 1's untap step.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(bears).unwrap().tapped, "nor can its untap step");
}

/// CR 615 — Purity turns noncombat damage to its controller into life; combat
/// damage and damage to other players are untouched.
#[test]
fn cr_615_purity_turns_noncombat_damage_into_life() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::purity());
    let life = g.players[0].life;
    let bolt = Effect::DealDamage { to: Selector::Player(crabomination::effect::PlayerRef::EachPlayer), amount: Value::Const(3) };
    g.resolve_effect(&bolt, &EffectContext::for_spell(1, None, 1, 0)).unwrap();
    assert_eq!(g.players[0].life, life + 3, "prevented, and gained");
    assert_eq!(g.players[2].life, life - 3);
}

/// Creeping Trailblazer pumps other Elementals' power and itself per Elemental.
#[test]
fn creeping_trailblazer_lords_and_pumps_per_elemental() {
    let mut g = pod(2);
    let tb = g.add_card_to_battlefield(0, catalog::creeping_trailblazer());
    let purity = g.add_card_to_battlefield(0, catalog::purity());
    assert_eq!(g.computed_permanent(purity).unwrap().power, 7);
    assert_eq!(g.computed_permanent(tb).unwrap().power, 2, "\"other\" Elementals");
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: tb,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .unwrap();
    drain_stack(&mut g);
    let cp = g.computed_permanent(tb).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "two Elementals");
}

/// Mantle of Leadership's creature gets +2/+2 whenever any creature enters.
#[test]
fn mantle_of_leadership_pumps_on_any_creature_entering() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mantle = g.add_card_to_hand(0, catalog::mantle_of_leadership());
    cast(&mut g, mantle, Some(Target::Permanent(bears)));
    let theirs = g.add_card_to_hand(1, catalog::grizzly_bears());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    g.perform_action(GameAction::CastSpell { card_id: theirs, target: None, additional_targets: vec![], mode: None, x_value: None })
        .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 4, "an opponent's creature counts");
}

/// Timeline Inquiry discards one unless teamwork was paid.
#[test]
fn timeline_inquiry_discards_unless_teamwork() {
    let mut g = pod(2);
    let ti = g.add_card_to_hand(0, catalog::timeline_inquiry());
    let hand = g.players[0].hand.len();
    cast(&mut g, ti, None);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3 - 1);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().summoning_sick = false;
    let ti = g.add_card_to_hand(0, catalog::timeline_inquiry());
    let hand = g.players[0].hand.len();
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked { card_id: ti, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("teamwork cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "teamwork skips the discard");
    assert!(g.battlefield_find(bears).unwrap().tapped);
}

/// CR 603.2 — Stormfront Riders bounces two of your creatures and makes a
/// Soldier for EACH creature returned to your hand (the kind fired once per
/// batch before it fanned out).
#[test]
fn stormfront_riders_bounces_two_and_makes_soldiers() {
    let mut g = pod(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let riders = g.add_card_to_hand(0, catalog::stormfront_riders());
    cast(&mut g, riders, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == a));
    assert!(g.players[0].hand.iter().any(|c| c.id == b));
    let soldiers = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 2);
}
