use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

/// Add generous mana of every color plus plenty of colorless so table-driven
/// tests don't need per-card mana setup.
fn add_generous_mana(g: &mut crabomination::game::Game) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 3);
    }
    g.players[0].mana_pool.add_colorless(10);
}

// ── Table-driven: magecraft "extra burn on instant cast" ────────────────────
// Card on bf; cast Bolt at opp; opp takes 3 (bolt) + N (magecraft).

#[test]
fn magecraft_burn_cards_ping_on_instant_cast() {
    for (def, extra) in [
        (catalog::prismari_sparkbinder(), 1),
        (catalog::silverquill_quillmage(), 1),
        (catalog::prismari_stormbringer(), 2),
        (catalog::lorehold_ember_priest(), 1),
        (catalog::lorehold_pyrosage(), 1),
        (catalog::prismari_pyrowriter(), 1),
        (catalog::prismari_pyrotechnician(), 1),
        (catalog::lorehold_reverberator(), 2),
        (catalog::lorehold_pyrescribe(), 1),
        (catalog::prismari_ember_channeler(), 1),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let life_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life_before - 3 - extra,
            "{name}: opp takes bolt 3 + magecraft {extra}");
    }
}

// ── Table-driven: magecraft drain 1 (opp -4 total, you +1) ──────────────────

#[test]
fn magecraft_drain_cards_drain_one_on_instant_cast() {
    for def in [
        catalog::witherbloom_seer(),
        catalog::inkling_coursebinder(),
        catalog::inkling_confessor(),
        catalog::witherbloom_lifebleeder(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let life0_before = g.players[0].life;
        let life1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life0_before + 1, "{name}: drained +1 life");
        assert_eq!(g.players[1].life, life1_before - 4, "{name}: opp -3 bolt -1 drain");
    }
}

// ── Table-driven: magecraft gain 1 life ─────────────────────────────────────

#[test]
fn magecraft_lifegain_cards_gain_one_on_instant_cast() {
    for def in [
        catalog::spelltongue_statute(),
        catalog::silverquill_witness(),
        catalog::lorehold_spectrescribe(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life_before + 1, "{name}: gained 1 life from cast");
    }
}

// ── Table-driven: magecraft self +1/+1 counter ──────────────────────────────

#[test]
fn magecraft_counter_cards_gain_counter_on_instant_cast() {
    for def in [
        catalog::silverquill_auctioneer(),
        catalog::quandrix_counterspeaker(),
        catalog::lorehold_bonepriest(),
        catalog::quandrix_doublecaster(),
        catalog::quandrix_sapsprout(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let count = g.battlefield_find(id).expect("on bf")
            .counter_count(CounterType::PlusOnePlusOne);
        assert_eq!(count, 1, "{name}: one +1/+1 counter from magecraft");
    }
}

// ── Table-driven: magecraft self-pump +1 power (EOT) ────────────────────────

#[test]
fn magecraft_selfpump_cards_pump_one_power_on_instant_cast() {
    for def in [
        catalog::silverquill_pupil(),
        catalog::lorehold_pyrebrand(),
        catalog::prismari_sparkmaster(),
        catalog::witherbloom_sapfiend(),
        catalog::witherbloom_sapdrinker(),
        catalog::prismari_drakelord(),
        catalog::silverquill_erudite(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let p_before = g.battlefield_find(id).map(|c| c.power()).unwrap_or(0);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let p_after = g.battlefield_find(id).map(|c| c.power()).unwrap_or(0);
        assert_eq!(p_after, p_before + 1, "{name}: self-pumps +1 power");
    }
}

// ── Table-driven: magecraft pumps friendly creature +1 ──────────────────────

#[test]
fn magecraft_pump_friendly_cards_pump_bear_on_instant_cast() {
    for def in [
        catalog::quandrix_scholar(),
        catalog::withergrowth_apprentice(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, def);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
        assert_eq!(p_after, p_before + 1, "{name}: Bear pumped +1");
    }
}

// ── Table-driven: magecraft loot (net hand -1) ──────────────────────────────

#[test]
fn magecraft_loot_cards_loot_on_instant_cast() {
    for def in [
        catalog::prismari_storm_caller(),
        catalog::prismari_stormcaster(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_hand(0, catalog::mountain()); // discard fodder
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // -1 (cast bolt) +1 (draw) -1 (discard) = -1 net.
        assert_eq!(g.players[0].hand.len(), hand_before - 1, "{name}: looted");
    }
}

// ── Table-driven: magecraft scry (library size unchanged, no panic) ─────────

#[test]
fn magecraft_scry_cards_scry_on_instant_cast() {
    for def in [
        catalog::silverquill_pen_pusher(),
        catalog::quandrix_scrycharmer(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let lib_before = g.players[0].library.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // Scry doesn't change library size (just reorders/sends top to bottom).
        assert_eq!(g.players[0].library.len(), lib_before, "{name}: scry only");
    }
}

// ── Table-driven: magecraft token minting ───────────────────────────────────

#[test]
fn magecraft_token_cards_mint_on_instant_cast() {
    for (def, token) in [
        (catalog::witherbloom_pestmancer(), "Pest"),
        (catalog::inkling_penmaster(), "Inkling"),
        (catalog::prismari_alchemist(), "Treasure"),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let tokens = g.battlefield.iter().filter(|c| {
            c.controller == 0 && c.is_token && c.definition.name == token
        }).count();
        assert_eq!(tokens, 1, "{name}: mints 1 {token} per instant cast");
    }
}

// ── Table-driven: ETB / on-cast token minting (untargeted spells) ───────────

#[test]
fn etb_token_cards_mint_named_tokens() {
    for (def, token, n) in [
        (catalog::pestbrood_grovecaller(), "Pest", 1),
        (catalog::witherbloom_pestbinder(), "Pest", 1),
        (catalog::witherbloom_pest_tender(), "Pest", 1),
        (catalog::witherbloom_mossfeeder(), "Pest", 1),
        (catalog::witherbloom_toxicultivator(), "Pest", 1),
        (catalog::witherbloom_pestkeeper(), "Pest", 1),
        (catalog::pest_cultivator(), "Pest", 2),
        (catalog::pest_outburst(), "Pest", 2),
        (catalog::pest_swarm(), "Pest", 3),
        (catalog::inkling_scribe(), "Inkling", 1),
        (catalog::inkling_brigade(), "Inkling", 2),
        (catalog::silverquill_sermon(), "Inkling", 2),
        (catalog::inkling_decree(), "Inkling", 1),
        (catalog::prismari_treasurewright(), "Treasure", 2),
        (catalog::prismari_chromaticist(), "Treasure", 1),
        (catalog::prismari_spellsmith(), "Treasure", 1),
        (catalog::lorehold_spiritcaller(), "Spirit", 1),
        (catalog::lorehold_echoist(), "Spirit", 1),
        (catalog::lorehold_spiritmaster(), "Spirit", 2),
        (catalog::lorehold_battlescroll(), "Spirit", 2),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let tokens = g.battlefield.iter().filter(|c| {
            c.controller == 0 && c.is_token && c.definition.name == token
        }).count();
        assert_eq!(tokens, n, "{name}: mints {n} {token} token(s)");
    }
}

// ── Table-driven: ETB / on-cast drain N (you +N, opp -N) ────────────────────

#[test]
fn etb_drain_cards_drain_n_life() {
    for (def, n, targeted) in [
        (catalog::witherbloom_hexweaver(), 2, false),
        (catalog::silverquill_drainmaster(), 3, false),
        (catalog::witherspell_drain(), 3, false),
        (catalog::witherbloom_reverie(), 3, false),
        (catalog::silverquill_castigant(), 1, false),
        (catalog::witherbloom_decoctor(), 2, false),
        (catalog::silverquill_heartrender(), 3, false),
        (catalog::defend_the_inkwell(), 2, false),
        (catalog::inkling_stormcaller(), 2, true),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g);
        let life0_before = g.players[0].life;
        let life1_before = g.players[1].life;
        let target = if targeted {
            Some(crabomination::game::types::Target::Player(1))
        } else { None };
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life0_before + n, "{name}: you gain {n}");
        assert_eq!(g.players[1].life, life1_before - n, "{name}: opp loses {n}");
    }
}

// ── Table-driven: ETB / on-cast burn opp for N ──────────────────────────────

#[test]
fn etb_burn_cards_deal_n_to_opp() {
    for (def, n, targeted) in [
        (catalog::lorehold_pyromage(), 3, true),
        (catalog::prismari_emberseer(), 2, false),
        (catalog::prismari_drakeward(), 2, false),
        (catalog::witherbloom_bonepicker(), 2, false),
        (catalog::prismari_ignite_apprentice(), 1, true),
        (catalog::lorehold_ember_brand(), 3, true),
        (catalog::silverquill_dictation(), 2, true),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g);
        let life_before = g.players[1].life;
        let target = if targeted {
            Some(crabomination::game::types::Target::Player(1))
        } else { None };
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life_before - n, "{name}: opp loses {n}");
    }
}

// ── Table-driven: ETB gain life ─────────────────────────────────────────────

#[test]
fn etb_lifegain_cards_gain_n_life() {
    for (def, n) in [
        (catalog::silverquill_loremender(), 2),
        (catalog::silverquill_marshal(), 2),
        (catalog::silverquill_archivist(), 1),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g);
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life_before + n, "{name}: gained {n} on ETB");
    }
}

// ── Table-driven: ETB draw (hand net 0 after cast) ──────────────────────────

#[test]
fn etb_draw_cards_replace_themselves() {
    for def in [
        catalog::prismari_mistcaller(),
        catalog::quandrix_symmetrist(),
        catalog::quandrix_wavewright(),
        catalog::quandrix_geomyst(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        // -1 (cast) +1 (etb draw) = same hand size.
        assert_eq!(g.players[0].hand.len(), hand_before, "{name}: replaced itself");
    }
}

// ── Table-driven: ETB scry, lands on battlefield ────────────────────────────

#[test]
fn etb_scry_cards_resolve_and_land_on_battlefield() {
    for def in [
        catalog::prismari_lightcaster(),
        catalog::quandrix_wavedancer(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.definition.name == name),
            "{name}: on battlefield after ETB scry");
    }
}

// ── Table-driven: ETB / spell loot (hand net -1) ────────────────────────────

#[test]
fn loot_cards_net_minus_one_hand() {
    for (def, targeted) in [
        (catalog::prismari_looter(), false),
        (catalog::prismari_spellsong(), true),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        g.add_card_to_hand(0, catalog::grizzly_bears()); // discard fodder
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g);
        let hand_before = g.players[0].hand.len();
        let target = if targeted {
            Some(crabomination::game::types::Target::Player(1))
        } else { None };
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        // -1 (cast) +1 (draw) -1 (discard) = -1 net.
        assert_eq!(g.players[0].hand.len(), hand_before - 1, "{name}: looted");
    }
}

// ── Table-driven: targeted removal kills/removes a Grizzly Bears ────────────

#[test]
fn removal_cards_remove_opp_bear() {
    for (def, mode) in [
        (catalog::silverquill_reaper(), None),
        (catalog::silverquill_reprimand(), None),
        (catalog::silverquill_censure(), None),
        (catalog::lorehold_ember_forge(), None),
        (catalog::prismari_volley(), None),
        (catalog::prismari_conflagration(), Some(0)),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(crabomination::game::types::Target::Permanent(bear)),
            additional_targets: vec![], mode, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "{name}: bear removed");
    }
}

// ── Table-driven: return low-MV creature from graveyard to hand ─────────────

#[test]
fn gy_return_cards_return_bear_to_hand() {
    for def in [
        catalog::witherbloom_reanimist(),
        catalog::silverquill_memorialist(),
        catalog::witherbloom_grand_necromancer(),
        catalog::witherbloom_recourse(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let _ = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(
            g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
            "{name}: Bear returned to hand"
        );
    }
}

// ── Individual tests (unique shapes, regressions, CR citations) ─────────────

#[test]
fn lorehold_cathedral_taps_for_red_or_white() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_cathedral());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("tap for R");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "added one red mana");
}

#[test]
fn quandrix_geomancer_etb_mints_fractals_per_land() {
    let mut g = two_player_game();
    // Pre-seed 4 lands.
    for _ in 0..4 {
        let _ = g.add_card_to_battlefield(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_geomancer());
    add_generous_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Geomancer castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal)
    }).collect();
    // 4 lands at ETB count → 4 Fractals.
    assert_eq!(fractals.len(), 4, "minted 4 Fractals for 4 lands");
}

#[test]
fn quandrix_fractalist_etb_enters_with_counters_per_hand() {
    let mut g = two_player_game();
    // Set hand to size 3 (after cast, hand is size 2). Add some filler.
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalist castable");
    drain_stack(&mut g);
    let bf = g.battlefield_find(id).expect("Fractalist on bf");
    // Hand has 2 (two islands) after the cast; ETB trigger reads
    // hand size = 2 → +2 +1/+1 counters.
    assert_eq!(bf.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn quandrix_skybinder_attack_drops_counter_on_friendly() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::quandrix_skybinder());
    let friendly = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // Synthesize an Attacks event for the Skybinder.
    {
        use crabomination::card::Effect;
        use crabomination::card::Selector;
        use crabomination::card::Value;
        let eff = Effect::AddCounter {
            what: Selector::Target(0),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        };
        use crabomination::game::effects::EffectContext;
        use crabomination::game::types::Target;
        let ctx = EffectContext::for_spell(0, Some(Target::Permanent(friendly)), 0, 0);
        g.resolve_effect(&eff, &ctx).expect("AddCounter resolves");
    }
    let bf = g.battlefield_find(friendly).expect("Bear");
    assert_eq!(bf.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_landmapper_ramps_and_scries() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Pre-seed a basic land in library + filler.
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::quandrix_landmapper());
    add_generous_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Landmapper castable");
    drain_stack(&mut g);
    // Land enters battlefield untapped per the Search.
    let forests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.name == "Forest"
    }).collect();
    assert_eq!(forests.len(), 1, "tutored a Forest");
}

#[test]
fn strixhaven_reservoir_taps_for_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::strixhaven_reservoir());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("Reservoir taps for color");
    drain_stack(&mut g);
    // AutoDecider picks a color (white by default).
    let mana = &g.players[0].mana_pool;
    let total = mana.amount(Color::White) + mana.amount(Color::Blue) + mana.amount(Color::Black)
        + mana.amount(Color::Red) + mana.amount(Color::Green);
    assert_eq!(total, 1, "got 1 mana from Reservoir");
}

#[test]
fn lone_rider_pumps_when_attacking_alone() {
    // Locks in CR 506.5 "attacking alone" predicate. The Lone Rider's
    // attack-trigger only fires when it's the only declared attacker.
    let mut g = two_player_game();
    let rider = g.add_card_to_battlefield(0, catalog::lone_rider());
    g.clear_sickness(rider);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rider, target: AttackTarget::Player(1),
    }])).expect("Rider attacks alone");
    drain_stack(&mut g);
    let view = g.computed_permanent(rider).expect("Rider on bf");
    assert_eq!(view.power, 3, "Rider 1 + 2 from alone-attack trigger");
    assert!(view.keywords.contains(&Keyword::Trample), "Trample EOT granted");
}

#[test]
fn lone_rider_does_not_pump_with_other_attackers() {
    let mut g = two_player_game();
    let rider = g.add_card_to_battlefield(0, catalog::lone_rider());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(rider);
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: rider, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("Both attack");
    drain_stack(&mut g);
    let view = g.computed_permanent(rider).expect("Rider on bf");
    assert_eq!(view.power, 1, "Rider not pumped (multiple attackers — not 'alone')");
    assert!(!view.keywords.contains(&Keyword::Trample), "No Trample (not alone)");
}

#[test]
fn solo_striker_pumps_when_attacking_alone() {
    let mut g = two_player_game();
    let striker = g.add_card_to_battlefield(0, catalog::solo_striker());
    g.clear_sickness(striker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: striker, target: AttackTarget::Player(1),
    }])).expect("Striker attacks alone");
    drain_stack(&mut g);
    let view = g.computed_permanent(striker).expect("Striker on bf");
    assert_eq!(view.power, 4, "Striker 3 + 1");
    assert_eq!(view.toughness, 4, "Striker 2 + 2");
    assert!(view.keywords.contains(&Keyword::Lifelink), "Lifelink granted");
    assert!(view.keywords.contains(&Keyword::Vigilance), "Vigilance intrinsic");
}

#[test]
fn quandrix_loremind_sac_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_battlefield(0, catalog::quandrix_loremind());
    g.clear_sickness(id);
    let hand_before = g.players[0].hand.len();
    add_generous_mana(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("Loremind activatable");
    drain_stack(&mut g);
    // Sacrificed → no longer on bf.
    assert!(g.battlefield_find(id).is_none(), "Loremind sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew 2 cards");
}

#[test]
fn inkrise_lifedrainer_combat_damage_gains_one_life() {
    let mut g = two_player_game();
    let drainer = g.add_card_to_battlefield(0, catalog::inkrise_lifedrainer());
    g.clear_sickness(drainer);
    while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: drainer, target: AttackTarget::Player(1),
    }])).expect("Drainer attacks");
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    while g.step != crabomination::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "gained 1 life from combat damage");
}

#[test]
fn silverquill_penman_and_anthemwriter_stats() {
    let def = catalog::silverquill_penman();
    assert_eq!(def.name, "Silverquill Penman");
    assert_eq!(def.power, 2);
    assert_eq!(def.toughness, 2);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Inkling));

    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_anthemwriter());
    let view = g.computed_permanent(id).expect("Anthemwriter on bf");
    assert_eq!(view.power, 4);
    assert_eq!(view.toughness, 4);
    assert!(view.keywords.contains(&Keyword::Flying));
    assert!(view.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn silverquill_inquisition_makes_opp_discard_a_card() {
    let mut g = two_player_game();
    let _ = g.add_card_to_hand(1, catalog::grizzly_bears());
    let _ = g.add_card_to_hand(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_inquisition());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inquisition castable");
    drain_stack(&mut g);
    // Opp lost 1 card (chosen by us — should be Bears since Island is filtered out).
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
}

#[test]
fn lorehold_bookburner_sac_pings_a_creature() {
    let mut g = two_player_game();
    let burner = g.add_card_to_battlefield(0, catalog::lorehold_bookburner());
    g.clear_sickness(burner);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: burner, ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None }).expect("Activatable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(burner).is_none(), "Burner sacrificed");
    // Bear (2 toughness) takes 2 damage → dies to SBA.
    assert!(g.battlefield_find(bear).is_none(), "Bear destroyed by 2 dmg");
}

#[test]
fn quandrix_tessellator_activated_mints_fractal_with_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_tessellator());
    g.clear_sickness(id);
    add_generous_mana(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("Activatable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Fractal"
    }).expect("Fractal token");
    let count = fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(count, 2, "Fractal has two +1/+1 counters");
}

#[test]
fn witherbloom_wanderer_pay_two_life_reanimates_creature() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_wanderer());
    add_generous_mana(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wanderer castable");
    drain_stack(&mut g);
    // -1 hand (cast Wanderer) + 1 hand (Bear returned) = same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].life, life_before - 2, "paid 2 life");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_gy),
        "Bear card in hand");
}

#[test]
fn strixhaven_vault_etb_scrys_then_sac_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::strixhaven_vault());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vault castable");
    drain_stack(&mut g);
    let vault = g.battlefield.iter().find(|c| c.definition.name == "Strixhaven Vault")
        .expect("Vault on bf");
    let vault_id = vault.id;
    // Now activate the sac-for-draw ability.
    g.clear_sickness(vault_id);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: vault_id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("Sac activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vault_id).is_none(), "Vault sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew 1 card");
}

#[test]
fn silverquill_judge_magecraft_taps_opponent_creature() {
    let mut g = two_player_game();
    let _judge = g.add_card_to_battlefield(0, catalog::silverquill_judge());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(bear).expect("Bear on bf");
    assert!(view.tapped, "Judge magecraft tapped the opp bear");
}

#[test]
fn silverquill_chronicle_drains_two_and_returns_is_card_from_graveyard() {
    let mut g = two_player_game();
    let bolt_gy = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::silverquill_chronicle());
    add_generous_mana(&mut g);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Chronicle castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 2, "gained 2 life from drain");
    assert_eq!(g.players[1].life, life1_before - 2, "opp lost 2 life from drain");
    // Hand: -1 (cast Chronicle) +1 (Bolt returned) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_gy), "Bolt back in hand");
}

#[test]
fn witherbloom_vinemaster_grows_on_pest_death() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let vm = g.add_card_to_battlefield(0, catalog::witherbloom_vinemaster());
    // Use a non-token Pest (Witherbloom Pest Eater) so the dying creature
    // stays in graveyard (not subject to token "ceases to exist" SBA).
    let pest = g.add_card_to_battlefield(0, catalog::witherbloom_pest_eater());
    drain_stack(&mut g);
    // Kill the Pest with two Bolts.
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(pest)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
    }
    let count = g.battlefield_find(vm).expect("VM on bf")
        .counter_count(CounterType::PlusOnePlusOne);
    assert!(count >= 1, "Vinemaster gained a +1/+1 counter on Pest death");
}

#[test]
fn lorehold_acolyte_etb_exiles_target_graveyard_card() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_acolyte());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Acolyte castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "Bolt in exile");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Bolt gone from graveyard");
}

#[test]
fn lorehold_warrior_priest_gains_life_on_attack() {
    use crabomination::game::types::{AttackTarget, Attack, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_warrior_priest());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Attackers declared");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "gained 1 life on attack");
}

#[test]
fn lorehold_skirmish_mints_a_spirit_with_haste_eot() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_skirmish());
    add_generous_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skirmish castable");
    drain_stack(&mut g);
    let spirit = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).expect("Spirit token minted");
    let view = g.computed_permanent(spirit.id).expect("Spirit on bf");
    assert!(view.keywords.contains(&Keyword::Haste),
        "Skirmish-minted Spirit has haste EOT");
}

#[test]
fn quandrix_summoner_etb_mints_one_one_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_summoner());
    add_generous_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Summoner castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Fractal"
    }).expect("Fractal token minted");
    let view = g.computed_permanent(fractal.id).expect("Fractal on bf");
    assert_eq!(view.power, 1, "Fractal 0 base + 1 counter = 1 power");
    assert_eq!(view.toughness, 1, "Fractal 0 base + 1 counter = 1 toughness");
}

#[test]
fn quandrix_ecologist_etb_self_pumps_with_counter() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_ecologist());
    add_generous_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ecologist castable");
    drain_stack(&mut g);
    let view = g.computed_permanent(id).expect("Ecologist on bf");
    assert_eq!(view.power, 5, "Ecologist 4 + 1 counter = 5 power");
    assert_eq!(view.toughness, 5, "Ecologist 4 + 1 counter = 5 toughness");
    assert!(view.keywords.contains(&Keyword::Trample));
}

#[test]
fn inkling_witness_gains_life_when_other_inkling_dies() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let _w = g.add_card_to_battlefield(0, catalog::inkling_witness());
    let other_ink = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    // Kill the Inkling Aspirant (2/1) with a Lightning Bolt.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(other_ink)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].life > life_before,
        "Witness gained at least 1 life from Inkling death (was {}, now {})",
        life_before, g.players[0].life);
}

#[test]
fn lorehold_loremaster_attack_mints_spirit_token() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let lm = g.add_card_to_battlefield(0, catalog::lorehold_loremaster());
    g.clear_sickness(lm);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lm,
        target: AttackTarget::Player(1),
    }])).expect("Loremaster attacks");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).collect();
    assert_eq!(spirits.len(), 1, "Loremaster mints 1 Spirit per attack");
}

#[test]
fn quandrix_reckoner_attack_adds_plus_one_counter() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let rk = g.add_card_to_battlefield(0, catalog::quandrix_reckoner());
    g.clear_sickness(rk);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rk,
        target: AttackTarget::Player(1),
    }])).expect("Reckoner attacks");
    drain_stack(&mut g);
    let view = g.battlefield_find(rk).expect("Reckoner present");
    let counters = view.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 1, "Reckoner gains a +1/+1 counter per attack");
    assert_eq!(view.power(), 3, "Reckoner is now 3/3");
}

#[test]
fn fractal_reinforcement_puts_counter_on_each_friendly_creature() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b_opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_reinforcement());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reinforcement castable");
    drain_stack(&mut g);
    let p1 = g.battlefield_find(b1).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    let p2 = g.battlefield_find(b2).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(p1, 1, "Bear 1 has +1/+1 counter");
    assert_eq!(p2, 1, "Bear 2 has +1/+1 counter");
}

// ── CR 115.5 self-target enforcement (engine improvement) ───────────────────

#[test]
fn cr_115_5_spell_targeting_itself_is_illegal_via_permanent_id() {
    // Cast a creature, then try Bury in Books (bounce target creature)
    // targeting the in-progress cast spell's own id. Bury in Books needs
    // a creature target, and we'll verify that re-using the bury card id
    // as its own target (Target::Permanent(card_id)) is rejected by
    // check_target_legality_with_source. The headline gameplay rule is
    // that a spell on the stack cannot target itself (CR 115.5).
    let mut g = two_player_game();
    let bury = g.add_card_to_hand(0, catalog::bury_in_books());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Casting Bury in Books targeting itself (its own card_id) is rejected:
    // the cast pipeline threads `Some(card_id)` to the target validator,
    // so the bury card cannot be its own bounce target.
    let result = g.perform_action(GameAction::CastSpell {
        card_id: bury,
        target: Some(crabomination::game::types::Target::Permanent(bury)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(result.is_err(), "Bury in Books targeting itself should be rejected (CR 115.5)");
}

#[test]
fn pest_swarm_inheritance_pumps_friendly_and_mints_pest() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::pest_swarm_inheritance());
    add_generous_mana(&mut g);
    let bear_p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Bequest castable");
    drain_stack(&mut g);
    let bear_p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    assert_eq!(bear_p_after, bear_p_before + 1, "Bear pumped +1/+1");
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Deathtouch),
        "Bear gained Deathtouch EOT");
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 1, "1 Pest token created");
}

#[test]
fn witherbloom_decayblossom_dies_shrinks_target() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let blossom = g.add_card_to_battlefield(0, catalog::witherbloom_decayblossom());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Kill our own Decayblossom with a Bolt.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(blossom)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Decayblossom's death trigger fires — auto-target picks the bear,
    // shrinking it to -1/-1 → 1/1 (from 2/2). Then SBA kills no-one;
    // it stays at 1/1 EOT.
    let bear_p = g.battlefield_find(opp_bear).map(|c| c.power()).unwrap_or(0);
    let bear_t = g.battlefield_find(opp_bear).map(|c| c.toughness()).unwrap_or(0);
    assert_eq!(bear_p, 1, "Bear shrunk to 1 power");
    assert_eq!(bear_t, 1, "Bear shrunk to 1 toughness");
}

#[test]
fn quandrix_fractalflow_mints_fractal_scaled_by_hand() {
    let mut g = two_player_game();
    // Seed the hand to 3 cards before the cast.
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalflow());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalflow castable");
    drain_stack(&mut g);
    // After cast: hand had 2 cards left (originals seeded above). The
    // Fractal token receives that many +1/+1 counters.
    let fractal = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Fractal"
    }).expect("1 Fractal minted");
    let counters = fractal.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 2, "Fractal scales counters to hand size");
}

#[test]
fn quandrix_multibinding_doubles_counters_after_adding() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_multibinding());
    add_generous_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Multibinding castable");
    drain_stack(&mut g);
    // Add 2 +1/+1, then double counts: 2 → 4 (the doubling adds 2 more).
    let counters = g.battlefield_find(bear).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(counters, 4, "Multibinding: 2 + (2 doubled = 2 more) = 4");
}

#[test]
fn quandrix_geomyst_and_reclamation_keyword_checks() {
    // Standalone def checks folded from consolidated tests.
    assert!(catalog::quandrix_geomyst().keywords.contains(&Keyword::Reach));
}

#[test]
fn lorehold_reclamation_returns_creature_to_battlefield() {
    let mut g = two_player_game();
    let _gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_reclamation());
    add_generous_mana(&mut g);
    let bf_before: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.name == "Grizzly Bears"
    }).collect();
    let bf_count_before = bf_before.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reclamation castable");
    drain_stack(&mut g);
    let bf_after: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.definition.name == "Grizzly Bears"
    }).collect();
    assert_eq!(bf_after.len(), bf_count_before + 1, "Bear returned to battlefield");
    // "It's a Spirit in addition to its other types." — a layer-4
    // additive grant, so read the computed (post-layers) types.
    let bear_id = bf_after[0].id;
    let computed = g.computed_permanent(bear_id).expect("computed bear");
    assert!(computed.subtypes.creature_types.contains(&crabomination::card::CreatureType::Spirit),
        "reanimated creature is a Spirit in addition to its other types");
    assert!(computed.subtypes.creature_types.contains(&crabomination::card::CreatureType::Bear),
        "printed creature type is kept (Spirit is additive)");
}

#[test]
fn pest_marauder_has_deathtouch_and_dies_grants_life() {
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::pest_marauder());
    // Hand-of-fate: kill the marauder via SBA by damage.
    let pm_card = g.battlefield_find_mut(pm).unwrap();
    pm_card.damage = 1; // 1/1 with 1 damage → SBA kills it
    let life_before = g.players[0].life;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(pm).is_none(), "Marauder died");
    assert_eq!(g.players[0].life, life_before + 1, "Marauder grants 1 life on death");
    let def = catalog::pest_marauder();
    assert!(def.keywords.contains(&Keyword::Deathtouch));
}

#[test]
fn silverquill_quillblade_pumps_by_creature_count() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Add 2 more creatures to make 3 total.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_quillblade());
    g.players[0].mana_pool.add(Color::White, 1);
    let p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Quillblade castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 3, "Bear pumped by 3 (3 creatures controlled)");
}

#[test]
fn pest_communion_mills_four_each_opp_and_drains_one() {
    let mut g = two_player_game();
    for _ in 0..10 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::pest_communion());
    add_generous_mana(&mut g);
    let opp_lib_before = g.players[1].library.len();
    let opp_gy_before = g.players[1].graveyard.len();
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Communion castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), opp_lib_before - 4);
    assert_eq!(g.players[1].graveyard.len(), opp_gy_before + 4);
    assert_eq!(g.players[0].life, life0_before + 1);
    assert_eq!(g.players[1].life, life1_before - 1);
}

#[test]
fn lorehold_recollect_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_recollect());
    add_generous_mana(&mut g);
    let bears_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recollect castable");
    drain_stack(&mut g);
    let bears_after = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears")
        .count();
    assert_eq!(bears_after, bears_before + 1, "Bear returned to battlefield");
}

#[test]
fn lorehold_anthemist_anthem_buffs_other_spirits() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_anthemist());
    // Mint a Spirit token via Lorehold Echoist's ETB.
    let echoist = g.add_card_to_hand(0, catalog::lorehold_echoist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: echoist, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echoist castable");
    drain_stack(&mut g);
    // Find the minted Spirit token id, then read its computed P/T via the layer system.
    let spirit_id = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Spirit"
    }).map(|c| c.id).expect("Spirit minted");
    let spirit_computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == spirit_id)
        .expect("Spirit on computed battlefield");
    // Lorehold Spirit token is 2/2; with +1/+1 anthem from Anthemist, should be 3/3.
    assert_eq!(spirit_computed.power, 3, "Spirit pumped to 3/3 by Anthemist");
    assert_eq!(spirit_computed.toughness, 3);
}

#[test]
fn fractal_growth_adds_counter_and_pumps_by_counter_count() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Growth castable");
    drain_stack(&mut g);
    // 0 prior counters → +1 counter (now 1) → +1/+1 EOT from 1 counter → 3/3 total (2 base + 1 counter)
    // Then PumpPT(+1/+1) → 4/4 EOT.
    // Actually: base 2/2, +1 counter → 3/3, then PumpPT +1/+1 → 4/4.
    let p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    assert_eq!(p_after, p_before + 2, "Bear +1 counter (+1) + EOT +1 = +2 power");
}

#[test]
fn quandrix_calculus_etb_mills_two_and_draws_one() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_calculus());
    add_generous_mana(&mut g);
    let lib_before = g.players[0].library.len();
    let gy_before = g.players[0].graveyard.len();
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Calculus castable");
    drain_stack(&mut g);
    // Mill 2 + Draw 1 = library -3, graveyard +2, hand: -1 (cast) +1 (draw) = 0
    assert_eq!(g.players[0].library.len(), lib_before - 3);
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_cantrip_deals_one_damage_and_cantrips() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_cantrip());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cantrip castable");
    drain_stack(&mut g);
    // -1 (cast) +1 (draw) = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Bear took 1 damage (now 2/2 with 1 damage marked).
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.damage, 1);
}

#[test]
fn prismari_flarespark_deals_two_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_flarespark());
    add_generous_mana(&mut g);
    let hand_before = g.players[0].hand.len();
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flarespark castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2);
    // -1 (cast) +1 (draw) = 0 net hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn silverquill_lawkeeper_etb_taps_opp_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_lawkeeper());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lawkeeper castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).expect("bear still on bf");
    assert!(bear_card.tapped, "Lawkeeper ETB taps opp creature");
    let def = catalog::silverquill_lawkeeper();
    assert!(def.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn silverquill_discipline_pumps_and_grants_lifelink() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_discipline());
    g.players[0].mana_pool.add(Color::White, 1);
    let p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    let t_before = g.battlefield_find(bear).map(|c| c.toughness()).unwrap_or(0);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Discipline castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
    let t_after = g.battlefield_find(bear).map(|c| c.toughness()).unwrap_or(0);
    assert_eq!(p_after, p_before + 2);
    assert_eq!(t_after, t_before + 1);
    let bear_card = g.battlefield_find(bear).expect("bear still on bf");
    assert!(bear_card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn lorehold_tomescholar_mints_spirit_when_exiling_creature_card() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_tomescholar());
    add_generous_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear_in_gy)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tomescholar castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 1, "Tomescholar mints Spirit when exiling creature card");
}

#[test]
fn lorehold_tomescholar_no_spirit_when_exiling_noncreature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bolt_in_gy = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_tomescholar());
    add_generous_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bolt_in_gy)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tomescholar castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 0, "No Spirit when exiling noncreature");
}

#[test]
fn lorehold_warband_pumps_by_other_attackers() {
    use crabomination::game::types::AttackTarget;
    let mut g = two_player_game();
    let wb = g.add_card_to_battlefield(0, catalog::lorehold_warband());
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Untap before declare attackers
    for cid in [wb, bear1, bear2] {
        g.clear_sickness(cid);
    }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: wb, target: AttackTarget::Player(1) },
        Attack { attacker: bear1, target: AttackTarget::Player(1) },
        Attack { attacker: bear2, target: AttackTarget::Player(1) },
    ])).expect("DeclareAttackers");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(wb).map(|c| c.power()).unwrap_or(0);
    // 3 base + 2 other attackers = 5
    assert_eq!(p_after, 5, "Warband pumped by 2 other attackers");
}

#[test]
fn fractal_bloom_mints_fractal_scaled_by_double_hand() {
    let mut g = two_player_game();
    // Set hand to exactly some count first, then cast.
    let id = g.add_card_to_hand(0, catalog::fractal_bloom());
    // Add 4 more cards to hand: total 5 in hand (1 will be cast)
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::island());
    }
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloom castable");
    drain_stack(&mut g);
    // After casting, hand = 4 islands. 2*4 = 8 +1/+1 counters.
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.is_token && c.definition.name == "Fractal")
        .expect("Fractal minted");
    let counters = fractal.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 8, "Fractal has 2*4=8 +1/+1 counters");
}

#[test]
fn quandrix_spellweaver_etb_draws_two_and_grows_on_cast() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_spellweaver());
    add_generous_mana(&mut g);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellweaver castable");
    drain_stack(&mut g);
    // -1 (cast) +2 (draw) = +1 net
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    // Cast another spell — Spellweaver should get +1/+1 counter
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let sw = g.battlefield.iter().find(|c| c.definition.name == "Quandrix Spellweaver").expect("Spellweaver");
    let counters = sw.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 1, "Spellweaver grew via magecraft");
}

#[test]
fn fractal_multiplier_doubles_counters_on_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Pre-load with 3 counters.
    if let Some(c) = g.battlefield_find_mut(bear) {
        c.add_counters(CounterType::PlusOnePlusOne, 3);
    }
    let id = g.add_card_to_hand(0, catalog::fractal_multiplier());
    add_generous_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Multiplier castable");
    drain_stack(&mut g);
    // 3 + 3 (doubled) = 6 counters.
    let counters = g.battlefield_find(bear).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(counters, 6, "Fractal Multiplier doubles 3 → 6");
}

#[test]
fn fractal_synthesis_adds_two_counters_and_draws() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_synthesis());
    add_generous_mana(&mut g);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Synthesis castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(bear).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(counters, 2);
    assert_eq!(g.players[0].hand.len(), hand_before); // -1 cast +1 draw = 0
}
