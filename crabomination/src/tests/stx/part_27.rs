//! Functionality tests for the extras_16 STX batch — the remaining
//! Lessons, X-spells, the spell-copy/counter package, and payoff creatures.
//! Exercises `ActivatedAbility.return_self_cost`, `Value::LifeGainedThisTurn`,
//! and `Value::DistinctPowerYouControl`.

use crate::card::{CreatureType, Keyword};
use crate::catalog;
use crate::game::types::Target;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;

// ── Lessons ──────────────────────────────────────────────────────────────────

#[test]
fn basic_conjuration_takes_a_creature_and_gains_life() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    // Top of library: a creature among some noncreature filler.
    g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::basic_conjuration());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "a creature card went to hand",
    );
    assert_eq!(g.players[0].life, life + 3, "gain 3 life");
}

#[test]
fn start_from_scratch_mode_one_destroys_an_artifact() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::start_from_scratch());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(rock)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("destroy-artifact mode castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "target artifact destroyed");
}

#[test]
fn teachings_of_the_archaics_draws_three_when_far_behind() {
    let mut g = two_player_game();
    g.players[0].hand.clear();
    g.players[1].hand.clear();
    for _ in 0..5 { g.add_card_to_hand(1, catalog::island()); } // opp +5 over you
    for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::teachings_of_the_archaics());
    let before = g.players[0].hand.len() - 1; // minus the spell about to be cast
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 3, "draw three when 4+ behind");
}

// ── X-spells ─────────────────────────────────────────────────────────────────

#[test]
fn blot_out_the_sky_makes_x_tapped_inklings() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blot_out_the_sky());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3); // X = 3
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("castable");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .collect();
    assert_eq!(inklings.len(), 3, "X=3 Inklings");
    assert!(inklings.iter().all(|c| c.tapped), "they enter tapped");
}

#[test]
fn blot_out_the_sky_wraths_noncreatures_at_x_six() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::blot_out_the_sky());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(6),
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "X≥6 destroys noncreature, nonland permanents");
}

#[test]
fn burn_down_the_house_mode0_sweeps_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::gnarled_professor()); // 5/4
    let id = g.add_card_to_hand(0, catalog::burn_down_the_house());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(),
        "5 damage to each creature kills both");
}

#[test]
fn burn_down_the_house_mode1_makes_three_hasty_devils() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::burn_down_the_house());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let devils: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Devil))
        .collect();
    assert_eq!(devils.len(), 3, "three Devil tokens");
    assert!(devils.iter().all(|c| c.has_keyword(&Keyword::Haste)), "devils have haste");
}

#[test]
fn geometric_nexus_charges_on_spell_cast_then_mints_a_scaled_fractal() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let nexus = g.add_card_to_battlefield(0, catalog::geometric_nexus());
    // Cast a 3-MV instant (Counterspell {U}{U}? use a known 3-MV IS). Use a
    // sorcery with cmc 3 — Golden Ratio ({1}{G}{U}) = 3.
    let spell = g.add_card_to_hand(0, catalog::golden_ratio());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a 3-MV sorcery");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(nexus).unwrap().counter_count(CounterType::Charge), 3,
        "charge counters equal the spell's mana value");
    // Activate: spend {6}, tap, remove all charge → 0/0 Fractal with 3 +1/+1.
    g.clear_sickness(nexus);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: nexus, ability_index: 0, target: None, x_value: None,
    }).expect("activate the Fractal-maker");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("a Fractal token");
    assert_eq!((fractal.power(), fractal.toughness()), (3, 3), "X = 3 charge counters removed");
    assert_eq!(g.battlefield_find(nexus).unwrap().counter_count(CounterType::Charge), 0,
        "all charge counters removed");
}

#[test]
fn culmination_of_studies_scales_treasure_draw_and_burn_by_exiled_types() {
    let mut g = two_player_game();
    g.players[0].hand.clear();
    // Top three of library: a land, a blue card, a red card.
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::frost_trickster()); // blue
    g.add_card_to_library(0, catalog::lightning_bolt());  // red
    // Filler below the top three so the blue-card draw has something to pull.
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::culmination_of_studies());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3); // X = 3
    let opp_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1,
        "one land → one Treasure");
    assert_eq!(g.players[0].hand.len(), 1, "one blue card → draw one");
    assert_eq!(g.players[1].life, opp_life - 1, "one red card → 1 damage to opponent");
}

#[test]
fn semesters_end_blinks_creature_and_returns_with_counter_at_end_step() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::semesters_end());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Exiled now; not on the battlefield.
    assert!(g.battlefield_find(bear).is_none(), "exiled by Semester's End");
    assert!(g.exile.iter().any(|c| c.id == bear), "in exile");
    // Advance to the end step so the delayed trigger returns it.
    while g.step != crate::game::types::TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    let back = g.battlefield_find(bear).expect("returned at end step");
    assert_eq!((back.power(), back.toughness()), (3, 3), "returns with a +1/+1 counter");
}

#[test]
fn claim_the_firstborn_steals_a_small_creature_with_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::claim_the_firstborn());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.controller, 0, "control stolen");
    assert!(!b.tapped, "untapped");
    assert!(b.has_keyword(&Keyword::Haste), "gains haste");
}

#[test]
fn guttural_response_counters_a_blue_instant_only() {
    let mut g = two_player_game();
    // Opponent casts a blue instant (Disperse, {1}{U}).
    let blue_instant = g.add_card_to_hand(1, catalog::disperse());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(1);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: blue_instant, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts blue instant");
    // P0 responds with Guttural Response.
    g.priority.player_with_priority = 0;
    let resp = g.add_card_to_hand(0, catalog::guttural_response());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: resp, target: Some(Target::Permanent(blue_instant)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("counter the blue instant");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == blue_instant), "blue instant countered");
}

#[test]
fn bond_of_flourishing_digs_for_a_permanent_and_gains_life() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.players[0].hand.clear();
    let perm = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt()); // nonpermanent filler
    let id = g.add_card_to_hand(0, catalog::bond_of_flourishing());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(perm))]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == perm), "took the permanent card");
    assert_eq!(g.players[0].life, life + 3, "gain 3 life");
}

#[test]
fn tamiyos_safekeeping_grants_hexproof_indestructible_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tamiyos_safekeeping());
    g.players[0].mana_pool.add(Color::Green, 1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let cb = computed.iter().find(|c| c.id == bear).unwrap();
    assert!(cb.keywords.contains(&Keyword::Hexproof) && cb.keywords.contains(&Keyword::Indestructible),
        "gains hexproof and indestructible");
    assert_eq!(g.players[0].life, life + 2, "gain 2 life");
}

#[test]
fn weather_the_storm_copies_for_prior_spells() {
    let mut g = two_player_game();
    // Pretend two spells were already cast this turn.
    g.spells_cast_this_turn = 2;
    let id = g.add_card_to_hand(0, catalog::weather_the_storm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Original + 2 Storm copies = 3 × gain 3 = 9 life.
    assert_eq!(g.players[0].life, life + 9, "Storm gains 3 life per copy + original");
}

#[test]
fn disperse_bounces_a_nonland_permanent_to_owners_hand() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::disperse());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(rock)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "bounced off the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == rock), "to owner's hand");
}

#[test]
fn make_your_move_hits_big_creature_and_artifact_but_not_small_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::gnarled_professor()); // 5/4
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let rock = g.add_card_to_battlefield(1, catalog::mind_stone()); // artifact
    // Small creature isn't a legal target.
    let id = g.add_card_to_hand(0, catalog::make_your_move());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(small)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "power-2 creature is not a legal target");
    // Big creature is.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("power-5 creature is legal");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "destroys the 5/4");
    assert!(g.battlefield_find(rock).is_some(), "artifact untouched this cast");
}

#[test]
fn sticky_fingers_grants_menace_and_mints_treasure_on_combat_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let aura = g.add_card_to_hand(0, catalog::sticky_fingers());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("aura castable");
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let cb = computed.iter().find(|c| c.id == bear).unwrap();
    assert!(cb.keywords.contains(&Keyword::Menace), "grants menace");
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![crate::game::Attack {
        attacker: bear, target: crate::game::AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != crate::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "combat damage to a player mints a Treasure");
}

#[test]
fn exponential_growth_doubles_power_x_times() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::exponential_growth());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(6); // {X}{X} with X=3 → 6 generic
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("castable");
    drain_stack(&mut g);
    // 2 power doubled 3 times = 2 * 2^3 = 16; toughness unchanged.
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (16, 2), "power doubled three times");
}

#[test]
fn serpentine_curve_scales_with_instants_and_sorceries_in_yards() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // 1 instant in gy
    g.add_card_to_exile(0, catalog::lightning_bolt()); // 1 instant in exile
    let id = g.add_card_to_hand(0, catalog::serpentine_curve());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // X = 1 + 1 (gy) + 1 (exile) = 3.
    let fractal = g.battlefield.iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("a Fractal token");
    assert_eq!(fractal.power(), 3, "0/0 + three +1/+1 counters");
}

// ── Spells ───────────────────────────────────────────────────────────────────

#[test]
fn flunk_shrinks_by_seven_minus_hand_size() {
    let mut g = two_player_game();
    g.players[1].hand.clear(); // empty hand → X = 7 → -7/-7
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::flunk());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "-7/-7 kills the 2/2");
}

#[test]
fn double_major_copies_a_creature_spell() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear on the stack");
    let dm = g.add_card_to_hand(0, catalog::double_major());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: dm, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Double Major targets the creature spell");
    drain_stack(&mut g);
    let bears = g.battlefield.iter()
        .filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "original + token copy resolve to battlefield");
}

#[test]
fn reject_counters_when_controller_cant_pay() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear on the stack");
    let rej = g.add_card_to_hand(0, catalog::reject());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: rej, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reject targets the creature spell");
    drain_stack(&mut g);
    // No floating mana and no lands → the controller can't pay {3}.
    assert!(g.battlefield_find(bear).is_none(), "spell countered, not on battlefield");
    // Reject exiles the countered spell instead of putting it in the graveyard.
    assert!(g.exile.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "countered spell exiled");
    assert!(!g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "not in graveyard");
}

#[test]
fn devouring_tendrils_deals_power_to_an_opponents_creature() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::gnarled_professor()); // 5/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::devouring_tendrils());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("castable");
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "5 power kills the 2/2");
    // The damaged permanent died this turn → you gain 2 life.
    assert_eq!(g.players[0].life, life_before + 2, "gain 2 when the permanent dies");
}

#[test]
fn study_break_taps_two_then_learns() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::study_break());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped,
        "both target creatures tapped");
}

#[test]
fn golden_ratio_draws_per_distinct_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2 (same)
    g.add_card_to_battlefield(0, catalog::gnarled_professor()); // power 5
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::golden_ratio());
    let before = g.players[0].hand.len() - 1;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 2, "two distinct powers (2 and 5) → draw 2");
}

#[test]
fn elemental_masterpiece_makes_two_elementals() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::elemental_masterpiece());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let elementals = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Elemental)).count();
    assert_eq!(elementals, 2, "two 4/4 Elementals");
}

#[test]
fn elemental_masterpiece_discard_from_hand_makes_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::elemental_masterpiece());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("{U/R}{U/R}, Discard this card: make a Treasure");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "discarded as cost");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "made a Treasure");
}

#[test]
fn harness_infinity_swaps_hand_and_graveyard() {
    let mut g = two_player_game();
    g.players[0].hand.clear();
    let h = g.add_card_to_hand(0, catalog::island());
    let gy1 = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let gy2 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::harness_infinity());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // The two graveyard cards are now in hand; the old hand card (island) is in gy.
    assert!(g.players[0].hand.iter().any(|c| c.id == gy1) && g.players[0].hand.iter().any(|c| c.id == gy2));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == h), "old hand card to graveyard");
    assert!(g.exile.iter().any(|c| c.id == id), "Harness Infinity exiles itself");
}

#[test]
fn dramatic_finale_anthems_tokens_and_mints_on_nontoken_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dramatic_finale());
    // Token anthem: a creature token you control gets +1/+1.
    let tok = g.add_token_to_battlefield(0, &catalog::sets::sos::inkling_token());
    let computed = g.compute_battlefield();
    let t = computed.iter().find(|c| c.id == tok).unwrap();
    assert_eq!((t.power, t.toughness), (2, 2), "1/1 Inkling token gets +1/+1");
    // Nontoken death mints a 2/1 Inkling.
    let before = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling)).count();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    let after = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling)).count();
    assert_eq!(after, before + 1, "a 2/1 Inkling minted on the nontoken death");
    // Triggers only once each turn: a second nontoken death this turn mints nothing.
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Permanent(bear2)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the second bear");
    drain_stack(&mut g);
    let after2 = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling)).count();
    assert_eq!(after2, after, "second death in the same turn mints no Inkling");
}

#[test]
fn deadly_brew_each_player_sacrifices() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::mind_stone()); // a permanent to return
    let id = g.add_card_to_hand(0, catalog::deadly_brew());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    // Accept the optional "return a permanent" rider.
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "you sacrifice a creature");
    assert!(g.battlefield_find(theirs).is_none(), "opponent sacrifices a creature");
    // You sacrificed, so the gated return fires and pulls the permanent back.
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Mind Stone"),
        "returned a permanent from graveyard");
}

#[test]
fn deadly_brew_no_return_if_you_didnt_sacrifice() {
    // You control no creature/PW to sacrifice → the "if you sacrificed" gate
    // closes and nothing is returned.
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::deadly_brew());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Mind Stone"),
        "no sacrifice → permanent stays in graveyard");
}

#[test]
fn kasmina_minus_x_makes_a_scaled_fractal() {
    let mut g = two_player_game();
    let k = g.add_card_to_battlefield(0, catalog::kasmina_enigma_sage());
    g.battlefield_find_mut(k).unwrap()
        .counters.insert(crate::card::CounterType::Loyalty, 5);
    // -X with X=3: pay 3 loyalty (5→2), Fractal enters as a 3/3.
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: k, ability_index: 1, target: None, x_value: Some(3),
    }).expect("-X activatable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("a Fractal token");
    assert_eq!(fractal.power(), 3, "0/0 + X=3 +1/+1 counters");
    assert_eq!(
        g.battlefield_find(k).unwrap().counter_count(crate::card::CounterType::Loyalty),
        2, "5 loyalty − X(3) = 2",
    );
}

#[test]
fn kasmina_minus_x_capped_at_current_loyalty() {
    let mut g = two_player_game();
    let k = g.add_card_to_battlefield(0, catalog::kasmina_enigma_sage());
    g.battlefield_find_mut(k).unwrap()
        .counters.insert(crate::card::CounterType::Loyalty, 2);
    // Requesting X=9 with only 2 loyalty clamps to X=2 (loyalty → 0).
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: k, ability_index: 1, target: None, x_value: Some(9),
    }).expect("-X clamps to available loyalty");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("a Fractal token");
    assert_eq!(fractal.power(), 2, "X clamped to 2");
    // Loyalty hit 0 → the planeswalker is put into the graveyard (CR 704.5i).
    assert!(g.battlefield_find(k).is_none(), "0-loyalty walker dies");
}

#[test]
fn biblioplex_taps_for_colorless_and_digs_for_spells() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::the_biblioplex());
    // Mana ability.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("{T}: Add {C}");
    assert_eq!(g.players[0].mana_pool.total(), 1, "added one colorless");
    // Dig: top card is an instant → goes to hand. Gate needs an empty hand.
    g.battlefield_find_mut(id).unwrap().tapped = false;
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None,
    }).expect("{2},{T}: dig");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "instant from top went to hand");
}

#[test]
fn biblioplex_dig_gated_to_empty_or_full_hand() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::the_biblioplex());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(2);
    // Three cards in hand → not 0 and not 7 → illegal.
    g.players[0].hand.clear();
    for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None,
    });
    assert!(err.is_err(), "dig is illegal with 3 cards in hand");
}

#[test]
fn detention_vortex_locks_activated_abilities() {
    // CR 602.5c — Detention Vortex shuts off the enchanted permanent's
    // activated abilities (and attack/block).
    let mut g = two_player_game();
    let prowler = g.add_card_to_battlefield(0, catalog::oriq_loremage()); // has a {T} ability
    g.clear_sickness(prowler);
    // Sanity: the ability works before the aura.
    g.perform_action(GameAction::ActivateAbility {
        card_id: prowler, ability_index: 0, target: None, x_value: None,
    }).expect("ability works pre-lock");
    drain_stack(&mut g);
    g.battlefield_find_mut(prowler).unwrap().tapped = false;
    // Now enchant it.
    let aura = g.add_card_to_hand(0, catalog::detention_vortex());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(prowler)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("aura castable");
    drain_stack(&mut g);
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: prowler, ability_index: 0, target: None, x_value: None,
    });
    assert!(err.is_err(), "activated abilities are locked while enchanted");
}

#[test]
fn detention_vortex_only_opponents_destroy_the_aura() {
    // The {3}: Destroy this Aura escape clause may only be activated by an
    // opponent of the Aura's controller, and only at sorcery speed.
    let mut g = two_player_game();
    let prowler = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::detention_vortex());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(prowler)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("aura castable");
    drain_stack(&mut g);
    // Controller (P0) can't activate the opponent-only escape.
    g.players[0].mana_pool.add_colorless(3);
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: None, x_value: None,
    });
    assert!(err.is_err(), "the Aura's controller can't activate the escape");
    // The opponent (P1), on their turn at sorcery speed, can.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: None, x_value: None,
    }).expect("an opponent may activate the escape");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "the Aura is destroyed");
}

// ── Creatures ────────────────────────────────────────────────────────────────

#[test]
fn gnarled_professor_learns_on_etb() {
    let mut g = two_player_game();
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::gnarled_professor());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().definition.keywords.contains(&Keyword::Trample));
}

#[test]
fn retriever_phoenix_returns_from_graveyard_instead_of_learning() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let phoenix = g.add_card_to_graveyard(0, catalog::retriever_phoenix());
    // Opt in to the replacement, then trigger a Learn (Gnarled Professor ETB).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.add_card_to_library(0, catalog::island());
    let prof = g.add_card_to_hand(0, catalog::gnarled_professor());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: prof, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(phoenix).is_some(), "Phoenix returned to battlefield");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == phoenix), "left the graveyard");
}

#[test]
fn retriever_phoenix_stays_in_graveyard_when_declined() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let phoenix = g.add_card_to_graveyard(0, catalog::retriever_phoenix());
    // Decline the return → normal learn (Draw 1 fallback) proceeds.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.add_card_to_library(0, catalog::island());
    let prof = g.add_card_to_hand(0, catalog::gnarled_professor());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: prof, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == phoenix), "Phoenix stays in graveyard");
}

#[test]
fn dream_strix_sacrifices_itself_when_targeted() {
    let mut g = two_player_game();
    let strix = g.add_card_to_battlefield(0, catalog::dream_strix());
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::island());
    // Aim a spell at it.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(strix)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(strix).is_none(), "sacrificed when targeted");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == strix), "Dream Strix died");
}

#[test]
fn accomplished_alchemist_taps_for_life_gained() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::accomplished_alchemist());
    g.clear_sickness(id);
    g.players[0].life_gained_this_turn = 4;
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, x_value: None,
    }).expect("life-gained mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), before + 4, "added X=4 mana of one color");
}

#[test]
fn oriq_loremage_mills_a_searched_card_to_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::oriq_loremage());
    g.clear_sickness(id);
    g.add_card_to_library(0, catalog::lightning_bolt());
    // The bot/scripted decider drives the search; AutoDecider declines, so
    // use a scripted search choosing the bolt.
    let target_card = g.players[0].library[0].id;
    g.decider = Box::new(crate::decision::ScriptedDecider::new(vec![
        crate::decision::DecisionAnswer::Search(Some(target_card)),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("{T}: search to graveyard");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == target_card),
        "searched card placed in graveyard");
    // The searched card was an instant → +1/+1 counter on Oriq Loremage.
    let oriq = g.battlefield_find(id).unwrap();
    assert_eq!((oriq.power(), oriq.toughness()), (4, 4), "instant search adds a +1/+1 counter");
}

#[test]
fn oriq_loremage_no_counter_when_searching_a_noninstant() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::oriq_loremage());
    g.clear_sickness(id);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let target_card = g.players[0].library[0].id;
    g.decider = Box::new(crate::decision::ScriptedDecider::new(vec![
        crate::decision::DecisionAnswer::Search(Some(target_card)),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("{T}: search to graveyard");
    drain_stack(&mut g);
    let oriq = g.battlefield_find(id).unwrap();
    assert_eq!((oriq.power(), oriq.toughness()), (3, 3), "creature search adds no counter");
}

#[test]
fn illustrious_historian_recurs_a_spirit_from_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::illustrious_historian());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("{5}, exile from gy: make a Spirit");
    drain_stack(&mut g);
    let spirit = g.battlefield.iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .expect("a Spirit token");
    assert_eq!((spirit.power(), spirit.toughness()), (3, 2), "3/2 Spirit");
    assert!(spirit.tapped, "enters tapped");
    assert!(g.exile.iter().any(|c| c.id == id), "Historian exiled as the cost");
}

#[test]
fn grinning_ignus_bounces_itself_for_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::grinning_ignus());
    g.players[0].mana_pool.add(Color::Red, 1); // pays the {R} cost
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("{R}, return self: add {C}{C}{R}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "Ignus returned to hand");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "net one red from the ability");
    assert!(g.players[0].mana_pool.total() >= 3, "added two colorless and one red");
}

#[test]
fn rootha_bounces_itself_to_copy_a_spell() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::rootha_mercurial_artist());
    g.clear_sickness(id);
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt on the stack");
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Permanent(bolt)), x_value: None,
    }).expect("{2}, return self: copy the bolt");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "Rootha returned to hand");
    assert_eq!(g.players[1].life, life - 6, "original + copy each deal 3");
}
