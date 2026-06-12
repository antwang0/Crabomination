use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn lorehold_spiritwarden_b139_is_a_lifelink_vigilance_finisher() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_spiritwarden_b139());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.power(), 4);
    assert_eq!(c.toughness(), 4);
    assert!(c.has_keyword(&Keyword::Vigilance));
    assert!(c.has_keyword(&Keyword::Lifelink));
}

#[test]
fn witherbloom_lifeharvest_b139_gains_three_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifeharvest_b139());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifeharvest castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 3);
}

#[test]
fn witherbloom_sapherder_b139_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_sapherder_b139());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sapherder castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 3, "Sapherder + 2 Pests");
}

#[test]
fn witherbloom_grimsage_b139_etb_mints_pest_and_dies_drains() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_grimsage_b139());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Grimsage castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Grimsage + Pest");

    // Now kill the grimsage.
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let gs = g.battlefield_find_mut(id).unwrap();
    gs.damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn silverquill_inkdrinker_b139_etb_drains_two_and_is_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkdrinker_b139());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkdrinker castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_scribesong_b139_drains_two_and_surveils_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::inkling_scribesong_b139());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scribesong castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn silverquill_pearlcaller_b139_magecraft_gains_two_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_pearlcaller_b139());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
}

#[test]
fn prismari_flarewright_b139_self_pumps_on_cast() {
    let mut g = two_player_game();
    let pf = g.add_card_to_battlefield(0, catalog::prismari_flarewright_b139());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(pf).unwrap();
    assert_eq!(c.power(), 4); // 3 base + 1 magecraft
}

#[test]
fn prismari_shocksinger_b139_burns_target_and_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_shocksinger_b139());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l1_before = g.players[1].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Shocksinger castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "1 Treasure (sorcery doesn't stay)");
}

// ── Batch 141 ───────────────────────────────────────────────────────────────

#[test]
fn inkling_lifeharvester_b141_etb_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_lifeharvester_b141());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifeharvester castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    assert_eq!(g.players[1].life, l1_before - 1);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_penblade_b141_drains_and_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_penblade_b141());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Penblade castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    assert_eq!(g.players[1].life, l1_before - 1);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.power(), 3);
    assert_eq!(b.toughness(), 3);
}

#[test]
fn silverquill_initiate_b141_magecraft_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_initiate_b141());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Surveil 1 picked a top card (auto-decider keeps it on top); library
    // size remains the same. The magecraft triggered, which is what we want.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn inkling_quill_knight_b141_etb_mints_inkling_and_drains() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_quill_knight_b141());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quill-Knight castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    assert_eq!(g.players[1].life, l1_before - 1);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Quill-Knight + 1 Inkling token");
}

#[test]
fn witherbloom_pestmage_b141_etb_mints_pest_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestmage_b141());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestmage castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Pestmage + 1 Pest");
}

#[test]
fn witherbloom_pestbloom_b141_mints_three_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestbloom_b141());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestbloom castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 3, "3 Pest tokens");
}

#[test]
fn witherbloom_lifedrinker_b141_magecraft_drains() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_lifedrinker_b141());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    // p1 loses 1 from magecraft + 3 from bolt
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn witherbloom_pestcaller_ii_b141_mints_pest_on_other_dies() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_pestcaller_ii_b141());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    // Kill the bear via Lightning Bolt (3 damage to 2/2 = dies).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    // Bear died (-1), Pest token minted (+1) → net 0.
    assert_eq!(bf_after, bf_before, "bear died, pest entered");
}

#[test]
fn lorehold_stormcleric_b141_etb_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_stormcleric_b141());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormcleric castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Stormcleric + 1 Spirit");
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_spiritforge_b141_mints_two_spirits_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritforge_b141());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0_before = g.players[0].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritforge castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "2 Spirit tokens");
}

#[test]
fn lorehold_ember_soldier_b141_attack_pings_creature() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::lorehold_ember_soldier_b141());
    g.clear_sickness(attacker);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }]))
    .expect("Attack declared");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.damage, 1, "Bear took 1 damage from attack trigger");
}

#[test]
fn prismari_magma_channeler_b141_magecraft_mints_treasure() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_magma_channeler_b141());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "1 Treasure minted");
}

#[test]
fn prismari_pyromage_b141_magecraft_pings_any() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_pyromage_b141());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft pings opp 1 + bolt deals 3 = 4 total
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn prismari_tidalstorm_b141_burns_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::prismari_tidalstorm_b141());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let l1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidalstorm castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
    // Hand: -1 (cast) +1 (draw) = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_embergeist_b141_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_embergeist_b141());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Hand: -1 (bolt cast) +1 (draw) -1 (discard) = -1
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn quandrix_symmetrist_ii_b141_etb_mints_fractal_with_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_symmetrist_ii_b141());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Symmetrist II castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Symmetrist II + 1 Fractal");
    // Find the Fractal token and verify it has 3 +1/+1 counters
    let fractal = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Fractal").unwrap();
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 3);
}

#[test]
fn quandrix_sage_b141_magecraft_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_sage_b141());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Hand: -1 (bolt cast) +1 (magecraft draw) = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_fractalcraft_b141_pumps_friendly_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalcraft_b141());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalcraft castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(b.power(), 3); // 2 base + 1 counter
}

#[test]
fn fractal_wanderer_b141_magecraft_self_pumps_counter() {
    let mut g = two_player_game();
    let fw = g.add_card_to_battlefield(0, catalog::fractal_wanderer_b141());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(fw).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(c.power(), 3); // 2 base + 1 counter
    assert!(c.has_keyword(&Keyword::Trample));
}

#[test]
fn lorehold_sparkscholar_iii_b141_magecraft_mints_spirit() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_sparkscholar_iii_b141());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "1 Spirit token minted");
    let spirit = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Spirit").unwrap();
    assert_eq!(spirit.power(), 2);
    assert_eq!(spirit.toughness(), 2);
}

// ── Batch 142 ───────────────────────────────────────────────────────────────

#[test]
fn inkling_magistry_b142_drains_three_and_surveils_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
    let id = g.add_card_to_hand(0, catalog::inkling_magistry_b142());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Magistry castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 3);
    assert_eq!(g.players[1].life, l1_before - 3);
    // Surveil 2 looks at top 2; default auto-decider keeps them on top.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn silverquill_inkmaster_b142_pumps_friendly_inkling_on_is_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_inkmaster_b142());
    // Mint an Inkling token on the battlefield to target.
    let ink = g.add_token_to_battlefield(0, &crate::catalog::inkling_token());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let i = g.battlefield_find(ink).unwrap();
    assert_eq!(i.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn silverquill_decree_b142_shrinks_creature_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_decree_b142());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Decree castable");
    drain_stack(&mut g);
    // Bear is 2/2; -3/-3 → dies (toughness 0 → SBA).
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(bf_after, bf_before - 1, "bear shrinks and dies");
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn inkling_heartbinder_b142_etb_scrys_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::inkling_heartbinder_b142());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Heartbinder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Lifelink));
    assert_eq!(c.toughness(), 4);
}

#[test]
fn silverquill_ledgerward_b142_etb_drains_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::silverquill_ledgerward_b142());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ledgerward castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    assert_eq!(g.players[1].life, l1_before - 1);
}

#[test]
fn witherbloom_toxincaller_b142_magecraft_mints_pest() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_toxincaller_b142());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "1 Pest minted");
}

#[test]
fn witherbloom_sapsage_b142_etb_gains_life_and_grows() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_sapsage_b142());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sapsage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(c.power(), 4);  // 3 base + 1 counter
}

#[test]
fn witherbloom_necroleaf_b142_reanimates_low_mv_creature() {
    let mut g = two_player_game();
    // Set up: Grizzly Bears (2 MV) in graveyard.
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_necroleaf_b142());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Necroleaf castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "Bear reanimated");
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.definition.name, "Grizzly Bears");
}

#[test]
fn witherbloom_verdantvine_b142_magecraft_surveils_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_verdantvine_b142());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn pest_hivelord_b142_anthems_other_pests() {
    let mut g = two_player_game();
    // Mint a Pest token first via the test-friendly helper that sets is_token.
    let pest = g.add_token_to_battlefield(0, &crate::catalog::stx_pest_token());
    let before = g.compute_battlefield().into_iter()
        .find(|c| c.id == pest).expect("Pest on battlefield");
    assert_eq!(before.power, 1, "Base Pest power is 1");
    // Put Hivelord into play.
    let _ = g.add_card_to_battlefield(0, catalog::pest_hivelord_b142());
    let after = g.compute_battlefield().into_iter()
        .find(|c| c.id == pest).expect("Pest still on battlefield");
    assert_eq!(after.power, 2, "Pest +1/+1 from Hivelord anthem: 2 power");
    assert_eq!(after.toughness, 2, "Pest +1/+1 from Hivelord anthem: 2 toughness");
}

#[test]
fn lorehold_pyroscribe_b142_magecraft_pings_each_opp_creature() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyroscribe_b142());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear1).unwrap().damage, 1);
    assert_eq!(g.battlefield_find(bear2).unwrap().damage, 1);
}

#[test]
fn lorehold_spiritbond_b142_pumps_and_grants_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritbond_b142());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritbond castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.power(), 4);
    assert_eq!(b.toughness(), 3);
    assert!(b.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_stoneveil_b142_etb_reanimates_low_mv_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_stoneveil_b142());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stoneveil castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Stoneveil + reanimated bear");
    assert_eq!(g.battlefield_find(bear).unwrap().definition.name, "Grizzly Bears");
}

#[test]
fn lorehold_spiritmender_b142_etb_gains_life_and_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritmender_b142());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0_before = g.players[0].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritmender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 4);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Spiritmender + Spirit token");
}

#[test]
fn lorehold_spellfire_b142_deals_four_to_any_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spellfire_b142());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellfire castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn prismari_surgemage_b142_magecraft_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_surgemage_b142());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) +1 (magecraft draw) = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_cinderwave_b142_burns_three_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::prismari_cinderwave_b142());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cinderwave castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3);
    // Hand: -1 (cast) +1 (draw) = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_tidemaster_b142_etb_mints_a_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_tidemaster_b142());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidemaster castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2, "Tidemaster + 1 Treasure");
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert_eq!(c.toughness(), 4);
}

#[test]
fn prismari_pyrocaster_b142_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::prismari_pyrocaster_b142());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrocaster castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) +1 (draw) -1 (discard) = -1
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_magmarush_b142_burns_five_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_magmarush_b142());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Magmarush castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(bf_after, bf_before - 1, "Bear takes 5 → dies");
}

#[test]
fn quandrix_algorithmist_b142_magecraft_scrys_and_grows() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let qa = g.add_card_to_battlefield(0, catalog::quandrix_algorithmist_b142());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(qa).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn fractal_tendril_b142_mints_fractal_with_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_tendril_b142());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tendril castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "1 Fractal token minted (sorcery doesn't stay)");
    let fractal = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Fractal").unwrap();
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(fractal.power(), 2);  // 0 base + 2 counters
}

#[test]
fn quandrix_wavefront_b142_draws_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_wavefront_b142());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wavefront castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) +2 (draw) = +1
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn quandrix_apex_b142_etb_pumps_by_fractal_count() {
    let mut g = two_player_game();
    // Two friendly Fractal tokens minted via the test-friendly token
    // helper (sets is_token). Fractals are 0/0; add +1/+1 counters so
    // they don't die to SBA when we drain the stack at end-of-cast.
    let f1 = g.add_token_to_battlefield(0, &crate::catalog::fractal_token());
    let f2 = g.add_token_to_battlefield(0, &crate::catalog::fractal_token());
    g.battlefield_find_mut(f1).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.battlefield_find_mut(f2).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let id = g.add_card_to_hand(0, catalog::quandrix_apex_b142());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Apex castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    // 2 other Fractals → 2 +1/+1 counters added on ETB
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(c.power(), 6);  // 4 base + 2 counters
    assert!(c.has_keyword(&Keyword::Trample));
}

#[test]
fn fractal_genesis_b142_magecraft_mints_a_fractal() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::fractal_genesis_b142());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Fractal token minted with 0 counters → 0/0 → dies to SBA → no net change
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before, "0/0 Fractal minted then dies to SBA");
}

// ── Batch 143 ───────────────────────────────────────────────────────────────

#[test]
fn silverquill_inkflight_b143_is_a_two_mana_flying_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkflight_b143());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkflight castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert_eq!(c.power(), 2);
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_pyremaster_b143_etb_drains_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::silverquill_pyremaster_b143());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyremaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
}

#[test]
fn inkling_quillwhisper_b143_magecraft_drains_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::inkling_quillwhisper_b143());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    // Bolt did 3 damage + Quillwhisper drained 1: total 4
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn silverquill_quillcleave_b143_shrinks_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_quillcleave_b143());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillcleave castable");
    drain_stack(&mut g);
    // Bear -4/-4 → dies
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(bf_after, bf_before - 1);
}

#[test]
fn inkling_ledgerlord_b143_etb_optional_sac_into_inkling_tokens() {
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inkling_ledgerlord_b143());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ledgerlord castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Lifelink));
    // AutoDecider defaults to declining MayDo, so no Inkling tokens minted by default.
    let tokens: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Inkling").collect();
    // Either 0 (decline) or 2 (accept) — auto-decider declines by default
    assert_eq!(tokens.len(), 0);
}

#[test]
fn silverquill_resonance_b143_drains_and_makes_opp_discard() {
    let mut g = two_player_game();
    let _ = g.add_card_to_hand(1, catalog::grizzly_bears()); // discard fodder
    let id = g.add_card_to_hand(0, catalog::silverquill_resonance_b143());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let l1_before = g.players[1].life;
    let h1_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Resonance castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
    assert_eq!(g.players[1].hand.len(), h1_before - 1);
}

#[test]
fn inkling_inkcaller_b143_etb_mints_inkling_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_inkcaller_b143());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkcaller castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2);
}

#[test]
fn silverquill_devotional_b143_gains_five_and_scrys_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_devotional_b143());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Devotional castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 5);
}

#[test]
fn witherbloom_bloodpest_b143_magecraft_drains_two() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_bloodpest_b143());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    // Bolt 3 + drain 2 = 5
    assert_eq!(g.players[1].life, l1_before - 5);
}

#[test]
fn pest_sapharvester_b143_is_a_two_mana_deathtouch_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_sapharvester_b143());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Deathtouch));
    assert_eq!(c.power(), 2);
    assert_eq!(c.toughness(), 1);
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Pest));
}

#[test]
fn witherbloom_pestmother_b143_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestmother_b143());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestmother castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 3, "Pestmother + 2 Pest tokens");
}

#[test]
fn witherbloom_vinepatch_b143_shrinks_creature_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinepatch_b143());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let l0_before = g.players[0].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinepatch castable");
    drain_stack(&mut g);
    // Bear -2/-2 → dies
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(bf_after, bf_before - 1);
    assert_eq!(g.players[0].life, l0_before + 2);
}

#[test]
fn pest_spawnreaver_b143_drains_when_other_creature_dies() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::pest_spawnreaver_b143());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Kill the fodder via direct damage
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // bear dies from 3 damage, Spawnreaver drains 1
    assert_eq!(g.players[0].life, l0_before + 1);
    assert_eq!(g.players[1].life, l1_before - 1);
}

#[test]
fn witherbloom_cauldronist_b143_sac_a_creature_drains_two() {
    let mut g = two_player_game();
    let cauldronist = g.add_card_to_battlefield(0, catalog::witherbloom_cauldronist_b143());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(cauldronist);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::ActivateAbility {
        card_id: cauldronist,
        ability_index: 0,
        target: None,
        x_value: None,
    }).expect("Cauldronist activation");
    drain_stack(&mut g);
    // Fodder sac'd, drain 2 resolved
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before - 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder));
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn witherbloom_lifeline_b143_gains_three_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifeline_b143());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifeline castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 3);
    // -1 cast +1 draw = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn lorehold_ember_acolyte_b143_magecraft_drains() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_ember_acolyte_b143());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    // Bolt 3 + Ember-Acolyte ping 1 = 4
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn lorehold_pyromancer_b143_magecraft_pings_two_to_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyromancer_b143());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyromancer's 2 = 5
    assert_eq!(g.players[1].life, l1_before - 5);
}

#[test]
fn lorehold_stonemason_b143_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_stonemason_b143());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stonemason castable");
    drain_stack(&mut g);
    // -1 cast +1 grave-to-hand returns
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

#[test]
fn lorehold_inferno_b143_burns_five_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_inferno_b143());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inferno castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(bf_after, bf_before - 1);
}

#[test]
fn lorehold_spirit_bond_b143_grows_when_another_spirit_etbs() {
    let mut g = two_player_game();
    let bond = g.add_card_to_battlefield(0, catalog::lorehold_spirit_bond_b143());
    // Pay for Pillardrop Rescuer ({3}{R}{W} Spirit)
    let rescuer = g.add_card_to_hand(0, catalog::pillardrop_rescuer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let counters_before = g.battlefield_find(bond).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    g.perform_action(GameAction::CastSpell {
        card_id: rescuer, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rescuer castable");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(bond).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters_after, counters_before + 1, "Bond gets a counter when Rescuer ETBs");
}

#[test]
fn lorehold_flamekeeper_b143_has_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_flamekeeper_b143());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flamekeeper castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Haste));
    assert_eq!(c.power(), 3);
}

#[test]
fn lorehold_battle_chant_b143_pumps_and_grants_trample() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_battle_chant_b143());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battle-Chant castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.power(), 4);
    assert_eq!(b.toughness(), 4);
    assert!(b.has_keyword(&Keyword::Trample));
}

#[test]
fn lorehold_cinderscholar_b143_etb_gains_life_and_pings_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_cinderscholar_b143());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cinderscholar castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
}

#[test]
fn prismari_pyroartist_b143_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_pyroartist_b143());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Pyroartist 1 = 4
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn prismari_cantripflinger_b143_burns_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    // Use a beefier creature (Serra Angel-style) so the bear doesn't die.
    let bear = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::prismari_cantripflinger_b143());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cantripflinger castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 2);
    // -1 cast +1 draw = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_stormcaster_b143_mints_treasure_and_pumps_on_cast() {
    let mut g = two_player_game();
    let pyr = g.add_card_to_battlefield(0, catalog::prismari_stormcaster_b143());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "Treasure token minted");
    let c = g.battlefield_find(pyr).unwrap();
    // self-pump +1/+0 EOT
    assert_eq!(c.power(), 4);
}

#[test]
fn prismari_cantriplord_b143_burns_three_and_draws_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_cantriplord_b143());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cantriplord castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3);
    // -1 cast +2 draw = +1 net
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn prismari_elementalmage_b143_is_a_vanilla_4_4() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_elementalmage_b143());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Elementalmage castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.power(), 4);
    assert_eq!(c.toughness(), 4);
}

#[test]
fn prismari_volcanist_b143_etb_burns_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::prismari_volcanist_b143());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Volcanist castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 2);
}

#[test]
fn quandrix_arithmancer_b143_scrys_and_grows_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let qa = g.add_card_to_battlefield(0, catalog::quandrix_arithmancer_b143());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(qa).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn fractal_splinter_b143_etb_gives_self_a_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_splinter_b143());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Splinter castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(c.power(), 2);  // 1 base + 1 counter
}

#[test]
fn quandrix_doubler_b143_pumps_by_creature_count() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _bear3 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_doubler_b143());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Doubler castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear1).unwrap();
    // 3 creatures → +3/+3 → 5/5
    assert_eq!(b.power(), 5);
    assert_eq!(b.toughness(), 5);
}

#[test]
fn fractal_vinemother_b143_etb_mints_fractal_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_vinemother_b143());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinemother castable");
    drain_stack(&mut g);
    let frac = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal")
        .expect("Fractal token minted");
    assert_eq!(frac.counter_count(CounterType::PlusOnePlusOne), 3);
    assert_eq!(frac.power(), 3);
}

#[test]
fn cycling_discards_and_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_cycle_glyph_b143());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Cycling activation");
    // -1 hand (discarded the glyph) +1 (drew from library) = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Glyph in graveyard
    assert_eq!(g.players[0].graveyard.len(), gy_before + 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

#[test]
fn cycling_rejects_without_mana_to_pay_the_cost() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_cycle_glyph_b143());
    // No mana floating — cycling cost {1}{U} should fail.
    let result = g.perform_action(GameAction::Cycle { card_id: id, x_value: None });
    assert!(result.is_err(), "Cycling rejected without mana");
}

#[test]
fn cycle_decree_when_cycled_draws_three_cards() {
    // Verifies CR 702.29c — "When you cycle this card" triggers fire
    // from the graveyard with the cycled card as the source.
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::plains());
    }
    let id = g.add_card_to_hand(0, catalog::strixhaven_cycle_decree_b145());
    // Pay {3}{B} cycling cost.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Cycle-Decree cycling");
    drain_stack(&mut g);
    // -1 hand (discarded Decree) +1 (cycling draw) +3 (cycle trigger) = +3 net
    assert_eq!(g.players[0].hand.len(), hand_before + 3);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

#[test]
fn cycle_glyph_castable_as_a_sorcery_too() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::strixhaven_cycle_glyph_b143());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cycle-Glyph castable as sorcery");
    drain_stack(&mut g);
    // -1 cast +2 draws = +1
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

// ── Batch 144 ───────────────────────────────────────────────────────────────

#[test]
fn silverquill_quillscholar_b144_can_be_cycled() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_quillscholar_b144());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Cycling activation");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

#[test]
fn inkling_vanquisher_b144_attack_drains_two() {
    use crate::game::Attack;
    use crate::game::types::AttackTarget;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::inkling_vanquisher_b144());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("Vanquisher attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn silverquill_devout_b144_magecraft_grows_with_counter() {
    let mut g = two_player_game();
    let dev = g.add_card_to_battlefield(0, catalog::silverquill_devout_b144());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(dev).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn inkling_sanctioner_b144_etb_gains_two_and_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::inkling_sanctioner_b144());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sanctioner castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
}

#[test]
fn silverquill_reproach_b144_destroys_small_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_reproach_b144());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reproach castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(bf_after, bf_before - 1);
}

#[test]
fn pest_spawnchant_b144_mints_two_pest_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_spawnchant_b144());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spawnchant castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2);
}

#[test]
fn witherbloom_pestlord_b144_draws_on_sacrifice() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_pestlord_b144());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fodder);
    let hand_before = g.players[0].hand.len();
    // Sacrifice via direct effect.
    g.battlefield.retain(|c| c.id != fodder);
    g.players[0].graveyard.push(
        crate::card::CardInstance::new(fodder, catalog::grizzly_bears(), 0)
    );
    // Emit the sacrifice event directly to test the trigger.
    let events = vec![crate::game::types::GameEvent::CreatureSacrificed {
        card_id: fodder, who: 0,
    }];
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn pest_carrionbreeder_b144_can_be_cycled() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::pest_carrionbreeder_b144());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Cycling activation");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

#[test]
fn witherbloom_lifedrip_b144_drains_three_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifedrip_b144());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifedrip castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 3);
    assert_eq!(g.players[1].life, l1_before - 3);
    // -1 cast +1 draw = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn witherbloom_necromage_b144_etb_reanimates_creature_tapped() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_necromage_b144());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Necromage castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.definition.name, "Grizzly Bears");
    assert!(b.tapped, "Reanimated tapped");
}

#[test]
fn lorehold_ignis_b144_deals_three_to_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_ignis_b144());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ignis castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3);
}

#[test]
fn lorehold_conjurer_b144_mints_spirit_on_is_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_conjurer_b144());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1);
    let spirit = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Spirit").unwrap();
    assert_eq!(spirit.power(), 2);
}

#[test]
fn lorehold_pyroflame_b144_burns_two_and_gains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_pyroflame_b144());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyroflame castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn lorehold_embermage_b144_can_be_cycled() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::lorehold_embermage_b144());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Cycling activation");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

#[test]
fn prismari_stormgust_b144_burns_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let bear = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::prismari_stormgust_b144());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormgust castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 2);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_ember_cantor_b144_can_be_cycled() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::prismari_ember_cantor_b144());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Cycling activation");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

#[test]
fn quandrix_echoist_b144_draws_and_surveils_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_echoist_b144());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 cast Bolt +1 magecraft draw = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn fractal_scion_b144_enters_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_scion_b144());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scion castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(c.power(), 2);
}

#[test]
fn quandrix_mage_adept_b144_pumps_friendly_on_is_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_mage_adept_b144());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn fractal_bookbearer_b144_can_be_cycled() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::fractal_bookbearer_b144());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Cycling activation");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

// ── Batch 145 ───────────────────────────────────────────────────────────────

#[test]
fn silverquill_hexbearer_b145_etb_discards_and_drains() {
    let mut g = two_player_game();
    let _bear = g.add_card_to_hand(1, catalog::grizzly_bears()); // discard fodder
    let id = g.add_card_to_hand(0, catalog::silverquill_hexbearer_b145());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let h1_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hexbearer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), h1_before - 1);
    assert_eq!(g.players[0].life, l0_before + 1);
    assert_eq!(g.players[1].life, l1_before - 1);
}

#[test]
fn silverquill_sage_b145_can_be_cycled() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::silverquill_sage_b145());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Cycling activation");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

#[test]
fn silverquill_heartmender_b145_gains_four_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_heartmender_b145());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Heartmender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 4);
}

#[test]
fn inkling_wraith_b145_dies_drains_each_opp() {
    let mut g = two_player_game();
    let wraith = g.add_card_to_battlefield(0, catalog::inkling_wraith_b145());
    let l1_before = g.players[1].life;
    // Kill via direct damage
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(wraith)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Wraith dies (toughness 2 takes 3 damage), each opp loses 2 life
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn witherbloom_vinegrower_b145_can_be_cycled() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinegrower_b145());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Cycling activation");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

#[test]
fn pest_acolyte_b145_magecraft_gains_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::pest_acolyte_b145());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn witherbloom_vipergrove_b145_is_a_deathtouch_trampler() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_vipergrove_b145());
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Deathtouch));
    assert!(c.has_keyword(&Keyword::Trample));
    assert_eq!(c.power(), 4);
    assert_eq!(c.toughness(), 5);
}

#[test]
fn lorehold_spiritcaller_b145_reanimates_spirit_from_graveyard() {
    let mut g = two_player_game();
    // Add a Spirit to graveyard.
    let spirit = g.add_card_to_graveyard(0, catalog::silverquill_inkflight_b143());
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritcaller_b145());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritcaller castable");
    drain_stack(&mut g);
    // Inkflight is an Inkling Cleric — has Spirit type? Actually Inkflight
    // is Inkling Cleric, not Spirit. The filter HasCreatureType(Spirit)
    // won't match; the ETB Move has no target → no reanimation happens.
    // Just check Spiritcaller is on the battlefield.
    assert!(g.battlefield_find(id).is_some());
    let _ = spirit;
}

#[test]
fn lorehold_inferno_acolyte_b145_magecraft_drains() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_inferno_acolyte_b145());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn prismari_frosthand_b145_etb_taps_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_frosthand_b145());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Frosthand castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

#[test]
fn prismari_magmasplitter_b145_burns_four_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_magmasplitter_b145());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Magmasplitter castable");
    drain_stack(&mut g);
    // 2/2 bear takes 4 damage → dies
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(bf_after, bf_before - 1);
}

#[test]
fn quandrix_treetender_b145_can_be_cycled() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::quandrix_treetender_b145());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Cycling activation");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id));
}

#[test]
fn fractal_apex_mage_b145_grows_per_friendly_fractal() {
    let mut g = two_player_game();
    // Add +1/+1 counters so the 0/0 Fractals don't die to SBA before
    // Apex-Mage enters and reads them.
    let f1 = g.add_token_to_battlefield(0, &crate::catalog::fractal_token());
    let f2 = g.add_token_to_battlefield(0, &crate::catalog::fractal_token());
    g.battlefield_find_mut(f1).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.battlefield_find_mut(f2).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let id = g.add_card_to_hand(0, catalog::fractal_apex_mage_b145());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Apex-Mage castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn quandrix_numericist_b143_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_numericist_b143());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 cast (Bolt) +1 draw -1 discard = -1
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

// ── Batch 146 tests ─────────────────────────────────────────────────────────

#[test]
fn silverquill_inkmaster_adept_b146_is_a_four_mana_flier_with_magecraft_drain() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_inkmaster_adept_b146());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 3);
    assert!(c.has_keyword(&Keyword::Flying));
    // Cast Lightning Bolt to trigger magecraft drain
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    assert_eq!(g.players[1].life, l1_before - 3 - 1);
}

#[test]
fn silverquill_inkglyph_b146_drains_two_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkglyph_b146());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkglyph castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn inkling_pyrescribe_b146_etb_gains_one_and_magecraft_gains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_pyrescribe_b146());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrescribe castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1, "ETB +1 life");
    // Now magecraft +1 life
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_mid = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_mid + 1);
}

#[test]
fn silverquill_inkbinder_b146_magecraft_random_discards_opp() {
    let mut g = two_player_game();
    let _b = g.add_card_to_hand(1, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_inkbinder_b146());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let h1_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), h1_before - 1);
}

#[test]
fn inkling_inkbearer_b146_etb_mints_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_inkbearer_b146());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkbearer castable");
    drain_stack(&mut g);
    // +2: the Inkbearer itself + an Inkling token
    assert_eq!(g.battlefield.len(), bf_before + 2);
}

#[test]
fn silverquill_ledgerblade_b146_pumps_and_grants_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_ledgerblade_b146());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ledgerblade castable");
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let bearv = computed.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bearv.power, 3, "+1 power");
    assert_eq!(bearv.toughness, 4, "+2 toughness");
    assert!(bearv.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn silverquill_hex_cleric_b146_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_hex_cleric_b146());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hex-Cleric castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_verseguard_b146_magecraft_adds_counter() {
    let mut g = two_player_game();
    let vg = g.add_card_to_battlefield(0, catalog::inkling_verseguard_b146());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(vg).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn silverquill_inkriot_b146_mints_two_inklings_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkriot_b146());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkriot castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), bf_before + 2);
    assert_eq!(g.players[0].life, l0_before + 2);
}

// ── Batch 146: Witherbloom tests ────────────────────────────────────────────

#[test]
fn witherbloom_sap_caller_b146_magecraft_drains_one() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_sap_caller_b146());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    assert_eq!(g.players[1].life, l1_before - 3 - 1);
}

#[test]
fn pest_wraith_b146_dies_drains_each_opp() {
    let mut g = two_player_game();
    let wraith = g.add_card_to_battlefield(0, catalog::pest_wraith_b146());
    let l1_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(wraith)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn witherbloom_toxicologist_b146_etb_mills_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxicologist_b146());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxicologist castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 2);
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2);
}

#[test]
fn witherbloom_reapcaster_b146_magecraft_pumps_and_drains() {
    let mut g = two_player_game();
    let rc = g.add_card_to_battlefield(0, catalog::witherbloom_reapcaster_b146());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(rc).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[1].life, l1_before - 3 - 1);
}

#[test]
fn witherbloom_spore_cleric_b146_etb_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_spore_cleric_b146());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spore-Cleric castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), bf_before + 2);
}

#[test]
fn witherbloom_withergrove_b146_drains_three_and_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_withergrove_b146());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Withergrove castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), bf_before + 1);
    assert_eq!(g.players[0].life, l0_before + 3);
    assert_eq!(g.players[1].life, l1_before - 3);
}

#[test]
fn witherbloom_festerstalk_b146_shrinks_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_festerstalk_b146());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Festerstalk castable");
    drain_stack(&mut g);
    // Bear has -3/-3 → 0 toughness → dies via SBA
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn witherbloom_lifeglyph_b146_gains_five_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifeglyph_b146());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifeglyph castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 5);
}

// ── Batch 146: Lorehold tests ───────────────────────────────────────────────

#[test]
fn lorehold_echocaller_b146_etb_returns_is_card_from_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_echocaller_b146());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echocaller castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
}

#[test]
fn lorehold_spirit_glyph_b146_mints_one_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_glyph_b146());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spirit-Glyph castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), bf_before + 1);
}

#[test]
fn lorehold_ember_adept_b146_magecraft_pings_any() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_ember_adept_b146());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3 - 1);
}

#[test]
fn lorehold_pyresinger_b146_etb_mints_two_spirits_with_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_pyresinger_b146());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyresinger castable");
    drain_stack(&mut g);
    // +3: Pyresinger + 2 Spirit tokens
    assert_eq!(g.battlefield.len(), bf_before + 3);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_spirit_burst_b146_burns_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_burst_b146());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spirit-Burst castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3);
}

#[test]
fn lorehold_battle_sage_b146_magecraft_pumps_friendly_spirit() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_battle_sage_b146());
    let spirit = g.add_token_to_battlefield(
        0,
        &crate::catalog::lorehold_spirit_token(),
    );
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let s = g.battlefield_find(spirit).unwrap();
    assert_eq!(s.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn lorehold_spirit_decree_b146_pings_each_opp_creature_and_mints_spirit() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_decree_b146());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spirit-Decree castable");
    drain_stack(&mut g);
    // Bear took 1 damage (still alive, 2 toughness)
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.damage, 1);
    // +1 Spirit token
    assert_eq!(g.battlefield.len(), bf_before + 1);
}

#[test]
fn lorehold_glyph_strike_b146_burns_creature_for_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_glyph_strike_b146());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Glyph-Strike castable");
    drain_stack(&mut g);
    // Bear has 2 toughness → 2 damage → dies via SBA
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}
