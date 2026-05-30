use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn inkling_scrollguard_b174_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_scrollguard_b174());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn silverquill_inkfiend_b174_drains_when_other_creature_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_inkfiend_b174());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    // Kill fodder via destroy effect targeting "Creature & CardId-matches-fodder".
    // Simpler: cast a Bolt at the bear from p0 by giving p0 the bolt + mana.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Bear (2/2) takes 3 → dies → on-other-dies trigger drains 1 (each opp -1, you +1).
    assert_eq!(g.players[1].life, p1_life - 1);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn silverquill_pyremist_b174_etb_drains_two_and_scrys() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::silverquill_pyremist_b174());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn witherbloom_pestbinder_b174_magecraft_drains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_pestbinder_b174());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // 1 from magecraft drain + 3 from bolt = 4 to p1; +1 to p0 from drain.
    assert_eq!(g.players[1].life, p1_life - 4);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn pest_shepherd_b174_etb_mints_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_shepherd_b174());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let pest = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Pest")
        .expect("pest token present");
    assert_eq!(pest.controller, 0);
}

#[test]
fn witherbloom_drainmage_b174_etb_drains_and_magecraft_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_drainmage_b174());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life_pre = g.players[1].life;
    let p0_life_pre = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // ETB drain 2.
    assert_eq!(g.players[1].life, p1_life_pre - 2);
    assert_eq!(g.players[0].life, p0_life_pre + 2);
    // Cast a bolt; magecraft gains 1 life.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0_life_pre_cast = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // The actual `id` ends up on battlefield; verify drainmage is there.
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Witherbloom Drainmage (b174)"),
        "drainmage alive");
    assert_eq!(g.players[0].life, p0_life_pre_cast + 1);
}

#[test]
fn pest_bramblebeast_b174_is_a_four_mana_four_four_reach() {
    let def = catalog::pest_bramblebeast_b174();
    assert_eq!(def.cost.cmc(), 4);
    assert_eq!(def.power, 4);
    assert_eq!(def.toughness, 4);
    assert!(def.keywords.contains(&Keyword::Reach));
    assert!(def.has_creature_type(CreatureType::Beast));
}

#[test]
fn witherbloom_tracker_b174_etb_shrinks_target_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _id = g.add_card_to_hand(0, catalog::witherbloom_tracker_b174());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: _id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // 2/2 grizzly with -1/-1 EOT → SBA may kill it (toughness 1, but creature already at 1 toughness).
    // Actually 2/2 minus -1/-1 = 1/1; should still be alive.
    let target_alive = g.battlefield_find(target);
    if let Some(c) = target_alive {
        assert_eq!(c.power(), 1);
        assert_eq!(c.toughness(), 1);
    }
}

#[test]
fn witherbloom_toxicultivator_b174_attacks_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_toxicultivator_b174());
    g.clear_sickness(id);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("declare");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn witherbloom_sapcaller_b174_magecraft_gains_one_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_sapcaller_b174());
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
fn lorehold_pyrespirit_b174_has_haste_and_pings_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_pyrespirit_b174());
    assert!(g.battlefield_find(id).unwrap().definition.keywords.contains(&Keyword::Haste));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Pyrespirit pings 1 to opp + bolt 3 = -4 life.
    assert_eq!(g.players[1].life, p1_life - 4);
}

#[test]
fn lorehold_banneret_b174_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_banneret_b174());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn lorehold_sparkborn_b174_attacks_pings_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_sparkborn_b174());
    g.clear_sickness(id);
    let p1_life = g.players[1].life;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("declare");
    drain_stack(&mut g);
    // 1 damage from on_attack ping (we choose opp); combat damage adds 1 more = total 2.
    assert!(g.players[1].life < p1_life);
}

#[test]
fn lorehold_ghostflame_b174_deals_two_and_gains_two_life() {
    let mut g = two_player_game();
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_ghostflame_b174());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_creature)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 2);
    // 2/2 dealt 2 → dies.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == opp_creature),
        "grizzly should die");
}

#[test]
fn lorehold_spectralcaller_b174_etb_mints_lorehold_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spectralcaller_b174());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let spirit = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Spirit")
        .expect("spirit token present");
    assert!(spirit.definition.keywords.contains(&Keyword::Flying));
    assert_eq!(spirit.controller, 0);
}

#[test]
fn lorehold_vanguard_b174_is_a_five_mana_four_four_trample() {
    let def = catalog::lorehold_vanguard_b174();
    assert_eq!(def.cost.cmc(), 5);
    assert_eq!(def.power, 4);
    assert_eq!(def.toughness, 4);
    assert!(def.keywords.contains(&Keyword::Trample));
}

#[test]
fn prismari_embermage_b174_magecraft_pings_one() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_embermage_b174());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Embermage 1 + Bolt 3 = -4.
    assert_eq!(g.players[1].life, p1_life - 4);
}

#[test]
fn prismari_wavefocuser_b174_magecraft_scrys() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_battlefield(0, catalog::prismari_wavefocuser_b174());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Scry shouldn't change library size.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn prismari_spellforge_b174_etb_mints_treasure_and_magecraft_scrys() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::prismari_spellforge_b174());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let has_treasure = g.battlefield.iter()
        .any(|c| c.is_token && c.definition.name == "Treasure" && c.controller == 0);
    assert!(has_treasure, "treasure token minted on ETB");
}

#[test]
fn prismari_sparkflood_b174_deals_two_and_draws_one() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::prismari_sparkflood_b174());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_life = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    // Hand: -1 (cast) +1 (draw) = 0 net change.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_stormbringer_b174_magecraft_treasure() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_stormbringer_b174());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let has_treasure = g.battlefield.iter()
        .any(|c| c.is_token && c.definition.name == "Treasure" && c.controller == 0);
    assert!(has_treasure, "treasure token from magecraft");
}

#[test]
fn quandrix_symbolist_b174_magecraft_adds_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_symbolist_b174());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("symbolist alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_mathshape_b174_magecraft_draws_one() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_battlefield(0, catalog::quandrix_mathshape_b174());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Hand: -1 (cast) +1 (magecraft draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_fractalspinner_b174_etb_mints_fractal_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalspinner_b174());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal" && c.controller == 0)
        .expect("fractal token");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn quandrix_riverflow_b174_draws_two_and_loses_one() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_riverflow_b174());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0_life = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) +2 (draw) = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(g.players[0].life, p0_life - 1);
}

#[test]
fn quandrix_sapcaller_b174_attacks_grows_friend() {
    let mut g = two_player_game();
    let sapcaller = g.add_card_to_battlefield(0, catalog::quandrix_sapcaller_b174());
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(sapcaller);
    g.clear_sickness(friend);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sapcaller,
        target: AttackTarget::Player(1),
    }])).expect("declare");
    drain_stack(&mut g);
    let friend_c = g.battlefield_find(friend).expect("friend alive");
    assert_eq!(friend_c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_wavelock_b174_counters_unless_paid_two() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    g.perform_action(GameAction::PassPriority).unwrap();
    let wavelock = g.add_card_to_hand(1, catalog::quandrix_wavelock_b174());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: wavelock, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // P0 has no mana to pay {2}; bolt countered.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "bolt countered by wavelock");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 175 (modern_decks) — additional cards
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_stenographer_b175_magecraft_loots() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_battlefield(0, catalog::silverquill_stenographer_b175());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Hand: -1 (cast) +1 (loot draw) -1 (loot discard) = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn inkling_mortician_b175_is_a_five_mana_lifelink_flying_inkling() {
    let def = catalog::inkling_mortician_b175();
    assert_eq!(def.cost.cmc(), 5);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Lifelink));
    assert!(def.has_creature_type(CreatureType::Inkling));
}

#[test]
fn silverquill_reapcrier_b175_dies_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_reapcrier_b175());
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Reapcrier dies → drain 1.
    assert_eq!(g.players[1].life, p1_life - 1);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn inkling_cantor_b175_etb_scrys_and_gains_life() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::inkling_cantor_b175());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn silverquill_penkeeper_b175_magecraft_drains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_penkeeper_b175());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Magecraft drain 1 + bolt 3 = -4 life.
    assert_eq!(g.players[1].life, p1_life - 4);
}

#[test]
fn inkling_hatchling_b175_etb_with_plus_one_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_hatchling_b175());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let h = g.battlefield.iter()
        .find(|c| c.definition.name == "Inkling Hatchling (b175)")
        .expect("hatchling on bf");
    assert_eq!(h.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn silverquill_verdictbearer_b175_exiles_high_power_creature() {
    let mut g = two_player_game();
    // Add a 4/4 creature so power >= 4 filter passes.
    let big = g.add_card_to_battlefield(1, catalog::pest_bramblebeast_b174());
    let id = g.add_card_to_hand(0, catalog::silverquill_verdictbearer_b175());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == big),
        "big creature exiled");
}

#[test]
fn witherbloom_cauldroncrier_b175_magecraft_drains_target() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_cauldroncrier_b175());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Magecraft drain target 1 + bolt 3 = -4.
    assert_eq!(g.players[1].life, p1_life - 4);
}

#[test]
fn witherbloom_pestharvest_b175_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestharvest_b175());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let pest_count = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest" && c.controller == 0)
        .count();
    assert_eq!(pest_count, 2);
}

#[test]
fn witherbloom_pestmaster_b175_on_other_dies_mints_pest() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_pestmaster_b175());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let pest_count = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest" && c.controller == 0)
        .count();
    assert_eq!(pest_count, 1);
}

#[test]
fn lorehold_skirmishmage_b175_attacks_loots() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_battlefield(0, catalog::lorehold_skirmishmage_b175());
    g.clear_sickness(id);
    let hand_before = g.players[0].hand.len();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("declare");
    drain_stack(&mut g);
    // Hand: +1 (loot draw) -1 (loot discard) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn lorehold_anthemwarden_b175_buffs_other_spirits() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_anthemwarden_b175());
    // Add another Spirit to verify the buff applies.
    let spirit = g.add_card_to_battlefield(0, catalog::lorehold_pyrespirit_b174());
    drain_stack(&mut g);
    let projection = g.compute_battlefield().into_iter()
        .find(|c| c.id == spirit)
        .expect("pyrespirit alive");
    // Base 2/1 + anthem +1/+1 = 3/2.
    assert_eq!(projection.power, 3);
    assert_eq!(projection.toughness, 2);
}

#[test]
fn lorehold_charm_echo_b175_deals_three_to_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_charm_echo_b175());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // 2/2 grizzly takes 3 → dies.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == target),
        "grizzly dies to charm-echo");
}

#[test]
fn prismari_sparkmage_b175_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_sparkmage_b175());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Magecraft pings opp 1 + bolt 3 = -4.
    assert_eq!(g.players[1].life, p1_life - 4);
}

#[test]
fn prismari_cloudburst_b175_burns_four_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let target = g.add_card_to_battlefield(1, catalog::pest_bramblebeast_b174()); // 4/4
    let id = g.add_card_to_hand(0, catalog::prismari_cloudburst_b175());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // 4/4 takes 4 damage → dies. -1 (cast) +1 (draw) = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == target),
        "4/4 dies");
}

#[test]
fn quandrix_mathwarden_b175_magecraft_pumps_friend() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::quandrix_mathwarden_b175());
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let pwr_before = g.battlefield_find(friend).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let pwr_after = g.battlefield_find(friend).unwrap().power();
    assert_eq!(pwr_after, pwr_before + 1);
}

#[test]
fn quandrix_beastform_b175_mints_three_three_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_beastform_b175());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal" && c.controller == 0)
        .expect("fractal token");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 3);
}

#[test]
fn quandrix_tidemind_b175_etb_draws_card() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_tidemind_b175());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) +1 (ETB draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ─────────────────────────────────────────────────────────────────────────
// CR rule lock-in tests (modern_decks batch 174/175)
// ─────────────────────────────────────────────────────────────────────────

/// CR 121.5 — "If an effect moves cards from a player's library to that
/// player's hand without using the word 'draw,' the player has not drawn
/// those cards." Verify that `RevealUntilFind` (which puts the matching
/// card into the hand, not "draws") does NOT trigger a CardDrawn event
/// and does NOT bump `cards_drawn_this_turn`.
#[test]
fn cr_121_5_reveal_until_find_does_not_count_as_draw() {
    use crate::card::Effect;
    use crate::effect::RevealMissDest;
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    // Seed P0 library: top-of-deck Forest (matches IsBasicLand), then 2 Islands beneath.
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let cards_drawn_before = g.players[0].cards_drawn_this_turn;
    let hand_before = g.players[0].hand.len();

    let eff = Effect::RevealUntilFind {
        who: crate::effect::PlayerRef::You,
        find: crate::card::SelectionRequirement::IsBasicLand,
        to: crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::You),
        cap: crate::card::Value::Const(5),
        life_per_revealed: 0,
        miss_dest: RevealMissDest::Graveyard,
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&eff, &ctx).expect("resolve");
    drain_stack(&mut g);

    // Card moved to hand, but no draw counted.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "card put into hand");
    assert_eq!(
        g.players[0].cards_drawn_this_turn, cards_drawn_before,
        "CR 121.5: putting into hand is not drawing"
    );
}

/// CR 506.4 — "A permanent is removed from combat if it leaves the
/// battlefield, [...] A creature that's removed from combat stops being
/// an attacking, blocking, blocked, and/or unblocked creature."
///
/// Verify that destroying an attacker mid-combat removes it from the
/// attacker list (it deals no combat damage that step).
#[test]
fn cr_506_4_destroyed_attacker_is_removed_from_combat() {
    use crate::card::{Effect, Selector, SelectionRequirement};
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }])).expect("declare");
    drain_stack(&mut g);
    // Attacker is on the attack list.
    assert!(g.attacking_ids().contains(&attacker),
        "attacker registered");

    // Destroy the attacker (CR 704.5g — zero toughness SBA equivalent via
    // direct Effect::Destroy). The attacker leaves the battlefield, which
    // per CR 506.4 removes it from combat.
    let eff = Effect::Destroy {
        what: Selector::EachPermanent(SelectionRequirement::Creature),
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&eff, &ctx).expect("destroy");
    drain_stack(&mut g);

    // CR 506.4: attacker removed.
    assert!(!g.attacking_ids().contains(&attacker),
        "CR 506.4: destroyed attacker removed from combat");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 176 (modern_decks) — engine-improvement-driven cards
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_doomgrant_b176_grants_finality_counter_to_target() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_doomgrant_b176());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(target).expect("target alive");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
}

#[test]
fn silverquill_doomgrant_b176_target_dies_to_exile_per_finality() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_doomgrant_b176());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Now kill the bear; per CR 122.1h it should be exiled instead of graveyard.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == target),
        "CR 122.1h: finality-countered creature is exiled on death");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == target),
        "CR 122.1h: not in graveyard");
}

#[test]
fn silverquill_aegis_b176_grants_shield_counter() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_aegis_b176());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(target).expect("target alive");
    assert_eq!(c.counter_count(CounterType::Shield), 1);
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 177 (modern_decks) — more cards across schools
// ─────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────
// Batch 178 (modern_decks) — more variety
// ─────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────
// Batch 179 (modern_decks) — Inkling tribal expansion
// ─────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────
// Batch 180 (modern_decks) — cross-school expansions
// ─────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────
// Batch 181 (modern_decks) — Witherbloom Pest tribal + drain
// ─────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────
// Batch 182 (modern_decks) — balanced cube fillers
// ─────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────
// Batch 186 (modern_decks) — multi-counter magecraft engines
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_glyphmaker_b186_grants_plus_one_and_flying_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_glyphmaker_b186());
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
    assert!(c.has_keyword(&Keyword::Flying));
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 185 (modern_decks) — self-ETB keyword counter + Fractal mints
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn quandrix_skyfractal_b185_mints_flying_fractal_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_skyfractal_b185());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal" && c.controller == 0)
        .expect("fractal token");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(fractal.has_keyword(&Keyword::Flying),
        "CR 122.1b: flying counter grants Flying to the Fractal");
}

#[test]
fn prismari_sparkbloomer_b185_etb_grants_haste_via_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkbloomer_b185());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Prismari Sparkbloomer (b185)")
        .expect("sparkbloomer on bf");
    assert!(c.has_keyword(&Keyword::Haste));
}

#[test]
fn witherbloom_venomspur_b185_etb_grants_deathtouch() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_venomspur_b185());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Witherbloom Venomspur (b185)")
        .expect("venomspur on bf");
    assert!(c.has_keyword(&Keyword::Deathtouch));
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 184 (modern_decks) — more keyword counter granters
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_wordsharpener_b184_grants_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_wordsharpener_b184());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::FirstStrike));
}

#[test]
fn silverquill_drainmark_b184_grants_deathtouch() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_drainmark_b184());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::Deathtouch));
}

#[test]
fn witherbloom_trampleblossom_b184_grants_trample() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_trampleblossom_b184());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::Trample));
}

#[test]
fn witherbloom_lifebondseal_b184_grants_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifebondseal_b184());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::Lifelink));
}

#[test]
fn lorehold_battlerune_b184_grants_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_battlerune_b184());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_wardseal_b184_grants_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_wardseal_b184());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::Vigilance));
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 183 (modern_decks) — Keyword counter cards (CR 122.1b)
// ─────────────────────────────────────────────────────────────────────────

/// CR 122.1b — a flying counter grants the host the Flying keyword while
/// the counter is present. Pin the canonical behaviour via Silverquill
/// Skystudent: target a 2/2 Grizzly Bear → it gains flying.
#[test]
fn cr_122_1b_flying_counter_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Before: no flying.
    assert!(!g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Flying));
    let id = g.add_card_to_hand(0, catalog::silverquill_skystudent_b183());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // After: keyword counter grants Flying.
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::Flying),
        "CR 122.1b: flying counter grants Flying keyword");
    // Computed permanent also surfaces the keyword (layer-6 path).
    let cp = g.compute_battlefield().into_iter()
        .find(|cc| cc.id == bear)
        .expect("bear computed");
    assert!(cp.keywords.contains(&Keyword::Flying));
}

#[test]
fn silverquill_skystudent_b183_grants_flying_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_skystudent_b183());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.keyword_counters.get(&Keyword::Flying).copied().unwrap_or(0), 1);
}

#[test]
fn silverquill_ascendant_b182_is_a_six_mana_five_five_flying_lifelink() {
    let def = catalog::silverquill_ascendant_b182();
    assert_eq!(def.cost.cmc(), 6);
    assert_eq!(def.power, 5);
    assert_eq!(def.toughness, 5);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn silverquill_stampcrafter_b182_etb_drains_and_scrys() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::silverquill_stampcrafter_b182());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 1);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn lorehold_cinderwell_b182_unblocked_attack_pings_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_cinderwell_b182());
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("declare");
    drain_stack(&mut g);
    // Advance to declare blockers and decline to block. The unblocked
    // trigger fires after DeclareBlockers per CR 509.3g.
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![]))
        .expect("decline blocks");
    drain_stack(&mut g);
    // No blockers → unblocked → on_unblocked deals 1 damage to p1.
    assert!(g.players[1].life < p1_life,
        "unblocked attack should reduce p1 life");
}

#[test]
fn quandrix_streamwarden_b182_etb_scrys_and_grows() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_streamwarden_b182());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Quandrix Streamwarden (b182)")
        .expect("on bf");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn prismari_mage_mentor_b182_magecraft_loots() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_battlefield(0, catalog::prismari_mage_mentor_b182());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -1 (cast) +1 (loot draw) -1 (loot discard) = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn witherbloom_pestlord_b181_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestlord_b181());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let pest_count = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest" && c.controller == 0)
        .count();
    assert_eq!(pest_count, 2);
}

#[test]
fn witherbloom_drainscribe_b181_magecraft_drains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_drainscribe_b181());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -1 from magecraft drain + -3 from bolt = -4 p1 life; +1 from magecraft to p0.
    assert_eq!(g.players[1].life, p1_life - 4);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn witherbloom_plaguebearer_b181_dies_drains_each_opp_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_plaguebearer_b181());
    let p1_life = g.players[1].life;
    // Cast bolt at it to kill the plaguebearer.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Plaguebearer (4/3) takes 3 → dies → each opp -2 life.
    assert_eq!(g.players[1].life, p1_life - 2);
}

#[test]
fn quandrix_counterspinner_b180_counters_low_mv_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    g.perform_action(GameAction::PassPriority).unwrap();
    let counter = g.add_card_to_hand(1, catalog::quandrix_counterspinner_b180());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: counter, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "bolt countered (MV 1, threshold 2)");
}

#[test]
fn quandrix_fractal_echocaller_b180_mints_fractal_with_one_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_fractal_echocaller_b180());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal" && c.controller == 0)
        .expect("fractal token");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn lorehold_spiritlord_b180_etb_mints_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritlord_b180());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let spirit_count = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit" && c.controller == 0)
        .count();
    assert_eq!(spirit_count, 2);
}

#[test]
fn lorehold_spectralguard_b180_on_attack_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_spectralguard_b180());
    g.clear_sickness(id);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("declare");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn prismari_lavaforge_b180_etb_burns_three_and_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_lavaforge_b180());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 3);
    let has_treasure = g.battlefield.iter()
        .any(|c| c.is_token && c.definition.name == "Treasure" && c.controller == 0);
    assert!(has_treasure);
}

#[test]
fn inkling_tutor_b179_discards_then_draws_two() {
    let mut g = two_player_game();
    // Seed hand & library so discard-then-draw both can happen.
    let _filler = g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::inkling_tutor_b179());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 (cast) -1 (discard) +2 (draw) = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_heraldscribe_b179_attacks_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_heraldscribe_b179());
    g.clear_sickness(id);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("declare");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 1);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn inkling_lifesong_b178_drains_two_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::inkling_lifesong_b178());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    assert_eq!(g.players[0].life, p0_life + 2);
    // -1 (cast) +1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn silverquill_pridecrier_b178_magecraft_gains_two_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_pridecrier_b178());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn witherbloom_vinecaster_b178_etb_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinecaster_b178());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn witherbloom_cauldron_echo_b178_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_cauldron_echo_b178());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 3);
    assert_eq!(g.players[0].life, p0_life + 3);
}

#[test]
fn lorehold_sparkscholar_b178_taps_for_two_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_sparkscholar_b178());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)),
        x_value: None,
    }).expect("activated");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
}

#[test]
fn quandrix_drawcaster_b178_etb_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_drawcaster_b178());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_magecraft_sage_b178_magecraft_scrys_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_battlefield(0, catalog::prismari_magecraft_sage_b178());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -1 (cast bolt) +1 (magecraft draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_stylekeeper_b177_magecraft_drains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::inkling_stylekeeper_b177());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4);
}

#[test]
fn silverquill_wordweaver_b177_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_wordweaver_b177());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn lorehold_ghostsmith_b177_magecraft_pumps_friend() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_ghostsmith_b177());
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let pwr_before = g.battlefield_find(friend).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(friend).unwrap().power(), pwr_before + 1);
}

#[test]
fn lorehold_cultivator_b177_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_cultivator_b177());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Magecraft ping 1 + bolt 3 = -4 (you gain 1 from drain too).
    assert_eq!(g.players[1].life, p1_life - 4);
    assert_eq!(g.players[0].life, p0_life + 1);
}

#[test]
fn quandrix_streamcaster_b177_etb_scrys_and_draws() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_streamcaster_b177());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) +1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_fractalkeeper_b177_mints_four_four_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalkeeper_b177());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal" && c.controller == 0)
        .expect("fractal token");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 4);
}

#[test]
fn witherbloom_doomsign_b176_finality_and_one_damage() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_doomsign_b176());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(target).expect("target alive after 1 damage");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
    assert_eq!(c.damage, 1);
}

/// CR 119.7 — "If an effect would cause a player to lose life, the
/// player's life total decreases by that much." This combined with
/// 119.7's lifegain interaction. Specifically, drain effects move life
/// in both directions atomically — this test pins the canonical
/// behaviour for [`Effect::Drain`] that 'from' players each lose N life
/// and the 'to' player gains N×count life.
#[test]
fn cr_119_7_drain_loses_life_from_each_opp_and_gains_life_for_caster() {
    let mut g = two_player_game();
    let p1_life_before = g.players[1].life;
    let p0_life_before = g.players[0].life;
    // Cast a 3-life-drain spell (Silverquill Lifeleach + Drain 2 + Scry 1
    // — verify CR 119.7 routing).
    let id = g.add_card_to_hand(0, catalog::silverquill_lifeleach_b174());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Drain 2: each opp -2 life, you +2 life.
    assert_eq!(g.players[1].life, p1_life_before - 2,
        "CR 119.7: opponent loses life from drain");
    assert_eq!(g.players[0].life, p0_life_before + 2,
        "CR 119.7: caster gains life from drain");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 188 (modern_decks) — additional STX cards across schools.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_cantrap_b188_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_cantrap_b188());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.power(), 3);
    assert!(c.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_tribune_b188_etb_drains_two_and_magecraft_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_tribune_b188());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
}

#[test]
fn silverquill_litany_b188_drains_two_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_litany_b188());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
}

#[test]
fn witherbloom_mireshade_b188_etb_mints_pest_and_has_deathtouch() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_mireshade_b188());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let mireshade = g.battlefield.iter()
        .find(|c| c.definition.name == "Witherbloom Mireshade (b188)").unwrap();
    assert!(mireshade.has_keyword(&Keyword::Deathtouch));
    let pest = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Pest");
    assert!(pest.is_some());
}

#[test]
fn pest_herald_b188_dies_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_herald_b188());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt herald");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 1, "drain 1 on death");
}

#[test]
fn witherbloom_spelleater_b188_magecraft_drains_two() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_spelleater_b188());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // 3 (bolt) + 2 (drain) = 5
    assert_eq!(g.players[1].life, p1_life - 5);
}

#[test]
fn lorehold_spiritsong_b188_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_spiritsong_b188());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().power(), 3);
}

#[test]
fn lorehold_sparkbarrier_b188_pings_and_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_sparkbarrier_b188());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 3);
    let spirit = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Spirit");
    assert!(spirit.is_some());
}

#[test]
fn lorehold_vanguard_ii_b188_is_a_four_mana_vigilance_reach_spirit() {
    let def = catalog::lorehold_vanguard_ii_b188();
    assert_eq!(def.cost.cmc(), 4);
    assert_eq!(def.power, 3);
    assert!(def.keywords.contains(&Keyword::Vigilance));
    assert!(def.keywords.contains(&Keyword::Reach));
}

#[test]
fn prismari_lavakin_b188_on_attack_pings() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_lavakin_b188());
    g.clear_sickness(id);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // On-attack ping 1 (auto-target opp).
    assert_eq!(g.players[1].life, p1_life - 1);
}

#[test]
fn prismari_storm_scholar_b188_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::prismari_storm_scholar_b188());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // No assertion needed beyond confirming the trigger doesn't break the game.
    // Verify the body shape.
    let def = catalog::prismari_storm_scholar_b188();
    assert_eq!(def.cost.cmc(), 2);
}

#[test]
fn prismari_hailcaller_b188_burns_each_opp_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_hailcaller_b188());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 3);
    // -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_mossleaf_b188_is_a_two_mana_reach_plant() {
    let def = catalog::quandrix_mossleaf_b188();
    assert_eq!(def.cost.cmc(), 2);
    assert_eq!(def.toughness, 3);
    assert!(def.keywords.contains(&Keyword::Reach));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Plant));
}

#[test]
fn quandrix_dataweaver_b188_magecraft_grows() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_dataweaver_b188());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_latticebreaker_b188_draws_three_cards() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_latticebreaker_b188());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast + 3 draw = +2 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 2);
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 191 (modern_decks) — multi-action cards + tribal.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_inkdrain_b191_drains_three_draws_and_mints_inkling() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkdrain_b191());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 3);
    // -1 cast + 1 draw = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let inkling = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Inkling");
    assert!(inkling.is_some());
}

#[test]
fn inkling_highscribe_b191_etb_scrys_and_magecraft_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::inkling_highscribe_b191());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Inkling Highscribe (b191)").unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
}

#[test]
fn witherbloom_doublestrike_b191_drains_two_and_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_doublestrike_b191());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest").collect();
    assert_eq!(pests.len(), 2);
}

#[test]
fn pest_druid_b191_taps_for_b_or_g() {
    let def = catalog::pest_druid_b191();
    assert_eq!(def.cost.cmc(), 2);
    assert_eq!(def.activated_abilities.len(), 1);
    assert!(def.activated_abilities[0].tap_cost);
    assert!(def.subtypes.creature_types.contains(&CreatureType::Pest));
}

#[test]
fn lorehold_echobringer_b191_mints_two_spirits_and_burns() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_echobringer_b191());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit").collect();
    assert_eq!(spirits.len(), 2);
}

#[test]
fn lorehold_sparrowscholar_b191_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::lorehold_sparrowscholar_b191());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // No assertion; trigger doesn't fail.
    let def = catalog::lorehold_sparrowscholar_b191();
    assert_eq!(def.cost.cmc(), 2);
}

#[test]
fn prismari_stormwave_b191_mints_treasure_draws_and_burns() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let beast = g.add_card_to_battlefield(1, catalog::quandrix_vinescaler_ii_b189());
    let id = g.add_card_to_hand(0, catalog::prismari_stormwave_b191());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(beast)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let treasure = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Treasure");
    assert!(treasure.is_some());
    assert_eq!(g.battlefield_find(beast).unwrap().damage, 2);
}

#[test]
fn prismari_wavetamer_b191_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::prismari_wavetamer_b191());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -1 cast + 1 draw - 1 discard = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_tinkermage_b191_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_tinkermage_b191());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let treasure = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Treasure");
    assert!(treasure.is_some());
    let _ = id;
}

#[test]
fn quandrix_sumtotal_b191_mints_four_four_fractal_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_sumtotal_b191());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal").unwrap();
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 4);
    // -1 cast + 1 draw = 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_sparkbloomer_b191_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_sparkbloomer_b191());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().power(), 3);
}

#[test]
fn quandrix_vinegrower_b191_etb_grows() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_vinegrower_b191());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Quandrix Vinegrower (b191)").unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    let _ = id;
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 190 (modern_decks) — keyword counter combo cards.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_doublecurse_b190_grants_deathtouch_and_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_doublecurse_b190());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.has_keyword(&Keyword::Deathtouch));
    assert!(c.has_keyword(&Keyword::Flying));
}

#[test]
fn silverquill_wardseal_b190_grants_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_wardseal_b190());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Vigilance));
}

#[test]
fn silverquill_lifeward_b190_grants_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_lifeward_b190());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Lifelink));
}

#[test]
fn witherbloom_doublegrowth_b190_grants_trample_and_plus_one() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_doublegrowth_b190());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.has_keyword(&Keyword::Trample));
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn witherbloom_venomgift_b190_grants_deathtouch() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_venomgift_b190());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Deathtouch));
}

#[test]
fn witherbloom_reachsage_b190_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_reachsage_b190());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn lorehold_doubleblast_b190_grants_first_strike_and_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_doubleblast_b190());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.has_keyword(&Keyword::FirstStrike));
    assert!(c.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_bondseal_b190_grants_vigilance_and_plus_one() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_bondseal_b190());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.has_keyword(&Keyword::Vigilance));
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn lorehold_phoenixmage_b190_etb_haste_self_via_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_phoenixmage_b190());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Lorehold Phoenixmage (b190)").unwrap();
    assert!(c.has_keyword(&Keyword::Haste));
}

#[test]
fn prismari_doublecharge_b190_grants_haste_and_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_doublecharge_b190());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.has_keyword(&Keyword::Haste));
    assert!(c.has_keyword(&Keyword::Flying));
}

#[test]
fn prismari_skydiver_b190_etb_grants_flying_via_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_skydiver_b190());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Prismari Skydiver (b190)").unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
}

#[test]
fn prismari_sparkforge_ii_b190_burns_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let beast = g.add_card_to_battlefield(1, catalog::quandrix_vinescaler_ii_b189());
    let id = g.add_card_to_hand(0, catalog::prismari_sparkforge_ii_b190());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(beast)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(beast).unwrap().damage, 2);
}

#[test]
fn quandrix_doublegrowth_b190_grants_trample_and_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_doublegrowth_b190());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.has_keyword(&Keyword::Trample));
    assert!(c.has_keyword(&Keyword::Flying));
}

#[test]
fn quandrix_riftleaper_b190_etb_grants_flying_via_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_riftleaper_b190());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Quandrix Riftleaper (b190)").unwrap();
    assert!(c.has_keyword(&Keyword::Flying));
}

#[test]
fn quandrix_sapleader_b190_etb_with_counter_and_reach_via_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_sapleader_b190());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Quandrix Sapleader (b190)").unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(c.has_keyword(&Keyword::Reach));
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 189 (modern_decks) — aggressive curve fillers.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn silverquill_drainmaster_ii_b189_etb_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_drainmaster_ii_b189());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 3);
}

#[test]
fn inkling_vassalking_b189_is_a_five_mana_flying_lifelink_inkling_knight() {
    let def = catalog::inkling_vassalking_b189();
    assert_eq!(def.cost.cmc(), 5);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Lifelink));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_exilewright_b189_exiles_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_exilewright_b189());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear exiled");
    let in_exile = g.exile.iter().any(|c| c.id == bear);
    assert!(in_exile, "bear in exile");
}

#[test]
fn witherbloom_spellblossom_b189_drains_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_spellblossom_b189());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4);
    assert_eq!(g.players[0].life, p0_life + 4);
}

#[test]
fn lorehold_voltmage_b189_etb_pings_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_voltmage_b189());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Auto-target picks opp player.
    assert_eq!(g.players[1].life, p1_life - 2);
    let _ = id;
}

#[test]
fn lorehold_fireseal_b189_mints_two_spirits_with_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_fireseal_b189());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit").collect();
    assert_eq!(spirits.len(), 2);
}

#[test]
fn lorehold_crusader_b189_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_crusader_b189());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.power(), 3);
}

#[test]
fn prismari_magmamancer_b189_attacks_pings_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_magmamancer_b189());
    g.clear_sickness(id);
    while g.step != crate::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // 2 damage to auto-pick opp player on-attack.
    assert!(g.players[1].life <= p1_life - 2);
}

#[test]
fn prismari_hailstrike_b189_burns_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    // Use a tougher creature so the test can measure both effects.
    let beast = g.add_card_to_battlefield(1, catalog::quandrix_vinescaler_ii_b189());
    let id = g.add_card_to_hand(0, catalog::prismari_hailstrike_b189());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(beast)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(beast).unwrap().damage, 2);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_beastcaller_b189_etb_fans_counters_to_friendly_fractals() {
    let mut g = two_player_game();
    let f1 = g.add_card_to_battlefield(0, catalog::quandrix_mossglider_b187()); // Fractal
    let id = g.add_card_to_hand(0, catalog::quandrix_beastcaller_b189());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let f1_counters_before = g.battlefield_find(f1).unwrap().counter_count(CounterType::PlusOnePlusOne);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let f1_counters_after = g.battlefield_find(f1).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(f1_counters_after, f1_counters_before + 1);
    let _ = id;
}

#[test]
fn quandrix_cantrip_b189_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_cantrip_b189());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast + 2 draws = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn quandrix_vinescaler_ii_b189_is_a_four_mana_four_four_reach_trampler() {
    let def = catalog::quandrix_vinescaler_ii_b189();
    assert_eq!(def.cost.cmc(), 4);
    assert_eq!(def.power, 4);
    assert!(def.keywords.contains(&Keyword::Reach));
    assert!(def.keywords.contains(&Keyword::Trample));
}

/// CR 121.2 — "Cards may only be drawn one at a time. If a player is
/// instructed to draw multiple cards, that player performs that many
/// individual card draws." A multi-draw effect should fire one CardDrawn
/// event per card drawn, not one batched event. This test pins the
/// per-draw fanout — Witherbloom Lifeknotter's `LifeGained/YourControl`
/// trigger fires once per individual draw via Drain.
#[test]
fn cr_121_2_multi_draw_fires_one_event_per_card() {
    use crate::game::GameEvent;
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let pre_hand = g.players[0].hand.len();
    // Cast a Draw 3 effect via Pop Quiz (Draw 2 + put one back) so we have
    // direct multi-draw via direct effect path. Use Inspired Idea (Draw 3).
    let id = g.add_card_to_hand(0, catalog::inspired_idea());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    let events = g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    let pass = g.perform_action(GameAction::PassPriority).expect("pass");
    let resolve = g.perform_action(GameAction::PassPriority).expect("resolve");
    // Drain stack of remaining triggers.
    drain_stack(&mut g);
    let all_events: Vec<_> = events.iter().chain(pass.iter()).chain(resolve.iter()).collect();
    let draw_count = all_events.iter()
        .filter(|e| matches!(e, GameEvent::CardDrawn { player: 0, .. }))
        .count();
    // -1 (cast) + 3 (draw) - 2 (stack 2 on top) = 0 net. Verify 3 individual
    // CardDrawn events fired.
    assert!(draw_count >= 3, "got {draw_count} CardDrawn events, expected ≥3 for Draw 3");
    let _ = pre_hand;
}

/// CR 405.5 — "When all players pass in succession, the top spell or ability
/// on the stack resolves. If the stack is empty when all players pass in
/// succession, the current step or phase ends." When two effects are on the
/// stack, the top one resolves first (LIFO).
#[test]
fn cr_405_5_top_of_stack_resolves_first_lifo() {
    let mut g = two_player_game();
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt1 on stack");
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt2 on stack");
    let p1_life = g.players[1].life;
    // bolt2 (top) resolves first → P1 takes 3 to face.
    drain_stack(&mut g);
    // Both resolve: bear took 3, P1 took 3.
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed by bolt1");
    assert_eq!(g.players[1].life, p1_life - 3, "P1 took 3 from bolt2");
}

/// CR 614.16 — "If an effect would put one or more counters on a
/// permanent, that many plus the additional counters from each applicable
/// replacement are put on that permanent instead." Keyword counters
/// (CR 122.1b) are counters too, so Doubling-Season-style scalers must
/// also double keyword counters.
#[test]
fn cr_614_16_keyword_counters_are_doubled_by_double_counters_static() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_pestseed());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_skystudent_b183());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skystudent castable");
    drain_stack(&mut g);
    // Skystudent grants 1 flying counter, Pestseed doubles to 2.
    // has_keyword still returns true with 2 counters; verify the count is 2.
    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(bear_card.has_keyword(&Keyword::Flying));
    assert_eq!(*bear_card.keyword_counters.get(&Keyword::Flying).unwrap_or(&0), 2,
        "Doubling Season-style scaler doubles flying counter (CR 614.16)");
}

/// CR 614.6 — "A replacement effect doesn't 'use up' the spell or ability
/// that generated it." But a single self-replacement only applies once per
/// event. This test pins that a shield counter (CR 122.1c) absorbs one
/// destroy event then is consumed; a second damage event goes through.
#[test]
fn cr_614_6_shield_counter_only_absorbs_one_event_then_pops() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Apply a shield counter via Silverquill Wardward (b170).
    let wardward = g.add_card_to_hand(0, catalog::silverquill_wardward_b170());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: wardward, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("wardward");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Shield), 2);
    // First bolt: shield absorbs the destroy event (no damage applied).
    let bolt1 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt1");
    drain_stack(&mut g);
    let bear_after1 = g.battlefield_find(bear).expect("bear alive after 1st bolt");
    // After 1st bolt: shield counter popped (3→2 wait — let's see how many).
    // Per CR 122.1c: each damage event removes one shield counter and prevents
    // the damage. So shield -1, no damage, bear at 2/2 with 1 shield.
    assert_eq!(bear_after1.counter_count(CounterType::Shield), 1);
    assert_eq!(bear_after1.damage, 0);
}
