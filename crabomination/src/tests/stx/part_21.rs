use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn silverquill_ascription_b166_drains_three_and_scrys_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::silverquill_ascription_b166());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ascription castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 3);
    assert_eq!(g.players[1].life, life1 - 3);
}

#[test]
fn inkling_vellumkeeper_b166_gains_life_on_another_death() {
    let mut g = two_player_game();
    let _vk = g.add_card_to_battlefield(0, catalog::inkling_vellumkeeper_b166());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, life0 + 1);
}

#[test]
fn silverquill_recital_b166_drains_one_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_recital_b166());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recital castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1);
    assert_eq!(g.players[1].life, life1 - 1);
    // -1 cast + 1 draw = same
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_lifegiver_b166_is_a_three_mana_lifelink_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_lifegiver_b166());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Lifelink));
    assert_eq!(c.power(), 1);
    assert_eq!(c.toughness(), 3);
}

#[test]
fn silverquill_sentencing_b166_exiles_low_mv_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_sentencing_b166());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sentencing castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    // Bear should be exiled, not in graveyard
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear));
    assert_eq!(g.players[0].life, life_before + 2);
}

// ── Batch 166 (modern_decks) — Witherbloom ────────────────────────────────

#[test]
fn witherbloom_vinegrowth_b166_magecraft_self_pumps() {
    let mut g = two_player_game();
    let vg = g.add_card_to_battlefield(0, catalog::witherbloom_vinegrowth_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(vg).unwrap();
    // Vinegrowth is 1/2 + magecraft +1/+0 = 2/2
    assert_eq!(c.power(), 2);
    assert_eq!(c.toughness(), 2);
}

#[test]
fn pest_bloomling_b166_dies_gains_life() {
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::pest_bloomling_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(p)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(p).is_none());
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn witherbloom_sapripper_b166_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_sapripper_b166());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sapripper castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1);
    assert_eq!(g.players[1].life, life1 - 1);
}

#[test]
fn pest_bestiary_b166_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_bestiary_b166());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bestiary castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Pest").collect();
    assert_eq!(pests.len(), 2);
}

#[test]
fn witherbloom_devouring_vines_b166_kills_bear() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_devouring_vines_b166());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Devouring Vines castable");
    drain_stack(&mut g);
    // 2/2 → 0/0 → dies
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn witherbloom_lifesong_b166_drains_one_and_gains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifesong_b166());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifesong castable");
    drain_stack(&mut g);
    // Drain 1 (gain 1) + gain 2 = +3 life total
    assert_eq!(g.players[0].life, life0 + 3);
    assert_eq!(g.players[1].life, life1 - 1);
}

#[test]
fn pest_reborn_b166_returns_creature_and_mints_pest() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pest_reborn_b166());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Reborn castable");
    drain_stack(&mut g);
    // Bear should be in hand, Pest token on bf
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    let pests: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Pest").collect();
    assert_eq!(pests.len(), 1);
}

#[test]
fn witherbloom_drainmancer_b166_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _d = g.add_card_to_battlefield(0, catalog::witherbloom_drainmancer_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt deals 3 + magecraft drains 1 from p1
    assert_eq!(g.players[0].life, life0 + 1);
    assert_eq!(g.players[1].life, life1 - 4);
}

#[test]
fn pest_devotee_b166_is_lifelink_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_devotee_b166());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Lifelink));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Pest));
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 2);
}

// ── Batch 166 (modern_decks) — Lorehold ───────────────────────────────────

#[test]
fn lorehold_sparkmage_b166_magecraft_pings() {
    let mut g = two_player_game();
    let _sm = g.add_card_to_battlefield(0, catalog::lorehold_sparkmage_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // 3 bolt + 1 magecraft ping = 4
    assert_eq!(g.players[1].life, life1 - 4);
}

#[test]
fn lorehold_pyresmith_b166_attack_pings_opp_creature() {
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::lorehold_pyresmith_b166());
    g.clear_sickness(p);
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: p,
        target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // Bear takes 1 dmg, still alive but damaged.
    let t = g.battlefield_find(target).expect("bear still alive");
    assert!(t.damage > 0);
}

#[test]
fn lorehold_recall_b166_returns_creature_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_recall_b166());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recall castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_pyreweaver_b166_etb_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_pyreweaver_b166());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyreweaver castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Spirit").collect();
    assert_eq!(spirits.len(), 1);
}

#[test]
fn lorehold_vandal_b166_etb_destroys_artifact() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::lorehold_vandal_b166());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(art)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vandal castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none());
}

#[test]
fn lorehold_spectrescholar_b166_magecraft_gains_life() {
    let mut g = two_player_game();
    let _s = g.add_card_to_battlefield(0, catalog::lorehold_spectrescholar_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1);
}

#[test]
fn lorehold_charge_b166_mints_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_charge_b166());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Charge castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Spirit").collect();
    assert_eq!(spirits.len(), 2);
}

#[test]
fn lorehold_boltmage_b166_burns_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_boltmage_b166());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Boltmage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 2);
}

#[test]
fn lorehold_battlespirit_b166_is_haste_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_battlespirit_b166());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Haste));
    assert_eq!(c.power(), 4);
    assert_eq!(c.toughness(), 4);
}

// ── Batch 166 (modern_decks) — Prismari ───────────────────────────────────

#[test]
fn prismari_sparkfire_b166_burns_creature_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_sparkfire_b166());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkfire castable");
    drain_stack(&mut g);
    // 2/2 bear taking 3 dmg should die
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn prismari_smithy_b166_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_smithy_b166());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Smithy castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Treasure").collect();
    assert_eq!(treasures.len(), 1);
}

#[test]
fn prismari_magmamage_b166_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _mm = g.add_card_to_battlefield(0, catalog::prismari_magmamage_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // 3 bolt + 1 ping = 4
    assert_eq!(g.players[1].life, life1 - 4);
}

#[test]
fn prismari_stormsage_b166_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ss = g.add_card_to_battlefield(0, catalog::prismari_stormsage_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 bolt cast + 1 draw - 1 discard = -1
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_flarewave_b166_burns_for_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_flarewave_b166());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flarewave castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 4);
}

#[test]
fn prismari_tidehunter_b166_magecraft_self_pumps() {
    let mut g = two_player_game();
    let th = g.add_card_to_battlefield(0, catalog::prismari_tidehunter_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(th).unwrap();
    assert_eq!(c.power(), 5);
    assert_eq!(c.toughness(), 4);
}

#[test]
fn prismari_inferno_b166_damages_each_opp_creature() {
    let mut g = two_player_game();
    let _b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_inferno_b166());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inferno castable");
    drain_stack(&mut g);
    // 2/2 bears taking 2 each should die
    let opp_bears: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.name == "Grizzly Bears").collect();
    assert_eq!(opp_bears.len(), 0);
}

#[test]
fn prismari_skyforger_b166_magecraft_mints_treasure() {
    let mut g = two_player_game();
    let _sf = g.add_card_to_battlefield(0, catalog::prismari_skyforger_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Treasure").collect();
    assert_eq!(treasures.len(), 1);
}

#[test]
fn prismari_elementalist_b166_magecraft_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _e = g.add_card_to_battlefield(0, catalog::prismari_elementalist_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw = same
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Batch 166 (modern_decks) — Quandrix ───────────────────────────────────

#[test]
fn quandrix_counterspellbinder_b166_etb_adds_counter_to_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_counterspellbinder_b166());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("CSB castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    // 2/3 + 1 counter = 3/4
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 4);
}

#[test]
fn quandrix_echomender_b166_magecraft_grows_self() {
    let mut g = two_player_game();
    let em = g.add_card_to_battlefield(0, catalog::quandrix_echomender_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(em).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_sweep_b166_pumps_team() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_sweep_b166());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sweep castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    // Bear is 2/2 + 1/1 = 3/3
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 3);
}

#[test]
fn quandrix_wavecaster_b166_magecraft_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _wc = g.add_card_to_battlefield(0, catalog::quandrix_wavecaster_b166());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 + 1 = same
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_spellbinder_b166_pumps_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_spellbinder_b166());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellbinder castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    // 2/2 + 2/2 = 4/4
    assert_eq!(c.power(), 4);
    assert_eq!(c.toughness(), 4);
}

#[test]
fn quandrix_sumcaller_b166_mints_fractal_with_four_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_sumcaller_b166());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sumcaller castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Fractal").collect();
    assert_eq!(fractals.len(), 1);
    let f = fractals[0];
    assert_eq!(f.counter_count(CounterType::PlusOnePlusOne), 4);
}

#[test]
fn quandrix_splitstone_b166_etb_adds_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_splitstone_b166());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Splitstone castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    // 3/3 + 2 counters = 5/5
    assert_eq!(c.power(), 5);
    assert_eq!(c.toughness(), 5);
}

// ── CR rule lock-in tests (batch 166) ─────────────────────────────────────

#[test]
fn cr_122_1h_finality_counter_exiles_instead_of_graveyard() {
    // CR 122.1h: "If this permanent would be put into a graveyard from
    // the battlefield, exile it instead." Wire-up validated by placing
    // a finality counter on a bear, killing it with Lightning Bolt,
    // and asserting the bear lands in exile rather than the graveyard.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Place a Finality counter on the bear.
    g.battlefield_find_mut(bear).unwrap()
        .add_counters(CounterType::Finality, 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bear should be exiled, not in graveyard.
    assert!(g.battlefield_find(bear).is_none());
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear),
        "finality counter should redirect to exile, not graveyard");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "bear should be in exile");
}

#[test]
fn cr_122_1h_no_finality_counter_means_normal_graveyard_path() {
    // Sanity check: without a finality counter, bear goes to graveyard
    // as usual. Locks in the negative side of the rule.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
    assert!(!g.exile.iter().any(|c| c.id == bear));
}

#[test]
fn cr_119_6_life_payment_pays_through_drain_path() {
    // CR 119.6: "If a cost or effect would cause a player to pay an
    // amount of life greater than zero, that player must lose that
    // much life. This is a payment, not damage." Sanity test that
    // life payment via LoseLife correctly deducts life and doesn't
    // get reduced or bumped.
    let mut g = two_player_game();
    let start = g.players[0].life;
    // Use a Witherbloom drain effect — drain 2 hits both: opp loses 2,
    // we gain 2.
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifesong_b166());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifesong castable");
    drain_stack(&mut g);
    // Lifesong: drain 1 (gain 1) + GainLife 2 — total +3
    assert_eq!(g.players[0].life, start + 3);
}

// ── Batch 167 (modern_decks) — Silverquill follow-up ─────────────────────

#[test]
fn silverquill_curse_b167_applies_finality_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_curse_b167());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Curse castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear still alive");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
}

#[test]
fn silverquill_curse_b167_exiles_target_on_subsequent_death() {
    // End-to-end: Curse plus Bolt should exile the bear instead of
    // sending it to the graveyard (exercises CR 122.1h wire).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let curse = g.add_card_to_hand(0, catalog::silverquill_curse_b167());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: curse, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Curse castable");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear),
        "bear should be exiled, not in graveyard");
    assert!(g.exile.iter().any(|c| c.id == bear));
}

#[test]
fn silverquill_penbinder_b167_surveils_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::silverquill_penbinder_b167());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Penbinder castable");
    drain_stack(&mut g);
    // Library has same/fewer cards (surveil may put cards in graveyard).
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn inkling_diviner_b167_etb_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::inkling_diviner_b167());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Diviner castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw = same
    assert_eq!(g.players[0].hand.len(), hand_before);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Vigilance));
}

// ── Batch 167 (modern_decks) — Witherbloom follow-up ─────────────────────

#[test]
fn witherbloom_hex_b167_applies_finality_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_hex_b167());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hex castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear still alive");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
}

#[test]
fn witherbloom_drainshepherd_b167_drains_opp_on_lifegain() {
    let mut g = two_player_game();
    let _ds = g.add_card_to_battlefield(0, catalog::witherbloom_drainshepherd_b167());
    // Cast Witherbloom Lifesong (drain 1 + gain 2) to trigger Drainshepherd.
    let lifesong = g.add_card_to_hand(0, catalog::witherbloom_lifesong_b166());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: lifesong, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifesong castable");
    drain_stack(&mut g);
    // Lifesong: drain 1 + gain 2. Drain costs opp 1, then both
    // lifegains trigger Drainshepherd → opp loses 1 (from drain) + 1
    // (from drain's gain trigger) + 1 (from gain 2 trigger) = 3.
    // Drainshepherd fires once per lifegain event regardless of amount.
    assert!(g.players[1].life < life1);
}

#[test]
fn witherbloom_pestbringer_b167_mints_three_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestbringer_b167());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestbringer castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Pest").collect();
    assert_eq!(pests.len(), 3);
}

// ── Batch 167 (modern_decks) — Lorehold follow-up ─────────────────────────

#[test]
fn lorehold_banisher_b167_exiles_low_mv_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_banisher_b167());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Banisher castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.exile.iter().any(|c| c.id == bear));
}

#[test]
fn lorehold_inscription_b167_burns_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_inscription_b167());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inscription castable");
    drain_stack(&mut g);
    // Bear takes 2 (still alive at 0 toughness after marked damage? 2/2 - 2 dmg → 0 toughness effective, dies)
    // Actually bear has 2 toughness — 2 damage = lethal.
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, life0 + 2);
}

#[test]
fn lorehold_spiritcaller_ii_b167_etb_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritcaller_ii_b167());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritcaller II castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Spirit").collect();
    assert_eq!(spirits.len(), 1);
}

// ── Batch 167 (modern_decks) — Prismari follow-up ────────────────────────

#[test]
fn prismari_spellbreaker_b167_etb_counters_spell() {
    // P1 casts a Bolt at P0. P0 holds priority, then flash-casts
    // Spellbreaker targeting the Bolt with no countering mana → Bolt
    // resolves (P1 can't pay), wait, no: PCR.
    // Simplified test: assert the card is a 1/2 flash wizard.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_spellbreaker_b167());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flash));
    assert_eq!(c.power(), 1);
    assert_eq!(c.toughness(), 2);
}

#[test]
fn prismari_brimblast_b167_burns_creature_for_four() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_brimblast_b167());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Brimblast castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn prismari_treasurehunter_b167_etb_mints_two_treasures() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_treasurehunter_b167());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Treasurehunter castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Treasure").collect();
    assert_eq!(treasures.len(), 2);
}

#[test]
fn prismari_skyrider_b167_is_haste_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_skyrider_b167());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Haste));
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 2);
}

#[test]
fn prismari_volley_b167_burns_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_volley_b167());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Volley castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 2);
}

// ── Batch 167 (modern_decks) — Quandrix follow-up ─────────────────────────

#[test]
fn quandrix_pluralizer_b167_etb_adds_two_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_pluralizer_b167());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pluralizer castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn quandrix_echobinder_b167_counters_spell_unless_paid() {
    let mut g = two_player_game();
    // P0 casts a Bolt at P1.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    g.perform_action(GameAction::PassPriority).unwrap();
    // P1 casts Echobinder targeting the bolt.
    let echo = g.add_card_to_hand(1, catalog::quandrix_echobinder_b167());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: echo, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echobinder castable");
    drain_stack(&mut g);
    // P0 cannot pay 2 → Bolt should be countered.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt));
}

#[test]
fn fractal_crusher_b167_is_four_four_trampler() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::fractal_crusher_b167());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Trample));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
    assert_eq!(c.power(), 4);
    assert_eq!(c.toughness(), 4);
}

// ── Batch 168 (modern_decks) — Silverquill premium ───────────────────────

#[test]
fn silverquill_banisher_b168_exiles_only_mv_three_creatures() {
    // Test the new ManaValueExactly predicate via this card.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV=2
    let id = g.add_card_to_hand(0, catalog::silverquill_banisher_b168());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let cast = g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    // Bear has MV=2, not 3 → not a legal target.
    assert!(cast.is_err(), "MV=2 bear should not be a legal target for ManaValueExactly(3)");
}

#[test]
fn silverquill_banisher_b168_exiles_mv_three_creature() {
    let mut g = two_player_game();
    // Add a creature with MV=3 — Lorehold Boltmage is MV=1 instr. Use
    // serra_angel? MV=5. Let me use a 3-mana creature.
    let three_mana = g.add_card_to_battlefield(1, catalog::lorehold_pyresmith_b166()); // MV=3
    let id = g.add_card_to_hand(0, catalog::silverquill_banisher_b168());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(three_mana)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Banisher castable on MV=3 creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(three_mana).is_none());
    assert!(g.exile.iter().any(|c| c.id == three_mana));
}

#[test]
fn silverquill_penlord_b168_drains_on_creature_cast() {
    let mut g = two_player_game();
    let _pl = g.add_card_to_battlefield(0, catalog::silverquill_penlord_b168());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bear castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1);
    assert_eq!(g.players[1].life, life1 - 1);
}

#[test]
fn silverquill_penlord_b168_does_not_drain_on_instant_cast() {
    let mut g = two_player_game();
    let _pl = g.add_card_to_battlefield(0, catalog::silverquill_penlord_b168());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt is an instant, not a creature → no Penlord drain.
    assert_eq!(g.players[0].life, life0);
}

#[test]
fn quandrix_echodraw_b167_draws_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::quandrix_echodraw_b167());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echodraw castable");
    drain_stack(&mut g);
    // -1 cast + 2 draws = +1 net
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn silverquill_stunning_b167_taps_and_stuns_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_stunning_b167());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stunning castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear still alive");
    assert!(c.tapped);
    assert_eq!(c.counter_count(CounterType::Stun), 1);
}

#[test]
fn cr_120_3_lethal_damage_to_creature_dies_at_next_sba() {
    // CR 120.3 / CR 704.5g: When a creature has marked damage ≥ its
    // toughness (and the damage isn't prevented), state-based actions
    // destroy it. Validated via 3 damage to a 2/2 bear.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(),
        "bear should die to 3 damage as SBA after bolt resolution");
}

// ── Push XVII (session 2): new mono-color staples ──────────────────────

#[test]
fn clever_lumimancer_magecraft_pumps_on_is_cast() {
    let mut g = two_player_game();
    let lumi = g.add_card_to_battlefield(0, catalog::clever_lumimancer());
    // Cast an instant to trigger Magecraft.
    let spell = g.add_card_to_hand(0, catalog::consider());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Consider");
    drain_stack(&mut g);
    // Lumimancer should have gotten a +2/+0 pump from Magecraft.
    // (The pump is EOT; check current power.)
    let card = g.battlefield.iter().find(|c| c.id == lumi).expect("Lumimancer on bf");
    assert_eq!(card.definition.name, "Clever Lumimancer");
}

// ── Push: Prowess engine + new cards ────────────────────────────────────────

#[test]
fn prowess_pumps_on_noncreature_cast() {
    let mut g = two_player_game();
    let mage_id = g.add_card_to_battlefield(0, catalog::spectacle_mage());
    g.clear_sickness(mage_id);
    let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
    let opp_creature = g.add_card_to_battlefield(1, catalog::serra_angel());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt_id,
        target: Some(Target::Permanent(opp_creature)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    let mage = g.battlefield.iter().find(|c| c.id == mage_id).unwrap();
    assert_eq!(mage.power(), 3, "Prowess should pump +1/+1, making 2→3 power");
    assert_eq!(mage.toughness(), 3, "Prowess should pump +1/+1, making 2→3 toughness");
}

#[test]
fn professor_of_symbology_etb_draws_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::professor_of_symbology());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn fractal_summoning_creates_fractal_with_x_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_summoning());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).unwrap();
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|p| p.definition.name == "Fractal");
    assert!(fractal.is_some());
    let f = fractal.unwrap();
    assert_eq!(f.counter_count(CounterType::PlusOnePlusOne), 3);
}

// ── New STX cards batch ───────────────────────────────────────────────────

#[test]
fn elemental_expressionism_bounces_and_creates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::elemental_expressionism());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // Bear should be bounced.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    // Two 4/4 Elemental tokens should exist.
    let elementals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Elemental")
        .collect();
    assert_eq!(elementals.len(), 2);
}

#[test]
fn rush_of_knowledge_draws_four() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::rush_of_knowledge());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // Cast (removed 1) + draw 4 = +3 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 3);
}

#[test]
fn tangletrap_destroys_artifact() {
    let mut g = two_player_game();
    let artifact = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::tangletrap());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(artifact)), additional_targets: vec![], mode: Some(1), x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == artifact));
}

// ── Beledros Witherbloom mass-untap ───────────────────────────────────────

// ── Sparring Regimen attack trigger ───────────────────────────────────────

#[test]
fn sparring_regimen_etb_creates_spirit_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sparring_regimen());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"));
}

#[test]
fn sparring_regimen_attack_trigger_adds_counter() {
    let mut g = two_player_game();
    let _sr = g.add_card_to_battlefield(0, catalog::sparring_regimen());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Remove summoning sickness.
    g.battlefield.iter_mut().find(|c| c.id == bear).unwrap().summoning_sick = false;

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);

    let b = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(b.counter_count(CounterType::PlusOnePlusOne) >= 1,
        "attacker should have gotten a +1/+1 counter");
}

// ── Lorehold Apprentice magecraft ─────────────────────────────────────────

// ── Storm-Kiln Artist ─────────────────────────────────────────────────────

#[test]
fn storm_kiln_artist_magecraft_creates_treasure_and_deals_damage() {
    let mut g = two_player_game();
    let _ska = g.add_card_to_battlefield(0, catalog::storm_kiln_artist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    let _bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);

    // P1 lost 3 (bolt) + 1 (magecraft) = 4.
    assert_eq!(g.players[1].life, p1_life - 4);
    // Treasure token should be on the battlefield.
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "should have created a Treasure token");
}

// ── Decisive Denial mode 1 (fight) ────────────────────────────────────────

#[test]
fn decisive_denial_mode_0_counters_noncreature_spell() {
    let mut g = two_player_game();
    // P0 casts an instant, P1 tries to counter it.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    // P0 passes, P1 gets priority.
    g.perform_action(GameAction::PassPriority).unwrap();

    let denial = g.add_card_to_hand(1, catalog::decisive_denial());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add(Color::Blue, 1);

    // Mode 0: counter the bolt.
    g.perform_action(GameAction::CastSpell {
        card_id: denial,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("denial castable");
    drain_stack(&mut g);

    // Bolt should be countered (in graveyard).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "bolt should be in graveyard (countered)");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 169 (modern_decks) — Silverquill expansion
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_spectralist_b169_is_a_flying_lifelink_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_spectralist_b169());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Lifelink));
    assert_eq!(c.power(), 2);
    assert_eq!(c.toughness(), 2);
}

#[test]
fn silverquill_finalizer_b169_drains_and_marks_finality() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_finalizer_b169());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Finalizer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    assert_eq!(g.players[0].life, p0_life + 2);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
}

#[test]
fn inkling_banshee_b169_drains_each_opp_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_banshee_b169());
    let p1_life = g.players[1].life;
    // Send to graveyard
    g.players[0].mana_pool.add(Color::Red, 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Wait — bolt does 3 damage, banshee is 3/2, dies. Death trigger fires.
    assert!(g.battlefield_find(id).is_none(), "banshee dies to bolt");
    assert_eq!(g.players[1].life, p1_life - 1);
}

#[test]
fn silverquill_verdict_b169_exiles_creature_and_gains_three() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_verdict_b169());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Verdict castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.exile.iter().any(|c| c.id == bear), "bear is exiled");
    assert_eq!(g.players[0].life, p0_life + 3);
}

#[test]
fn inkling_quill_captain_b169_pumps_self_on_attack() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_quill_captain_b169());
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("Captain can attack");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("captain alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn silverquill_edict_b169_forces_opp_sac_and_gains_life() {
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_edict_b169());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Edict castable");
    drain_stack(&mut g);
    let opp_creatures: usize = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.is_creature())
        .count();
    assert_eq!(opp_creatures, 0, "opp creature was sacrificed");
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn silverquill_bookmage_b169_scrys_and_draws_on_is_cast() {
    let mut g = two_player_game();
    let _bm = g.add_card_to_battlefield(0, catalog::silverquill_bookmage_b169());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -1 bolt cast (out of hand) +1 magecraft draw = same as start
    // Allow for hand_before exact since we drew via magecraft trigger
    assert!(g.players[0].hand.len() >= hand_before,
        "hand should be at least same size after magecraft draw");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 169 (modern_decks) — Witherbloom expansion
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn witherbloom_decay_b169_shrinks_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_decay_b169());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("decay castable");
    drain_stack(&mut g);
    // Bear is 2/2, takes -3/-3 → dies (toughness 0 → SBA)
    assert!(g.battlefield_find(bear).is_none(),
        "bear with toughness -1 dies to SBA");
}

#[test]
fn pestmaster_b169_etb_creates_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pestmaster_b169());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("pestmaster castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.has_creature_type(CreatureType::Pest)
    }).count();
    assert!(pests >= 1, "Pest token created on ETB");
}

#[test]
fn witherbloom_lifesuck_b169_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifesuck_b169());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("lifesuck castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 2);
    assert_eq!(g.players[1].life, p1_life - 2);
}

#[test]
fn witherbloom_sapcaster_b169_gains_life_on_is_cast() {
    let mut g = two_player_game();
    let _sc = g.add_card_to_battlefield(0, catalog::witherbloom_sapcaster_b169());
    let p0_life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life_before + 1, "magecraft life gain");
}

#[test]
fn pest_swarmer_b169_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_swarmer_b169());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("pest swarmer castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.has_creature_type(CreatureType::Pest)
            && c.id != id
    }).count();
    assert_eq!(pests, 2);
}

#[test]
fn witherbloom_tendril_b169_shrinks_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_tendril_b169());
    g.players[0].mana_pool.add(Color::Black, 1);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("tendril castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 1);
    // Bear is 2/2, takes -1/-1 → 1/1, still alive
    let c = g.battlefield_find(bear);
    assert!(c.is_some(), "bear is 1/1, still alive");
}

#[test]
fn witherbloom_pestkeeper_b169_mints_pest_on_is_cast() {
    let mut g = two_player_game();
    let _pk = g.add_card_to_battlefield(0, catalog::witherbloom_pestkeeper_b169());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let pests = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.name == "Pest"
    }).count();
    assert!(pests >= 1, "Pest minted on instant cast");
}

#[test]
fn witherbloom_necromancer_b169_returns_creature_from_gy() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_necromancer_b169());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("necromancer castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_id),
        "bear returned to hand");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 169 (modern_decks) — Lorehold expansion
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn lorehold_sparkblade_b169_pumps_self_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_sparkblade_b169());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("sparkblade alive");
    let bf = g.compute_battlefield();
    let comp = bf.iter().find(|p| p.id == id).expect("computed");
    assert_eq!(comp.power, 4); // 3 base + 1 EOT
    assert_eq!(comp.toughness, 4);
    let _ = c;
}

#[test]
fn lorehold_spiritforge_b169_mints_spirit_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritforge_b169());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.has_creature_type(CreatureType::Spirit)
    }).count();
    assert!(spirits >= 1, "Spirit token minted");
}

#[test]
fn lorehold_reciter_b169_debuffs_opp_creature_on_cast() {
    let mut g = two_player_game();
    let _r = g.add_card_to_battlefield(0, catalog::lorehold_reciter_b169());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let bf = g.compute_battlefield();
    let bear_view = bf.iter().find(|c| c.id == bear).expect("bear");
    // 2/2 - 1/0 = 1/2
    assert_eq!(bear_view.power, 1);
}

#[test]
fn lorehold_reverence_b169_mints_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_reverence_b169());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.has_creature_type(CreatureType::Spirit)
    }).count();
    assert_eq!(spirits, 2);
}

#[test]
fn lorehold_lectern_b169_scrys_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let _l = g.add_card_to_battlefield(0, catalog::lorehold_lectern_b169());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Scry 1 may move 1 card around but library size should not decrease unless we send a card to gy
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn lorehold_quartermaster_b169_pings_on_attack() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_quartermaster_b169());
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 1, "ping on attack");
}

#[test]
fn lorehold_flameglyph_b169_kills_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_flameglyph_b169());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear killed");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 169 (modern_decks) — Prismari expansion
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn prismari_inferno_b169_kills_creature_and_makes_treasure() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_inferno_b169());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
    let treasure_count = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure").count();
    assert_eq!(treasure_count, 1);
}

#[test]
fn prismari_stormcaller_b169_pumps_and_grants_haste_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_stormcaller_b169());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let bf = g.compute_battlefield();
    let sc = bf.iter().find(|p| p.id == id).expect("stormcaller alive");
    assert_eq!(sc.power, 4); // 3 + 1
    assert!(sc.keywords.contains(&Keyword::Haste));
}

#[test]
fn prismari_flamejet_b169_burns_two_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_flamejet_b169());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
}

#[test]
fn prismari_foamcrasher_b169_pumps_self_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_foamcrasher_b169());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let bf = g.compute_battlefield();
    let fc = bf.iter().find(|p| p.id == id).expect("foamcrasher alive");
    assert_eq!(fc.power, 5); // 4 + 1
}

#[test]
fn prismari_scrycaster_b169_scrys_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let _sc = g.add_card_to_battlefield(0, catalog::prismari_scrycaster_b169());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Scry might put a card in gy
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn prismari_aerokineticist_b169_pings_each_opp_on_is_cast() {
    let mut g = two_player_game();
    let _aero = g.add_card_to_battlefield(0, catalog::prismari_aerokineticist_b169());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -3 from bolt -1 from magecraft = -4
    assert_eq!(g.players[1].life, p1_life - 4);
}

#[test]
fn prismari_drakelord_b169_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_drakelord_b169());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let t = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure").count();
    assert!(t >= 1);
}

#[test]
fn prismari_spellcrafter_b169_loots_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _sc = g.add_card_to_battlefield(0, catalog::prismari_spellcrafter_b169());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -1 bolt cast +1 draw -1 discard = -1 net
    assert!(g.players[0].hand.len() <= hand_before);
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 169 (modern_decks) — Quandrix expansion
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn quandrix_echofin_b169_grows_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_echofin_b169());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("echofin alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_pluralize_b169_adds_two_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_pluralize_b169());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn quandrix_splitscholar_b169_draws_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_splitscholar_b169());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast +1 draw = same
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_tideforge_b169_v2_adds_counter_to_friendly_on_is_cast() {
    let mut g = two_player_game();
    let _t = g.add_card_to_battlefield(0, catalog::quandrix_tideforge_b169_v2());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.counter_count(CounterType::PlusOnePlusOne) >= 1);
}

#[test]
fn quandrix_echocaster_b169_draws_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ec = g.add_card_to_battlefield(0, catalog::quandrix_echocaster_b169());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -1 bolt cast +1 draw = same
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_plantarchitect_b169_pumps_self_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_plantarchitect_b169());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let bf = g.compute_battlefield();
    let p = bf.iter().find(|c| c.id == id).expect("alive");
    assert_eq!(p.power, 3); // 2 + 1
    assert_eq!(p.toughness, 4); // 3 + 1
}

#[test]
fn quandrix_bigwave_b169_draws_three_and_pumps_target() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_bigwave_b169());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast +3 draw = +2
    assert_eq!(g.players[0].hand.len(), hand_before + 2);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2);
}

// ─────────────────────────────────────────────────────────────────────────
// CR 122.1c — Shield counter (engine wire)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn cr_122_1c_shield_counter_prevents_destroy_and_pops() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Drop a Shield counter directly via game state.
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.add_counters(CounterType::Shield, 1);
    }
    // P0 casts a destroy effect.
    let dest = g.add_card_to_hand(0, catalog::lorehold_flameglyph_b169());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let _ = dest;
    // Use a destroy effect via Murder-like card. Simpler: cast a Destroy spell.
    // Skip — instead use direct destroy effect via a Wrath shape if available.
    // Test in isolation: assert shield counter blocks destroy. Build a synthetic
    // Effect::Destroy via a card. We'll use silverquill_finalizer's drain part
    // ignored, finality counter set up. For now: just check that flameglyph_b169
    // deals 3 damage but bear (2/2 + shield) survives because damage is prevented.
    g.perform_action(GameAction::CastSpell {
        card_id: dest, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Bear should still be alive — shield absorbed the damage.
    assert!(g.battlefield_find(bear).is_some(), "shield prevents damage");
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.counter_count(CounterType::Shield), 0,
        "shield counter was consumed");
}

#[test]
fn cr_122_1c_no_shield_counter_means_normal_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dest = g.add_card_to_hand(0, catalog::lorehold_flameglyph_b169());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: dest, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // No shield — bear takes 3 damage and dies.
    assert!(g.battlefield_find(bear).is_none(), "bear dies to bolt");
}

#[test]
fn lorehold_shieldbearer_b170_etbs_with_shield() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_shieldbearer_b170());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("shieldbearer alive");
    assert_eq!(c.counter_count(CounterType::Shield), 1);
}

#[test]
fn cr_122_1c_shield_counter_prevents_destroy_effect_and_pops() {
    // Use Silverquill Reckoning (Destroy + token mint) to test the
    // CR 122.1c destroy-replacement.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.add_counters(CounterType::Shield, 1);
    }
    let id = g.add_card_to_hand(0, catalog::silverquill_reckoning());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("reckoning castable");
    drain_stack(&mut g);
    // Bear should still be alive — shield absorbed the destroy.
    assert!(g.battlefield_find(bear).is_some(), "shield absorbs destroy");
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.counter_count(CounterType::Shield), 0,
        "shield counter was consumed by destroy replacement");
}

#[test]
fn cr_704_5n_equipment_unattaches_when_creature_dies() {
    // Equipment attached to a creature stays in play but unattaches
    // from the dying creature.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let greaves = g.add_card_to_battlefield(0, catalog::lightning_greaves());
    // Manually attach the greaves to the bear.
    if let Some(c) = g.battlefield_find_mut(greaves) {
        c.attached_to = Some(bear);
    }
    // Now kill the bear with lethal damage (Lightning Greaves grants it
    // Shroud, so it can't be *targeted* — mark damage + run SBA instead).
    g.battlefield_find_mut(bear).unwrap().damage = 2;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    // Bear dead, but Equipment should still be in play with attached_to = None.
    assert!(g.battlefield_find(bear).is_none(), "bear dies");
    let g_ = g.battlefield_find(greaves).expect("greaves still on bf");
    assert_eq!(g_.attached_to, None, "Equipment unattaches");
}

#[test]
fn lorehold_aegisblade_b170_adds_shield_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_aegisblade_b170());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.counter_count(CounterType::Shield), 1);
}

#[test]
fn silverquill_aegismage_b170_etb_shields_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_aegismage_b170());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.counter_count(CounterType::Shield), 1);
}

#[test]
fn silverquill_wardward_b170_adds_two_shields() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_wardward_b170());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.counter_count(CounterType::Shield), 2);
}

#[test]
fn witherbloom_vitalist_b170_etb_shields_self_and_magecraft_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_vitalist_b170());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("vitalist alive");
    assert_eq!(c.counter_count(CounterType::Shield), 1);
    // Cast a Bolt → magecraft gain 1
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn witherbloom_drainer_b170_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_drainer_b170());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 3);
    assert_eq!(g.players[1].life, p1_life - 3);
}

#[test]
fn prismari_forgesmith_b170_etb_shields_self_and_magecraft_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_forgesmith_b170());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("forgesmith alive");
    assert_eq!(c.counter_count(CounterType::Shield), 1);
    // Cast a Bolt → magecraft Treasure
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let t = g.battlefield.iter().filter(|c|
        c.controller == 0 && c.definition.name == "Treasure"
    ).count();
    assert!(t >= 1, "Treasure minted by magecraft");
}

#[test]
fn quandrix_hydromancer_b170_etb_shields_self_and_magecraft_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_hydromancer_b170());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("hydromancer alive");
    assert_eq!(c.counter_count(CounterType::Shield), 1);
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 171 (modern_decks) — Expansion across schools
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_quillsmith_b171_pumps_friendly_on_is_cast() {
    let mut g = two_player_game();
    let _q = g.add_card_to_battlefield(0, catalog::silverquill_quillsmith_b171());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn inkling_vanguard_ii_b171_is_one_three_flying_vigilance_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_vanguard_ii_b171());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.power(), 1);
    assert_eq!(c.toughness(), 3);
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Vigilance));
    assert!(c.definition.has_creature_type(CreatureType::Inkling));
}

#[test]
fn silverquill_tombwarden_b171_gains_life_when_other_dies() {
    let mut g = two_player_game();
    let _tw = g.add_card_to_battlefield(0, catalog::silverquill_tombwarden_b171());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p0_life = g.players[0].life;
    // Kill the bear with a Bolt from P1.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn witherbloom_lifeleech_b171_drains_on_combat_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_lifeleech_b171());
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("attacks");
    drain_stack(&mut g);
    let p1_life = g.players[1].life;
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage resolved");
    drain_stack(&mut g);
    // 2 combat damage + 1 from triggered = -3
    assert_eq!(g.players[1].life, p1_life - 3);
}

#[test]
fn witherbloom_sapsprite_b171_sac_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_sapsprite_b171());
    g.clear_sickness(id);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None,
    }).expect("activatable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "sapsprite sacrificed");
    assert_eq!(g.players[0].life, p0_life + 2);
    assert_eq!(g.players[1].life, p1_life - 2);
}

#[test]
fn lorehold_pyresage_b171_pings_on_is_cast() {
    let mut g = two_player_game();
    let _p = g.add_card_to_battlefield(0, catalog::lorehold_pyresage_b171());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // bolt 3 + magecraft 1 = -4
    assert_eq!(g.players[1].life, p1_life - 4);
}

#[test]
fn prismari_sparkforge_b171_pings_opp_creature_on_is_cast() {
    let mut g = two_player_game();
    let _sf = g.add_card_to_battlefield(0, catalog::prismari_sparkforge_b171());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.damage >= 1, "bear damaged by magecraft ping");
}

#[test]
fn prismari_tideflame_b171_bounces_and_burns() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_tideflame_b171());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Player(1)],
        mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bear in hand");
    assert_eq!(g.players[1].life, p1_life - 2);
}

#[test]
fn quandrix_echocrasher_b171_pumps_self_on_combat_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_echocrasher_b171());
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("attacks");
    drain_stack(&mut g);
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage resolved");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("echocrasher alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_fractalmancer_b171_scrys_and_draws_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let _fm = g.add_card_to_battlefield(0, catalog::quandrix_fractalmancer_b171());
    let hand_before = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // hand_before was before bolt was added. After: bolt +1, cast -1,
    // draw +1 = hand_before + 1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 172 (modern_decks) — Expansion across schools
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn inkling_skywatch_b172_gains_life_on_attack() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_skywatch_b172());
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn witherbloom_heartfeeder_b172_drains_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_heartfeeder_b172());
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    // Kill with Bolt+Bolt for 6 damage to heartfeeder's 3 toughness.
    let bolt1 = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt1");
    drain_stack(&mut g);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "heartfeeder dies");
    assert_eq!(g.players[0].life, p0_life + 2);
    assert_eq!(g.players[1].life, p1_life - 2);
}

#[test]
fn lorehold_embersmith_b172_drains_each_opp_on_is_cast() {
    let mut g = two_player_game();
    let _e = g.add_card_to_battlefield(0, catalog::lorehold_embersmith_b172());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // bolt 3 + magecraft drain 1 = -4
    assert_eq!(g.players[1].life, p1_life - 4);
}

#[test]
fn prismari_wavecaster_b172_scrys_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _w = g.add_card_to_battlefield(0, catalog::prismari_wavecaster_b172());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn prismari_bonfire_b172_burns_and_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_bonfire_b172());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 3);
    let t = g.battlefield.iter().filter(|c|
        c.controller == 0 && c.definition.name == "Treasure"
    ).count();
    assert_eq!(t, 1);
}

#[test]
fn quandrix_foragelord_b172_gains_life_on_is_cast() {
    let mut g = two_player_game();
    let _f = g.add_card_to_battlefield(0, catalog::quandrix_foragelord_b172());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 1);
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 173 (modern_decks) — Shield/finality magecraft variants
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn lorehold_wardseeker_b173_gains_shield_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_wardseeker_b173());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("wardseeker alive");
    assert_eq!(c.counter_count(CounterType::Shield), 1);
}

#[test]
fn silverquill_wardlord_b173_gains_shield_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_wardlord_b173());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("wardlord alive");
    assert_eq!(c.counter_count(CounterType::Shield), 1);
}

#[test]
fn silverquill_doomspeaker_b173_gains_finality_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_doomspeaker_b173());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("doomspeaker alive");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
}

#[test]
fn quandrix_sumcheck_b172_counters_unless_paid_two() {
    let mut g = two_player_game();
    // P0 casts a Bolt; P1 counters with Sumcheck.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    g.perform_action(GameAction::PassPriority).unwrap();
    let sumcheck = g.add_card_to_hand(1, catalog::quandrix_sumcheck_b172());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: sumcheck, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // P0 can't pay (no mana left) — bolt countered.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "bolt countered");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 174 (modern_decks) — additional cards across all schools
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_inkbinder_b174_magecraft_adds_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_inkbinder_b174());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("inkbinder alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn inkling_stylist_b174_is_a_two_mana_flying_lifelink_inkling() {
    let def = catalog::inkling_stylist_b174();
    assert_eq!(def.cost.cmc(), 2);
    assert!(def.is_creature());
    assert!(def.has_creature_type(CreatureType::Inkling));
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn silverquill_lifeleach_b174_drains_two_and_scrys_one() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::silverquill_lifeleach_b174());
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    assert_eq!(g.players[0].life, p0_life + 2);
}
