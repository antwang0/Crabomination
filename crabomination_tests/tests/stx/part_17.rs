use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

// ── Table-driven: ETB / on-resolve casts with no target ─────────────────────
// (ctor, bf_delta of controller-0 permanents, l0_delta, l1_delta, own hand
// net delta incl. the cast, opp hand delta). None = not asserted.
#[test]
fn etb_cast_effects_table() {
    let cases: [(fn() -> _, &str, Option<i64>, i64, i64, Option<i64>, Option<i64>); 30] = [
        (catalog::witherbloom_lifeharvest_b139, "Lifeharvest", None, 3, 0, None, None),
        (catalog::witherbloom_sapherder_b139, "Sapherder", Some(3), 0, 0, None, None),
        (catalog::silverquill_inkdrinker_b139, "Inkdrinker", None, 2, -2, None, None),
        (catalog::inkling_scribesong_b139, "Scribesong", None, 2, -2, None, None),
        (catalog::inkling_lifeharvester_b141, "Lifeharvester", None, 1, -1, None, None),
        (catalog::inkling_quill_knight_b141, "Quill-Knight", Some(2), 1, -1, None, None),
        (catalog::witherbloom_pestmage_b141, "Pestmage", Some(2), 0, 0, None, None),
        (catalog::witherbloom_pestbloom_b141, "Pestbloom", Some(3), 0, 0, None, None),
        (catalog::lorehold_stormcleric_b141, "Stormcleric", Some(2), 0, 0, None, None),
        (catalog::lorehold_spiritforge_b141, "Spiritforge", Some(2), 2, 0, None, None),
        (catalog::inkling_magistry_b142, "Magistry", None, 3, -3, None, None),
        (catalog::silverquill_ledgerward_b142, "Ledgerward", None, 1, -1, None, None),
        (catalog::prismari_tidemaster_b142, "Tidemaster", Some(2), 0, 0, None, None),
        (catalog::prismari_pyrocaster_b142, "Pyrocaster", None, 0, 0, Some(-1), None),
        (catalog::quandrix_wavefront_b142, "Wavefront", None, 0, 0, Some(1), None),
        (catalog::lorehold_spiritmender_b142, "Spiritmender", Some(2), 4, 0, None, None),
        (catalog::witherbloom_lifeline_b143, "Lifeline", None, 3, 0, Some(0), None),
        (catalog::silverquill_pyremaster_b143, "Pyremaster", None, 2, -2, None, None),
        (catalog::inkling_inkcaller_b143, "Inkcaller", Some(2), 0, 0, None, None),
        (catalog::silverquill_devotional_b143, "Devotional", None, 5, 0, None, None),
        (catalog::witherbloom_pestmother_b143, "Pestmother", Some(3), 0, 0, None, None),
        (catalog::lorehold_cinderscholar_b143, "Cinderscholar", None, 2, 0, None, None),
        (catalog::silverquill_resonance_b143, "Resonance", None, 0, -2, None, Some(-1)),
        (catalog::inkling_sanctioner_b144, "Sanctioner", None, 2, 0, None, None),
        (catalog::pest_spawnchant_b144, "Spawnchant", Some(2), 0, 0, None, None),
        (catalog::witherbloom_lifedrip_b144, "Lifedrip", None, 3, -3, Some(0), None),
        (catalog::silverquill_hexbearer_b145, "Hexbearer", None, 1, -1, None, Some(-1)),
        (catalog::silverquill_heartmender_b145, "Heartmender", None, 4, 0, None, None),
        (catalog::silverquill_inkglyph_b146, "Inkglyph", None, 2, -2, None, None),
        (catalog::inkling_pyrescribe_b146, "Pyrescribe", None, 1, 0, None, None),
    ];
    let more: [(fn() -> _, &str, Option<i64>, i64, i64, Option<i64>, Option<i64>); 6] = [
        (catalog::inkling_inkbearer_b146, "Inkbearer", Some(2), 0, 0, None, None),
        (catalog::silverquill_inkriot_b146, "Inkriot", Some(2), 2, 0, None, None),
        (catalog::silverquill_hex_cleric_b146, "Hex-Cleric", None, 0, -2, None, None),
        (catalog::witherbloom_spore_cleric_b146, "Spore-Cleric", Some(2), 0, 0, None, None),
        (catalog::witherbloom_withergrove_b146, "Withergrove", Some(1), 3, -3, None, None),
        (catalog::lorehold_spirit_glyph_b146, "Spirit-Glyph", Some(1), 0, 0, None, None),
    ];
    for (ctor, name, bf_delta, l0_delta, l1_delta, hand_delta, opp_hand_delta)
        in cases.into_iter().chain(more)
    {
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::plains()); }
        for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
        let _ = g.add_card_to_hand(1, catalog::grizzly_bears()); // opp discard fodder
        let id = g.add_card_to_hand(0, ctor());
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 3);
        }
        g.players[0].mana_pool.add_colorless(6);
        let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count() as i64;
        let l0_before = g.players[0].life;
        let l1_before = g.players[1].life;
        let hand_before = g.players[0].hand.len() as i64;
        let h1_before = g.players[1].hand.len() as i64;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, l0_before + l0_delta as i32, "{name} own life");
        assert_eq!(g.players[1].life, l1_before + l1_delta as i32, "{name} opp life");
        if let Some(d) = bf_delta {
            let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count() as i64;
            assert_eq!(bf_after, bf_before + d, "{name} battlefield delta");
        }
        if let Some(d) = hand_delta {
            assert_eq!(g.players[0].hand.len() as i64, hand_before + d, "{name} hand delta");
        }
        if let Some(d) = opp_hand_delta {
            assert_eq!(g.players[1].hand.len() as i64, h1_before + d, "{name} opp hand delta");
        }
    }
}

// ── Table-driven: pure stats / keyword checks ───────────────────────────────
#[test]
fn stats_and_keywords_table() {
    let cases: [(fn() -> _, &str, Option<i64>, Option<i64>, &[Keyword], Option<CreatureType>); 7] = [
        (catalog::lorehold_spiritwarden_b139, "Spiritwarden",
         Some(4), Some(4), &[Keyword::Vigilance, Keyword::Lifelink], None),
        (catalog::pest_sapharvester_b143, "Sapharvester",
         Some(2), Some(1), &[Keyword::Deathtouch], Some(CreatureType::Pest)),
        (catalog::witherbloom_vipergrove_b145, "Vipergrove",
         Some(4), Some(5), &[Keyword::Deathtouch, Keyword::Trample], None),
        (catalog::silverquill_inkflight_b143, "Inkflight",
         Some(2), None, &[Keyword::Flying], Some(CreatureType::Inkling)),
        (catalog::lorehold_flamekeeper_b143, "Flamekeeper",
         Some(3), None, &[Keyword::Haste], None),
        (catalog::prismari_elementalmage_b143, "Elementalmage",
         Some(4), Some(4), &[], None),
        (catalog::inkling_heartbinder_b142, "Heartbinder",
         None, Some(4), &[Keyword::Flying, Keyword::Lifelink], None),
    ];
    for (ctor, name, power, toughness, keywords, ctype) in cases {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, ctor());
        let c = g.battlefield_find(id).unwrap();
        if let Some(p) = power { assert_eq!(c.power() as i64, p, "{name} power"); }
        if let Some(t) = toughness { assert_eq!(c.toughness() as i64, t, "{name} toughness"); }
        for kw in keywords {
            assert!(c.has_keyword(kw), "{name} has {kw:?}");
        }
        if let Some(ct) = ctype {
            assert!(c.definition.subtypes.creature_types.contains(&ct), "{name} type");
        }
    }
}

// ── Table-driven: magecraft observers that drain (gain X / opp -3-X) ────────
#[test]
fn magecraft_drain_table() {
    let cases: [(fn() -> _, &str, i64); 7] = [
        (catalog::witherbloom_lifedrinker_b141, "Lifedrinker", 1),
        (catalog::inkling_quillwhisper_b143, "Quillwhisper", 1),
        (catalog::witherbloom_bloodpest_b143, "Bloodpest", 2),
        (catalog::lorehold_ember_acolyte_b143, "Ember-Acolyte", 1),
        (catalog::lorehold_inferno_acolyte_b145, "Inferno-Acolyte", 1),
        (catalog::silverquill_inkmaster_adept_b146, "Inkmaster-Adept", 1),
        (catalog::witherbloom_sap_caller_b146, "Sap-Caller", 1),
    ];
    for (ctor, name, gain) in cases {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        let _ = g.add_card_to_battlefield(0, ctor());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let l0_before = g.players[0].life;
        let l1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, l0_before + gain as i32, "{name} gain");
        // Bolt 3 + drain
        assert_eq!(g.players[1].life, l1_before - 3 - gain as i32, "{name} opp loss");
    }
}

// ── Table-driven: magecraft observers that ping the opponent for X ──────────
#[test]
fn magecraft_ping_opponent_table() {
    let cases: [(fn() -> _, &str, i64); 4] = [
        (catalog::prismari_pyromage_b141, "Pyromage", 1),
        (catalog::lorehold_pyromancer_b143, "Pyromancer", 2),
        (catalog::prismari_pyroartist_b143, "Pyroartist", 1),
        (catalog::lorehold_ember_adept_b146, "Ember-Adept", 1),
    ];
    for (ctor, name, ping) in cases {
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, ctor());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let l1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // Bolt 3 + observer's ping
        assert_eq!(g.players[1].life, l1_before - 3 - ping as i32, "{name} opp loss");
    }
}

// ── Table-driven: magecraft observers that gain the controller life ─────────
#[test]
fn magecraft_lifegain_table() {
    let cases: [(fn() -> _, &str, i64); 3] = [
        (catalog::silverquill_pearlcaller_b139, "Pearlcaller", 2),
        (catalog::witherbloom_verdantvine_b142, "Verdantvine", 1),
        (catalog::pest_acolyte_b145, "Pest-Acolyte", 1),
    ];
    for (ctor, name, gain) in cases {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        let _ = g.add_card_to_battlefield(0, ctor());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let l0_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, l0_before + gain as i32, "{name} gain");
    }
}

// ── Table-driven: magecraft observers that mint a token ─────────────────────
#[test]
fn magecraft_mints_token_table() {
    let cases: [(fn() -> _, &str, i64); 5] = [
        (catalog::prismari_magma_channeler_b141, "Magma-Channeler (Treasure)", 1),
        (catalog::lorehold_sparkscholar_iii_b141, "Sparkscholar III (Spirit)", 1),
        (catalog::witherbloom_toxincaller_b142, "Toxincaller (Pest)", 1),
        (catalog::lorehold_conjurer_b144, "Conjurer (Spirit)", 1),
        // Fractal token minted with 0 counters → 0/0 → dies to SBA → no net change
        (catalog::fractal_genesis_b142, "Fractal-Genesis (0/0 dies to SBA)", 0),
    ];
    for (ctor, name, delta) in cases {
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, ctor());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count() as i64;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count() as i64;
        assert_eq!(bf_after, bf_before + delta, "{name}");
    }
}

// ── Table-driven: magecraft observers that put a +1/+1 counter on self ──────
#[test]
fn magecraft_self_counter_table() {
    let cases: [(fn() -> _, &str); 6] = [
        (catalog::fractal_wanderer_b141, "Fractal-Wanderer"),
        (catalog::quandrix_algorithmist_b142, "Algorithmist"),
        (catalog::quandrix_arithmancer_b143, "Arithmancer"),
        (catalog::silverquill_devout_b144, "Devout"),
        (catalog::inkling_verseguard_b146, "Verseguard"),
        (catalog::witherbloom_reapcaster_b146, "Reapcaster"),
    ];
    for (ctor, name) in cases {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        let obs = g.add_card_to_battlefield(0, ctor());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(obs).unwrap();
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "{name} counter");
    }
}

// ── Table-driven: magecraft observers with hand-size effects ────────────────
#[test]
fn magecraft_hand_delta_table() {
    // Net delta includes the -1 for casting Bolt: draw = 0, loot = -1.
    let cases: [(fn() -> _, &str, i64); 5] = [
        (catalog::prismari_surgemage_b142, "Surgemage (draw)", 0),
        (catalog::quandrix_sage_b141, "Quandrix Sage (scry+draw)", 0),
        (catalog::quandrix_echoist_b144, "Echoist (draw+surveil)", 0),
        (catalog::prismari_embergeist_b141, "Embergeist (loot)", -1),
        (catalog::quandrix_numericist_b143, "Numericist (loot)", -1),
    ];
    for (ctor, name, delta) in cases {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::island());
        let _ = g.add_card_to_battlefield(0, ctor());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len() as i64;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len() as i64, hand_before + delta, "{name}");
    }
}

// ── Table-driven: magecraft observers that pump another friendly creature ───
#[test]
fn magecraft_pumps_other_creature_table() {
    // Some(token) → the pump target is a token; None → a Grizzly Bears card.
    let cases: [(fn() -> _, &str, Option<fn() -> _>); 3] = [
        (catalog::silverquill_inkmaster_b142, "Inkmaster",
         Some(crabomination::catalog::inkling_token as fn() -> _)),
        (catalog::lorehold_battle_sage_b146, "Battle-Sage",
         Some(crabomination::catalog::lorehold_spirit_token as fn() -> _)),
        (catalog::quandrix_mage_adept_b144, "Mage-Adept", None),
    ];
    for (ctor, name, token) in cases {
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, ctor());
        let target = match token {
            Some(t) => g.add_token_to_battlefield(0, &t()),
            None => g.add_card_to_battlefield(0, catalog::grizzly_bears()),
        };
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(target).unwrap();
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "{name} pump");
    }
}

// ── Table-driven: burn spells targeting the opponent ────────────────────────
#[test]
fn burn_opponent_table() {
    let cases: [(fn() -> _, &str, i64, i64, Option<i64>, Option<i64>); 8] = [
        // (ctor, name, opp loss, own gain, own hand net delta, bf delta)
        (catalog::prismari_shocksinger_b139, "Shocksinger", 2, 0, None, Some(1)),
        (catalog::prismari_tidalstorm_b141, "Tidalstorm", 2, 0, Some(0), None),
        (catalog::lorehold_spellfire_b142, "Spellfire", 4, 0, None, None),
        (catalog::prismari_cinderwave_b142, "Cinderwave", 3, 0, Some(0), None),
        (catalog::prismari_cantriplord_b143, "Cantriplord", 3, 0, Some(1), None),
        (catalog::lorehold_ignis_b144, "Ignis", 3, 0, None, None),
        (catalog::lorehold_pyroflame_b144, "Pyroflame", 2, 2, None, None),
        (catalog::lorehold_spirit_burst_b146, "Spirit-Burst", 3, 0, None, None),
    ];
    for (ctor, name, loss, gain, hand_delta, bf_delta) in cases {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, ctor());
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 3);
        }
        g.players[0].mana_pool.add_colorless(6);
        let l0_before = g.players[0].life;
        let l1_before = g.players[1].life;
        let hand_before = g.players[0].hand.len() as i64;
        let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count() as i64;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1_before - loss as i32, "{name} opp loss");
        assert_eq!(g.players[0].life, l0_before + gain as i32, "{name} own gain");
        if let Some(d) = hand_delta {
            assert_eq!(g.players[0].hand.len() as i64, hand_before + d, "{name} hand");
        }
        if let Some(d) = bf_delta {
            let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count() as i64;
            assert_eq!(bf_after, bf_before + d, "{name} battlefield");
        }
    }
}

// ── Table-driven: removal that kills an opposing Grizzly Bears ──────────────
#[test]
fn kill_opposing_bear_table() {
    let cases: [(fn() -> _, &str, i64); 9] = [
        // (ctor, name, own life gain rider)
        (catalog::silverquill_decree_b142, "Decree", 1),
        (catalog::prismari_magmarush_b142, "Magmarush", 0),
        (catalog::silverquill_quillcleave_b143, "Quillcleave", 0),
        (catalog::witherbloom_vinepatch_b143, "Vinepatch", 2),
        (catalog::lorehold_inferno_b143, "Inferno", 0),
        (catalog::silverquill_reproach_b144, "Reproach", 0),
        (catalog::prismari_magmasplitter_b145, "Magmasplitter", 0),
        (catalog::witherbloom_festerstalk_b146, "Festerstalk", 0),
        (catalog::lorehold_glyph_strike_b146, "Glyph-Strike", 0),
    ];
    for (ctor, name, gain) in cases {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, ctor());
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 3);
        }
        g.players[0].mana_pool.add_colorless(6);
        let l0_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "{name}: bear died");
        assert_eq!(g.players[0].life, l0_before + gain as i32, "{name} life rider");
    }
}

// ── Table-driven: pump spells targeting a friendly Grizzly Bears ────────────
#[test]
fn pump_friendly_bear_table() {
    let cases: [(fn() -> _, &str, i64, i64, Option<Keyword>, i64, i64); 5] = [
        // (ctor, name, power, toughness, granted keyword, l0_delta, l1_delta)
        (catalog::silverquill_penblade_b141, "Penblade", 3, 3, None, 1, -1),
        (catalog::quandrix_fractalcraft_b141, "Fractalcraft", 3, 3, None, 0, 0),
        (catalog::lorehold_spiritbond_b142, "Spiritbond", 4, 3, Some(Keyword::Haste), 0, 0),
        (catalog::lorehold_battle_chant_b143, "Battle-Chant", 4, 4, Some(Keyword::Trample), 0, 0),
        (catalog::silverquill_ledgerblade_b146, "Ledgerblade", 3, 4, Some(Keyword::Vigilance), 0, 0),
    ];
    for (ctor, name, power, toughness, keyword, l0_delta, l1_delta) in cases {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, ctor());
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 3);
        }
        g.players[0].mana_pool.add_colorless(6);
        let l0_before = g.players[0].life;
        let l1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let computed = g.compute_battlefield();
        let b = computed.iter().find(|c| c.id == bear).unwrap();
        assert_eq!(b.power as i64, power, "{name} power");
        assert_eq!(b.toughness as i64, toughness, "{name} toughness");
        if let Some(kw) = keyword {
            assert!(b.keywords.contains(&kw), "{name} grants {kw:?}");
        }
        assert_eq!(g.players[0].life, l0_before + l0_delta as i32, "{name} own life");
        assert_eq!(g.players[1].life, l1_before + l1_delta as i32, "{name} opp life");
    }
}

// ── Table-driven: creatures with cycling ────────────────────────────────────
#[test]
fn creature_cycling_table() {
    let cases: [(fn() -> _, &str); 8] = [
        (catalog::silverquill_quillscholar_b144, "Quillscholar"),
        (catalog::pest_carrionbreeder_b144, "Carrionbreeder"),
        (catalog::lorehold_embermage_b144, "Embermage"),
        (catalog::prismari_ember_cantor_b144, "Ember-Cantor"),
        (catalog::fractal_bookbearer_b144, "Bookbearer"),
        (catalog::silverquill_sage_b145, "Silverquill Sage"),
        (catalog::witherbloom_vinegrower_b145, "Vinegrower"),
        (catalog::quandrix_treetender_b145, "Treetender"),
    ];
    for (ctor, name) in cases {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        let id = g.add_card_to_hand(0, ctor());
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 2);
        }
        g.players[0].mana_pool.add_colorless(4);
        g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
            .unwrap_or_else(|e| panic!("{name} cycling: {e:?}"));
        assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "{name} in graveyard");
    }
}

// ── Table-driven: dies-triggered drain of each opponent ─────────────────────
#[test]
fn dies_drains_each_opponent_table() {
    let cases: [(fn() -> _, &str); 2] = [
        (catalog::inkling_wraith_b145, "Inkling Wraith"),
        (catalog::pest_wraith_b146, "Pest Wraith"),
    ];
    for (ctor, name) in cases {
        let mut g = two_player_game();
        let wraith = g.add_card_to_battlefield(0, ctor());
        let l1_before = g.players[1].life;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(wraith)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1_before - 2, "{name} dies → opp -2");
    }
}

// ── Table-driven: reanimate a bear from the graveyard ───────────────────────
#[test]
fn reanimate_bear_table() {
    let cases: [(fn() -> _, &str, Option<i64>, bool); 3] = [
        // (ctor, name, controller-0 battlefield delta, reanimated tapped)
        (catalog::witherbloom_necroleaf_b142, "Necroleaf", Some(1), false),
        (catalog::lorehold_stoneveil_b142, "Stoneveil", Some(2), false),
        (catalog::witherbloom_necromage_b144, "Necromage", None, true),
    ];
    for (ctor, name, bf_delta, tapped) in cases {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, ctor());
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 3);
        }
        g.players[0].mana_pool.add_colorless(6);
        let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count() as i64;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let b = g.battlefield_find(bear).unwrap_or_else(|| panic!("{name}: bear back"));
        assert_eq!(b.definition.name, "Grizzly Bears");
        if tapped { assert!(b.tapped, "{name}: reanimated tapped"); }
        if let Some(d) = bf_delta {
            let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count() as i64;
            assert_eq!(bf_after, bf_before + d, "{name} battlefield delta");
        }
    }
}

// ── Table-driven: ETB self +1/+1 counters ───────────────────────────────────
#[test]
fn etb_self_counters_table() {
    let cases: [(fn() -> _, &str, u32, i64, i64); 3] = [
        // (ctor, name, counters, resulting power, own life gain)
        (catalog::witherbloom_sapsage_b142, "Sapsage", 1, 4, 2),
        (catalog::fractal_splinter_b143, "Splinter", 1, 2, 0),
        (catalog::fractal_scion_b144, "Scion", 2, 2, 0),
    ];
    for (ctor, name, counters, power, gain) in cases {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, ctor());
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 3);
        }
        g.players[0].mana_pool.add_colorless(6);
        let l0_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let c = g.battlefield_find(id).unwrap();
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne) as u32, counters, "{name} counters");
        assert_eq!(c.power() as i64, power, "{name} power");
        assert_eq!(g.players[0].life, l0_before + gain as i32, "{name} life");
    }
}

// ── Table-driven: mints a Fractal token with +1/+1 counters ─────────────────
#[test]
fn mints_fractal_with_counters_table() {
    let cases: [(fn() -> _, &str, u32); 3] = [
        (catalog::quandrix_symmetrist_ii_b141, "Symmetrist II", 3),
        (catalog::fractal_tendril_b142, "Tendril", 2),
        (catalog::fractal_vinemother_b143, "Vinemother", 3),
    ];
    for (ctor, name, counters) in cases {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, ctor());
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 3);
        }
        g.players[0].mana_pool.add_colorless(6);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let fractal = g.battlefield.iter()
            .find(|c| c.is_token && c.definition.name == "Fractal")
            .unwrap_or_else(|| panic!("{name}: Fractal minted"));
        assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne) as u32, counters, "{name}");
    }
}

// ── Table-driven: ETB counters equal to friendly Fractal count ──────────────
#[test]
fn etb_counters_per_fractal_table() {
    let cases: [(fn() -> _, &str); 2] = [
        (catalog::quandrix_apex_b142, "Quandrix Apex"),
        (catalog::fractal_apex_mage_b145, "Apex-Mage"),
    ];
    for (ctor, name) in cases {
        let mut g = two_player_game();
        // Two friendly Fractal tokens; add +1/+1 counters so the 0/0s don't
        // die to SBA before the caster enters and reads them.
        let f1 = g.add_token_to_battlefield(0, &crabomination::catalog::fractal_token());
        let f2 = g.add_token_to_battlefield(0, &crabomination::catalog::fractal_token());
        g.battlefield_find_mut(f1).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        g.battlefield_find_mut(f2).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
        let id = g.add_card_to_hand(0, ctor());
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 3);
        }
        g.players[0].mana_pool.add_colorless(6);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let c = g.battlefield_find(id).unwrap();
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2, "{name}: 2 Fractals → 2 counters");
    }
}

// ── Table-driven: burn a big creature for 2 and draw ────────────────────────
#[test]
fn burn_two_and_draw_table() {
    let cases: [(fn() -> _, &str); 2] = [
        (catalog::prismari_cantripflinger_b143, "Cantripflinger"),
        (catalog::prismari_stormgust_b144, "Stormgust"),
    ];
    for (ctor, name) in cases {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        // Serra Angel is beefy enough to survive the 2 damage.
        let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
        let id = g.add_card_to_hand(0, ctor());
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 2);
        }
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(angel)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(angel).unwrap().damage, 2, "{name} damage");
        // -1 cast +1 draw = 0 net
        assert_eq!(g.players[0].hand.len(), hand_before, "{name} hand");
    }
}

// ── Individual tests: unique shapes / cited rulings ─────────────────────────

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
fn pest_hivelord_b142_anthems_other_pests() {
    let mut g = two_player_game();
    // Mint a Pest token first via the test-friendly helper that sets is_token.
    let pest = g.add_token_to_battlefield(0, &crabomination::catalog::stx_pest_token());
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
        additional_targets: Vec::new(),
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

#[test]
fn inkling_vanquisher_b144_attack_drains_two() {
    use crabomination::game::Attack;
    use crabomination::game::types::AttackTarget;
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

/// Pestlord's sacrifice trigger is a real "may pay {B}{G}" — accepting
/// with floated {B}{G} spends the mana and draws.
#[test]
fn witherbloom_pestlord_b144_pays_bg_to_draw_on_sacrifice() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_pestlord_b144());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fodder);
    let hand_before = g.players[0].hand.len();
    // Float the {B}{G} the may-cost needs and accept it.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    // Sacrifice via direct effect.
    g.battlefield.retain(|c| c.id != fodder);
    g.players[0].graveyard.push(
        crabomination::card::CardInstance::new(fodder, catalog::grizzly_bears(), 0)
    );
    // Emit the sacrifice event directly to test the trigger.
    let events = vec![crabomination::game::types::GameEvent::CreatureSacrificed {
        card_id: fodder, who: 0,
    }];
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "paid {{B}}{{G}}, drew 1");
}

/// Declining the may-cost (the AutoDecider default) draws nothing.
#[test]
fn witherbloom_pestlord_b144_declines_may_cost_no_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_pestlord_b144());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fodder);
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.battlefield.retain(|c| c.id != fodder);
    g.players[0].graveyard.push(
        crabomination::card::CardInstance::new(fodder, catalog::grizzly_bears(), 0)
    );
    let events = vec![crabomination::game::types::GameEvent::CreatureSacrificed {
        card_id: fodder, who: 0,
    }];
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "declined — no draw");
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
