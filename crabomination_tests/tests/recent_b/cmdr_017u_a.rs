//! Commander: the 2026-10-19 (`017ujEs3`) Wan Shi Tong closers
//! (`decks::cmdr_017u_a`) — "cards put into a library from anywhere".

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn duel() -> GameState {
    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    g.players[seat].mana_pool.add(Color::Blue, 6);
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn counters(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// Wan Shi Tong's ruling (2025-10-02): "triggers only when cards are put into a
/// library from somewhere other than the library" — a scry is no trigger; a
/// permanent tucked is one (CR 400.7: the card left the battlefield); and
/// Brainstorm's two cards back from hand are ONE trigger ("one or more").
#[test]
fn put_into_a_library_counts_arrivals_not_reorders() {
    let mut g = duel();
    let seeker = g.add_card_to_battlefield(0, catalog::dutiful_knowledge_seeker());
    flood(&mut g, 0);
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast(&mut g, opt, None);
    assert_eq!(counters(&g, seeker), 0, "a scry reorders the library: no trigger");

    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let evict = g.add_card_to_hand(0, catalog::synchronized_eviction());
    cast(&mut g, evict, Some(Target::Permanent(bear)));
    assert_eq!(g.players[1].library[1].id, bear, "second from the top of its owner's library");
    assert_eq!(counters(&g, seeker), 1, "a tucked permanent was put into a library");

    let bs = g.add_card_to_hand(0, catalog::brainstorm());
    cast(&mut g, bs, None);
    assert_eq!(counters(&g, seeker), 2, "drawn then put back from hand: one batch, one trigger");
}

/// Dutiful Knowledge Seeker's {3}: a card from any graveyard to the bottom of
/// its owner's library — which is itself a card put into a library.
#[test]
fn dutiful_knowledge_seeker_tucks_a_graveyard_card_and_grows() {
    let mut g = duel();
    let seeker = g.add_card_to_battlefield(0, catalog::dutiful_knowledge_seeker());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: seeker,
        ability_index: 0,
        target: Some(Target::Permanent(dead)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(dead), "bottom of its owner's library");
    assert_eq!(counters(&g, seeker), 1);
}

/// Wan Shi Tong, All-Knowing tucks an opponent's permanent on entering, and
/// that tuck is the first batch its own trigger sees: two Spirits.
#[test]
fn wan_shi_tong_tucks_and_makes_two_spirits() {
    let mut g = duel();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wst = g.add_card_to_hand(0, catalog::wan_shi_tong_all_knowing());
    flood(&mut g, 0);
    cast(&mut g, wst, None);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the bear left the battlefield");
    assert!(g.players[1].library.iter().any(|c| c.id == bear), "into its owner's library");
    let spirits = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Spirit").count();
    assert_eq!(spirits, 2);
}

/// Swat Away hits a spell: the owner's choice of top or bottom of their
/// library (the auto owner takes the top), and the spell never resolves.
#[test]
fn swat_away_puts_a_spell_into_its_owners_library() {
    let mut g = duel();
    g.players[1].mana_pool.add(Color::Green, 2);
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast bears");
    g.perform_action(GameAction::PassPriority).expect("pass");
    let swat = g.add_card_to_hand(0, catalog::swat_away());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: swat,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("swat");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "the spell never resolved");
    assert!(g.players[1].library.iter().any(|c| c.id == bears), "into its owner's library");
}

/// Baboon Spirit: another nontoken Spirit entering makes one Spirit token; the
/// token's own entry doesn't feed it again.
#[test]
fn baboon_spirit_answers_nontoken_spirits_only() {
    let mut g = duel();
    g.add_card_to_battlefield(0, catalog::baboon_spirit());
    let seeker = g.add_card_to_hand(0, catalog::dutiful_knowledge_seeker());
    flood(&mut g, 0);
    cast(&mut g, seeker, None);
    let tokens = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Spirit").count();
    assert_eq!(tokens, 1);
}
