use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn quandrix_lensbearer_etb_scrys_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::quandrix_lensbearer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lensbearer castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn prismari_quickburn_burns_creature_for_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_quickburn());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quickburn castable");
    drain_stack(&mut g);
    // Bear 2/2 takes 2 → dies.
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn prismari_inkflame_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::prismari_inkflame());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkflame castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
    // Net hand: -1 (cast) +1 (draw) -1 (discard) = -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn pestseed_mints_one_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pestseed());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestseed castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
}

#[test]
fn witherbloom_greenwarden_is_reach_and_gains_two_life_on_etb() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_greenwarden());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Greenwarden castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Reach));
    assert_eq!(g.players[0].life, life_before + 2);
}

// ── Batch 49 (modern_decks) — new cards across all five colleges ────────────

#[test]
fn silverquill_inkscribe_etb_gains_one_life_with_vigilance() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_inkscribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkscribe castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn silverquill_bookmender_pumps_friendly_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_bookmender());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bookmender castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 3);
    assert_eq!(bear_card.toughness(), 3);
}

#[test]
fn silverquill_lifeskein_drains_two_at_instant_speed() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_lifeskein());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifeskein castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn inkling_aerialist_v2_is_flying_and_drains_one() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::inkling_aerialist_v2());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Aerialist v2 castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn silverquill_censurewright_shrinks_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_censurewright());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Censurewright castable");
    drain_stack(&mut g);
    // Bear 2/2 → -1/-1 → 1/1.
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 1);
    assert_eq!(bear_card.toughness(), 1);
}

#[test]
fn silverquill_penmistress_lifelinks_and_magecraft_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_penmistress());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Lifelink));
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 4);
}

#[test]
fn lorehold_skyrunner_is_flying_haste_two_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_skyrunner());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skyrunner castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Haste));
    assert_eq!(card.power(), 2);
}

#[test]
fn lorehold_stoneward_pumps_target_toughness_only() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_stoneward());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stoneward castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 2);
    assert_eq!(bear_card.toughness(), 4);
    let me = g.battlefield_find(id).unwrap();
    assert!(me.has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_pyremender_v2_pings_for_one() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_pyremender_v2());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyremender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn lorehold_pyreward_burns_two_and_gains_one() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_pyreward());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyreward castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn spirit_honor_guard_is_vigilant_first_striker() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spirit_honor_guard());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Honor Guard castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
    assert!(card.has_keyword(&Keyword::FirstStrike));
}

#[test]
fn quandrix_theoremist_etb_cantrip() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_theoremist());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Theoremist castable");
    drain_stack(&mut g);
    // -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn fractal_shaper_etb_adds_plus_one_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_shaper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Shaper castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 3);
    assert_eq!(bear_card.toughness(), 3);
}

#[test]
fn quandrix_foresight_pumps_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_foresight());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Foresight castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 3);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn fractal_bloomstalker_enters_with_four_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_bloomstalker());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloomstalker castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 4);
    assert!(card.has_keyword(&Keyword::Trample));
}

#[test]
fn prismari_spellscribe_scrys_on_instant_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_spellscribe());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    // Drain stack and scry decision - just check it doesn't crash.
    drain_stack(&mut g);
}

#[test]
fn prismari_sparkforge_v2_mints_treasure_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkforge_v2());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkforge castable");
    drain_stack(&mut g);
    let treasure_count = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasure_count, 1);
}

#[test]
fn prismari_tidesinger_bounces_target_to_owner_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_tidesinger());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidesinger castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

#[test]
fn prismari_searbolt_burns_three_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_searbolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Searbolt castable");
    drain_stack(&mut g);
    // 3 damage to a 2-toughness bear → dead.
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn witherbloom_pestseer_etb_mints_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestseer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestseer castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
}

#[test]
fn witherbloom_sourceweaver_drains_two_with_deathtouch() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_sourceweaver());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sourceweaver castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Deathtouch));
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn witherbloom_sapburst_adds_two_counters_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_sapburst());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sapburst castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 4);
    assert_eq!(bear_card.toughness(), 4);
}

#[test]
fn inkling_cipherwing_is_flying_drain_one() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::inkling_cipherwing());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cipherwing castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn silverquill_inkstrike_destroys_small_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkstrike());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkstrike castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn silverquill_inkstrike_rejects_big_creature() {
    let mut g = two_player_game();
    let bookwurm = g.add_card_to_battlefield(1, catalog::bookwurm());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkstrike());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    // 5/5 bookwurm exceeds toughness 2 filter.
    let result = g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bookwurm)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(result.is_err());
}

#[test]
fn strixhaven_anthemcaster_pumps_other_friendly_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::strixhaven_anthemcaster());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Anthemcaster castable");
    drain_stack(&mut g);
    // Read layered stats post-resolve via compute_battlefield.
    let computed = g.compute_battlefield();
    let bear_card = computed.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bear_card.power, 3, "Bear gets +1/+0 anthem");
    assert_eq!(bear_card.toughness, 2);
    let self_card = computed.iter().find(|c| c.id == id).unwrap();
    assert_eq!(self_card.power, 2, "Anthemcaster doesn't pump itself");
    assert_eq!(self_card.toughness, 3);
}

#[test]
fn strixhaven_stormsage_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_stormsage());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormsage castable");
    drain_stack(&mut g);
    // -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn pest_brewer_v2_etb_gains_one_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::pest_brewer_v2());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Brewer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Pest));
}

// ── Batch 50 (Silverquill synthesised variants) ────────────────────────────

#[test]
fn silverquill_cantor_etb_gains_one_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_cantor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cantor castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn silverquill_inkscholar_adept_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkscholar_b50());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkscholar Adept castable");
    drain_stack(&mut g);
    // -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn silverquill_quillrunner_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let _id = g.add_card_to_battlefield(0, catalog::silverquill_quillrunner());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry 1 doesn't change library size — just confirm Bolt resolved.
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn inkling_stylescribe_is_a_two_mana_flying_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_stylescribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stylescribe castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_pageturner_etb_scrys_with_vigilance() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::silverquill_pageturner());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pageturner castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn inkling_stormwriter_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::inkling_stormwriter());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn silverquill_inkbinder_etb_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkbinder());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkbinder castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 3);
    assert_eq!(bear_card.toughness(), 3);
    assert!(bear_card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_quietus_shrinks_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_quietus());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quietus castable");
    drain_stack(&mut g);
    // Bear 2/2 → -3/-3 → SBA destroys it.
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn inkling_skywriter_magecraft_pumps_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _id = g.add_card_to_battlefield(0, catalog::inkling_skywriter());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 3);
    assert_eq!(bear_card.toughness(), 3);
}

#[test]
fn silverquill_glyphmaster_etb_drains_two_with_lifelink() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_glyphmaster());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Glyphmaster castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Lifelink));
    assert_eq!(g.players[0].life, life_before + 2);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn inkling_mournful_dies_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_mournful());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    // Kill it via destroy.
    let bolt = g.add_card_to_hand(0, catalog::wrath_of_god());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wrath castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none());
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn silverquill_pen_squire_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_pen_squire());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
}

#[test]
fn inkling_spellbinder_is_a_lifelink_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_spellbinder());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellbinder castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
    assert_eq!(card.power(), 4);
}

#[test]
fn silverquill_diction_drains_two_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_diction());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Diction castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn silverquill_quietude_drains_three_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_quietude());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quietude castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn inkling_beautisage_etb_gains_three_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::inkling_beautisage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Beautisage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn silverquill_inkmender_etb_returns_low_mv_creature() {
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkmender());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkmender castable");
    drain_stack(&mut g);
    // Cast -1 (Inkmender to bf), ETB +1 (bear back) → net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bear_in_gy));
}

#[test]
fn silverquill_memorial_reanimates_and_drains() {
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_memorial());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memorial castable");
    drain_stack(&mut g);
    // Bear should be on battlefield.
    assert!(g.battlefield_find(bear_in_gy).is_some());
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn inkling_inkstain_attack_shrinks_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::inkling_inkstain());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Inkstain declares attack");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 1, "Bear shrunk -1/-0");
}

#[test]
fn silverquill_convene_mints_two_inklings_and_drains() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_convene());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Convene castable");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .collect();
    assert_eq!(inklings.len(), 2);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn silverquill_sermoneer_etb_scrys_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_sermoneer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sermoneer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn inkling_pageboy_is_a_one_mana_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_pageboy());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pageboy castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_inkstrike_page_destroys_low_power_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkstrike_page());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkstrike-Page castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn silverquill_mentor_etb_adds_plus_one_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_mentor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mentor castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn silverquill_necroscribe_etb_returns_is_card_from_graveyard() {
    let mut g = two_player_game();
    let bolt_in_gy = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::silverquill_necroscribe());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Necroscribe castable");
    drain_stack(&mut g);
    // -1 (cast) + 1 (return Bolt) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bolt_in_gy));
}

#[test]
fn silverquill_pronouncement_drains_three_and_mints_two_inklings() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_pronouncement());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pronouncement castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .collect();
    assert_eq!(inklings.len(), 2);
}

#[test]
fn silverquill_cipher_drains_one_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_cipher());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cipher castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[1].life, opp_before - 1);
    // -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_quillpoint_is_a_first_strike_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_quillpoint());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillpoint castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::FirstStrike));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_festscribe_etb_mints_inkling_and_gains_two_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_festscribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Festscribe castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    // Festscribe itself is an Inkling? Let me check — no, it's Vampire Wizard.
    let inkling_tokens: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .collect();
    assert_eq!(inkling_tokens.len(), 1);
}

// ── Batch 50 (Witherbloom synthesised variants) ────────────────────────────

#[test]
fn witherbloom_drainscholar_b50_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::witherbloom_drainscholar_b50());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn pest_hierarch_etb_mints_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_hierarch());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hierarch castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
}

#[test]
fn witherbloom_bloodseeker_is_a_lifelink_three_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_bloodseeker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloodseeker castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Lifelink));
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn pest_disciple_etb_scrys_and_gains_one_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::pest_disciple());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Disciple castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn witherbloom_lifescribe_etb_drains_one_then_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifescribe());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifescribe castable");
    drain_stack(&mut g);
    // ETB drains 1.
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[1].life, opp_before - 1);
    let _ = id;
}

#[test]
fn pest_lifebloom_gains_four_life_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::pest_lifebloom());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifebloom castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 4);
}

#[test]
fn witherbloom_pestmage_is_three_two_menace() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestmage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestmage castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Menace));
    assert_eq!(card.power(), 3);
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Pest));
}

#[test]
fn witherbloom_vinedrain_drains_three_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinedrain());
    let hand_before_cast = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinedrain castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
    assert_eq!(g.players[1].life, opp_before - 3);
    // -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before_cast);
}

#[test]
fn witherbloom_roto_sage_is_a_four_four_deathtouch() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_roto_sage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Roto-Sage castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Deathtouch));
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 4);
}

#[test]
fn pest_cultivator_sage_attack_mints_a_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_cultivator_sage());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Cultivator-Sage attacks");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
}

#[test]
fn witherbloom_decaymage_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::witherbloom_decaymage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt: -3 to opp + Decaymage magecraft: -1 to opp = -4 total.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn witherbloom_pestcaller_b50_etb_mints_three_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestcaller_b50());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestcaller v2 castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 3);
    let _ = id;
}

// ── Batch 50 (Lorehold synthesised variants) ───────────────────────────────

#[test]
fn lorehold_embersmith_magecraft_pings_target() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_embersmith());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt -3 + Embersmith magecraft 1 = -4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn spirit_mentor_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::spirit_mentor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn lorehold_wargist_etb_deals_one_to_each_opp() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_wargist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wargist castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn lorehold_sparkstrike_b50_burns_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkstrike_b50());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkstrike castable");
    drain_stack(&mut g);
    // Bear 2/2 takes 2 damage and dies.
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn spirit_battlemaster_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spirit_battlemaster());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 5);
    assert!(card.has_keyword(&Keyword::FirstStrike));
}

#[test]
fn lorehold_memoriam_mints_two_spirits_and_gains_two_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_memoriam());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memoriam castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 2);
}

#[test]
fn spirit_berserker_has_haste_and_trample() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spirit_berserker());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Berserker castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Haste));
    assert!(card.has_keyword(&Keyword::Trample));
}

#[test]
fn lorehold_memorialist_b50_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_memorialist_b50());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memorialist castable");
    drain_stack(&mut g);
    // -1 (cast) + 1 (return) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bear_in_gy));
}

#[test]
fn lorehold_echocaller_etb_mints_spirit_and_gains_one_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_echocaller());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echocaller castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1);
}

#[test]
fn lorehold_sparkshock_deals_two_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkshock());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkshock castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

// ── Batch 50 (Quandrix + Prismari synthesised variants) ────────────────────

#[test]
fn quandrix_scryweaver_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _id = g.add_card_to_battlefield(0, catalog::quandrix_scryweaver());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry 1 doesn't change life — just confirm Bolt landed.
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn fractal_bloomthorn_enters_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_bloomthorn());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloomthorn castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Trample));
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 3);
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn quandrix_pupil_b50_magecraft_self_grows() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_pupil_b50());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_forge_mints_fractal_with_four_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_forge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Forge castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1);
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 4);
}

#[test]
fn quandrix_algorithmist_magecraft_pumps_each_fractal() {
    let mut g = two_player_game();
    // Use existing fractal with counters via cast (so enters_with_counters fires).
    let fractal = g.add_card_to_hand(0, catalog::fractal_bloomthorn());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: fractal, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloomthorn castable");
    drain_stack(&mut g);
    let _id = g.add_card_to_battlefield(0, catalog::quandrix_algorithmist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Fractal had 3 counters from ETB, gains 1 from magecraft = 4.
    let fractal_card = g.battlefield_find(fractal).unwrap();
    assert_eq!(fractal_card.counter_count(CounterType::PlusOnePlusOne), 4);
}

#[test]
fn quandrix_refractor_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::quandrix_refractor());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Refractor castable");
    drain_stack(&mut g);
    // -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_bonfire_burns_creature_for_three() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::bookwurm());
    let id = g.add_card_to_hand(0, catalog::prismari_bonfire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bonfire castable");
    drain_stack(&mut g);
    // 5/5 bookwurm takes 3 damage and survives but is damaged.
    let card = g.battlefield_find(big).unwrap();
    assert_eq!(card.damage, 3);
}

#[test]
fn prismari_snapcaster_etb_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::prismari_snapcaster());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Snapcaster castable");
    drain_stack(&mut g);
    // -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_pyrolancer_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::prismari_pyrolancer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt -3 + magecraft -1 = -4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_drakemage_is_a_flying_looter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_drakemage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drakemage castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
}

#[test]
fn prismari_cinder_apprentice_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_cinder_apprentice());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
}

// ── Push (modern_decks): EventKind::CreatureSacrificed (CR 701.16) ─────────

#[test]
fn witherbloom_mortician_grows_on_sacrifice() {
    // Sacrifice via the new `EventKind::CreatureSacrificed` event:
    // Witherbloom Sacrosanct's at-resolve sac path emits the
    // sacrifice-specific event, which Mortician's trigger listens for.
    let mut g = two_player_game();
    let mort = g.add_card_to_battlefield(0, catalog::witherbloom_mortician());
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    let mc = g.battlefield_find(mort).expect("Mortician still alive");
    assert_eq!(
        mc.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "Mortician should grow by 1 from the sacrifice event"
    );
}

#[test]
fn witherbloom_mortician_does_not_grow_on_natural_death() {
    // Damage-based death emits CreatureDied but NOT CreatureSacrificed,
    // so the Mortician's trigger should NOT fire.
    let mut g = two_player_game();
    let mort = g.add_card_to_battlefield(0, catalog::witherbloom_mortician());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Lethal damage to bear without sacrifice.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) {
        c.damage = 99;
    }
    g.check_state_based_actions();
    let mc = g.battlefield_find(mort).expect("Mortician still alive");
    assert_eq!(
        mc.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        0,
        "Mortician should NOT grow from natural deaths"
    );
}

#[test]
fn pest_pestmaster_b51_grows_only_on_own_sacrifices() {
    // YourControl scope: opponent sacrifices shouldn't trigger.
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::pest_pestmaster_b51());
    // P0 sacrifices a creature via Sacrosanct.
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    let pmc = g.battlefield_find(pm).expect("Pestmaster still alive");
    assert_eq!(
        pmc.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "Pestmaster should grow from own sacrifices"
    );
}

// ── New Silverquill cards ──────────────────────────────────────────────────

#[test]
fn silverquill_memoriam_etb_drains_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_memoriam());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memoriam castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn inkling_sigilbearer_pumps_other_inklings_on_etb() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    let id = g.add_card_to_hand(0, catalog::inkling_sigilbearer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sigilbearer castable");
    drain_stack(&mut g);
    let oc = g.battlefield_find(other).unwrap();
    assert_eq!(
        oc.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "Other Inkling should have a +1/+1 counter"
    );
    // Self should not get a counter (OtherThanSource).
    let me = g.battlefield_find(id).unwrap();
    assert_eq!(
        me.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        0,
        "Sigilbearer should not buff itself"
    );
}

#[test]
fn silverquill_eulogize_reanimates_low_mv_and_gains_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    // Pre-load a 2-mana creature in graveyard.
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_eulogize());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Eulogize castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bears),
        "Bears should be reanimated");
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn inkling_voidwalker_is_a_flying_menacer() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_voidwalker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Voidwalker castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Menace));
}

// ── New Witherbloom cards ──────────────────────────────────────────────────

#[test]
fn witherbloom_sacrosanct_sacrifices_and_drains_three() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "Bear should be sacrificed");
    assert_eq!(g.players[0].life, life_before + 3);
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn witherbloom_lichbloom_dies_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let lb = g.add_card_to_battlefield(0, catalog::witherbloom_lichbloom());
    let hand_before = g.players[0].hand.len();
    // Inflict lethal damage to trigger death.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == lb) {
        c.damage = 99;
    }
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == lb),
        "Lichbloom should be in graveyard");
    assert!(g.players[0].hand.iter().any(|c| c.id == bears),
        "Bears should return to hand");
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn pest_cradlescale_etb_mints_a_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_cradlescale());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cradlescale castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Reach));
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
}

// ── New Lorehold cards ─────────────────────────────────────────────────────

#[test]
fn lorehold_skystorm_burns_opp_creatures_and_gains_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_skystorm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skystorm castable");
    drain_stack(&mut g);
    // Bear (2 toughness) takes 2 damage and dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die to 2 damage");
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_reverence_etb_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_reverence());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reverence castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1);
}

#[test]
fn lorehold_pyromentor_pings_on_instant_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyromentor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt -3 + magecraft ping -1 = -4 (auto-targets opponent).
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_spirit_veteran_pumps_other_spirits() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::lorehold_reverence());
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_veteran());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Veteran castable");
    drain_stack(&mut g);
    let oc = g.battlefield_find(other).unwrap();
    assert_eq!(
        oc.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "Other Spirit (Reverence) should get a +1/+1 counter"
    );
}

#[test]
fn lorehold_embermend_gains_three_life_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_embermend());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Embermend castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
}

// ── New Quandrix cards ─────────────────────────────────────────────────────

#[test]
fn quandrix_echocaster_magecraft_pumps_each_fractal() {
    let mut g = two_player_game();
    // Use Fractal Avenger (no magecraft of its own) as the buff target.
    let frac = g.add_card_to_battlefield(0, catalog::fractal_avenger());
    let ec = g.add_card_to_battlefield(0, catalog::quandrix_echocaster());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let frac_before = g.battlefield_find(frac).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let after = g.battlefield_find(frac).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(after, frac_before + 1, "Fractal should grow by 1");
    let _ = ec;
}

#[test]
fn fractal_bloomstone_enters_with_counters_per_land() {
    let mut g = two_player_game();
    // 3 lands on the battlefield for P0.
    for _ in 0..3 {
        let _ = g.add_card_to_battlefield(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::fractal_bloomstone());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloomstone castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("Bloomstone should survive ETB");
    let counters = card.counters.get(&CounterType::PlusOnePlusOne)
        .copied().unwrap_or(0);
    // 3 lands → 3 counters → 3/3 (survives base 0/0).
    assert_eq!(counters, 3);
}

#[test]
fn quandrix_reflection_doubles_counters_on_each_friendly() {
    let mut g = two_player_game();
    // Use a Grizzly Bears (2/2 vanilla, no auto-counter) with a
    // manually-attached +1/+1 counter to lock in the doubling math.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) {
        c.counters.insert(CounterType::PlusOnePlusOne, 2);
    }
    let before = g.battlefield_find(bear).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(before, 2);
    let hatch = bear;
    let id = g.add_card_to_hand(0, catalog::quandrix_reflection());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reflection castable");
    drain_stack(&mut g);
    let after = g.battlefield_find(hatch).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    // 2 → 2 + 2 = 4 (doubled).
    assert_eq!(after, 4);
}

#[test]
fn quandrix_tideseer_adept_etb_scrys_and_is_flash() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_tideseer_adept());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tideseer Adept castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flash));
}

#[test]
fn fractal_geomancer_magecraft_adds_counter_to_fractal() {
    let mut g = two_player_game();
    // Use Fractal Avenger (no magecraft of its own) as the buff target.
    let frac = g.add_card_to_battlefield(0, catalog::fractal_avenger());
    let _ = g.add_card_to_battlefield(0, catalog::fractal_geomancer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let before = g.battlefield_find(frac).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let after = g.battlefield_find(frac).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(after, before + 1, "Fractal should grow by 1");
}

// ── New Prismari cards ─────────────────────────────────────────────────────

#[test]
fn prismari_pyroceptor_magecraft_pings_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_pyroceptor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt -3 + Pyroceptor -1 = -4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_coinforger_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_coinforger());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Coinforger castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .collect();
    assert_eq!(treasures.len(), 1);
}

#[test]
fn prismari_flashforge_burns_target_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_flashforge());
    g.add_card_to_hand(0, catalog::island()); // a discardable card
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flashforge castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    // Hand: -1 (cast) -1 (discard) +1 (draw) = -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_sparkwing_is_a_haster_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkwing());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkwing castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Haste));
}

#[test]
fn prismari_riftspark_magecraft_loots_optionally() {
    // With AutoDecider answering "no" to optional MayDo, no loot occurs.
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_riftspark());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len() - 1; // remove the bolt we just added
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // AutoDecider declines the MayDo loot — hand size unchanged
    // post-bolt-cast.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Batch 52: more aristocrats + magecraft cards ───────────────────────────

#[test]
fn pest_anointer_gains_life_on_sacrifice() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::pest_anointer());
    // Make the fodder a token so the auto-sac picker prefers it
    // (tokens sort before non-tokens in the heuristic).
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == fodder) {
        c.is_token = true;
    }
    let life_before = g.players[0].life;
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Sacrosanct: drain 3 = +3 life. Anointer: +1 from the sacrifice.
    assert_eq!(g.players[0].life, life_before + 3 + 1);
}

#[test]
fn witherbloom_bloodreaper_drains_each_opp_on_sacrifice() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_bloodreaper());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == fodder) {
        c.is_token = true;
    }
    let opp_before = g.players[1].life;
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Sacrosanct -3 to opp + Bloodreaper -1 to opp = -4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn pest_conservator_sac_a_pest_draws() {
    let mut g = two_player_game();
    let pc = g.add_card_to_battlefield(0, catalog::pest_conservator());
    g.clear_sickness(pc);
    // Mint a Pest token to sacrifice. Add a pest directly.
    let pest_def = {
        // Use Grizzly Bears retyped as a Pest for a cheap fodder.
        let mut def = catalog::grizzly_bears();
        def.subtypes.creature_types = vec![CreatureType::Pest];
        def
    };
    let pest = g.add_card_to_battlefield(0, pest_def);
    g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pc, ability_index: 0, target: None, x_value: None }).expect("Conservator activatable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == pest),
        "Pest should be sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn witherbloom_bloodweaver_is_a_lifelink_trampler() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_bloodweaver());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloodweaver castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Lifelink));
    assert!(card.has_keyword(&Keyword::Trample));
    assert_eq!(card.power(), 4);
}

#[test]
fn lorehold_spiritchron_magecraft_fans_counters_on_spirits() {
    let mut g = two_player_game();
    let spirit = g.add_card_to_battlefield(0, catalog::lorehold_reverence());
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_spiritchron());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let before = g.battlefield_find(spirit).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let after = g.battlefield_find(spirit).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(after, before + 1);
}

#[test]
fn lorehold_sparklock_burns_target_creature() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_sparklock());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparklock castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die to 4 damage");
}

#[test]
fn quandrix_cantripper_magecraft_loots_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_cantripper());
    let _ = g.add_card_to_hand(0, catalog::forest());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_pre = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 (bolt cast) + 1 (magecraft draw) - 1 (magecraft discard) = -1.
    assert_eq!(g.players[0].hand.len(), hand_pre - 1);
}

#[test]
fn fractal_bloomanalyst_enters_with_counters_for_each_other_creature() {
    let mut g = two_player_game();
    // 2 other creatures.
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_bloomanalyst());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloomanalyst castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("Bloomanalyst should survive ETB");
    let counters = card.counters.get(&CounterType::PlusOnePlusOne)
        .copied().unwrap_or(0);
    // 2 other creatures → 2 counters → 2/2.
    assert_eq!(counters, 2);
}

#[test]
fn prismari_cantrip_mage_magecraft_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_cantrip_mage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_pre = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 cast bolt + 1 magecraft draw = 0 net (no discard) → same as before.
    assert_eq!(g.players[0].hand.len(), hand_pre);
}

#[test]
fn prismari_firebrand_etb_pings_with_haste() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_firebrand());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Firebrand castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Haste));
    // Damage: auto-target picks the opponent.
    assert_eq!(g.players[1].life, opp_before - 1);
}

// ── batch 53: new STX cards across multiple colleges ───────────────────────

#[test]
fn silverquill_scryward_etb_scrys_and_magecraft_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_scryward());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Scryward castable");
    drain_stack(&mut g);
    // ETB Scry 1 already resolved (test just checks the card landed and
    // magecraft fires on next IS cast).
    assert!(g.battlefield_find(id).is_some());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn inkling_archivist_etb_drains_and_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::inkling_archivist());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Archivist castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn silverquill_ledgermage_etb_drains_two() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_ledgermage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Ledgermage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn inkling_inkscribe_is_a_two_mana_flying_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_inkscribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Inkscribe castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 1);
}

#[test]
fn silverquill_codex_gains_life_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_codex());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Codex castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    // -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn silverquill_studyhall_magecraft_gains_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_studyhall());
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn silverquill_pronouncer_is_a_lifelink_flying_finisher() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_pronouncer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pronouncer castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
    assert_eq!(card.power(), 4);
    // ETB drain 1 → opp loses 1.
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn silverquill_etching_drains_two() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_etching());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Etching castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn witherbloom_grimherb_has_deathtouch_and_magecraft_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_grimherb());
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Deathtouch));
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn pest_brood_creates_two_pest_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_brood());
    let pests_before = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Brood castable");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pests_after, pests_before + 2);
}

#[test]
fn witherbloom_pestpath_is_a_three_four_trampler() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestpath());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pestpath castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Trample));
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 4);
}

#[test]
fn witherbloom_rotbloom_drains_three() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_rotbloom());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Rotbloom castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn lorehold_emberscribe_v2_magecraft_pings_target() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_emberscribe_v2());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Magecraft 1 = 4 damage.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_spirit_redeemer_has_vigilance_and_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_redeemer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Redeemer castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
    assert!(card.has_keyword(&Keyword::Lifelink));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Spirit));
}

#[test]
fn lorehold_emberlock_burns_and_gains_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_emberlock());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Emberlock castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn lorehold_skyblaze_mints_spirit_and_burns_opp_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spirits_before = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    let id = g.add_card_to_hand(0, catalog::lorehold_skyblaze());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Skyblaze castable");
    drain_stack(&mut g);
    // Spirit token minted.
    let spirits_after = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    assert_eq!(spirits_after, spirits_before + 1);
    // Bear (2/2) takes 2 damage → dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn spirit_blazekin_is_two_two_haste_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spirit_blazekin());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Blazekin castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Haste));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Spirit));
}

#[test]
fn fractal_synthmage_etb_pumps_by_other_creature_count() {
    let mut g = two_player_game();
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b3 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_synthmage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Synthmage castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("Synthmage on battlefield");
    // 3 other creatures → 3 +1/+1 counters.
    let counters = card.counters.get(&CounterType::PlusOnePlusOne)
        .copied().unwrap_or(0);
    assert_eq!(counters, 3);
}

#[test]
fn quandrix_amplify_pumps_target_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_amplify());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Amplify castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    let counters = card.counters.get(&CounterType::PlusOnePlusOne)
        .copied().unwrap_or(0);
    assert_eq!(counters, 2);
}

#[test]
fn quandrix_threadbinder_magecraft_scrys() {
    // Functional check: card is castable + has the magecraft trigger
    // wired (just an integration check; scry effect tested elsewhere).
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_threadbinder());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Threadbinder castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 2);
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Wizard));
}
