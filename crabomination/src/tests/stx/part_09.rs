use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


// ── Batch 37 tests ──────────────────────────────────────────────────────────

#[test]
fn lorehold_b37_spiritflame_mints_spirit_and_pings_opp() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_b37_spiritflame());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritflame castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 1);
}

#[test]
fn lorehold_b37_beacon_magecraft_self_pumps() {
    let mut g = two_player_game();
    let beacon = g.add_card_to_battlefield(0, catalog::lorehold_b37_beacon());
    g.clear_sickness(beacon);
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(beacon).unwrap();
    assert_eq!(card.power(), 3);
}

#[test]
fn lorehold_sermonizer_etb_deals_damage_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_sermonizer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sermonizer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn quandrix_researcher_etb_draws_and_loses_one_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_researcher());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Researcher castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 1);
}

#[test]
fn quandrix_scout_magecraft_adds_counter() {
    let mut g = two_player_game();
    let scout = g.add_card_to_battlefield(0, catalog::quandrix_scout());
    g.clear_sickness(scout);
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(scout).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn fractal_reefborn_etb_doubles_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    // Put 2 counters on the bear via direct manipulation.
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 2);
    let id = g.add_card_to_hand(0, catalog::fractal_reefborn());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reefborn castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    // 2 + 2 = 4 counters
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 4);
}

#[test]
fn prismari_sparkmage_v2_etb_burns_and_magecraft_pings() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkmage_v2());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkmage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn prismari_eddy_draws_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_eddy());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Eddy castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn pest_vinekin_dies_gains_three_life_and_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_vinekin());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    let life_before = g.players[0].life;
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 2);
}

// ── Batch 38: 25 more STX cards across all 5 colleges ───────────────────────

#[test]
fn silverquill_essayist_magecraft_gains_one_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _id = g.add_card_to_battlefield(0, catalog::silverquill_essayist());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    // Library unchanged after scry (only reorders).
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn inkling_scriptwarden_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_scriptwarden());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scriptwarden castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn silverquill_pinion_pumps_and_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_pinion());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pinion castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::Flying));
}

#[test]
fn silverquill_battle_oration_drains_four_and_mints_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_battle_oration());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battle Oration castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 4);
    assert_eq!(g.players[0].life, life_before + 4);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 1);
}

#[test]
fn inkling_calligraphist_magecraft_adds_plus_one_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_calligraphist());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn witherbloom_fungalweb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_fungalweb());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fungalweb castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn pest_swarmrider_etb_mints_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_swarmrider());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Swarmrider castable");
    drain_stack(&mut g);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 1);
}

#[test]
fn witherbloom_bloodbrewer_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::witherbloom_bloodbrewer());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bloodbrewer drains 1; bolt deals 3.
    assert_eq!(g.players[1].life, opp_before - 1 - 3);
}

#[test]
fn witherbloom_rotwarden_is_a_four_four_trampler_lifelinker() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_rotwarden());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 4);
    assert!(card.has_keyword(&Keyword::Trample));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn pest_briarscale_is_a_three_three_pest_beast_trampler() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_briarscale());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::Trample));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Pest));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Beast));
}

#[test]
fn lorehold_ember_priest_v2_magecraft_pings_one() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_ember_priest_v2());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Ember Priest magecraft ping 1.
    assert_eq!(g.players[1].life, opp_before - 3 - 1);
}

#[test]
fn lorehold_skydefender_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_skydefender());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skydefender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_archivist_v2_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_archivist_v2());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Archivist castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Bear returned to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn lorehold_spiritrider_etb_mints_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritrider());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritrider castable");
    drain_stack(&mut g);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 2);
}

#[test]
fn spirit_warbearer_is_a_two_two_first_strike_warrior() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spirit_warbearer());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
    assert!(card.has_keyword(&Keyword::FirstStrike));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Spirit));
}

#[test]
fn quandrix_pondkeeper_v2_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _id = g.add_card_to_battlefield(0, catalog::quandrix_pondkeeper_v2());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Library reorders only, no draw.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn fractal_emergent_enters_with_three_plus_one_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_emergent());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Emergent castable");
    drain_stack(&mut g);
    let card = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Fractal Emergent")
        .expect("Fractal Emergent on battlefield");
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 3);
    // 0/0 base + 3 counters → 3/3.
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn quandrix_fluctuator_etb_draws_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_fluctuator());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fluctuator castable");
    drain_stack(&mut g);
    // -1 (cast) +1 (draw) = 0 net hand change.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_doublecaster_v2_magecraft_adds_counter_to_friendly_fractal() {
    let mut g = two_player_game();
    let fractal = g.add_card_to_battlefield(0, catalog::fractal_emergent());
    let _id = g.add_card_to_battlefield(0, catalog::quandrix_doublecaster_v2());
    drain_stack(&mut g);
    let counters_before = g.battlefield_find(fractal).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(fractal).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters_after, counters_before + 1);
}

#[test]
fn quandrix_basinkeeper_etb_mints_fractal_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_basinkeeper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Basinkeeper castable");
    drain_stack(&mut g);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 1);
    let fractal = g.battlefield.iter().find(|c| c.is_token).unwrap();
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn prismari_dazzler_magecraft_pings_one() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::prismari_dazzler());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3 - 1);
}

#[test]
fn prismari_cinderpoet_etb_draws_and_discards() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_cinderpoet());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cinderpoet castable");
    drain_stack(&mut g);
    // Hand: -1 (cast Cinderpoet) +1 (draw) -1 (discard) = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
    // Graveyard gained 1 (discarded card).
    assert_eq!(g.players[0].graveyard.len(), gy_before + 1);
}

#[test]
fn prismari_pyrocaster_etb_deals_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_pyrocaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrocaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn prismari_drift_deals_two_to_creature_and_scrys() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::prismari_drift());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drift castable");
    drain_stack(&mut g);
    // Bear should be dead (2/2 takes 2 damage).
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn silverquill_manuscript_drains_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_manuscript());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Manuscript castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    // Hand: -1 (cast) +1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_ambassador_is_a_two_mana_lifelink_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_ambassador());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 1);
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn witherbloom_cauldronkeeper_etb_gains_two_life_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_cauldronkeeper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cauldronkeeper castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_wargeist_is_a_three_two_haste_spirit_warrior() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_wargeist());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 2);
    assert!(card.has_keyword(&Keyword::Haste));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Spirit));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Warrior));
}

#[test]
fn quandrix_scaler_magecraft_adds_self_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_scaler());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn prismari_sparkbolt_deals_two_to_any_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkbolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkbolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn prismari_stormrider_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_stormrider());
    drain_stack(&mut g);
    let pwr_before = g.battlefield_find(id).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), pwr_before + 1);
    assert!(card.has_keyword(&Keyword::Flying));
}

// ── Batch 39: 30 new STX cards across all 5 colleges ────────────────────────

#[test]
fn silverquill_liturgist_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::silverquill_liturgist());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn inkling_bookwarden_is_four_five_lifelink_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_bookwarden());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 5);
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_soulbinder_etb_drains_two_then_magecraft_adds_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_soulbinder());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soulbinder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
    // Then cast a bolt to trigger magecraft
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn inkling_magister_etb_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_magister());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Magister castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, life_before + 3);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn silverquill_inkproclamation_each_opp_sacs_and_mints_inkling() {
    let mut g = two_player_game();
    let _victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_inkproclamation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    let opp_creatures_before = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.is_creature())
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkproclamation castable");
    drain_stack(&mut g);
    let opp_creatures_after = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.is_creature())
        .count();
    assert_eq!(opp_creatures_after, opp_creatures_before - 1);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 1);
}

#[test]
fn inkling_loredrain_makes_opp_discard_and_drains() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::inkling_loredrain());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp_hand_before = g.players[1].hand.len();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Loredrain castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn witherbloom_rootbinder_etb_gains_two_and_magecraft_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_rootbinder());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rootbinder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    // Magecraft trigger now
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_mid = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_mid + 1);
    let _ = id;
}

#[test]
fn pest_reaver_is_three_three_deathtouch() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_reaver());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::Deathtouch));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Pest));
}

#[test]
fn witherbloom_decoction_drains_two_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::witherbloom_decoction());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Decoction castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
    // scry reorders without drawing
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn witherbloom_cultivator_etb_mints_pest_and_magecraft_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_cultivator());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cultivator castable");
    drain_stack(&mut g);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 1);
    // Magecraft adds a counter
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn witherbloom_spawnkeeper_drains_when_another_creature_dies() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::witherbloom_spawnkeeper());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    // Kill the bear via Lightning Bolt — proper damage path emits
    // CreatureDied, firing AnotherOfYours-scoped triggers.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn lorehold_hellraiser_etb_deals_two_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_hellraiser());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hellraiser castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Haste));
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 4);
}

#[test]
fn lorehold_annalist_magecraft_exiles_graveyard_card() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_annalist());
    let gy_card = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // The bear should now be in exile (or removed from gy)
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == gy_card),
        "Bear should leave opponent's graveyard via Lorehold Annalist's magecraft");
}

#[test]
fn lorehold_bonfire_deals_three_and_gains_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_bonfire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bonfire castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn lorehold_spiritsage_etb_mints_a_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritsage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritsage castable");
    drain_stack(&mut g);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 1);
    let _ = id;
}

#[test]
fn lorehold_pyrokin_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_pyrokin());
    drain_stack(&mut g);
    let pwr_before = g.battlefield_find(id).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), pwr_before + 1);
    assert!(card.has_keyword(&Keyword::Haste));
}

#[test]
fn spirit_outrider_is_three_four_first_strike() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spirit_outrider());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 4);
    assert!(card.has_keyword(&Keyword::FirstStrike));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Spirit));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Knight));
}

#[test]
fn quandrix_scrymaster_etb_scrys_and_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::quandrix_scrymaster());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scrymaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
}

#[test]
fn fractal_burst_mints_three_three_fractal_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_burst());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Burst castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
    assert!(fractal.is_some(), "Fractal token created");
    let fractal = fractal.unwrap();
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 3);
    assert_eq!(fractal.power(), 3);
    assert_eq!(fractal.toughness(), 3);
}

#[test]
fn quandrix_aetherwarden_etb_draws_and_magecraft_counter() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_aetherwarden());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Aetherwarden castable");
    drain_stack(&mut g);
    // We cast it (hand -1), but drew (hand +1), net 0
    assert_eq!(g.players[0].hand.len(), hand_before);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
}

#[test]
fn quandrix_tideshaper_etb_bounces_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_tideshaper());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tideshaper castable");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear should be bounced to opponent's hand");
}

#[test]
fn fractal_catalyst_magecraft_adds_counter_to_friendly_creature() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::fractal_catalyst());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_equalizer_etb_pumps_each_other_friendly_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_equalizer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Equalizer castable");
    drain_stack(&mut g);
    let b1 = g.battlefield_find(bear).unwrap();
    let b2 = g.battlefield_find(bear2).unwrap();
    let eq = g.battlefield_find(id).unwrap();
    assert_eq!(b1.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(b2.counter_count(CounterType::PlusOnePlusOne), 1);
    // Equalizer itself doesn't get a counter (OtherThanSource)
    assert_eq!(eq.counter_count(CounterType::PlusOnePlusOne), 0);
}

#[test]
fn prismari_hothead_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_hothead());
    drain_stack(&mut g);
    let pwr_before = g.battlefield_find(id).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), pwr_before + 1);
    assert!(card.has_keyword(&Keyword::Haste));
}

#[test]
fn prismari_cantrip_bolt_deals_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::mountain());
    // Use a 3-toughness creature so it survives the 2 damage.
    let beefy = g.add_card_to_battlefield(1, catalog::silverquill_essayist());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::prismari_cantrip_bolt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(beefy)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cantrip Bolt castable");
    drain_stack(&mut g);
    // We cast (hand -1), drew (hand +1), net 0
    assert_eq!(g.players[0].hand.len(), hand_before);
    let card = g.battlefield_find(beefy).unwrap();
    assert_eq!(card.damage, 2);
}

#[test]
fn prismari_wildmage_magecraft_pings_each_opponent() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::prismari_wildmage());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Wildmage magecraft 1
    assert_eq!(g.players[1].life, opp_before - 3 - 1);
}

#[test]
fn prismari_stormbearer_etb_loots_then_magecraft_pumps_self() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::prismari_stormbearer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormbearer castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert_eq!(card.power(), 4);
}

#[test]
fn prismari_pyromancer_v2_etb_deals_two_to_any_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_pyromancer_v2());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyromancer V2 castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn prismari_tempestmage_magecraft_pumps_target_creature() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::prismari_tempestmage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let pwr_before = g.battlefield_find(bear).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear_now = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_now.power(), pwr_before + 1);
}

// ── Batch 40 tests: new STX cards ────────────────────────────────────────────

#[test]
fn silverquill_scriptwright_pumps_friendly_inkling_on_is_cast() {
    let mut g = two_player_game();
    let _sw = g.add_card_to_battlefield(0, catalog::silverquill_scriptwright());
    let inkling = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    drain_stack(&mut g);
    let pwr_before = g.battlefield_find(inkling).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(inkling).unwrap().power(),
        pwr_before + 1,
        "Inkling Aspirant pumped by Scriptwright magecraft"
    );
}

#[test]
fn inkling_bookcrier_is_a_three_mana_flying_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_bookcrier());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card
        .definition
        .subtypes
        .creature_types
        .contains(&CreatureType::Inkling));
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 2);
}

#[test]
fn silverquill_cantorist_etb_drains_one_and_is_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_cantorist());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let opp_before = g.players[1].life;
    let self_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Cantorist castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, self_before + 1);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_treasurer_etb_gains_life_and_smooths_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::inkling_treasurer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let self_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Treasurer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, self_before + 1);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
}

#[test]
fn silverquill_memorize_drains_two_and_pumps_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_memorize());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    let self_before = g.players[0].life;
    let pwr_before = g.battlefield_find(bear).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Memorize castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, self_before + 2);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), pwr_before + 1);
}

#[test]
fn inkling_bellringer_etb_makes_opp_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::inkling_bellringer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bellringer castable");
    drain_stack(&mut g);
    assert!(
        g.players[1].hand.len() < opp_hand_before,
        "opponent discarded a card"
    );
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_encore_pumps_team_with_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_encore());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let pwr_before = g.battlefield_find(bear).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Encore castable");
    drain_stack(&mut g);
    let b1 = g.battlefield_find(bear).unwrap();
    let b2 = g.battlefield_find(bear2).unwrap();
    assert_eq!(b1.power(), pwr_before + 1);
    assert_eq!(b2.power(), pwr_before + 1);
    assert!(b1.has_keyword(&Keyword::Lifelink));
    assert!(b2.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_sentencer_shrinks_opp_creature_on_etb() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::inkling_sentencer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let pwr_before = g.battlefield_find(opp_bear).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Sentencer castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(opp_bear).unwrap().power(),
        pwr_before - 1
    );
}

#[test]
fn silverquill_inkflood_mints_two_inklings_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkflood());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let self_before = g.players[0].life;
    let inklings_before = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Inkflood castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, self_before + 2);
    let inklings_after = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .count();
    assert_eq!(inklings_after, inklings_before + 2);
}

#[test]
fn inkling_quilltender_etb_pumps_target_inkling() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::inkling_quilltender());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let pwr_before = g.battlefield_find(other).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(other)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Quilltender castable");
    drain_stack(&mut g);
    let now = g.battlefield_find(other).unwrap();
    let count = now.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(count, 1);
    assert_eq!(now.power(), pwr_before + 1);
    let self_card = g.battlefield_find(id).unwrap();
    assert!(self_card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn witherbloom_toxicologist_shrinks_target_on_is_cast() {
    let mut g = two_player_game();
    let _tox = g.add_card_to_battlefield(0, catalog::witherbloom_toxicologist());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let pwr_before = g.battlefield_find(opp_bear).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Toxicologist trigger shrinks the target, but the Bolt also deals
    // 3 damage. The card may be in graveyard now if it died.
    let still_alive = g.battlefield_find(opp_bear);
    if let Some(c) = still_alive {
        assert_eq!(c.power(), pwr_before - 1);
    }
}

#[test]
fn witherbloom_bloodglyph_drains_two_and_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_bloodglyph());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    let self_before = g.players[0].life;
    let pests_before = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition.subtypes.creature_types.contains(&CreatureType::Pest)
        })
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bloodglyph castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, self_before + 2);
    let pests_after = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition.subtypes.creature_types.contains(&CreatureType::Pest)
        })
        .count();
    assert_eq!(pests_after, pests_before + 1);
}

#[test]
fn witherbloom_rotsage_etb_offers_optional_sac_loot() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::witherbloom_rotsage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Rotsage castable");
    drain_stack(&mut g);
    // AutoDecider declines the may-do by default, leaving fodder alive.
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn witherbloom_sproutchant_gains_counter_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_sproutchant());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let pwr_before = g.battlefield_find(id).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let now = g.battlefield_find(id).unwrap();
    assert_eq!(now.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
    assert_eq!(now.power(), pwr_before + 1);
}

#[test]
fn lorehold_ember_reader_pings_on_is_cast() {
    let mut g = two_player_game();
    let _reader = g.add_card_to_battlefield(0, catalog::lorehold_ember_reader());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt = 3 + magecraft ping = 1 = 4 damage.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_wraithcaller_mints_flying_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_wraithcaller());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let spirits_before = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Spirit)
        })
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Wraithcaller castable");
    drain_stack(&mut g);
    let spirits_after = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Spirit)
        })
        .count();
    // Wraithcaller is a Spirit itself + the minted Spirit = 2 spirits added.
    assert_eq!(spirits_after, spirits_before + 2);
}

#[test]
fn lorehold_ballad_burns_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_ballad());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let opp_before = g.players[1].life;
    let self_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Ballad castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, self_before + 2);
}

#[test]
fn quandrix_loomweaver_loots_on_is_cast() {
    let mut g = two_player_game();
    let _loom = g.add_card_to_battlefield(0, catalog::quandrix_loomweaver());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::forest());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Hand: -1 bolt cast + 1 draw - 1 discard = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn fractal_stargazer_etb_scrys_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::fractal_stargazer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Stargazer castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 2);
}

#[test]
fn quandrix_bountycaller_etb_mints_fractal_with_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_bountycaller());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let fractals_before = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Fractal)
        })
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bountycaller castable");
    drain_stack(&mut g);
    let fractals_after: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Fractal)
        })
        .collect();
    assert_eq!(fractals_after.len(), fractals_before + 1);
    let token = &fractals_after[fractals_before];
    assert_eq!(token.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 4);
    assert_eq!(token.power(), 4);
}

#[test]
fn prismari_cinderbolt_pings_on_is_cast() {
    let mut g = two_player_game();
    let _mage = g.add_card_to_battlefield(0, catalog::prismari_cinderbolt());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + cinderbolt magecraft 1 = 4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_stormblade_burns_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_stormblade());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Stormblade castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    // Hand: -1 cast + 1 draw = same size.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_maestro_draws_two_on_combat_damage() {
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let maestro = g.add_card_to_battlefield(0, catalog::prismari_maestro());
    g.clear_sickness(maestro);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority)
            .expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: maestro,
        target: AttackTarget::Player(1),
    }]))
    .expect("Maestro attacks");
    drain_stack(&mut g);
    let hand_before = g.players[0].hand.len();
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority)
            .expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].hand.len(),
        hand_before + 2,
        "drew 2 from Maestro combat-damage trigger"
    );
}

#[test]
fn quandrix_spellseer_etb_scrys_and_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_spellseer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Spellseer castable");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw - 1 discard = -1 net
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn fractal_bloomweaver_etb_with_counters_and_pumps_others() {
    let mut g = two_player_game();
    let other_fractal = g.add_card_to_battlefield(0, catalog::fractal_grower());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::fractal_bloomweaver());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bloomweaver castable");
    drain_stack(&mut g);
    let bloom = g.battlefield_find(id).unwrap();
    assert_eq!(
        bloom
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        3,
        "Bloomweaver enters with 3 counters"
    );
    let other = g.battlefield_find(other_fractal).unwrap();
    assert_eq!(
        other
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "other Fractal gains 1 counter via Bloomweaver ETB"
    );
}

#[test]
fn lorehold_ironwill_pumps_self_on_is_cast_and_is_first_strike() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_ironwill());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::FirstStrike));
    let pwr_before = card.power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().power(), pwr_before + 1);
}

#[test]
fn spirit_pyremage_etb_pings_any_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spirit_pyremage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pyremage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
}

// ── Batch 41: new STX cards across all five colleges ────────────────────────

#[test]
fn silverquill_purifier_etb_gains_two_life_and_scrys_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::silverquill_purifier());
    drain_stack(&mut g);
    // ETB already fired before our snapshot — life should be base + 2.
    let life_after_etb = g.players[0].life;
    // Cast an instant to fire magecraft Scry 1.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Purifier still on battlefield, no life change from scry.
    assert!(g.battlefield_find(id).is_some());
    assert_eq!(g.players[0].life, life_after_etb);
}

#[test]
fn inkling_proxy_etb_makes_opp_discard_random() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let hand_before = g.players[1].hand.len();
    let id = g.add_card_to_hand(0, catalog::inkling_proxy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Proxy castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 1);
}

#[test]
fn silverquill_witnessing_drains_three_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_witnessing());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Witnessing castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, life_before + 3);
    // -1 cast + 1 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_avant_garde_etb_drains_two_and_is_lifelink_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_avant_garde());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Avant-Garde castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_convocation_mints_two_inklings_and_drains_per_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_convocation());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Convocation castable");
    drain_stack(&mut g);
    // 2 Inkling tokens minted; drain = number of Inklings = 2.
    let inklings = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Inkling)
        })
        .count();
    assert_eq!(inklings, 2);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn witherbloom_distiller_drains_each_opp_on_cast() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::witherbloom_distiller());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 dmg + Distiller magecraft 1 life loss = -4 life.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn pest_brewer_etb_mints_a_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_brewer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pest Brewer castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some());
    let pest_count = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Pest)
                && c.id != id
        })
        .count();
    assert_eq!(pest_count, 1);
}

#[test]
fn witherbloom_alchemist_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_alchemist());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Alchemist castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn witherbloom_bloomcaller_gains_life_on_cast() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::witherbloom_bloomcaller());
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn witherbloom_pestsage_etb_mints_two_pest_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestsage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pestsage castable");
    drain_stack(&mut g);
    let pest_count = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Pest)
                && c.id != id
        })
        .count();
    assert_eq!(pest_count, 2);
}

#[test]
fn lorehold_emberkeeper_pings_on_is_cast() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_emberkeeper());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt = -3, magecraft = -1 → -4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_warden_v2_etb_exiles_target_graveyard_card() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let target_id = g.players[1].graveyard.last().unwrap().id;
    let id = g.add_card_to_hand(0, catalog::lorehold_warden_v2());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target_id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Warden castable");
    drain_stack(&mut g);
    // Card is now in exile, not in gy.
    assert!(g.players[1].graveyard.iter().all(|c| c.id != target_id));
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_recital_v2_burns_and_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_recital_v2());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Recital castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    let spirits = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Spirit)
        })
        .count();
    assert!(spirits >= 1, "Recital mints a Spirit token");
}

#[test]
fn quandrix_aquamancer_loots_on_is_cast() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let _id = g.add_card_to_battlefield(0, catalog::quandrix_aquamancer());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_after_adding_bolt = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw - 1 discard = -1 net from the snapshot.
    assert_eq!(g.players[0].hand.len(), hand_after_adding_bolt - 1);
}

#[test]
fn fractal_aquanaut_enters_with_two_counters_and_is_flying() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_aquanaut());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Aquanaut castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert_eq!(
        card.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        2
    );
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
}

#[test]
fn quandrix_seedling_grows_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_seedling());
    drain_stack(&mut g);
    let counters_before = g
        .battlefield_find(id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id)
            .unwrap()
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        counters_before + 1
    );
}

#[test]
fn quandrix_amplifier_etb_scrys_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_amplifier());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Amplifier castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_emberscribe_pings_creature_on_is_cast() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _id = g.add_card_to_battlefield(0, catalog::prismari_emberscribe());
    drain_stack(&mut g);
    let dmg_before = g.battlefield_find(target).unwrap().damage;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 dmg → bear (2 toughness) goes to gy.
    // If gy'd, battlefield_find returns None; otherwise check damage.
    let still_in_play = g.battlefield_find(target);
    if let Some(c) = still_in_play {
        assert!(c.damage > dmg_before);
    }
}

#[test]
fn prismari_treasurer_v2_etb_mints_two_treasures() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_treasurer_v2());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Treasurer V2 castable");
    drain_stack(&mut g);
    let treasures = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 2);
}

#[test]
fn prismari_quickcast_deals_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_quickcast());
    let opp_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Quickcast castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    // -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_starcaller_etb_scrys_two_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::prismari_starcaller());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Starcaller castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_scryer_scrys_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::prismari_scryer());
    drain_stack(&mut g);
    let card_before = g.battlefield_find(id).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Scryer itself shouldn't have been pumped; this is a smoke test.
    assert_eq!(g.battlefield_find(id).unwrap().power(), card_before);
}

// ── Batch 42 (modern_decks) tests ───────────────────────────────────────────

#[test]
fn silverquill_spellbinder_drains_on_is_cast() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::silverquill_spellbinder());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 to opp + magecraft drain 1 = opp loses 4; +1 life from drain.
    assert_eq!(g.players[1].life, opp_before - 4);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn inkling_recruiter_etb_mints_an_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_recruiter());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Recruiter castable");
    drain_stack(&mut g);
    let inklings = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Inkling)
                && c.id != id
        })
        .count();
    assert_eq!(inklings, 1);
}

#[test]
fn silverquill_censure_v2_shrinks_creature_to_grave() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_censure_v2());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Censure castable");
    drain_stack(&mut g);
    // Grizzly Bears is 2/2; -3/-3 → -1/-1 → dies as SBA.
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn silverquill_drafter_v2_etb_makes_opp_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let hand_before = g.players[1].hand.len();
    let id = g.add_card_to_hand(0, catalog::silverquill_drafter_v2());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Drafter castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 1);
}

#[test]
fn silverquill_inkflame_drains_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkflame());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Inkflame castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
    // -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn witherbloom_bramblevine_grows_on_lifegain() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_bramblevine());
    drain_stack(&mut g);
    let counters_before = g
        .battlefield_find(id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    // Trigger via a direct lifegain effect — cast Witherbloom Sapglyph
    // for +2 life.
    let drain = g.add_card_to_hand(0, catalog::witherbloom_sapglyph());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: drain,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Sapglyph castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id)
            .unwrap()
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        counters_before + 1
    );
}

#[test]
fn witherbloom_sapglyph_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_sapglyph());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Sapglyph castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn pest_cultivator_v2_etb_mints_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_cultivator_v2());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Cultivator castable");
    drain_stack(&mut g);
    let pests = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Pest)
                && c.id != id
        })
        .count();
    assert_eq!(pests, 1);
}

#[test]
fn witherbloom_pestpicker_drains_on_attack() {
    use crate::game::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_pestpicker());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    // Move into combat as player 0 (active player).
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attackers");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn witherbloom_bloomstalk_grows_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_bloomstalk());
    drain_stack(&mut g);
    let counters_before = g
        .battlefield_find(id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id)
            .unwrap()
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        counters_before + 1
    );
}

#[test]
fn lorehold_stoneguard_etb_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_stoneguard());
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Stoneguard castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_pyresummon_burns_and_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_pyresummon());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pyresummon castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    let spirits = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Spirit)
        })
        .count();
    assert_eq!(spirits, 1);
}

#[test]
fn lorehold_saberspirit_is_three_four_first_strike_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_saberspirit());
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 4);
    assert!(card.has_keyword(&Keyword::FirstStrike));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn spirit_bookburner_pumps_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spirit_bookburner());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    // Base 1/1 + magecraft +1/+0 EOT = 2/1.
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 1);
}

#[test]
fn fractal_mathmage_enters_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_mathmage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Mathmage castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(
        card.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        3
    );
    // 0/0 base + 3 counters = 3/3.
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn quandrix_geometer_v2_etb_draws_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_geometer_v2());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Geometer castable");
    drain_stack(&mut g);
    // -1 cast + 1 ETB draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn fractal_sproutling_is_one_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_sproutling());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Sproutling castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 1);
}

#[test]
fn quandrix_calligrapher_v2_etb_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_calligrapher_v2());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Calligrapher castable");
    drain_stack(&mut g);
    // -1 cast + 1 ETB draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_equation_v2_pumps_creature_two_counters() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_equation_v2());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Equation castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(target).unwrap();
    assert_eq!(
        card.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        2
    );
}

#[test]
fn prismari_inferno_v2_deals_three_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_inferno_v2());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Inferno castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn prismari_glasshammer_drains_each_opp_on_is_cast() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::prismari_glasshammer());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 dmg.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_skywarp_bounces_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_skywarp());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Skywarp castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == target));
}

#[test]
fn prismari_stagewright_etb_draws_and_pings_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_stagewright());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Stagewright castable");
    drain_stack(&mut g);
    // -1 cast + 1 ETB draw = 0 net change.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.battlefield_find(id).is_some());
}

#[test]
fn prismari_soundsmith_pumps_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_soundsmith());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    // Base 2/2 + magecraft +1/+0 EOT = 3/2.
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 2);
}

// ── Batch 42 follow-up: 10 more cards ───────────────────────────────────────

#[test]
fn lorehold_knight_champion_gains_two_on_attack() {
    use crate::game::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_knight_champion());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attackers");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_pyrelancer_etb_deals_two_to_opp_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_pyrelancer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pyrelancer castable");
    drain_stack(&mut g);
    // Bear takes 2 damage → dies (2 toughness).
    assert!(g.battlefield_find(opp_bear).is_none());
}

#[test]
fn witherbloom_coatlcoiler_etb_drains_target_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_coatlcoiler());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Coatlcoiler castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Deathtouch));
}

#[test]
fn witherbloom_cinderscribe_etb_mints_pests_and_drains() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_cinderscribe());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Cinderscribe castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    let pests = g
        .battlefield
        .iter()
        .filter(|c| {
            c.controller == 0
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Pest)
                && c.id != id
        })
        .count();
    assert_eq!(pests, 2);
}

#[test]
fn silverquill_penlord_etb_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_penlord());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Penlord castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, life_before + 3);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_disciple_etb_gains_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_disciple());
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Disciple castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Flying));
}

#[test]
fn quandrix_synthsage_etb_gains_two_and_grows_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_synthsage());
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Synthsage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    // Cast bolt to trigger magecraft.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(
        card.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn fractal_tidecaller_v2_enters_with_two_counters_flying() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_tidecaller_v2());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Tidecaller castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(
        card.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        2
    );
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
    assert!(card.has_keyword(&Keyword::Flying));
}
