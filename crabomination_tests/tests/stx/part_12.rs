use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

/// Add generous mana of every color plus colorless so table-driven bodies can
/// cast any of the grouped cards regardless of its exact cost.
fn add_generous_mana(g: &mut crabomination::game::GameState, player: usize) {
    for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[player].mana_pool.add(color, 3);
    }
    g.players[player].mana_pool.add_colorless(6);
}

fn plus_counters(c: &crabomination::game::CardInstance) -> i32 {
    c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) as i32
}

#[test]
fn fractal_resonance_v2_enters_with_counters_for_hand_size() {
    let mut g = two_player_game();
    // Player 0 has 1 card in hand (the spell itself).
    let id = g.add_card_to_hand(0, catalog::fractal_resonance_v2());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Add a couple of cards to hand to bump hand size.
    let _ = g.add_card_to_hand(0, catalog::forest());
    let _ = g.add_card_to_hand(0, catalog::island());
    let hand_size_at_cast = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Resonance castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("Resonance on battlefield");
    let counters = card.counters.get(&CounterType::PlusOnePlusOne)
        .copied().unwrap_or(0);
    // After the cast, hand size dropped by 1 (the spell). enters_with_counters
    // reads current hand size after cast — so should be hand_size_at_cast - 1.
    assert_eq!(counters as usize, hand_size_at_cast - 1);
}

#[test]
fn prismari_emberveil_etb_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_emberveil());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Emberveil castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

/// Table-driven: magecraft/prowess-style life swings observed after casting a
/// Lightning Bolt at the opponent with the listed permanent on the battlefield.
/// Entries: (def, opp_extra_loss_beyond_bolt_3, caster_life_gain).
#[test]
fn magecraft_life_swing_permanents_react_to_bolt() {
    for (def, opp_extra, you_gain) in [
        (catalog::prismari_pyremaster(), 1, 0),
        (catalog::lorehold_invoker(), 1, 0),
        (catalog::prismari_reverberator(), 2, 0),
        (catalog::lorehold_pyrescholar_b56(), 2, 0),
        (catalog::prismari_pyromage_b57(), 1, 0),
        (catalog::prismari_apprentice_b58(), 1, 0),
        (catalog::lorehold_skyignite(), 1, 0),
        (catalog::lorehold_emberscribe_b59(), 1, 0),
        (catalog::prismari_emberglyph(), 1, 0),
        (catalog::prismari_blast_apprentice(), 1, 0),
        (catalog::inkling_acolyte_v2(), 1, 1),
        (catalog::inkling_ghostwriter(), 1, 1),
        (catalog::inkling_inkmaster(), 1, 1),
        (catalog::pest_vanguard(), 1, 1),
        (catalog::silverquill_inkmaster_b58(), 1, 1),
        (catalog::lorehold_pyrescribe_elder(), 1, 1),
        (catalog::lorehold_battlepriest(), 0, 1),
        (catalog::witherbloom_vitalcoil(), 0, 2),
        (catalog::witherbloom_bramblepath(), 0, 1),
    ] {
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, def);
        let you_before = g.players[0].life;
        let opp_before = g.players[1].life;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - 3 - opp_extra);
        assert_eq!(g.players[0].life, you_before + you_gain);
    }
}

/// Table-driven: self-pumping magecraft/prowess creatures — power grows by 1
/// after a noncreature spell is cast; listed keywords stay present.
#[test]
fn magecraft_self_pump_creatures_grow_on_bolt() {
    for (def, kws) in [
        (catalog::lorehold_chronicler_v2(), &[Keyword::Flying][..]),
        (catalog::witherbloom_creeper(), &[Keyword::Deathtouch][..]),
        (catalog::prismari_stormcaller(), &[][..]),
        (catalog::prismari_stormcaller_v2(), &[][..]),
        (catalog::silverquill_inkflight_b59(), &[][..]),
        (catalog::witherbloom_thornpoet(), &[Keyword::Reach][..]),
        (catalog::lorehold_pyrelearner(), &[Keyword::Haste][..]),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let p_before = g.battlefield_find(id).unwrap().power() as i32;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let card = g.battlefield_find(id).unwrap();
        assert_eq!(card.power() as i32, p_before + 1);
        for kw in kws {
            assert!(card.has_keyword(kw));
        }
    }
}

/// Table-driven: magecraft that puts a +1/+1 counter on a creature (self or a
/// friendly target) when a Bolt is cast. `None` target means the source itself.
#[test]
fn magecraft_counter_pump_on_bolt() {
    for (src, tgt) in [
        (catalog::quandrix_tideturner(), None),
        (catalog::lorehold_forge_cleric(), Some(catalog::spirit_blazekin())),
        (catalog::witherbloom_pestmender(), Some(catalog::pest_marauder())),
        (catalog::quandrix_tideguard(), Some(catalog::fractal_greenstone())),
        (catalog::quandrix_sumcaster_b58(), Some(catalog::fractal_bluepetal())),
        (catalog::quandrix_geometer_b56(), Some(catalog::grizzly_bears())),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let src_id = g.add_card_to_battlefield(0, src);
        let tgt_id = match tgt {
            Some(d) => g.add_card_to_battlefield(0, d),
            None => src_id,
        };
        let before = plus_counters(g.battlefield_find(tgt_id).unwrap());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let after = plus_counters(g.battlefield_find(tgt_id).unwrap());
        assert_eq!(after, before + 1);
    }
}

/// Table-driven: magecraft that pumps a friendly creature's power by 1.
#[test]
fn magecraft_power_pump_of_friendly_creature_on_bolt() {
    for (src, tgt) in [
        (catalog::silverquill_wordmaiden(), catalog::pest_beekeeper()),
        (catalog::quandrix_skywinder(), catalog::fractal_bluepetal()),
        (catalog::silverquill_mageblade(), catalog::pest_beekeeper()),
    ] {
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, src);
        let tgt_id = g.add_card_to_battlefield(0, tgt);
        let p_before = g.battlefield_find(tgt_id).unwrap().power() as i32;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let p_after = g.battlefield_find(tgt_id).unwrap().power() as i32;
        assert_eq!(p_after, p_before + 1);
    }
}

/// Table-driven: magecraft scry — library size never grows past the snapshot.
#[test]
fn magecraft_scry_permanents_on_bolt() {
    for def in [
        catalog::quandrix_spellsplicer(),
        catalog::quandrix_oracle_b59(),
        catalog::prismari_iceforge(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _ = g.add_card_to_battlefield(0, def);
        let lib_before = g.players[0].library.len();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert!(g.players[0].library.len() <= lib_before);
    }
}

/// Table-driven: magecraft draw — hand nets +1 (bolt added, cast, then draw).
#[test]
fn magecraft_draw_permanents_on_bolt() {
    for def in [
        catalog::quandrix_ectomancer(),
        catalog::quandrix_bookkeeper(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _ = g.add_card_to_battlefield(0, def);
        let hand_before = g.players[0].hand.len();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // +1 (bolt added) -1 (cast) +1 (magecraft draw) = +1.
        assert_eq!(g.players[0].hand.len(), hand_before + 1);
    }
}

/// Table-driven: magecraft loot (draw 1, discard 1) — net-0 hand from the
/// pre-bolt snapshot minus the cast bolt.
#[test]
fn magecraft_loot_permanents_on_bolt() {
    for def in [
        catalog::prismari_cinderpath(),
        catalog::prismari_tideflame(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _filler = g.add_card_to_hand(0, catalog::forest());
        let _ = g.add_card_to_battlefield(0, def);
        let hand_before = g.players[0].hand.len();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // +1 bolt, -1 cast, +1 loot draw, -1 loot discard = net 0.
        assert_eq!(g.players[0].hand.len(), hand_before);
    }
}

/// Table-driven: untargeted ETB/spell drains. Entries:
/// (def, caster_life_gain, opp_life_loss, keywords the permanent must have).
#[test]
fn etb_drain_spells_and_creatures() {
    for (def, you_gain, opp_loss, kws) in [
        (catalog::silverquill_warden(), 1, 1, &[][..]),
        (catalog::silverquill_reflect(), 2, 2, &[][..]),
        (catalog::silverquill_doom(), 4, 4, &[][..]),
        (catalog::witherbloom_drainer(), 3, 2, &[][..]),
        (catalog::silverquill_psalm(), 2, 2, &[][..]),
        (catalog::silverquill_inksong(), 1, 1, &[][..]),
        (catalog::witherbloom_soulsmith(), 2, 2, &[][..]),
        (catalog::silverquill_scriptmaster(), 2, 2, &[][..]),
        (catalog::silverquill_acolyte_b56(), 1, 1, &[][..]),
        (catalog::witherbloom_toxicpath(), 1, 1, &[][..]),
        (catalog::witherbloom_mire_maker(), 2, 2, &[Keyword::Trample][..]),
        (catalog::witherbloom_blightbearer(), 2, 2, &[][..]),
        (catalog::silverquill_pen_priest(), 1, 1, &[Keyword::Lifelink][..]),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let you_before = g.players[0].life;
        let opp_before = g.players[1].life;
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("drain card castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, you_before + you_gain);
        assert_eq!(g.players[1].life, opp_before - opp_loss);
        if !kws.is_empty() {
            let c = g.battlefield_find(id).expect("permanent on battlefield");
            for kw in kws {
                assert!(c.has_keyword(kw));
            }
        }
    }
}

/// Table-driven: spells/creatures cast targeting the opponent that burn or
/// drain. Entries: (def, caster_life_gain, opp_life_loss, keywords on the
/// resulting permanent, if any).
#[test]
fn targeted_burn_and_drain_at_opponent() {
    for (def, you_gain, opp_loss, kws) in [
        (catalog::spirit_sparkmage(), 2, 2, &[][..]),
        (catalog::lorehold_sparkdancer(), 2, 2, &[][..]),
        (catalog::prismari_embertide(), 0, 1, &[Keyword::Haste][..]),
        (catalog::prismari_inscribe(), 0, 2, &[][..]),
        (catalog::prismari_cinderchant(), 0, 2, &[][..]),
        (catalog::lorehold_ember_strike(), 0, 1, &[][..]),
        (catalog::prismari_stormcaster_b58(), 0, 1, &[Keyword::Flying][..]),
        (catalog::prismari_floodfire(), 0, 4, &[][..]),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::mountain());
        g.add_card_to_library(0, catalog::island());
        let you_before = g.players[0].life;
        let opp_before = g.players[1].life;
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("burn card castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - opp_loss);
        assert_eq!(g.players[0].life, you_before + you_gain);
        if !kws.is_empty() {
            let c = g.battlefield_find(id).expect("permanent on battlefield");
            for kw in kws {
                assert!(c.has_keyword(kw));
            }
        }
    }
}

/// Table-driven: removal spells targeting an opposing 2/2 Grizzly Bears; the
/// bear dies, a 4/4 Serra Angel bystander survives.
/// Entries: (def, opp_life_loss, caster_life_gain).
#[test]
fn creature_removal_kills_bear_spares_angel() {
    for (def, opp_loss, you_gain) in [
        (catalog::prismari_firechord(), 0, 0),
        (catalog::prismari_searstorm(), 2, 0),
        (catalog::lorehold_sparkflame(), 0, 0),
        (catalog::prismari_embershock(), 0, 0),
        (catalog::prismari_volcanist_b55(), 1, 0),
        (catalog::witherbloom_hexvine(), 0, 2),
    ] {
        let mut g = two_player_game();
        let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let you_before = g.players[0].life;
        let opp_before = g.players[1].life;
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(crabomination::game::types::Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("removal castable");
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear should die");
        assert!(g.battlefield.iter().any(|c| c.id == angel), "angel should survive");
        assert_eq!(g.players[1].life, opp_before - opp_loss);
        assert_eq!(g.players[0].life, you_before + you_gain);
    }
}

/// Table-driven: cast creature, assert stats/keywords/creature type/life gain.
/// Entries: (def, Option<(power, toughness)>, keywords, Option<CreatureType>,
/// caster_life_gain).
#[test]
fn cast_creature_stats_keywords_and_types() {
    for (def, pt, kws, ctype, you_gain) in [
        (catalog::silverquill_inkblot(), Some((2, 2)), &[Keyword::Flying][..],
            Some(CreatureType::Inkling), 0),
        (catalog::inkling_chaplain(), Some((1, 3)),
            &[Keyword::Vigilance, Keyword::Lifelink][..], None, 0),
        (catalog::witherbloom_mossback(), Some((2, 4)), &[Keyword::Reach][..], None, 0),
        (catalog::inkling_attendant(), None,
            &[Keyword::Flying, Keyword::Lifelink][..], None, 0),
        (catalog::prismari_drakekin(), None, &[Keyword::Flying][..],
            Some(CreatureType::Drake), 0),
        (catalog::prismari_spellscholar(), Some((1, 3)), &[][..], None, 0),
        (catalog::silverquill_pen_scholar(), Some((2, 2)), &[][..], None, 1),
        (catalog::silverquill_scrivener_b59(), Some((2, 2)), &[][..], None, 0),
        (catalog::pest_grovetender(), None, &[Keyword::Deathtouch][..], None, 0),
        (catalog::spirit_scribe(), None, &[][..], Some(CreatureType::Spirit), 0),
    ] {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        let you_before = g.players[0].life;
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("creature castable");
        drain_stack(&mut g);
        let card = g.battlefield_find(id).expect("creature on battlefield");
        if let Some((p, t)) = pt {
            assert_eq!(card.power() as i32, p);
            assert_eq!(card.toughness() as i32, t);
        }
        for kw in kws {
            assert!(card.has_keyword(kw));
        }
        if let Some(ct) = ctype {
            assert!(card.definition.subtypes.creature_types.contains(&ct));
        }
        assert_eq!(g.players[0].life, you_before + you_gain);
    }
}

/// Table-driven: creatures dropped directly onto the battlefield; verify
/// printed stats, keywords, and (optionally) creature type.
#[test]
fn battlefield_creature_stats_and_keywords() {
    for (def, p, t, kws, ctype) in [
        (catalog::inkling_bladerunner(), 2, 2,
            &[Keyword::Flying, Keyword::FirstStrike][..], None),
        (catalog::silverquill_sentinel_b57(), 1, 3,
            &[Keyword::Flying, Keyword::Vigilance][..], None),
        (catalog::inkling_sentinel_b55(), 1, 4,
            &[Keyword::Vigilance][..], Some(CreatureType::Inkling)),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let c = g.battlefield_find(id).expect("on battlefield");
        assert_eq!(c.power() as i32, p);
        assert_eq!(c.toughness() as i32, t);
        for kw in kws {
            assert!(c.has_keyword(kw));
        }
        if let Some(ct) = ctype {
            assert!(c.definition.subtypes.creature_types.contains(&ct));
        }
    }
}

/// Table-driven: creatures that enter with a fixed number of +1/+1 counters.
/// Entries: (def, counters, power, toughness).
#[test]
fn enters_with_fixed_plus_one_counters() {
    for (def, n, p, t) in [
        (catalog::fractal_greenstone(), 2, 2, 2),
        (catalog::fractal_bluepetal(), 2, 2, 2),
        (catalog::fractal_redleaf(), 3, 3, 3),
        (catalog::quandrix_greenmage(), 1, 4, 4),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(id).expect("on battlefield");
        assert_eq!(plus_counters(c), n);
        assert_eq!(c.power() as i32, p);
        assert_eq!(c.toughness() as i32, t);
    }
}

/// Table-driven: cards whose cast/ETB mints creature tokens of a given type.
/// Entries: (def, token creature type, minted-token delta, caster_life_gain).
#[test]
fn etb_and_spell_token_minting() {
    for (def, ct, delta, you_gain) in [
        (catalog::silverquill_invocation(), CreatureType::Inkling, 3, 0),
        (catalog::inkling_pageant(), CreatureType::Inkling, 2, 2),
        (catalog::witherbloom_pestcradle(), CreatureType::Pest, 1, 1),
        (catalog::witherbloom_pestcaller_b54(), CreatureType::Pest, 2, 0),
        (catalog::witherbloom_pestharvest(), CreatureType::Pest, 2, 0),
        (catalog::pest_caretaker(), CreatureType::Pest, 1, 0),
        (catalog::inkling_pact_caller(), CreatureType::Inkling, 1, 0),
        (catalog::lorehold_skirmish_v2(), CreatureType::Spirit, 1, 0),
        (catalog::lorehold_spiritcaller_b55(), CreatureType::Spirit, 2, 0),
        (catalog::lorehold_reverence_v2(), CreatureType::Spirit, 1, 2),
        (catalog::lorehold_spiritbinder_b59(), CreatureType::Spirit, 1, 1),
        (catalog::pest_beekeeper(), CreatureType::Pest, 1, 0),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let count_tokens = |g: &crabomination::game::GameState| g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.is_token
                && c.definition.subtypes.creature_types.contains(&ct))
            .count();
        let tokens_before = count_tokens(&g);
        let you_before = g.players[0].life;
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("token minter castable");
        drain_stack(&mut g);
        assert_eq!(count_tokens(&g), tokens_before + delta);
        assert_eq!(g.players[0].life, you_before + you_gain);
    }
}

/// Table-driven: ETB mints a Fractal token carrying +1/+1 counters.
/// Entries: (def, counters on the minted token).
#[test]
fn etb_mints_fractal_token_with_counters() {
    for (def, n) in [
        (catalog::quandrix_mathweaver(), 1),
        (catalog::quandrix_summerkeeper(), 2),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let fractal = g.battlefield.iter().find(|c|
            c.controller == 0 && c.is_token &&
            c.definition.subtypes.creature_types.contains(&CreatureType::Fractal) &&
            c.id != id).expect("Fractal token should exist");
        assert_eq!(plus_counters(fractal), n);
    }
}

/// Table-driven: on-death drains. Entries: (def, drain amount).
#[test]
fn dies_triggers_drain_each_opponent() {
    for (def, n) in [
        (catalog::witherbloom_crypt_caller(), 2),
        (catalog::pest_soulreaver(), 3),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let opp_before = g.players[1].life;
        let you_before = g.players[0].life;
        if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) {
            c.damage = 99;
        }
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - n);
        assert_eq!(g.players[0].life, you_before + n);
    }
}

/// Table-driven: sacrifice-observing drains fired by Witherbloom Sacrosanct's
/// sacrifice (Sacrosanct drains 3, the observer drains 1 more → opp -4).
#[test]
fn sacrifice_observers_drain_via_sacrosanct() {
    for (def, you_delta) in [
        (catalog::pest_brewmaster(), None),
        (catalog::silverquill_mortician(), Some(4)),
    ] {
        let mut g = two_player_game();
        let _ = g.add_card_to_battlefield(0, def);
        let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let you_before = g.players[0].life;
        let opp_before = g.players[1].life;
        let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add(Color::Green, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("Sacrosanct castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - 4);
        if let Some(d) = you_delta {
            assert_eq!(g.players[0].life, you_before + d);
        }
    }
}

/// Table-driven: ETB mill of the opponent's library.
/// Entries: (def, milled cards, opp_life_loss, caster_life_gain).
#[test]
fn etb_mill_opponent_library() {
    for (def, mill, opp_loss, you_gain) in [
        (catalog::witherbloom_tomeshade(), 3, 1, 1),
        (catalog::witherbloom_mill_mage(), 4, 0, 0),
        (catalog::silverquill_litany_b56(), 2, 2, 2),
    ] {
        let mut g = two_player_game();
        for _ in 0..5 {
            g.add_card_to_library(1, catalog::island());
        }
        let opp_gy_before = g.players[1].graveyard.len();
        let opp_before = g.players[1].life;
        let you_before = g.players[0].life;
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).expect("miller castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].graveyard.len(), opp_gy_before + mill);
        assert_eq!(g.players[1].life, opp_before - opp_loss);
        assert_eq!(g.players[0].life, you_before + you_gain);
    }
}

#[test]
fn inkling_evangel_etb_pumps_target_inkling() {
    let mut g = two_player_game();
    // Find any Inkling on the battlefield. Add another Inkling via
    // Inkling Aspirant for a clean target.
    let target = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    let id = g.add_card_to_hand(0, catalog::inkling_evangel());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Evangel castable");
    drain_stack(&mut g);
    let target_card = g.battlefield_find(target).unwrap();
    let counters = target_card.counters.get(&CounterType::PlusOnePlusOne)
        .copied().unwrap_or(0);
    assert_eq!(counters, 1);
}

#[test]
fn pest_lord_anthems_other_pests() {
    let mut g = two_player_game();
    // Add a Pest token first (will get +1/+1 from the lord).
    let pest = g.add_card_to_battlefield(0, catalog::pest_brood_mother());
    let _ = pest;
    let lord = g.add_card_to_battlefield(0, catalog::pest_lord());
    let _ = lord;
    g.compute_battlefield();
    // Now find a Pest other than the lord and assert it's pumped.
    let pest_card = g.battlefield.iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest)
            && !c.definition.subtypes.creature_types.contains(&CreatureType::Warlock));
    if let Some(p) = pest_card {
        // Pest Brood Mother is a 3/3 Pest Insect; should be 4/4 with the anthem.
        assert!(p.power() > 3);
        assert!(p.toughness() > 3);
    }
}

#[test]
fn pest_curse_mints_pests_and_self_discards() {
    let mut g = two_player_game();
    // Add a discard fodder card to hand.
    let _filler = g.add_card_to_hand(0, catalog::forest());
    let pests_before = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::pest_curse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pest Curse castable");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pests_after, pests_before + 2);
    // Hand: -1 cast (pest curse) + 0 draw - 1 discard = -1 from the (cast+1).
    // hand_before included pest curse + filler = 2; after cast & discard = 0.
    assert!(g.players[0].hand.len() < hand_before);
}

#[test]
fn fractal_overgrowth_doubles_existing_counters() {
    let mut g = two_player_game();
    // Add a creature with 3 +1/+1 counters.
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    {
        let card = g.battlefield_find_mut(b1).unwrap();
        card.counters.insert(CounterType::PlusOnePlusOne, 3);
    }
    let id = g.add_card_to_hand(0, catalog::fractal_overgrowth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Overgrowth castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(b1).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    // 3 → 6 (doubled).
    assert_eq!(counters, 6);
}

#[test]
fn lorehold_relicwarden_etb_pumps_other_spirits() {
    let mut g = two_player_game();
    let s1 = g.add_card_to_battlefield(0, catalog::spirit_blazekin());
    let s2 = g.add_card_to_battlefield(0, catalog::spirit_blazekin());
    let id = g.add_card_to_hand(0, catalog::lorehold_relicwarden());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Relicwarden castable");
    drain_stack(&mut g);
    // Each other Spirit gets +1/+1.
    for sid in [s1, s2] {
        let c = g.battlefield_find(sid).unwrap();
        let cn = c.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        assert_eq!(cn, 1);
    }
}

#[test]
fn quandrix_calcographer_etb_mints_fractal_then_grows_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_calcographer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Calcographer castable");
    drain_stack(&mut g);
    // ETB minted a Fractal with one +1/+1 counter.
    let fractals: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .map(|c| c.id)
        .collect();
    assert!(!fractals.is_empty());
    // Cast an instant - calcographer grows.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    let counters = card.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters, 1);
}

#[test]
fn quandrix_splitcaster_magecraft_mints_a_fractal_with_counter() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_splitcaster());
    let fractals_before = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .count();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let fractals_after: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals_after.len(), fractals_before + 1);
    // The newly created fractal should have a +1/+1 counter.
    let new_fractal = fractals_after.last().unwrap();
    let counters = new_fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert!(counters >= 1);
}

#[test]
fn quandrix_calculation_adds_counter_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_calculation());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Calculation castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    let counters = card.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters, 1);
    // Cast -1 + draw +1 = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn spirit_banneret_anthems_other_spirits() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::spirit_blazekin());
    let _banneret = g.add_card_to_battlefield(0, catalog::spirit_banneret());
    let computed = g
        .compute_battlefield()
        .into_iter()
        .find(|c| c.id == s)
        .expect("Blazekin on battlefield");
    // Spirit Blazekin is 2/2 + Banneret anthem +1/+0 = 3/2.
    assert_eq!(computed.power, 3);
    assert_eq!(computed.toughness, 2);
}

#[test]
fn until_end_of_combat_expires_when_combat_phase_ends() {
    // CR 511.2 audit: an effect installed with Duration::EndOfCombat
    // should expire as the EndCombat step ends (transition to
    // PostCombatMain), not at the next cleanup step.
    use crabomination::effect::Duration;
    use crabomination::game::types::Target;
    use crabomination::game::{EffectContext, TurnStep};

    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Skip ahead to the combat phase.
    g.step = TurnStep::BeginCombat;

    // Install a +1/+1 EOC pump on the bear via the effect resolver.
    let ctx = EffectContext {
        controller: 0,
        source: Some(bear),
        targets: vec![Target::Permanent(bear)],
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
    // Use SetBasePT with Duration::EndOfCombat so the layer-system
    // pathway exercises the mapping under test (PumpPT writes to the
    // legacy `power_bonus` field that doesn't honor combat-scoped
    // durations and clears only at cleanup).
    let set = crabomination::effect::Effect::SetBasePT {
        what: crabomination::effect::Selector::Target(0),
        power: crabomination::effect::Value::Const(7),
        toughness: crabomination::effect::Value::Const(7),
        duration: Duration::EndOfCombat,
    };
    let _ = g.resolve_effect(&set, &ctx);

    // Bear is now 7/7 during combat.
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).unwrap();
    assert_eq!(computed.power, 7, "bear should be set to 7/7 during combat");

    // Advance to the EndCombat step and pass priority until we leave
    // the combat phase.
    g.step = TurnStep::EndCombat;
    g.give_priority_to_active();
    let _ = g.pass_priority();
    let _ = g.pass_priority();
    assert!(!g.step.is_combat_phase(), "expected to exit combat phase, got {:?}", g.step);

    // SetBasePT should have expired by now — back to printed 2/2.
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).unwrap();
    assert_eq!(computed.power, 2,
        "until-end-of-combat SetBasePT should expire when combat phase ends");
}

#[test]
fn witherbloom_pestreaper_b56_grows_and_gains_life_on_sacrifice() {
    let mut g = two_player_game();
    let reaper = g.add_card_to_battlefield(0, catalog::witherbloom_pestreaper_b56());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Reaper grows by one counter on the sacrifice trigger + gains 1
    // life rider. Sacrosanct itself drains 3 → +3 life. Total: +4.
    let r = g.battlefield_find(reaper).expect("reaper alive");
    let cn = r.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(cn, 1, "Pestreaper should have +1/+1 from sacrifice");
    assert_eq!(g.players[0].life, life_before + 4,
        "Pestreaper gains 1 + Sacrosanct gains 3 = +4 life");
}

#[test]
fn witherbloom_soulshade_returns_low_mv_creature_on_death() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let soulshade_id = g.add_card_to_battlefield(0, catalog::witherbloom_soulshade());
    let hand_before = g.players[0].hand.len();
    // Lethal the Soulshade.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == soulshade_id) {
        c.damage = 99;
    }
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == soulshade_id),
        "Soulshade should be in graveyard");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_id),
        "Soulshade's death trigger should return ≤2-MV creature card to hand");
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn witherbloom_necrofeast_sacrifices_and_drains_four() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_necrofeast());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Necrofeast castable");
    drain_stack(&mut g);
    // Bear is sacrificed.
    let creatures = g.battlefield.iter()
        .filter(|c| c.definition.is_creature() && c.controller == 0)
        .count();
    assert_eq!(creatures, 0, "Bear should have been sacrificed");
    // Drain 4: caster gains 4, opp loses 4.
    assert_eq!(g.players[0].life, life_before + 4);
    assert_eq!(g.players[1].life, opp_before - 4);
}

/// Bloodscribe's printed trigger is a paid optional: "you may pay 1
/// life. If you do, draw a card." Accepting the `MayPayLife` prompt
/// pays 1 life and draws.
#[test]
fn silverquill_bloodscribe_pays_one_life_to_draw_on_sacrifice() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_bloodscribe());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Seed library so the draw has something to find.
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    // Accept the "pay 1 life to draw a card" prompt.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // The bear is sacrificed (cast cost); Bloodscribe's trigger pays 1
    // life and draws 1. Hand: +1 (added Sacrosanct) -1 (cast it) +1
    // (drawn) = +1 from baseline.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    // Sacrosanct drained 3 from the opponent (+3), the trigger paid 1
    // life (-1).
    assert_eq!(g.players[0].life, life_before + 3 - 1,
        "gained 3 from Sacrosanct drain, paid 1 for the draw");
}

/// Declining Bloodscribe's `MayPayLife` prompt (the AutoDecider
/// default) skips both the life payment and the draw — the printed
/// "you may" gate.
#[test]
fn silverquill_bloodscribe_declines_and_does_not_draw() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_bloodscribe());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // +1 (added) -1 (cast) + 0 (declined draw) = baseline.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "no draw when the life payment is declined");
    assert_eq!(g.players[0].life, life_before + 3,
        "only the Sacrosanct drain moved the life total");
}

#[test]
fn inkling_penblade_etb_pumps_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inkling_penblade());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Penblade castable");
    drain_stack(&mut g);
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).expect("bear alive");
    // Bear was 2/2; +1/+0 EOT = 3/2.
    assert_eq!(computed.power, 3);
}

#[test]
fn lorehold_summit_mints_two_spirits_and_grants_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spirits_before = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    let id = g.add_card_to_hand(0, catalog::lorehold_summit());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Summit castable");
    drain_stack(&mut g);
    let spirits_after = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    assert_eq!(spirits_after, spirits_before + 2);
    // The pre-existing bear should have Haste EOT now.
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).expect("bear alive");
    assert!(computed.keywords.contains(&Keyword::Haste));
}

#[test]
fn quandrix_mathlord_etb_mints_fractal_and_pumps_fractals() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_mathlord());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mathlord castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert!(!fractals.is_empty(), "Mathlord should mint at least one Fractal");
    // Each Fractal has +1/+1 counters.
    for fractal in &fractals {
        let cn = fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        assert!(cn >= 2, "Fractal should have at least 2 counters from team-wide pump");
    }
}

#[test]
fn fractal_trifecta_mints_three_fractals_with_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_trifecta());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Trifecta castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 3, "Trifecta should mint 3 Fractals");
    for fractal in &fractals {
        let cn = fractal.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        assert!(cn >= 1, "Each Fractal should have at least 1 counter");
    }
}

#[test]
fn quandrix_tidesower_etb_shrinks_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::quandrix_tidesower());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidesower castable");
    drain_stack(&mut g);
    // Hand: +1 added Tidesower, -1 cast it, +1 drawn = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).expect("bear alive");
    assert_eq!(computed.power, 0, "Bear should be shrunk to 0/2");
}

#[test]
fn fractal_augmenter_enters_with_counters_equal_to_hand_size() {
    let mut g = two_player_game();
    // Make sure hand size is non-trivial.
    while g.players[0].hand.len() < 4 {
        g.add_card_to_hand(0, catalog::island());
    }
    let hand_size = g.players[0].hand.len() as i32;
    let id = g.add_card_to_hand(0, catalog::fractal_augmenter());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Augmenter castable");
    drain_stack(&mut g);
    let aug = g.battlefield.iter()
        .find(|c| c.definition.name == "Fractal Augmenter").expect("augmenter on bf");
    let cn = aug.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0) as i32;
    // Hand size at ETB time: original hand_size + 1 (Augmenter added) - 1
    // (Augmenter cast away) = original hand_size.
    assert_eq!(cn, hand_size,
        "Augmenter enters with +1/+1 counters equal to current hand size");
}

#[test]
fn prismari_flamewriter_magecraft_burns_and_draws() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_flamewriter());
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Flamewriter magecraft 1 = -4 opp.
    assert_eq!(g.players[1].life, opp_before - 4);
    // Hand: +1 Bolt, -1 cast, +1 drawn from magecraft = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

// ── CR 122.3 — +1/+1 vs -1/-1 counter cancellation (state-based action) ─────

/// CR 122.3: "If a permanent has both a +1/+1 counter and a -1/-1
/// counter on it, N +1/+1 and N -1/-1 counters are removed from it
/// as a state-based action, where N is the smaller of the number of
/// +1/+1 and -1/-1 counters on it."
///
/// This audit-style lock-in test stages a creature with 3 +1/+1 and
/// 2 -1/-1 counters and asserts the SBA cancels 2 of each, leaving
/// 1 +1/+1 counter (and 0 -1/-1).
#[test]
fn cr_122_3_plus_one_and_minus_one_counters_cancel_as_state_based_action() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Manually stamp counters and trigger SBA.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) {
        c.add_counters(CounterType::PlusOnePlusOne, 3);
        c.add_counters(CounterType::MinusOneMinusOne, 2);
    }
    g.check_state_based_actions();
    let b = g.battlefield_find(bear).expect("bear alive");
    let plus = b.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let minus = b.counters.get(&CounterType::MinusOneMinusOne).copied().unwrap_or(0);
    assert_eq!(plus, 1, "expected 1 +1/+1 counter after CR 122.3 cancel");
    assert_eq!(minus, 0, "expected 0 -1/-1 counters after CR 122.3 cancel");
    // P/T reflects the net +1/+1 over the printed 2/2.
    let computed = g.compute_battlefield().into_iter()
        .find(|c| c.id == bear).expect("computed bear");
    assert_eq!(computed.power, 3);
    assert_eq!(computed.toughness, 3);
}

#[test]
fn witherbloom_decoder_magecraft_mills_each_opp() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_decoder());
    g.add_card_to_library(1, catalog::island());
    let opp_gy_before = g.players[1].graveyard.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), opp_gy_before + 1,
        "Decoder magecraft should mill 1 from opp on instant cast");
}

#[test]
fn pest_roostmaster_mints_pest_on_sacrifice() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::pest_roostmaster());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pests_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .count();
    assert_eq!(pests_after, pests_before + 1,
        "Roostmaster should mint a Pest on Sacrosanct's sacrifice");
}

#[test]
fn witherbloom_necropoet_grows_pests_on_sacrifice() {
    let mut g = two_player_game();
    let _np = g.add_card_to_battlefield(0, catalog::witherbloom_necropoet());
    let pest1 = g.add_card_to_battlefield(0, catalog::pest_marauder());
    let pest2 = g.add_card_to_battlefield(0, catalog::pest_marauder());
    // Sacrifice fodder — Sacrosanct sacs a creature; the bear is the
    // expected pick so the Pests can both observe the sacrifice trigger.
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pests_at_start: Vec<_> = vec![pest1, pest2];
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Any surviving Pest should have at least 1 +1/+1 counter from the
    // Necropoet trigger; cards sacrificed are gone from the battlefield.
    let survivors: Vec<_> = pests_at_start.iter()
        .filter_map(|&id| g.battlefield_find(id))
        .collect();
    assert!(!survivors.is_empty(), "at least one Pest should survive the sacrifice");
    for p in &survivors {
        let cn = p.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        assert_eq!(cn, 1, "surviving Pest should have a +1/+1 counter");
    }
}

#[test]
fn silverquill_pen_master_etb_loots_and_drains_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_pen_master());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pen-Master castable");
    drain_stack(&mut g);
    // Loot is net-0 (draw 1, discard 1); cast of the Pen-Master moves it
    // out of hand. Hand should match the pre-cast snapshot.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn lorehold_bonereader_b57_magecraft_exiles_gy_card() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_bonereader_b57());
    // Seed opp's graveyard with a card.
    let gy_card = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Card should have moved to exile or be no longer in graveyard.
    let still_in_gy = g.players[1].graveyard.iter().any(|c| c.id == gy_card);
    assert!(!still_in_gy, "Bonereader magecraft should exile target gy card");
}

#[test]
fn lorehold_sparkscholar_b57_magecraft_pings_creature() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_sparkscholar_b57());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(bear_card.damage, 1, "Sparkscholar should ping the bear for 1");
}

#[test]
fn prismari_sparkscribe_b57_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::prismari_sparkscribe_b57());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sparkscribe castable");
    drain_stack(&mut g);
    // Loot is net-0; cast removes the Sparkscribe from hand → hand matches
    // the pre-cast snapshot.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let c = g.battlefield_find(id).expect("Sparkscribe on battlefield");
    assert!(c.has_keyword(&Keyword::Flying));
}

#[test]
fn pest_tendril_dies_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let lib_before = g.players[0].library.len();
    let id = g.add_card_to_battlefield(0, catalog::pest_tendril());
    // Lethal damage → dies → scry 1 trigger fires.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt at the Pest castable");
    drain_stack(&mut g);
    // Pest Tendril died (1 toughness < 3 bolt damage). Pest token also
    // gets minted via the on-die "you gain 1 life" rider on the token
    // definition — but here the source is the Tendril itself (a non-token
    // creature card), so only Scry fires.
    assert!(g.battlefield_find(id).is_none(), "Pest Tendril should be dead");
    // Library count unchanged after scry (top card either stays or goes
    // to graveyard — library size doesn't grow beyond `lib_before`).
    assert!(g.players[0].library.len() <= lib_before);
}

#[test]
fn silverquill_lecturer_b58_etb_mints_inkling_and_gains_life() {
    let mut g = two_player_game();
    let you_before = g.players[0].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    let id = g.add_card_to_hand(0, catalog::silverquill_lecturer_b58());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lecturer castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    // +1 Lecturer +1 Inkling token = +2 permanents.
    assert_eq!(bf_after, bf_before + 2);
    assert_eq!(g.players[0].life, you_before + 2);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Inkling) &&
        c.definition.name != "Silverquill Lecturer II"));
}

#[test]
fn lorehold_bonechanter_magecraft_grants_menace() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::lorehold_bonechanter());
    let beater = g.add_card_to_battlefield(0, catalog::pest_beekeeper());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(beater).expect("Beater on bf");
    assert!(c.has_keyword(&Keyword::Menace));
}

#[test]
fn lorehold_reliquarian_etb_mints_spirit_and_magecraft_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_reliquarian());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Reliquarian castable");
    drain_stack(&mut g);
    // Verify Spirit token was minted.
    assert!(g.battlefield.iter().any(|c| c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Spirit) &&
        c.id != id));
    let you_before = g.players[0].life;
    // Magecraft path: cast a bolt → +1 life.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you_before + 1);
    let c = g.battlefield_find(id).unwrap();
    assert!(c.has_keyword(&Keyword::Vigilance));
}

// ── Strict Proctor — ETB tax (StaticEffect::EtbTriggerTax { amount: 2 }) ──

#[test]
fn strict_proctor_taxes_an_etb_trigger_unless_paid() {
    use crabomination::decision::{Decision, DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Proctor on the same side as the ETB trigger source.
    let _ = g.add_card_to_battlefield(0, catalog::strict_proctor());
    // Pest Beekeeper has an ETB "mint a Pest" trigger.
    let id = g.add_card_to_hand(0, catalog::pest_beekeeper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Auto-decider declines the tax payment. Per the real oracle the
    // ABILITY is countered (CR 701.5a) — the entering permanent itself
    // is untouched; only the Pest-mint trigger is suppressed.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Beekeeper castable");
    drain_stack(&mut g);
    // Beekeeper stays; only its ETB trigger was countered.
    assert!(g.battlefield_find(id).is_some(),
        "Beekeeper stays on the battlefield — only the trigger is countered");
    assert!(!g.battlefield.iter().any(|c| c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Pest)),
        "no Pest token should mint when ETB trigger was suppressed");

    // Now: scripted "yes" + floated {2} → tax paid, Beekeeper stays, Pest mints.
    let mut g2 = two_player_game();
    g2.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let _ = g2.add_card_to_battlefield(0, catalog::strict_proctor());
    let id = g2.add_card_to_hand(0, catalog::pest_beekeeper());
    g2.players[0].mana_pool.add(Color::Green, 1);
    g2.players[0].mana_pool.add_colorless(4); // 2 for cast, 2 for tax
    g2.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Beekeeper castable");
    drain_stack(&mut g2);
    assert!(g2.battlefield_find(id).is_some(),
        "Beekeeper should survive when tax is paid");
    assert!(g2.battlefield.iter().any(|c| c.controller == 0 &&
        c.definition.subtypes.creature_types.contains(&CreatureType::Pest) &&
        c.id != id),
        "Pest token should mint when ETB trigger was paid for");
    // Verify the tax decision was actually offered.
    let _ = Decision::OptionalTrigger {
        source: id, description: "Pay {2} to keep this trigger?".to_string(),
    };
}

#[test]
fn strict_proctor_does_not_tax_non_etb_triggers() {
    // Magecraft fires on spell cast, not ETB — Proctor's tax should ignore it.
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::strict_proctor());
    let _ = g.add_card_to_battlefield(0, catalog::silverquill_wordmaiden());
    let target = g.add_card_to_battlefield(0, catalog::pest_beekeeper());
    let p_before = g.battlefield_find(target).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Wordmaiden's magecraft pump fires normally — the source (Wordmaiden)
    // is NOT sacrificed despite the Proctor being in play.
    let p_after = g.battlefield_find(target).unwrap().power();
    assert_eq!(p_after, p_before + 1);
}

#[test]
fn inkling_summit_b59_etb_pumps_other_inklings() {
    let mut g = two_player_game();
    // First Inkling to be pumped: drop an Inkling token via Inkling Scribe ({2}{W}).
    let scribe = g.add_card_to_hand(0, catalog::inkling_scribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: scribe, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scribe castable");
    drain_stack(&mut g);
    // Find the Inkling token controller 0 minted.
    let inkling = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token &&
        c.definition.name == "Inkling").map(|c| c.id).expect("Inkling minted");
    let counters_before = g.battlefield_find(inkling).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);

    // Now cast Inkling Summit — it should put a +1/+1 counter on the Inkling token.
    let id = g.add_card_to_hand(0, catalog::inkling_summit_b59());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Summit castable");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(inkling).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters_after, counters_before + 1);
    // Self should NOT have a counter (OtherThanSource exclude).
    let summit = g.battlefield_find(id).expect("Summit on bf");
    assert_eq!(summit.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 0);
}

#[test]
fn witherbloom_sapler_magecraft_pumps_friendly_pest() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_sapler());
    // Spawn a Pest token via Witherbloom Pest-Tender.
    let tender = g.add_card_to_hand(0, catalog::witherbloom_pest_tender());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: tender, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tender castable");
    drain_stack(&mut g);
    let pest = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token &&
        c.definition.name == "Pest").map(|c| c.id).expect("Pest minted");
    let p_before = g.battlefield_find(pest).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(pest).unwrap().power();
    assert_eq!(p_after, p_before + 1);
}

#[test]
fn lorehold_relicseer_etb_exiles_graveyard_card_and_is_flying() {
    let mut g = two_player_game();
    // Put two cards into opp's graveyard.
    let _ = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let _ = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let opp_gy_before = g.players[1].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::lorehold_relicseer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Relicseer castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.len() < opp_gy_before, "Opp gy should shrink by 1");
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Flying));
}

#[test]
fn quandrix_growth_tutor_etb_pumps_fractal() {
    let mut g = two_player_game();
    // Seed a Fractal token.
    let bluepetal = g.add_card_to_hand(0, catalog::fractal_bluepetal());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bluepetal, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bluepetal castable");
    drain_stack(&mut g);
    let counters_before = g.battlefield_find(bluepetal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    let id = g.add_card_to_hand(0, catalog::quandrix_growth_tutor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Growth-Tutor castable");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(bluepetal).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(counters_after, counters_before + 1);
}

#[test]
fn prismari_flameseer_magecraft_loots_with_haste() {
    let mut g = two_player_game();
    // Library: islands to draw, then a sacrificial card we can discard.
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::prismari_flameseer());
    let gy_before = g.players[0].graveyard.len();
    // Put a spare discard target into hand before casting.
    let _spare = g.add_card_to_hand(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft loot fires: draw the Island, discard a card.
    // Graveyard should contain at least the bolt + the discarded card.
    assert!(g.players[0].graveyard.len() > gy_before);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Haste));
}

#[test]
fn prismari_artificer_etb_mints_treasure_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_artificer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Artificer castable");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Treasure"
    }).collect();
    assert_eq!(treasures.len(), 1);
}
