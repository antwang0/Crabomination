//! Functionality tests for the keyword-trigger shortcuts
//! (`effect::shortcut::{frenzy, afflict, afterlife}`) — CR 702.68 /
//! 702.131 / 702.135. Each test builds a synthetic creature carrying the
//! keyword trigger and drives combat (or a death) to observe the rider.
//! (Bushido / Flanking / Rampage already ship as `Keyword::*` combat
//! rules wired in `combat.rs`.)

use crate::card::{CardDefinition, CardType, Subtypes, TriggeredAbility};
use crate::catalog;
use crate::effect::shortcut;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::game::types::TurnStep;

/// A bare N/M creature carrying the given triggered abilities.
fn body(name: &'static str, p: i32, t: i32, trig: Vec<TriggeredAbility>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: p,
        toughness: t,
        triggered_abilities: trig,
        ..Default::default()
    }
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

// ── CR 702.68 Frenzy ─────────────────────────────────────────────────────────

#[test]
fn cr_702_68_frenzy_pumps_only_when_unblocked() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Berserker", 2, 2, vec![shortcut::frenzy(3)]));
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // No blockers declared → frenzy fires for +3/+0.
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no block");
    drain_stack(&mut g);
    let s = g.battlefield_find(atk).unwrap();
    assert_eq!((s.power(), s.toughness()), (5, 2), "frenzy 3 pumps an unblocked attacker");
}

#[test]
fn cr_702_68_frenzy_silent_when_blocked() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Berserker", 2, 2, vec![shortcut::frenzy(3)]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    let s = g.battlefield_find(atk).unwrap();
    assert_eq!((s.power(), s.toughness()), (2, 2), "frenzy does NOT fire when blocked");
}

// ── CR 702.158 Connive ─────────────────────────────────────────────────────

#[test]
fn cr_702_158_connive_draws_discards_and_counters_per_nonland() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::quandrix_cryptomancer()); // 2/2
    g.clear_sickness(atk);
    // Hand + library both nonland so the discard is guaranteed nonland → +1.
    g.add_card_to_hand(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::counterspell());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let s = g.battlefield_find(atk).unwrap();
    assert_eq!((s.power(), s.toughness()), (3, 3),
        "connive drew + pitched a nonland → one +1/+1 counter");
    assert_eq!(g.players[0].hand.len(), 1, "drew one, discarded one");
}

// ── CR 702.149 Training ────────────────────────────────────────────────────

#[test]
fn cr_702_149_training_counters_when_attacking_with_bigger_creature() {
    let mut g = two_player_game();
    let trainee = g.add_card_to_battlefield(0, body("Trainee", 2, 2, vec![shortcut::training()]));
    let mentor = g.add_card_to_battlefield(0, body("Veteran", 3, 3, vec![]));
    g.clear_sickness(trainee);
    g.clear_sickness(mentor);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: trainee, target: AttackTarget::Player(1) },
        Attack { attacker: mentor, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    let s = g.battlefield_find(trainee).unwrap();
    assert_eq!((s.power(), s.toughness()), (3, 3),
        "trains off the bigger co-attacker → one +1/+1 counter");
}

#[test]
fn cr_702_149_training_silent_without_bigger_co_attacker() {
    let mut g = two_player_game();
    let trainee = g.add_card_to_battlefield(0, body("Trainee", 2, 2, vec![shortcut::training()]));
    let small = g.add_card_to_battlefield(0, body("Runt", 1, 1, vec![]));
    g.clear_sickness(trainee);
    g.clear_sickness(small);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: trainee, target: AttackTarget::Player(1) },
        Attack { attacker: small, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    let s = g.battlefield_find(trainee).unwrap();
    assert_eq!((s.power(), s.toughness()), (2, 2), "no bigger co-attacker → no counter");
}

// ── CR 702.131 Afflict ───────────────────────────────────────────────────────

#[test]
fn cr_702_131_afflict_drains_defender_on_becoming_blocked() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Raptor", 3, 3, vec![shortcut::afflict(2)]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    let life_before = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 2, "afflict 2 drains the defending player");
}

// ── Combat math preview (ClientView.combat_preview) ──────────────────────────

#[test]
fn combat_preview_reports_unblocked_damage() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body("Brute", 3, 3, vec![]));
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let view = crate::server::view::project(&g, 0);
    let prev = view.combat_preview.expect("preview during combat");
    assert_eq!(prev.damage_to_players, vec![(1, 3)], "unblocked 3-power swing → 3 to P1");
    assert!(prev.dying_creatures.is_empty(), "no blocks, nothing dies");
}

#[test]
fn combat_preview_flags_a_losing_trade() {
    let mut g = two_player_game();
    // 3/3 attacker into a 2/2 blocker: blocker dies, attacker lives.
    let atk = g.add_card_to_battlefield(0, body("Brute", 3, 3, vec![]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    let prev = crate::server::view::project(&g, 0).combat_preview.expect("preview");
    assert_eq!(prev.dying_creatures, vec![blk], "the 2/2 blocker is projected to die");
    assert!(prev.damage_to_players.is_empty(), "no trample, no player damage");
}

// ── CR 702.135 Afterlife ─────────────────────────────────────────────────────

#[test]
fn cr_702_135_afterlife_mints_spirits_on_death() {
    use crate::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, body("Cleric", 1, 1, vec![shortcut::afterlife(2)]));
    // Kill it: drop its toughness below 1 so SBA destroys it.
    g.battlefield_find_mut(c).unwrap().toughness_bonus -= 1;
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|p| p.controller == 0
            && p.is_token
            && p.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 2, "afterlife 2 mints two Spirit tokens");
    assert!(spirits.iter().all(|s| s.has_keyword(&Keyword::Flying)), "Spirits have flying");
}

// ── CR 702.19 Trample (with Deathtouch) ──────────────────────────────────────

#[test]
fn cr_702_19_deathtouch_trample_assigns_one_then_tramples_rest() {
    // CR 702.19e + 702.2c: a deathtouch+trample attacker need only assign 1
    // (lethal) to each blocker, tramping the remainder to the player.
    use crate::card::{CardType, Keyword, Subtypes};
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, CardDefinition {
        name: "Wurm", card_types: vec![CardType::Creature], subtypes: Subtypes::default(),
        power: 5, toughness: 5,
        keywords: vec![Keyword::Trample, Keyword::Deathtouch],
        ..Default::default()
    });
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(atk);
    let life = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, life - 4,
        "1 lethal to the 2/2 (deathtouch), 4 trample to the player");
    assert!(g.battlefield_find(blk).is_none(), "the blocker died to deathtouch");
}
