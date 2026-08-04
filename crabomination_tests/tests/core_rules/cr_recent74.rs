//! CR conformance: 616.1e replacement choice, 121.2a Chains of Mephistopheles,
//! 509.2 banding, and 808 team-vs-team resources.

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{Decision, DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction};
use crabomination::game::*;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::mountain());
        }
    }
    g
}

/// Stand up Parallel Thoughts with a one-card exiled pile.
fn parallel_pile(g: &mut GameState) -> CardId {
    let src = g.add_card_to_battlefield(0, catalog::parallel_thoughts());
    let stashed = g.add_card_to_exile(0, catalog::grizzly_bears());
    if let Some(c) = g.exile.iter_mut().find(|c| c.id == stashed) {
        c.exiled_with = Some(src);
    }
    stashed
}

/// Archmage Ascension, quest complete.
fn live_ascension(g: &mut GameState) {
    let asc = g.add_card_to_battlefield(0, catalog::archmage_ascension());
    if let Some(c) = g.battlefield_find_mut(asc) {
        c.add_counters(CounterType::Quest, 6);
    }
}

// ── CR 616.1e — the affected player picks which replacement applies ────────

#[test]
fn cr_616_1e_drawing_player_picks_among_applicable_replacements() {
    let mut g = main_phase();
    let stashed = parallel_pile(&mut g);
    live_ascension(&mut g);
    g.players[0].wants_ui = true;
    // Mode(1) = Archmage's tutor, skipping the canonically-first exiled pile;
    // Bool(true) accepts it, and the search then finds nothing.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Mode(1),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(None),
    ]));
    let hand = g.players[0].hand.len();
    let mut events = Vec::new();
    assert!(g.draw_one(0, &mut events));
    assert!(g.exile.iter().any(|c| c.id == stashed), "the pile was not the chosen replacement");
    assert_eq!(g.players[0].hand.len(), hand, "the tutor applied and found nothing");
}

#[test]
fn cr_616_1e_declining_the_chosen_replacement_offers_the_rest() {
    let mut g = main_phase();
    let stashed = parallel_pile(&mut g);
    live_ascension(&mut g);
    g.players[0].wants_ui = true;
    // Pick the tutor, decline it, then take the pile on the second offer.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Mode(1),
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(true),
    ]));
    let mut events = Vec::new();
    assert!(g.draw_one(0, &mut events));
    assert!(g.players[0].hand.iter().any(|c| c.id == stashed), "the declined pick fell through");
}

#[test]
fn cr_616_1e_a_lone_replacement_is_not_a_choice() {
    let mut g = main_phase();
    let stashed = parallel_pile(&mut g);
    g.players[0].wants_ui = true;
    let mut decider = ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]);
    decider.asked.clear();
    g.decider = Box::new(decider);
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    assert!(g.players[0].hand.iter().any(|c| c.id == stashed));
    let asked = match g.decider.kind() {
        crabomination::decision::DeciderKind::Scripted { asked, .. } => asked,
        _ => unreachable!(),
    };
    assert!(
        !asked.iter().any(|d| matches!(d, Decision::ChooseMode { .. })),
        "one applicable replacement needs no CR 616.1e prompt"
    );
}

// ── CR 121.2a — Chains of Mephistopheles ───────────────────────────────────

#[test]
fn cr_121_2a_chains_replaces_each_extra_draw_once() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::chains_of_mephistopheles());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    let hand = g.players[0].hand.len();
    let mut events = Vec::new();
    // CR 614.5 — the draw Chains hands back isn't replaced again, so each
    // draw costs exactly one card out of hand and puts one back.
    for _ in 0..2 {
        g.draw_one(0, &mut events);
    }
    assert_eq!(g.players[0].hand.len(), hand);
    assert_eq!(g.players[0].graveyard.len(), 2);
}

// ── CR 509.2 — a banding blocker takes over damage assignment ──────────────

#[test]
fn cr_509_2_banding_gained_midcombat_still_routes_assignment() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::game::types::ResumeContext;
    let mut g = main_phase();
    g.players[0].wants_ui = true;
    let beater = CardDefinition {
        name: "Three Three",
        card_types: vec![CardType::Creature],
        power: 3,
        toughness: 3,
        ..Default::default()
    };
    let attacker = g.add_card_to_battlefield(1, beater);
    let caltrops = g.add_card_to_battlefield(0, catalog::wall_of_caltrops());
    let other = g.add_card_to_battlefield(0, catalog::wall_of_stone());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(caltrops, attacker), (other, attacker)]))
        .expect("block");
    drain_stack(&mut g);
    // Wall of Caltrops' block trigger granted banding, so the assignment goes
    // to the defending player rather than the attacker's controller.
    while g.pending_decision.is_none() {
        g.perform_action(GameAction::PassPriority).expect("pass");
        assert!(!g.is_game_over());
    }
    let pd = g.pending_decision.as_ref().expect("combat suspends on ordering");
    assert!(
        matches!(pd.resume, ResumeContext::CombatDamage { player: 0, .. }),
        "banding granted this combat still routes assignment to the defender, got {:?}",
        pd.resume
    );
}

// ── CR 808 — Team vs. Team keeps each seat's resources separate ────────────

#[test]
fn cr_808_5_teammates_do_not_share_resources() {
    let mut g = crabomination::game::multi_player_game(4);
    g.assign_teams(vec![vec![0, 2], vec![1, 3]]).expect("teams");
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].life = 12;
    assert!(g.players[2].hand.is_empty(), "hands aren't shared");
    assert_eq!(g.players[2].mana_pool.total(), 0, "mana isn't shared");
    assert_eq!(g.players[2].life, 20, "life totals aren't shared (contrast CR 810 2HG)");
    assert!(g.same_team(0, 2) && !g.same_team(0, 1));
}

#[test]
fn cr_808_3a_teammates_are_not_legal_attack_targets() {
    let mut g = crabomination::game::multi_player_game(4);
    g.assign_teams(vec![vec![0, 2], vec![1, 3]]).expect("teams");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(2) }])
            .is_err()
    );
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
            .is_ok(),
        "CR 802 — any opposing player is a legal defender"
    );
}

// ── CR 508.1 — the view's attackable-players list honours prohibitions ─────

#[test]
fn cr_508_1_attackable_players_drops_a_locked_out_defender() {
    let mut g = main_phase();
    assert_eq!(g.attackable_players_for(0), vec![1]);
    g.add_card_to_battlefield(0, catalog::arboria());
    assert!(
        g.attackable_players_for(0).is_empty(),
        "seat 1 did nothing on their last turn, so no attacker can be declared at them"
    );
}
