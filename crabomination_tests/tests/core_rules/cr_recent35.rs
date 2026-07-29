//! CR conformance for multi-block (the modern_decks `block_map` fan-out):
//! - CR 509.1b — a creature blocks one attacker unless an effect says otherwise
//!   (`CanBlockAdditional` / `CanBlockAnyNumber`).
//! - CR 509.2 / 510.1e — a creature blocking several attackers divides its
//!   combat damage among them; it doesn't deal full power to each.
//! - CR 506.4 — a blocker leaving combat stops blocking every attacker.
//! - CR 603.6e — Lumbering Battlement's any-number exile-until-it-leaves.

use crabomination::card::{CardDefinition, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;

/// Two 2/2 attackers into one defender creature, stopped at declare-blockers.
fn two_attackers(blocker: fn() -> CardDefinition) -> (GameState, CardId, CardId, CardId) {
    let mut g = two_player_game();
    let a1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let a2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, blocker());
    g.clear_sickness(a1);
    g.clear_sickness(a2);
    g.active_player_idx = 0;
    g.attacking = vec![
        Attack { attacker: a1, target: AttackTarget::Player(1) },
        Attack { attacker: a2, target: AttackTarget::Player(1) },
    ];
    g.step = TurnStep::DeclareBlockers;
    (g, a1, a2, b)
}

/// CR 509.1b — a plain creature can't be declared against two attackers, and a
/// rejected batch leaves no partial state.
#[test]
fn cr_509_1b_plain_blocker_cant_block_two() {
    let (mut g, a1, a2, b) = two_attackers(catalog::grizzly_bears);
    assert!(g.declare_blockers(vec![(b, a1), (b, a2)]).is_err());
    assert!(g.block_map.is_empty());
}

/// CR 509.1b — `CanBlockAnyNumber` (Guardian of the Gateless) blocks both.
#[test]
fn cr_509_1b_can_block_any_number() {
    let (mut g, a1, a2, b) = two_attackers(catalog::guardian_of_the_gateless);
    g.declare_blockers(vec![(b, a1), (b, a2)]).expect("blocks both");
    assert_eq!(g.attackers_blocked_by(b), [a1, a2]);
    assert_eq!(g.blocker_count_of(a1), 1);
    assert_eq!(g.blocker_count_of(a2), 1);
}

/// CR 509.1b — `CanBlockAdditional(1)` (Knight of Sorrows) allows exactly two.
#[test]
fn cr_509_1b_can_block_additional_caps_at_two() {
    let mut g = two_player_game();
    let a: Vec<CardId> = (0..3)
        .map(|_| {
            let id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(id);
            id
        })
        .collect();
    let b = g.add_card_to_battlefield(1, catalog::knight_of_sorrows());
    g.active_player_idx = 0;
    g.attacking =
        a.iter().map(|&id| Attack { attacker: id, target: AttackTarget::Player(1) }).collect();
    g.step = TurnStep::DeclareBlockers;
    assert!(g.declare_blockers(vec![(b, a[0]), (b, a[1]), (b, a[2])]).is_err(), "1 + 1 is the cap");
    g.declare_blockers(vec![(b, a[0]), (b, a[1])]).expect("two is legal");
    assert_eq!(g.attackers_blocked_by(b).len(), 2);
}

/// Valor Made Real grants the allowance for the turn.
#[test]
fn valor_made_real_grants_multi_block() {
    let (mut g, a1, a2, b) = two_attackers(catalog::grizzly_bears);
    grant_any_number_block(&mut g, b);
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::CanBlockAnyNumber));
    g.declare_blockers(vec![(b, a1), (b, a2)]).expect("granted multi-block");
    assert_eq!(g.attackers_blocked_by(b).len(), 2);
}

/// CR 510.1e — a multi-block blocker divides its power instead of dealing full
/// power to each attacker: a 2/2 blocking two 2/2s kills only the first.
#[test]
fn cr_510_1e_multi_block_divides_blocker_damage() {
    let (mut g, a1, a2, b) = two_attackers(catalog::grizzly_bears);
    grant_any_number_block(&mut g, b);
    g.declare_blockers(vec![(b, a1), (b, a2)]).expect("blocks both");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(a1).is_none(), "lethal went to the first attacker in order");
    assert!(g.battlefield_find(a2).is_some(), "nothing left over to kill the second");
}

/// Cast Valor Made Real from seat 1 onto `target`.
fn grant_any_number_block(g: &mut GameState, target: CardId) {
    let cast = g.add_card_to_hand(1, catalog::valor_made_real());
    let prior = g.priority.player_with_priority;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(crabomination::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: cast,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Valor Made Real");
    drain_stack(g);
    g.priority.player_with_priority = prior;
}

/// CR 506.4 — a blocker leaving the battlefield stops blocking every attacker,
/// and both stay blocked (CR 510.1c).
#[test]
fn cr_506_4_removed_multi_blocker_clears_all_its_blocks() {
    let (mut g, a1, a2, b) = two_attackers(catalog::guardian_of_the_gateless);
    g.declare_blockers(vec![(b, a1), (b, a2)]).expect("blocks both");
    drain_stack(&mut g);
    g.remove_from_battlefield_to_graveyard_raw(b);
    assert!(g.block_map.is_empty(), "no dangling blocker entry");
    assert!(g.blocked_attackers().contains(&a1) && g.blocked_attackers().contains(&a2));
}

/// CR 509.3a — "whenever this creature blocks" fires once even when it blocks
/// two attackers; the pump reads the whole blocked set (+2/+2, not +2/+2 twice).
#[test]
fn cr_509_3a_blocks_trigger_fires_once_per_multi_block() {
    let (mut g, a1, a2, b) = two_attackers(catalog::guardian_of_the_gateless);
    let evs = g.declare_blockers(vec![(b, a1), (b, a2)]).expect("blocks both");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let cp = g.computed_permanent(b).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "3/3 + one +2/+2");
}

/// CR 509.3c — "whenever this creature becomes blocked" fires once even under a
/// double block (Rakdos Roustabout pings the defender a single time).
#[test]
fn cr_509_3c_becomes_blocked_trigger_fires_once_per_double_block() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::rakdos_roustabout());
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.active_player_idx = 0;
    g.attacking = vec![Attack { attacker: a, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    let evs = g.declare_blockers(vec![(b1, a), (b2, a)]).expect("double block");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
}

/// CR 509.3b — "whenever this blocks a creature" affects every attacker a
/// multi-blocker is blocking, from the single trigger instance.
#[test]
fn cr_509_3b_blocks_a_creature_reaches_every_blocked_attacker() {
    let (mut g, a1, a2, b) = two_attackers(catalog::wall_of_frost);
    grant_any_number_block(&mut g, b);
    let evs = g.declare_blockers(vec![(b, a1), (b, a2)]).expect("blocks both");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    for a in [a1, a2] {
        assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::Stun), 1);
    }
}

/// CR 603.6e — Lumbering Battlement exiles the chosen bodies until it leaves
/// and grows +2/+2 for each.
#[test]
fn lumbering_battlement_exiles_and_grows() {
    let mut g = two_player_game();
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let batt = g.move_card_to_battlefield_for_test(0, catalog::lumbering_battlement());
    drain_stack(&mut g);
    assert!(g.battlefield_find(friend).is_none(), "the bear is exiled with it");
    let cp = g.computed_permanent(batt).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 7), "4/5 + 2/+2 for one exiled card");
    g.remove_from_battlefield_to_graveyard_raw(batt);
    g.check_state_based_actions();
    assert!(g.battlefield_find(friend).is_some(), "returns when the Battlement leaves");
}

/// CR 611.2b — an "until your next turn" continuous effect survives the
/// intervening turn and expires as its player's next turn begins.
#[test]
fn cr_611_2b_until_your_next_turn_spans_one_turn_cycle() {
    use crabomination::game::layers::{
        AffectedPermanents, ContinuousEffect, EffectDuration, Layer, Modification, PtSublayer,
    };
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let timestamp = g.next_timestamp();
    let installed_turn = g.turn_number;
    g.add_continuous_effect(ContinuousEffect {
        timestamp,
        source: bear,
        affected: AffectedPermanents::Specific(vec![bear]),
        layer: Layer::L7PowerTough,
        sublayer: Some(PtSublayer::SetValue),
        duration: EffectDuration::UntilYourNextTurn { player: 0, installed_turn },
        modification: Modification::SetPowerToughness(6, 6),
    });
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6);

    // Drive turns until the opponent is active: still 6/6.
    advance_until_active(&mut g, 1);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6, "spans the intervening turn");

    // Back to player 0: expired.
    advance_until_active(&mut g, 0);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2, "expired on your next turn");
}

/// Step the game until `seat` is the active player.
fn advance_until_active(g: &mut GameState, seat: usize) {
    for _ in 0..200 {
        if g.active_player_idx == seat && g.step == TurnStep::Untap {
            return;
        }
        let _ = g.advance_step(Vec::new());
        if g.active_player_idx == seat {
            return;
        }
    }
    panic!("never reached seat {seat}");
}

/// CR 509.3e — "whenever this creature blocks two or more creatures" fires
/// once the block assignment reaches the count, and not on a single block.
#[test]
fn cr_509_3e_blocks_two_or_more_counts_the_batch() {
    let (mut g, a1, _a2, b) = two_attackers(catalog::lairwatch_giant);
    let evs = g.declare_blockers(vec![(b, a1)]).expect("single block");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(!g.computed_permanent(b).unwrap().keywords.contains(&Keyword::FirstStrike));

    let (mut g, a1, a2, b) = two_attackers(catalog::lairwatch_giant);
    let evs = g.declare_blockers(vec![(b, a1), (b, a2)]).expect("blocks both");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::FirstStrike));
}
