use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;


#[test]
fn silverquill_quilledict_b154_drains_three_and_mints_two_inklings() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::silverquill_quilledict_b154());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Quilledict castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3, "Drain 3");
    let inklings = g.battlefield.iter()
        .filter(|c| c.definition.name == "Inkling").count();
    assert_eq!(inklings, 2, "Mints exactly 2 Inkling tokens");
}

// ── Consolidated magecraft tables ──────────────────────────────────────────
// Shared shape: the card sits on the battlefield; caster bolts the opponent;
// the magecraft trigger fires. Table entries vary only the card + expectation.

#[test]
fn magecraft_mints_token_table() {
    for (def, token) in [
        (catalog::quandrix_fractalsmith_b154(), "Fractal"),
        (catalog::quandrix_fractalist_b155(), "Fractal"),
        (catalog::prismari_treasurelord_b154(), "Treasure"),
        (catalog::prismari_treasureseeker_b155(), "Treasure"),
        (catalog::prismari_sparkmage_ii_b158(), "Treasure"),
        (catalog::inkling_spellbinder_b155(), "Inkling"),
        (catalog::pest_hivescholar_b155(), "Pest"),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let _c = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let before = g.battlefield.iter()
            .filter(|c| c.definition.name == token).count();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let after = g.battlefield.iter()
            .filter(|c| c.definition.name == token).count();
        assert_eq!(after, before + 1, "{name}: magecraft mints one {token}");
    }
}

#[test]
fn magecraft_self_counter_table() {
    for def in [
        catalog::quandrix_equationmage_b154(),
        catalog::quandrix_hatchling_b155(),
        catalog::witherbloom_vinepoet_b155(),
        catalog::pest_engorger_ii_b158(),
        catalog::quandrix_fractaltender_b158(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(id).expect("on bf");
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1,
            "{name}: +1/+1 counter from magecraft");
    }
}

#[test]
fn magecraft_self_pump_table() {
    for def in [
        catalog::prismari_tempestmage_b154(),
        catalog::inkling_slipscribe_b155(),
        catalog::prismari_flameshape_b155(),
        catalog::pest_wretch_b158(),
        catalog::lorehold_spirit_drummer_b158(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let power_before = g.battlefield_find(id).expect("on bf").power();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let power_after = g.battlefield_find(id).expect("on bf").power();
        assert_eq!(power_after, power_before + 1,
            "{name}: power +1 from magecraft");
    }
}

#[test]
fn magecraft_gain_life_table() {
    for (def, gain) in [
        (catalog::silverquill_sentinel_b154(), 1),
        (catalog::silverquill_reciter_b155(), 2),
        (catalog::pest_acolyte_ii_b157(), 1),
        (catalog::lorehold_echocaller_b155(), 1),
        (catalog::silverquill_pen_bearer_b158(), 1),
        (catalog::witherbloom_vinepoet_ii_b158(), 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let _c = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life_before + gain,
            "{name}: magecraft gains {gain} life");
    }
}

#[test]
fn magecraft_opp_loses_one_table() {
    // Opp takes Bolt (3) + magecraft ping/drain (1) = -4; drainers also
    // gain their controller 1 life (self_gain = 1).
    for (def, self_gain) in [
        (catalog::lorehold_searingscholar_b154(), 1),
        (catalog::lorehold_glyphbearer_b155(), 0),
        (catalog::lorehold_chronicler_b155(), 0),
        (catalog::lorehold_embermage_b158(), 0),
        (catalog::lorehold_spectermage_b158(), 0),
        (catalog::prismari_sparkmaster_b155(), 0),
        (catalog::inkling_scriptor_b158(), 1),
        (catalog::witherbloom_decantor_b158(), 1),
        (catalog::inkling_striplark_b155(), 1),
        (catalog::silverquill_liturgist_ii_b155(), 1),
        (catalog::witherbloom_bonebinder_b155(), 1),
        (catalog::lorehold_pyromancer_b155(), 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let _c = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let life0_before = g.players[0].life;
        let life1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1_before - 1 - 3,
            "{name}: opp -1 magecraft + -3 bolt");
        assert_eq!(g.players[0].life, life0_before + self_gain,
            "{name}: caster gains {self_gain}");
    }
}

#[test]
fn magecraft_draws_table() {
    for def in [
        catalog::quandrix_tidesinger_b154(),
        catalog::quandrix_researcher_b158(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _c = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // hand: -bolt +draw = same
        assert_eq!(g.players[0].hand.len(), hand_before,
            "{name}: magecraft drew 1 to replace the cast Bolt");
    }
}

#[test]
fn magecraft_may_draw_table() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    for def in [
        catalog::quandrix_mathwarden_b155(),
        catalog::prismari_tidepainter_b155(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let _c = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // Hand: -1 (cast Bolt) +1 (draw via magecraft May) = 0
        assert_eq!(g.players[0].hand.len(), hand_before, "{name}: may-draw taken");
    }
}

#[test]
fn magecraft_loots_table() {
    for def in [
        catalog::prismari_crashbinder_b154(),
        catalog::prismari_flameseeker_b154(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _c = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // hand: -bolt +draw -discard = -1
        assert_eq!(g.players[0].hand.len(), hand_before - 1,
            "{name}: looted, net -1 hand");
    }
}

#[test]
fn magecraft_loot_smoke_table() {
    // Loot with a near-empty hand — decider behavior varies, so just lock
    // in that the trigger resolves without crashing.
    for def in [
        catalog::prismari_pyroshaper_b155(),
        catalog::quandrix_equalist_b158(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _c = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
    }
}

#[test]
fn magecraft_scry_smoke_table() {
    // Scry doesn't change library size; lock in the trigger fires cleanly.
    for def in [
        catalog::fractal_magus_b155(),
        catalog::quandrix_scriptor_b155(),
        catalog::prismari_tinkertinker_b155(),
        catalog::quandrix_coursetaker_b158(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let _c = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let lib_before = g.players[0].library.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib_before,
            "{name}: scry leaves library size unchanged");
    }
}

// ── Consolidated ETB / cast-spell tables ───────────────────────────────────
// Shared shape: cast the card from hand with its exact mana, drain, assert.

#[test]
fn etb_gain_life_table() {
    for (def, colorless, colors, gain) in [
        (catalog::lorehold_cinderward_b154(), 2, &[Color::White][..], 3),
        (catalog::lorehold_watchspirit_b155(), 2, &[Color::White][..], 2),
        (catalog::witherbloom_cultivator_b155(), 2, &[Color::Black, Color::Green][..], 2),
        (catalog::witherbloom_mossdrinker_b155(), 3, &[Color::Green][..], 3),
        (catalog::lorehold_reverent_b155(), 0, &[Color::Red, Color::White][..], 2),
        (catalog::silverquill_inkwarden_b158(), 1, &[Color::White][..], 2),
        (catalog::lorehold_wallscribe_b158(), 1, &[Color::White][..], 1),
        (catalog::inkling_penlord_b158(), 3, &[Color::White, Color::Black][..], 3),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add_colorless(colorless);
        for &c in colors { g.players[0].mana_pool.add(c, 1); }
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life_before + gain,
            "{name}: ETB gains {gain} life");
    }
}

#[test]
fn cast_drain_each_opp_table() {
    // Creatures with ETB drains and drain sorceries share the same shape:
    // cast untargeted, opp loses N, caster gains N.
    for (def, colorless, colors, amount) in [
        (catalog::inkling_lifepoet_b155(), 1, &[Color::White, Color::Black][..], 2),
        (catalog::witherbloom_inkblossom_b155(), 0, &[Color::Black, Color::Green][..], 2),
        (catalog::witherbloom_bramblelord_ii_b155(), 3, &[Color::Black, Color::Green][..], 2),
        (catalog::silverquill_pen_crier_b158(), 1, &[Color::White, Color::Black][..], 2),
        (catalog::witherbloom_drainfeeder_b158(), 2, &[Color::Black, Color::Green][..], 2),
        (catalog::silverquill_battlescholar_b158(), 2, &[Color::White, Color::Black][..], 1),
        (catalog::inkling_pen_verseman_b155(), 3, &[Color::White, Color::Black][..], 1),
        (catalog::silverquill_inkdrain_b158(), 2, &[Color::White, Color::Black][..], 3),
        (catalog::silverquill_sphereturn_b154(), 2, &[Color::White, Color::Black][..], 4),
        (catalog::silverquill_vow_b158(), 0, &[Color::White, Color::Black][..], 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add_colorless(colorless);
        for &c in colors { g.players[0].mana_pool.add(c, 1); }
        let life0_before = g.players[0].life;
        let life1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1_before - amount, "{name}: opp -{amount}");
        assert_eq!(g.players[0].life, life0_before + amount, "{name}: caster +{amount}");
    }
}

#[test]
fn etb_mint_tokens_table() {
    for (def, colorless, colors, token, count) in [
        (catalog::pest_conjuror_b155(), 2, &[Color::Black, Color::Green][..], "Pest", 2),
        (catalog::pest_cultivator_ii_b158(), 0, &[Color::Black, Color::Green][..], "Pest", 1),
        (catalog::pest_swarmrider_b158(), 3, &[Color::Black, Color::Green][..], "Pest", 2),
        (catalog::lorehold_spirit_caller_b155(), 3, &[Color::Red, Color::White][..], "Spirit", 2),
        (catalog::lorehold_spirit_caster_b158(), 2, &[Color::Red, Color::White][..], "Spirit", 1),
        (catalog::lorehold_stonewright_b158(), 0, &[Color::Red, Color::White][..], "Spirit", 2),
        (catalog::prismari_treasure_spawner_b155(), 3, &[Color::Blue, Color::Red][..], "Treasure", 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add_colorless(colorless);
        for &c in colors { g.players[0].mana_pool.add(c, 1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let minted = g.battlefield.iter()
            .filter(|c| c.definition.name == token).count();
        assert_eq!(minted, count, "{name}: mints {count} {token} token(s)");
    }
}

#[test]
fn etb_self_counter_table() {
    for (def, colorless, colors) in [
        (catalog::inkling_skydrifter_b155(), 3, &[Color::White][..]),
        (catalog::quandrix_embodiment_b155(), 2, &[Color::Green, Color::Blue][..]),
        (catalog::fractal_skydweller_b158(), 3, &[Color::Green, Color::Blue][..]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add_colorless(colorless);
        for &c in colors { g.players[0].mana_pool.add(c, 1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let c = g.battlefield_find(id).expect("on bf");
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1,
            "{name}: enters with a +1/+1 counter");
    }
}

#[test]
fn etb_counters_on_target_table() {
    for (def, colorless, colors, target_def, n) in [
        (catalog::quandrix_riftguard_b154(), 3, &[Color::Green, Color::Blue][..],
            catalog::grizzly_bears(), 2),
        (catalog::inkling_vespermage_b155(), 2, &[Color::White, Color::Black][..],
            catalog::inkling_aspirant(), 1),
        (catalog::quandrix_logician_b155(), 1, &[Color::Green, Color::Blue][..],
            catalog::grizzly_bears(), 1),
        (catalog::quandrix_echo_b158(), 1, &[Color::Green, Color::Blue][..],
            catalog::grizzly_bears(), 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let tgt = g.add_card_to_battlefield(0, target_def);
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add_colorless(colorless);
        for &c in colors { g.players[0].mana_pool.add(c, 1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(tgt)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let counters = g.battlefield_find(tgt).map(|c| {
            c.counters.iter()
                .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
                .map(|(_, v)| *v).sum::<u32>()
        }).unwrap_or(0);
        assert_eq!(counters, n, "{name}: target gets {n} +1/+1 counter(s)");
    }
}

#[test]
fn removal_kills_opp_bear_table() {
    // 2+ damage or destroy on a 2/2 — the bear dies (or at least eats 2).
    for (def, colorless, colors) in [
        (catalog::prismari_combustion_b155(), 0, &[Color::Blue, Color::Red][..]),
        (catalog::lorehold_stoneflame_b158(), 1, &[Color::Red][..]),
        (catalog::silverquill_eulogist_b155(), 1, &[Color::White, Color::Black][..]),
        (catalog::lorehold_pyrebolt_b155(), 0, &[Color::Red][..]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add_colorless(colorless);
        for &c in colors { g.players[0].mana_pool.add(c, 1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none()
            || g.battlefield_find(bear).map(|c| c.damage).unwrap_or(0) >= 2,
            "{name}: bear dead or took at least 2 damage");
    }
}

#[test]
fn etb_scry_table() {
    for (def, colorless, colors) in [
        (catalog::prismari_calligrapher_b154(), 2, &[Color::Blue][..]),
        (catalog::silverquill_penkeeper_b158(), 2, &[Color::White][..]),
        (catalog::quandrix_inquirer_b158(), 1, &[Color::Blue][..]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add_colorless(colorless);
        for &c in colors { g.players[0].mana_pool.add(c, 1); }
        let lib_before = g.players[0].library.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        // Scry doesn't change library size (no draw).
        assert_eq!(g.players[0].library.len(), lib_before, "{name}: scry only");
    }
}

#[test]
fn etb_cast_stat_lock_in_table() {
    for (def, colorless, colors, p, t) in [
        (catalog::silverquill_manuscriber_b155(), 2, &[Color::White][..], 2, 3),
        (catalog::quandrix_cartographer_b155(), 1, &[Color::Green][..], 2, 2),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add_colorless(colorless);
        for &c in colors { g.players[0].mana_pool.add(c, 1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let c = g.battlefield_find(id).expect("on bf");
        assert_eq!(c.definition.power, p, "{name} power");
        assert_eq!(c.definition.toughness, t, "{name} toughness");
    }
}

#[test]
fn inkling_stat_and_keyword_lock_in_table() {
    for (def, p, t, keywords, is_inkling) in [
        (catalog::inkling_bookwarden_b154(), 2, 3,
            &[Keyword::Flying, Keyword::Lifelink][..], true),
        (catalog::inkling_pinionguard_b158(), 2, 2,
            &[Keyword::Flying, Keyword::Lifelink][..], true),
        (catalog::inkling_aerogate_b158(), 1, 3,
            &[Keyword::Flying, Keyword::Vigilance][..], true),
        (catalog::inkling_veilwarden_b158(), 4, 4,
            &[Keyword::Flying, Keyword::Lifelink][..], false),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let c = g.battlefield_find(id).expect("on bf");
        assert_eq!(c.definition.power, p, "{name} power");
        assert_eq!(c.definition.toughness, t, "{name} toughness");
        for kw in keywords {
            assert!(c.definition.keywords.contains(kw), "{name} has {kw:?}");
        }
        if is_inkling {
            assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Inkling),
                "{name} is an Inkling");
        }
    }
}

#[test]
fn etb_draws_one_table() {
    for (def, colorless, colors) in [
        (catalog::quandrix_forecaster_b155(), 1, &[Color::Green, Color::Blue][..]),
        (catalog::quandrix_coursebearer_b155(), 2, &[Color::Green, Color::Blue][..]),
        (catalog::prismari_flowcaster_b155(), 2, &[Color::Blue][..]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add_colorless(colorless);
        for &c in colors { g.players[0].mana_pool.add(c, 1); }
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        // Hand: -1 (cast) +1 (draw) = 0 net
        assert_eq!(g.players[0].hand.len(), hand_before, "{name}: ETB drew 1");
    }
}

// ── Individually-shaped card tests ─────────────────────────────────────────

#[test]
fn quandrix_calculation_b154_mints_four_four_fractal_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::quandrix_calculation_b154());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Calculation castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Fractal").collect();
    assert_eq!(fractals.len(), 1);
    let counters: u32 = fractals[0].counters.iter()
        .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
        .map(|(_, n)| *n).sum();
    assert_eq!(counters, 4, "0/0 Fractal + 4 counters = 4/4");
    assert_eq!(g.players[0].hand.len(), hand_before, "spell out, draw 1 in");
}

#[test]
fn lorehold_strikeritual_b154_burns_and_mints_spirit() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::lorehold_strikeritual_b154());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Strikeritual castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits, 1, "Mints exactly one Spirit token");
}

#[test]
fn quandrix_wavebreaker_b154_etb_bounces_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wb = g.add_card_to_hand(0, catalog::quandrix_wavebreaker_b154());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: wb, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wavebreaker castable");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear returned to opp's hand");
}

#[test]
fn quandrix_bloomguard_b154_etb_fans_counters_on_each_friendly_creature() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bg = g.add_card_to_hand(0, catalog::quandrix_bloomguard_b154());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bg, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bloomguard castable");
    drain_stack(&mut g);
    for id in [b1, b2] {
        let c = g.battlefield_find(id).unwrap();
        let counters: u32 = c.counters.iter()
            .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n).sum();
        assert_eq!(counters, 1, "Each friendly bear gets a +1/+1 counter");
    }
}

#[test]
fn prismari_inferno_b154_burns_target_for_five() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::prismari_inferno_b154());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(serra)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inferno castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == serra),
        "Serra Angel (4 toughness) takes 5 → dies");
}

#[test]
fn prismari_sparkglyph_b154_burns_target_for_three() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::prismari_sparkglyph_b154());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkglyph castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 3);
}

#[test]
fn prismari_stormbreaker_b154_etb_burns_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let sb = g.add_card_to_hand(0, catalog::prismari_stormbreaker_b154());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: sb, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Stormbreaker castable");
    drain_stack(&mut g);
    // The ETB trigger auto-targets — typically the opponent gets the
    // 2 damage. Caster also draws 1 (Island onto hand).
    assert!(g.players[1].life < life1_before
            || g.battlefield.iter().any(|c| c.damage >= 2 && c.id != sb),
        "ETB dealt 2 damage somewhere — either to opp or a creature");
    // Hand: -spell +draw = same total
    assert_eq!(g.players[0].hand.len(), hand_before,
        "ETB drew a card to replace the cast spell");
}

// ── batch 154 helper shortcut lock-in tests ────────────────────────────────

#[test]
fn shortcut_magecraft_mint_pest_uses_magecraft_trigger_with_create_token_body() {
    // Lock in that magecraft_mint_pest() builds a magecraft trigger
    // (SpellCast scope + IS filter) whose body is Effect::CreateToken
    // with the stx_pest_token() definition. Future refactors can't
    // collapse this onto magecraft_mint_inkling or magecraft_treasure.
    use crabomination::effect::shortcut::magecraft_mint_pest;
    let trig = magecraft_mint_pest();
    assert_eq!(trig.event.kind, crabomination::effect::EventKind::SpellCast);
    match trig.effect {
        crabomination::effect::Effect::CreateToken { count, ref definition, .. } => {
            assert_eq!(count, crabomination::effect::Value::Const(1));
            assert_eq!(definition.name, "Pest");
        }
        _ => panic!("body must be CreateToken with Pest definition"),
    }
}

#[test]
fn shortcut_magecraft_mint_inkling_uses_inkling_token() {
    // Lock in that magecraft_mint_inkling() mints a 1/1 W/B flying
    // Inkling. Distinguishes from magecraft_mint_pest (B/G Pest) and
    // magecraft_mint_spirit (R/W Spirit).
    use crabomination::effect::shortcut::magecraft_mint_inkling;
    let trig = magecraft_mint_inkling();
    assert_eq!(trig.event.kind, crabomination::effect::EventKind::SpellCast);
    match trig.effect {
        crabomination::effect::Effect::CreateToken { ref definition, .. } => {
            assert_eq!(definition.name, "Inkling");
            assert!(definition.keywords.contains(&Keyword::Flying));
        }
        _ => panic!("body must be CreateToken with Inkling definition"),
    }
}

#[test]
fn shortcut_magecraft_mint_fractal_seq_creates_token_then_stamps_counters() {
    // Lock in that magecraft_mint_fractal(N) is a Seq[CreateToken,
    // AddCounter(LastCreatedToken, +1/+1, N)] body — the printed Quandrix
    // "create a 0/0 Fractal with N +1/+1 counters" pattern.
    use crabomination::effect::shortcut::magecraft_mint_fractal;
    let trig = magecraft_mint_fractal(2);
    assert_eq!(trig.event.kind, crabomination::effect::EventKind::SpellCast);
    match &trig.effect {
        crabomination::effect::Effect::Seq(steps) => {
            assert_eq!(steps.len(), 2);
            assert!(matches!(steps[0], crabomination::effect::Effect::CreateToken { .. }));
            match &steps[1] {
                crabomination::effect::Effect::AddCounter { what, kind, amount } => {
                    assert!(matches!(what, crabomination::effect::Selector::LastCreatedToken));
                    assert_eq!(*kind, CounterType::PlusOnePlusOne);
                    assert_eq!(*amount, crabomination::effect::Value::Const(2));
                }
                _ => panic!("step 1 must be AddCounter"),
            }
        }
        _ => panic!("body must be Seq[CreateToken, AddCounter]"),
    }
}

#[test]
fn shortcut_magecraft_mint_and_drain_seq_mints_then_drains() {
    // Lock in that magecraft_mint_and_drain(def, count, amount) builds a
    // magecraft trigger whose body is Seq[CreateToken(count), Drain(amount)]
    // — the Pest-aristocrats "mint a body then drain the table per spell"
    // shape. Mint must precede the drain so the token is on the battlefield
    // before any "if you gained life" / sacrifice payoff sees the drain.
    use crabomination::effect::shortcut::magecraft_mint_and_drain;
    let trig = magecraft_mint_and_drain(crabomination::catalog::stx_pest_token(), 1, 2);
    assert_eq!(trig.event.kind, crabomination::effect::EventKind::SpellCast);
    match &trig.effect {
        crabomination::effect::Effect::Seq(steps) => {
            assert_eq!(steps.len(), 2);
            match &steps[0] {
                crabomination::effect::Effect::CreateToken { count, definition, .. } => {
                    assert_eq!(*count, crabomination::effect::Value::Const(1));
                    assert_eq!(definition.name, "Pest");
                }
                _ => panic!("step 0 must be CreateToken"),
            }
            match &steps[1] {
                crabomination::effect::Effect::Drain { amount, .. } => {
                    assert_eq!(*amount, crabomination::effect::Value::Const(2));
                }
                _ => panic!("step 1 must be Drain"),
            }
        }
        _ => panic!("body must be Seq[CreateToken, Drain]"),
    }
}

#[test]
fn shortcut_dies_mint_pest_uses_creature_died_self_source() {
    // Lock in that dies_mint_pest() builds a CreatureDied/SelfSource
    // trigger whose body mints a Pest. Pulls the self-replacing-Pest
    // pattern (Pest Swarmer, future Pest cards) onto a one-liner.
    use crabomination::effect::shortcut::dies_mint_pest;
    let trig = dies_mint_pest();
    assert_eq!(trig.event.kind, crabomination::effect::EventKind::CreatureDied);
    assert!(matches!(trig.event.scope, crabomination::effect::EventScope::SelfSource));
    match trig.effect {
        crabomination::effect::Effect::CreateToken { ref definition, .. } => {
            assert_eq!(definition.name, "Pest");
        }
        _ => panic!("body must be CreateToken with Pest definition"),
    }
}

#[test]
fn shortcut_on_attack_mint_lorehold_spirit_uses_attacks_self_source() {
    // Lock in that on_attack_mint_lorehold_spirit() builds an
    // Attacks/SelfSource trigger whose body mints a 2/2 R/W Spirit.
    // Distinguishes from on_attack_create_token<T> which is the generic
    // form — this shortcut bakes the Lorehold Spirit token definition.
    use crabomination::effect::shortcut::on_attack_mint_lorehold_spirit;
    let trig = on_attack_mint_lorehold_spirit();
    assert_eq!(trig.event.kind, crabomination::effect::EventKind::Attacks);
    assert!(matches!(trig.event.scope, crabomination::effect::EventScope::SelfSource));
    match trig.effect {
        crabomination::effect::Effect::CreateToken { ref definition, .. } => {
            assert_eq!(definition.name, "Spirit");
            assert_eq!(definition.power, 2);
            assert_eq!(definition.toughness, 2);
        }
        _ => panic!("body must be CreateToken with Lorehold Spirit definition"),
    }
}

#[test]
fn shortcut_magecraft_add_counter_self_targets_self_with_plus_one_plus_one() {
    // Lock in that magecraft_add_counter_self() is a magecraft trigger
    // whose body is AddCounter(Selector::This, +1/+1, 1). Prevents
    // future refactors from accidentally collapsing onto
    // magecraft_add_counter_to_friendly (which targets a friendly).
    use crabomination::effect::shortcut::magecraft_add_counter_self;
    let trig = magecraft_add_counter_self();
    assert_eq!(trig.event.kind, crabomination::effect::EventKind::SpellCast);
    match trig.effect {
        crabomination::effect::Effect::AddCounter { ref what, kind, amount } => {
            assert!(matches!(what, crabomination::effect::Selector::This));
            assert_eq!(kind, CounterType::PlusOnePlusOne);
            assert_eq!(amount, crabomination::effect::Value::Const(1));
        }
        _ => panic!("body must be AddCounter targeting Self"),
    }
}

// ── Batch 155: attack-trigger card tests ────────────────────────────────────

#[test]
fn pest_acolyte_b155_gains_one_life_on_attack() {
    let mut g = two_player_game();
    let pid = g.add_card_to_battlefield(0, catalog::pest_acolyte_b155());
    g.clear_sickness(pid);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pid,
        target: AttackTarget::Player(1),
    }]))
    .expect("attacker declare");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn lorehold_spiritforge_b155_attack_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_spiritforge_b155());
    g.clear_sickness(id);
    let spirits_before = g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit").count();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attacker declared");
    drain_stack(&mut g);
    let spirits_after = g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits_after, spirits_before + 1, "Attack mints a Spirit");
}

// ── batch 155 — Silverquill cards ──────────────────────────────────────────

#[test]
fn silverquill_adjudicator_b155_exiles_creature_and_drains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let adj = g.add_card_to_hand(0, catalog::silverquill_adjudicator_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: adj, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Adjudicator castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear exiled");
    assert!(g.exile.iter().any(|c| c.id == bear), "Bear in exile");
    assert_eq!(g.players[1].life, life1_before - 1);
    assert_eq!(g.players[0].life, life0_before + 1);
}

#[test]
fn silverquill_quillplay_b155_drains_one_and_mints_inkling() {
    let mut g = two_player_game();
    let qp = g.add_card_to_hand(0, catalog::silverquill_quillplay_b155());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: qp, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Quillplay castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1);
    assert_eq!(g.players[0].life, life0_before + 1);
    let inklings = g.battlefield.iter()
        .filter(|c| c.definition.name == "Inkling").count();
    assert_eq!(inklings, 1, "Quillplay mints an Inkling token");
}

#[test]
fn silverquill_curatorial_b155_drains_and_reanimates() {
    let mut g = two_player_game();
    let dead_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let cur = g.add_card_to_hand(0, catalog::silverquill_curatorial_b155());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: cur, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Curatorial castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2, "Opp -2 drain");
    assert!(g.battlefield_find(dead_bear).is_some(), "Bear reanimated to bf");
}

#[test]
fn silverquill_recital_b155_each_opp_sacs_and_mints() {
    let mut g = two_player_game();
    let _opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let recital = g.add_card_to_hand(0, catalog::silverquill_recital_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: recital, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Recital castable");
    drain_stack(&mut g);
    let opp_creatures = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.is_creature()).count();
    assert_eq!(opp_creatures, 0, "Opp sacrificed their creature");
    let my_inklings = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Inkling").count();
    assert_eq!(my_inklings, 1, "Recital mints an Inkling");
    assert_eq!(g.players[0].life, life0_before + 1, "+1 life");
}

#[test]
fn silverquill_caesura_b155_taps_creature_and_cantrips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let caes = g.add_card_to_hand(0, catalog::silverquill_caesura_b155());
    g.players[0].mana_pool.add(Color::White, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: caes, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Caesura castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).expect("on bf").tapped, "Bear tapped");
    // Hand: -1 (cast) +1 (draw) = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── batch 155 — Witherbloom cards ──────────────────────────────────────────

#[test]
fn witherbloom_sapling_b155_activation_grows_self() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::witherbloom_sapling_b155());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Activation succeeds");
    drain_stack(&mut g);
    let counters = g.battlefield_find(s).map(|c| {
        c.counters.iter()
            .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n).sum::<u32>()
    }).unwrap_or(0);
    assert_eq!(counters, 1, "Activation adds +1/+1 counter");
}

#[test]
fn witherbloom_tutor_b155_searches_creature_to_hand() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear_in_lib = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear_in_lib))]));
    let tutor = g.add_card_to_hand(0, catalog::witherbloom_tutor_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: tutor, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Tutor castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 2, "-2 life paid");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_in_lib),
        "Bear searched to hand");
}

// ── batch 155 — Lorehold cards ─────────────────────────────────────────────

#[test]
fn lorehold_pyrescholar_b155_etb_pings_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_pyrescholar_b155());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pyrescholar castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1,
        "Pyrescholar ETB pings opp player for 1");
}

#[test]
fn lorehold_battlechant_b155_deals_damage_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bc = g.add_card_to_hand(0, catalog::lorehold_battlechant_b155());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bc, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battlechant castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none() ||
        g.battlefield_find(bear).map(|c| c.damage).unwrap_or(0) == 2,
        "Bear takes 2 damage (or dies from SBA)");
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_ancestralist_b155_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_ancestralist_b155());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Ancestralist castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead),
        "Bear returned to hand from graveyard");
}

#[test]
fn prismari_surge_b155_draw_2_discard_1() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_surge_b155());
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {1}{U}{R}");
    drain_stack(&mut g);
    // -1 (cast) +2 (draw) -1 (discard) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn elemental_whirlwind_b155_damages_each_opponent_and_draws() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::elemental_whirlwind_b155());
    g.add_card_to_library(1, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_before = g.players[1].life;
    let p1_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {3}{U}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 4);
    assert_eq!(g.players[1].hand.len(), p1_hand_before + 1);
}

// ── Batch 155: CR rule lock-in tests ────────────────────────────────────────

#[test]
fn cr_506_5_attacks_trigger_fires_per_attacker_in_batch() {
    // CR 506.5: when multiple attackers are declared in one batch, each
    // per-attacker "whenever this creature attacks" trigger fires for
    // its own attacker. Fixture: two Quandrix Reckoners ("Whenever this
    // creature attacks, put a +1/+1 counter on it") — each attacker
    // should pick up its own +1/+1 counter. (Sparring Regimen no longer
    // works as the fixture: its real oracle is "whenever you attack",
    // which fires once per combat with a single target.)
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::quandrix_reckoner());
    let bear2 = g.add_card_to_battlefield(0, catalog::quandrix_reckoner());
    g.clear_sickness(bear1);
    g.clear_sickness(bear2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: bear1, target: AttackTarget::Player(1) },
        Attack { attacker: bear2, target: AttackTarget::Player(1) },
    ]))
    .expect("bears can attack");
    drain_stack(&mut g);
    let b1 = g.battlefield.iter().find(|c| c.id == bear1).unwrap();
    let b2 = g.battlefield.iter().find(|c| c.id == bear2).unwrap();
    assert_eq!(b1.counter_count(CounterType::PlusOnePlusOne), 1,
        "first attacker should pick up one +1/+1 counter");
    assert_eq!(b2.counter_count(CounterType::PlusOnePlusOne), 1,
        "second attacker should pick up one +1/+1 counter");
}

#[test]
fn cr_603_attacks_trigger_broadcast_skips_opponent_anchors() {
    // CR 603.6 — "Whenever YOU attack" triggers from a YourControl
    // scoped Attacks listener should NOT fire when the OPPONENT
    // declares attackers, because their broadcast walks the opponent's
    // permanents (not yours). Lock-in: opponent attacks with their own
    // creature, but my Sparring Regimen sits on the bf — no counter
    // should land on the opponent's attacker.
    let mut g = two_player_game();
    let _regimen = g.add_card_to_battlefield(0, catalog::sparring_regimen());
    drain_stack(&mut g);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(0),
    }]))
    .expect("opponent's bear can attack");
    drain_stack(&mut g);
    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        bear_card.counter_count(CounterType::PlusOnePlusOne),
        0,
        "opponent's attacker should NOT get my Regimen's counter"
    );
}

#[test]
fn cr_118_8_exile_from_graveyard_cost_pre_flight_no_mana_burned() {
    // CR 118.8 — "If a player can't pay the costs of a spell or
    // ability, they can't cast or activate it." Lock-in: when no
    // graveyard card matches the exile-from-gy cost, activation is
    // rejected before mana / tap is committed.
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::lorehold_pledgemage());
    g.clear_sickness(pm);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let pool_red = g.players[0].mana_pool.amount(Color::Red);
    let pool_white = g.players[0].mana_pool.amount(Color::White);
    let pool_colorless = g.players[0].mana_pool.colorless_amount();
    // No card in graveyard — activation must be rejected and mana
    // pool must NOT be drained.
    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: pm,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    });
    assert!(result.is_err(), "must reject without legal gy-exile target");
    // Mana pool unchanged.
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), pool_red);
    assert_eq!(g.players[0].mana_pool.amount(Color::White), pool_white);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), pool_colorless);
    // Source not tapped.
    let pm_card = g.battlefield.iter().find(|c| c.id == pm).unwrap();
    assert!(!pm_card.tapped, "source should not be tapped on failed activation");
}

// ── Batch 156: attack-anchor lock-in tests (multi-attacker fan-out) ────────

#[test]
fn lorehold_banner_b156_pumps_each_attacker_in_batch() {
    // Push c4b7b14's batch-fanout fix: a multi-attacker swing fans the
    // Lorehold Banner's "another attacks" trigger to each attacker.
    let mut g = two_player_game();
    let _banner = g.add_card_to_battlefield(0, catalog::lorehold_banner_b156());
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(b1);
    g.clear_sickness(b2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: b1, target: AttackTarget::Player(1) },
        Attack { attacker: b2, target: AttackTarget::Player(1) },
    ]))
    .expect("both bears can attack");
    drain_stack(&mut g);
    let bear1 = g.battlefield.iter().find(|c| c.id == b1).unwrap();
    let bear2 = g.battlefield.iter().find(|c| c.id == b2).unwrap();
    // Each attacker should be 3/2 EOT (2/2 printed + 1/+0 EOT).
    assert_eq!(bear1.power(), 3, "attacker 1 should be pumped");
    assert_eq!(bear2.power(), 3, "attacker 2 should be pumped");
}

#[test]
fn lorehold_marshal_b156_gains_life_per_other_attacker() {
    let mut g = two_player_game();
    let _marshal = g.add_card_to_battlefield(0, catalog::lorehold_marshal_b156());
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(b1);
    g.clear_sickness(b2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: b1, target: AttackTarget::Player(1) },
        Attack { attacker: b2, target: AttackTarget::Player(1) },
    ]))
    .expect("both bears attack");
    drain_stack(&mut g);
    // Marshal fires once per attacker (both bears are "other"), so
    // life should go up by 2.
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn silverquill_tactician_b156_mints_inkling_per_other_attacker() {
    let mut g = two_player_game();
    let _tact = g.add_card_to_battlefield(0, catalog::silverquill_tactician_b156());
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(b1);
    g.clear_sickness(b2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: b1, target: AttackTarget::Player(1) },
        Attack { attacker: b2, target: AttackTarget::Player(1) },
    ]))
    .expect("both bears attack");
    drain_stack(&mut g);
    let inklings = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Inkling")
        .count();
    assert_eq!(inklings, 2, "should mint two Inklings (one per other-attacker)");
}

#[test]
fn quandrix_mathematician_ii_b156_counters_each_attacker() {
    let mut g = two_player_game();
    let _m = g.add_card_to_battlefield(0, catalog::quandrix_mathematician_ii_b156());
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(b1);
    g.clear_sickness(b2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: b1, target: AttackTarget::Player(1) },
        Attack { attacker: b2, target: AttackTarget::Player(1) },
    ]))
    .expect("both bears attack");
    drain_stack(&mut g);
    let bear1 = g.battlefield.iter().find(|c| c.id == b1).unwrap();
    let bear2 = g.battlefield.iter().find(|c| c.id == b2).unwrap();
    assert_eq!(bear1.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(bear2.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn pest_hivebreeder_b156_mints_pest_on_other_creature_death() {
    // Use Lightning Bolt to kill the pest via real combat-damage / SBA
    // flow so the unified dispatcher fires AnotherOfYours triggers
    // (`remove_to_graveyard_with_triggers` only handles SelfSource).
    let mut g = two_player_game();
    let _hb = g.add_card_to_battlefield(0, catalog::pest_hivebreeder_b156());
    let pest = g.add_card_to_battlefield(0, catalog::pest_acolyte_b155());
    drain_stack(&mut g);
    let pests_before = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest").count();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(pest)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest").count();
    assert_eq!(pests_after, pests_before + 1, "should mint a Pest on death");
}

// ── Bot AI lock-ins (push: planeswalker attack heuristic) ──────────────────

#[test]
fn bot_attacks_finishable_planeswalker_with_proper_power() {
    // The bot's DeclareAttackers handler now redirects attacks at an
    // opponent's planeswalker when our total attacking power can finish
    // it off in one swing. Lock-in: with a 5-loyalty PW + a 5-power
    // attacker (Grizzly Bears pumped to 5/5 with +1/+1 counters), the
    // bot should aim AT the walker.
    use crabomination::server::bot::{Bot, RandomBot};
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(beater);
    // Pump bear to 5/5 via three +1/+1 counters.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == beater) {
        c.add_counters(CounterType::PlusOnePlusOne, 3);
    }
    // Opp's planeswalker (Dellian Fel: 5 base loyalty).
    let pw = g.add_card_to_battlefield(1, catalog::professor_dellian_fel());
    let loyalty = g.battlefield.iter().find(|c| c.id == pw).unwrap()
        .counter_count(CounterType::Loyalty);
    assert!(loyalty >= 1, "PW should have loyalty");

    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let mut bot = RandomBot::new();
    let action = bot.next_action(&g, 0);
    // Bot should DeclareAttackers with the beater aimed at the walker
    // since 5 power finishes off a 5-loyalty walker.
    match action {
        Some(GameAction::DeclareAttackers(attacks)) => {
            let aimed_at_pw = attacks.iter().any(|a| {
                matches!(a.target, AttackTarget::Planeswalker(p) if p == pw)
            });
            assert!(
                aimed_at_pw,
                "bot should aim at the finishable PW (loyalty {loyalty}); got {attacks:?}",
            );
        }
        other => panic!("expected DeclareAttackers, got {other:?}"),
    }
}

#[test]
fn bot_stifles_a_threatening_opponent_ability() {
    // Server/bot: react to an opponent ability on the stack with Stifle.
    use crabomination::server::bot::{Bot, RandomBot};
    let mut g = two_player_game();
    // P0 casts Devourer of Destiny — its on-cast Scry trigger lands on the stack.
    let dev = g.add_card_to_hand(0, catalog::devourer_of_destiny());
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpell {
        card_id: dev, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    // Seat 1 holds Stifle and gets priority with the ability on the stack.
    let stifle = g.add_card_to_hand(1, catalog::stifle());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 1;
    let mut bot = RandomBot::new();
    match bot.next_action(&g, 1) {
        Some(GameAction::CastSpell { card_id, target, .. }) => {
            assert_eq!(card_id, stifle, "bot casts Stifle");
            assert_eq!(target, Some(Target::Permanent(dev)),
                "Stifle targets the ability's source");
        }
        other => panic!("expected the bot to Stifle the trigger, got {other:?}"),
    }
}

#[test]
fn bot_does_not_aim_at_walker_too_tough_to_finish() {
    // Symmetric lock-in: when the bot's attacking power is below the
    // walker's loyalty, the bot should NOT throw attackers at the
    // walker.
    use crabomination::server::bot::{Bot, RandomBot};
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(beater);
    let pw = g.add_card_to_battlefield(1, catalog::professor_dellian_fel());
    // Walker has full base loyalty (5); our 2-power bear can't finish it.
    let loyalty = g.battlefield.iter().find(|c| c.id == pw).unwrap()
        .counter_count(CounterType::Loyalty);
    let our_power: i32 = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature())
        .map(|c| c.power())
        .sum();
    assert!((our_power as u32) < loyalty);

    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let mut bot = RandomBot::new();
    let action = bot.next_action(&g, 0);
    match action {
        Some(GameAction::DeclareAttackers(attacks)) => {
            let aimed_at_pw = attacks.iter().any(|a| {
                matches!(a.target, AttackTarget::Planeswalker(_))
            });
            assert!(
                !aimed_at_pw,
                "bot should NOT aim at a walker it can't finish off; got {attacks:?}",
            );
        }
        other => panic!("expected DeclareAttackers, got {other:?}"),
    }
}

#[test]
fn bot_prefers_surviving_trade_over_deathtouch_attacker() {
    // With one 3/3 blocker and two 2/2 attackers — one vanilla, one with
    // deathtouch — the bot should block the vanilla one (3/3 kills it and
    // survives) rather than the deathtouch one (which would kill the 3/3).
    use crabomination::card::Keyword;
    use crabomination::server::bot;
    let mut g = two_player_game();
    // Deathtouch attacker declared first (so the old tie-break would pick it).
    let dt_atk = {
        let mut d = catalog::grizzly_bears();
        d.name = "Venom Bear";
        d.keywords = vec![Keyword::Deathtouch];
        g.add_card_to_battlefield(1, d)
    };
    let vanilla_atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(dt_atk);
    g.clear_sickness(vanilla_atk);
    g.attacking.push(Attack { attacker: dt_atk, target: AttackTarget::Player(0) });
    g.attacking.push(Attack { attacker: vanilla_atk, target: AttackTarget::Player(0) });
    let blocker = {
        let mut d = catalog::grizzly_bears();
        d.name = "Wall Bear";
        d.power = 3;
        d.toughness = 3;
        g.add_card_to_battlefield(0, d)
    };
    g.clear_sickness(blocker);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].1, vanilla_atk,
        "bot blocks the vanilla attacker it can kill and survive, not the deathtouch one");
}

#[test]
fn bot_blocks_smart_value_trade() {
    // Push (this run): smarter blocker AI. With one 3/3 attacker
    // attacking us and a 2/2 blocker, the blocker should still chump
    // (life-threatened logic) when our life is low. With a 3/4
    // blocker (clean kill, survives), it should block.
    use crabomination::server::bot;
    let mut g = two_player_game();
    g.players[0].life = 5;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareBlockers;
    // Opp's 4/3 attacker.
    let beater_def = {
        let mut def = catalog::grizzly_bears();
        def.name = "Big Bear";
        def.power = 4;
        def.toughness = 3;
        def
    };
    let attacker = g.add_card_to_battlefield(1, beater_def);
    g.clear_sickness(attacker);
    g.attacking.push(Attack {
        attacker,
        target: AttackTarget::Player(0),
    });
    // Our 3/4 blocker — clean kill, survives.
    let blocker_def = {
        let mut def = catalog::grizzly_bears();
        def.name = "Wall Bear";
        def.power = 3;
        def.toughness = 4;
        def
    };
    let blocker = g.add_card_to_battlefield(0, blocker_def);
    g.clear_sickness(blocker);

    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert_eq!(blocks.len(), 1, "should block the attacker");
    assert_eq!(blocks[0].0, blocker);
    assert_eq!(blocks[0].1, attacker);
}

#[test]
fn bot_gang_blocks_to_kill_when_life_threatened() {
    // Facing lethal from a 6/6 no single blocker can kill, the bot should
    // gang two 3/3s onto it (combined power 6 ≥ toughness 6) to remove the
    // threat rather than scatter chumps.
    use crabomination::server::bot;
    let mut g = two_player_game();
    g.players[0].life = 5; // a 6-power attacker is lethal
    let big = {
        let mut d = catalog::grizzly_bears();
        d.name = "Huge Bear";
        d.power = 6;
        d.toughness = 6;
        d
    };
    let attacker = g.add_card_to_battlefield(1, big);
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let mk = |g: &mut crabomination::game::GameState| {
        let mut d = catalog::grizzly_bears();
        d.power = 3;
        d.toughness = 3;
        let id = g.add_card_to_battlefield(0, d);
        g.clear_sickness(id);
        id
    };
    let b1 = mk(&mut g);
    let b2 = mk(&mut g);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert_eq!(blocks.len(), 2, "both blockers gang the lethal attacker");
    assert!(blocks.iter().all(|(_, a)| *a == attacker));
    let blockers: std::collections::HashSet<_> = blocks.iter().map(|(b, _)| *b).collect();
    assert!(blockers.contains(&b1) && blockers.contains(&b2));
}

#[test]
fn bot_assigns_two_blockers_to_a_menace_attacker() {
    // CR 509.1b — a Menace 4/4 must be blocked by two creatures. With two
    // idle 2/3 blockers the bot must commit both (a lone block is illegal).
    use crabomination::card::Keyword;
    use crabomination::server::bot;
    let mut g = two_player_game();
    g.players[0].life = 4; // lethal pressure so the bot wants to block
    let menace = {
        let mut d = catalog::grizzly_bears();
        d.name = "Menace Bear";
        d.power = 4;
        d.toughness = 4;
        d.keywords = vec![Keyword::Menace];
        d
    };
    let attacker = g.add_card_to_battlefield(1, menace);
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let mk = |g: &mut crabomination::game::GameState| {
        let mut d = catalog::grizzly_bears();
        d.power = 2;
        d.toughness = 3;
        let id = g.add_card_to_battlefield(0, d);
        g.clear_sickness(id);
        id
    };
    mk(&mut g);
    mk(&mut g);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    let on_menace = blocks.iter().filter(|(_, a)| *a == attacker).count();
    assert!(on_menace == 0 || on_menace >= 2,
        "Menace attacker gets 0 or ≥2 blockers, never a lone (illegal) block; got {on_menace}");
    assert_eq!(on_menace, 2, "two idle blockers available → commit both");
}

#[test]
fn bot_drops_lone_block_on_menace_when_no_second_blocker() {
    // With only one creature available, a Menace attacker can't be legally
    // blocked — the bot must leave it unblocked rather than emit an illegal
    // single block.
    use crabomination::card::Keyword;
    use crabomination::server::bot;
    let mut g = two_player_game();
    g.players[0].life = 3;
    let menace = {
        let mut d = catalog::grizzly_bears();
        d.name = "Menace Bear";
        d.power = 3;
        d.toughness = 3;
        d.keywords = vec![Keyword::Menace];
        d
    };
    let attacker = g.add_card_to_battlefield(1, menace);
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let lone = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(lone);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert!(blocks.iter().all(|(_, a)| *a != attacker),
        "no legal block on the Menace attacker → leave it unblocked");
}

// ── CR 603.4 — intervening 'if' clause re-check at resolve time ────────────

#[test]
fn cr_603_4_intervening_if_re_checked_at_resolve_time() {
    // CR 603.4 — "If the condition isn't true at that time [resolve],
    // the ability is removed from the stack and does nothing."
    //
    // Push a trigger directly onto the stack with an intervening_if
    // predicate that's currently false, then drain. Verify the body
    // never runs (life total unchanged).
    use crabomination::card::Predicate;
    use crabomination::effect::PlayerRef;
    use crabomination::game::types::StackItem;

    let mut g = two_player_game();
    // Predicate that will be false: "your hand size is at least 100"
    let pred = Predicate::ValueAtLeast(
        crabomination::effect::Value::HandSizeOf(PlayerRef::You),
        crabomination::effect::Value::Const(100),
    );
    // Body: gain 50 life — would be observable if it ran.
    let body = crabomination::effect::Effect::GainLife {
        who: crabomination::effect::Selector::You,
        amount: crabomination::effect::Value::Const(50),
    };
    // Manufacture a trigger source so the resolution context has a
    // valid `source` id.
    let src = g.add_card_to_battlefield(0, catalog::island());
    g.stack.push(StackItem::Trigger {
        source: src,
        controller: 0,
        effect: Box::new(body),
        target: None,
        mode: None,
        x_value: 0,
        converged_value: 0,
        trigger_source: None,
        mana_spent: 0,
        event_amount: 0,
        intervening_if: Some(pred),
        additional_targets: Vec::new(),
        activated: false,
    });
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before,
        "Trigger fizzled per CR 603.4 — body didn't run");
}

#[test]
fn cr_603_4_intervening_if_runs_when_true_at_resolve_time() {
    // Sanity check: trigger with intervening_if = Some(true_predicate)
    // resolves normally.
    use crabomination::card::Predicate;
    use crabomination::effect::PlayerRef;
    use crabomination::game::types::StackItem;

    let mut g = two_player_game();
    // True predicate: "your hand size is at least 0" (always true)
    let pred = Predicate::ValueAtLeast(
        crabomination::effect::Value::HandSizeOf(PlayerRef::You),
        crabomination::effect::Value::Const(0),
    );
    let body = crabomination::effect::Effect::GainLife {
        who: crabomination::effect::Selector::You,
        amount: crabomination::effect::Value::Const(7),
    };
    let src = g.add_card_to_battlefield(0, catalog::island());
    g.stack.push(StackItem::Trigger {
        source: src,
        controller: 0,
        effect: Box::new(body),
        target: None,
        mode: None,
        x_value: 0,
        converged_value: 0,
        trigger_source: None,
        mana_spent: 0,
        event_amount: 0,
        intervening_if: Some(pred),
        additional_targets: Vec::new(),
        activated: false,
    });
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 7,
        "Trigger body ran — predicate held at resolve time");
}

// ── CR 705.3 — Krark's Thumb-style coin-flip advantage ─────────────────────

#[test]
fn cr_705_3_coin_flip_advantage_lets_tails_be_recovered() {
    // Direct exercise of the new `Player.coin_flip_advantage` field:
    // with advantage = 1, a flip-coin effect that would default to tails
    // (via a ScriptedDecider that always returns Bool(false)) should
    // still see heads on at least one of the two replayed flips.
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::Effect;

    let mut g = two_player_game();
    g.players[0].coin_flip_advantage = 1;
    // ScriptedDecider returning false twice then true means:
    //   - Without advantage: 1 flip returns false → tails branch.
    //   - With advantage=1:  2 flips. The first returns false, the
    //     second returns false. heads_seen stays false → tails branch.
    //   (To force heads_seen=true we'd need ≥1 Bool(true) in the script.)
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(true),  // the SECOND replay wins
    ]));

    // Build a flip effect that adds +5 life on heads, -5 life on tails.
    let body = Effect::FlipCoin {
        count: crabomination::effect::Value::Const(1),
        on_heads: Box::new(Effect::GainLife {
            who: crabomination::effect::Selector::You,
            amount: crabomination::effect::Value::Const(5),
        }),
        on_tails: Box::new(Effect::LoseLife {
            who: crabomination::effect::Selector::You,
            amount: crabomination::effect::Value::Const(5),
        }),
    };
    // Drop it on the stack as a Trigger to exercise the resolver path.
    let src = g.add_card_to_battlefield(0, catalog::island());
    g.stack.push(crabomination::game::types::StackItem::Trigger {
        source: src,
        controller: 0,
        effect: Box::new(body),
        target: None,
        mode: None,
        x_value: 0,
        converged_value: 0,
        trigger_source: None,
        mana_spent: 0,
        event_amount: 0,
        intervening_if: None,
        additional_targets: Vec::new(),
        activated: false,
    });
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    // With advantage=1, even though the first flip returned false, the
    // second returned true → heads_seen = true → +5 life.
    assert_eq!(g.players[0].life, life_before + 5,
        "Coin-flip advantage lets us redeem a tails result");
}

#[test]
fn cr_705_3_no_advantage_means_one_flip_one_result() {
    // Without advantage, a single Bool(false) → tails branch.
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::Effect;

    let mut g = two_player_game();
    assert_eq!(g.players[0].coin_flip_advantage, 0, "default advantage is 0");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));

    let body = Effect::FlipCoin {
        count: crabomination::effect::Value::Const(1),
        on_heads: Box::new(Effect::GainLife {
            who: crabomination::effect::Selector::You,
            amount: crabomination::effect::Value::Const(5),
        }),
        on_tails: Box::new(Effect::LoseLife {
            who: crabomination::effect::Selector::You,
            amount: crabomination::effect::Value::Const(5),
        }),
    };
    let src = g.add_card_to_battlefield(0, catalog::island());
    g.stack.push(crabomination::game::types::StackItem::Trigger {
        source: src,
        controller: 0,
        effect: Box::new(body),
        target: None,
        mode: None,
        x_value: 0,
        converged_value: 0,
        trigger_source: None,
        mana_spent: 0,
        event_amount: 0,
        intervening_if: None,
        additional_targets: Vec::new(),
        activated: false,
    });
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 5,
        "Without advantage, tails fires the lose-life branch");
}

// ── CR 122.4 — max counters of a kind SBA ──────────────────────────────────

#[test]
fn cr_122_4_excess_counters_pruned_by_sba() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::mana::cost;

    let mut g = two_player_game();
    let def = CardDefinition {
        name: "Pinnacle Test (synthetic)",
        cost: cost(&[]),
        card_types: vec![CardType::Artifact],
        max_counters_of_kind: Some((CounterType::PlusOnePlusOne, 3)),
        ..Default::default()
    };
    let id = g.add_card_to_battlefield(0, def);
    {
        let c = g.battlefield_find_mut(id).expect("on bf");
        c.add_counters(CounterType::PlusOnePlusOne, 7);
    }
    let _ = g.check_state_based_actions();
    let after = g.battlefield_find(id).expect("on bf");
    assert_eq!(after.counter_count(CounterType::PlusOnePlusOne), 3,
        "Excess counters pruned down to the cap (3)");
}

#[test]
fn quandrix_expansor_b155_creates_fractal_with_x_counters() {
    let mut g = two_player_game();
    let exp = g.add_card_to_hand(0, catalog::quandrix_expansor_b155());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: exp, target: None, additional_targets: vec![],
        mode: None, x_value: Some(3),
    }).expect("Expansor castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.definition.name == "Fractal").expect("Fractal minted");
    let counters: u32 = fractal.counters.iter()
        .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
        .map(|(_, n)| *n).sum();
    assert_eq!(counters, 3, "Fractal has X=3 +1/+1 counters");
}

// ── batch 155 — Prismari cards ─────────────────────────────────────────────

#[test]
fn prismari_forgewright_b155_etb_pings_each_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_forgewright_b155());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Forgewright castable");
    drain_stack(&mut g);
    let bear_damage = g.battlefield_find(bear).map(|c| c.damage).unwrap_or(0);
    assert_eq!(bear_damage, 1, "ETB pings each opp creature for 1");
}

#[test]
fn prismari_spellsign_b155_deals_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let ss = g.add_card_to_hand(0, catalog::prismari_spellsign_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: ss, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellsign castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2, "Deal 2");
    assert_eq!(g.players[0].hand.len(), hand_before, "-1 cast + 1 draw = 0");
}

// ── batch 158 — Silverquill / Witherbloom / Lorehold / Quandrix singles ────

#[test]
fn silverquill_censurer_b158_etb_taps_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_censurer_b158());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Censurer castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).expect("bear still on bf");
    assert!(b.tapped, "opp bear should be tapped");
}

#[test]
fn silverquill_edicter_b158_forces_opp_sac_and_gains_one_life() {
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_edicter_b158());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    let opp_creatures_before = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.card_types.iter().any(|t| matches!(t, crabomination::card::CardType::Creature)))
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Edicter castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    let opp_creatures_after = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.card_types.iter().any(|t| matches!(t, crabomination::card::CardType::Creature)))
        .count();
    assert_eq!(opp_creatures_after, opp_creatures_before - 1, "opp sacrificed one");
}

#[test]
fn witherbloom_faminescion_b158_drains_three_and_mills() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::witherbloom_faminescion_b158());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let lib1_before = g.players[1].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Faminescion castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 3);
    assert_eq!(g.players[1].life, life1_before - 3);
    assert_eq!(g.players[1].library.len(), lib1_before - 2);
}

#[test]
fn witherbloom_toxinspear_b158_kills_two_toughness_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxinspear_b158());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxinspear castable");
    drain_stack(&mut g);
    // -2/-1 on a 2/2: 0/1
    let b = g.battlefield_find(bear);
    match b {
        Some(c) => {
            assert!(c.power() <= 0 || c.toughness() <= 1);
        }
        None => {
            // dead is fine — 0 power 1 toughness gets put on the field
            assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
        }
    }
}

#[test]
fn lorehold_spellsong_b158_burns_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spellsong_b158());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellsong castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2);
    assert_eq!(g.players[0].life, life0_before + 2);
}

#[test]
fn quandrix_bigbrain_b158_mints_fractal_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_bigbrain_b158());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bigbrain castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal")
        .expect("Fractal minted");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn quandrix_counterpoint_b158_counters_unless_paid() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    // p0 responds with Counterpoint
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::quandrix_counterpoint_b158());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Counterpoint castable");
    drain_stack(&mut g);
    // P1 had no extra {1} → bolt countered, P0 still at 20.
    assert_eq!(g.players[0].life, 20);
}

#[test]
fn bot_declines_bad_block_into_first_strike() {
    // Push (claude/modern_decks): first-strike-aware blocking. A 2/2
    // first-strike attacker kills a 2/2 vanilla blocker before it can
    // strike back, so the "trade" is illusory. With full life the bot
    // must NOT make that block.
    use crabomination::server::bot;
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareBlockers;
    let attacker = {
        let mut d = catalog::grizzly_bears();
        d.name = "First Striker";
        d.keywords = vec![Keyword::FirstStrike];
        g.add_card_to_battlefield(1, d)
    };
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 vanilla
    g.clear_sickness(blocker);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert!(blocks.is_empty(),
        "bot should not chump-trade a 2/2 into a 2/2 first-striker at full life");
}

#[test]
fn bot_blocks_first_striker_it_outsizes() {
    // The same attacker, but our 3/3 blocker survives the first-strike
    // damage and kills it on the regular step — a real clean trade.
    use crabomination::server::bot;
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareBlockers;
    let attacker = {
        let mut d = catalog::grizzly_bears();
        d.name = "First Striker";
        d.keywords = vec![Keyword::FirstStrike];
        g.add_card_to_battlefield(1, d)
    };
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let blocker = {
        let mut d = catalog::grizzly_bears();
        d.name = "Wall Bear";
        d.power = 3;
        d.toughness = 3;
        g.add_card_to_battlefield(0, d)
    };
    g.clear_sickness(blocker);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert_eq!(blocks, vec![(blocker, attacker)],
        "a 3/3 survives the first-strike 2 and kills the 2/2 first-striker");
}

#[test]
fn cr_705_3_static_grants_coin_flip_advantage() {
    // Krark's-Thumb-style: advantage comes from a battlefield static, not the
    // Player field. A scripted (tails, heads) flip should resolve as heads.
    use crabomination::card::{CardDefinition, CardType, StaticAbility, Supertype};
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::effect::{Effect, PlayerStaticTarget, Selector, StaticEffect, Value};

    let thumb = CardDefinition {
        name: "Test Thumb",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Coin-flip advantage.",
            effect: StaticEffect::CoinFlipAdvantage { target: PlayerStaticTarget::Controller },
        }],
        ..Default::default()
    };
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, thumb);
    assert_eq!(g.coin_flip_advantage_now(0), 1, "static grants advantage to its controller");
    assert_eq!(g.coin_flip_advantage_now(1), 0, "opponent gets none");

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(true), // the second replay wins
    ]));
    let body = Effect::FlipCoin {
        count: Value::Const(1),
        on_heads: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(5) }),
        on_tails: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(5) }),
    };
    let src = g.add_card_to_battlefield(0, catalog::island());
    g.stack.push(crabomination::game::types::StackItem::Trigger {
        source: src, controller: 0, effect: Box::new(body), target: None, mode: None,
        x_value: 0, converged_value: 0, trigger_source: None, mana_spent: 0,
        event_amount: 0, intervening_if: None,
        additional_targets: Vec::new(),
        activated: false,
    });
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 5,
        "static-granted advantage redeems the tails flip");
}
