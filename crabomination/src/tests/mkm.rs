//! Tests for the Murders at Karlov Manor (MKM) and Lost Caverns of Ixalan
//! (LCI) keyword actions: Suspect (701.60), Collect Evidence (701.59), and
//! Discover (701.57).

use crate::catalog;
use crate::card::{CounterType, Keyword};
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::game::types::Target;
use crate::mana::Color;
use crate::TurnStep;

// ── Suspect (CR 701.60) ──────────────────────────────────────────────────────

/// A suspected creature gains menace and can't block (computed keywords).
#[test]
fn barbed_servitor_etb_suspects_itself() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::barbed_servitor());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Barbed Servitor");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().suspected, "ETB suspected itself");
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.keywords.contains(&Keyword::Menace), "suspected → menace");
    assert!(cp.keywords.contains(&Keyword::CantBlock), "suspected → can't block");
}

/// Repeat Offender suspects itself, then on a second activation (while
/// suspected) grows with a +1/+1 counter instead.
#[test]
fn repeat_offender_toggles_suspect_then_grows() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::repeat_offender());
    drain_stack(&mut g);
    // First activation: not suspected → suspect it.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().suspected, "now suspected");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    // Second activation: suspected → +1/+1 counter.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("activate again");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "suspected → grows instead",
    );
}

/// Reasonable Doubt suspects a creature alongside the counter clause.
#[test]
fn reasonable_doubt_suspects_target_creature() {
    let mut g = two_player_game();
    // The creature to suspect, on the battlefield.
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // P1 casts a spell for the counter clause to target (P1 can't pay {2}).
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a spell");
    let id = g.add_card_to_hand(0, catalog::reasonable_doubt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(spell)),
        additional_targets: vec![Target::Permanent(victim)], mode: None, x_value: None,
    }).expect("cast Reasonable Doubt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().suspected, "creature suspected");
}

// ── Collect Evidence (CR 701.59) ─────────────────────────────────────────────

/// Sample Collector collects evidence on attack and grows a creature.
#[test]
fn sample_collector_collects_evidence_and_grows() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sample_collector());
    g.clear_sickness(id);
    // Graveyard fodder totaling MV ≥ 3 (two MV-2 bears = 4).
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // Yes to "collect evidence 3"; the +1/+1 counter auto-targets the attacker.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.len() < 2, "evidence exiled from graveyard");
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "collected evidence → +1/+1 counter",
    );
}

/// Without enough evidence in the graveyard, the payoff does not fire.
#[test]
fn sample_collector_without_evidence_does_nothing() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sample_collector());
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
        "no evidence → no counter",
    );
}

// ── Discover (CR 701.57) ─────────────────────────────────────────────────────

/// Geological Appraiser discovers 3 and the controller casts the hit for free.
#[test]
fn geological_appraiser_discovers_and_casts() {
    let mut g = two_player_game();
    // Top of library: a cheap creature the discover will hit.
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let id = g.add_card_to_hand(0, catalog::geological_appraiser());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Geological Appraiser");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some(), "discovered creature cast for free");
}

/// Declining the free cast puts the discovered card into hand instead.
#[test]
fn discover_decline_puts_card_in_hand() {
    let mut g = two_player_game();
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    let id = g.add_card_to_hand(0, catalog::geological_appraiser());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Geological Appraiser");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears), "declined → went to hand");
}

// ── Investigate / Map tokens ─────────────────────────────────────────────────

/// Deduce draws a card and investigates (mints a Clue token).
#[test]
fn deduce_draws_and_investigates() {
    let mut g = two_player_game();
    let drawn = g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::deduce());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Deduce");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == drawn), "drew the top card");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Clue"),
        "investigated → Clue token",
    );
}

/// Novice Inspector investigates on enter.
#[test]
fn novice_inspector_investigates_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::novice_inspector());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Novice Inspector");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"), "ETB Clue token");
}

/// Spyglass Siren makes a Map token on enter.
#[test]
fn spyglass_siren_makes_map_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spyglass_siren());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Spyglass Siren");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Map"), "ETB Map token");
}

/// Izoni collects evidence on enter and makes two Spider tokens.
#[test]
fn izoni_collects_evidence_for_spiders() {
    let mut g = two_player_game();
    // Graveyard fodder MV ≥ 4 (two MV-2 bears + a Bolt → ≥ 4).
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let id = g.add_card_to_hand(0, catalog::izoni_center_of_the_web());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Izoni");
    drain_stack(&mut g);
    let spiders = g.battlefield.iter().filter(|c| c.definition.name == "Spider").count();
    assert_eq!(spiders, 2, "collected evidence → two Spider tokens");
}

/// Trumpeting Carnosaur is a 7/6 trampler that discovers 5 on enter.
#[test]
fn trumpeting_carnosaur_discovers_five() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // MV 1 ≤ 5
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    let id = g.add_card_to_hand(0, catalog::trumpeting_carnosaur());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Trumpeting Carnosaur");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().definition.keywords.contains(&Keyword::Trample));
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "discover 5 → declined to hand");
}

// ── More MKM cards ───────────────────────────────────────────────────────────

/// Cold Case Cracker investigates when it dies.
#[test]
fn cold_case_cracker_investigates_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::cold_case_cracker());
    drain_stack(&mut g);
    // Kill it with a Bolt.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the Cracker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "Cracker died");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"), "death → Clue token");
}

/// Not on My Watch exiles an attacking creature.
#[test]
fn not_on_my_watch_exiles_attacker() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    // P0 responds with the instant.
    let id = g.add_card_to_hand(0, catalog::not_on_my_watch());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(attacker)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Not on My Watch");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "attacker exiled");
    assert!(g.exile.iter().any(|c| c.id == attacker), "attacker is in exile");
}

/// Person of Interest suspects itself and makes a Detective token.
#[test]
fn person_of_interest_suspects_self_and_makes_detective() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::person_of_interest());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Person of Interest");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().suspected, "suspected itself");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Detective"), "made a Detective");
}

/// Get a Leg Up pumps +1/+1 per creature you control and grants reach.
#[test]
fn get_a_leg_up_pumps_per_creature_and_grants_reach() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2 creatures controlled
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::get_a_leg_up());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(a)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Get a Leg Up");
    drain_stack(&mut g);
    let cp = g.computed_permanent(a).unwrap();
    assert_eq!(cp.power, 4, "2/2 base + (2 creatures) = 4 power");
    assert!(cp.keywords.contains(&Keyword::Reach), "gained reach");
}

/// Inside Source makes a Detective token on enter.
#[test]
fn inside_source_makes_detective() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inside_source());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Inside Source");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Detective"), "ETB Detective token");
}

/// Defossilize reanimates a creature and explores it twice.
#[test]
fn defossilize_reanimates_and_explores_twice() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    // Two nonland cards on top so both explores land +1/+1 counters.
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::defossilize());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(dead)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Defossilize");
    drain_stack(&mut g);
    let c = g.battlefield_find(dead).expect("reanimated");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2, "explored twice → two counters");
}

/// Goldvein Hydra enters with X +1/+1 counters and keeps its keywords.
#[test]
fn goldvein_hydra_enters_with_x_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::goldvein_hydra());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3); // X = 3
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast Goldvein Hydra for X=3");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!(cp.power, 3, "0/0 + 3 counters = 3 power");
    assert!(cp.keywords.contains(&Keyword::Trample) && cp.keywords.contains(&Keyword::Haste));
}

/// Slimy Dualleech buffs a small creature at the start of combat.
#[test]
fn slimy_dualleech_buffs_small_creature_at_combat() {
    let mut g = two_player_game();
    let slimy = g.add_card_to_battlefield(0, catalog::slimy_dualleech());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2, power 2
    drain_stack(&mut g);
    // Advance into combat so the begin-combat trigger fires.
    while g.step != TurnStep::BeginCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    // The trigger buffs one of the two eligible (power ≤ 2) creatures.
    let buffed = [slimy, bears].into_iter().any(|id| {
        let cp = g.computed_permanent(id).unwrap();
        cp.keywords.contains(&Keyword::Deathtouch)
    });
    assert!(buffed, "a small creature gained deathtouch from Slimy Dualleech");
}
