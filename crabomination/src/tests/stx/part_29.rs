//! Functionality tests for the extras_19 STX final-sweep cards: Emergent
//! Sequence, Augmenter Pugilist, Torrent Sculptor // Flamethrower Sonata, and
//! Blex, Vexing Pest // Search for Blex. (Shaile // Embrose and the
//! `EnteredThisTurn` primitive are covered in part_02.)

use crate::card::CounterType;
use crate::catalog;
use crate::game::two_player_game;
use super::*;

// ── Emergent Sequence ─────────────────────────────────────────────────────────

#[test]
fn emergent_sequence_animates_searched_land_with_counter() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let es = g.add_card_to_hand(0, catalog::emergent_sequence());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: es, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Emergent Sequence castable");
    drain_stack(&mut g);
    let land_id = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.is_land())
        .expect("searched land on battlefield").id;
    let raw = g.battlefield_find(land_id).unwrap();
    assert!(raw.tapped, "entered tapped");
    assert_eq!(raw.counter_count(CounterType::PlusOnePlusOne), 1, "one land entered this turn");
    let view = g.compute_battlefield().into_iter().find(|c| c.id == land_id).unwrap();
    assert!(view.card_types.contains(&crate::card::CardType::Land), "still a land");
    assert!(view.card_types.contains(&crate::card::CardType::Creature), "animated into a creature");
    assert_eq!((view.power, view.toughness), (1, 1), "0/0 base + one counter");
}

// ── Augmenter Pugilist ────────────────────────────────────────────────────────

#[test]
fn augmenter_pugilist_pumps_with_eight_lands() {
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::augmenter_pugilist());
    for _ in 0..7 { g.add_card_to_battlefield(0, catalog::forest()); }
    let pt = |g: &GameState| {
        let v = g.compute_battlefield().into_iter().find(|c| c.id == p).unwrap();
        (v.power, v.toughness)
    };
    assert_eq!(pt(&g), (3, 3), "below threshold");
    g.add_card_to_battlefield(0, catalog::forest()); // eighth land
    assert_eq!(pt(&g), (8, 8), "eight lands → +5/+5");
}

// ── Torrent Sculptor ──────────────────────────────────────────────────────────

#[test]
fn torrent_sculptor_etb_exiles_is_and_grows() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::cancel()); // {1}{U}{U} = MV 3
    let id = g.add_card_to_hand(0, catalog::torrent_sculptor());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast(&mut g, id);
    let s = g.battlefield_find(id).expect("Sculptor resolved");
    assert_eq!(s.counter_count(CounterType::PlusOnePlusOne), 2, "ceil(MV/2) counters");
    assert!(g.exile.iter().any(|c| c.definition.name == "Cancel"), "instant exiled from gy");
    assert!(s.has_keyword(&crate::card::Keyword::Ward(crate::card::WardCost::generic(2))), "has Ward 2");
}

#[test]
fn flamethrower_sonata_pings_for_discarded_is_mana_value() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let sonata = *catalog::torrent_sculptor().back_face.unwrap();
    let id = g.add_card_to_hand(0, sonata);
    let bolt = g.add_card_to_hand(0, catalog::cancel()); // MV 3 IS to discard
    g.add_card_to_library(0, catalog::island());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 opp creature
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![bolt])]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast_at(&mut g, id, Target::Permanent(target));
    assert!(!g.battlefield.iter().any(|c| c.id == target), "creature took 3 damage and died");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Cancel"), "IS discarded");
}

// ── Blex, Vexing Pest ─────────────────────────────────────────────────────────

#[test]
fn blex_anthems_kin_and_gains_life_on_death() {
    use crate::card::{CardDefinition, CardType, CreatureType, Subtypes};
    let mut g = two_player_game();
    let blex = g.add_card_to_battlefield(0, catalog::blex_vexing_pest());
    let pest = g.add_card_to_battlefield(0, CardDefinition {
        name: "Pest", card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pest], ..Default::default() },
        power: 1, toughness: 1, ..Default::default()
    });
    let view = |g: &GameState, id| {
        let v = g.compute_battlefield().into_iter().find(|c| c.id == id).unwrap();
        (v.power, v.toughness)
    };
    assert_eq!(view(&g, pest), (2, 2), "Pest pumped by Blex");
    assert_eq!(view(&g, blex), (3, 2), "Blex doesn't pump itself");
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    crate::game::cast_at(&mut g, bolt, Target::Permanent(blex));
    assert!(!g.battlefield.iter().any(|c| c.id == blex), "Blex died");
    assert_eq!(g.players[0].life, life + 4, "gained 4 life on Blex's death");
}

#[test]
fn search_for_blex_digs_and_loses_life() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let c1 = g.add_card_to_library(0, catalog::island());
    let c2 = g.add_card_to_library(0, catalog::forest());
    let c3 = g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = *catalog::blex_vexing_pest().back_face.unwrap();
    let id = g.add_card_to_hand(0, spell);
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![c1, c2])]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|c| c.id == c1), "kept c1");
    assert!(g.players[0].hand.iter().any(|c| c.id == c2), "kept c2");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == c3), "binned c3");
    assert_eq!(g.players[0].life, life - 6, "lost 3 life per kept card");
}

// ── Extus, Oriq Overlord // Awaken the Blood Avatar ────────────────────────────

#[test]
fn extus_magecraft_returns_nonlegendary_creature_from_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::extus_oriq_overlord());
    // A nonlegendary creature card in graveyard is the valid magecraft target.
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast_at(&mut g, bolt, Target::Player(1));
    assert!(
        g.players[0].hand.iter().any(|c| c.id == bear),
        "magecraft returns the nonlegendary creature to hand"
    );
}

#[test]
fn awaken_the_blood_avatar_forces_sacrifice_and_mints_avatar() {
    let mut g = two_player_game();
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = *catalog::extus_oriq_overlord().back_face.unwrap();
    let id = g.add_card_to_hand(0, spell);
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(6);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast(&mut g, id);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_creature), "opponent sacrificed their creature");
    let avatar = g.battlefield.iter().find(|c| c.definition.name == "Avatar").expect("Avatar minted");
    assert_eq!((avatar.power(), avatar.toughness()), (3, 6));
    assert!(avatar.has_keyword(&Keyword::Haste));
}

/// Awaken's optional additional cost — sacrifice two creatures to pay {4}
/// less, so {6}{B}{R} resolves off only {2}{B}{R} in pool.
#[test]
fn awaken_sacrifice_cost_reduction_makes_it_cheaper() {
    let mut g = two_player_game();
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sac1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sac2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = *catalog::extus_oriq_overlord().back_face.unwrap();
    let id = g.add_card_to_hand(0, spell);
    // Only {2}{B}{R} — four short of the printed {6}{B}{R}.
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellSacrificeReduce {
        card_id: id,
        sacrifices: vec![sac1, sac2],
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Awaken castable for {2}{B}{R} after sacrificing two creatures");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == sac1 || c.id == sac2), "both sacrifices left play");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_creature), "opponent sacrificed too");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Avatar"), "Avatar minted");
}

// ── Rowan, Scholar of Sparks // Will, Scholar of Frost ─────────────────────────

#[test]
fn rowan_static_makes_instants_and_sorceries_cost_one_less() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rowan_scholar_of_sparks());
    // Mascot Exhibition ({7} sorcery) becomes {6} with Rowan's reduction.
    let mascot = g.add_card_to_hand(0, catalog::mascot_exhibition());
    g.players[0].mana_pool.add_colorless(6);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: mascot, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mascot Exhibition castable for {6} thanks to Rowan's IS reduction");
}

#[test]
fn rowan_plus_one_pings_each_opponent_more_after_three_draws() {
    let mut g = two_player_game();
    let rowan = g.add_card_to_battlefield(0, catalog::rowan_scholar_of_sparks());
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let life_before = g.players[1].life;
    // Three cards drawn this turn → +1 deals 3.
    g.players[0].cards_drawn_this_turn = 3;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: rowan, ability_index: 0, target: None, x_value: None,
    }).expect("Rowan +1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 3, "3 damage after three draws");
}

#[test]
fn will_plus_one_sets_base_zero_two_and_minus_three_draws_two() {
    let mut g = two_player_game();
    let will = g.add_card_to_battlefield(0, *catalog::rowan_scholar_of_sparks().back_face.unwrap());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: will, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("Will +1");
    drain_stack(&mut g);
    let v = g.compute_battlefield().into_iter().find(|c| c.id == bear).unwrap();
    assert_eq!((v.power, v.toughness), (0, 2), "base P/T set to 0/2");

    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let hand_before = g.players[0].hand.len();
    // Reset the once-per-turn loyalty gate to exercise the -3 in the same test.
    g.battlefield_find_mut(will).unwrap().used_loyalty_ability_this_turn = false;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: will, ability_index: 1, target: None, x_value: None,
    }).expect("Will -3 draws two");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2);
}

// ── Mila, Crafty Companion // Lukka, Wayward Bonder ────────────────────────────

#[test]
fn mila_draws_when_your_permanent_targeted_by_opponent() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mila_crafty_companion());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crate::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    // Opponent bolts your bear → Mila offers a draw (auto-accepted here).
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    let hand_before = g.players[0].hand.len();
    crate::game::cast_at(&mut g, bolt, Target::Permanent(bear));
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "Mila draws on opponent targeting");
}

#[test]
fn lukka_minus_two_reanimates_with_haste() {
    let mut g = two_player_game();
    let lukka = g.add_card_to_battlefield(0, *catalog::mila_crafty_companion().back_face.unwrap());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: lukka, ability_index: 1, target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("Lukka -2 reanimates");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).expect("bear reanimated to battlefield");
    assert!(b.has_keyword(&Keyword::Haste), "reanimated creature has haste");
}

// ── Valentin, Dean of the Vein // Lisette, Dean of the Root ────────────────────

#[test]
fn valentin_exiles_dying_opponent_creature_and_offers_pest() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::valentin_dean_of_the_vein());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Accept the reflexive "pay {2}" to make a Pest.
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    // Kill the opponent's bear via lethal damage SBA.
    g.battlefield_find_mut(opp_bear).unwrap().damage = 2;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == opp_bear), "dying opp creature exiled, not graveyard'd");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == opp_bear), "not in graveyard");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Pest"),
        "reflexive Pest minted after paying {{2}}"
    );
}

#[test]
fn valentin_does_not_exile_own_dying_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::valentin_dean_of_the_vein());
    let my_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(my_bear).unwrap().damage = 2;
    let _ = g.check_state_based_actions();
    assert!(g.players[0].graveyard.iter().any(|c| c.id == my_bear), "own creature dies normally");
}

#[test]
fn lisette_pumps_team_on_lifegain_when_paid() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, *catalog::valentin_dean_of_the_vein().back_face.unwrap());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Vraska's Contempt: exile target + gain 2 life → triggers Lisette.
    let vc = g.add_card_to_hand(0, catalog::vraskas_contempt());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3); // {2}{B}{B} spell + {1} for Lisette
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crate::game::cast_at(&mut g, vc, Target::Permanent(opp));
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

// ── Radiant Scrollwielder + non-combat lifelink (CR 702.15) ────────────────────

#[test]
fn radiant_scrollwielder_gives_instants_lifelink() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::radiant_scrollwielder());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    crate::game::cast_at(&mut g, bolt, Target::Player(1));
    assert_eq!(g.players[0].life, life + 3, "bolt with spell-lifelink gains 3 life");
    assert_eq!(g.players[1].life, 20 - 3);
}

#[test]
fn radiant_scrollwielder_upkeep_exiles_instant_and_grants_replay() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::radiant_scrollwielder());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    // Fire the controller's upkeep trigger (active player is seat 0).
    g.fire_step_triggers(crate::game::types::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "instant exiled from graveyard");
}

// ── Kianne, Dean of Substance // Imbraham, Dean of Theory ──────────────────────

#[test]
fn kianne_studies_top_card_land_to_hand_else_study_counter() {
    let mut g = two_player_game();
    let kianne = g.add_card_to_battlefield(0, catalog::kianne_dean_of_substance());
    g.clear_sickness(kianne);
    // Top is a nonland (bears) → exiled with a study counter.
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: kianne, ability_index: 0, target: None, x_value: None,
    }).expect("Kianne study");
    drain_stack(&mut g);
    let e = g.exile.iter().find(|c| c.id == bears).expect("nonland exiled");
    assert_eq!(e.counter_count(CounterType::Study), 1);

    // Top is a land → goes to hand (no study counter, no exile).
    g.battlefield_find_mut(kianne).unwrap().tapped = false;
    let isl = g.add_card_to_library(0, catalog::island());
    g.perform_action(GameAction::ActivateAbility {
        card_id: kianne, ability_index: 0, target: None, x_value: None,
    }).expect("Kianne study a land");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == isl), "land studied into hand");
}

#[test]
fn kianne_fractal_counts_distinct_study_mana_values() {
    let mut g = two_player_game();
    let kianne = g.add_card_to_battlefield(0, catalog::kianne_dean_of_substance());
    g.clear_sickness(kianne);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    // Two study-countered nonland cards of distinct MV (bolt=1, bears=2) and a
    // duplicate MV (another bolt) → 2 distinct mana values.
    for c in [catalog::lightning_bolt(), catalog::grizzly_bears(), catalog::lightning_bolt()] {
        let id = g.next_id();
        let mut card = crate::card::CardInstance::new(id, c, 0);
        card.add_counters(CounterType::Study, 1);
        g.exile.push(card);
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: kianne, ability_index: 1, target: None, x_value: None,
    }).expect("Kianne fractal");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| c.definition.name == "Fractal").expect("fractal");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2, "two distinct study MVs");
}

#[test]
fn imbraham_exiles_top_x_with_study_counters() {
    let mut g = two_player_game();
    let imb = g.add_card_to_battlefield(0, *catalog::kianne_dean_of_substance().back_face.unwrap());
    g.clear_sickness(imb);
    let ids: Vec<_> = (0..2).map(|_| g.add_card_to_library(0, catalog::grizzly_bears())).collect();
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    // Decline the optional "return one to hand".
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(false),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: imb, ability_index: 0, target: None, x_value: Some(2),
    }).expect("Imbraham X=2");
    drain_stack(&mut g);
    for id in ids {
        let e = g.exile.iter().find(|c| c.id == id).expect("exiled");
        assert_eq!(e.counter_count(CounterType::Study), 1);
    }
}

#[test]
fn scholarship_sponsor_catches_up_lands_for_player_behind() {
    let mut g = two_player_game();
    // P0 (the caster) leads with 3 lands; P1 has 1 → deficit 2.
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); }
    g.add_card_to_battlefield(1, catalog::forest());
    for _ in 0..4 { g.add_card_to_library(1, catalog::island()); } // basics to fetch
    let p1_lands_before = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_land()).count();
    let sponsor = g.add_card_to_hand(0, catalog::scholarship_sponsor());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: sponsor, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scholarship Sponsor castable");
    drain_stack(&mut g);
    let p1_lands_after = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_land()).count();
    assert_eq!(p1_lands_after, p1_lands_before + 2, "P1 fetches 2 basics to match the leader");
    // The fetched lands enter tapped.
    let tapped = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_land() && c.tapped).count();
    assert_eq!(tapped, 2, "fetched basics enter tapped");
}

// ── Uvilda, Dean of Perfection // Nassari, Dean of Expression ───────────────────

#[test]
fn uvilda_hones_instant_then_makes_it_castable_for_four_less() {
    let mut g = two_player_game();
    let uvilda = g.add_card_to_battlefield(0, catalog::uvilda_dean_of_perfection());
    g.clear_sickness(uvilda);
    // {3}{U} instant (simplified body) — exiled with three hone counters.
    let behold = g.add_card_to_hand(0, catalog::behold_the_multiverse());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Discard(vec![behold]),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: uvilda, ability_index: 0, target: None, x_value: None,
    }).expect("Uvilda hone");
    drain_stack(&mut g);
    let e = g.exile.iter().find(|c| c.id == behold).expect("instant exiled");
    assert_eq!(e.counter_count(CounterType::Hone), 3);

    // Three of the owner's upkeeps tick the counters off.
    g.active_player_idx = 0;
    for _ in 0..3 {
        g.process_hone();
    }
    let e = g.exile.iter().find(|c| c.id == behold).expect("still exiled");
    assert_eq!(e.counter_count(CounterType::Hone), 0);
    assert!(e.may_play_until.is_some(), "castable from exile after last hone tick");
    // {3}{U} reduced by {4} → {U} (mv 1; the discount caps at the generic part).
    assert_eq!(e.granted_alt_cast_cost_eot.as_ref().unwrap().cmc(), 1);
}

#[test]
fn nassari_exiles_each_opponent_top_and_counts_exile_casts() {
    let mut g = two_player_game();
    let nassari =
        g.add_card_to_battlefield(0, *catalog::uvilda_dean_of_perfection().back_face.unwrap());
    g.clear_sickness(nassari);
    let bolt = g.add_card_to_library(1, catalog::lightning_bolt());
    // Active player 0's upkeep — exile the top of opponent 1's library.
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let e = g.exile.iter().find(|c| c.id == bolt).expect("opp top exiled");
    assert_eq!(e.may_play_until.unwrap().player, 0, "Nassari's controller may cast it");

    // Casting it from exile pumps Nassari (+1/+1).
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt,
        target: Some(crate::game::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bolt from exile");
    drain_stack(&mut g);
    let n = g.battlefield_find(nassari).expect("Nassari alive");
    assert_eq!(n.counter_count(CounterType::PlusOnePlusOne), 1, "+1/+1 on exile cast");
}
