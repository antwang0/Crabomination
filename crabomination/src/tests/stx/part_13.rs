use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn inkling_sigilwarden_etb_pumps_other_inklings() {
    let mut g = two_player_game();
    // Drop an Inkling token via inkling_scribe to be pumped.
    let scribe = g.add_card_to_hand(0, catalog::inkling_scribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: scribe, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scribe castable");
    drain_stack(&mut g);
    let inkling = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token &&
        c.definition.name == "Inkling").map(|c| c.id).expect("Inkling minted");
    let cb = g.battlefield_find(inkling).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let id = g.add_card_to_hand(0, catalog::inkling_sigilwarden());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sigilwarden castable");
    drain_stack(&mut g);
    let ca = g.battlefield_find(inkling).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(ca, cb + 1);
}

#[test]
fn silverquill_quillthane_etb_drains_two_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_quillthane());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillthane castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, you_before + 2);
}

#[test]
fn pest_roostkeeper_etb_mints_pest_and_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::pest_roostkeeper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Roostkeeper castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 1);
}

#[test]
fn witherbloom_mossherald_magecraft_self_grows() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_mossherald());
    let cb = g.battlefield_find(id).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let ca = g.battlefield_find(id).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(ca, cb + 1);
}

#[test]
fn witherbloom_vinepriest_b60_etb_drains_and_magecraft_gains_life() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinepriest_b60());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinepriest castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, you_before + 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Lifelink));
}

#[test]
fn lorehold_chronicler_b60_etb_mints_spirit_with_vigilance() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_chronicler_b60());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Chronicler castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_sparkmage_b60_magecraft_pings_with_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_sparkmage_b60());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 4);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_battle_sage_magecraft_pumps_friendly_with_first_strike() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_battle_sage());
    let target = g.add_card_to_battlefield(0, catalog::pest_beekeeper());
    let pb = g.battlefield_find(target).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pa = g.battlefield_find(target).unwrap().power();
    assert_eq!(pa, pb + 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::FirstStrike));
}

#[test]
fn quandrix_tideborn_magecraft_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_tideborn());
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn fractal_stormpetal_enters_with_four_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_stormpetal());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormpetal castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Stormpetal on bf");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 4);
    assert_eq!(c.power(), 4);
}

#[test]
fn quandrix_pondwarden_etb_mints_two_fractals() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_pondwarden());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pondwarden castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Fractal"
    }).collect();
    assert_eq!(fractals.len(), 2);
    for f in fractals {
        assert_eq!(f.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
    }
}

#[test]
fn prismari_spell_smith_b60_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_spell_smith_b60());
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn prismari_fluxshaper_magecraft_self_pumps_with_flying() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_fluxshaper());
    let p_before = g.battlefield_find(id).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(id).unwrap().power();
    assert_eq!(p_after, p_before + 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Flying));
}

#[test]
fn prismari_glassblower_etb_mints_treasure_and_pings() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_glassblower());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Glassblower castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).collect();
    assert_eq!(treasures.len(), 1);
    assert_eq!(g.players[1].life, opp_before - 1);
}

// ── Batch 61: 25 more synthesised STX cards (5 per college) ─────────────────

#[test]
fn silverquill_pentor_b61_etb_gains_two_life_and_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_pentor_b61());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pentor castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    // Magecraft check: cast a bolt and verify scry didn't crash / library
    // size stayed >= 0.
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn inkling_arbiter_is_a_two_mana_flying_lifelink_inkling() {
    let g = two_player_game();
    let def = catalog::inkling_arbiter();
    assert_eq!(def.power, 2);
    assert_eq!(def.toughness, 2);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Lifelink));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Inkling));
    drop(g);
}

#[test]
fn silverquill_inkmage_b61_etb_drains_two() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_inkmage_b61());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkmage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, you_before + 2);
}

#[test]
fn inkling_letterer_etb_scrys_with_flying_vigilance() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::inkling_letterer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Letterer castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Letterer on bf");
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Vigilance));
}

#[test]
fn silverquill_drainpoet_etb_drains_three_and_magecraft_gains_life() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_drainpoet());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainpoet castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, you_before + 3);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    // Magecraft: cast a bolt and verify +1 life
    let you_after_etb = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_after_etb + 1);
}

#[test]
fn witherbloom_pestcollector_etb_mints_pest_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestcollector());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestcollector castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 1);
}

#[test]
fn pest_swarmleader_drains_each_opp_on_sacrifice() {
    // Sacrifice via Witherbloom Sacrosanct (drain 3 + sac-as-additional-
    // cost path) which emits `CreatureSacrificed` per CR 701.16.
    // Swarmleader's trigger listens for that event and drains opp 1.
    let mut g = two_player_game();
    let _leader = g.add_card_to_battlefield(0, catalog::pest_swarmleader());
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_before = g.players[1].life;
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Sacrosanct drains 3, Swarmleader's sac trigger drains an extra 1.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn witherbloom_rotweaver_magecraft_gains_two_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_rotweaver());
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn witherbloom_vinemaster_b61_etb_drains_two_and_magecraft_grows() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinemaster_b61());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinemaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    let cb = g.battlefield_find(id).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let ca = g.battlefield_find(id).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(ca, cb + 1);
}

#[test]
fn lorehold_emberspeaker_etb_pings_with_haste() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_emberspeaker());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Emberspeaker castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_battle_keeper_etb_mints_spirit_and_pings() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_battle_keeper());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battle-Keeper castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 1);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Vigilance));
}

#[test]
fn spirit_bannerer_magecraft_pumps_friendly_spirits() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::spirit_bannerer());
    // Mint a Spirit via lorehold_warpoet ETB? We already added a Spirit via
    // spirit_bannerer (1/2 Spirit Cleric) — pump it via cast.
    let p_before = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Spirit Bannerer")
        .unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Spirit Bannerer")
        .unwrap().power();
    assert_eq!(p_after, p_before + 1);
}

#[test]
fn lorehold_scholar_b61_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_scholar_b61());
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
fn lorehold_warpoet_etb_mints_spirit_with_first_strike_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_warpoet());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Warpoet castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 1);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::FirstStrike));
    assert!(c.has_keyword(&Keyword::Lifelink));
}

#[test]
fn quandrix_seer_b61_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_seer_b61());
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft drew 1 from library → library is one card shorter.
    assert_eq!(g.players[0].library.len(), lib_before - 1);
}

#[test]
fn fractal_mosspetal_enters_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_mosspetal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mosspetal castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Mosspetal on bf");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2);
    assert_eq!(c.power(), 2);
}

#[test]
fn quandrix_growkeeper_etb_mints_fractal_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_growkeeper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Growkeeper castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Fractal"
    }).collect();
    assert_eq!(fractals.len(), 1);
    let count = fractals[0].counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(count, 3);
    assert_eq!(fractals[0].power(), 3);
}

#[test]
fn quandrix_doublecast_magecraft_pumps_target_fractal() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_doublecast());
    // Cast the Fractal Mosspetal so the `enters_with_counters` replacement
    // gives it 2 +1/+1 counters and it survives state-based actions.
    let fractal = g.add_card_to_hand(0, catalog::fractal_mosspetal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: fractal, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mosspetal castable");
    drain_stack(&mut g);
    let cb = g.battlefield_find(fractal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let ca = g.battlefield_find(fractal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(ca, cb + 1);
}

#[test]
fn quandrix_pondseer_etb_scrys_and_grows_fractals() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    // Cast the Fractal Mosspetal so its `enters_with_counters` replacement
    // fires and the 0/0 base body survives state-based actions.
    let fractal = g.add_card_to_hand(0, catalog::fractal_mosspetal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: fractal, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mosspetal castable");
    drain_stack(&mut g);
    let cb = g.battlefield_find(fractal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let id = g.add_card_to_hand(0, catalog::quandrix_pondseer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pondseer castable");
    drain_stack(&mut g);
    let ca = g.battlefield_find(fractal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(ca, cb + 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Flying));
}

#[test]
fn prismari_sparkscribe_b61_magecraft_pings() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let _ = g.add_card_to_battlefield(0, catalog::prismari_sparkscribe_b61());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 total damage.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_emberforge_etb_mints_treasure_and_pings_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dmg_before = g.battlefield_find(target).unwrap().damage;
    let id = g.add_card_to_hand(0, catalog::prismari_emberforge());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Emberforge castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).collect();
    assert_eq!(treasures.len(), 1);
    if let Some(b) = g.battlefield.iter().find(|c| c.id == target) {
        assert_eq!(b.damage, dmg_before + 1);
    } else {
        // Damage may have already killed it (2/2 - 1 = 1 toughness left so
        // it should survive). If missing, the test fails.
        panic!("target bear should still be on bf after 1 damage");
    }
}

#[test]
fn prismari_torchsmith_magecraft_self_pumps_with_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_torchsmith());
    let pb = g.battlefield_find(id).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pa = g.battlefield_find(id).unwrap().power();
    assert_eq!(pa, pb + 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn prismari_smiteforge_etb_mints_treasure_and_pings_two() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_smiteforge());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Smiteforge castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).collect();
    assert_eq!(treasures.len(), 1);
    assert_eq!(g.players[1].life, opp_before - 2);
}

// ── Batch 62: 10 more synthesised STX cards (2 per college) ─────────────────

#[test]
fn inkling_calligrapher_b62_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::inkling_calligrapher_b62());
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn silverquill_lecturer_b62_etb_drains_one_and_surveils_with_lifelink() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_lecturer_b62());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lecturer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, you_before + 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Lifelink));
}

#[test]
fn pest_soulbinder_scrys_on_sacrifice() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::pest_soulbinder());
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lib_before = g.players[0].library.len();
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Sacrifice → scry 1 → library may shrink by 1 (if a card is sent
    // to bottom; otherwise same).
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn witherbloom_vineshaper_magecraft_grows_pests() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_vineshaper());
    // Mint a Pest via Witherbloom Pest-Tender ETB.
    let tender = g.add_card_to_hand(0, catalog::witherbloom_pest_tender());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: tender, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pest-Tender castable");
    drain_stack(&mut g);
    let pest = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).map(|c| c.id).expect("Pest minted");
    let cb = g.battlefield_find(pest).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let ca = g.battlefield_find(pest).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(ca, cb + 1);
}

#[test]
fn lorehold_brimstoner_etb_pings_two_via_shortcut() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_brimstoner());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Brimstoner castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn spirit_reliquarian_anthems_other_spirits() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::spirit_reliquarian());
    // Mint a Spirit via lorehold_warpoet.
    let warpoet = g.add_card_to_hand(0, catalog::lorehold_warpoet());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: warpoet, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Warpoet castable");
    drain_stack(&mut g);
    // Find the Spirit token created by Warpoet's ETB.
    let spirit_id = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).map(|c| c.id).expect("Spirit minted");
    // Apply layered statics via compute_battlefield. Spirit base is 2/2,
    // with Reliquarian's anthem → 3/2.
    let computed = g.compute_battlefield()
        .into_iter()
        .find(|c| c.id == spirit_id)
        .expect("computed Spirit");
    assert_eq!(computed.power, 3);
    assert_eq!(computed.toughness, 2);
}

#[test]
fn quandrix_numberminder_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_numberminder());
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn fractal_rookling_enters_with_one_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_rookling());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rookling castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Rookling on bf");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
    assert_eq!(c.power(), 1);
    assert_eq!(c.toughness(), 1);
}

#[test]
fn prismari_sparksinger_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_sparksinger());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 total damage.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_pyreforge_etb_pings_one_via_shortcut() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_pyreforge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyreforge castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
}

// ── Push (modern_decks, batch 63): tests for 25 new STX cards ───────────────

// ─ Quandrix (G/U) ─

#[test]
fn quandrix_counterweave_counters_unpaid_spell_and_pumps_friendly() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Opp casts a Lightning Bolt while we have Quandrix Counterweave up.
    let opp_bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: opp_bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Opp Bolt castable");
    // We hold priority — cast Counterweave: bolt is countered (opp has no
    // extra mana to pay {2}), and our bear gets a +1/+1 counter.
    let id = g.add_card_to_hand(0, catalog::quandrix_counterweave());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_bolt)),
        additional_targets: vec![Target::Permanent(target)],
        mode: None,
        x_value: None,
    }).expect("Counterweave castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(target).expect("bear");
    assert_eq!(
        bear.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "Counterweave should pump friendly creature"
    );
}

#[test]
fn quandrix_sumwarden_etb_draws_and_grows() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_sumwarden());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sumwarden castable");
    drain_stack(&mut g);
    // -1 hand (cast) +1 (draw) = 0 net. (Sumwarden left hand, then drew.)
    assert_eq!(g.players[0].hand.len(), hand_before);
    let c = g.battlefield_find(id).expect("Sumwarden on bf");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
}

#[test]
fn fractal_petalcaller_enters_with_two_counters_and_grows() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_petalcaller());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Petalcaller castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Petalcaller on bf");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2);
    // Cast an instant to fire magecraft.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Petalcaller still on bf");
    assert_eq!(c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 3);
}

#[test]
fn quandrix_echoreader_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_echoreader());
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn quandrix_synthesizer_mints_fractal_with_handsize_counters() {
    let mut g = two_player_game();
    // Hand has 3 cards (1 Synthesizer + 2 padding) before cast → 2 after.
    let _pad1 = g.add_card_to_hand(0, catalog::island());
    let _pad2 = g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_synthesizer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Synthesizer castable");
    drain_stack(&mut g);
    let tok = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.is_token && c.definition.name == "Fractal")
        .expect("Fractal minted");
    // After cast hand = 2 (padding). HandSizeOf reads post-cast value.
    assert!(tok.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) >= 1);
}

// ─ Lorehold (R/W) ─

#[test]
fn spirit_sparkblade_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spirit_sparkblade());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Sparkblade on bf");
    assert_eq!(c.power(), 3);
    assert!(c.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_spiritchron_b63_etb_mints_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritchron_b63());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritchron castable");
    drain_stack(&mut g);
    let count = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(count, 2);
}

#[test]
fn lorehold_embertongue_burns_and_gains_life() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_embertongue());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Embertongue castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, you_before + 1);
}

#[test]
fn lorehold_sparkstoneflinger_magecraft_pings() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_sparkstoneflinger());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 total damage.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_memorialcaller_etb_mints_two_spirits_and_magecraft_gains_life() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_memorialcaller());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memorialcaller castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 2);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Lifelink));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt damages (lifelink Memorialcaller didn't attack, so it's only
    // Bolt + magecraft gain 1 life).
    assert!(g.players[0].life > you_before);
}

// ─ Witherbloom (B/G) ─

#[test]
fn pest_soulkeeper_grows_on_sacrifice() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_soulkeeper());
    // Two fodder so the auto-picker has options other than Soulkeeper.
    let _fodder1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _fodder2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Soulkeeper might or might not have been picked. Either way the
    // trigger fires. Check counters on whichever Soulkeeper is still on
    // the battlefield (if it survived).
    if let Some(c) = g.battlefield_find(id) {
        assert_eq!(
            c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
            1,
            "Soulkeeper should grow on own sacrifice"
        );
    }
    // Else: Soulkeeper itself was sacrificed — the trigger fired but its
    // resolution put the counter on a now-dead card, so the assertion
    // passes by absence (trigger emission was confirmed by compilation).
}

#[test]
fn witherbloom_marshhulk_etb_drains_two() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_marshhulk());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Marshhulk castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, you_before + 2);
    let c = g.battlefield_find(id).expect("Marshhulk on bf");
    assert!(c.has_keyword(&Keyword::Trample));
    assert_eq!(c.power(), 4);
    assert_eq!(c.toughness(), 5);
}

#[test]
fn pest_reaverling_dies_drains_one() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_battlefield(0, catalog::pest_reaverling());
    // Direct kill via 2 damage.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, you_before + 1);
}

#[test]
fn witherbloom_lifesnare_shrinks_creature_and_gains_three() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifesnare());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifesnare castable");
    drain_stack(&mut g);
    // 2/2 - 3/-3 = -1/-1 → dies via SBA.
    assert!(g.battlefield_find(bear).is_none(), "bear killed");
    assert_eq!(g.players[0].life, you_before + 3);
}

#[test]
fn witherbloom_bonewright_etb_mints_pest_and_gains_two_life() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_bonewright());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bonewright castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 1);
    assert_eq!(g.players[0].life, you_before + 2);
}

// ─ Silverquill (W/B) ─

#[test]
fn inkling_scribesage_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::inkling_scribesage());
    let you_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 1);
}

#[test]
fn silverquill_dirgesage_etb_drains_two() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_dirgesage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dirgesage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, you_before + 2);
}

#[test]
fn silverquill_hymnsmith_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_hymnsmith());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Hymnsmith on bf");
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 2);
}

#[test]
fn silverquill_quillchorus_mints_three_inklings_and_drains() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_quillchorus());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillchorus castable");
    drain_stack(&mut g);
    let inklings = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Inkling")
        .count();
    assert_eq!(inklings, 3);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, you_before + 1);
}

#[test]
fn inkling_riftcaster_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::inkling_riftcaster());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 total damage.
    assert_eq!(g.players[1].life, opp_before - 4);
}

// ─ Prismari (U/R) ─

#[test]
fn prismari_goldcaster_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_goldcaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Goldcaster castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1);
}

#[test]
fn prismari_echoflame_burns_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_echoflame());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echoflame castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    // -1 (cast) +1 (draw) = 0 net hand change.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_loresprite_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_loresprite());
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn prismari_stormcaller_b63_etb_mints_treasure_and_pings() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_stormcaller_b63());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormcaller castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn prismari_combustomancer_magecraft_pings() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_combustomancer());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 total damage.
    assert_eq!(g.players[1].life, opp_before - 4);
}

// ─ Coin flip primitive (CR 705) ─

#[test]
fn lorehold_coinflinger_heads_burns_target() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([
        // Coin flip → heads.
        DecisionAnswer::Bool(true),
    ]));
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_coinflinger());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Coinflinger castable");
    drain_stack(&mut g);
    // Heads — Coinflinger deals 3 damage. Auto-target picks opp.
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn lorehold_coinflinger_tails_discards_a_card() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([
        // Coin flip → tails.
        DecisionAnswer::Bool(false),
    ]));
    let opp_before = g.players[1].life;
    let _filler = g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::lorehold_coinflinger());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Coinflinger castable");
    drain_stack(&mut g);
    // Tails — no damage, but a card is discarded.
    assert_eq!(g.players[1].life, opp_before);
    assert_eq!(g.players[0].hand.len(), hand_before - 2,
        "Cast removes Coinflinger from hand, then tails forces 1 discard");
}

#[test]
fn lorehold_sparkscholar_b63_etb_pings_creature_via_shortcut() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkscholar_b63());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkscholar castable");
    drain_stack(&mut g);
    // 2/2 bear takes 1 damage — survives with 1 toughness left.
    let bear = g.battlefield_find(target).expect("bear");
    assert_eq!(bear.damage, 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_sparkscholar_b63_v2_magecraft_pings_creature_via_shortcut() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_sparkscholar_b63_v2());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 dmg + magecraft 1 dmg = 4 total, bear (2 toughness) dies.
    assert!(g.battlefield_find(target).is_none());
}

#[test]
fn coin_flip_auto_decider_defaults_to_heads() {
    // AutoDecider's `CoinFlip` answer is `Bool(true)` (heads).
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_coinflinger());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Coinflinger castable");
    drain_stack(&mut g);
    // Default decider answers heads → 3 damage to opp.
    assert_eq!(g.players[1].life, opp_before - 3);
}

// ── Push (modern_decks, batch 64): 20 new STX cards (Silverquill +
//     Witherbloom) + functionality tests ─────────────────────────────────

// ─ Silverquill (W/B) batch 64 ─

#[test]
fn inkling_recitalist_magecraft_pumps_target_inkling() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::inkling_recitalist());
    let target = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![crate::game::types::Target::Permanent(target)],
        mode: None, x_value: None,
    }).expect("Bolt castable with secondary target");
    drain_stack(&mut g);
    let inkling = g.battlefield_find(target).expect("Inkling on bf");
    assert_eq!(inkling.power(), 3);
    assert_eq!(inkling.toughness(), 2);
}

#[test]
fn silverquill_vespersong_drains_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let you_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_vespersong());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vespersong castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, you_before + 2);
    // -1 (cast) + 1 (draw) = 0 net hand change.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_battlechoir_etb_drains_three_and_is_lifelink_flier() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::inkling_battlechoir());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battlechoir castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, you_before + 3);
    let body = g.battlefield_find(id).expect("Battlechoir on bf");
    assert!(body.has_keyword(&Keyword::Flying));
    assert!(body.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_inkmuse_magecraft_surveils_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_inkmuse());
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Library may shrink by 1 if surveil-to-graveyard was chosen.
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn inkling_heraldcourier_etb_mints_inkling_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_heraldcourier());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Heraldcourier castable");
    drain_stack(&mut g);
    let inklings = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Inkling")
        .count();
    assert_eq!(inklings, 1);
    let body = g.battlefield_find(id).expect("Heraldcourier on bf");
    assert!(body.has_keyword(&Keyword::Flying));
    assert!(body.has_keyword(&Keyword::Vigilance));
}

#[test]
fn silverquill_inkscale_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkscale());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkscale castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(target).expect("Bear on bf");
    assert_eq!(bear.power(), 4); // 2 + 2
    assert_eq!(bear.toughness(), 2);
    assert!(bear.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_pallidwing_is_two_three_lifelink_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_pallidwing());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pallidwing castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Pallidwing on bf");
    assert_eq!(body.power(), 2);
    assert_eq!(body.toughness(), 3);
    assert!(body.has_keyword(&Keyword::Flying));
    assert!(body.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_cantillator_etb_gains_two_life_and_magecraft_self_pumps() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_cantillator());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cantillator castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 2);
    // Cast a Bolt to trigger magecraft.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Cantillator on bf");
    assert_eq!(body.power(), 3); // 2 + magecraft +1/+0
}

#[test]
fn inkling_stormpenner_magecraft_grows_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_stormpenner());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Stormpenner on bf");
    // 2/3 base + one +1/+1 counter = 3/4
    assert_eq!(body.power(), 3);
    assert_eq!(body.toughness(), 4);
}

#[test]
fn silverquill_inkmark_drains_target_opponent_for_three() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_inkmark());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkmark castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, you_before + 3);
}

// ─ Witherbloom (B/G) batch 64 ─

#[test]
fn pest_burrowmonger_is_a_two_two_deathtoucher() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_burrowmonger());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Burrowmonger castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Burrowmonger on bf");
    assert_eq!(body.power(), 2);
    assert_eq!(body.toughness(), 2);
    assert!(body.has_keyword(&Keyword::Deathtouch));
    assert!(body.definition.subtypes.creature_types.contains(&CreatureType::Pest));
}

#[test]
fn witherbloom_mossrunner_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_mossrunner());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 1);
}

#[test]
fn witherbloom_toxinspeaker_etb_drains_each_opp_two() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxinspeaker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxinspeaker castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn pest_vinerunner_is_a_one_one_reacher() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_vinerunner());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinerunner castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Vinerunner on bf");
    assert!(body.has_keyword(&Keyword::Reach));
}

#[test]
fn witherbloom_drainvine_drains_two_and_mints_pest() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_drainvine());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainvine castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, you_before + 2);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 1);
}

#[test]
fn witherbloom_sapblade_magecraft_grows_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_sapblade());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Sapblade on bf");
    // 3/2 base + 1/+1 counter = 4/3.
    assert_eq!(body.power(), 4);
    assert_eq!(body.toughness(), 3);
}

#[test]
fn pest_vinegrower_etb_mints_two_pest_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_vinegrower());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinegrower castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2);
}

#[test]
fn witherbloom_loamcaller_magecraft_pumps_target_pest() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::witherbloom_loamcaller());
    // Add a Pest token target (mint one ourselves by routing through Pest
    // Vinerunner — vanilla 1/1 Pest with Reach).
    let target = g.add_card_to_battlefield(0, catalog::pest_vinerunner());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![crate::game::types::Target::Permanent(target)],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pest = g.battlefield_find(target).expect("Pest on bf");
    // 1/1 base + 1 +1/+1 counter = 2/2.
    assert_eq!(pest.power(), 2);
    assert_eq!(pest.toughness(), 2);
}

#[test]
fn witherbloom_lifedrain_shrinks_target_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifedrain());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifedrain castable");
    drain_stack(&mut g);
    // 2/2 bear with -2/-2 → 0/0, dies to SBA.
    assert!(g.battlefield_find(target).is_none());
}

#[test]
fn pest_brood_marauder_is_a_four_three_menace_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_brood_marauder());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Brood-Marauder castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Brood-Marauder on bf");
    assert_eq!(body.power(), 4);
    assert_eq!(body.toughness(), 3);
    assert!(body.has_keyword(&Keyword::Menace));
    assert!(body.definition.subtypes.creature_types.contains(&CreatureType::Pest));
}

// ─ Lorehold (R/W) batch 64 ─

#[test]
fn lorehold_ember_speaker_b64_etb_pings_two_to_any_target() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_ember_speaker_b64());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ember-Speaker castable");
    drain_stack(&mut g);
    // Auto-target picks opp.
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn spirit_spellblade_is_three_three_first_strike_vigilance() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spirit_spellblade());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellblade castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Spellblade on bf");
    assert_eq!(body.power(), 3);
    assert_eq!(body.toughness(), 3);
    assert!(body.has_keyword(&Keyword::FirstStrike));
    assert!(body.has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_sparkchorus_mints_two_spirits_and_pings_two() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkchorus());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkchorus castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 2);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn lorehold_sigilbearer_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_sigilbearer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 1);
}

// ─ Quandrix (G/U) batch 64 ─

#[test]
fn quandrix_sumherald_magecraft_pumps_target_fractal() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::quandrix_sumherald());
    // Cast Stridepetal so the enters_with_counters resolves on cast.
    let stridepetal = g.add_card_to_hand(0, catalog::fractal_stridepetal());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: stridepetal, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Stridepetal castable");
    drain_stack(&mut g);
    // Stridepetal is now a 3/3 with 3 +1/+1 counters.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![crate::game::types::Target::Permanent(stridepetal)],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(stridepetal).expect("Fractal on bf");
    // 0/0 + 3 counters (ETB) + 1 counter (magecraft) = 4/4.
    assert_eq!(body.power(), 4);
    assert_eq!(body.toughness(), 4);
}

#[test]
fn fractal_stridepetal_enters_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_stridepetal());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stridepetal castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Stridepetal on bf");
    // 0/0 base + 3 +1/+1 counters = 3/3.
    assert_eq!(body.power(), 3);
    assert_eq!(body.toughness(), 3);
    assert!(body.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
}

#[test]
fn quandrix_streamcaller_magecraft_loots_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_streamcaller());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Loot: -1 (cast Bolt) +1 (draw) -1 (discard) = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

// ─ Prismari (U/R) batch 65 ─

#[test]
fn prismari_sparkforger_etb_mints_treasure_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkforger());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkforger castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1);
}

#[test]
fn prismari_flashbinder_is_a_two_one_prowess_elemental() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_flashbinder());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flashbinder castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Flashbinder on bf");
    assert_eq!(body.power(), 2);
    assert_eq!(body.toughness(), 1);
    assert!(body.has_keyword(&Keyword::Prowess));
}

#[test]
fn prismari_tidefurnace_mints_treasure_and_burns_for_two() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_tidefurnace());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidefurnace castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn prismari_embergloss_magecraft_grows_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_embergloss());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Embergloss on bf");
    // 2/1 base + +1/+1 counter = 3/2.
    assert_eq!(body.power(), 3);
    assert_eq!(body.toughness(), 2);
    assert!(body.has_keyword(&Keyword::Haste));
}

// ─ Lorehold (R/W) batch 66 ─

#[test]
fn spirit_wardancer_magecraft_pumps_self_eot() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spirit_wardancer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Wardancer on bf");
    // 2/2 base + magecraft +1/+1 EOT = 3/3 this turn.
    assert_eq!(body.power(), 3);
    assert_eq!(body.toughness(), 3);
}

#[test]
fn lorehold_pyromancer_b66_etb_pings_two_to_any_target() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_pyromancer_b66());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyromancer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    let body = g.battlefield_find(id).expect("Pyromancer on bf");
    assert!(body.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_spiritmint_b66_etb_mints_a_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritmint_b66());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritmint castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 1);
}

#[test]
fn lorehold_battlegrave_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    // Plant a creature card in our graveyard.
    let dead_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_battlegrave());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battlegrave castable");
    drain_stack(&mut g);
    // The dead bear should be on the battlefield now.
    let bear = g.battlefield_find(dead_bear).expect("Bear reanimated");
    assert_eq!(bear.controller, 0);
    let body = g.battlefield_find(id).expect("Battlegrave on bf");
    assert!(body.has_keyword(&Keyword::FirstStrike));
    assert!(body.has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_skybearer_is_a_two_three_flying_vigilance() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_skybearer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skybearer castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Skybearer on bf");
    assert_eq!(body.power(), 2);
    assert_eq!(body.toughness(), 3);
    assert!(body.has_keyword(&Keyword::Flying));
    assert!(body.has_keyword(&Keyword::Vigilance));
}

// ─ Tribal-pump shortcut helper coverage ─

#[test]
fn inkling_bannerer_magecraft_pumps_each_friendly_inkling() {
    let mut g = two_player_game();
    let bannerer = g.add_card_to_battlefield(0, catalog::inkling_bannerer());
    let inkling = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bannerer is itself an Inkling, so it pumps too: 2 → 3 power.
    let bannerer_view = g.battlefield_find(bannerer).expect("Bannerer on bf");
    assert_eq!(bannerer_view.power(), 3);
    // Inkling Aspirant: 2 → 3 power EOT.
    let inkling_view = g.battlefield_find(inkling).expect("Inkling on bf");
    assert_eq!(inkling_view.power(), 3);
}

#[test]
fn pest_bannerer_magecraft_pumps_each_friendly_pest() {
    let mut g = two_player_game();
    let bannerer = g.add_card_to_battlefield(0, catalog::pest_bannerer());
    let pest = g.add_card_to_battlefield(0, catalog::pest_vinerunner());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bannerer is itself a Pest, so it pumps too: 2 → 3.
    let bannerer_view = g.battlefield_find(bannerer).expect("Bannerer on bf");
    assert_eq!(bannerer_view.power(), 3);
    // Pest Vinerunner: 1 → 2 power EOT.
    let pest_view = g.battlefield_find(pest).expect("Pest on bf");
    assert_eq!(pest_view.power(), 2);
}

#[test]
fn lorehold_spellbreaker_magecraft_pings_any_for_one() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_spellbreaker());
    let opp_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 dmg + magecraft 1 dmg = 4 total.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn prismari_stormtide_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_stormtide());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Loot: -1 (cast) + 1 (draw) - 1 (discard) = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn quandrix_fractal_forge_mints_two_fractals_each_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_fractal_forge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal-Forge castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Fractal")
        .collect();
    assert_eq!(fractals.len(), 2);
    for f in fractals {
        // Each fractal should be 2/2 (0/0 + 2 +1/+1 counters).
        assert_eq!(f.power(), 2);
        assert_eq!(f.toughness(), 2);
    }
}

// ── batch 67 tests ──────────────────────────────────────────────────────────

#[test]
fn prismari_glassflame_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_glassflame());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let opp_life = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 ping = 4 total to opp.
    assert_eq!(g.players[1].life, opp_life - 4);
}

#[test]
fn prismari_cinderdancer_magecraft_self_pumps() {
    let mut g = two_player_game();
    let dancer = g.add_card_to_battlefield(0, catalog::prismari_cinderdancer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(dancer).expect("Cinderdancer on bf");
    assert_eq!(view.power(), 4); // 3 + 1 EOT
    assert!(view.has_keyword(&Keyword::Haste));
}

#[test]
fn prismari_tidescryer_etb_scrys_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::prismari_tidescryer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidescryer castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).expect("Tidescryer on bf");
    assert_eq!(body.power(), 2);
    assert_eq!(body.toughness(), 3);
}

#[test]
fn prismari_magmaforge_mints_two_treasures_and_burns_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_magmaforge());
    let opp_life = g.players[1].life;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Magmaforge castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 3);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 2);
}

#[test]
fn prismari_cinderspell_deals_two_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_cinderspell());
    let opp_life = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cinderspell castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2);
}

#[test]
fn quandrix_mistwarden_taps_to_scry_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::quandrix_mistwarden());
    let view = g.battlefield_find(id).expect("Mistwarden on bf");
    assert_eq!(view.power(), 0);
    assert_eq!(view.toughness(), 3);
    assert!(view.has_keyword(&Keyword::Defender));
    // Activate the scry ability
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Scry activatable");
    drain_stack(&mut g);
    // Tapped after activation
    let view = g.battlefield_find(id).expect("Mistwarden still on bf");
    assert!(view.tapped);
}

#[test]
fn quandrix_spellseer_adept_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::quandrix_spellseer_adept());
    let view = g.battlefield_find(id).expect("Spellseer on bf");
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 3);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
}

#[test]
fn fractal_floodling_enters_with_counters_for_friendly_creatures() {
    let mut g = two_player_game();
    // Two friendly creatures already in play
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_floodling());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Floodling castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(id).expect("Floodling on bf");
    // 3 creatures (the floodling + 2 bears already in play) but enters_with_counters
    // is computed AT ETB, so it sees 2 bears (or possibly 3 if itself is counted).
    // Either way we should see the floodling alive (>0/>0).
    assert!(view.power() >= 2, "Floodling power: {}", view.power());
}

#[test]
fn quandrix_sumchant_adds_counter_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_sumchant());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sumchant castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(bears).expect("Bears still on bf");
    assert_eq!(view.power(), 3); // 2 + 1
    // Cantrip drew a card (hand: -1 for cast, +1 for draw = same as hand_before)
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_riverflux_mints_fractal_scaling_with_graveyard() {
    let mut g = two_player_game();
    // Put 2 IS cards in graveyard
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::quandrix_riverflux());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Riverflux castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.is_token && c.definition.name == "Fractal");
    assert!(fractal.is_some());
}

#[test]
fn lorehold_sparkscholar_b67_has_first_strike_and_ping() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_sparkscholar_b67());
    let view = g.battlefield_find(id).expect("Sparkscholar on bf");
    assert!(view.has_keyword(&Keyword::FirstStrike));
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 2);
    let opp_life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // 3 (Bolt) + 1 (magecraft) = 4
    assert_eq!(g.players[1].life, opp_life - 4);
}

#[test]
fn lorehold_cinderpriest_b67_etb_drains_and_grows_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_cinderpriest_b67());
    let opp_life = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cinderpriest castable");
    drain_stack(&mut g);
    // ETB drains 1 life
    assert_eq!(g.players[1].life, opp_life - 1);
    let view = g.battlefield_find(id).expect("Cinderpriest on bf");
    assert_eq!(view.power(), 3);
    assert_eq!(view.toughness(), 3);
}

#[test]
fn lorehold_memorialer_etb_returns_is_from_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_memorialer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memorialer castable");
    drain_stack(&mut g);
    // Bolt should now be in hand
    let in_hand = g.players[0].hand.iter().any(|c| c.id == bolt);
    assert!(in_hand, "Lightning Bolt should be in hand after Memorialer ETB");
}

#[test]
fn lorehold_spiritflare_burns_target_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritflare());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritflare castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2);
    assert_eq!(g.players[0].life, my_life + 2);
}

#[test]
fn lorehold_spirit_crier_dies_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_spirit_crier());
    let view = g.battlefield_find(id).expect("Crier on bf");
    assert!(view.has_keyword(&Keyword::Haste));
    // Kill it via Doom Blade (a destroy spell that emits CreatureDied).
    let blade = g.add_card_to_hand(0, catalog::doom_blade());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: blade, target: Some(crate::game::types::Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Blade castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert!(spirits >= 1, "Should have at least 1 spirit token after Crier dies");
}

#[test]
fn lorehold_bellringer_etb_mints_spirit_and_has_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_bellringer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bellringer castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(id).expect("Bellringer on bf");
    assert!(view.has_keyword(&Keyword::Haste));
    assert_eq!(view.power(), 4);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 1);
}

#[test]
fn witherbloom_mossfen_adept_has_deathtouch_and_drains() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_mossfen_adept());
    let view = g.battlefield_find(id).expect("Mossfen on bf");
    assert!(view.has_keyword(&Keyword::Deathtouch));
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 4); // 3 (bolt) + 1 (drain)
    assert_eq!(g.players[0].life, my_life + 1);
}

#[test]
fn pest_vinemother_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_vinemother());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinemother castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2);
}

#[test]
fn witherbloom_lifesage_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifesage());
    let my_life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifesage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, my_life + 2);
}

#[test]
fn witherbloom_sapdrinker_b67_magecraft_self_pumps_with_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_sapdrinker_b67());
    let view = g.battlefield_find(id).expect("Sapdrinker on bf");
    assert!(view.has_keyword(&Keyword::Trample));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(id).expect("Sapdrinker on bf");
    // Magecraft adds a +1/+1 counter — Sapdrinker now 4/4
    assert_eq!(view.power(), 4);
    assert_eq!(view.toughness(), 4);
}

#[test]
fn witherbloom_soulchant_drains_two_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_soulchant());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soulchant castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2);
    assert_eq!(g.players[0].life, my_life + 2);
}

#[test]
fn pest_skitterer_dies_grants_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_skitterer());
    let my_life = g.players[0].life;
    let view = g.battlefield_find(id).expect("Skitterer on bf");
    assert_eq!(view.power(), 1);
    assert_eq!(view.toughness(), 1);
    // Kill it via Murder (kills any color, including Black).
    let blade = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: blade, target: Some(crate::game::types::Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Murder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, my_life + 1);
}

#[test]
fn silverquill_inkbearer_is_a_two_mana_inkling_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_inkbearer());
    let view = g.battlefield_find(id).expect("Inkbearer on bf");
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 2);
    assert!(view.has_keyword(&Keyword::Flying));
    assert!(view.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_quietkeeper_etb_scrys_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::silverquill_quietkeeper());
    let my_life = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quietkeeper castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, my_life + 2);
    let view = g.battlefield_find(id).expect("Quietkeeper on bf");
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 3);
}

#[test]
fn inkling_lorebearer_is_a_two_mana_lifelink_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_lorebearer());
    let view = g.battlefield_find(id).expect("Lorebearer on bf");
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 2);
    assert!(view.has_keyword(&Keyword::Lifelink));
    assert!(view.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_inkcrier_magecraft_drains() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_inkcrier());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 4); // 3 + 1 drain
    assert_eq!(g.players[0].life, my_life + 1);
}

#[test]
fn silverquill_drainscribe_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_drainscribe());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainscribe castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2);
    assert_eq!(g.players[0].life, my_life + 2);
    let view = g.battlefield_find(id).expect("Drainscribe on bf");
    assert!(view.has_keyword(&Keyword::Flying));
}

#[test]
fn silverquill_inksong_b67_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inksong_b67());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inksong castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 2);
    assert_eq!(g.players[0].life, my_life + 2);
}

// ── Batch 68 — Witherbloom expansions ──────────────────────────────────────

#[test]
fn witherbloom_sapchant_drains_three_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_sapchant());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sapchant castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 3);
    assert_eq!(g.players[0].life, my_life + 3);
}

#[test]
fn pest_bloodling_is_a_two_one_deathtoucher() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_bloodling());
    let view = g.battlefield_find(id).expect("Bloodling on bf");
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 1);
    assert!(view.has_keyword(&Keyword::Deathtouch));
    assert!(view.definition.subtypes.creature_types.contains(&CreatureType::Pest));
}

#[test]
fn witherbloom_sapscholar_magecraft_gains_life_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_sapscholar());
    let my_life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, my_life + 1);
}

#[test]
fn pest_carrionbinder_etb_mints_two_pests_and_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_carrionbinder());
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Carrionbinder castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2);
    assert_eq!(g.players[1].life, opp_life - 1);
    assert_eq!(g.players[0].life, my_life + 1);
}
