//! Commander: one-card gaps of EDHREC ranks 3501-4500 (`decks::cmdr_rank4k_k`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
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
