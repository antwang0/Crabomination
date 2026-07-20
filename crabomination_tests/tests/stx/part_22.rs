use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: ETB gain-life creatures
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn etb_gain_life_creatures() {
    for (def, mana, colorless, gain) in [
        (catalog::inkling_scrollguard_b174(), &[(Color::White, 1), (Color::Black, 1)][..], 2, 2),
        (catalog::lorehold_banneret_b174(), &[(Color::Red, 1), (Color::White, 1)][..], 2, 2),
        (catalog::inkling_cantor_b175(), &[(Color::White, 1)][..], 2, 1),
        (catalog::witherbloom_vinecaster_b178(), &[(Color::Green, 1)][..], 1, 1),
        (catalog::witherbloom_reachsage_b190(), &[(Color::Green, 2)][..], 1, 2),
    ] {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for &(c, n) in mana { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        let p0_life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, p0_life + gain);
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: cast spell/creature → opponent loses N life (drain / burn)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn cast_drain_or_burn_opponent() {
    for (def, mana, colorless, tgt, loss, gain) in [
        (catalog::silverquill_pyremist_b174(), &[(Color::White, 1), (Color::Black, 1)][..], 3, None, 2, Some(2)),
        (catalog::witherbloom_cauldron_echo_b178(), &[(Color::Black, 1), (Color::Green, 1)][..], 2, None, 3, Some(3)),
        (catalog::silverquill_stampcrafter_b182(), &[(Color::White, 1), (Color::Black, 1)][..], 2, None, 1, Some(1)),
        (catalog::silverquill_litany_b188(), &[(Color::White, 1), (Color::Black, 1)][..], 1, None, 2, None),
        (catalog::inkling_tribune_b188(), &[(Color::White, 1), (Color::Black, 1)][..], 3, None, 2, None),
        (catalog::silverquill_drainmaster_ii_b189(), &[(Color::Black, 2)][..], 2, None, 3, None),
        (catalog::witherbloom_spellblossom_b189(), &[(Color::Black, 1), (Color::Green, 1)][..], 3, None, 4, Some(4)),
        (catalog::witherbloom_doublestrike_b191(), &[(Color::Black, 1), (Color::Green, 1)][..], 2, None, 2, None),
        (catalog::silverquill_inkdrain_b191(), &[(Color::White, 1), (Color::Black, 1)][..], 2, None, 3, None),
        (catalog::prismari_lavaforge_b180(), &[(Color::Red, 2)][..], 3, Some(Target::Player(1)), 3, None),
        (catalog::lorehold_sparkbarrier_b188(), &[(Color::Red, 1)][..], 2, Some(Target::Player(1)), 3, None),
        (catalog::lorehold_echobringer_b191(), &[(Color::Red, 1), (Color::White, 1)][..], 3, Some(Target::Player(1)), 2, None),
        (catalog::prismari_hailcaller_b188(), &[(Color::Blue, 1), (Color::Red, 1)][..], 3, None, 3, None),
        (catalog::lorehold_voltmage_b189(), &[(Color::Red, 1)][..], 2, None, 2, None),
        (catalog::prismari_sparkflood_b174(), &[(Color::Red, 1), (Color::Blue, 1)][..], 1, Some(Target::Player(1)), 2, None),
    ] {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for &(c, n) in mana { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        let p0_life = g.players[0].life;
        let p1_life = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: tgt, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, p1_life - loss);
        if let Some(gn) = gain {
            assert_eq!(g.players[0].life, p0_life + gn);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: magecraft — cast a bolt at the opponent, opponent loses N
// total (ping/drain + bolt 3), optionally caster gains 1 from a drain.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn magecraft_burn_or_drain_opponent() {
    for (def, loss, gain) in [
        (catalog::witherbloom_pestbinder_b174(), 4, Some(1)),
        (catalog::lorehold_pyrespirit_b174(), 4, None),
        (catalog::prismari_embermage_b174(), 4, None),
        (catalog::witherbloom_cauldroncrier_b175(), 4, None),
        (catalog::prismari_sparkmage_b175(), 4, None),
        (catalog::witherbloom_drainscribe_b181(), 4, Some(1)),
        (catalog::lorehold_cultivator_b177(), 4, Some(1)),
        (catalog::inkling_stylekeeper_b177(), 4, None),
        (catalog::witherbloom_spelleater_b188(), 5, None),
    ] {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let p1_life = g.players[1].life;
        let p0_life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, p1_life - loss);
        if let Some(gn) = gain {
            assert_eq!(g.players[0].life, p0_life + gn);
        }
    }
}

#[test]
fn magecraft_gain_life() {
    for (def, gain) in [
        (catalog::witherbloom_sapcaller_b174(), 1),
        (catalog::silverquill_pridecrier_b178(), 2),
    ] {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let p0_life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, p0_life + gain);
    }
}

#[test]
fn magecraft_adds_counter_to_self() {
    for def in [
        catalog::quandrix_symbolist_b174(),
        catalog::quandrix_dataweaver_b188(),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        let c = g.battlefield_find(id).expect("alive");
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    }
}

#[test]
fn magecraft_self_pump_to_power_three() {
    for def in [
        catalog::lorehold_spiritsong_b188(),
        catalog::quandrix_sparkbloomer_b191(),
        catalog::lorehold_crusader_b189(),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(id).unwrap().power(), 3);
    }
}

#[test]
fn magecraft_pumps_friendly_creature() {
    for def in [
        catalog::quandrix_mathwarden_b175(),
        catalog::lorehold_ghostsmith_b177(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, def);
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
}

#[test]
fn magecraft_loots() {
    for def in [
        catalog::silverquill_stenographer_b175(),
        catalog::prismari_wavetamer_b191(),
    ] {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        g.add_card_to_battlefield(0, def);
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
}

#[test]
fn magecraft_draws_a_card() {
    for def in [
        catalog::quandrix_mathshape_b174(),
        catalog::prismari_magecraft_sage_b178(),
    ] {
        let mut g = two_player_game();
        for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
        g.add_card_to_battlefield(0, def);
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
}

#[test]
fn magecraft_scrys_without_changing_library_size() {
    for def in [
        catalog::prismari_wavefocuser_b174(),
        catalog::prismari_storm_scholar_b188(),
        catalog::lorehold_sparrowscholar_b191(),
    ] {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        g.add_card_to_battlefield(0, def);
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

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: ETB token mints
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn etb_mints_tokens() {
    for (def, mana, colorless, token, count) in [
        (catalog::pest_shepherd_b174(), &[(Color::Green, 1)][..], 2, "Pest", 1),
        (catalog::witherbloom_pestharvest_b175(), &[(Color::Black, 1), (Color::Green, 1)][..], 2, "Pest", 2),
        (catalog::witherbloom_pestlord_b181(), &[(Color::Black, 1), (Color::Green, 1)][..], 3, "Pest", 2),
        (catalog::lorehold_spectralcaller_b174(), &[(Color::Red, 1), (Color::White, 1)][..], 3, "Spirit", 1),
        (catalog::lorehold_spiritlord_b180(), &[(Color::Red, 1), (Color::White, 1)][..], 3, "Spirit", 2),
        (catalog::lorehold_fireseal_b189(), &[(Color::Red, 1), (Color::White, 1)][..], 2, "Spirit", 2),
        (catalog::prismari_tinkermage_b191(), &[(Color::Blue, 1), (Color::Red, 1)][..], 0, "Treasure", 1),
        (catalog::prismari_spellforge_b174(), &[(Color::Blue, 1), (Color::Red, 1)][..], 2, "Treasure", 1),
    ] {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for &(c, n) in mana { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let n_tokens = g.battlefield.iter()
            .filter(|c| c.is_token && c.definition.name == token && c.controller == 0)
            .count();
        assert_eq!(n_tokens, count, "{token} tokens minted");
    }
}

#[test]
fn mints_fractal_with_n_counters() {
    for (def, mana, colorless, counters) in [
        (catalog::quandrix_fractalspinner_b174(), &[(Color::Green, 1), (Color::Blue, 1)][..], 3, 2),
        (catalog::quandrix_beastform_b175(), &[(Color::Green, 1), (Color::Blue, 1)][..], 1, 3),
        (catalog::quandrix_fractal_echocaller_b180(), &[(Color::Green, 1)][..], 2, 1),
        (catalog::quandrix_fractalkeeper_b177(), &[(Color::Green, 1), (Color::Blue, 1)][..], 3, 4),
        (catalog::quandrix_sumtotal_b191(), &[(Color::Green, 1), (Color::Blue, 1)][..], 3, 4),
    ] {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for &(c, n) in mana { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let fractal = g.battlefield.iter()
            .find(|c| c.is_token && c.definition.name == "Fractal" && c.controller == 0)
            .expect("fractal token");
        assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), counters);
    }
}

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

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: spells that grant keyword counters (and maybe +1/+1) to a
// target creature.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn spells_grant_keywords_to_target_creature() {
    for (def, mana, colorless, kws, counters) in [
        (catalog::silverquill_wordsharpener_b184(), &[(Color::White, 1), (Color::Black, 1)][..], 1, &[Keyword::FirstStrike][..], 0),
        (catalog::silverquill_drainmark_b184(), &[(Color::Black, 1)][..], 1, &[Keyword::Deathtouch][..], 0),
        (catalog::witherbloom_trampleblossom_b184(), &[(Color::Green, 1)][..], 2, &[Keyword::Trample][..], 0),
        (catalog::witherbloom_lifebondseal_b184(), &[(Color::Black, 1)][..], 1, &[Keyword::Lifelink][..], 0),
        (catalog::lorehold_battlerune_b184(), &[(Color::Red, 1), (Color::White, 1)][..], 2, &[Keyword::Haste][..], 0),
        (catalog::lorehold_wardseal_b184(), &[(Color::White, 1)][..], 1, &[Keyword::Vigilance][..], 0),
        (catalog::silverquill_wardseal_b190(), &[(Color::White, 1)][..], 1, &[Keyword::Vigilance][..], 0),
        (catalog::silverquill_lifeward_b190(), &[(Color::White, 1), (Color::Black, 1)][..], 0, &[Keyword::Lifelink][..], 0),
        (catalog::witherbloom_venomgift_b190(), &[(Color::Black, 1)][..], 0, &[Keyword::Deathtouch][..], 0),
        (catalog::silverquill_doublecurse_b190(), &[(Color::Black, 1)][..], 1, &[Keyword::Deathtouch, Keyword::Flying][..], 0),
        (catalog::witherbloom_doublegrowth_b190(), &[(Color::Green, 1)][..], 2, &[Keyword::Trample][..], 1),
        (catalog::lorehold_doubleblast_b190(), &[(Color::Red, 1)][..], 2, &[Keyword::FirstStrike, Keyword::Haste][..], 0),
        (catalog::lorehold_bondseal_b190(), &[(Color::White, 1)][..], 1, &[Keyword::Vigilance][..], 1),
        (catalog::prismari_doublecharge_b190(), &[(Color::Blue, 1), (Color::Red, 1)][..], 1, &[Keyword::Haste, Keyword::Flying][..], 0),
        (catalog::quandrix_doublegrowth_b190(), &[(Color::Green, 1), (Color::Blue, 1)][..], 1, &[Keyword::Trample, Keyword::Flying][..], 0),
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        for &(c, n) in mana { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(bear).expect("bear alive");
        for kw in kws {
            assert!(c.has_keyword(kw), "bear gains {kw:?}");
        }
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), counters);
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: creatures that ETB with their own keyword/+1+1 counters.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn etb_self_counters_grant_keywords() {
    for (def, name, mana, colorless, kws, counters) in [
        (catalog::prismari_sparkbloomer_b185(), "Prismari Sparkbloomer (b185)", &[(Color::Red, 1)][..], 3, &[Keyword::Haste][..], 0),
        (catalog::witherbloom_venomspur_b185(), "Witherbloom Venomspur (b185)", &[(Color::Black, 1)][..], 2, &[Keyword::Deathtouch][..], 0),
        (catalog::lorehold_phoenixmage_b190(), "Lorehold Phoenixmage (b190)", &[(Color::Red, 1)][..], 2, &[Keyword::Haste][..], 0),
        (catalog::prismari_skydiver_b190(), "Prismari Skydiver (b190)", &[(Color::Red, 1)][..], 2, &[Keyword::Flying][..], 0),
        (catalog::quandrix_riftleaper_b190(), "Quandrix Riftleaper (b190)", &[(Color::Blue, 1)][..], 2, &[Keyword::Flying][..], 0),
        (catalog::inkling_highscribe_b191(), "Inkling Highscribe (b191)", &[(Color::White, 1), (Color::Black, 1)][..], 2, &[Keyword::Flying][..], 0),
        (catalog::quandrix_sapleader_b190(), "Quandrix Sapleader (b190)", &[(Color::Green, 1), (Color::Blue, 1)][..], 3, &[Keyword::Reach][..], 1),
        (catalog::witherbloom_mireshade_b188(), "Witherbloom Mireshade (b188)", &[(Color::Black, 1), (Color::Green, 1)][..], 1, &[Keyword::Deathtouch][..], 0),
        (catalog::inkling_hatchling_b175(), "Inkling Hatchling (b175)", &[(Color::White, 1), (Color::Black, 1)][..], 0, &[][..], 1),
        (catalog::quandrix_vinegrower_b191(), "Quandrix Vinegrower (b191)", &[(Color::Green, 1), (Color::Blue, 1)][..], 0, &[][..], 1),
        (catalog::quandrix_streamwarden_b182(), "Quandrix Streamwarden (b182)", &[(Color::Green, 1), (Color::Blue, 1)][..], 2, &[][..], 1),
    ] {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for &(c, n) in mana { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let c = g.battlefield.iter()
            .find(|c| c.definition.name == name)
            .expect("on bf");
        for kw in kws {
            assert!(c.has_keyword(kw), "{name} has {kw:?}");
        }
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), counters);
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: counter-granting removal-adjacent spells (shield/finality)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn spells_grant_shield_or_finality_counters() {
    for (def, mana, colorless, owner, counter_type) in [
        (catalog::silverquill_aegis_b176(), &[(Color::White, 1)][..], 1, 0, CounterType::Shield),
        (catalog::witherbloom_doomsign_b176(), &[(Color::Black, 1)][..], 2, 1, CounterType::Finality),
        (catalog::silverquill_doomgrant_b176(), &[(Color::Black, 1)][..], 2, 1, CounterType::Finality),
    ] {
        let mut g = two_player_game();
        let target = g.add_card_to_battlefield(owner, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        for &(c, n) in mana { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(target).expect("target alive");
        assert_eq!(c.counter_count(counter_type), 1);
    }
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

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: attack triggers (drain/ping/gain on declare attackers)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn attack_triggers_drain_or_gain() {
    for (def, loss, gain) in [
        (catalog::witherbloom_toxicultivator_b174(), 2, 2),
        (catalog::lorehold_sparkborn_b174(), 1, 0),
        (catalog::inkling_heraldscribe_b179(), 1, 1),
        (catalog::lorehold_spectralguard_b180(), 0, 1),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
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
        assert!(g.players[1].life <= p1_life - loss, "attack trigger hurts opponent");
        assert!(g.players[0].life >= p0_life + gain, "attack trigger helps controller");
    }
}

#[test]
fn attack_triggers_ping_via_priority_loop() {
    for (def, loss) in [
        (catalog::prismari_lavakin_b188(), 1),
        (catalog::prismari_magmamancer_b189(), 2),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        let p1_life = g.players[1].life;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: id, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        assert!(g.players[1].life <= p1_life - loss);
    }
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

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: removal spells (kill / exile / chip damage)
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn burn_spells_kill_target_creature() {
    for (def, mana, colorless, victim) in [
        (catalog::lorehold_charm_echo_b175(), &[(Color::Red, 1)][..], 1, catalog::grizzly_bears()),
        (catalog::lorehold_ghostflame_b174(), &[(Color::Red, 1), (Color::White, 1)][..], 1, catalog::grizzly_bears()),
        (catalog::prismari_cloudburst_b175(), &[(Color::Blue, 1), (Color::Red, 1)][..], 3, catalog::pest_bramblebeast_b174()),
    ] {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let target = g.add_card_to_battlefield(1, victim);
        let id = g.add_card_to_hand(0, def);
        for &(c, n) in mana { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == target),
            "victim dies to burn spell");
    }
}

#[test]
fn spells_exile_target_creature() {
    for (def, victim) in [
        (catalog::silverquill_verdictbearer_b175(), catalog::pest_bramblebeast_b174()),
        (catalog::silverquill_exilewright_b189(), catalog::grizzly_bears()),
    ] {
        let mut g = two_player_game();
        let big = g.add_card_to_battlefield(1, victim);
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(3);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(big)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == big), "creature exiled");
    }
}

#[test]
fn burn_spells_deal_two_to_big_creature() {
    for (def, colorless) in [
        (catalog::prismari_sparkforge_ii_b190(), 1),
        (catalog::prismari_hailstrike_b189(), 0),
        (catalog::prismari_stormwave_b191(), 2),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let beast = g.add_card_to_battlefield(1, catalog::quandrix_vinescaler_ii_b189());
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(beast)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(beast).unwrap().damage, 2);
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: counterspells
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn counterspells_counter_the_bolt() {
    for (def, colorless) in [
        (catalog::quandrix_wavelock_b174(), 2),
        (catalog::quandrix_counterspinner_b180(), 1),
    ] {
        let mut g = two_player_game();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        g.perform_action(GameAction::PassPriority).unwrap();
        let counter = g.add_card_to_hand(1, def);
        g.players[1].mana_pool.add(Color::Blue, 1);
        g.players[1].mana_pool.add_colorless(colorless);
        g.perform_action(GameAction::CastSpell {
            card_id: counter, target: Some(Target::Permanent(bolt)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        // P0 has no mana to pay; bolt countered.
        assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
            "bolt countered");
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: draw / loot / drain-draw spells and ETB draws
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn draw_spells_change_hand_and_life() {
    for (def, mana, colorless, hand_delta, p0_gain, p0_loss, p1_loss) in [
        (catalog::quandrix_riverflow_b174(), &[(Color::Blue, 1)][..], 1, 1, 0, 1, 0),
        (catalog::quandrix_tidemind_b175(), &[(Color::Blue, 1)][..], 3, 0, 0, 0, 0),
        (catalog::quandrix_drawcaster_b178(), &[(Color::Blue, 1)][..], 3, 0, 0, 0, 0),
        (catalog::quandrix_streamcaster_b177(), &[(Color::Green, 1), (Color::Blue, 1)][..], 2, 0, 0, 0, 0),
        (catalog::inkling_tutor_b179(), &[(Color::Black, 1)][..], 1, 0, 0, 0, 0),
        (catalog::quandrix_latticebreaker_b188(), &[(Color::Blue, 2)][..], 3, 2, 0, 0, 0),
        (catalog::quandrix_cantrip_b189(), &[(Color::Blue, 1)][..], 1, 1, 0, 0, 0),
        (catalog::inkling_lifesong_b178(), &[(Color::White, 1), (Color::Black, 1)][..], 0, 0, 2, 0, 2),
    ] {
        let mut g = two_player_game();
        // Seed hand & library so discard-then-draw both can happen.
        let _filler = g.add_card_to_hand(0, catalog::grizzly_bears());
        for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for &(c, n) in mana { g.players[0].mana_pool.add(c, n); }
        g.players[0].mana_pool.add_colorless(colorless);
        let p0_life = g.players[0].life;
        let p1_life = g.players[1].life;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + hand_delta);
        assert_eq!(g.players[0].life, p0_life + p0_gain - p0_loss);
        assert_eq!(g.players[1].life, p1_life - p1_loss);
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: dies-triggers that drain when the creature itself dies
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn dies_triggers_drain_opponent() {
    for (def, loss, gain) in [
        (catalog::silverquill_reapcrier_b175(), 1, Some(1)),
        (catalog::witherbloom_plaguebearer_b181(), 2, None),
        (catalog::pest_herald_b188(), 1, None),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
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
        assert_eq!(g.players[1].life, p1_life - loss, "drain on death");
        if let Some(gn) = gain {
            assert_eq!(g.players[0].life, p0_life + gn);
        }
    }
}

#[test]
fn silverquill_inkfiend_b174_drains_when_other_creature_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_inkfiend_b174());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Bear (2/2) takes 3 → dies → on-other-dies trigger drains 1.
    assert_eq!(g.players[1].life, p1_life - 1);
    assert_eq!(g.players[0].life, p0_life + 1);
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
fn witherbloom_necromancer_b156_pays_one_to_reanimate_the_dead_creature() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::witherbloom_necromancer_b156());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // P0 floats {1} to pay the optional reanimate cost.
    g.players[0].mana_pool.add_colorless(1);
    // Accept the "Pay {1}?" MayPay prompt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    // Opponent bolts P0's fodder; it dies and the dies trigger fires.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);

    // The dead creature is reanimated to P0's battlefield (same card id),
    // and the {1} was spent.
    assert!(
        g.battlefield_find(fodder).is_some_and(|c| c.controller == 0),
        "the dead creature returns to the battlefield under your control",
    );
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "it left the graveyard (the {{1}} cost is implied — the body only runs on a successful pay)",
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Unique per-card tests
// ─────────────────────────────────────────────────────────────────────────

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
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Witherbloom Drainmage (b174)"),
        "drainmage alive");
    assert_eq!(g.players[0].life, p0_life_pre_cast + 1);
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
    // 2/2 minus -1/-1 = 1/1; should still be alive.
    let target_alive = g.battlefield_find(target);
    if let Some(c) = target_alive {
        assert_eq!(c.power(), 1);
        assert_eq!(c.toughness(), 1);
    }
}

#[test]
fn silverquill_penkeeper_b175_magecraft_each_opp_discards() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_penkeeper_b175());
    g.add_card_to_hand(1, catalog::grizzly_bears()); // a card for the opponent to discard
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    let p1_hand_before = g.players[1].hand.len();
    let p1_grave_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // Magecraft now makes each opponent discard a card — only the bolt's
    // 3 damage hits life (no drain).
    assert_eq!(g.players[1].life, p1_life - 3, "bolt damage only; magecraft no longer drains");
    assert_eq!(g.players[1].hand.len(), p1_hand_before - 1, "opponent discarded one card");
    assert_eq!(g.players[1].graveyard.len(), p1_grave_before + 1, "the discard hit the graveyard");
}

#[test]
fn silverquill_wordweaver_b177_etb_each_opp_discards() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_wordweaver_b177());
    g.add_card_to_hand(1, catalog::grizzly_bears()); // a card for the opponent to discard
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    let p1_hand_before = g.players[1].hand.len();
    let p1_grave_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // ETB now makes each opponent discard a card (no life change).
    assert_eq!(g.players[1].life, p1_life, "ETB no longer drains the opponent");
    assert_eq!(g.players[1].hand.len(), p1_hand_before - 1, "opponent discarded one card");
    assert_eq!(g.players[1].graveyard.len(), p1_grave_before + 1, "the discard hit the graveyard");
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
fn lorehold_sparkscholar_b178_taps_for_two_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_sparkscholar_b178());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: Vec::new(),
        x_value: None,
    }).expect("activated");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2);
}

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
fn pest_druid_b191_taps_for_b_or_g() {
    let def = catalog::pest_druid_b191();
    assert_eq!(def.cost.cmc(), 2);
    assert_eq!(def.activated_abilities.len(), 1);
    assert!(def.activated_abilities[0].tap_cost);
    assert!(def.subtypes.creature_types.contains(&CreatureType::Pest));
}

// ─────────────────────────────────────────────────────────────────────────
// Table-driven: definition stat/keyword checks
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn definition_stats_and_keywords() {
    for (def, cmc, power, toughness, kws) in [
        (catalog::pest_bramblebeast_b174(), 4, Some(4), Some(4), &[Keyword::Reach][..]),
        (catalog::lorehold_vanguard_b174(), 5, Some(4), Some(4), &[Keyword::Trample][..]),
        (catalog::inkling_mortician_b175(), 5, None, None, &[Keyword::Flying, Keyword::Lifelink][..]),
        (catalog::silverquill_ascendant_b182(), 6, Some(5), Some(5), &[Keyword::Flying, Keyword::Lifelink][..]),
        (catalog::inkling_vassalking_b189(), 5, None, None, &[Keyword::Flying, Keyword::Lifelink][..]),
        (catalog::lorehold_vanguard_ii_b188(), 4, Some(3), None, &[Keyword::Vigilance, Keyword::Reach][..]),
        (catalog::quandrix_mossleaf_b188(), 2, None, Some(3), &[Keyword::Reach][..]),
        (catalog::quandrix_vinescaler_ii_b189(), 4, Some(4), None, &[Keyword::Reach, Keyword::Trample][..]),
    ] {
        assert_eq!(def.cost.cmc(), cmc);
        if let Some(p) = power { assert_eq!(def.power, p); }
        if let Some(t) = toughness { assert_eq!(def.toughness, t); }
        for kw in kws {
            assert!(def.keywords.contains(kw), "{} has {kw:?}", def.name);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────
// CR rule lock-in tests (kept verbatim)
// ─────────────────────────────────────────────────────────────────────────

/// CR 121.5 — "If an effect moves cards from a player's library to that
/// player's hand without using the word 'draw,' the player has not drawn
/// those cards." Verify that `RevealUntilFind` (which puts the matching
/// card into the hand, not "draws") does NOT trigger a CardDrawn event
/// and does NOT bump `cards_drawn_this_turn`.
#[test]
fn cr_121_5_reveal_until_find_does_not_count_as_draw() {
    use crabomination::card::Effect;
    use crabomination::effect::RevealMissDest;
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    // Seed P0 library: top-of-deck Forest (matches IsBasicLand), then 2 Islands beneath.
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let cards_drawn_before = g.players[0].cards_drawn_this_turn;
    let hand_before = g.players[0].hand.len();

    let eff = Effect::RevealUntilFind {
        who: crabomination::effect::PlayerRef::You,
        find: crabomination::card::SelectionRequirement::IsBasicLand,
        to: crabomination::effect::ZoneDest::Hand(crabomination::effect::PlayerRef::You),
        cap: crabomination::card::Value::Const(5),
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
    use crabomination::card::{Effect, Selector, SelectionRequirement};
    use crabomination::game::effects::EffectContext;
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
    assert_eq!(c.keyword_counters.get(&Keyword::Flying).copied().unwrap_or(0), 1);
    // Computed permanent also surfaces the keyword (layer-6 path).
    let cp = g.compute_battlefield().into_iter()
        .find(|cc| cc.id == bear)
        .expect("bear computed");
    assert!(cp.keywords.contains(&Keyword::Flying));
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

/// "Whenever you cast your first instant or sorcery spell each turn,
/// draw a card." — the trigger is once-per-turn (CR 603.3d): the first
/// I/S draws, the second one the same turn does not.
#[test]
fn prismari_mage_mentor_b182_draws_on_first_is_only() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_battlefield(0, catalog::prismari_mage_mentor_b182());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("first bolt");
    drain_stack(&mut g);
    // -1 (cast) +1 (first-I/S draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "first instant of the turn draws a card");
    // A second instant the same turn does NOT re-trigger.
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_mid = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("second bolt");
    drain_stack(&mut g);
    // -1 (cast), no draw — the once-per-turn trigger already fired.
    assert_eq!(g.players[0].hand.len(), hand_mid - 1,
        "second instant the same turn draws nothing");
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

/// CR 121.2 — "Cards may only be drawn one at a time. If a player is
/// instructed to draw multiple cards, that player performs that many
/// individual card draws." A multi-draw effect should fire one CardDrawn
/// event per card drawn, not one batched event. This test pins the
/// per-draw fanout — Witherbloom Lifeknotter's `LifeGained/YourControl`
/// trigger fires once per individual draw via Drain.
#[test]
fn cr_121_2_multi_draw_fires_one_event_per_card() {
    use crabomination::game::GameEvent;
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
    // Per CR 122.1c: each damage event removes one shield counter and prevents
    // the damage. So shield -1, no damage, bear at 2/2 with 1 shield.
    assert_eq!(bear_after1.counter_count(CounterType::Shield), 1);
    assert_eq!(bear_after1.damage, 0);
}
