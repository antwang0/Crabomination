//! Functionality tests for the extras_17 STX batch — the remaining
//! single-faced printed cards (graveyard recursion, conditional keyword
//! share, reanimation, impulse draw).

use crate::card::Keyword;
use crate::catalog;
use crate::game::types::{Attack, AttackTarget};
use crate::game::{drain_stack, two_player_game};
use super::*;

// ── Efreet Flamepainter ──────────────────────────────────────────────────────

#[test]
fn efreet_flamepainter_recasts_instant_from_graveyard_on_damage() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let efreet = g.add_card_to_battlefield(0, catalog::efreet_flamepainter());
    g.clear_sickness(efreet);
    // A Lightning Bolt waiting in the graveyard to be re-cast for free.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    // Accept the optional free cast.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: efreet, target: AttackTarget::Player(1),
    }])).expect("attacks");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    // The recast spell was exiled (CR: "exile it instead") — proves it was cast.
    assert!(!g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "Bolt left the graveyard");
    assert!(g.exile.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "Bolt exiled after the free cast");
}

// ── Thunderous Orator ────────────────────────────────────────────────────────

#[test]
fn thunderous_orator_shares_flying_on_attack() {
    let mut g = two_player_game();
    let orator = g.add_card_to_battlefield(0, catalog::thunderous_orator());
    g.clear_sickness(orator);
    // A flier you control hands its flying to the Orator when it attacks.
    let _flier = g.add_card_to_battlefield(0, catalog::wind_drake());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: orator, target: AttackTarget::Player(1),
    }])).expect("attacks");
    drain_stack(&mut g);
    let o = g.battlefield_find(orator).expect("orator");
    assert!(o.has_keyword(&Keyword::Flying), "gained flying from the Wind Drake");
}

#[test]
fn thunderous_orator_no_share_without_keyworded_creature() {
    let mut g = two_player_game();
    let orator = g.add_card_to_battlefield(0, catalog::thunderous_orator());
    g.clear_sickness(orator);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: orator, target: AttackTarget::Player(1),
    }])).expect("attacks");
    drain_stack(&mut g);
    let o = g.battlefield_find(orator).expect("orator");
    assert!(!o.has_keyword(&Keyword::Flying), "no flier means no flying");
}

// ── Venerable Warsinger ──────────────────────────────────────────────────────

#[test]
fn venerable_warsinger_reanimates_on_combat_damage() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let ws = g.add_card_to_battlefield(0, catalog::venerable_warsinger());
    g.clear_sickness(ws);
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2, ≤ 3
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ws, target: AttackTarget::Player(1),
    }])).expect("attacks");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Grizzly Bears"),
        "Bears reanimated onto the battlefield",
    );
}

// ── Ardent Dustspeaker ───────────────────────────────────────────────────────

#[test]
fn ardent_dustspeaker_exiles_top_two_on_attack() {
    let mut g = two_player_game();
    let ds = g.add_card_to_battlefield(0, catalog::ardent_dustspeaker());
    g.clear_sickness(ds);
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let before = g.exile.iter().filter(|c| c.owner == 0).count();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ds, target: AttackTarget::Player(1),
    }])).expect("attacks");
    drain_stack(&mut g);
    let after = g.exile.iter().filter(|c| c.owner == 0).count();
    assert_eq!(after, before + 2, "two cards impulsed into exile");
}
