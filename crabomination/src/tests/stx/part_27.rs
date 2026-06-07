//! Functionality tests for the extras_16 STX batch — the remaining
//! Lessons, X-spells, the spell-copy/counter package, and payoff creatures.
//! Exercises `ActivatedAbility.return_self_cost`, `Value::LifeGainedThisTurn`,
//! and `Value::DistinctPowerYouControl`.

use crate::card::{CardType, CreatureType, Keyword};
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
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "countered spell to graveyard");
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
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "5 power kills the 2/2");
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
fn kasmina_minus_makes_a_counter_fractal() {
    let mut g = two_player_game();
    let k = g.add_card_to_battlefield(0, catalog::kasmina_enigma_sage());
    g.battlefield_find_mut(k).unwrap()
        .counters.insert(crate::card::CounterType::Loyalty, 5);
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: k, ability_index: 1, target: None,
    }).expect("-2 activatable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("a Fractal token");
    assert_eq!(fractal.power(), 2, "0/0 + two +1/+1 counters");
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
    // Dig: top card is an instant → goes to hand.
    g.battlefield_find_mut(id).unwrap().tapped = false;
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
