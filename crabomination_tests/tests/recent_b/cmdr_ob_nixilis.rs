//! Commander: the Sworn to Darkness precon (C14, Ob Nixilis of the Black
//! Oath, `decks::cmdr_ob_nixilis`) and the primitives it needed.

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

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) -> Result<(), String> {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

/// CR 508.1d — "attacks that player this combat if able": the creature must
/// be declared, and only against the opponent stamped on it.
#[test]
fn cr_508_1d_must_attack_chosen_player_binds_attacker_and_defender() {
    let mut g = pod(3);
    let mut def = catalog::grizzly_bears();
    def.keywords.push(Keyword::MustAttackChosenPlayer);
    let bear = g.add_card_to_battlefield(0, def);
    g.clear_sickness(bear);
    g.battlefield_find_mut(bear).unwrap().chosen_player = Some(2);

    let snapshot = g.clone();
    assert!(declare(&mut g, vec![]).is_err(), "it must attack");
    let mut g = snapshot.clone();
    assert!(declare(&mut g, vec![(bear, 1)]).is_err(), "and only the chosen opponent");
    let mut g = snapshot.clone();
    declare(&mut g, vec![(bear, 2)]).expect("attacking the chosen opponent is legal");
}

/// CR 508.1d — with its chosen seat gone (CR 800.4a), nothing binds it.
#[test]
fn cr_508_1d_must_attack_chosen_player_lapses_when_that_player_left() {
    let mut g = pod(3);
    let mut def = catalog::grizzly_bears();
    def.keywords.push(Keyword::MustAttackChosenPlayer);
    let bear = g.add_card_to_battlefield(0, def);
    g.clear_sickness(bear);
    g.battlefield_find_mut(bear).unwrap().chosen_player = Some(2);
    g.players[2].eliminated = true;
    declare(&mut g, vec![]).expect("no live chosen opponent — no requirement");
}
