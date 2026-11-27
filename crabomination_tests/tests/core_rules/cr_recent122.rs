//! CR 702.132a — Assist: another player the caster chooses may pay any amount
//! of the spell's generic mana before the caster pays the rest. Also CR 701.9b
//! random discards.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::TurnStep;

/// Seat 0 holds Spellweaver Duo ({6}{U}, assist) with `own` Islands; every
/// other seat has `others` untapped Islands.
fn table(n: usize, own: usize, others: usize) -> (GameState, CardId) {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    for seat in 0..n {
        for _ in 0..if seat == 0 { own } else { others } {
            g.add_card_to_battlefield(seat, catalog::island());
        }
    }
    let duo = g.add_card_to_hand(0, catalog::spellweaver_duo());
    (g, duo)
}

fn cast(g: &mut GameState, id: CardId) -> Result<Vec<GameEvent>, GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
}

fn untapped(g: &GameState, seat: usize) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && !c.tapped).count()
}

/// CR 702.132a — a teammate (Two-Headed Giant, CR 810) pays the generic part
/// it can; the caster pays only {2}{U} of {6}{U}.
#[test]
fn cr_702_132a_a_teammate_pays_the_generic_mana() {
    let (mut g, duo) = table(4, 3, 4);
    g.assign_teams(vec![vec![0, 2], vec![1, 3]]).unwrap();
    cast(&mut g, duo).expect("assisted cast");
    assert_eq!((untapped(&g, 0), untapped(&g, 2), untapped(&g, 1)), (0, 0, 4));
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == duo));
}

/// CR 702.132a — the caster names any other player, who pays any amount; a
/// scripted seat names seat 2 and pays {2}.
#[test]
fn cr_702_132a_the_chosen_player_pays_any_amount() {
    let (mut g, duo) = table(3, 5, 3);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(2), DecisionAnswer::Amount(2)]));
    cast(&mut g, duo).expect("assisted cast");
    assert_eq!((untapped(&g, 0), untapped(&g, 1), untapped(&g, 2)), (0, 3, 1));
}

/// CR 702.132a / 601.2h — the caster's share unpaid, the cast fails and the
/// assister's mana is restored.
#[test]
fn cr_702_132a_a_failed_cast_gives_the_assister_its_mana_back() {
    let (mut g, duo) = table(3, 1, 3);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(1), DecisionAnswer::Amount(3)]));
    assert!(cast(&mut g, duo).is_err());
    assert_eq!((untapped(&g, 0), untapped(&g, 1)), (1, 3));
    assert!(g.players[0].hand.iter().any(|c| c.id == duo));
}

/// CR 702.132a — choosing an assister is optional, and a headless opponent
/// with nothing to gain declines: the caster pays it all or can't cast.
#[test]
fn cr_702_132a_a_headless_opponent_does_not_assist_for_nothing() {
    let (mut g, duo) = table(4, 3, 4);
    assert!(cast(&mut g, duo).is_err());
    assert!((1..4).all(|q| untapped(&g, q) == 4));
}

/// CR 701.9b — a random discard draws its cards off the seeded RNG (bug fix:
/// `Effect::Discard { random: true }` always took the first cards in hand).
/// Over a few seeds Hymn to Tourach hits the back of the hand, and a seed
/// replays its pick exactly.
#[test]
fn cr_701_9b_a_random_discard_is_random_and_seeded() {
    let hymn = |seed: u64| {
        let mut g = multi_player_game(2);
        g.rng = crabomination::game::rng::GameRng::seeded(seed);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::PreCombatMain;
        let hand: Vec<CardId> = (0..4).map(|_| g.add_card_to_hand(1, catalog::grizzly_bears())).collect();
        let h = g.add_card_to_hand(0, catalog::hymn_to_tourach());
        g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
        g.perform_action(GameAction::CastSpell {
            card_id: h,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        let kept: Vec<usize> = (0..4).filter(|&i| g.players[1].hand.iter().any(|c| c.id == hand[i])).collect();
        assert_eq!(kept.len(), 2);
        kept
    };
    assert!((0..16).any(|s| hymn(s) != vec![2, 3]), "some seed discards from the back of the hand");
    assert_eq!(hymn(7), hymn(7));
}
