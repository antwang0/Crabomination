//! Functionality tests for `catalog::sets::decks::recent169` — DFT gap cards on
//! existing primitives (Vehicles, Speed payoffs, bite, cost reductions, exile).

use crabomination::card::{ArtifactSubtype, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Skybox Ferry is a 4/4 flying Vehicle with Crew 2 and Cycling {2}.
#[test]
fn skybox_ferry_keywords() {
    let d = catalog::skybox_ferry();
    assert!(d.keywords.contains(&Keyword::Flying));
    assert!(d.keywords.contains(&Keyword::Crew(2)));
    assert!(d.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Vehicle));
    assert!(d.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
}

/// Ripclaw Wrangler's ETB makes each opponent discard.
#[test]
fn ripclaw_wrangler_etb_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::forest());
    let before = g.players[1].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::ripclaw_wrangler());
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), before - 1, "opponent discarded one");
}

/// Pothole Mole mills three and returns a land from the graveyard to hand.
#[test]
fn pothole_mole_mills_and_returns_land() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::pothole_mole());
    drain_stack(&mut g);
    // A land was milled and returned → +1 hand; graveyard holds the milled
    // non-land plus whatever wasn't taken.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "returned a land to hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the milled creature stays in the graveyard");
}

/// Roadside Blowout costs {2} less when it targets a mana-value-1 permanent.
#[test]
fn roadside_blowout_cost_reduction_and_bounce() {
    let mut g = two_player_game();
    // MV-1 target: {U} alone (2 less than {2}{U}) pays it.
    let target = g.add_card_to_battlefield(1, catalog::savannah_lions()); // {W} 2/1, MV 1
    let spell = g.add_card_to_hand(0, catalog::roadside_blowout());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(spell, Some(Target::Permanent(target)), vec![], None, None)
        .expect("{U} pays the MV-1-reduced cost");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Savannah Lions"),
        "returned to owner's hand");
}

/// Run Over is a one-sided bite: your creature deals its power to an opponent's.
#[test]
fn run_over_one_sided_bite() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::run_over());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.cast_spell(spell, Some(Target::Permanent(mine)), vec![Target::Permanent(theirs)], None, None)
        .expect("cast Run Over");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "their 2/2 took 2 and died");
    assert!(g.battlefield_find(mine).is_some(), "one-sided — my creature is untouched");
}

/// Pride of the Road grants double strike at the start of combat when at max
/// speed.
#[test]
fn pride_of_the_road_max_speed_double_strike() {
    let mut g = two_player_game();
    let pride = g.add_card_to_battlefield(0, catalog::pride_of_the_road());
    g.clear_sickness(pride);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].speed = 4;
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(pride).unwrap().keywords.contains(&Keyword::DoubleStrike),
        "max-speed begin-combat granted double strike"
    );
}

/// Rangers' Refueler draws when you activate its exhaust ability, which also
/// animates it with a +1/+1 counter.
#[test]
fn rangers_refueler_exhaust_animate_and_draw() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::rangers_refueler());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: veh, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("exhaust animate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew off the exhaust trigger");
    let cp = g.computed_permanent(veh).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature), "animated");
    assert_eq!(g.battlefield_find(veh).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Rocketeer Boostbuggy makes a Treasure whenever it attacks.
#[test]
fn rocketeer_boostbuggy_attack_treasure() {
    let mut g = two_player_game();
    let veh = g.add_card_to_battlefield(0, catalog::rocketeer_boostbuggy());
    g.clear_sickness(veh);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Animate it via its exhaust ability so it can attack.
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: veh, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("exhaust animate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(veh).unwrap().card_types.contains(&CardType::Creature));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: veh, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "made a Treasure on attack"
    );
}

/// Point the Way searches basics equal to your speed onto the battlefield.
#[test]
fn point_the_way_searches_basics_equal_to_speed() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(0, catalog::point_the_way());
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f1)),
        DecisionAnswer::Search(Some(f2)),
    ]));
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].speed = 2;
    let lands_before = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ench, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("sac: search basics = speed");
    drain_stack(&mut g);
    let lands_after = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    assert_eq!(lands_after - lands_before, 2, "fetched 2 basics (speed 2)");
}

/// Perilous Snare exiles an opponent's permanent until it leaves the field.
#[test]
fn perilous_snare_exiles_until_leaves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let snare = g.move_card_to_battlefield_for_test(0, catalog::perilous_snare());
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim exiled");
    // Snare leaves → victim returns.
    g.remove_from_battlefield_to_graveyard_raw(snare);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 1),
        "victim returned when the snare left"
    );
}
