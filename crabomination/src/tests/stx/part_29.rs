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
