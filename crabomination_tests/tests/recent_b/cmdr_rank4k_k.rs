//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_k`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::mana::Color;
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn has(g: &GameState, id: CardId, k: Keyword) -> bool {
    g.computed_permanent(id).unwrap().keywords().contains(&k)
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack {
                attacker,
                target: AttackTarget::Player(p),
            })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// CR 701.20 — Gollum's guess: exactly one branch runs — removed from combat,
/// or a card drawn and unblockable.
#[test]
fn gollum_runs_exactly_one_branch() {
    let mut g = pod(2);
    let gollum = ready(&mut g, 0, catalog::gollum_scheming_guide());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    declare(&mut g, vec![(gollum, 1)]);
    let attacking = g.attacking.iter().any(|a| a.attacker == gollum);
    let drew = g.players[0].hand.len() == hand + 1;
    assert!(attacking == drew, "removed and drew nothing, or kept attacking and drew");
    assert_eq!(has(&g, gollum, Keyword::Unblockable), drew);
}

/// An empty library is a right guess: Gollum leaves combat.
#[test]
fn gollum_with_an_empty_library_leaves_combat() {
    let mut g = pod(2);
    let gollum = ready(&mut g, 0, catalog::gollum_scheming_guide());
    g.players[0].library.clear();
    declare(&mut g, vec![(gollum, 1)]);
    assert!(!g.attacking.iter().any(|a| a.attacker == gollum));
}

/// Poison-Blade Mentor gives another attacking Assassin deathtouch.
#[test]
fn poison_blade_mentor_arms_an_assassin() {
    let mut g = pod(2);
    let mentor = ready(&mut g, 0, catalog::poison_blade_mentor());
    let other = ready(&mut g, 0, catalog::poison_blade_mentor());
    std::sync::Arc::make_mut(g.battlefield_find_mut(other).unwrap().definition_mut()).keywords.clear();
    declare(&mut g, vec![(mentor, 1)]);
    assert!(has(&g, other, Keyword::Deathtouch));
}

fn tapped(g: &GameState, id: CardId) -> bool {
    g.battlefield_find(id).unwrap().tapped
}

/// Mudslide: a grounded creature skips its untap step, a flyer doesn't, and
/// the upkeep {2} buys the grounded one back.
#[test]
fn mudslide_holds_ground_creatures_until_paid_for() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mudslide());
    for seat in 0..2 {
        g.add_card_to_library(seat, catalog::island());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bird = g.add_card_to_battlefield(1, catalog::wind_drake());
    let lands: Vec<CardId> =
        (0..2).map(|_| g.add_card_to_battlefield(1, catalog::forest())).collect();
    for id in [bear, bird].into_iter().chain(lands.iter().copied()) {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    assert!(g.untap_prevented_by_static(bear));
    assert!(!g.untap_prevented_by_static(bird));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.active_player_idx == 0 || g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(!tapped(&g, bird), "the flyer untaps normally");
    assert!(!tapped(&g, bear), "the {{2}} at upkeep untapped the bear");
    assert!(lands.iter().all(|&l| tapped(&g, l)), "both lands paid for it");
}

/// With no mana to pay, the grounded creature stays tapped.
#[test]
fn mudslide_keeps_an_unpaid_creature_tapped() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::mudslide());
    for seat in 0..2 {
        g.add_card_to_library(seat, catalog::island());
    }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    while g.active_player_idx == 0 || g.step != TurnStep::Draw {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert!(tapped(&g, bear));
}

/// String of Disappearances: the bounced creature's controller pays {U}{U}
/// and points the copy back at the caster's creature.
#[test]
fn string_of_disappearances_chains_back_at_the_caster() {
    let mut g = pod(2);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::string_of_disappearances());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(mine)),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("castable");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "bounced home");
    assert!(g.players[0].hand.iter().any(|c| c.id == mine), "the copy bounced mine");
}
