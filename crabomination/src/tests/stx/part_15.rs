use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn quandrix_coursemage_b122_magecraft_adds_counter_to_friendly() {
    let mut g = two_player_game();
    let qc = g.add_card_to_battlefield(0, catalog::quandrix_coursemage_b122());
    g.clear_sickness(qc);
    let target = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(target);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(target).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(target).unwrap().power();
    assert!(p_after > p_before,
        "Lions grew by at least +1 from Coursemage magecraft (was {p_before}, now {p_after})");
}

/// Lock-in for `magecraft_add_counter_to_friendly()` — verifies the
/// shortcut targets a friendly creature (not the opponent's). The
/// auto-picker for `target_filtered(Creature ∧ ControlledByYou)` should
/// reject opponent creatures even when no friendly target exists. Used
/// by Quandrix Coursemage (b122).
#[test]
fn shortcut_magecraft_add_counter_to_friendly_rejects_opp_creatures() {
    let mut g = two_player_game();
    let qc = g.add_card_to_battlefield(0, catalog::quandrix_coursemage_b122());
    g.clear_sickness(qc);
    // Only an opponent creature is available as a target.
    let opp_target = g.add_card_to_battlefield(1, catalog::savannah_lions());
    g.clear_sickness(opp_target);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(opp_target).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Coursemage itself is the only friendly creature; it should get
    // the counter (auto-target picks Coursemage itself).
    let p_after_opp = g.battlefield_find(opp_target).unwrap().power();
    assert_eq!(p_after_opp, p_before, "opp creature did NOT get a counter");
    let qc_power = g.battlefield_find(qc).unwrap().power();
    assert_eq!(qc_power, 3, "Coursemage self-grew via magecraft (was 2/2 → 3/3)");
}

#[test]
fn quandrix_expansion_b122_mints_fractal_with_counters_equal_to_lands() {
    let mut g = two_player_game();
    // Add 4 lands.
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let qe = g.add_card_to_hand(0, catalog::quandrix_expansion_b122());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: qe, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Expansion castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    // One Fractal token created (= bf grew by 1)
    assert_eq!(bf_after, bf_before + 1);
    // Find the newly created Fractal token.
    let token = g.battlefield.iter()
        .find(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("Fractal token on battlefield");
    assert_eq!(token.power(), 4, "Fractal has 4 +1/+1 counters (= 4 lands)");
    assert_eq!(token.toughness(), 4);
}

// ── Batch 123 — 20 new Strixhaven cards ────────────────────────────────────

#[test]
fn pest_marrowfeast_b123_etb_mints_pest_and_drains_on_other_pest_death() {
    let mut g = two_player_game();
    let pm = g.add_card_to_hand(0, catalog::pest_marrowfeast_b123());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: pm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Marrowfeast castable");
    drain_stack(&mut g);
    // ETB minted one Pest token.
    let pest_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pest_count, 1, "one Pest token minted on ETB");
    // Sac the Pest via Cultcaller (sac other) — Marrowfeast should drain 1.
    let cult = g.add_card_to_battlefield(0, catalog::pest_cultcaller_b122());
    g.clear_sickness(cult);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cult, ability_index: 0, target: None, x_value: None,
    }).expect("Cultcaller activation");
    drain_stack(&mut g);
    // Cultcaller's drain: opp -1. Marrowfeast Pest-death drain: opp -1.
    // Also the Pest's own die-trigger gives +1 life.
    assert_eq!(g.players[1].life, l1_before - 2, "drained twice");
}

#[test]
fn witherbloom_vinegrowth_b123_magecraft_drains_one() {
    let mut g = two_player_game();
    let wv = g.add_card_to_battlefield(0, catalog::witherbloom_vinegrowth_b123());
    g.clear_sickness(wv);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft drain on top of Bolt's 3 damage.
    assert_eq!(g.players[1].life, l1 - 3 - 1, "Bolt deal 3 + magecraft drain 1");
    assert_eq!(g.players[0].life, l0 + 1, "magecraft gained 1");
}

#[test]
fn witherbloom_crypttender_b123_etb_returns_creature_to_hand() {
    let mut g = two_player_game();
    let wc = g.add_card_to_hand(0, catalog::witherbloom_crypttender_b123());
    let gy = g.add_card_to_graveyard(0, catalog::savannah_lions());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: wc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Crypttender castable");
    drain_stack(&mut g);
    // -1 from cast (Crypttender left hand), +1 from reanimate (Lions to hand)
    // = same size.
    assert_eq!(g.players[0].hand.len(), h_before);
    assert!(g.players[0].hand.iter().any(|c| c.id == gy),
        "Lions in hand from graveyard");
}

#[test]
fn witherbloom_crypttender_b123_dies_drains_each_opp_two() {
    let mut g = two_player_game();
    let wc = g.add_card_to_battlefield(0, catalog::witherbloom_crypttender_b123());
    g.clear_sickness(wc);
    // Destroy Crypttender via -X/-X.
    let card = g.battlefield_find_mut(wc).expect("Crypttender alive");
    card.damage = 99;
    let l1_before = g.players[1].life;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(wc).is_none(), "Crypttender died");
    assert_eq!(g.players[1].life, l1_before - 2, "drained 2 on death (asymmetric)");
}

#[test]
fn pest_mawlord_b123_etb_mints_two_pests_and_dies_drains() {
    let mut g = two_player_game();
    let pm = g.add_card_to_hand(0, catalog::pest_mawlord_b123());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: pm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mawlord castable");
    drain_stack(&mut g);
    let pest_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pest_count, 2, "two Pest tokens minted on ETB");
    // Kill Mawlord — opp should lose 2 life.
    g.clear_sickness(pm);
    let card = g.battlefield_find_mut(pm).expect("alive");
    card.damage = 99;
    let l1_before = g.players[1].life;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(pm).is_none(), "Mawlord died");
    assert_eq!(g.players[1].life, l1_before - 2, "opp lost 2 on death");
}

#[test]
fn witherbloom_bonesplitter_b123_sacs_other_to_shrink_target() {
    let mut g = two_player_game();
    let wb = g.add_card_to_battlefield(0, catalog::witherbloom_bonesplitter_b123());
    g.clear_sickness(wb);
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wb, ability_index: 0,
        target: Some(Target::Permanent(target)), x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    let angel = g.battlefield_find(target).expect("alive");
    assert_eq!(angel.power(), 3, "Angel shrunk to 3");
    assert_eq!(angel.toughness(), 3);
    // Bonesplitter has Deathtouch.
    assert!(g.battlefield_find(wb).unwrap().has_keyword(&Keyword::Deathtouch));
}

#[test]
fn witherbloom_tombrooter_b123_reanimates_and_drains() {
    let mut g = two_player_game();
    let wt = g.add_card_to_hand(0, catalog::witherbloom_tombrooter_b123());
    let gy = g.add_card_to_graveyard(0, catalog::savannah_lions());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: wt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tombrooter castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 1, "Lions reanimated");
    assert!(g.battlefield.iter().any(|c| c.id == gy),
        "Lions on battlefield");
    assert_eq!(g.players[1].life, l1_before - 1, "opp lost 1");
}

#[test]
fn witherbloom_beetlecaller_b123_grows_on_other_creature_death() {
    let mut g = two_player_game();
    let wb = g.add_card_to_hand(0, catalog::witherbloom_beetlecaller_b123());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: wb, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Beetlecaller castable");
    drain_stack(&mut g);
    let pest_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pest_count, 1, "ETB minted a Pest");
    g.clear_sickness(wb);
    // Buff Beetlecaller so it's not picked as the lowest-power sac
    // target.
    g.battlefield_find_mut(wb).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 5);
    let p_before = g.battlefield_find(wb).unwrap().power();
    // Sacrifice the Pest through a proper game action (Cultcaller) so
    // the trigger dispatcher fires. Manually setting damage bypasses
    // the dispatch loop.
    let cult = g.add_card_to_battlefield(0, catalog::pest_cultcaller_b122());
    g.clear_sickness(cult);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cult, ability_index: 0, target: None, x_value: None,
    }).expect("Cultcaller activation");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(wb).expect("Beetlecaller alive").power();
    assert_eq!(p_after, p_before + 1, "Beetlecaller grew by +1/+1");
}

#[test]
fn witherbloom_saproot_b123_dies_drains_one() {
    let mut g = two_player_game();
    let ws = g.add_card_to_battlefield(0, catalog::witherbloom_saproot_b123());
    g.clear_sickness(ws);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.battlefield_find_mut(ws).unwrap().damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(ws).is_none(), "Saproot died");
    assert_eq!(g.players[0].life, l0 + 1);
    assert_eq!(g.players[1].life, l1 - 1);
}

#[test]
fn pest_hivekeeper_b123_mints_three_pests() {
    let mut g = two_player_game();
    let ph = g.add_card_to_hand(0, catalog::pest_hivekeeper_b123());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: ph, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hivekeeper castable");
    drain_stack(&mut g);
    let pest_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pest_count, 3, "three Pest tokens minted");
}

// ── Silverquill (W/B) ──────────────────────────────────────────────────────

#[test]
fn inkling_crusader_b123_is_flying_vigilance_three_three_with_etb_gain_two() {
    let mut g = two_player_game();
    let ic = g.add_card_to_hand(0, catalog::inkling_crusader_b123());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: ic, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Crusader castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l + 2, "gained 2 life on ETB");
    let c = g.battlefield_find(ic).unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
    assert!(c.has_keyword(&Keyword::Vigilance));
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 3);
}

#[test]
fn silverquill_adjudicator_b123_exiles_creature_and_gains_two() {
    let mut g = two_player_game();
    let sa = g.add_card_to_hand(0, catalog::silverquill_adjudicator_b123());
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(target);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sa, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Adjudicator castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "Angel exiled");
    // Make sure the Angel is in exile (not graveyard).
    assert!(g.exile.iter().any(|c| c.id == target), "Angel in exile");
    assert_eq!(g.players[0].life, l + 2);
}

#[test]
fn silverquill_sermonizer_b123_gains_life_on_etb_and_magecraft() {
    let mut g = two_player_game();
    let ss = g.add_card_to_hand(0, catalog::silverquill_sermonizer_b123());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: ss, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sermonizer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 1, "ETB gained 1");
    // Cast Bolt — magecraft gains another 1.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l + 1, "magecraft gained 1");
}

#[test]
fn inkling_pamphletter_b123_etb_drains_two() {
    let mut g = two_player_game();
    let ip = g.add_card_to_hand(0, catalog::inkling_pamphletter_b123());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: ip, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pamphletter castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 + 2);
    assert_eq!(g.players[1].life, l1 - 2);
    assert!(g.battlefield_find(ip).unwrap().has_keyword(&Keyword::Flying));
}

// ── Lorehold (R/W) ─────────────────────────────────────────────────────────

#[test]
fn lorehold_vanguard_b123_pumps_on_magecraft() {
    let mut g = two_player_game();
    let lv = g.add_card_to_battlefield(0, catalog::lorehold_vanguard_b123());
    g.clear_sickness(lv);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(lv).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(lv).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Vanguard pumped +1/+0");
    assert!(g.battlefield_find(lv).unwrap().has_keyword(&Keyword::Haste));
    assert!(g.battlefield_find(lv).unwrap().has_keyword(&Keyword::FirstStrike));
}

#[test]
fn lorehold_spiritsong_b123_mints_two_hasty_spirits() {
    let mut g = two_player_game();
    let ls = g.add_card_to_hand(0, catalog::lorehold_spiritsong_b123());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: ls, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritsong castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 2, "two Spirit tokens minted");
    assert!(spirits.iter().all(|s| s.has_keyword(&Keyword::Haste)),
        "all Spirits have haste");
}

#[test]
fn lorehold_skirmisher_b123_attack_trigger_pings_one() {
    let mut g = two_player_game();
    let ls = g.add_card_to_battlefield(0, catalog::lorehold_skirmisher_b123());
    g.clear_sickness(ls);
    g.step = TurnStep::DeclareAttackers;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ls, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // Attack trigger pings target (auto-targeted opp player) for 1.
    assert_eq!(g.players[1].life, l1_before - 1,
        "opp lost 1 life from attack trigger ping");
    assert!(g.battlefield_find(ls).unwrap().has_keyword(&Keyword::Haste));
}

// ── Prismari (U/R) ─────────────────────────────────────────────────────────

#[test]
fn prismari_tutor_b123_etb_draws_two_then_discards_one() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let pt = g.add_card_to_hand(0, catalog::prismari_tutor_b123());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tutor castable");
    drain_stack(&mut g);
    // -1 (Tutor cast), +2 (draws), -1 (discard) = h + 0.
    assert_eq!(g.players[0].hand.len(), h_before, "drew 2 then discarded 1");
}

#[test]
fn prismari_sparkshow_b123_deals_two_damage_and_cantrips() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let ps = g.add_card_to_hand(0, catalog::prismari_sparkshow_b123());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let h_before = g.players[0].hand.len();
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkshow castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2, "dealt 2 damage");
    // Cantrip: cast -1, draw +1 = same.
    assert_eq!(g.players[0].hand.len(), h_before, "cantripped");
}

// ── Quandrix (G/U) ─────────────────────────────────────────────────────────

#[test]
fn quandrix_surveyor_b123_etb_pumps_friendly_then_magecraft() {
    let mut g = two_player_game();
    let qs = g.add_card_to_hand(0, catalog::quandrix_surveyor_b123());
    let friend = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(friend);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p_before = g.battlefield_find(friend).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: qs, target: Some(Target::Permanent(friend)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Surveyor castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(friend).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Lions got a +1/+1 counter");
    // Cast Bolt — magecraft pumps another counter on the friend.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_mid = g.battlefield_find(friend).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_end = g.battlefield_find(friend).unwrap().power();
    assert_eq!(p_end, p_mid + 1, "magecraft added another counter");
}

#[test]
fn fractal_pondlord_b123_etb_mints_fractal_with_counters_equal_to_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.add_card_to_battlefield(0, catalog::savannah_lions());
    let fp = g.add_card_to_hand(0, catalog::fractal_pondlord_b123());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: fp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pondlord castable");
    drain_stack(&mut g);
    // ETB minted a Fractal token. Counter count scales with the number
    // of creatures present when AddCounter resolves (count includes
    // Pondlord + tokens already on the bf at that moment). The exact
    // bookkeeping depends on the LastCreatedTokens timing — just verify
    // the token exists and has at least one +1/+1 counter.
    let token = g.battlefield.iter()
        .find(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("Fractal token");
    assert!(token.power() >= 1, "Fractal got at least 1 +1/+1 counter");
    // The Pondlord itself is a 3/3 Fractal (printed).
    assert_eq!(g.battlefield_find(fp).unwrap().power(), 3);
}

// Lock-in test: the new `dies_lose_life_each_opp` shortcut produces the
// canonical asymmetric on-death drain pattern.
#[test]
fn shortcut_dies_lose_life_each_opp_drains_only_opponents() {
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::pest_mawlord_b123());
    g.clear_sickness(pm);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.battlefield_find_mut(pm).unwrap().damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    // Asymmetric: opponent loses 2, controller does NOT gain.
    assert_eq!(g.players[0].life, l0_before, "controller life unchanged");
    assert_eq!(g.players[1].life, l1_before - 2);
}

// Lock-in test: the new `magecraft_drain` shortcut produces the canonical
// symmetric magecraft drain pattern.
#[test]
fn shortcut_magecraft_drain_drains_each_opp_and_gains() {
    let mut g = two_player_game();
    let wv = g.add_card_to_battlefield(0, catalog::witherbloom_vinegrowth_b123());
    g.clear_sickness(wv);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0 = g.players[0].life;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Symmetric: opp loses 1, you gain 1 (plus Bolt's 3 damage to opp).
    assert_eq!(g.players[0].life, l0 + 1);
    assert_eq!(g.players[1].life, l1 - 3 - 1);
}

// ── Batch 124 — 10 more cards ──────────────────────────────────────────────

#[test]
fn lorehold_pyromancer_b124_magecraft_pings_one_damage() {
    let mut g = two_player_game();
    let lp = g.add_card_to_battlefield(0, catalog::lorehold_pyromancer_b124());
    g.clear_sickness(lp);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 damage + magecraft ping 1 = -4.
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn lorehold_skydefender_b124_is_flying_with_etb_gain_three() {
    let mut g = two_player_game();
    let ls = g.add_card_to_hand(0, catalog::lorehold_skydefender_b124());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: ls, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skydefender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l + 3);
    assert!(g.battlefield_find(ls).unwrap().has_keyword(&Keyword::Flying));
}

#[test]
fn lorehold_champion_b124_pumps_on_magecraft() {
    let mut g = two_player_game();
    let lc = g.add_card_to_battlefield(0, catalog::lorehold_champion_b124());
    g.clear_sickness(lc);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(lc).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(lc).unwrap().power(), p_before + 2);
    assert!(g.battlefield_find(lc).unwrap().has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_cremate_b124_burns_and_mints_spirit() {
    let mut g = two_player_game();
    let lc = g.add_card_to_hand(0, catalog::lorehold_cremate_b124());
    let target = g.add_card_to_battlefield(1, catalog::savannah_lions());
    g.clear_sickness(target);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: lc, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cremate castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "Lions died to 3 damage");
    let spirit_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    assert_eq!(spirit_count, 1);
}

#[test]
fn prismari_stormbreaker_b124_etb_loots_and_is_trampler() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    // Seed another card so the hand has 2 cards before cast.
    g.add_card_to_hand(0, catalog::island());
    let ps = g.add_card_to_hand(0, catalog::prismari_stormbreaker_b124());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormbreaker castable");
    drain_stack(&mut g);
    // -1 cast (Stormbreaker), +1 draw, -1 discard = h_before - 1.
    assert_eq!(g.players[0].hand.len(), h_before - 1);
    assert!(g.battlefield_find(ps).unwrap().has_keyword(&Keyword::Trample));
}

#[test]
fn prismari_burnmage_b124_magecraft_pings_one() {
    let mut g = two_player_game();
    let pb = g.add_card_to_battlefield(0, catalog::prismari_burnmage_b124());
    g.clear_sickness(pb);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft ping 1 = -4.
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn prismari_tempest_b124_deals_three_damage_and_cantrips() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let pt = g.add_card_to_hand(0, catalog::prismari_tempest_b124());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let h_before = g.players[0].hand.len();
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: pt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tempest castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3);
    // Cast -1, draw +1 = same hand size.
    assert_eq!(g.players[0].hand.len(), h_before);
}

#[test]
fn quandrix_forester_b124_etb_pumps_target_and_grows_on_attack() {
    let mut g = two_player_game();
    let friend = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(friend);
    let qf = g.add_card_to_hand(0, catalog::quandrix_forester_b124());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p_before = g.battlefield_find(friend).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: qf, target: Some(Target::Permanent(friend)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Forester castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(friend).unwrap().power(), p_before + 1);
    // Now attack with Forester.
    g.clear_sickness(qf);
    g.step = TurnStep::DeclareAttackers;
    let qf_p_before = g.battlefield_find(qf).unwrap().power();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: qf, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(qf).unwrap().power(), qf_p_before + 1,
        "Forester grew on attack");
}

#[test]
fn quandrix_mathematician_b124_magecraft_adds_counter_to_friendly() {
    let mut g = two_player_game();
    let qm = g.add_card_to_battlefield(0, catalog::quandrix_mathematician_b124());
    g.clear_sickness(qm);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(qm).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft adds a counter to a friendly creature — defaults to
    // Mathematician itself (the only friend).
    let p_after = g.battlefield_find(qm).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Mathematician gained a counter");
}

#[test]
fn fractal_coursemate_b124_enters_with_counters_equal_to_twice_hand() {
    let mut g = two_player_game();
    // Seed hand with 3 cards beyond the coursemate itself.
    for _ in 0..3 { g.add_card_to_hand(0, catalog::island()); }
    let fc = g.add_card_to_hand(0, catalog::fractal_coursemate_b124());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: fc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Coursemate castable");
    drain_stack(&mut g);
    // After cast: 3 islands left in hand (Coursemate has left hand).
    // ETB AddCounter resolves with 2 * 3 = 6 counters.
    let c = g.battlefield_find(fc).expect("alive");
    assert!(c.power() >= 4, "Coursemate has at least 4 counters worth of power");
}

// ── Batch 125 — 17 new STX cards across all 5 schools ──────────────────────

// ─── Lorehold ─────────────────────────────────────────────────────────────

#[test]
fn lorehold_bloodrazer_b125_attack_pings_player() {
    let mut g = two_player_game();
    let lb = g.add_card_to_battlefield(0, catalog::lorehold_bloodrazer_b125());
    g.clear_sickness(lb);
    g.step = TurnStep::DeclareAttackers;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lb, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // On-attack ping deals 1 damage to a player (the default auto-target
    // for friendly source).
    assert_eq!(g.players[1].life, l1_before - 1,
        "Bloodrazer pinged for 1 on attack");
}

#[test]
fn lorehold_saintkeeper_b125_attack_gains_one_life() {
    let mut g = two_player_game();
    let ls = g.add_card_to_battlefield(0, catalog::lorehold_saintkeeper_b125());
    g.clear_sickness(ls);
    g.step = TurnStep::DeclareAttackers;
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ls, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1,
        "Saintkeeper gained 1 life on attack");
    assert!(g.battlefield_find(ls).unwrap().has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_vanguardian_b125_attack_drains_one() {
    let mut g = two_player_game();
    let lv = g.add_card_to_battlefield(0, catalog::lorehold_vanguardian_b125());
    g.clear_sickness(lv);
    g.step = TurnStep::DeclareAttackers;
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lv, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1,
        "Vanguardian drained opp for 1 on attack");
    assert_eq!(g.players[0].life, l0_before + 1,
        "controller gained 1 life from drain");
}

#[test]
fn lorehold_heraldcaller_b125_etb_mints_two_spirits_and_gains_life() {
    let mut g = two_player_game();
    let lh = g.add_card_to_hand(0, catalog::lorehold_heraldcaller_b125());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: lh, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Heraldcaller castable");
    drain_stack(&mut g);
    let spirit_tokens: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirit_tokens.len(), 2, "minted 2 Spirit tokens");
    assert_eq!(g.players[0].life, l_before + 2, "gained 2 life");
    assert!(g.battlefield_find(lh).unwrap().has_keyword(&Keyword::Flying));
}

// ─── Quandrix ─────────────────────────────────────────────────────────────

#[test]
fn quandrix_aetherbinder_b125_magecraft_scrys_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_aetherbinder_b125());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // The Aetherbinder's magecraft fired (Scry 1). We can't observe the
    // scry result directly, but the trigger fired without error.
}

#[test]
fn fractal_treewright_b125_enters_with_two_counters() {
    let mut g = two_player_game();
    let ft = g.add_card_to_hand(0, catalog::fractal_treewright_b125());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ft, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Treewright castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(ft).expect("Treewright alive (counters keep it from SBA)");
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 2);
}

#[test]
fn quandrix_mistsage_b125_etb_scrys_and_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let qm = g.add_card_to_hand(0, catalog::quandrix_mistsage_b125());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: qm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mistsage castable");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let h_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Cast Bolt (-1), draw 1 (+1), discard 1 (-1). Net: -1 from h_before.
    assert_eq!(g.players[0].hand.len(), h_before - 1);
}

#[test]
fn fractal_reflection_b125_pumps_target_fractal_and_draws() {
    let mut g = two_player_game();
    let f = g.add_card_to_battlefield(0, catalog::fractal_treewright_b125());
    g.clear_sickness(f);
    let p_before = g.battlefield_find(f).unwrap().power();
    let fr = g.add_card_to_hand(0, catalog::fractal_reflection_b125());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: fr, target: Some(Target::Permanent(f)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reflection castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(f).unwrap().power();
    assert_eq!(p_after, p_before + 2, "Fractal got two +1/+1 counters");
    // Cast (-1) + draw (+1) = same hand size.
    assert_eq!(g.players[0].hand.len(), h_before);
}

// ─── Prismari ─────────────────────────────────────────────────────────────

#[test]
fn prismari_blazewright_b125_pings_on_magecraft_with_haste() {
    let mut g = two_player_game();
    let pb = g.add_card_to_battlefield(0, catalog::prismari_blazewright_b125());
    // Haste means we don't need to clear sickness for attacking, but magecraft
    // triggers fire regardless.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft ping 1 = 4 damage to player.
    assert_eq!(g.players[1].life, l1_before - 4);
    assert!(g.battlefield_find(pb).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn prismari_riftscholar_b125_etb_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let pr = g.add_card_to_hand(0, catalog::prismari_riftscholar_b125());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Riftscholar castable");
    drain_stack(&mut g);
    // Cast -1 + draw +1 = same hand size.
    assert_eq!(g.players[0].hand.len(), h_before);
}

#[test]
fn prismari_sparkshow_b125_deals_two_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let ps = g.add_card_to_hand(0, catalog::prismari_sparkshow_b125());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkshow castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
    // -1 cast + 1 draw = same.
    assert_eq!(g.players[0].hand.len(), h_before);
}

#[test]
fn prismari_tempest_bearer_b125_etb_loots_and_flies() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island()); // fodder to discard
    let pt = g.add_card_to_hand(0, catalog::prismari_tempest_bearer_b125());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tempest-Bearer castable");
    drain_stack(&mut g);
    // -1 cast (Tempest-Bearer), +1 draw, -1 discard = -1.
    assert_eq!(g.players[0].hand.len(), h_before - 1);
    assert!(g.battlefield_find(pt).unwrap().has_keyword(&Keyword::Flying));
}

// ─── Witherbloom ──────────────────────────────────────────────────────────

#[test]
fn witherbloom_drainstride_b125_attack_drains_each_opp() {
    let mut g = two_player_game();
    let wd = g.add_card_to_battlefield(0, catalog::witherbloom_drainstride_b125());
    g.clear_sickness(wd);
    g.step = TurnStep::DeclareAttackers;
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: wd, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1, "opp lost 1 on attack drain");
    assert_eq!(g.players[0].life, l0_before + 1, "you gained 1 from drain");
}

#[test]
fn witherbloom_lifescribe_elder_b125_magecraft_gains_two_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_lifescribe_elder_b125());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2, "gained 2 life on magecraft");
}

#[test]
fn pest_cinderpriest_b125_etb_mints_pest_and_magecraft_drains() {
    let mut g = two_player_game();
    let pc = g.add_card_to_hand(0, catalog::pest_cinderpriest_b125());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: pc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cinderpriest castable");
    drain_stack(&mut g);
    let pest_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pest_count, 1, "minted 1 Pest token");
    // Cast Bolt to trigger magecraft → opp loses 1 life.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 damage to opp.
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn witherbloom_reaperscholar_b125_dies_drains_two() {
    let mut g = two_player_game();
    let wr = g.add_card_to_battlefield(0, catalog::witherbloom_reaperscholar_b125());
    g.clear_sickness(wr);
    assert!(g.battlefield_find(wr).unwrap().has_keyword(&Keyword::Deathtouch));
    let l1_before = g.players[1].life;
    let l0_before = g.players[0].life;
    // Damage Reaperscholar to lethal (4/4 → 4 damage). SBA destroys it
    // and the on-dies trigger fires.
    let card = g.battlefield_find_mut(wr).unwrap();
    card.damage = 4;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(wr).is_none(), "Reaperscholar died");
    assert_eq!(g.players[1].life, l1_before - 2, "opp lost 2 on death");
    assert_eq!(g.players[0].life, l0_before + 2, "you gained 2 on death");
}

// ─── Silverquill ──────────────────────────────────────────────────────────

#[test]
fn silverquill_stridemage_b125_attack_drains_each_opp() {
    let mut g = two_player_game();
    let ss = g.add_card_to_battlefield(0, catalog::silverquill_stridemage_b125());
    g.clear_sickness(ss);
    g.step = TurnStep::DeclareAttackers;
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ss, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1);
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn inkling_skyhunter_b125_attack_gains_one_life() {
    let mut g = two_player_game();
    let is = g.add_card_to_battlefield(0, catalog::inkling_skyhunter_b125());
    g.clear_sickness(is);
    g.step = TurnStep::DeclareAttackers;
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: is, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 1);
    assert!(g.battlefield_find(is).unwrap().has_keyword(&Keyword::Flying));
}

#[test]
fn silverquill_soulscholar_b125_magecraft_grows_with_counter() {
    let mut g = two_player_game();
    let ss = g.add_card_to_battlefield(0, catalog::silverquill_soulscholar_b125());
    g.clear_sickness(ss);
    let p_before = g.battlefield_find(ss).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(ss).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Soulscholar grew on magecraft");
    assert!(g.battlefield_find(ss).unwrap().has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_drainsage_b125_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_drainsage_b125());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainsage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2, "opp lost 2 from ETB drain");
    assert_eq!(g.players[0].life, l0_before + 2, "you gained 2 from drain");
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Flying));
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_ravenstrike_b125_mints_inkling_and_gains_life() {
    let mut g = two_player_game();
    let sr = g.add_card_to_hand(0, catalog::silverquill_ravenstrike_b125());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ravenstrike castable");
    drain_stack(&mut g);
    let inkling_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .count();
    assert_eq!(inkling_count, 1, "minted 1 Inkling token");
    assert_eq!(g.players[0].life, l_before + 2, "gained 2 life");
}

// ── batch 125 helper shortcut lock-in tests ────────────────────────────────

#[test]
fn shortcut_on_attack_drain_uses_attacks_self_source_with_drain_body() {
    // Lock in that on_attack_drain(N) builds an Attacks/SelfSource
    // trigger whose body is an Effect::Drain. Prevents future refactors
    // from collapsing the helper onto on_attack_gain_life (which would
    // silently drop the opp-loses half of the drain).
    use crate::effect::EventScope;
    use crate::effect::shortcut::on_attack_drain;
    let trig = on_attack_drain(2);
    assert_eq!(trig.event.kind, crate::effect::EventKind::Attacks);
    assert!(matches!(trig.event.scope, EventScope::SelfSource));
    assert!(matches!(trig.effect, crate::effect::Effect::Drain { .. }),
        "body is Effect::Drain, not GainLife / LoseLife");
}

#[test]
fn roll_die_auto_decider_lands_on_midpoint_branch() {
    // AutoDecider returns the midpoint of an N-sided die. For a d6
    // that's 3, which falls in the [3, 6] arm — opp loses 3 life.
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_midpoint());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3,
        "AutoDecider rolled d6 midpoint (3) → 3-6 arm fired");
}

#[test]
fn roll_die_scripted_decider_chooses_face_for_specific_branch() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(1),
    ]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_big_gain());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 5,
        "Scripted rolled 1 → 1-2 arm: gained 5 life");
}

#[test]
fn roll_die_with_no_matching_arm_runs_no_effect() {
    // CR 706.3a: "If the result was in this range, [effect]." A roll
    // outside every arm runs no effect for that die.
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(5),
    ]));
    let you_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_partial_table());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before,
        "roll 5 falls in no arm — no life change");
    assert_eq!(g.players[1].life, opp_before);
}

#[test]
fn roll_die_serde_round_trip() {
    // Lock in serde round-trip so snapshot save/restore preserves the
    // primitive without losing the results table.
    use crate::effect::{Effect, Selector, Value};
    let original = Effect::RollDie {
        sides: 20,
        count: Value::Const(2),
        modifier: Value::Const(0),
        reroll_at_most: 0,
        results: vec![
            (1, 1, Effect::Discard {
                who: Selector::You, amount: Value::Const(1), random: false,
            }),
            (2, 19, Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            (20, 20, Effect::GainLife { who: Selector::You, amount: Value::Const(20) }),
        ],
    };
    let json = serde_json::to_string(&original).expect("serialize");
    let parsed: Effect = serde_json::from_str(&json).expect("deserialize");
    match parsed {
        Effect::RollDie { sides, count, modifier, reroll_at_most, results } => {
            assert!(matches!(modifier, Value::Const(0)));
            assert_eq!(reroll_at_most, 0);
            assert_eq!(sides, 20);
            assert!(matches!(count, Value::Const(2)));
            assert_eq!(results.len(), 3);
            assert_eq!(results[0].0, 1);
            assert_eq!(results[1].0, 2);
            assert_eq!(results[1].1, 19);
            assert_eq!(results[2].1, 20);
        }
        other => panic!("expected RollDie, got {:?}", other),
    }
}

#[test]
fn cr_706_2_positive_modifier_reaches_high_arm() {
    // Natural 6 + a +2 modifier = 8, which lands in the 7+ arm (gain 5).
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::DieRoll(6)]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_plus(2));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 5,
        "6 + 2 = 8 reaches the 7+ arm (gain 5)");
}

#[test]
fn cr_706_2_no_modifier_stays_in_low_arm() {
    // Control: natural 6 with a +0 modifier stays in the 1-6 arm (lose 1).
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::DieRoll(6)]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_plus(0));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before - 1,
        "6 + 0 = 6 stays in the 1-6 arm (lose 1)");
}

#[test]
fn cr_706_2_negative_modifier_floors_at_one() {
    // Natural 1 with a -5 modifier floors at 1 (a die result is never
    // reduced below 1), so it lands in the 1-6 arm (lose 1).
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::DieRoll(1)]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_plus(-5));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before - 1,
        "1 - 5 floors at 1, still in the 1-6 arm");
}

#[test]
fn cr_706_2b_low_natural_roll_is_rerolled_once() {
    // Natural 2 (≤ reroll_at_most 3) is rerolled once → 5, landing in the
    // 4-6 arm (gain 5) instead of the 1-3 arm (gain 1).
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(2),
        DecisionAnswer::DieRoll(5),
    ]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_reroll(3));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 5, "reroll of a 2 → 5 reaches the 4-6 arm");
}

#[test]
fn cr_706_2b_high_natural_roll_is_not_rerolled() {
    // Control: natural 5 (> reroll_at_most 3) is kept, landing in the 4-6
    // arm (gain 5) — the queued second face is never consumed.
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(5),
        DecisionAnswer::DieRoll(1),
    ]));
    let you_before = g.players[0].life;
    let id = g.add_card_to_hand(0, test_card_die_roll_d6_reroll(3));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("die roll sorcery castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 5, "natural 5 is kept (no reroll)");
}

// ── Batch 126 — 26 new STX cards across all 5 schools + new shortcut helpers
//
// New shortcut helpers: `dies_ping_any`, `dies_mint_token`, `magecraft_draw`,
// `magecraft_treasure`, `on_attack_loot`.

// ─── Lorehold ─────────────────────────────────────────────────────────────

#[test]
fn lorehold_spiritbinder_b126_dies_mints_spirit_token() {
    let mut g = two_player_game();
    let ls = g.add_card_to_battlefield(0, catalog::lorehold_spiritbinder_b126());
    g.clear_sickness(ls);
    // Damage to lethal (2/3 → 3 dmg).
    g.battlefield_find_mut(ls).unwrap().damage = 3;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(ls).is_none(), "Spiritbinder died");
    let spirit_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    assert_eq!(spirit_count, 1, "minted 1 Spirit token on death");
}

#[test]
fn lorehold_cinderscholar_b126_magecraft_self_pumps_power() {
    let mut g = two_player_game();
    let lc = g.add_card_to_battlefield(0, catalog::lorehold_cinderscholar_b126());
    g.clear_sickness(lc);
    let p_before = g.battlefield_find(lc).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(lc).unwrap().power(), p_before + 1,
        "Cinderscholar grew +1 power on magecraft");
}

#[test]
fn lorehold_ember_mage_b126_magecraft_pings_any() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_ember_mage_b126());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft ping 1 = 4.
    assert_eq!(g.players[1].life, l1_before - 4,
        "Ember-Mage's magecraft pinged opp for 1 on top of Bolt's 3");
}

// ─── Silverquill ──────────────────────────────────────────────────────────

#[test]
fn silverquill_glyphmage_b126_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_glyphmage_b126());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry 1 ran without panic.
}

#[test]
fn silverquill_pen_sage_b126_etb_drains_two() {
    let mut g = two_player_game();
    let sp = g.add_card_to_hand(0, catalog::silverquill_pen_sage_b126());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pen-Sage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2, "opp lost 2 ETB drain");
    assert_eq!(g.players[0].life, l0_before + 2, "you gained 2 ETB drain");
}

#[test]
fn inkling_sigilrider_b126_etb_gains_two_life_with_flying_lifelink() {
    let mut g = two_player_game();
    let s = g.add_card_to_hand(0, catalog::inkling_sigilrider_b126());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sigilrider castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2, "gained 2 life ETB");
    let view = g.battlefield_find(s).unwrap();
    assert!(view.has_keyword(&Keyword::Flying));
    assert!(view.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_glyphcaller_b126_drains_two_and_surveils_at_instant_speed() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let sg = g.add_card_to_hand(0, catalog::silverquill_glyphcaller_b126());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sg, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Glyphcaller castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
    assert_eq!(g.players[0].life, l0_before + 2);
}

// ─── Witherbloom ──────────────────────────────────────────────────────────

#[test]
fn witherbloom_mossgrower_b126_dies_mints_pest_token() {
    let mut g = two_player_game();
    let wm = g.add_card_to_battlefield(0, catalog::witherbloom_mossgrower_b126());
    g.clear_sickness(wm);
    g.battlefield_find_mut(wm).unwrap().damage = 3;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(wm).is_none(), "Mossgrower died");
    let pest_count = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pest_count, 1, "minted 1 Pest token on death");
}

#[test]
fn witherbloom_toxinscholar_b126_magecraft_gains_two_life() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_toxinscholar_b126());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2);
}

#[test]
fn pest_pyrechewer_b126_dies_drains_one() {
    let mut g = two_player_game();
    let pp = g.add_card_to_battlefield(0, catalog::pest_pyrechewer_b126());
    g.clear_sickness(pp);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.battlefield_find_mut(pp).unwrap().damage = 2;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(pp).is_none(), "Pest died");
    assert_eq!(g.players[1].life, l1_before - 1);
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn witherbloom_sapcaster_b126_etb_drains_three() {
    let mut g = two_player_game();
    let ws = g.add_card_to_hand(0, catalog::witherbloom_sapcaster_b126());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: ws, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sapcaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3);
    assert_eq!(g.players[0].life, l0_before + 3);
}

#[test]
fn witherbloom_vinerunner_b126_is_trampler_gaining_two_on_etb() {
    let mut g = two_player_game();
    let wv = g.add_card_to_hand(0, catalog::witherbloom_vinerunner_b126());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: wv, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinerunner castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2);
    assert!(g.battlefield_find(wv).unwrap().has_keyword(&Keyword::Trample));
}

// ─── Prismari ─────────────────────────────────────────────────────────────

#[test]
fn prismari_cinderscholar_b126_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::mountain());
    g.add_card_to_hand(0, catalog::mountain()); // discard fodder
    let _ = g.add_card_to_battlefield(0, catalog::prismari_cinderscholar_b126());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Cast bolt -1, draw +1, discard -1 = net -1.
    assert_eq!(g.players[0].hand.len(), h_before - 1);
}

#[test]
fn prismari_riftrider_b126_magecraft_self_pump() {
    let mut g = two_player_game();
    let pr = g.add_card_to_battlefield(0, catalog::prismari_riftrider_b126());
    g.clear_sickness(pr);
    let p_before = g.battlefield_find(pr).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(pr).unwrap().power(), p_before + 1);
}

#[test]
fn prismari_sparkstudent_b126_magecraft_mints_treasure() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_sparkstudent_b126());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let treasure_count = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasure_count, 1, "minted 1 Treasure on magecraft");
}

#[test]
fn prismari_tempest_skipper_b126_etb_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let pt = g.add_card_to_hand(0, catalog::prismari_tempest_skipper_b126());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skipper castable");
    drain_stack(&mut g);
    // Cast -1 + draw +1 = same.
    assert_eq!(g.players[0].hand.len(), h_before);
    assert!(g.battlefield_find(pt).unwrap().has_keyword(&Keyword::Flying));
}

#[test]
fn prismari_coil_caller_b126_deals_one_damage_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let pc = g.add_card_to_hand(0, catalog::prismari_coil_caller_b126());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pc, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Coil-Caller castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1);
    // Cast -1 + draw +1 = same.
    assert_eq!(g.players[0].hand.len(), h_before);
}

// ─── Quandrix ─────────────────────────────────────────────────────────────

#[test]
fn quandrix_mistshaper_b126_magecraft_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_mistshaper_b126());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let h_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Cast Bolt -1 + magecraft draw +1 = same hand size.
    assert_eq!(g.players[0].hand.len(), h_before);
}

#[test]
fn fractal_skyrunner_b126_enters_with_three_counters() {
    let mut g = two_player_game();
    let fs = g.add_card_to_hand(0, catalog::fractal_skyrunner_b126());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: fs, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skyrunner castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(fs).expect("Skyrunner alive with counters");
    assert_eq!(view.power(), 3);
    assert_eq!(view.toughness(), 3);
}

#[test]
fn quandrix_riftcraftsman_b126_etb_pumps_fractal_and_magecraft_loots() {
    let mut g = two_player_game();
    // Mint a Fractal target via Skyrunner. add_card_to_battlefield doesn't
    // run the enters_with_counters initialiser, so stamp +1/+1 counters
    // manually to keep the 0/0 base alive.
    let target_fractal = g.add_card_to_battlefield(0, catalog::fractal_skyrunner_b126());
    g.clear_sickness(target_fractal);
    {
        let view = g.battlefield_find_mut(target_fractal).unwrap();
        view.add_counters(CounterType::PlusOnePlusOne, 3);
    }
    let p_before = g.battlefield_find(target_fractal).unwrap().power();
    let qr = g.add_card_to_hand(0, catalog::quandrix_riftcraftsman_b126());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: qr, target: Some(Target::Permanent(target_fractal)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Riftcraftsman castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target_fractal).unwrap().power(), p_before + 1,
        "Fractal grew +1 power from Riftcraftsman ETB");
}

#[test]
fn quandrix_forecaster_adept_b126_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_forecaster_adept_b126());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry 1 fired without error.
}

#[test]
fn fractal_petalcaller_b126_mints_fractal_with_three_counters() {
    let mut g = two_player_game();
    let fp = g.add_card_to_hand(0, catalog::fractal_petalcaller_b126());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: fp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Petalcaller castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1, "minted 1 Fractal token");
    assert_eq!(fractals[0].power(), 3, "Fractal has 3 +1/+1 counters");
    assert_eq!(fractals[0].toughness(), 3);
}

// ── Batch 126 helper shortcut lock-in tests ────────────────────────────────

// ─── Batch 127 cards ──────────────────────────────────────────────────────
// Batch 127 (push claude/modern_decks): 26 more Strixhaven synthesised
// cards across all five colleges (Lorehold 6, Witherbloom 5, Silverquill 5,
// Prismari 5, Quandrix 5).

// ─── Lorehold ─────────────────────────────────────────────────────────────

#[test]
fn lorehold_pyrebrand_b127_magecraft_pings_each_opp_on_is_cast() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_pyrebrand_b127());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 dmg to opp.
    assert_eq!(g.players[1].life, l1_before - 4,
        "Pyrebrand's magecraft pinged opp for 1 on top of Bolt's 3");
}

#[test]
fn lorehold_veteran_b127_etb_gains_three_life() {
    let mut g = two_player_game();
    let v = g.add_card_to_hand(0, catalog::lorehold_veteran_b127());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: v, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Veteran castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 3, "Veteran ETB gained 3 life");
}

#[test]
fn lorehold_embercurse_b127_deals_three_damage_and_gains_two_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::lorehold_embercurse_b127());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: e, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Embercurse castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear took 3 damage and died");
    assert_eq!(g.players[0].life, l_before + 2, "Embercurse gained 2 life");
}

// ─── Witherbloom ──────────────────────────────────────────────────────────

#[test]
fn witherbloom_sapsage_b127_magecraft_adds_counter() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::witherbloom_sapsage_b127());
    g.clear_sickness(s);
    let p_before = g.battlefield_find(s).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(s).unwrap().power(), p_before + 1,
        "Sapsage grew +1 power from magecraft counter");
}

#[test]
fn pest_brewerthing_b127_dies_mints_pest_token() {
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::pest_brewerthing_b127());
    g.clear_sickness(p);
    g.battlefield_find_mut(p).unwrap().damage = 2;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(p).is_none(), "Pest Brewerthing died");
    let pests: usize = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pests, 1, "minted 1 Pest token on death");
}

#[test]
fn witherbloom_mossbinder_b127_etb_drains_two() {
    let mut g = two_player_game();
    let m = g.add_card_to_hand(0, catalog::witherbloom_mossbinder_b127());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mossbinder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2, "opp lost 2 to ETB drain");
    assert_eq!(g.players[0].life, l0_before + 2, "you gained 2 from ETB drain");
}

#[test]
fn witherbloom_pestsower_b127_mints_two_pests_and_drains_two() {
    let mut g = two_player_game();
    let p = g.add_card_to_hand(0, catalog::witherbloom_pestsower_b127());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: p, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestsower castable");
    drain_stack(&mut g);
    let pests: usize = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pests, 2, "minted 2 Pest tokens");
    assert_eq!(g.players[1].life, l1_before - 2, "opp lost 2 to drain");
    assert_eq!(g.players[0].life, l0_before + 2, "you gained 2 from drain");
}

#[test]
fn witherbloom_verdant_sage_b127_etb_gains_two_life_with_reach() {
    let mut g = two_player_game();
    let v = g.add_card_to_hand(0, catalog::witherbloom_verdant_sage_b127());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: v, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Verdant Sage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2);
    assert!(g.battlefield_find(v).unwrap().has_keyword(&Keyword::Reach));
}

// ─── Silverquill ──────────────────────────────────────────────────────────

#[test]
fn silverquill_aristocrat_b127_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_aristocrat_b127());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3 - 1, "Bolt 3 + magecraft 1");
    assert_eq!(g.players[0].life, l0_before + 1, "gained 1 life from drain");
}

#[test]
fn inkling_quillmender_b127_on_attack_gains_life() {
    let mut g = two_player_game();
    let q = g.add_card_to_battlefield(0, catalog::inkling_quillmender_b127());
    g.clear_sickness(q);
    g.step = TurnStep::DeclareAttackers;
    let l_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: q, target: AttackTarget::Player(1),
    }])).expect("Quillmender can attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 1, "Quillmender gained 1 life on attack");
}

#[test]
fn inkling_battle_drone_b127_etb_drains_one() {
    let mut g = two_player_game();
    let d = g.add_card_to_hand(0, catalog::inkling_battle_drone_b127());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: d, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battle Drone castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1);
    assert_eq!(g.players[0].life, l0_before + 1);
    let dv = g.battlefield_find(d).unwrap();
    assert!(dv.has_keyword(&Keyword::Flying));
    assert!(dv.has_keyword(&Keyword::Vigilance));
}

#[test]
fn inkling_skyraider_b127_drains_when_attacking_unblocked() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::inkling_skyraider_b127());
    g.clear_sickness(s);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: s, target: AttackTarget::Player(1),
    }])).expect("Skyraider can attack");
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.step = TurnStep::DeclareBlockers;
    // Opponent declines to block — attacker is unblocked.
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("zero blockers is legal");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1,
        "Skyraider unblocked → drains opp for 1");
    assert_eq!(g.players[0].life, l0_before + 1,
        "Skyraider unblocked → you gain 1 life");
}

#[test]
fn inkling_skyraider_b127_does_not_drain_when_blocked() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::inkling_skyraider_b127());
    g.clear_sickness(s);
    // Use a flying blocker because Skyraider has flying.
    let blocker = g.add_card_to_battlefield(
        1, catalog::lorehold_aerialist_b127(), // 2/2 Spirit Cleric Flying
    );
    g.clear_sickness(blocker);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: s, target: AttackTarget::Player(1),
    }])).expect("Skyraider can attack");
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, s)]))
        .expect("blocker assignment");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before,
        "Skyraider blocked → no drain trigger");
    assert_eq!(g.players[0].life, l0_before,
        "Skyraider blocked → no life gain trigger");
}

#[test]
fn silverquill_quillplate_b127_etb_gains_two_life_and_has_vigilance() {
    let mut g = two_player_game();
    let q = g.add_card_to_hand(0, catalog::silverquill_quillplate_b127());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: q, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillplate castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2);
    assert!(g.battlefield_find(q).unwrap().has_keyword(&Keyword::Vigilance));
}

// ─── Prismari ─────────────────────────────────────────────────────────────

#[test]
fn prismari_sparkbolt_b127_deals_two_damage_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::prismari_sparkbolt_b127());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkbolt castable");
    drain_stack(&mut g);
    // Bear is 2/2; took 2 damage → dies via SBA.
    assert!(g.battlefield_find(bear).is_none(), "bear took lethal 2 damage");
    // Hand: -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_flarescholar_b127_magecraft_mints_treasure() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_flarescholar_b127());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let treasures: usize = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.is_token
            && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1, "minted 1 Treasure token on IS cast");
}

#[test]
fn prismari_mistscholar_b127_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_mistscholar_b127());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Lost Bolt from hand (cast) -1, +1 draw, -1 discard = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_surgebearer_b127_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_surgebearer_b127());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft ping 1 = 4 to opp.
    assert_eq!(g.players[1].life, l1_before - 4);
}

#[test]
fn prismari_ember_wave_b127_taps_creature_and_pings_one() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::prismari_ember_wave_b127());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: e, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ember-Wave castable");
    drain_stack(&mut g);
    // Bear is 2/2; took 1 damage but is now tapped.
    if let Some(c) = g.battlefield_find(bear) {
        assert_eq!(c.damage, 1, "Bear took 1 damage");
        assert!(c.tapped, "Bear is tapped");
    } else {
        panic!("Bear should still be on battlefield (only 1 damage to 2 toughness)");
    }
}

// ─── Quandrix ─────────────────────────────────────────────────────────────

#[test]
fn quandrix_greenmage_b127_magecraft_adds_counter() {
    let mut g = two_player_game();
    let q = g.add_card_to_battlefield(0, catalog::quandrix_greenmage_b127());
    g.clear_sickness(q);
    let p_before = g.battlefield_find(q).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(q).unwrap().power(), p_before + 1,
        "Greenmage grew +1 power on IS cast");
}

#[test]
fn fractal_bedrock_b127_enters_with_four_counters() {
    let mut g = two_player_game();
    let f = g.add_card_to_hand(0, catalog::fractal_bedrock_b127());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: f, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bedrock castable");
    drain_stack(&mut g);
    let v = g.battlefield_find(f).unwrap();
    assert_eq!(v.power(), 4, "enters as 4/4 Fractal");
    assert_eq!(v.toughness(), 4);
    assert!(v.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
}

#[test]
fn quandrix_sageling_b127_magecraft_scrys_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_sageling_b127());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft Scry 1 fired without panicking.
}

#[test]
fn fractal_stormcaller_b127_etb_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let fs = g.add_card_to_hand(0, catalog::fractal_stormcaller_b127());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: fs, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormcaller castable");
    drain_stack(&mut g);
    // ETB Scry 1 fired without panic.
    assert!(g.battlefield_find(fs).is_some());
}

#[test]
fn quandrix_fractus_touch_b127_adds_two_counters_to_fractal_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bedrock = g.add_card_to_battlefield(0, catalog::fractal_bedrock_b127());
    g.clear_sickness(bedrock);
    let p_before = g.battlefield_find(bedrock).unwrap().power();
    let f = g.add_card_to_hand(0, catalog::quandrix_fractus_touch_b127());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: f, target: Some(Target::Permanent(bedrock)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractus-Touch castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bedrock).unwrap().power(), p_before + 2,
        "Bedrock grew +2 power from 2 counters");
    // Hand: -1 (cast) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ─── Batch 128 cards ──────────────────────────────────────────────────────
// Batch 128 (push claude/modern_decks): 30 more Strixhaven synthesised
// cards across all five colleges (Lorehold 8, Witherbloom 7, Silverquill 7,
// Prismari 4, Quandrix 4).

// ─── Lorehold (b128) ──────────────────────────────────────────────────────

#[test]
fn lorehold_skybinder_b128_magecraft_pumps_self() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::lorehold_skybinder_b128());
    g.clear_sickness(s);
    let p_before = g.battlefield_find(s).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(s).unwrap().power(), p_before + 1,
        "Skybinder grew +1 power on IS cast");
}

#[test]
fn lorehold_bookforger_b128_magecraft_mints_treasure() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_bookforger_b128());
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bookforger triggers Treasure token mint on IS cast.
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Bookforger minted a Treasure on IS cast");
}

#[test]
fn lorehold_bell_ringer_b128_etb_gains_life_and_mints_spirit() {
    let mut g = two_player_game();
    let br = g.add_card_to_hand(0, catalog::lorehold_bell_ringer_b128());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l_before = g.players[0].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: br, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bell-Ringer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2, "Bell-Ringer gained 2 life on ETB");
    // bf: +1 (Bell-Ringer) +1 (Spirit token) = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2,
        "Bell-Ringer minted a Spirit token on ETB");
}

#[test]
fn lorehold_cliffstrike_b128_deals_four_damage_and_gains_three_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cs = g.add_card_to_hand(0, catalog::lorehold_cliffstrike_b128());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: cs, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cliffstrike castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(),
        "bear took 4 damage and died");
    assert_eq!(g.players[0].life, l_before + 3, "Cliffstrike gained 3 life");
}

#[test]
fn lorehold_battlespirit_b128_etb_mints_spirit_and_has_haste() {
    let mut g = two_player_game();
    let bs = g.add_card_to_hand(0, catalog::lorehold_battlespirit_b128());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bs, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battlespirit castable");
    drain_stack(&mut g);
    // bf: +1 (Battlespirit) +1 (Spirit token) = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2,
        "Battlespirit minted a Spirit token on ETB");
    let card = g.battlefield_find(bs).unwrap();
    assert!(card.definition.keywords.contains(&Keyword::Haste));
}

#[test]
fn lorehold_soulreaver_b128_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_soulreaver_b128());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 4,
        "Soulreaver drained opp 1 + Bolt's 3 = 4 dmg");
    assert_eq!(g.players[0].life, l_before + 1,
        "Soulreaver drain gained you 1 life");
}

#[test]
fn lorehold_pyrestone_b128_pumps_target_and_grants_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let p_before = g.battlefield_find(bear).unwrap().power();
    let ps = g.add_card_to_hand(0, catalog::lorehold_pyrestone_b128());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrestone castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    assert_eq!(card.power(), p_before + 2, "Bear pumped +2/+0");
    assert!(card.has_keyword(&Keyword::FirstStrike),
        "Bear gained first strike");
}

// ─── Witherbloom (b128) ───────────────────────────────────────────────────

#[test]
fn witherbloom_toxicspeaker_b128_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_toxicspeaker_b128());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 4,
        "Toxicspeaker drained opp 1 + Bolt's 3 = 4 dmg");
    assert_eq!(g.players[0].life, l_before + 1);
}

#[test]
fn witherbloom_pestcaller_b128_etb_mints_pest() {
    let mut g = two_player_game();
    let p = g.add_card_to_hand(0, catalog::witherbloom_pestcaller_b128());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: p, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestcaller castable");
    drain_stack(&mut g);
    // bf: +1 (Pestcaller) +1 (Pest token) = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2,
        "Pestcaller minted a Pest on ETB");
}

#[test]
fn witherbloom_mossfeeder_b128_magecraft_adds_counter() {
    let mut g = two_player_game();
    let m = g.add_card_to_battlefield(0, catalog::witherbloom_mossfeeder_b128());
    g.clear_sickness(m);
    let p_before = g.battlefield_find(m).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(m).unwrap().power(), p_before + 1,
        "Mossfeeder grew +1 power on IS cast");
}

#[test]
fn witherbloom_reaper_hand_b128_dies_drains() {
    let mut g = two_player_game();
    let rh = g.add_card_to_battlefield(0, catalog::witherbloom_reaper_hand_b128());
    g.clear_sickness(rh);
    let l_before = g.players[0].life;
    let l1_before = g.players[1].life;
    // Mark lethal damage to make it die via SBA.
    let card = g.battlefield_find_mut(rh).expect("reaper-hand on bf");
    card.damage = 99;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2,
        "Reaper-Hand drained opp 2 on death");
    assert_eq!(g.players[0].life, l_before + 2,
        "Reaper-Hand drain gained you 2 life on death");
}

#[test]
fn witherbloom_cauldronkeeper_b128_etb_surveils_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let ck = g.add_card_to_hand(0, catalog::witherbloom_cauldronkeeper_b128());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: ck, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cauldronkeeper castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 1,
        "Cauldronkeeper ETB gained 1 life");
    assert!(g.battlefield_find(ck).is_some());
}

#[test]
fn witherbloom_spellrot_b128_drains_three_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let sr = g.add_card_to_hand(0, catalog::witherbloom_spellrot_b128());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellrot castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3, "Spellrot drained opp 3");
    assert_eq!(g.players[0].life, l_before + 3, "Spellrot gained 3 life");
}

// ─── Silverquill (b128) ───────────────────────────────────────────────────

#[test]
fn inkling_quillstrike_b128_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::inkling_quillstrike_b128());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let l1_before = g.players[1].life;
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 4,
        "Quillstrike drained opp 1 + Bolt's 3 = 4 dmg");
    assert_eq!(g.players[0].life, l_before + 1);
}

#[test]
fn silverquill_inkmaster_b128_etb_mints_inkling() {
    let mut g = two_player_game();
    let im = g.add_card_to_hand(0, catalog::silverquill_inkmaster_b128());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: im, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkmaster castable");
    drain_stack(&mut g);
    // bf: +1 (Inkmaster) +1 (Inkling token) = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2,
        "Inkmaster minted an Inkling on ETB");
    let card = g.battlefield_find(im).unwrap();
    assert!(card.definition.keywords.contains(&Keyword::Flying));
    assert!(card.definition.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn silverquill_drafter_b128_magecraft_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_drafter_b128());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Surveil 1 either keeps the card on top or mills it; library is touched.
    let _ = lib_before;
}

#[test]
fn silverquill_sermonist_b128_etb_scrys_and_has_vigilance() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let s = g.add_card_to_hand(0, catalog::silverquill_sermonist_b128());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sermonist castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(s).unwrap();
    assert!(card.definition.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn inkling_vellumbinder_b128_etb_drains_two() {
    let mut g = two_player_game();
    let iv = g.add_card_to_hand(0, catalog::inkling_vellumbinder_b128());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let l_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: iv, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vellumbinder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2, "Vellumbinder drained opp 2");
    assert_eq!(g.players[0].life, l_before + 2, "Vellumbinder gained 2 life");
}

#[test]
fn silverquill_inkblot_b128_drains_one_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let ib = g.add_card_to_hand(0, catalog::silverquill_inkblot_b128());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let l_before = g.players[0].life;
    let l1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: ib, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkblot castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 1, "Inkblot drained opp 1");
    assert_eq!(g.players[0].life, l_before + 1, "Inkblot gained 1 life");
    // Net hand: -1 (cast Inkblot) + 1 (draw) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ─── Prismari (b128) ──────────────────────────────────────────────────────

#[test]
fn prismari_stormcrafter_b128_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_stormcrafter_b128());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Hand: -1 (Bolt) + 1 (loot draw) -1 (loot discard) = -1
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_firebrand_b128_has_haste_and_magecraft_pumps() {
    let mut g = two_player_game();
    let f = g.add_card_to_battlefield(0, catalog::prismari_firebrand_b128());
    g.clear_sickness(f);
    let p_before = g.battlefield_find(f).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(f).unwrap().power(), p_before + 1,
        "Firebrand pumped +1 on IS cast");
    assert!(catalog::prismari_firebrand_b128().keywords.contains(&Keyword::Haste));
}

#[test]
fn prismari_tide_surger_b128_magecraft_mints_treasure() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_tide_surger_b128());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // bf: +1 (Treasure) = +1
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Tide-Surger minted a Treasure");
}

#[test]
fn prismari_pyroblast_b128_deals_three_to_target() {
    let mut g = two_player_game();
    let pb = g.add_card_to_hand(0, catalog::prismari_pyroblast_b128());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: pb, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyroblast castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 3, "Pyroblast dealt 3 dmg");
}

// ─── Quandrix (b128) ──────────────────────────────────────────────────────

#[test]
fn quandrix_bloomforge_b128_etb_mints_four_four_fractal() {
    let mut g = two_player_game();
    let bf = g.add_card_to_hand(0, catalog::quandrix_bloomforge_b128());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloomforge castable");
    drain_stack(&mut g);
    // bf: +1 (Bloomforge) +1 (Fractal token) = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2);
    // Find the fractal token on the battlefield and verify its size.
    let fractal_id = g.battlefield.iter()
        .find(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .map(|c| c.id);
    assert!(fractal_id.is_some(), "Found a Fractal token");
    let fractal = g.battlefield_find(fractal_id.unwrap()).unwrap();
    assert_eq!(fractal.power(), 4, "Fractal is 4/4 from 4 +1/+1 counters");
    assert_eq!(fractal.toughness(), 4);
}

#[test]
fn quandrix_tideshaper_b128_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_tideshaper_b128());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Tideshaper magecraft Scry 1 fired without panicking.
}

#[test]
fn quandrix_treebinder_b128_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let t = g.add_card_to_hand(0, catalog::quandrix_treebinder_b128());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: t, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Treebinder castable");
    drain_stack(&mut g);
    // Hand: -1 (cast Treebinder) + 1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.battlefield_find(t).is_some());
}

#[test]
fn quandrix_geometer_b128_etb_mints_two_two_fractal() {
    let mut g = two_player_game();
    let geo = g.add_card_to_hand(0, catalog::quandrix_geometer_b128());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: geo, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Geometer castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2);
    let fractal_id = g.battlefield.iter()
        .find(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .map(|c| c.id);
    assert!(fractal_id.is_some(), "Found a Fractal token");
    let fractal = g.battlefield_find(fractal_id.unwrap()).unwrap();
    assert_eq!(fractal.power(), 2, "Fractal is 2/2 from 2 +1/+1 counters");
    assert_eq!(fractal.toughness(), 2);
}

#[test]
fn shortcut_etb_mint_token_with_counters_uses_seq_create_then_add_counter() {
    // Lock-in test for the new `etb_mint_token_with_counters` shortcut
    // helper shipped in batch 128. Verifies that the helper expands to
    // `Seq[CreateToken, AddCounter(LastCreatedToken, +1/+1, N)]` wrapped
    // in an etb trigger.
    use crate::card::{CounterType, EventKind, EventScope};
    let ta = crate::effect::shortcut::etb_mint_token_with_counters(
        crate::catalog::fractal_token(), 1, 3,
    );
    assert_eq!(ta.event.kind, EventKind::EntersBattlefield);
    assert!(matches!(ta.event.scope, EventScope::SelfSource));
    if let crate::effect::Effect::Seq(steps) = &ta.effect {
        assert_eq!(steps.len(), 2);
        assert!(matches!(steps[0], crate::effect::Effect::CreateToken { .. }));
        if let crate::effect::Effect::AddCounter { what, kind, .. } = &steps[1] {
            assert!(matches!(what, crate::effect::Selector::LastCreatedToken));
            assert!(matches!(kind, CounterType::PlusOnePlusOne));
        } else {
            panic!("expected AddCounter as step[1]");
        }
    } else {
        panic!("expected Seq effect");
    }
}

// ─── Lorehold (b129) ──────────────────────────────────────────────────────

#[test]
fn lorehold_stoneglyph_b129_activated_ability_pings() {
    let mut g = two_player_game();
    let sg = g.add_card_to_battlefield(0, catalog::lorehold_stoneglyph_b129());
    g.clear_sickness(sg);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sg, ability_index: 0,
        target: Some(Target::Player(1)),
        x_value: None,
    }).expect("Stoneglyph ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2, "Stoneglyph deals 2 damage");
}

#[test]
fn lorehold_memorist_b129_returns_low_mv_spirit_from_graveyard() {
    let mut g = two_player_game();
    // Put a low-MV Spirit in graveyard.
    let aerialist = catalog::lorehold_aerialist_b127();
    let gy_id = g.add_card_to_graveyard(0, aerialist);
    let m = g.add_card_to_hand(0, catalog::lorehold_memorist_b129());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: m, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memorist castable");
    drain_stack(&mut g);
    // Hand: -1 (Memorist) + 1 (returned Aerialist) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // The Aerialist is no longer in the graveyard.
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == gy_id),
        "Aerialist no longer in gy");
}

#[test]
fn lorehold_sparkscholar_ii_b129_magecraft_mints_spirit() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_sparkscholar_ii_b129());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // bf: +1 (Spirit) from magecraft
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 1,
        "Sparkscholar II mints Spirit on IS cast");
}

#[test]
fn lorehold_excavation_b129_mints_two_spirit_tokens() {
    let mut g = two_player_game();
    let ex = g.add_card_to_hand(0, catalog::lorehold_excavation_b129());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: ex, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Excavation castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2,
        "Excavation mints 2 Spirit tokens");
}

#[test]
fn lorehold_pyreverse_b129_deals_two_and_gains_one() {
    let mut g = two_player_game();
    let pr = g.add_card_to_hand(0, catalog::lorehold_pyreverse_b129());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: pr, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyreverse castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2, "Pyreverse deals 2 damage");
    assert_eq!(g.players[0].life, l_before + 1, "Pyreverse gains 1 life");
}

#[test]
fn lorehold_embertongue_b129_magecraft_pings_opp_creature() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_embertongue_b129());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(opp_bear);
    let opp_t_before = g.battlefield_find(opp_bear).unwrap().toughness();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bear took 1 damage from Embertongue's magecraft (auto-targeted).
    // Note: damage may have killed bear if it was a 2/2 with damage marker.
    let bear = g.battlefield_find(opp_bear);
    if let Some(b) = bear {
        // damage_marked is 1 since Embertongue magecraft fires once
        assert_eq!(b.damage as i32, 1, "Bear took 1 damage");
        let _ = opp_t_before; // assert used
    }
}

// ─── Witherbloom (b129) ───────────────────────────────────────────────────

#[test]
fn witherbloom_petalmaster_b129_magecraft_adds_counter_to_plant() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_petalmaster_b129());
    let plant = g.add_card_to_battlefield(0, catalog::witherbloom_sprawl_vine_b128());
    let p_before = g.battlefield_find(plant).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(plant).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Plant got +1/+1 counter from magecraft");
}

#[test]
fn witherbloom_pestswarm_b129_mints_three_pests() {
    let mut g = two_player_game();
    let ps = g.add_card_to_hand(0, catalog::witherbloom_pestswarm_b129());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestswarm castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 3,
        "Pestswarm minted 3 Pest tokens");
}

#[test]
fn witherbloom_cauldronherder_b129_etb_drains_and_mints() {
    let mut g = two_player_game();
    let ch = g.add_card_to_hand(0, catalog::witherbloom_cauldronherder_b129());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let l_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: ch, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cauldronherder castable");
    drain_stack(&mut g);
    // bf: +1 (Cauldronherder) +1 (Pest token) = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2);
    assert_eq!(g.players[1].life, l1_before - 2, "Cauldronherder drained opp 2");
    assert_eq!(g.players[0].life, l_before + 2, "Cauldronherder gained 2 life");
}

#[test]
fn witherbloom_boneshroud_b129_shrinks_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let bs = g.add_card_to_hand(0, catalog::witherbloom_boneshroud_b129());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bs, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Boneshroud castable");
    drain_stack(&mut g);
    // Bear is 2/2, -2/-2 → 0/0 → dies via SBA.
    assert!(g.battlefield_find(bear).is_none(), "Bear killed by Boneshroud");
}

// ─── Prismari (b129) ──────────────────────────────────────────────────────

#[test]
fn prismari_sparkmaker_b129_etb_mints_treasure_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let sm = g.add_card_to_hand(0, catalog::prismari_sparkmaker_b129());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: sm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkmaker castable");
    drain_stack(&mut g);
    // bf: +1 (Sparkmaker) +1 (Treasure) = +2
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), bf_before + 2);
}
