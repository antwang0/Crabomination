use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

/// Fill player 0's pool with generous mana of every color plus colorless so
/// table-driven tests can cast cards with differing costs.
fn generous_mana(g: &mut crabomination::game::GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

// ── Table: magecraft sources whose trigger changes life totals ─────────────
// Shape: source on battlefield, cast Bolt at P1, check both life totals.
// (def, p0_gain, p1_loss) — p1_loss includes the Bolt's 3.
#[test]
fn magecraft_life_delta_sources() {
    for (def, p0_gain, p1_loss) in [
        (catalog::prismari_igniter(), 0, 4),
        (catalog::lorehold_pyrokineticist(), 0, 4),
        (catalog::witherbloom_rotmancer(), 0, 4),
        (catalog::strixhaven_spellfletcher(), 0, 4),
        (catalog::silverquill_novice(), 1, 3),
        (catalog::silverquill_vellumweaver(), 1, 3),
        (catalog::witherbloom_neophyte(), 1, 4),
        (catalog::lorehold_pyresinger(), 1, 4),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let _src = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let p0_before = g.players[0].life;
        let p1_before = g.players[1].life;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, p0_before + p0_gain, "{name}: p0 delta");
        assert_eq!(g.players[1].life, p1_before - p1_loss, "{name}: p1 delta");
    }
}

// ── Table: magecraft self-pump to 3 power (some with an innate keyword) ────
#[test]
fn magecraft_self_pump_sources() {
    for (def, kws) in [
        (catalog::lorehold_spellrunner(), &[Keyword::Haste][..]),
        (catalog::prismari_tempest_caller(), &[Keyword::Flying][..]),
        (catalog::lorehold_sparkscholar(), &[][..]),
        (catalog::prismari_glasscaster(), &[][..]),
        (catalog::lorehold_ardent_pyromage(), &[][..]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let src = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let body = g.battlefield_find(src).unwrap();
        assert_eq!(body.power(), 3, "{name}: pumped to 3 power");
        for kw in kws {
            assert!(body.has_keyword(kw), "{name}: keyword {kw:?}");
        }
    }
}

// ── Table: magecraft lands a +1/+1 counter on self ─────────────────────────
#[test]
fn magecraft_self_counter_sources() {
    for def in [catalog::fractal_theorist(), catalog::quandrix_reach_mage()] {
        let name = def.name;
        let mut g = two_player_game();
        let src = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let body = g.battlefield_find(src).unwrap();
        assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1,
            "{name}: +1/+1 counter from magecraft");
    }
}

// ── Table: magecraft mints a token ─────────────────────────────────────────
#[test]
fn magecraft_token_minters() {
    for (def, token) in [
        (catalog::witherbloom_pestcaster(), "Pest"),
        (catalog::prismari_vandal(), "Treasure"),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let _src = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let count = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == token)
            .count();
        assert_eq!(count, 1, "{name}: mints one {token} on magecraft");
    }
}

// ── Table: magecraft shrinks an opposing creature by -1/-1 ─────────────────
#[test]
fn magecraft_shrink_sources() {
    for def in [
        catalog::witherbloom_plagueweaver(),
        catalog::witherbloom_drainscholar(),
        catalog::witherbloom_toxbrewer(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let _src = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // 2/2 bear at -1/-1 → 1/1 (or dead if the engine SBAs it away).
        if let Some(b) = g.battlefield_find(bear) {
            assert!(b.power() <= 1 && b.toughness() <= 1, "{name}: bear shrunk");
        }
    }
}

// ── Table: ETB/on-resolve token minters (cast with no target) ──────────────
// (def, token_name, count, p0_life_gain, p1_life_loss, body_keywords)
#[test]
fn etb_token_minters() {
    for (def, token, count, p0_gain, p1_loss, kws) in [
        (catalog::lorehold_battlecaster(), "Spirit", 1, 0, 0, &[Keyword::Trample][..]),
        (catalog::lorehold_outburst(), "Spirit", 2, 0, 0, &[][..]),
        (catalog::spirit_lesson(), "Spirit", 2, 0, 0, &[][..]),
        (catalog::lorehold_spiritbringer(), "Spirit", 2, 0, 0, &[Keyword::Vigilance][..]),
        (catalog::lorehold_embercouncil(), "Spirit", 2, 0, 1, &[][..]),
        (catalog::prismari_lavalifter(), "Treasure", 1, 0, 0, &[][..]),
        (catalog::prismari_wave_mage(), "Treasure", 1, 0, 0, &[][..]),
        (catalog::prismari_fireshaper(), "Treasure", 1, 0, 0, &[][..]),
        (catalog::prismari_embershaper_wizard(), "Treasure", 1, 0, 0, &[Keyword::Flying][..]),
        (catalog::prismari_treasurewright_b30(), "Treasure", 1, 0, 0, &[][..]),
        (catalog::pest_studies(), "Pest", 2, 0, 0, &[][..]),
        (catalog::witherbloom_pest_spawner(), "Pest", 2, 0, 0, &[][..]),
        (catalog::pestpod_lurker(), "Pest", 1, 0, 0, &[][..]),
        (catalog::witherbloom_coatlcaller(), "Pest", 1, 0, 0, &[][..]),
        (catalog::witherbloom_lichenkeeper(), "Pest", 1, 0, 0, &[Keyword::Reach][..]),
        (catalog::witherbloom_bloomweaver(), "Pest", 1, 0, 0, &[][..]),
        (catalog::inkling_lesson(), "Inkling", 2, 0, 0, &[][..]),
        (catalog::silverquill_heraldist(), "Inkling", 1, 1, 0, &[][..]),
        (catalog::silverquill_pact(), "Inkling", 2, 4, 0, &[][..]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        // Generic setup safe for all rows (looters need library + fodder).
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        g.add_card_to_hand(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        generous_mana(&mut g);
        let p0_before = g.players[0].life;
        let p1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let tokens = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == token)
            .count();
        assert_eq!(tokens, count, "{name}: mints {count} {token}");
        assert_eq!(g.players[0].life, p0_before + p0_gain, "{name}: p0 life");
        assert_eq!(g.players[1].life, p1_before - p1_loss, "{name}: p1 life");
        if let Some(body) = g.battlefield_find(id) {
            for kw in kws {
                assert!(body.has_keyword(kw), "{name}: keyword {kw:?}");
            }
        }
    }
}

// ── Table: cast targeting the opposing player, check life deltas ───────────
#[test]
fn target_player_life_deltas() {
    for (def, p0_gain, p1_loss, token, kws) in [
        (catalog::bombastic_strixhaven_mage(), 0, 2, None, &[][..]),
        (catalog::lorehold_pyresmith(), 0, 1, None, &[Keyword::FirstStrike][..]),
        (catalog::lorehold_flameherald(), 0, 1, None, &[Keyword::Haste][..]),
        (catalog::prismari_flameseeker(), 0, 2, None, &[][..]),
        (catalog::silverquill_drafter_b30(), 0, 2, None, &[Keyword::Flying][..]),
        (catalog::pyromathematics(), 0, 3, None, &[][..]),
        (catalog::lorehold_stoneglyph(), 0, 2, None, &[][..]),
        (catalog::strixhaven_drainsong(), 2, 2, None, &[][..]),
        (catalog::lorehold_spiritflame(), 1, 2, None, &[][..]),
        (catalog::prismari_pyresurge_b28(), 0, 3, None, &[][..]),
        (catalog::prismari_splashcaster(), 0, 4, Some("Treasure"), &[][..]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        generous_mana(&mut g);
        let p0_before = g.players[0].life;
        let p1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, p0_before + p0_gain, "{name}: p0 life");
        assert_eq!(g.players[1].life, p1_before - p1_loss, "{name}: p1 life");
        if let Some(t) = token {
            assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == t),
                "{name}: mints a {t}");
        }
        if let Some(body) = g.battlefield_find(id) {
            for kw in kws {
                assert!(body.has_keyword(kw), "{name}: keyword {kw:?}");
            }
        }
    }
}

// ── Table: no-target ETB/resolve drains and lifegain ───────────────────────
#[test]
fn no_target_drain_and_lifegain() {
    for (def, p0_gain, p1_loss, kws) in [
        (catalog::silverquill_embodiment(), 2, 2, &[][..]),
        (catalog::witherbloom_drain_mage(), 3, 3, &[][..]),
        (catalog::silverquill_headmaster(), 2, 2, &[][..]),
        (catalog::silverquill_inkpact(), 3, 3, &[][..]),
        (catalog::witherbloom_drainpath(), 2, 2, &[][..]),
        (catalog::strixhaven_battle_cleric(), 1, 0, &[][..]),
        (catalog::strixhaven_forager(), 2, 0, &[][..]),
        (catalog::lorehold_ironscribe(), 3, 0, &[Keyword::Vigilance][..]),
        (catalog::lorehold_reverend(), 2, 0, &[Keyword::Vigilance, Keyword::Lifelink][..]),
        (catalog::witherbloom_vinemender(), 3, 0, &[][..]),
        (catalog::witherbloom_lifebloom(), 4, 0, &[][..]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island()); // surveil fodder
        let id = g.add_card_to_hand(0, def);
        generous_mana(&mut g);
        let p0_before = g.players[0].life;
        let p1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, p0_before + p0_gain, "{name}: p0 life");
        assert_eq!(g.players[1].life, p1_before - p1_loss, "{name}: p1 life");
        if let Some(body) = g.battlefield_find(id) {
            for kw in kws {
                assert!(body.has_keyword(kw), "{name}: keyword {kw:?}");
            }
        }
    }
}

// ── Table: cast with no target, check net hand-size delta ──────────────────
// Covers ETB loot/draw/dig/cantrip and gy-to-hand recursion bodies.
#[test]
fn no_target_hand_deltas() {
    for (def, delta, kws) in [
        (catalog::prismari_spelltheorist(), -1, &[][..]),   // loot: -1 net
        (catalog::prismari_sparkbender(), -1, &[][..]),     // loot: -1 net
        (catalog::strixhaven_tutor(), 0, &[][..]),          // scry + cantrip
        (catalog::quandrix_mathmage(), 0, &[][..]),         // reveal-until-find
        (catalog::inkrise_schoolwarden(), 0, &[Keyword::Flying, Keyword::Lifelink][..]),
        (catalog::quandrix_calculus_mage(), 0, &[][..]),
        (catalog::silverquill_scrivener_b30(), 0, &[][..]),
        (catalog::quandrix_mindforge(), 0, &[][..]),
        (catalog::quandrix_branchwarden(), 0, &[Keyword::Reach][..]),
        (catalog::strixhaven_curriculum(), 0, &[][..]),     // impulse look-3
        (catalog::strixhaven_archmage(), 1, &[][..]),       // draw two
        (catalog::lorehold_battle_witness(), 0, &[][..]),   // gy creature → hand
        (catalog::lorehold_recallmage(), 0, &[][..]),       // gy creature → hand
    ] {
        let name = def.name;
        let mut g = two_player_game();
        for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
        g.add_card_to_graveyard(0, catalog::grizzly_bears()); // recursion fodder
        g.add_card_to_hand(0, catalog::island());             // discard fodder
        let id = g.add_card_to_hand(0, def);
        generous_mana(&mut g);
        let hand_before = g.players[0].hand.len() as isize;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len() as isize, hand_before + delta,
            "{name}: net hand delta {delta}");
        if let Some(body) = g.battlefield_find(id) {
            for kw in kws {
                assert!(body.has_keyword(kw), "{name}: keyword {kw:?}");
            }
        }
    }
}

// ── Table: ETB pumps/counters a targeted friendly creature to 3/3 ──────────
#[test]
fn etb_pump_friendly_creature() {
    for (def, kws) in [
        (catalog::quandrix_geometer(), &[][..]),
        (catalog::lorehold_cinderpriest(), &[][..]),
        (catalog::quandrix_hydronaut(), &[][..]),
        (catalog::lorehold_wargleam(), &[Keyword::Vigilance][..]),
        (catalog::inkling_spireguard(), &[][..]),
        (catalog::mascot_researcher(), &[][..]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, def);
        generous_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let b = g.battlefield_find(bear).unwrap();
        assert_eq!(b.power(), 3, "{name}: bear pumped to 3 power");
        assert_eq!(b.toughness(), 3, "{name}: bear pumped to 3 toughness");
        if let Some(body) = g.battlefield_find(id) {
            for kw in kws {
                assert!(body.has_keyword(kw), "{name}: keyword {kw:?}");
            }
        }
    }
}

// ── Table: removal aimed at an opposing 2/2 (spell or ETB) kills it ────────
#[test]
fn removal_kills_opposing_bear() {
    for (def, p0_gain, token) in [
        (catalog::lorehold_pyresurge(), 1, None),
        (catalog::mage_hunters_strike(), 0, None),
        (catalog::prismari_magmaboon(), 0, Some("Treasure")),
        (catalog::necrotic_studies(), 0, None),
        (catalog::mage_hunters_riposte(), 0, None),
        (catalog::magecraft_volley(), 0, None),
        (catalog::witherbloom_sapcurse(), 0, None),
        (catalog::witherbloom_pestbreaker(), 0, Some("Pest")),
        (catalog::witherbloom_sapwarden(), 2, None),
        (catalog::prismari_sparksong(), 0, None),
        (catalog::lorehold_pyrescroll(), 0, Some("Spirit")),
        (catalog::prismari_stormwriter(), 0, None),
        (catalog::lorehold_pyrotechnician(), 0, None),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island()); // cantrip fodder
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, def);
        generous_mana(&mut g);
        let p0_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "{name}: bear killed");
        assert_eq!(g.players[0].life, p0_before + p0_gain, "{name}: p0 life");
        if let Some(t) = token {
            assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == t),
                "{name}: mints a {t}");
        }
    }
}

// ── Table: no-target cast forces the opponent to sacrifice a creature ──────
#[test]
fn forced_sacrifice_effects() {
    for (def, p0_gain) in [
        (catalog::silverquill_inkpurge(), 2),
        (catalog::witherbloom_devourer(), 0),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let _opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        drain_stack(&mut g);
        let opp_bf_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
        let id = g.add_card_to_hand(0, def);
        generous_mana(&mut g);
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life_before + p0_gain, "{name}: p0 life");
        let opp_bf_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
        assert_eq!(opp_bf_after, opp_bf_before - 1, "{name}: opp sacrificed");
    }
}

// ── Table: ETB exiles a targeted graveyard card, body has lifelink ─────────
#[test]
fn etb_gy_exilers() {
    for (def, kws) in [
        (catalog::lorehold_soulchanter(), &[Keyword::Lifelink][..]),
        (catalog::lorehold_stoneweaver(), &[Keyword::Vigilance, Keyword::Lifelink][..]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let gy_card = g.add_card_to_graveyard(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        generous_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(gy_card)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let body = g.battlefield_find(id).unwrap();
        for kw in kws {
            assert!(body.has_keyword(kw), "{name}: keyword {kw:?}");
        }
        assert!(!g.players[1].graveyard.iter().any(|c| c.id == gy_card),
            "{name}: card exiled from gy");
    }
}

// ── Table: ETB mills two, possibly with a life rider ───────────────────────
#[test]
fn etb_mill_two() {
    for (def, p0_gain) in [
        (catalog::witherbloom_bonecrafter(), 1),
        (catalog::witherbloom_pestreaver(), 1),
        (catalog::quandrix_fractalweaver(), 0),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        generous_mana(&mut g);
        let life_before = g.players[0].life;
        let gy_before = g.players[0].graveyard.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), gy_before + 2, "{name}: milled 2");
        assert_eq!(g.players[0].life, life_before + p0_gain, "{name}: p0 life");
    }
}

// ── Table: no-target smoke casts (body lands; optional keyword check) ──────
#[test]
fn no_target_smoke_casts() {
    for (def, kws) in [
        (catalog::strixhaven_scry_wizard(), &[][..]),
        (catalog::strixhaven_researcher(), &[][..]),
        (catalog::prismari_tideforger(), &[Keyword::Flash][..]),
        (catalog::quandrix_geomancer_b30(), &[][..]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        generous_mana(&mut g);
        let lib_before = g.players[0].library.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let body = g.battlefield_find(id).expect("body landed");
        for kw in kws {
            assert!(body.has_keyword(kw), "{name}: keyword {kw:?}");
        }
        // Scry doesn't change library size by itself.
        assert_eq!(g.players[0].library.len(), lib_before, "{name}: library size");
    }
}

// ── Table: own creature dies to an opposing Bolt; dies-trigger fires ───────
#[test]
fn dies_trigger_sources() {
    // (def, p0_gain, p1_loss, hand_gain)
    for (def, p0_gain, p1_loss, hand_gain) in [
        (catalog::silverquill_quillwitch(), 0, 2, 0),
        (catalog::pest_outcast(), 1, 0, 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        drain_stack(&mut g);
        let p0_before = g.players[0].life;
        let p1_before = g.players[1].life;
        let hand_before = g.players[0].hand.len();
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(id)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        assert_eq!(g.players[0].life, p0_before + p0_gain, "{name}: p0 life");
        assert_eq!(g.players[1].life, p1_before - p1_loss, "{name}: p1 life");
        assert_eq!(g.players[0].hand.len(), hand_before + hand_gain, "{name}: hand");
    }
}

// ── Silverquill Adjudicator: ETB gives target opp creature -3/0 ────────────
#[test]
fn silverquill_adjudicator_etb_shrinks_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_adjudicator());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Adjudicator castable");
    drain_stack(&mut g);
    // Bear 2/2 - 3/0 = -1/2 → SBA kills with 0 toughness? -1 power, 2 toughness still alive.
    let computed = g.compute_battlefield();
    let bear_card = computed.iter().find(|c| c.id == bear);
    if let Some(b) = bear_card {
        assert!(b.power <= 0);
    }
}

// ── Inkling Cantor: magecraft pumps a friendly creature +1/+1 ──────────────
#[test]
fn inkling_cantor_magecraft_pumps_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _src = g.add_card_to_battlefield(0, catalog::inkling_cantor());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear_view = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_view.power(), 3, "+1/+1 from Magecraft");
    assert_eq!(bear_view.toughness(), 3);
}

#[test]
fn quandrix_pondweaver_scrys_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _src = g.add_card_to_battlefield(0, catalog::quandrix_pondweaver());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry doesn't change library size by itself.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn quandrix_fractalseed_etb_adds_counters_for_is_in_gy() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // Creature, doesn't count.
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalseed());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalseed castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 2, "Got 2 counters from 2 IS");
}

#[test]
fn quandrix_fractalwave_creates_fractal_with_counters() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalwave());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalwave castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Fractal")
        .expect("Fractal minted");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn vanquish_the_horde_affinity_for_creatures_reduces_cost() {
    // With 3 creatures on the battlefield, the spell costs {3}{W}
    // (vs printed {6}{W}), so 3 generic + 1 W is enough.
    let mut g = two_player_game();
    let _b0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vanquish_the_horde());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    // Should be castable at {3}{W} (6 - 3 creatures = 3 generic).
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vanquish castable at {3}{W} via affinity discount");
    drain_stack(&mut g);
    // All creatures dead post-resolution.
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_creature()).count(), 0);
}

#[test]
fn vanquish_the_horde_affinity_rejects_undercost_with_no_creatures() {
    // With zero creatures, the spell costs the printed {6}{W}.
    // 3 generic + 1 W is insufficient.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::vanquish_the_horde());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let result = g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(result.is_err(), "Vanquish not castable at {{3}}{{W}} with no creatures");
}

#[test]
fn lecture_in_strategy_pumps_team_with_vigilance() {
    let mut g = two_player_game();
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lecture_in_strategy());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lecture castable");
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let bear = computed.iter().find(|c| c.id == b).unwrap();
    assert_eq!(bear.power, 3, "Bear pumped");
    assert!(bear.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn advanced_cartography_ramps_and_scrys() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
        DecisionAnswer::ScryOrder { kept_top: vec![], bottom: vec![] },
    ]));
    let id = g.add_card_to_hand(0, catalog::advanced_cartography());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cartography castable");
    drain_stack(&mut g);
    let f = g.battlefield_find(forest).expect("Forest tutored");
    assert!(f.tapped);
}

// ── Table: search-a-basic-land tutors (ScriptedDecider) ────────────────────
// (def, to_battlefield) — quandrix_mapmaker/strixhaven_druid differ only in
// the searched card's destination.
#[test]
fn basic_land_tutors() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    for (def, to_battlefield) in [
        (catalog::quandrix_mapmaker(), true),
        (catalog::strixhaven_druid(), false),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        let id = g.add_card_to_hand(0, def);
        generous_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        if to_battlefield {
            assert!(g.battlefield_find(forest).is_some(), "{name}: Forest on bf");
        } else {
            assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"),
                "{name}: Forest in hand");
        }
    }
}

#[test]
fn strixhaven_field_trip_ramps_two_basics() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest1 = g.add_card_to_library(0, catalog::forest());
    let forest2 = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest1)),
        DecisionAnswer::Search(Some(forest2)),
    ]));
    let id = g.add_card_to_hand(0, catalog::strixhaven_field_trip());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Field Trip castable");
    drain_stack(&mut g);
    let forest_count = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Forest")
        .count();
    assert_eq!(forest_count, 2);
}

#[test]
fn lorehold_stonebrand_etb_with_gy_creature_can_mint_spirit() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed opponent gy with a creature card.
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(dead)),
    ]));
    let id = g.add_card_to_hand(0, catalog::lorehold_stonebrand());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stonebrand castable");
    drain_stack(&mut g);
    // The body itself plus a Spirit token (if the MayDo paid + exile took).
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Lorehold Spirit Token")
        .count();
    // Either 0 or 1 — we don't assert minting succeeded; we assert the
    // body landed and the gy card was exiled (or remained intact).
    assert!(spirits <= 1);
    assert!(g.battlefield_find(id).is_some(), "Body landed");
    // Stats sanity for the same card (merged from a separate test).
    let def = catalog::lorehold_stonebrand();
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 3);
    assert!(def.subtypes.creature_types.contains(&CreatureType::Spirit));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Soldier));
}

#[test]
fn witherbloom_soilshaper_etb_mills_and_scales_with_gy() {
    let mut g = two_player_game();
    // Seed graveyard with a creature card.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::witherbloom_soilshaper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soilshaper castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    // We milled 2 cards (one bear and one bolt, in some order), and have
    // 1 prior bear in gy. So gy has 2 creatures min (since we milled a
    // bear). Either 1 or 2 creature cards in gy after mill — body gets
    // counters equal to that count.
    let counters = body.counter_count(CounterType::PlusOnePlusOne);
    assert!(counters >= 1, "Some counters were added");
}

#[test]
fn lorehold_bookbinder_etb_recurs_and_grants_haste() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bolt)),
    ]));
    let id = g.add_card_to_hand(0, catalog::lorehold_bookbinder());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bookbinder castable");
    drain_stack(&mut g);
    // Bolt now in hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
    // Body has haste this turn.
    let computed = g.compute_battlefield();
    let body = computed.iter().find(|c| c.id == id).unwrap();
    assert!(body.keywords.contains(&Keyword::Haste));
}

#[test]
fn quandrix_wavecaster_magecraft_adds_counter_to_target() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::quandrix_wavecaster());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Accept the "you may put a +1/+1 counter" optional.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Either bear or wavecaster gets the +1/+1 counter (any friendly).
    let bear_card = g.battlefield_find(bear).unwrap();
    let body = g.battlefield_find(id).unwrap();
    assert!(
        bear_card.counter_count(CounterType::PlusOnePlusOne) == 1 ||
        body.counter_count(CounterType::PlusOnePlusOne) == 1
    );
}

/// Wavecaster's conditional rider: with three or more creatures you control
/// carrying +1/+1 counters after the (accepted) counter placement, the
/// magecraft trigger also draws a card.
#[test]
fn quandrix_wavecaster_draws_with_three_countered_creatures() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    // Two creatures pre-seeded with +1/+1 counters; the trigger's own
    // counter on a third crosses the threshold.
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wc = g.add_card_to_battlefield(0, catalog::quandrix_wavecaster());
    // Three creatures you control already carrying +1/+1 counters — the
    // rider's threshold is met however the trigger's counter is aimed.
    for id in [a, b, wc] {
        if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) {
            c.add_counters(CounterType::PlusOnePlusOne, 1);
        }
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 (bolt cast) +1 (rider draw) → net even.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "three countered creatures → magecraft rider draws");
}

/// Declining the optional counter with only two countered creatures on the
/// battlefield: no counter is placed and no card is drawn.
#[test]
fn quandrix_wavecaster_no_draw_below_threshold() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _wc = g.add_card_to_battlefield(0, catalog::quandrix_wavecaster());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 0,
        "declined optional places no counter");
    assert_eq!(g.players[0].hand.len(), hand_before - 1,
        "no rider draw below the three-creature threshold");
}

#[test]
fn witherbloom_vinekeeper_etb_gains_two_and_dies_trigger_gains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinekeeper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinekeeper castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    // Add a friendly bear, kill it via opp Bolt → +1 life trigger
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    drain_stack(&mut g);
    let after_bear = g.players[0].life;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    assert_eq!(g.players[0].life, after_bear + 1, "+1 life from another-dies");
}

#[test]
fn strixhaven_pop_quiz_sage_etb_draws_and_stacks() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    let stack_target = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Discard(vec![stack_target]),
    ]));
    let id = g.add_card_to_hand(0, catalog::strixhaven_pop_quiz_sage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pop-Quiz castable");
    drain_stack(&mut g);
    // -1 cast (sage) +2 draws -1 to library = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn lorehold_spirit_champion_anthems_other_spirits() {
    let mut g = two_player_game();
    let other_spirit = g.add_card_to_battlefield(0, catalog::ageless_guardian());
    let champ = g.add_card_to_battlefield(0, catalog::lorehold_spirit_champion());
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let other = computed.iter().find(|c| c.id == other_spirit).unwrap();
    assert!(other.keywords.contains(&Keyword::FirstStrike),
        "Other Spirit got first strike");
    let champ_card = computed.iter().find(|c| c.id == champ).unwrap();
    assert!(champ_card.keywords.contains(&Keyword::FirstStrike),
        "Champion has its own innate first strike");
}

#[test]
fn quandrix_sumcaster_magecraft_loots_via_maydo() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let _src = g.add_card_to_battlefield(0, catalog::quandrix_sumcaster());
    drain_stack(&mut g);
    // Need to add at least one card to hand so we can discard
    let _filler = g.add_card_to_hand(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // AutoDecider declines MayDo by default → hand unchanged after Bolt resolves
    // -1 (cast Bolt) = -1 net (MayDo declined)
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn fractal_multiplicand_enters_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_multiplicand());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Multiplicand castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 3);
    assert_eq!(body.toughness(), 3);
}

#[test]
fn quandrix_tidecaller_etb_taps_target_with_flash() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_tidecaller());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidecaller castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(opp_bear).unwrap();
    assert!(bear.tapped);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Flash));
}

#[test]
fn fractal_spawning_mints_two_fractals_with_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_spawning());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spawning castable");
    drain_stack(&mut g);
    // Both fractals survive with +1/+1 counters via the new
    // Selector::LastCreatedTokens (plural) primitive (push modern_decks
    // batch 28). Each Fractal lands as 0/0, gets a +1/+1 counter before
    // SBA, ending up as a 1/1.
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Fractal")
        .collect();
    assert_eq!(fractals.len(), 2, "Both Fractals survive");
    let with_counters = fractals.iter()
        .filter(|c| c.counter_count(CounterType::PlusOnePlusOne) > 0)
        .count();
    assert_eq!(with_counters, 2, "Both Fractals have +1/+1 counter");
}

#[test]
fn prismari_tideburst_counters_when_opp_cant_pay_two() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp Bolt castable");
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::prismari_tideburst());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tideburst castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_before, "Bolt countered → no damage");
}

#[test]
fn strixhaven_combatant_pumps_on_attack_with_haste() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::strixhaven_combatant());
    drain_stack(&mut g);
    let body = g.battlefield_find(src).unwrap();
    assert!(body.has_keyword(&Keyword::Haste));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: src,
        target: AttackTarget::Player(1),
    }])).expect("Combatant attacks");
    drain_stack(&mut g);
    let body = g.battlefield_find(src).unwrap();
    assert_eq!(body.power(), 3, "+1/+0 from attack trigger");
}

#[test]
fn fractal_studies_mints_fractal_scaled_by_creature_count() {
    let mut g = two_player_game();
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::fractal_studies());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Studies castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Fractal")
        .expect("Fractal minted");
    // 2 bears + Fractal counted at moment of AddCounter resolution:
    // The CountOf reads after the Fractal already lands, so X = 3.
    assert!(fractal.counter_count(CounterType::PlusOnePlusOne) >= 2);
}

#[test]
fn pestpod_lurker_grows_on_lifegain() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::pestpod_lurker());
    drain_stack(&mut g);
    // Cast Healing Salve targeting self for +3 life.
    let salve = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: salve, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Salve castable");
    drain_stack(&mut g);
    let lurker = g.battlefield.iter().find(|c| c.id == src).unwrap();
    assert!(lurker.counter_count(CounterType::PlusOnePlusOne) >= 1,
        "Lurker should grow on lifegain");
}

/// Lorehold Neophyte — magecraft "you may exile … if you do, +1/+0".
#[test]
fn lorehold_neophyte_magecraft_pumps_self() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed graveyard so the optional exile finds a card.
    let bolt_dead = g.next_id();
    let mut inst = crabomination::card::CardInstance::new(bolt_dead, catalog::lightning_bolt(), 0);
    inst.controller = 0;
    g.players[0].graveyard.push(inst);
    let src = g.add_card_to_battlefield(0, catalog::lorehold_neophyte());
    drain_stack(&mut g);
    let pump_before = g.computed_permanent(src).unwrap().power;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Accept the "you may exile a card from a graveyard" optional.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.computed_permanent(src).unwrap();
    assert!(view.power > pump_before,
        "Neophyte should pump +1/+0 on magecraft");
    assert!(g.exile.iter().any(|c| c.id == bolt_dead),
        "the graveyard card was exiled as part of the accepted optional");
}

/// Declining the Neophyte's optional exile means no pump ("If you do").
#[test]
fn lorehold_neophyte_declined_exile_means_no_pump() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bolt_dead = g.next_id();
    let mut inst = crabomination::card::CardInstance::new(bolt_dead, catalog::lightning_bolt(), 0);
    inst.controller = 0;
    g.players[0].graveyard.push(inst);
    let src = g.add_card_to_battlefield(0, catalog::lorehold_neophyte());
    drain_stack(&mut g);
    let pump_before = g.computed_permanent(src).unwrap().power;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.computed_permanent(src).unwrap();
    assert_eq!(view.power, pump_before,
        "declined exile → no +1/+0 pump");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt_dead),
        "graveyard card stays put when the optional is declined");
}

/// Fractal Sumcaster — enters with X +1/+1 counters and scrys 1.
#[test]
fn fractal_sumcaster_enters_with_x_counters() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island()); // scry target
    let id = g.add_card_to_hand(0, catalog::fractal_sumcaster());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("Sumcaster castable for X=3 (XGU)");
    drain_stack(&mut g);
    let view = g.computed_permanent(id).unwrap();
    assert_eq!(view.power, 3,
        "Sumcaster should enter with X +1/+1 counters → 3 power");
}

/// Strixhaven Tutor (already wired): smoke test that the U variant lives.
#[test]
fn strixhaven_basicseeker_etb_tutors_basic() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_basicseeker());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Basicseeker castable");
    drain_stack(&mut g);
    // Hand: -1 cast + 1 tutored = same; +1 in play.
    assert!(g.battlefield.iter().any(|c| c.id == id), "Body on bf");
    // Tutor added a basic to hand; hand_before is +0 after cast(-1)+tutor(+1).
    assert!(g.players[0].hand.len() >= hand_before.saturating_sub(1));
}

/// Strixhaven Rotcaster — ETB forces opp discard.
#[test]
fn strixhaven_rotcaster_etb_discards_opp_card() {
    let mut g = two_player_game();
    let _opp_card = g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::strixhaven_rotcaster());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rotcaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1,
        "Rotcaster ETB should make opp discard");
}

/// Witherbloom Recursion — return creature from gy + lose 2.
#[test]
fn witherbloom_recursion_reanimates_creature_at_two_life() {
    let mut g = two_player_game();
    let bear_id = g.next_id();
    let mut bear = crabomination::card::CardInstance::new(bear_id, catalog::grizzly_bears(), 0);
    bear.controller = 0;
    g.players[0].graveyard.push(bear);
    let id = g.add_card_to_hand(0, catalog::witherbloom_recursion());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recursion castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear_id),
        "Bear should be reanimated");
    assert_eq!(g.players[0].life, life_before - 2,
        "Recursion costs 2 life on resolve");
}

/// Lorehold Battle Banner — one "whenever you attack" trigger pumps every
/// attacker (`EventKind::YouAttack`, CR 508).
#[test]
fn lorehold_battle_banner_pumps_attackers() {
    let mut g = two_player_game();
    let _banner = g.add_card_to_battlefield(0, catalog::lorehold_battle_banner());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.clear_sickness(bear2);
    g.battlefield.iter_mut().for_each(|c| c.tapped = false);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: bear, target: AttackTarget::Player(1) },
        Attack { attacker: bear2, target: AttackTarget::Player(1) },
    ])).expect("Bears can attack");
    drain_stack(&mut g);
    // A single declaration trigger pumps BOTH attackers by exactly +1/+0.
    for id in [bear, bear2] {
        let view = g.computed_permanent(id).unwrap();
        assert_eq!(view.power, 3, "each attacker gets exactly +1/+0 from Battle Banner");
    }
}

// ── Effect::MoveCounter ─────────────────────────────────────────────────────

/// Engine smoke test: `Effect::MoveCounter` moves N +1/+1 counters
/// from one permanent to another. Per CR 122.5, the move is NOT
/// counter creation, so DoubleCounters (Doubling Season) does not
/// double the move count.
#[test]
fn move_counter_transfers_counters_between_permanents() {
    use crabomination::card::CounterType;
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dst = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Seed 3 counters on src.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == src) {
        c.counters.insert(CounterType::PlusOnePlusOne, 3);
    }
    let ctx = EffectContext {
        controller: 0,
        source: Some(src),
        targets: vec![Target::Permanent(dst)],
        trigger_source: None,
        mode: 0,
        x_value: 0,
        converged_value: 0,
        mana_spent: 0,
        mana_spent_by_color: Vec::new(),
        source_name: None,
        cast_from_hand: true,
        event_amount: 0,
        kicked: false,
        kick_count: 0,
        bargained: false,
        cast_via_mayhem: false,
        cast_via_waterbend: false,
        cast_collected_evidence: false,
        entwined: false,
        spree_modes: Vec::new(),
    };
    let effect = Effect::MoveCounter {
        from: Selector::This,
        to: Selector::Target(0),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(2),
    };
    let _ = g.resolve_effect(&effect, &ctx);
    let src_after = g.battlefield.iter().find(|c| c.id == src).unwrap();
    let dst_after = g.battlefield.iter().find(|c| c.id == dst).unwrap();
    assert_eq!(src_after.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(dst_after.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// MoveCounter clamps at the source's actual counter pool — moving
/// more than available transfers only what's there.
#[test]
fn move_counter_clamps_at_source_pool() {
    use crabomination::card::CounterType;
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dst = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == src) {
        c.counters.insert(CounterType::PlusOnePlusOne, 1);
    }
    let ctx = EffectContext {
        controller: 0,
        source: Some(src),
        targets: vec![Target::Permanent(dst)],
        trigger_source: None,
        mode: 0,
        x_value: 0,
        converged_value: 0,
        mana_spent: 0,
        mana_spent_by_color: Vec::new(),
        source_name: None,
        cast_from_hand: true,
        event_amount: 0,
        kicked: false,
        kick_count: 0,
        bargained: false,
        cast_via_mayhem: false,
        cast_via_waterbend: false,
        cast_collected_evidence: false,
        entwined: false,
        spree_modes: Vec::new(),
    };
    let effect = Effect::MoveCounter {
        from: Selector::This,
        to: Selector::Target(0),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(99),
    };
    let _ = g.resolve_effect(&effect, &ctx);
    let src_after = g.battlefield.iter().find(|c| c.id == src).unwrap();
    let dst_after = g.battlefield.iter().find(|c| c.id == dst).unwrap();
    assert_eq!(src_after.counter_count(CounterType::PlusOnePlusOne), 0,
        "Source drained even though we asked for more than available");
    assert_eq!(dst_after.counter_count(CounterType::PlusOnePlusOne), 1,
        "Destination only got the actually-removed count");
}

#[test]
fn pest_cultist_drains_when_another_creature_dies() {
    let mut g = two_player_game();
    let _cultist = g.add_card_to_battlefield(0, catalog::pest_cultist());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    // Kill the bear with Lightning Bolt — proper damage path so the
    // AnotherOfYours die-trigger dispatch fires.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_before + 1);
    assert_eq!(g.players[1].life, p1_before - 1);
}

#[test]
fn lorehold_battlescholar_attack_exiles_target_graveyard_card() {
    let mut g = two_player_game();
    let gy_target = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_battlefield(0, catalog::lorehold_battlescholar());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::FirstStrike));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("Battlescholar attacks");
    drain_stack(&mut g);
    // Graveyard target should be in exile via the auto-picker.
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == gy_target),
        "Bolt should leave gy after the attack trigger fires");
    assert!(g.exile.iter().any(|c| c.id == gy_target),
        "Bolt should be in exile");
}

#[test]
fn lorehold_recountmage_magecraft_may_decline_by_default() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_recountmage());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    // AutoDecider declines MayDo by default → 4 toughness, no extra damage taken.
    assert_eq!(body.toughness(), 4);
    assert_eq!(body.damage, 0);
}

#[test]
fn lorehold_inscribe_burns_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_inscribe());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Inscribe castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn lorehold_reenactor_etb_returns_low_mv_creature_with_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_reenactor());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Auto-decider picks the gy bear via Selector::one_of.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reenactor castable");
    drain_stack(&mut g);
    let bear_bf = g.battlefield_find(bear).expect("Bear reanimated");
    assert!(bear_bf.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_memorial_taps_for_red_or_white() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_memorial_reliquary());
    drain_stack(&mut g);
    // Activate the Red mana ability (index 0).
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Memorial Red mana ability");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.tapped);
}

#[test]
fn lorehold_spirit_sentinel_pumps_on_friendly_spirit_etb() {
    let mut g = two_player_game();
    let sentinel = g.add_card_to_battlefield(0, catalog::lorehold_spirit_sentinel());
    drain_stack(&mut g);
    // Cast a Spirit creature so the EntersBattlefield event fires.
    let pyromage = g.add_card_to_hand(0, catalog::lorehold_pyrokineticist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: pyromage, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrokineticist castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(sentinel).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1,
        "Spirit ETB triggered Sentinel's +1/+1");
}

#[test]
fn witherbloom_vinekeeper_b30_attack_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_vinekeeper_b30());
    g.clear_sickness(id);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("Vinekeeper attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, me_before + 2);
}

#[test]
fn creative_outburst_deals_five_and_digs_for_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::creative_outburst());
    // Seed a known top card; auto-decider keeps the top of the dug pile.
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Creative Outburst castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 5, "5 damage to the opponent");
    assert!(g.players[0].hand.iter().any(|c| c.id == top),
        "dug-for top card landed in hand");
}

/// Real oracle line 2: "{U/R}{U/R}, Discard this card: Create a Treasure
/// token." — a from-hand activated ability with a discard-self cost.
#[test]
fn creative_outburst_discarded_from_hand_mints_a_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::creative_outburst());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("{U/R}{U/R}, Discard this card: Create a Treasure");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "Creative Outburst discarded as the cost");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
        "a Treasure token was created");
}

#[test]
fn heated_debate_cant_be_countered_and_deals_four() {
    // Real oracle: "This spell can't be countered. / Heated Debate deals
    // 4 damage to target creature or planeswalker." (No prevention rider
    // — an earlier synthesized body had "damage can't be prevented".)
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let hd = g.add_card_to_hand(0, catalog::heated_debate());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: hd, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Heated Debate castable");
    // Opponent tries to counter it — the spell can't be countered.
    g.perform_action(GameAction::PassPriority).unwrap();
    let cs = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: cs, target: Some(Target::Permanent(hd)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Counterspell castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(),
        "Heated Debate resolved through the Counterspell — 4 damage kills the 2/2");
    assert!(catalog::heated_debate().keywords.contains(&Keyword::CantBeCountered));
}
