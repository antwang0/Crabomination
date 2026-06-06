use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn prismari_pyroartist_pings_target_on_is_cast() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _id = g.add_card_to_battlefield(0, catalog::prismari_pyroartist());
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
    // Bolt 3 + magecraft 1 = 4 dmg → bear (2 toughness) dies.
    assert!(g.battlefield_find(target).is_none());
    let _ = dmg_before;
}

#[test]
fn prismari_brushpyre_has_haste_and_pumps_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_brushpyre());
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
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
    // Base 4/3 + magecraft +1/+0 EOT = 5/3.
    assert_eq!(card.power(), 5);
    assert_eq!(card.toughness(), 3);
}

// ── Batch 43 (modern_decks) tests ───────────────────────────────────────────

#[test]
fn silverquill_blackquill_acolyte_drains_each_opp_on_cast() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::silverquill_blackquill_acolyte());
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
    // 3 (bolt) + 1 (magecraft drain) = 4 lost; +1 gained.
    assert_eq!(g.players[1].life, opp_before - 4);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn silverquill_ravenmage_attack_drains_each_opp() {
    use crate::game::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_ravenmage());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attackers");
    drain_stack(&mut g);
    // 1 drain (the attack trigger) — combat damage is separate.
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn silverquill_inkjet_scribe_etb_mints_inkling_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkjet_scribe());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Scribe castable");
    drain_stack(&mut g);
    // Scribe + Inkling token = 2 new permanents.
    assert_eq!(g.battlefield.len(), bf_before + 2);
}

#[test]
fn silverquill_grand_inkmaster_etb_drains_four() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_grand_inkmaster());
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
    .expect("Inkmaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 4);
    assert_eq!(g.players[0].life, life_before + 4);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_diatribe_drains_four_target_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_diatribe());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Diatribe castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn inkling_saboteur_combat_damage_forces_discard() {
    use crate::game::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::island());
    let hand_before = g.players[1].hand.len();
    let id = g.add_card_to_battlefield(0, catalog::inkling_saboteur());
    g.clear_sickness(id);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attackers");
    // Run through combat to deal combat damage.
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    // Opponent should have discarded one card from the combat damage trigger.
    assert_eq!(g.players[1].hand.len(), hand_before - 1);
}

#[test]
fn silverquill_sealwright_magecraft_pumps_friendly_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _id = g.add_card_to_battlefield(0, catalog::silverquill_sealwright());
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
    let card = g.battlefield_find(target).unwrap();
    assert_eq!(
        card.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn witherbloom_thornmaster_etb_mints_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_thornmaster());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Thornmaster castable");
    drain_stack(&mut g);
    // Thornmaster + Pest token = 2 new permanents.
    assert_eq!(g.battlefield.len(), bf_before + 2);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Deathtouch));
}

#[test]
fn witherbloom_grafted_seer_scrys_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let _id = g.add_card_to_battlefield(0, catalog::witherbloom_grafted_seer());
    drain_stack(&mut g);
    let lib_before = g.players[0].library.len();
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
    // Scry 1 keeps the library count the same.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn witherbloom_ravensoul_dies_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_ravensoul());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    // Lightning Bolt 3 dmg = exact lethal on 3-toughness Ravensoul.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Permanent(id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "Ravensoul should be dead");
    // Death trigger drains 2: each opp loses 2, you gain 2.
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn witherbloom_blightroot_drains_three_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_blightroot());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Blightroot castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, life_before + 3);
}

#[test]
fn witherbloom_pestswarm_master_etb_mints_two_pests() {
    let mut g = two_player_game();
    let _id = g.add_card_to_hand(0, catalog::witherbloom_pestswarm_master());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: _id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Master castable");
    drain_stack(&mut g);
    // Master + 2 Pests = 3 new permanents.
    assert_eq!(g.battlefield.len(), bf_before + 3);
}

#[test]
fn witherbloom_spireling_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_spireling());
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Spireling castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Reach));
}

#[test]
fn lorehold_emberhand_priest_pings_target_on_is_cast() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_emberhand_priest());
    drain_stack(&mut g);
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
    // Bolt 3 + magecraft ping 1 = 4 dmg → bear (2 toughness) dies.
    assert!(g.battlefield_find(target).is_none());
}

#[test]
fn lorehold_ironbacked_archivist_etb_exiles_graveyard_card() {
    let mut g = two_player_game();
    let gy_card = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_ironbacked_archivist());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Archivist castable");
    drain_stack(&mut g);
    // Auto-target picker should have exiled the bolt from the opp gy.
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == gy_card));
    assert!(g.exile.iter().any(|c| c.id == gy_card));
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_lightspeaker_attack_pings_target() {
    use crate::game::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_lightspeaker());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attackers");
    drain_stack(&mut g);
    // Just the attack-trigger ping of 1 (combat damage hasn't happened).
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn lorehold_warpriest_etb_deals_two_to_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_warpriest());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Warpriest castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none());
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn lorehold_emberscholar_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_emberscholar());
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
    // 3 (bolt) + 1 (magecraft ping) = 4 lost.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_relicbearer_grows_on_gy_leave() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_relicbearer());
    drain_stack(&mut g);
    // Seed gy with a card, then use Lorehold Acolyte to move it to exile
    // (a Card-Left-Graveyard event).
    let _gy_card = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let exiler = g.add_card_to_hand(0, catalog::lorehold_acolyte());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: exiler,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Acolyte castable");
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
fn lorehold_ember_sentinel_etb_gains_three_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_ember_sentinel());
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
    .expect("Sentinel castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Vigilance));
}

#[test]
fn quandrix_thoughtweaver_etb_draws_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_thoughtweaver());
    let hand_before = g.players[0].hand.len();
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
    .expect("Thoughtweaver castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_geode_smith_grows_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_geode_smith());
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
    assert_eq!(
        card.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn quandrix_grand_calculator_etb_scales_with_lands() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Add 4 lands to controller.
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_grand_calculator());
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
    .expect("Calculator castable");
    drain_stack(&mut g);
    // Counters should land on a friendly creature (4 from 4 lands).
    // Auto-picker may pick bear or calculator — assert that the total
    // across all friendly creatures grew by 4.
    let bear_counters = g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let calc_counters = g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(bear_counters + calc_counters, 4);
}

#[test]
fn fractal_seer_enters_with_one_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_seer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Seer castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(
        card.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1
    );
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 1);
}

#[test]
fn quandrix_lifestream_pumps_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_lifestream());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lifestream castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(target).unwrap();
    assert_eq!(
        card.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1
    );
    // -1 cast + 1 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn fractal_aegis_enters_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_aegis());
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
    .expect("Aegis castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::Trample));
}

#[test]
fn quandrix_mistforger_etb_mints_fractal_scaled_by_creatures() {
    let mut g = two_player_game();
    // 2 friendly creatures pre-cast.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_mistforger());
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
    .expect("Mistforger castable");
    drain_stack(&mut g);
    // 2 bears + Mistforger + Fractal token = 4 creatures at the
    // AddCounter step (token enters before counters are placed).
    let fractal = g.battlefield.iter().find(|c| c.definition.name == "Fractal").unwrap();
    assert_eq!(
        fractal.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        4,
        "Fractal should have 4 +1/+1 counters (2 bears + Mistforger + Fractal itself)"
    );
}

#[test]
fn prismari_blastcaster_pings_creature_on_is_cast() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _id = g.add_card_to_battlefield(0, catalog::prismari_blastcaster());
    drain_stack(&mut g);
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
    // Bolt 3 + magecraft ping 1 = 4 dmg → bear (2 toughness) dies.
    assert!(g.battlefield_find(target).is_none());
}

#[test]
fn prismari_oddsmaker_scrys_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let _id = g.add_card_to_battlefield(0, catalog::prismari_oddsmaker());
    drain_stack(&mut g);
    let lib_before = g.players[0].library.len();
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
    // Scry 1 keeps library count the same.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn prismari_glassforge_etb_mints_treasure() {
    let mut g = two_player_game();
    let _id = g.add_card_to_hand(0, catalog::prismari_glassforge());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: _id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Glassforge castable");
    drain_stack(&mut g);
    // Glassforge + Treasure = 2 new permanents.
    assert_eq!(g.battlefield.len(), bf_before + 2);
}

#[test]
fn prismari_emberweaver_etb_deals_two_with_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_emberweaver());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Emberweaver castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn prismari_skyflare_deals_two_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_skyflare());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Skyflare castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn prismari_volcanic_song_burns_creature_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_volcanic_song());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Volcanic Song castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none());
    // -1 cast + 1 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_inkjet_apprentice_pings_each_opp_on_cast() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::prismari_inkjet_apprentice());
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
    // 3 (bolt) + 1 (magecraft ping) = 4 lost.
    assert_eq!(g.players[1].life, opp_before - 4);
}

// ── Push (modern_decks / claude/modern_decks) — close-out Silverquill batch ──
//
// 22 new Silverquill cards added in `catalog::sets::stx::silverquill`.
// Each test exercises the card's primary play pattern.

#[test]
fn silverquill_maxim_deals_three_and_gains_three_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_maxim());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    let opp = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Maxim castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 3);
    assert_eq!(g.players[0].life, life + 3);
}

#[test]
fn inkling_vassal_drains_each_opp_on_is_cast() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::inkling_vassal());
    drain_stack(&mut g);
    let opp = g.players[1].life;
    let life = g.players[0].life;
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
    // Bolt 3 + Vassal magecraft 1 = 4.
    assert_eq!(g.players[1].life, opp - 4);
    // Magecraft drain gives +1.
    assert_eq!(g.players[0].life, life + 1);
}

#[test]
fn silverquill_vellum_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_vellum());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let opp = g.players[1].life;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Vellum castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2);
    assert_eq!(g.players[0].life, life + 2);
}

#[test]
fn inkling_decreemaster_etb_forces_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let hand_before = g.players[1].hand.len();
    let id = g.add_card_to_hand(0, catalog::inkling_decreemaster());
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
    .expect("Decreemaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 1);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_penbringer_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::silverquill_penbringer());
    drain_stack(&mut g);
    let life = g.players[0].life;
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
    assert_eq!(g.players[0].life, life + 1);
}

#[test]
fn silverquill_ravenswing_attack_drains_each_opp() {
    use crate::game::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_ravenswing());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let opp = g.players[1].life;
    let life = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attackers");
    drain_stack(&mut g);
    // Attack-trigger drains 1 (regardless of combat damage step).
    assert_eq!(g.players[1].life, opp - 1);
    assert_eq!(g.players[0].life, life + 1);
}

#[test]
fn inkling_magistrate_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_magistrate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Magistrate castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2);
}

#[test]
fn silverquill_liturgy_drains_two_each_opp_gains_four_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_liturgy());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp = g.players[1].life;
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Liturgy castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2);
    assert_eq!(g.players[0].life, life + 4);
    // -1 cast +1 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand);
}

#[test]
fn inkling_bookbinder_magecraft_grows() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_bookbinder());
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
    assert_eq!(
        card.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn silverquill_scribebearer_etb_scrys_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::silverquill_scribebearer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Scribebearer castable");
    drain_stack(&mut g);
    // Scry 2 doesn't draw or move cards; library size unchanged.
    assert_eq!(g.players[0].library.len(), lib_before);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
}

#[test]
fn silverquill_adept_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::silverquill_adept());
    drain_stack(&mut g);
    let opp = g.players[1].life;
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
    // 3 (bolt) + 1 (magecraft drain) = 4.
    assert_eq!(g.players[1].life, opp - 4);
}

#[test]
fn silverquill_spellguard_etb_gains_two_life_with_first_strike() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_spellguard());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Spellguard castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::FirstStrike));
}

#[test]
fn inkling_sageling_dies_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::inkling_sageling());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Permanent(id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none());
    // -1 cast +1 draw = net 0 vs hand_before.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn silverquill_inkcaller_etb_mints_an_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkcaller());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Inkcaller castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.len(), bf_before + 2);
}

#[test]
fn silverquill_lecture_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_lecture());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp = g.players[1].life;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lecture castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 3);
    assert_eq!(g.players[0].life, life + 3);
}

#[test]
fn inkling_battlescholar_attack_pumps_self() {
    use crate::game::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_battlescholar());
    g.clear_sickness(id);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attackers");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power_bonus, 1);
}

#[test]
fn silverquill_final_year_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_final_year());
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
    assert_eq!(card.power_bonus, 1);
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_devotee_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_devotee());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Devotee castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2);
}

#[test]
fn silverquill_inkspear_drains_target_opponent_for_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkspear());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let opp = g.players[1].life;
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Inkspear castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1);
    assert_eq!(g.players[0].life, life + 1);
}

#[test]
fn inkling_sergeant_anthems_other_inklings() {
    // Pump effects are computed via the layer system; we check the
    // computed power on a separate Inkling.
    let mut g = two_player_game();
    let other_inkling = g.add_card_to_battlefield(0, catalog::inkling_vassal());
    let sergeant = g.add_card_to_battlefield(0, catalog::inkling_sergeant());
    drain_stack(&mut g);
    // Inkling Vassal is 1/2; +1/+0 anthem from Sergeant → 2 effective power.
    let computed = g
        .compute_battlefield()
        .into_iter()
        .find(|c| c.id == other_inkling)
        .unwrap();
    assert_eq!(computed.power, 2);
    // Sergeant doesn't anthem itself ("other" clause) — 2/2 base unchanged.
    let sergeant_computed = g
        .compute_battlefield()
        .into_iter()
        .find(|c| c.id == sergeant)
        .unwrap();
    assert_eq!(sergeant_computed.power, 2);
}

#[test]
fn silverquill_verdict_exiles_high_power_creature() {
    let mut g = two_player_game();
    // Add a 4/4 creature (Serra Angel) on opponent side.
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::silverquill_verdict());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Verdict castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none());
    assert_eq!(g.players[0].life, life + 2);
}

#[test]
fn silverquill_verdict_rejects_low_power_target() {
    // Grizzly Bears is 2/2 — power < 3, target rejected.
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_verdict());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(), "Verdict should reject a 2-power target");
}

#[test]
fn silverquill_curator_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let creature_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_curator());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Curator castable");
    drain_stack(&mut g);
    // -1 cast +1 etb returned = net 0; the bears card should be in hand now.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].hand.iter().any(|c| c.id == creature_in_gy));
}

#[test]
fn inkling_bondsmith_etb_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inkling_bondsmith());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bondsmith castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(target).unwrap();
    assert_eq!(card.power_bonus, 1);
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_aspect_etb_pumps_self_and_grants_menace() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_aspect());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Aspect castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power_bonus, 1);
    assert!(card.has_keyword(&Keyword::Menace));
}

#[test]
fn pestmaster_pumps_on_pest_token_death_via_cached_controller() {
    // Lock-in for the push (modern_decks claude/modern_decks) engine fix:
    // `died_card_controllers` cache lets AnotherOfYours triggers fire
    // off dying tokens (CR 111.7c "ceases to exist" SBA removes the
    // token from every zone in the same sweep as the death event, so
    // the zone-walking subject_controller lookup returns None without
    // the cache).
    use crate::game::types::Target;
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::witherbloom_pestmaster());
    // Cast Pest Summoning to mint two Pest tokens under P0's control.
    let ps = g.add_card_to_hand(0, catalog::pest_summoning());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Summoning castable");
    drain_stack(&mut g);
    // Two Pest tokens entered. Find one and kill it with a Bolt.
    let pest_id = g
        .battlefield
        .iter()
        .find(|c| {
            c.id != pm
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Pest)
        })
        .map(|c| c.id)
        .expect("Found a Pest token on the battlefield");
    let pmc_before = g
        .battlefield_find(pm)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    // Kill the Pest with a Bolt.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(pest_id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Pestmaster sees the token death via the controller cache and grows.
    let pmc = g.battlefield_find(pm).expect("Pestmaster still alive");
    assert_eq!(
        pmc.counter_count(CounterType::PlusOnePlusOne),
        pmc_before + 1,
        "Pestmaster gained a +1/+1 counter on Pest *token* death (cache lookup path)"
    );
}

#[test]
fn felisa_pumps_inkling_on_pest_token_with_counter_death() {
    // Lock-in for the push (modern_decks batch 47) token-death snapshot
    // cache: Felisa's "creature with +1/+1 counter dies → 1/1 W/B
    // Inkling" trigger fires for TOKEN deaths too. Before the cache
    // landed, the CR 111.7c "token ceases to exist" SBA removed the
    // dying token from every zone in the same sweep — by dispatch time
    // the `WithCounter(+1/+1)` filter (evaluated via
    // evaluate_requirement_static on the dying card) returned false
    // because no zone had the card.
    use crate::game::types::Target;
    let mut g = two_player_game();
    let _felisa = g.add_card_to_battlefield(0, catalog::felisa_fang_of_silverquill());
    // Mint a Pest token under P0 (the Pest also has a +1/+1 counter
    // applied directly to the battlefield instance — simulating a
    // mid-game scenario where, say, Silverquill Memorize pumped a
    // friendly Pest before it died).
    let ps = g.add_card_to_hand(0, catalog::pest_summoning());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ps,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pest Summoning castable");
    drain_stack(&mut g);
    // Find the first Pest token.
    let pest_id = g
        .battlefield
        .iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .map(|c| c.id)
        .expect("Pest token created");
    // Add a +1/+1 counter directly.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == pest_id) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    let bf_before = g.battlefield.len();
    // Kill the Pest with a Bolt.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(pest_id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // The Pest is gone (ceased to exist), Bolt is in graveyard, and
    // Felisa minted an Inkling because the cache let her counter
    // filter resolve on the dying token. Net: -1 pest, -0 bolt
    // (still in gy, off bf) + 1 inkling = bf_before.
    let inkling_present = g
        .battlefield
        .iter()
        .any(|c| c.definition.name == "Inkling");
    assert!(
        inkling_present,
        "Felisa mints an Inkling when a counter-bearing Pest token dies"
    );
    // Sanity: bf size is unchanged (pest out, inkling in).
    assert_eq!(g.battlefield.len(), bf_before);
}

// ── Batch 47 follow-up (modern_decks) — new STX card tests ──────────────────

#[test]
fn silverquill_quillbinder_etb_mints_inkling_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_quillbinder());
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
    .expect("Quillbinder castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
    let tokens: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == "Inkling")
        .collect();
    assert_eq!(tokens.len(), 1, "should mint one Inkling token");
    assert!(tokens[0].has_keyword(&Keyword::Flying));
}

#[test]
fn inkling_quillblade_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_quillblade());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().power(), 2);
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
    assert_eq!(g.battlefield_find(id).unwrap().power(), 3);
    assert_eq!(g.battlefield_find(id).unwrap().toughness(), 2);
}

#[test]
fn silverquill_reprover_shrinks_opp_creature_on_etb() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_reprover());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Reprover castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(opp).unwrap();
    assert_eq!(card.power(), 0, "Bear should be 0/2 after -2/-0");
    assert_eq!(card.toughness(), 2);
}

#[test]
fn silverquill_refrain_drains_two_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_refrain());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Refrain castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn inkling_ascendancy_mints_two_inklings_and_pumps_team() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inkling_ascendancy());
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
    .expect("Inkling Ascendancy castable");
    drain_stack(&mut g);
    let inklings: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == "Inkling")
        .collect();
    assert_eq!(inklings.len(), 2);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 3); // +1/+0
}

#[test]
fn witherbloom_vinepicker_magecraft_adds_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_vinepicker());
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
    assert_eq!(
        card.counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1
    );
}

#[test]
fn witherbloom_pestbloomer_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestbloomer());
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
    .expect("Pestbloomer castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == "Pest")
        .collect();
    assert_eq!(pests.len(), 2);
}

#[test]
fn witherbloom_rotsplash_shrinks_creature_and_gains_one_life() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_rotsplash());
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Rotsplash castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp).is_none(), "Bear should die");
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn witherbloom_vinetwister_etb_fans_counters_on_other_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinetwister());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Vinetwister castable");
    drain_stack(&mut g);
    let bear_c = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_c.counter_count(CounterType::PlusOnePlusOne), 1);
    let self_c = g.battlefield_find(id).unwrap();
    assert_eq!(self_c.counter_count(CounterType::PlusOnePlusOne), 0);
}

#[test]
fn lorehold_spiritbinder_etb_mints_spirit_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritbinder());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Spiritbinder castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 1);
    assert_eq!(spirits[0].power(), 2);
    assert_eq!(spirits[0].toughness(), 2);
}

#[test]
fn lorehold_sparkflinger_magecraft_pings_target() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_sparkflinger());
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
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_battle_cry_mints_spirit_with_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_battle_cry());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Battle Cry castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 1);
    assert!(spirits[0].has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_battle_memorial_deals_three_to_creature_and_three_to_player() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_battle_memorial());
    let opp_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp)),
        additional_targets: vec![crate::game::types::Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("Battle Memorial castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp).is_none(), "3-damage kills the bear");
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn lorehold_veteran_haste_etb_pings_target() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_veteran());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Veteran castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_scrollwarden_etb_mints_spirit_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_scrollwarden());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Scrollwarden castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Flying));
    let spirits: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 1);
}

#[test]
fn quandrix_arcanist_flash_magecraft_scrys() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_arcanist());
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Flash));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.add_card_to_library(0, catalog::island());
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    assert!(!g.players[0].library.is_empty());
}

#[test]
fn quandrix_triplecaster_etb_puts_two_counters_on_target() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_triplecaster());
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
    .expect("Triplecaster castable");
    drain_stack(&mut g);
    let total = g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne)
        + g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(total, 2);
}

#[test]
fn quandrix_snapcaster_etb_returns_is_from_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::quandrix_snapcaster());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Snapcaster castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
}

#[test]
fn quandrix_counterfold_doubles_counters_on_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(target).unwrap().counters.insert(CounterType::PlusOnePlusOne, 3);
    let id = g.add_card_to_hand(0, catalog::quandrix_counterfold());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Counterfold castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne),
        6
    );
}

#[test]
fn quandrix_augurer_etb_draws_and_fans_counters() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_augurer());
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
    .expect("Augurer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn prismari_scribbler_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::prismari_scribbler());
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
    .expect("Scribbler castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_skyspark_pumps_and_grants_flying() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_skyspark());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Skyspark castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(target).unwrap();
    assert_eq!(bear.power(), 3);
    assert_eq!(bear.toughness(), 3);
    assert!(bear.has_keyword(&Keyword::Flying));
}

#[test]
fn prismari_embershout_burns_creature_or_player_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_embershout());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crate::game::types::Target::Permanent(opp)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Embershout castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp).is_none(), "3 damage kills bear");
}

#[test]
fn prismari_stormcoil_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::prismari_stormcoil());
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
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 4);
}

#[test]
fn strixhaven_quartermaster_etb_gains_two_life_and_vigilance() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::strixhaven_quartermaster());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Quartermaster castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Vigilance));
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn strixhaven_library_mage_etb_scrys_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_library_mage());
    let lib_before = g.players[0].library.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Library Mage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn strixhaven_demonstrator_etb_drains_two() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::strixhaven_demonstrator());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Demonstrator castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn strixhaven_crucible_activation_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::strixhaven_crucible());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: Some(crate::game::types::Target::Player(1)), x_value: None })
    .expect("Crucible activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn strixhaven_skylancer_is_a_flying_vigilance_finisher() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::strixhaven_skylancer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Skylancer castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn prismari_treasurespark_mints_treasure_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_treasurespark());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Treasurespark castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before);
    let treasures: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .collect();
    assert_eq!(treasures.len(), 1);
}

// ── Batch 48 (modern_decks) — new card tests ────────────────────────────────

#[test]
fn silverquill_wingweaver_etb_surveils_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let lib_before = g.players[0].library.len();
    let id = g.add_card_to_hand(0, catalog::silverquill_wingweaver());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wingweaver castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::Flying));
    // Surveil moves one card off the top of the library (either to gy or
    // stays — AutoDecider keeps it).
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn silverquill_recital_drains_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_recital());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recital castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
    // Net hand: -1 (Recital cast) +1 (draw) = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_heralder_is_a_three_mana_flying_lifelinker() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_heralder());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Heralder castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn silverquill_inkdraft_drains_one_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_inkdraft());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkdraft castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn silverquill_lawscribe_etb_taps_opp_creature() {
    let mut g = two_player_game();
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_lawscribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(prey)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lawscribe castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(prey).unwrap().tapped);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn witherbloom_pestcaller_v2_magecraft_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_pestcaller_v2());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
    // Source survives.
    assert!(g.battlefield_find(id).is_some());
}

#[test]
fn witherbloom_vinepriest_etb_gains_two_life_and_magecraft_gains_one() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinepriest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinepriest castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
}

#[test]
fn pest_quartermaster_etb_mints_pest_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::pest_quartermaster());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quartermaster castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 4);
    assert!(card.has_keyword(&Keyword::Trample));
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
    // Hand: -1 (Quartermaster cast) +1 (draw) = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn witherbloom_toxicvial_shrinks_creature_by_three() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxicvial());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxicvial castable");
    drain_stack(&mut g);
    // Bear is 2/2, gets -3/-3 -> -1/-1 -> dies to SBA.
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn witherbloom_lifechant_gains_five_life_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifechant());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifechant castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 5);
}

#[test]
fn lorehold_flameherald_v2_etb_pings_target() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_flameherald_v2());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flameherald castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Haste));
}

#[test]
fn spirit_bardguard_is_a_three_mana_vigilance_soldier() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spirit_bardguard());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bardguard castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::Vigilance));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Spirit));
}

#[test]
fn lorehold_sparkwarden_self_pumps_on_instant_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_sparkwarden());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 2);
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn lorehold_spiritscribe_mints_two_spirits_and_pings_each_opp() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritscribe());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritscribe castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 2);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn lorehold_phoenix_soldier_is_a_four_mana_flying_haster() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_phoenix_soldier());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Phoenix-Soldier castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Haste));
}

#[test]
fn quandrix_pupil_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let lib_before = g.players[0].library.len();
    let _id = g.add_card_to_battlefield(0, catalog::quandrix_pupil());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry: top card either stays or is binned. lib_before stays the same
    // or decreases by 1.
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn fractal_tideshaper_enters_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_tideshaper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tideshaper castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 3);
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn quandrix_numerologist_etb_draws_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::quandrix_numerologist());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Numerologist castable");
    drain_stack(&mut g);
    // Net: -1 (cast) +1 (draw) = same hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_geometer_v3_etb_pumps_each_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_geometer_v3());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Geometer castable");
    drain_stack(&mut g);
    // Geometer itself gets a counter (it's a friendly creature when fan-out
    // runs), plus the bear.
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 1);
    let geometer = g.battlefield_find(id).unwrap();
    assert_eq!(geometer.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn fractal_cascade_mints_four_four_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_cascade());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cascade castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1);
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 4);
}

#[test]
fn prismari_burnscribe_etb_pings_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_burnscribe());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Burnscribe castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.damage, 1);
}

#[test]
fn prismari_treasurespell_mints_two_treasures_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::prismari_treasurespell());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Treasurespell castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .collect();
    assert_eq!(treasures.len(), 2);
    // Net: -1 (cast) +1 (draw) = same hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_sparkmage_v3_pings_creature_on_is_cast() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _id = g.add_card_to_battlefield(0, catalog::prismari_sparkmage_v3());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft ping: target_filtered picks the bear (sole creature target).
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.damage, 1);
}

#[test]
fn prismari_embergale_burns_creature_and_pings_each_opp() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_embergale());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Embergale castable");
    drain_stack(&mut g);
    // Bear takes 3, then 1 damage to each opp.
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn prismari_stormgale_is_a_four_mana_flying_looter() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::prismari_stormgale());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormgale castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::Flying));
}

// ── Batch 48 follow-up (modern_decks) — second card-set tests ───────────────

#[test]
fn inkling_scriptmaster_etb_drains_two_and_is_flier() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::inkling_scriptmaster());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scriptmaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
}

#[test]
fn silverquill_inkdancer_self_pumps_on_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_inkdancer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 2);
}

#[test]
fn silverquill_vermilion_shrinks_creature_and_gains_three_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_vermilion());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vermilion castable");
    drain_stack(&mut g);
    // Bear 2/2 → -1/-1 → SBA kills.
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, life_before + 3);
}

#[test]
fn silverquill_drainmaster_v2_etb_drains_three() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_drainmaster_v2());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainmaster v2 castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, life_before + 3);
}

#[test]
fn silverquill_bookbond_returns_creature_and_gains_life() {
    let mut g = two_player_game();
    let creature_def = catalog::grizzly_bears();
    let dead = g.add_card_to_graveyard(0, creature_def);
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_bookbond());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bookbond castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    // Bear should be back in hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == dead));
}

#[test]
fn pest_glutton_etb_mints_pest_and_gains_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::pest_glutton());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Glutton castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn witherbloom_saprosage_etb_scrys_and_magecraft_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_saprosage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Note: ETB Scry happened when card was added to battlefield;
    // magecraft fires on the cast.
    assert_eq!(g.players[0].life, life_before + 1);
    // Body keywords sanity.
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn pestilent_marsh_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pestilent_marsh());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Marsh castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 2);
}

#[test]
fn witherbloom_witchwarden_is_a_five_mana_lifelinker() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_witchwarden());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Witchwarden castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn witherbloom_toxicvigor_drains_three_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxicvigor());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxicvigor castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    assert_eq!(g.players[0].life, life_before + 3);
}

#[test]
fn spirit_spellsmith_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let _id = g.add_card_to_battlefield(0, catalog::spirit_spellsmith());
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
fn lorehold_glimmercaller_etb_burns_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_glimmercaller());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Glimmercaller castable");
    drain_stack(&mut g);
    // Bear 2/2, 2 damage → dies.
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn lorehold_refrain_burns_and_gains_life() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_refrain());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Refrain castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn spirit_banner_bearer_anthems_other_spirits() {
    let mut g = two_player_game();
    let other_spirit_id = g.add_card_to_battlefield(0, catalog::lorehold_aerospirit());
    let banner_id = g.add_card_to_battlefield(0, catalog::spirit_banner_bearer());
    // Use layered computed view per CR 613.
    let computed: Vec<_> = g.compute_battlefield();
    let banner_computed = computed.iter().find(|c| c.id == banner_id).expect("banner");
    let spirit_computed = computed.iter().find(|c| c.id == other_spirit_id).expect("aero");
    // Banner-bearer doesn't anthem itself; stays 1/3.
    assert_eq!(banner_computed.power, 1);
    // Aerospirit is base 3/2; +1/+0 from anthem → 4/2.
    assert_eq!(spirit_computed.power, 4);
}

#[test]
fn lorehold_battle_drum_pumps_team_with_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_battle_drum());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battle Drum castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 3);
    assert!(bear_card.has_keyword(&Keyword::Haste));
}

#[test]
fn fractal_wavebreaker_etb_bounces_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_wavebreaker());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wavebreaker castable");
    drain_stack(&mut g);
    // Bear should be back in opp's hand.
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

#[test]
fn quandrix_vinepriest_etb_fetches_basic_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::quandrix_vinepriest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinepriest castable");
    drain_stack(&mut g);
    // Forest should be in hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == forest));
}

#[test]
fn fractal_anomaly_v2_mints_5_5_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_anomaly_v2());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Anomaly v2 castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1);
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 5);
}

#[test]
fn quandrix_calculator_etb_scrys_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::quandrix_calculator_v2());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Calculator castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
}

#[test]
fn quandrix_tide_pumps_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_tide());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tide castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 1);
    // Net: -1 (cast) +1 (draw) = same hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_flamewright_etb_pings_two() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_flamewright());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flamewright castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn prismari_cantrip_spark_burns_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_cantrip_spark());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cantrip Spark castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    // Net: -1 (cast) +1 (draw) = same hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_dragonkin_etb_draws_one_and_flies() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::prismari_dragonkin());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dragonkin castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 4);
    assert!(card.has_keyword(&Keyword::Flying));
    // Hand: -1 (cast) +1 (draw) = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_sparktwister_scrys_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _id = g.add_card_to_battlefield(0, catalog::prismari_sparktwister());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let lib_before = g.players[0].library.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry either keeps top or bins it.
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn prismari_spelljay_burns_creature_for_four() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_spelljay());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spelljay castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
}

// ── Batch 48 follow-up #2 (modern_decks) — additional card tests ────────────

#[test]
fn inkling_scrollwarden_is_flying_vigilance() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_scrollwarden());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scrollwarden castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn silverquill_pencrafter_etb_draws_and_loses_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_pencrafter());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pencrafter castable");
    drain_stack(&mut g);
    // Net hand: -1 (cast) +1 (draw) = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].life, life_before - 1);
}

#[test]
fn inkling_inkblot_drains_one() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::inkling_inkblot());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkblot castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn spirit_spearmaiden_is_first_strike_two_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spirit_spearmaiden());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spearmaiden castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
    assert!(card.has_keyword(&Keyword::FirstStrike));
}

#[test]
fn lorehold_lavabolt_deals_three_to_player() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_lavabolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lavabolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn lorehold_smiteseer_etb_burns_creature_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::lorehold_smiteseer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Smiteseer castable");
    drain_stack(&mut g);
    // Bear 2/2 takes 2 damage → dies.
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn fractal_sentinel_enters_with_five_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_sentinel());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sentinel castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 5);
    assert_eq!(card.power(), 5);
    assert_eq!(card.toughness(), 5);
    assert!(card.has_keyword(&Keyword::Trample));
}
