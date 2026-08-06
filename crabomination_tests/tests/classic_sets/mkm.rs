//! Tests for the Murders at Karlov Manor (MKM) and Lost Caverns of Ixalan
//! (LCI) keyword actions: Suspect (701.60), Collect Evidence (701.59), and
//! Discover (701.57).

use crabomination::catalog;
use crabomination::card::{CounterType, Keyword};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::game::types::Target;
use crabomination::mana::Color;
use crabomination::TurnStep;

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
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().suspected, "now suspected");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    // Second activation: suspected → +1/+1 counter.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
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

/// "Suspect up to one target creature" — the suspect clause is optional, so
/// Reasonable Doubt still counters with no creature target supplied.
#[test]
fn reasonable_doubt_resolves_with_no_creature_target() {
    let mut g = two_player_game();
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
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Reasonable Doubt with no creature target");
    drain_stack(&mut g);
    // The countered spell never resolved → not on the battlefield.
    assert!(g.battlefield_find(spell).is_none(), "spell countered without a creature to suspect");
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

/// A `wants_ui` controller picks exactly which graveyard cards to exile for
/// evidence; the engine honors that choice over the auto cheapest-set.
#[test]
fn collect_evidence_ui_picker_honors_chosen_cards() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sample_collector());
    g.clear_sickness(id);
    g.players[0].wants_ui = true;
    // MV-2 bears ×2 plus an MV-1 Bolt. Auto would take Bolt+one bear (=3);
    // the human instead exiles both bears (=4), leaving the Bolt behind.
    let bear_a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bear_b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear_a, bear_b])]));
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "Bolt left in graveyard");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear_a || c.id == bear_b),
        "chosen bears exiled");
    assert_eq!(
        g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "evidence collected → +1/+1 counter",
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

/// "If you cast it" — Geological Appraiser put directly onto the battlefield
/// (not cast) does not discover (CR 603.x, SourceWasCast gate).
#[test]
fn geological_appraiser_no_discover_when_not_cast() {
    let mut g = two_player_game();
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::geological_appraiser());
    drain_stack(&mut g);
    assert!(g.players[0].library.iter().any(|c| c.id == bears), "no discover: library top untouched");
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

/// Goldvein Hydra's dies-trigger reads its *last-known* (counter-boosted)
/// power via CR 603.10 LKI, minting that many Treasures — not the 0 printed
/// power its graveyard copy would otherwise report.
#[test]
fn goldvein_hydra_dies_mints_power_many_treasures() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::goldvein_hydra());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4); // X = 4
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(4),
    }).expect("cast Goldvein Hydra for X=4");
    drain_stack(&mut g);
    // Lethal damage → dies via SBA, dies-trigger goes on the stack.
    g.battlefield_find_mut(id).unwrap().damage = 4;
    g.check_state_based_actions();
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 4, "4-power hydra dies → four Treasures");
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

// ── 2026-08 gap wave ────────────────────────────────────────────────────────

/// Delney doubles a small creature's trigger and leaves a big one alone.
#[test]
fn delney_doubles_small_creature_triggers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::delney_streetwise_lookout());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    drain_stack(&mut g);
    let before = g.players[0].hand.len();
    // Novice Inspector (1/1) investigates on entry — Delney fires it twice.
    g.move_card_to_battlefield_for_test(0, catalog::novice_inspector());
    drain_stack(&mut g);
    let clues = g.battlefield.iter().filter(|c| c.definition.name == "Clue").count();
    assert_eq!(clues, 2, "the power-1 body's ETB triggered an additional time");
    assert_eq!(g.players[0].hand.len(), before, "no draws, just Clues");
}

/// Delney's other half keeps big blockers off your small creatures.
#[test]
fn delney_walls_off_big_blockers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::delney_streetwise_lookout());
    let lions = g.add_card_to_battlefield(0, catalog::savannah_lions());
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(lions)
            .unwrap()
            .keywords
            .contains(&Keyword::CantBeBlockedByPowerAtLeast(3)),
        "power-2 creatures dodge power-3 blockers"
    );
}

/// Lost in the Maze freezes the creatures it taps and hides your tapped ones.
#[test]
fn lost_in_the_maze_stuns_and_hides() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let maze = g.add_card_to_hand(0, catalog::lost_in_the_maze());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: maze,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast Lost in the Maze");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).unwrap().tapped, "X=2 taps both creatures");
    assert!(g.battlefield_find(mine).unwrap().tapped);
    assert!(
        g.computed_permanent(mine).unwrap().keywords.contains(&Keyword::Hexproof),
        "your tapped creature has hexproof"
    );
}

/// Relive the Past brings an artifact back as a 5/5 Elemental.
#[test]
fn relive_the_past_reanimates_as_an_elemental() {
    let mut g = two_player_game();
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let spell = g.add_card_to_hand(0, catalog::relive_the_past());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(ring)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Relive the Past");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ring).expect("back on the battlefield");
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(
        cp.card_types.contains(&crabomination::card::CardType::Artifact),
        "still an artifact"
    );
}

/// Teysa turns a spent Clue into a Spirit, but only once a turn.
#[test]
fn teysa_mints_one_spirit_per_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::teysa_opulent_oligarch());
    drain_stack(&mut g);
    for _ in 0..2 {
        let mut evs = vec![];
        let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
        evs = g
            .resolve_effect(
                &crabomination::effect::Effect::CreateToken {
                    who: crabomination::effect::PlayerRef::You,
                    count: crabomination::card::Value::ONE,
                    definition: crabomination::game::effects::clue_token(),
                },
                &ctx,
            )
            .expect("mint a Clue");
        let clue = g
            .battlefield
            .iter()
            .find(|c| c.definition.name == "Clue")
            .map(|c| c.id)
            .expect("a Clue");
        let mut evs = vec![];
        g.sacrifice_one(clue, 0, &mut evs);
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    let spirits = g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits, 1, "the Clue trigger is once each turn");
}

/// The Pride of Hull Clade gets cheaper behind a wall of toughness.
#[test]
fn the_pride_of_hull_clade_discounts_by_toughness() {
    let mut g = two_player_game();
    let def = catalog::the_pride_of_hull_clade();
    assert_eq!(def.cost.cmc(), 11);
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // toughness 2
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::the_pride_of_hull_clade());
    let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
    assert_eq!(
        crabomination::game::actions::cost_reduction_for_spell(&g, 0, &card, None),
        2,
        "two toughness on board shaves the cost by 2"
    );
}
