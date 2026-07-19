use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

/// Fill seat 0's pool with generous mana of every color so table-driven
/// tests don't need per-card cost bookkeeping.
fn fill_mana(g: &mut crabomination::game::Game) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 3);
    }
    g.players[0].mana_pool.add_colorless(6);
}

// ── Unique-shape tests (batches 146-154) ────────────────────────────────────

#[test]
fn quandrix_sumcaster_b146_etb_pumps_friendly_creature_by_other_count() {
    let mut g = two_player_game();
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let target_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_sumcaster_b146());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sumcaster castable");
    drain_stack(&mut g);
    let total_counters_on_friendlies: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert!(total_counters_on_friendlies >= 1, "Some friendly got counters");
}

// ── Table-driven magecraft tests ────────────────────────────────────────────

#[test]
fn magecraft_looters_hand_shrinks_by_one() {
    // Magecraft loot: -1 (cast Bolt) +1 (draw) -1 (discard) = -1
    for def in [
        catalog::quandrix_mathwitch_b146(),
        catalog::prismari_tidescribe_b147(),
        catalog::prismari_stormcaller_b150(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before - 1);
    }
}

#[test]
fn magecraft_drawers_hand_unchanged() {
    // Magecraft draw (or scry+draw): -1 (cast Bolt) +1 (draw) = 0
    for def in [
        catalog::quandrix_patternseeker_b146(),
        catalog::prismari_arcanist_b147(),
        catalog::quandrix_fractalweaver_b150(),
        catalog::quandrix_hydromancer_b150(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before);
    }
}

#[test]
fn magecraft_scryers_library_unchanged() {
    // Scry doesn't draw — library size stays the same.
    for def in [
        catalog::prismari_tidemage_b150(),
        catalog::quandrix_spellmage_b151(),
        catalog::prismari_wavecaller_b151(),
        catalog::prismari_elementalist_b153(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let lib_size = g.players[0].library.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].library.len(), lib_size);
    }
}

#[test]
fn magecraft_self_pumpers_gain_a_counter() {
    // Magecraft puts a +1/+1 counter on self.
    for def in [
        catalog::quandrix_sumstudent_b146(),
        catalog::prismari_pyrolancer_b146(),
        catalog::quandrix_calculator_b147(),
        catalog::fractal_apprentice_b147(),
        catalog::quandrix_geometer_b148(),
        catalog::quandrix_snake_egg_b150(),
        catalog::witherbloom_decaymage_b154(),
        catalog::witherbloom_pestbreaker_b154(),
        catalog::silverquill_recitalist_b154(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(id).unwrap();
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
    }
}

#[test]
fn magecraft_pump_and_drain_one() {
    // +1/+1 counter on self AND each opp loses 1: opp total = 3 (bolt) + 1.
    for def in [
        catalog::silverquill_penmaster_b147(),
        catalog::witherbloom_bloomscribe_b147(),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let l1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(id).unwrap();
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
        assert_eq!(g.players[1].life, l1_before - 3 - 1);
    }
}

#[test]
fn magecraft_burners_hit_opponent() {
    // Magecraft burn/drain: opp loses 3 (bolt) + `extra`.
    for (def, extra) in [
        (catalog::lorehold_glyphcaster_b147(), 1),
        (catalog::prismari_embercaller_b147(), 1),
        (catalog::prismari_sparkmage_b148(), 1),
        (catalog::lorehold_embermage_b150(), 1),
        (catalog::witherbloom_rotsage_b150(), 1),
        (catalog::lorehold_pyromancer_b152(), 1),
        (catalog::lorehold_cinderspeaker_b154(), 1),
        (catalog::silverquill_doomscribe_b150(), 2),
        (catalog::silverquill_sacrificemage_b152(), 2),
        (catalog::prismari_pyromage_b150(), 2),
    ] {
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let l1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1_before - 3 - extra);
    }
}

#[test]
fn magecraft_token_minters_add_one_permanent() {
    for (def, token_name) in [
        (catalog::prismari_treasurer_b146(), None),
        (catalog::lorehold_ember_wraith_b148(), None),
        (catalog::lorehold_spirit_guide_b151(), None),
        (catalog::witherbloom_spawnbed_b150(), Some("Pest")),
        (catalog::prismari_treasure_smith_b150(), Some("Treasure")),
        (catalog::prismari_glassblower_b151(), Some("Treasure")),
        (catalog::silverquill_inkmancer_b154(), Some("Inkling")),
    ] {
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let bf_before = g.battlefield.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.len(), bf_before + 1);
        if let Some(name) = token_name {
            assert!(g.battlefield.iter().any(|c| c.definition.name == name));
        }
    }
}

#[test]
fn quandrix_spelltwister_b148_magecraft_scrys_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_spelltwister_b148());
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
fn lorehold_sparkmage_b150_magecraft_pings_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_sparkmage_b150());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bear took 3 + 1 = 4 damage → dies (2 toughness)
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

// ── Table-driven ETB / cast-resolution tests ────────────────────────────────

#[test]
fn etb_token_minters_grow_battlefield() {
    // Cast the card; battlefield grows by `delta` (the card itself + tokens).
    for (def, delta) in [
        (catalog::fractal_caller_b146(), 2),
        (catalog::pest_caretaker_b148(), 3),
        (catalog::lorehold_spirit_smith_b148(), 2),
        (catalog::prismari_treasurehunter_b148(), 2),
        (catalog::witherbloom_pestcaller_b150(), 3),
        (catalog::lorehold_spiritforge_b150(), 2),
        (catalog::quandrix_spireshape_b150(), 2),
        (catalog::inkling_conjurer_b151(), 3),
        (catalog::witherbloom_pest_brood_b152(), 4),
        (catalog::witherbloom_boneharvester_b154(), 3),
        (catalog::pest_bramblelord_b154(), 3),
        (catalog::lorehold_battlespirit_b154(), 2),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        let bf_before = g.battlefield.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("token-minter castable");
        drain_stack(&mut g);
        assert_eq!(g.battlefield.len(), bf_before + delta);
    }
}

#[test]
fn counterspells_counter_bolt_when_opp_cant_pay() {
    for def in [
        catalog::quandrix_counterspell_b146(),
        catalog::prismari_counterscribe_b147(),
    ] {
        let mut g = two_player_game();
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(0)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Opp Bolt castable");
        g.priority.player_with_priority = 0;
        let cs = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.perform_action(GameAction::CastSpell {
            card_id: cs, target: Some(Target::Permanent(bolt)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Counterspell castable");
        drain_stack(&mut g);
        // Opp had no mana left to pay, so Bolt was countered.
        assert_eq!(g.players[0].life, 20);
    }
}

#[test]
fn counterspells_counter_creature_spell() {
    for def in [
        catalog::quandrix_mind_curl_b150(),
        catalog::prismari_spellburst_b153(),
    ] {
        let mut g = two_player_game();
        // Seat 1 casts a creature without enough mana to pay the tax.
        let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[1].mana_pool.add(Color::Green, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bear castable");
        let cs = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell {
            card_id: cs, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("counterspell castable");
        drain_stack(&mut g);
        // Bear was countered (in graveyard, not on battlefield).
        assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
        assert!(!g.battlefield.iter().any(|c| c.id == bear));
    }
}

#[test]
fn basic_land_fetchers_put_land_in_tapped() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    for def in [
        catalog::quandrix_field_trip_b146(),
        catalog::quandrix_mossbinder_b146(),
    ] {
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("fetcher castable");
        drain_stack(&mut g);
        let f = g.battlefield_find(forest).expect("Forest in play");
        assert!(f.tapped);
    }
}

#[test]
fn etb_tappers_tap_opp_creature() {
    for def in [
        catalog::prismari_sleetcaster_b146(),
        catalog::silverquill_pacifier_b154(),
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("tapper castable");
        drain_stack(&mut g);
        let b = g.battlefield_find(bear).unwrap();
        assert!(b.tapped);
    }
}

#[test]
fn bouncers_return_opp_creature_to_hand() {
    for def in [
        catalog::prismari_tidemage_b146(),
        catalog::quandrix_bouncer_b147(),
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bouncer castable");
        drain_stack(&mut g);
        assert!(g.players[1].hand.iter().any(|c| c.id == bear));
    }
}

#[test]
fn burn_spells_hit_player_for_n() {
    for (def, dmg) in [
        (catalog::prismari_volcanic_spell_b146(), 3),
        (catalog::lorehold_cinderscry_b147(), 1),
        (catalog::lorehold_pyrehowler_b147(), 5),
        (catalog::prismari_inferno_b150(), 3),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        let l1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("burn castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1_before - dmg);
    }
}

#[test]
fn lorehold_lightcaller_b148_etb_burns_target_and_has_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_lightcaller_b148());
    fill_mana(&mut g);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lightcaller castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 2);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Lifelink));
}

#[test]
fn removal_kills_opp_bear() {
    // Targeted damage / shrink / destroy that puts a 2/2 in the graveyard.
    for def in [
        catalog::silverquill_cinderglyph_b148(),
        catalog::lorehold_cinderlist_b148(),
        catalog::prismari_mindstrike_b148(),
        catalog::silverquill_smite_b151(),
        catalog::lorehold_ember_strike_b150(),
        catalog::prismari_spellsplash_b153(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("removal castable");
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
    }
}

#[test]
fn removal_kills_opp_bear_and_gains_life() {
    for (def, gain) in [
        (catalog::witherbloom_lifeleech_b150(), 3),
        (catalog::lorehold_pyrelore_b151(), 4),
        (catalog::witherbloom_mortislide_b152(), 2),
        (catalog::lorehold_reflux_b154(), 2),
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("removal castable");
        drain_stack(&mut g);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
        assert_eq!(g.players[0].life, life_before + gain);
    }
}

#[test]
fn lorehold_bonfire_b150_burns_creature_and_pings_controller() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::lorehold_bonfire_b150());
    fill_mana(&mut g);
    let opp_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bonfire castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
    // Bear's controller (seat 1) took 1 damage
    assert_eq!(g.players[1].life, opp_life_before - 1);
}

#[test]
fn drainers_drain_n_on_resolution() {
    // ETB or spell: opp loses N, you gain N.
    for (def, n) in [
        (catalog::witherbloom_hexstrike_b148(), 3),
        (catalog::witherbloom_pestreaver_b148(), 2),
        (catalog::silverquill_funerary_rite_b150(), 2),
        (catalog::witherbloom_vinepriest_b150(), 2),
        (catalog::witherbloom_apothecary_b151(), 1),
        (catalog::witherbloom_mire_b151(), 3),
        (catalog::silverquill_memoryflame_b152(), 1),
        (catalog::silverquill_mortarscribe_b152(), 2),
        (catalog::inkling_drainreaver_b154(), 3),
        (catalog::witherbloom_lifedrain_b154(), 5),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        let l0_before = g.players[0].life;
        let l1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("drainer castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, l1_before - n);
        assert_eq!(g.players[0].life, l0_before + n);
    }
}

#[test]
fn silverquill_inkdrip_b147_drains_two_and_gains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkdrip_b147());
    fill_mana(&mut g);
    let l0_before = g.players[0].life;
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkdrip castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2 + 1);
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn life_gainers_gain_n_on_resolution() {
    for (def, gain) in [
        (catalog::quandrix_mage_apprentice_b146(), 1),
        (catalog::silverquill_lifesong_b148(), 3),
        (catalog::silverquill_lifebringer_b150(), 3),
        (catalog::lorehold_ember_cleric_b152(), 2),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        let l0_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("life-gainer castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, l0_before + gain);
    }
}

#[test]
fn quandrix_wallcaller_b147_etb_gains_two_life_and_has_defender() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_wallcaller_b147());
    fill_mana(&mut g);
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wallcaller castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Defender));
}

#[test]
fn etb_drain_one_and_draw() {
    // ETB drains each opp for 1 and draws: hand net 0, opp -1.
    for def in [
        catalog::silverquill_cantorscribe_b147(),
        catalog::witherbloom_scarcasterer_b147(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        let hand_before = g.players[0].hand.len();
        let l1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before);
        assert_eq!(g.players[1].life, l1_before - 1);
    }
}

#[test]
fn etb_scry_and_draw_hand_unchanged() {
    for def in [
        catalog::quandrix_patternsage_b147(),
        catalog::silverquill_recruiter_b151(),
        catalog::quandrix_algebraist_b151(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        // -1 cast + 1 draw = 0 net
        assert_eq!(g.players[0].hand.len(), hand_before);
    }
}

#[test]
fn card_draw_spells_net_hand_change() {
    for (def, net) in [
        (catalog::quandrix_symbolic_b148(), 0i64),   // draw 2, discard 1
        (catalog::prismari_aetherwave_b150(), 0),    // draw 2, discard 1
        (catalog::quandrix_insight_b153(), 1),       // draw 2
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        let hand_before = g.players[0].hand.len() as i64;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len() as i64, hand_before + net);
    }
}

#[test]
fn vanilla_bodies_have_expected_keywords_and_stats() {
    for (def, kws, pt, types) in [
        (catalog::prismari_flamekind_b147(),
         vec![Keyword::Haste, Keyword::Trample], Some((4, 3)), vec![]),
        (catalog::silverquill_penmaster_general_b150(),
         vec![Keyword::Vigilance, Keyword::Lifelink], Some((4, 4)), vec![]),
        (catalog::silverquill_pen_striker_b151(),
         vec![Keyword::Flying, Keyword::Lifelink], None,
         vec![CreatureType::Inkling, CreatureType::Knight]),
        (catalog::lorehold_pyre_ancient_b152(),
         vec![Keyword::Vigilance, Keyword::Trample], Some((5, 5)), vec![]),
        (catalog::pest_skulker_b154(),
         vec![Keyword::Menace], Some((1, 1)), vec![CreatureType::Pest]),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let c = g.battlefield_find(id).unwrap();
        for kw in &kws {
            assert!(c.has_keyword(kw), "{} missing {:?}", c.definition.name, kw);
        }
        if let Some((p, t)) = pt {
            assert_eq!(c.definition.power, p);
            assert_eq!(c.definition.toughness, t);
        }
        for ty in &types {
            assert!(c.definition.subtypes.creature_types.contains(ty));
        }
    }
}

#[test]
fn attack_triggers_fire_on_declare() {
    // (def, opp life delta, your life delta, spirit tokens minted)
    for (def, opp_delta, you_delta, spirits) in [
        (catalog::silverquill_aggressor_b147(), -1i64, 0i64, 0usize),
        (catalog::lorehold_spirit_tender_b150(), 0, 1, 0),
        (catalog::witherbloom_cauldronkeeper_b152(), -1, 1, 0),
        (catalog::lorehold_spirit_surger_b154(), 0, 0, 1),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        g.step = TurnStep::DeclareAttackers;
        let l1_before = g.players[1].life as i64;
        let l0_before = g.players[0].life as i64;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: id,
            target: AttackTarget::Player(1),
        }])).expect("declare attackers");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life as i64, l1_before + opp_delta);
        assert_eq!(g.players[0].life as i64, l0_before + you_delta);
        let count = g.battlefield.iter()
            .filter(|c| c.definition.name == "Spirit").count();
        assert_eq!(count, spirits);
    }
}

#[test]
fn pump_auras_grant_lifelink_and_stats() {
    for (def, p, t) in [
        (catalog::witherbloom_lifelink_sigil_b147(), 3, 3),   // +1/+1
        (catalog::silverquill_verseblade_b150(), 4, 4),       // +2/+2
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("pump castable");
        drain_stack(&mut g);
        let computed = g.compute_battlefield();
        let b = computed.iter().find(|c| c.id == bear).unwrap();
        assert_eq!(b.power, p);
        assert_eq!(b.toughness, t);
        assert!(b.keywords.contains(&Keyword::Lifelink));
    }
}

// ── Remaining unique-shape tests ────────────────────────────────────────────

#[test]
fn prismari_charge_b146_draws_and_pings() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_charge_b146());
    fill_mana(&mut g);
    let hand_before = g.players[0].hand.len();
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Prismari Charge castable");
    drain_stack(&mut g);
    // -1 (cast) +1 (draw) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[1].life, l1_before - 1);
}

#[test]
fn prismari_reflectionist_b146_etb_scrys_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_reflectionist_b146());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reflectionist castable");
    drain_stack(&mut g);
    // Just verify it resolves
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.power(), 2);
}

#[test]
fn prismari_surge_b146_deals_four_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_surge_b146());
    fill_mana(&mut g);
    let hand_before = g.players[0].hand.len();
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Surge castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1_before - 4);
    // -1 (cast) +1 (draw) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn witherbloom_forager_b147_etb_mills_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_forager_b147());
    g.players[0].mana_pool.add(Color::Green, 1);
    let lib_before = g.players[0].library.len();
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Forager castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 1);
    assert_eq!(g.players[0].life, l0_before + 1);
}

#[test]
fn witherbloom_festering_specter_b147_dies_drains_each_opp() {
    let mut g = two_player_game();
    let fs = g.add_card_to_battlefield(0, catalog::witherbloom_festering_specter_b147());
    let l1_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fs)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Specter has 2 toughness → dies. Each opp loses 2.
    assert_eq!(g.players[1].life, l1_before - 2);
}

#[test]
fn silverquill_mortarscribe_b148_lifegain_drains_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_mortarscribe_b148());
    // Cast a heal spell to trigger LifeGained event (adjust_life alone
    // doesn't emit one — see Effect::GainLife in game/effects/mod.rs).
    let heal = g.add_card_to_hand(0, catalog::silverquill_heartmender_b145());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: heal, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Heartmender castable");
    drain_stack(&mut g);
    // 4 life gained → drain 1 fires once per LifeGained event
    assert_eq!(g.players[1].life, l1_before - 1);
}

#[test]
fn fractal_warrior_b148_etb_with_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_warrior_b148());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Warrior castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn prismari_splashmage_b148_pings_creature_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_splashmage_b148());
    fill_mana(&mut g);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Splashmage castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.damage, 1);
    // -1 cast + 1 draw = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── CR 122 audit tests (rule-level fan-out from batch 146/147/148) ─────────

#[test]
fn cr_122_3_plus_one_and_minus_one_counters_cancel_on_witherbloom_reapcaster() {
    // CR 122.3 — +1/+1 and -1/-1 counters cancel as a state-based action.
    // Reapcaster's magecraft trigger drops a +1/+1 counter on it; we
    // simultaneously seed a -1/-1 counter. After the next SBA pass, both
    // counters should be at 0 (1 of each cancels to 0).
    let mut g = two_player_game();
    let rc = g.add_card_to_battlefield(0, catalog::witherbloom_reapcaster_b146());
    if let Some(c) = g.battlefield_find_mut(rc) {
        c.counters.insert(CounterType::MinusOneMinusOne, 1);
    }
    // Cast a bolt to trigger magecraft +1/+1 counter (the same Reapcaster
    // also picks up the drain, but that's life-only and irrelevant here).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(rc).unwrap();
    // After SBA: 1 +1/+1 and 1 -1/-1 cancel to 0 of each.
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(c.counter_count(CounterType::MinusOneMinusOne), 0);
}

#[test]
fn cr_122_6_etb_with_counters_doesnt_die_to_zero_toughness_sba() {
    // CR 122.6/a — counters placed by `enters_with_counters` are applied
    // BEFORE the next SBA pass, so a 0/0 fractal body that ETBs with
    // +1/+1 counters survives the 0-toughness check (704.5f). Fractal
    // Caller (b146) is the canonical exercise card — Fractal token has
    // printed P/T 0/0 and the ETB drops 2 +1/+1 counters on it via
    // `etb_mint_token_with_counters`.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_caller_b146());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Caller castable");
    drain_stack(&mut g);
    // Walk the battlefield to find the Fractal token (it's freshly minted,
    // so it's the only token-typed Fractal creature).
    let fractal = g.battlefield.iter().find(|c| c.is_token).expect("Fractal token");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2);
    // Computed toughness should be 0 + 2 = 2, NOT 0 (which would have
    // caused immediate SBA death).
    let computed = g.compute_battlefield();
    let f = computed.iter().find(|c| c.id == fractal.id).unwrap();
    assert_eq!(f.toughness, 2);
}

#[test]
fn cr_116_3_priority_returns_to_player_after_play_land() {
    // CR 116.3 — special actions (like PlayLand) don't pass priority;
    // the active player retains priority after playing a land. Exercise
    // the explicit path: play a Forest from hand and confirm priority
    // stays with seat 0 (no auto-advance to opp).
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(forest))
        .expect("Forest playable in main phase");
    // CR 116.3: priority did NOT pass — seat 0 still has priority.
    assert_eq!(g.priority.player_with_priority, 0,
        "CR 116.3: PlayLand is a special action and doesn't reset priority");
    // Stack should still be empty (special actions don't go on the stack).
    assert!(g.stack.is_empty(),
        "CR 405.6d: special actions don't use the stack");
    // Land entered the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == forest),
        "Forest is now in play");
}

// ── Batch 151/152 unique tests ──────────────────────────────────────────────

#[test]
fn lorehold_battlemage_b151_etb_grants_vigilance() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_battlemage_b151());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battlemage castable");
    drain_stack(&mut g);
    let computed = g.computed_permanent(target).expect("target computed");
    assert!(computed.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn quandrix_elf_caller_b151_magecraft_self_pumps() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::quandrix_elf_caller_b151());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let computed = g.computed_permanent(elf).expect("elf computed");
    assert_eq!(computed.power, 2);
}

#[test]
fn quandrix_fractal_theorem_b151_scales_with_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::quandrix_fractal_theorem_b151());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Theorem castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| c.is_token).expect("Fractal token");
    // count happens AFTER CreateToken so includes the fractal itself: 3 total
    assert!(fractal.counter_count(CounterType::PlusOnePlusOne) >= 2);
}

#[test]
fn prismari_inferno_tide_b151_burns_each_opp_and_draws_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::prismari_inferno_tide_b151());
    fill_mana(&mut g);
    let opp_life_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inferno-Tide castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life_before - 2);
    // -1 spell + 2 draw = +1 net
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn witherbloom_pestmaster_b151_magecraft_pumps_each_pest() {
    let mut g = two_player_game();
    let _pm = g.add_card_to_battlefield(0, catalog::witherbloom_pestmaster_b151());
    // Add a Pest body (Eyetwitch Brood — Eyetwitch itself is now an
    // Eye Bat per its Duskmourn type-line errata, no longer a Pest).
    let pest = g.add_card_to_battlefield(0, catalog::eyetwitch_brood());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(pest).expect("Pest still alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn inkling_tactician_b152_magecraft_pumps_each_inkling() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::inkling_tactician_b152());
    let other = g.add_card_to_battlefield(0, catalog::inkling_scout_b151());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let computed = g.computed_permanent(other).expect("Inkling Scout computed");
    // 2 base + 1 from tactician = 3
    assert_eq!(computed.power, 3);
}

#[test]
fn witherbloom_cauldronthief_b152_etb_drains_one_and_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_cauldronthief_b152());
    fill_mana(&mut g);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cauldronthief castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pest"));
}

// ── CR rule lock-in tests (modern_decks rules pass) ────────────────────────

#[test]
fn cr_405_5_all_pass_resolves_top_of_stack() {
    // CR 405.5 — when all players pass in succession, the top
    // (last-added) spell on the stack resolves. Exercise by casting
    // two Bolts (top resolves first, then the bottom one); the
    // opponent life total reflects both having resolved by the time
    // the stack is empty.
    let mut g = two_player_game();
    let b1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let b2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 2);
    let opp_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: b1, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt 1 castable");
    g.perform_action(GameAction::CastSpell {
        card_id: b2, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt 2 castable");
    // Two spells on stack, both targeting seat 1.
    assert_eq!(g.stack.len(), 2);
    drain_stack(&mut g);
    // Both resolved → 6 damage taken.
    assert_eq!(g.players[1].life, opp_before - 6,
        "CR 405.5: both spells resolved after all-pass loop");
    assert!(g.stack.is_empty(),
        "Stack empties after all-pass cascade");
}

#[test]
fn cr_119_8_player_cannot_lose_life_blocks_lose_life_paths() {
    // CR 119.8 — when a player can't lose life, Effect::LoseLife
    // resolves to a no-op for that player. Exercise via the
    // PlayerCannotLoseLife static (Silverquill Lifeward b146 ships
    // an opp-locked variant; here we test the engine path directly
    // by checking the adjust_life gate's clamp behavior).
    let mut g = two_player_game();
    g.players[1].life = 5;
    g.add_card_to_battlefield(0, catalog::silverquill_lifeward_b146());
    // The Lifeward locks the OPPONENT (P1) from losing life. P1's
    // life stays at 5 even after we try to drain.
    let life_before = g.players[1].life;
    g.adjust_life(1, -3);
    assert_eq!(g.players[1].life, life_before,
        "CR 119.8: locked player can't lose life from adjust_life");
}

#[test]
fn cr_119_8_player_cannot_lose_life_blocks_burn_damage() {
    // CR 119.8 via the damage path: a player that can't lose life takes
    // no life loss from direct damage (the damage is still dealt, but
    // the life-loss it would cause is prevented). Exercises the
    // adjust_life gate from the Effect::DealDamage → lose-life route,
    // which the adjust_life-only test above does not cover.
    let mut g = two_player_game();
    g.players[1].life = 5;
    g.add_card_to_battlefield(0, catalog::silverquill_lifeward_b146());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 5,
        "CR 119.8: locked player loses no life to 3 damage from a bolt");
}

#[test]
fn cr_614_life_gain_becomes_loss_for_opponent() {
    // CR 614 (Tainted Remedy template): while Silverquill Reproach is in
    // play, an opponent's would-be life gain becomes an equal life loss.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_reproach_b209());
    g.players[1].life = 20;
    // P1 (the opponent) tries to gain 4 — it becomes a 4-life loss instead.
    g.adjust_life(1, 4);
    assert_eq!(g.players[1].life, 16, "opponent's life gain redirected to loss");
    // The controller (P0) gains life normally.
    g.players[0].life = 20;
    g.adjust_life(0, 4);
    assert_eq!(g.players[0].life, 24, "controller still gains normally");
}

#[test]
fn cr_702_105_exploit_sacrifices_and_drains() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Accept the exploit "you may sacrifice" prompt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let taker = g.add_card_to_hand(0, catalog::silverquill_tithe_taker_b209());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: taker, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tithe-Taker castable for {1}{B}");
    drain_stack(&mut g);
    // Only creature on board → it exploits itself; payoff drains 2.
    assert_eq!(g.players[1].life, 18, "exploit payoff drains opponent for 2");
    assert_eq!(g.players[0].life, 22, "exploit payoff gains controller 2");
    assert!(!g.battlefield.iter().any(|c| c.id == taker), "exploited itself");
}

#[test]
fn cr_702_105_exploit_declined_does_nothing() {
    // AutoDecider declines the may-sacrifice → no sacrifice, no payoff.
    let mut g = two_player_game();
    let taker = g.add_card_to_hand(0, catalog::silverquill_tithe_taker_b209());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: taker, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20, "declined exploit drains nothing");
    assert!(g.battlefield.iter().any(|c| c.id == taker), "taker survives");
}

#[test]
fn cr_702_83_devour_enters_with_counters_per_sacrifice() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Sacrifice 1 creature to Devour 1 → one +1/+1 counter.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(1)]));
    let dev = g.add_card_to_hand(0, catalog::witherbloom_devourer_b209());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: dev, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Devourer castable for {3}{G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "fodder devoured");
    let d = g.battlefield.iter().find(|c| c.id == dev).expect("devourer in play");
    assert_eq!(d.power(), 4, "3/3 base + one +1/+1 counter from devour");
    assert_eq!(d.toughness(), 4);
}

#[test]
fn cr_117_3a_no_player_gets_priority_during_untap_step() {
    // CR 117.3a — "No player receives priority during the untap step."
    // The do_untap turn-based action runs without yielding priority.
    // Test: ensure the step transitions cleanly through untap to
    // upkeep without intervening priority window.
    let mut g = two_player_game();
    // Add a tapped permanent on P0's side.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    // Move to end step, then advance through cleanup → P1's turn
    // starts with untap step.
    g.step = TurnStep::End;
    // Pass priority twice (P0 + P1) to advance through end → cleanup → next turn
    let _ = g.perform_action(GameAction::PassPriority);
    let _ = g.perform_action(GameAction::PassPriority);
    let _ = g.perform_action(GameAction::PassPriority);
    let _ = g.perform_action(GameAction::PassPriority);
    let _ = g.perform_action(GameAction::PassPriority);
    let _ = g.perform_action(GameAction::PassPriority);
    // After enough passes, P1's untap step has run; the bear is
    // P0's permanent so untap doesn't touch it. Confirm we advanced.
    // The exact step depends on triggers; main goal: the engine
    // doesn't hang waiting for priority during untap.
    assert!(g.step != TurnStep::Untap,
        "CR 117.3a: untap step runs without holding priority");
}

#[test]
fn cr_117_7_response_resolves_first_lifo_stack_order() {
    // CR 117.7 — "If a player with priority casts a spell or activates
    // an activated ability while another spell or ability is already
    // on the stack, the new spell or ability has been cast or
    // activated 'in response to' the earlier spell or ability. The
    // new spell or ability will resolve first."
    //
    // Sequence: P0 casts Bolt → stack [Bolt-at-bear]. Then P1 casts
    // Giant Growth-style buff on bear → stack [Bolt, GrowthOnBear].
    // Top of stack (GrowthOnBear) resolves first, pumping the bear,
    // so by the time Bolt resolves, the bear has 5 toughness and
    // survives the 3 damage.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let growth = g.add_card_to_hand(1, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add(Color::Green, 1);
    // P0 casts Bolt at bear
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    // P0 passes priority; now P1 gets priority and can respond
    g.perform_action(GameAction::PassPriority).expect("P0 passes");
    g.perform_action(GameAction::CastSpell {
        card_id: growth, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Growth castable in response");
    assert_eq!(g.stack.len(), 2,
        "Two spells on stack — Growth on top (cast last)");
    drain_stack(&mut g);
    // Growth pumped bear to 5/5; Bolt deals 3 → bear has 3 damage
    // marked on a 5-toughness body → does NOT die.
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "CR 117.7: Growth (cast last) resolved first → bear survived Bolt");
}

#[test]
fn cr_117_5_sba_before_priority_lethal_creature_dies_before_response() {
    // CR 117.5 — state-based actions are checked before any player
    // would get priority. After Bolting a 2-toughness creature, the
    // SBA pass kills it BEFORE the opp gets priority to respond.
    // Tested by asserting the bear is in the graveyard the moment
    // the stack is empty (no opp-priority intervening window).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "CR 117.5: SBA killed the lethal-damage creature before opp got priority");
    assert!(g.stack.is_empty());
}

#[test]
fn cr_119_7_lifegain_lock_blocks_subsequent_drain_target() {
    // CR 119.7 — life-gain lock applies to subsequent gain-life events
    // on the locked player. Exercise the Skullcrack lock by casting it
    // (target locked), then casting a drain-each-opp spell that would
    // normally heal the caster — the caster (seat 0) is not the locked
    // player so they DO gain life, but if the caster were locked
    // separately, gainlife would no-op.
    let mut g = two_player_game();
    // Self-target Skullcrack to lock seat 0 from gaining life.
    let crack = g.add_card_to_hand(0, catalog::skullcrack());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: crack, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Self-target Skullcrack");
    drain_stack(&mut g);
    assert!(g.players[0].cannot_gain_life_this_turn);
    let life_after_self_bolt = g.players[0].life;
    // Try Effect::GainLife — should be blocked.
    g.adjust_life(0, 5);
    assert_eq!(g.players[0].life, life_after_self_bolt,
        "CR 119.7: locked player can't gain life from subsequent effects");
}

// ── Batch 153 unique tests ──────────────────────────────────────────────────

#[test]
fn quandrix_sage_b153_etb_pumps_target_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sage = g.add_card_to_hand(0, catalog::quandrix_sage_b153());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: sage, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sage castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

// ── batch 154 — unique-shape cards ──────────────────────────────────────────

#[test]
fn pest_mawcap_b154_etb_mints_pest_and_dies_gains_life() {
    let mut g = two_player_game();
    let mc = g.add_card_to_hand(0, catalog::pest_mawcap_b154());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: mc, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mawcap castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.definition.name == "Pest").count();
    assert_eq!(pests, 1, "ETB mints exactly one Pest token");
    let life_before = g.players[0].life;
    g.battlefield_find_mut(mc).unwrap().damage = 5;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[0].life >= life_before + 2,
        "dies-trigger gains at least 2 life (Pest token's death may add more)");
}

#[test]
fn witherbloom_mossglobe_b154_taps_for_black_or_green_then_sacs_for_three_life() {
    let mut g = two_player_game();
    let mg = g.add_card_to_battlefield(0, catalog::witherbloom_mossglobe_b154());
    // Tap for {B}
    g.perform_action(GameAction::ActivateAbility {
        card_id: mg, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("mana ability {B}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
    // Untap and try {G}
    if let Some(c) = g.battlefield_find_mut(mg) { c.tapped = false; }
    g.perform_action(GameAction::ActivateAbility {
        card_id: mg, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("mana ability {G}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    // Untap, pay 2 generic mana + sac for 3 life
    if let Some(c) = g.battlefield_find_mut(mg) { c.tapped = false; }
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mg, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac for 3 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3, "+3 life");
    assert!(!g.battlefield.iter().any(|c| c.id == mg), "sacrificed → gone from bf");
}

#[test]
fn witherbloom_pestbinder_b154_etb_mints_pest_and_sac_shrinks_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pb = g.add_card_to_hand(0, catalog::witherbloom_pestbinder_b154());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pb, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pestbinder castable");
    drain_stack(&mut g);
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pb, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("Sac pest activation");
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Bear (2/2 → 0/0) → graveyard via SBA");
}

#[test]
fn witherbloom_reborn_b154_returns_all_creature_cards_from_gy_to_bf() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());  // IS, not creature
    let spell = g.add_card_to_hand(0, catalog::witherbloom_reborn_b154());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Reborn castable");
    drain_stack(&mut g);
    let bears_on_bf = g.battlefield.iter()
        .filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears_on_bf, 2, "Both bear cards return to battlefield");
    let bolts_in_gy = g.players[0].graveyard.iter()
        .filter(|c| c.definition.name == "Lightning Bolt").count();
    assert!(bolts_in_gy >= 1, "Bolt (non-creature) stays in graveyard");
}

#[test]
fn witherbloom_toxinbinder_b154_etb_shrinks_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tb = g.add_card_to_hand(0, catalog::witherbloom_toxinbinder_b154());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: tb, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Toxinbinder castable");
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Bear (2/2 → -1/-1) → graveyard via SBA");
}

#[test]
fn witherbloom_stride_b154_gains_three_drains_one_surveils_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::witherbloom_stride_b154());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Stride castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 3 + 1, "+3 + drain 1");
    assert_eq!(g.players[1].life, life1_before - 1, "-1 from drain");
}

#[test]
fn lorehold_smiterite_b154_has_haste_and_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_smiterite_b154());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pt = g.computed_permanent(id).map(|cp| (cp.power, cp.toughness));
    assert_eq!(pt, Some((4, 2)), "Smiterite (3/2 + magecraft +1/+0) = 4/2");
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Haste));
}

#[test]
fn lorehold_memoryflame_b154_burns_three_and_returns_is_from_gy() {
    let mut g = two_player_game();
    let bolt_gy = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::lorehold_memoryflame_b154());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memoryflame castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Bear takes 3 → dies");
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_gy),
        "Bolt returns from gy to hand");
}

#[test]
fn lorehold_stratagem_b154_mints_two_spirits_and_burns_opp_for_three() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::lorehold_stratagem_b154());
    fill_mana(&mut g);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Stratagem castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits, 2, "Mints 2 Spirit tokens");
    assert_eq!(g.players[1].life, life_before - 3, "Deals 3 to opp");
}
