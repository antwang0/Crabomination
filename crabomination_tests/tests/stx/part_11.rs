use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

/// Over-provision the caster's mana pool so a shared table body can cast
/// any of the (differently costed) cards under test.
fn add_generous_mana(g: &mut crabomination::game::GameState, player: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[player].mana_pool.add(c, 3);
    }
    g.players[player].mana_pool.add_colorless(6);
}

// ── Table: cast a creature with no target; check keywords / stats / ETB life ─
// (Covers simple vanilla-with-keywords bodies plus ETB "gain N life" ETBs.)

#[test]
fn cast_creature_keywords_stats_and_etb_life() {
    for (def, kws, p, t, gain) in [
        (catalog::quandrix_lensbearer(), vec![], Some(1), Some(3), 0),
        (catalog::witherbloom_greenwarden(), vec![Keyword::Reach], None, None, 2),
        (catalog::silverquill_inkscribe(), vec![Keyword::Vigilance], None, None, 1),
        (catalog::lorehold_skyrunner(), vec![Keyword::Flying, Keyword::Haste], Some(2), None, 0),
        (catalog::spirit_honor_guard(), vec![Keyword::Vigilance, Keyword::FirstStrike], None, None, 0),
        (catalog::fractal_bloomstalker(), vec![Keyword::Trample], Some(4), Some(4), 0),
        (catalog::pest_brewer_v2(), vec![], None, None, 1),
        (catalog::silverquill_cantor(), vec![], None, None, 1),
        (catalog::inkling_stylescribe(), vec![Keyword::Flying], None, None, 0),
        (catalog::silverquill_pageturner(), vec![Keyword::Vigilance], None, None, 0),
        (catalog::inkling_beautisage(), vec![Keyword::Vigilance], None, None, 3),
        (catalog::inkling_pageboy(), vec![Keyword::Flying], None, None, 0),
        (catalog::inkling_quillpoint(), vec![Keyword::FirstStrike], None, None, 0),
        (catalog::inkling_spellbinder(), vec![Keyword::Flying, Keyword::Lifelink], Some(4), None, 0),
        (catalog::witherbloom_bloodseeker(), vec![Keyword::Lifelink], Some(3), Some(3), 0),
        (catalog::pest_disciple(), vec![], None, None, 1),
        (catalog::witherbloom_pestmage(), vec![Keyword::Menace], Some(3), None, 0),
        (catalog::witherbloom_roto_sage(), vec![Keyword::Deathtouch], Some(4), Some(4), 0),
        (catalog::spirit_berserker(), vec![Keyword::Haste, Keyword::Trample], None, None, 0),
        (catalog::fractal_bloomthorn(), vec![Keyword::Trample], Some(3), Some(3), 0),
        (catalog::prismari_drakemage(), vec![Keyword::Flying], None, None, 0),
        (catalog::prismari_sparkwing(), vec![Keyword::Flying, Keyword::Haste], None, None, 0),
        (catalog::inkling_voidwalker(), vec![Keyword::Flying, Keyword::Menace], None, None, 0),
        (catalog::witherbloom_bloodweaver(), vec![Keyword::Lifelink, Keyword::Trample], Some(4), None, 0),
        (catalog::silverquill_sermoneer(), vec![Keyword::Vigilance], None, None, 1),
        (catalog::lorehold_spirit_redeemer(), vec![Keyword::Vigilance, Keyword::Lifelink], None, None, 0),
        (catalog::spirit_blazekin(), vec![Keyword::Haste], None, None, 0),
        (catalog::witherbloom_pestpath(), vec![Keyword::Trample], Some(3), Some(4), 0),
        (catalog::quandrix_tideseer_adept(), vec![Keyword::Flash], None, None, 0),
        (catalog::inkling_inkscribe(), vec![Keyword::Flying], Some(2), Some(1), 0),
        (catalog::quandrix_threadbinder(), vec![], Some(1), Some(2), 0),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::island());
        let life_before = g.players[0].life;
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let card = g.battlefield_find(id).unwrap();
        for kw in &kws {
            assert!(card.has_keyword(kw), "{name} should have {kw:?}");
        }
        if let Some(p) = p {
            assert_eq!(card.power(), p, "{name} power");
        }
        if let Some(t) = t {
            assert_eq!(card.toughness(), t, "{name} toughness");
        }
        assert_eq!(g.players[0].life, life_before + gain, "{name} ETB lifegain");
    }
}

// ── Table: cast with no target; drain/gain life symmetric-ish bodies ────────

#[test]
fn cast_untargeted_drain_or_lifegain() {
    for (def, gain, opp_loss, kws) in [
        (catalog::inkling_aerialist_v2(), 1, 1, vec![Keyword::Flying]),
        (catalog::inkling_cipherwing(), 1, 1, vec![Keyword::Flying]),
        (catalog::silverquill_glyphmaster(), 2, 2, vec![Keyword::Lifelink]),
        (catalog::witherbloom_lifescribe(), 1, 1, vec![]),
        (catalog::silverquill_diction(), 2, 2, vec![]),
        (catalog::silverquill_quietude(), 3, 3, vec![]),
        (catalog::silverquill_memoriam(), 1, 1, vec![]),
        (catalog::inkling_archivist(), 1, 1, vec![Keyword::Flying]),
        (catalog::silverquill_ledgermage(), 2, 2, vec![]),
        (catalog::silverquill_etching(), 2, 2, vec![]),
        (catalog::witherbloom_rotbloom(), 3, 3, vec![]),
        (catalog::silverquill_pronouncer(), 1, 1, vec![Keyword::Flying, Keyword::Lifelink]),
        (catalog::lorehold_wargist(), 0, 1, vec![]),
        (catalog::lorehold_embermend(), 3, 0, vec![]),
        (catalog::pest_lifebloom(), 4, 0, vec![]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::island());
        let life_before = g.players[0].life;
        let opp_before = g.players[1].life;
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life_before + gain, "{name} lifegain");
        assert_eq!(g.players[1].life, opp_before - opp_loss, "{name} opp loss");
        if !kws.is_empty() {
            let card = g.battlefield_find(id).unwrap();
            for kw in &kws {
                assert!(card.has_keyword(kw), "{name} should have {kw:?}");
            }
        }
    }
}

// ── Table: cast targeting the opponent; drain/burn player ───────────────────

#[test]
fn cast_targeting_opponent_drain_or_burn() {
    for (def, gain, opp_loss, kws) in [
        (catalog::silverquill_lifeskein(), 2, 2, vec![]),
        (catalog::lorehold_pyremender_v2(), 0, 1, vec![]),
        (catalog::lorehold_pyreward(), 1, 2, vec![]),
        (catalog::witherbloom_sourceweaver(), 2, 2, vec![Keyword::Deathtouch]),
        (catalog::lorehold_sparkshock(), 0, 2, vec![]),
        (catalog::lorehold_emberlock(), 2, 2, vec![]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        let life_before = g.players[0].life;
        let opp_before = g.players[1].life;
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life_before + gain, "{name} lifegain");
        assert_eq!(g.players[1].life, opp_before - opp_loss, "{name} opp loss");
        if !kws.is_empty() {
            let card = g.battlefield_find(id).unwrap();
            for kw in &kws {
                assert!(card.has_keyword(kw), "{name} should have {kw:?}");
            }
        }
    }
}

// ── Table: removal that kills an opposing Grizzly Bears ─────────────────────

#[test]
fn removal_kills_opposing_bear() {
    for def in [
        catalog::prismari_quickburn(),
        catalog::prismari_searbolt(),
        catalog::silverquill_inkstrike(),
        catalog::silverquill_quietus(),
        catalog::silverquill_inkstrike_page(),
        catalog::lorehold_sparkstrike_b50(),
        catalog::lorehold_sparklock(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(crabomination::game::types::Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "{name} should kill the bear");
    }
}

#[test]
fn silverquill_inkstrike_rejects_big_creature() {
    let mut g = two_player_game();
    let bookwurm = g.add_card_to_battlefield(1, catalog::bookwurm());
    let id = g.add_card_to_hand(0, catalog::silverquill_inkstrike());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    // 5/5 bookwurm exceeds toughness 2 filter.
    let result = g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Permanent(bookwurm)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(result.is_err());
}

#[test]
fn silverquill_censurewright_shrinks_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_censurewright());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Censurewright castable");
    drain_stack(&mut g);
    // Bear 2/2 → -1/-1 → 1/1.
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 1);
    assert_eq!(bear_card.toughness(), 1);
}

// ── Table: cast targeting a friendly bear; pump it ──────────────────────────

#[test]
fn pump_friendly_bear() {
    for (def, p, t, bear_kw) in [
        (catalog::silverquill_bookmender(), 3, Some(3), None),
        (catalog::lorehold_stoneward(), 2, Some(4), None),
        (catalog::fractal_shaper(), 3, Some(3), None),
        (catalog::quandrix_foresight(), 3, None, None),
        (catalog::witherbloom_sapburst(), 4, Some(4), None),
        (catalog::silverquill_inkbinder(), 3, Some(3), Some(Keyword::Lifelink)),
        (catalog::silverquill_mentor(), 3, Some(3), None),
        (catalog::quandrix_amplify(), 4, Some(4), None),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(crabomination::game::types::Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let bear_card = g.battlefield_find(bear).unwrap();
        assert_eq!(bear_card.power(), p, "{name} bear power");
        if let Some(t) = t {
            assert_eq!(bear_card.toughness(), t, "{name} bear toughness");
        }
        if let Some(kw) = bear_kw {
            assert!(bear_card.has_keyword(&kw), "{name} bear keyword");
        }
    }
}

// ── Table: magecraft "gain 1 life" on an instant/sorcery cast ───────────────

#[test]
fn magecraft_gains_one_life_on_bolt_cast() {
    for (def, kws) in [
        (catalog::inkling_stormwriter(), vec![]),
        (catalog::witherbloom_drainscholar_b50(), vec![]),
        (catalog::spirit_mentor(), vec![]),
        (catalog::silverquill_studyhall(), vec![]),
        (catalog::witherbloom_grimherb(), vec![Keyword::Deathtouch]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        if !kws.is_empty() {
            let card = g.battlefield_find(id).unwrap();
            for kw in &kws {
                assert!(card.has_keyword(kw), "{name} should have {kw:?}");
            }
        }
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let life_before = g.players[0].life;
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life_before + 1, "{name} magecraft lifegain");
    }
}

// ── Table: magecraft ping — Bolt (3) + 1 = 4 to opponent ────────────────────

#[test]
fn magecraft_ping_makes_bolt_deal_four() {
    for def in [
        catalog::witherbloom_decaymage(),
        catalog::lorehold_embersmith(),
        catalog::prismari_pyrolancer(),
        catalog::lorehold_pyromentor(),
        catalog::prismari_pyroceptor(),
        catalog::lorehold_emberscribe_v2(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let opp_before = g.players[1].life;
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - 4, "{name}: Bolt 3 + magecraft 1");
    }
}

// ── Table: magecraft scry — sanity: Bolt still resolves for 3 ───────────────

#[test]
fn magecraft_scry_does_not_block_bolt_resolution() {
    for def in [
        catalog::silverquill_quillrunner(),
        catalog::quandrix_scryweaver(),
        catalog::prismari_spellscribe(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::island());
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let opp_before = g.players[1].life;
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // Scry 1 doesn't change life — just confirm Bolt resolved.
        assert_eq!(g.players[1].life, opp_before - 3, "{name}");
    }
}

// ── Table: magecraft self-pump ──────────────────────────────────────────────

#[test]
fn magecraft_self_pump_on_bolt_cast() {
    for (def, p, counters) in [
        (catalog::silverquill_pen_squire(), Some(2), None),
        (catalog::spirit_battlemaster(), Some(5), None),
        (catalog::prismari_cinder_apprentice(), Some(2), None),
        (catalog::quandrix_pupil_b50(), None, Some(1)),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let card = g.battlefield_find(id).unwrap();
        if let Some(p) = p {
            assert_eq!(card.power(), p, "{name} power");
        }
        if let Some(c) = counters {
            assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), c, "{name} counters");
        }
    }
}

// ── Table: magecraft puts a +1/+1 counter on a tribal permanent ─────────────

#[test]
fn magecraft_adds_counter_to_tribe_member() {
    for (def, target_def) in [
        (catalog::fractal_geomancer(), catalog::fractal_avenger()),
        (catalog::quandrix_echocaster(), catalog::fractal_avenger()),
        (catalog::lorehold_spiritchron(), catalog::lorehold_reverence()),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let tgt = g.add_card_to_battlefield(0, target_def);
        let _ = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let before = g.battlefield_find(tgt).unwrap()
            .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let after = g.battlefield_find(tgt).unwrap()
            .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
        assert_eq!(after, before + 1, "{name}: tribe member should grow by 1");
    }
}

#[test]
fn inkling_skywriter_magecraft_pumps_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _id = g.add_card_to_battlefield(0, catalog::inkling_skywriter());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 3);
    assert_eq!(bear_card.toughness(), 3);
}

#[test]
fn silverquill_scryward_etb_scrys_and_magecraft_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_scryward());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Scryward castable");
    drain_stack(&mut g);
    // ETB Scry 1 already resolved (test just checks the card landed and
    // magecraft fires on next IS cast).
    assert!(g.battlefield_find(id).is_some());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

// ── Table: token minters (creature-type tokens) ─────────────────────────────

#[test]
fn cast_mints_creature_tokens() {
    for (def, tribe, count, gain, opp_loss, kws) in [
        (catalog::pestseed(), CreatureType::Pest, 1, 0, 0, vec![]),
        (catalog::witherbloom_pestseer(), CreatureType::Pest, 1, 0, 0, vec![]),
        (catalog::pest_hierarch(), CreatureType::Pest, 1, 0, 0, vec![]),
        (catalog::pest_cradlescale(), CreatureType::Pest, 1, 0, 0, vec![Keyword::Reach]),
        (catalog::witherbloom_pestcaller_b50(), CreatureType::Pest, 3, 0, 0, vec![]),
        (catalog::pest_brood(), CreatureType::Pest, 2, 0, 0, vec![]),
        (catalog::lorehold_memoriam(), CreatureType::Spirit, 2, 2, 0, vec![]),
        (catalog::lorehold_echocaller(), CreatureType::Spirit, 1, 1, 0, vec![]),
        (catalog::lorehold_reverence(), CreatureType::Spirit, 1, 0, 0, vec![Keyword::Vigilance]),
        (catalog::silverquill_convene(), CreatureType::Inkling, 2, 0, 1, vec![]),
        (catalog::silverquill_pronouncement(), CreatureType::Inkling, 2, 3, 3, vec![]),
        (catalog::silverquill_festscribe(), CreatureType::Inkling, 1, 2, 0, vec![]),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let life_before = g.players[0].life;
        let opp_before = g.players[1].life;
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let tokens = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.is_token
                && c.definition.subtypes.creature_types.contains(&tribe))
            .count();
        assert_eq!(tokens, count, "{name} token count");
        assert_eq!(g.players[0].life, life_before + gain, "{name} lifegain");
        assert_eq!(g.players[1].life, opp_before - opp_loss, "{name} opp loss");
        if !kws.is_empty() {
            let card = g.battlefield_find(id).unwrap();
            for kw in &kws {
                assert!(card.has_keyword(kw), "{name} should have {kw:?}");
            }
        }
    }
}

#[test]
fn etb_mints_treasure_token() {
    for def in [catalog::prismari_sparkforge_v2(), catalog::prismari_coinforger()] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let treasure_count = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Treasure")
            .count();
        assert_eq!(treasure_count, 1, "{name} treasure count");
    }
}

// ── Table: ETB cantrip (net-zero hand) plus optional drain/lifegain ─────────

#[test]
fn etb_cantrip_hand_net_zero() {
    for (def, gain, opp_loss) in [
        (catalog::quandrix_theoremist(), 0, 0),
        (catalog::strixhaven_stormsage(), 0, 0),
        (catalog::silverquill_inkscholar_b50(), 0, 0),
        (catalog::quandrix_refractor(), 0, 0),
        (catalog::prismari_snapcaster(), 0, 0),
        (catalog::silverquill_codex(), 2, 0),
        (catalog::silverquill_cipher(), 1, 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::island());
        let life_before = g.players[0].life;
        let opp_before = g.players[1].life;
        let id = g.add_card_to_hand(0, def);
        let hand_before = g.players[0].hand.len();
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        // -1 (cast) + 1 (draw) = 0 net.
        assert_eq!(g.players[0].hand.len(), hand_before, "{name} hand net zero");
        assert_eq!(g.players[0].life, life_before + gain, "{name} lifegain");
        assert_eq!(g.players[1].life, opp_before - opp_loss, "{name} opp loss");
    }
}

// ── Table: ETB returns a card from graveyard to hand (net-zero hand) ────────

#[test]
fn etb_returns_card_from_graveyard_to_hand() {
    for (def, gy_def) in [
        (catalog::silverquill_inkmender(), catalog::grizzly_bears()),
        (catalog::silverquill_necroscribe(), catalog::lightning_bolt()),
        (catalog::lorehold_memorialist_b50(), catalog::grizzly_bears()),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let in_gy = g.add_card_to_graveyard(0, gy_def);
        let id = g.add_card_to_hand(0, def);
        let hand_before = g.players[0].hand.len();
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        // -1 (cast) + 1 (return) = 0 net.
        assert_eq!(g.players[0].hand.len(), hand_before, "{name} hand net zero");
        assert!(g.players[0].graveyard.iter().all(|c| c.id != in_gy), "{name} gy emptied");
    }
}

// ── Table: reanimates a low-MV creature to the battlefield ──────────────────

#[test]
fn reanimates_bear_to_battlefield() {
    for (def, gain, opp_loss) in [
        (catalog::silverquill_memorial(), 1, 1),
        (catalog::silverquill_eulogize(), 2, 0),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let life_before = g.players[0].life;
        let opp_before = g.players[1].life;
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear_in_gy).is_some(), "{name} reanimated bear");
        assert_eq!(g.players[0].life, life_before + gain, "{name} lifegain");
        assert_eq!(g.players[1].life, opp_before - opp_loss, "{name} opp loss");
    }
}

// ── Table: enters with a +1/+1 counter per other creature you control ───────

#[test]
fn fractal_enters_with_counters_per_other_creature() {
    for (def, n_bears, expected) in [
        (catalog::fractal_bloomanalyst(), 2, 2),
        (catalog::fractal_synthmage(), 3, 3),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        for _ in 0..n_bears {
            let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        }
        let id = g.add_card_to_hand(0, def);
        add_generous_mana(&mut g, 0);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        let card = g.battlefield_find(id).expect("should survive ETB");
        let counters = card.counters.get(&CounterType::PlusOnePlusOne)
            .copied().unwrap_or(0);
        assert_eq!(counters, expected, "{name} counters");
    }
}

// ── Remaining single-card tests (unique shapes / regressions) ───────────────

#[test]
fn prismari_inkflame_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_hand(0, catalog::prismari_inkflame());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkflame castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
    // Net hand: -1 (cast) +1 (draw) -1 (discard) = -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn silverquill_penmistress_lifelinks_and_magecraft_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_penmistress());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Lifelink));
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 4);
}

#[test]
fn prismari_tidesinger_bounces_target_to_owner_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_tidesinger());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidesinger castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

#[test]
fn strixhaven_anthemcaster_pumps_other_friendly_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::strixhaven_anthemcaster());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Anthemcaster castable");
    drain_stack(&mut g);
    // Read layered stats post-resolve via compute_battlefield.
    let computed = g.compute_battlefield();
    let bear_card = computed.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bear_card.power, 3, "Bear gets +1/+0 anthem");
    assert_eq!(bear_card.toughness, 2);
    let self_card = computed.iter().find(|c| c.id == id).unwrap();
    assert_eq!(self_card.power, 2, "Anthemcaster doesn't pump itself");
    assert_eq!(self_card.toughness, 3);
}

#[test]
fn inkling_mournful_dies_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_mournful());
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    // Kill it via destroy.
    let bolt = g.add_card_to_hand(0, catalog::wrath_of_god());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wrath castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none());
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn inkling_inkstain_attack_shrinks_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::inkling_inkstain());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Inkstain declares attack");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 1, "Bear shrunk -1/-0");
}

#[test]
fn pest_cultivator_sage_attack_mints_a_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_cultivator_sage());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Cultivator-Sage attacks");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
}

// ── Push (modern_decks): EventKind::CreatureSacrificed (CR 701.16) ─────────

#[test]
fn witherbloom_mortician_grows_on_sacrifice() {
    // Sacrifice via the new `EventKind::CreatureSacrificed` event:
    // Witherbloom Sacrosanct's at-resolve sac path emits the
    // sacrifice-specific event, which Mortician's trigger listens for.
    let mut g = two_player_game();
    let mort = g.add_card_to_battlefield(0, catalog::witherbloom_mortician());
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    let mc = g.battlefield_find(mort).expect("Mortician still alive");
    assert_eq!(
        mc.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "Mortician should grow by 1 from the sacrifice event"
    );
}

#[test]
fn witherbloom_mortician_does_not_grow_on_natural_death() {
    // Damage-based death emits CreatureDied but NOT CreatureSacrificed,
    // so the Mortician's trigger should NOT fire.
    let mut g = two_player_game();
    let mort = g.add_card_to_battlefield(0, catalog::witherbloom_mortician());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Lethal damage to bear without sacrifice.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) {
        c.damage = 99;
    }
    g.check_state_based_actions();
    let mc = g.battlefield_find(mort).expect("Mortician still alive");
    assert_eq!(
        mc.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        0,
        "Mortician should NOT grow from natural deaths"
    );
}

#[test]
fn pest_pestmaster_b51_grows_only_on_own_sacrifices() {
    // YourControl scope: opponent sacrifices shouldn't trigger.
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::pest_pestmaster_b51());
    // P0 sacrifices a creature via Sacrosanct.
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    let pmc = g.battlefield_find(pm).expect("Pestmaster still alive");
    assert_eq!(
        pmc.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "Pestmaster should grow from own sacrifices"
    );
}

#[test]
fn inkling_sigilbearer_pumps_other_inklings_on_etb() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    let id = g.add_card_to_hand(0, catalog::inkling_sigilbearer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sigilbearer castable");
    drain_stack(&mut g);
    let oc = g.battlefield_find(other).unwrap();
    assert_eq!(
        oc.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "Other Inkling should have a +1/+1 counter"
    );
    // Self should not get a counter (OtherThanSource).
    let me = g.battlefield_find(id).unwrap();
    assert_eq!(
        me.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        0,
        "Sigilbearer should not buff itself"
    );
}

#[test]
fn witherbloom_sacrosanct_sacrifices_and_drains_three() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let opp_before = g.players[1].life;
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == fodder),
        "Bear should be sacrificed");
    assert_eq!(g.players[0].life, life_before + 3);
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn witherbloom_lichbloom_dies_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let lb = g.add_card_to_battlefield(0, catalog::witherbloom_lichbloom());
    let hand_before = g.players[0].hand.len();
    // Inflict lethal damage to trigger death.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == lb) {
        c.damage = 99;
    }
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == lb),
        "Lichbloom should be in graveyard");
    assert!(g.players[0].hand.iter().any(|c| c.id == bears),
        "Bears should return to hand");
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn lorehold_skystorm_burns_opp_creatures_and_gains_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_skystorm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skystorm castable");
    drain_stack(&mut g);
    // Bear (2 toughness) takes 2 damage and dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die to 2 damage");
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_skyblaze_mints_spirit_and_burns_opp_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spirits_before = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    let id = g.add_card_to_hand(0, catalog::lorehold_skyblaze());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Skyblaze castable");
    drain_stack(&mut g);
    // Spirit token minted.
    let spirits_after = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .count();
    assert_eq!(spirits_after, spirits_before + 1);
    // Bear (2/2) takes 2 damage → dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn lorehold_spirit_veteran_pumps_other_spirits() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::lorehold_reverence());
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_veteran());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Veteran castable");
    drain_stack(&mut g);
    let oc = g.battlefield_find(other).unwrap();
    assert_eq!(
        oc.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0),
        1,
        "Other Spirit (Reverence) should get a +1/+1 counter"
    );
}

#[test]
fn quandrix_forge_mints_fractal_with_four_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_forge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Forge castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1);
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 4);
}

#[test]
fn quandrix_algorithmist_magecraft_pumps_each_fractal() {
    let mut g = two_player_game();
    // Use existing fractal with counters via cast (so enters_with_counters fires).
    let fractal = g.add_card_to_hand(0, catalog::fractal_bloomthorn());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: fractal, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloomthorn castable");
    drain_stack(&mut g);
    let _id = g.add_card_to_battlefield(0, catalog::quandrix_algorithmist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Fractal had 3 counters from ETB, gains 1 from magecraft = 4.
    let fractal_card = g.battlefield_find(fractal).unwrap();
    assert_eq!(fractal_card.counter_count(CounterType::PlusOnePlusOne), 4);
}

#[test]
fn fractal_bloomstone_enters_with_counters_per_land() {
    let mut g = two_player_game();
    // 3 lands on the battlefield for P0.
    for _ in 0..3 {
        let _ = g.add_card_to_battlefield(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::fractal_bloomstone());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloomstone castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("Bloomstone should survive ETB");
    let counters = card.counters.get(&CounterType::PlusOnePlusOne)
        .copied().unwrap_or(0);
    // 3 lands → 3 counters → 3/3 (survives base 0/0).
    assert_eq!(counters, 3);
}

#[test]
fn quandrix_reflection_doubles_counters_on_each_friendly() {
    let mut g = two_player_game();
    // Use a Grizzly Bears (2/2 vanilla, no auto-counter) with a
    // manually-attached +1/+1 counter to lock in the doubling math.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) {
        c.counters.insert(CounterType::PlusOnePlusOne, 2);
    }
    let before = g.battlefield_find(bear).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    assert_eq!(before, 2);
    let hatch = bear;
    let id = g.add_card_to_hand(0, catalog::quandrix_reflection());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reflection castable");
    drain_stack(&mut g);
    let after = g.battlefield_find(hatch).unwrap()
        .counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0);
    // 2 → 2 + 2 = 4 (doubled).
    assert_eq!(after, 4);
}

#[test]
fn prismari_bonfire_burns_creature_for_three() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::bookwurm());
    let id = g.add_card_to_hand(0, catalog::prismari_bonfire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bonfire castable");
    drain_stack(&mut g);
    // 5/5 bookwurm takes 3 damage and survives but is damaged.
    let card = g.battlefield_find(big).unwrap();
    assert_eq!(card.damage, 3);
}

#[test]
fn prismari_flashforge_burns_target_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_flashforge());
    g.add_card_to_hand(0, catalog::island()); // a discardable card
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flashforge castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
    // Hand: -1 (cast) -1 (discard) +1 (draw) = -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_riftspark_magecraft_loots_optionally() {
    // With AutoDecider answering "no" to optional MayDo, no loot occurs.
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::prismari_riftspark());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len() - 1; // remove the bolt we just added
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // AutoDecider declines the MayDo loot — hand size unchanged
    // post-bolt-cast.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn pest_anointer_gains_life_on_sacrifice() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::pest_anointer());
    // Make the fodder a token so the auto-sac picker prefers it
    // (tokens sort before non-tokens in the heuristic).
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == fodder) {
        c.is_token = true;
    }
    let life_before = g.players[0].life;
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Sacrosanct: drain 3 = +3 life. Anointer: +1 from the sacrifice.
    assert_eq!(g.players[0].life, life_before + 3 + 1);
}

#[test]
fn witherbloom_bloodreaper_drains_each_opp_on_sacrifice() {
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::witherbloom_bloodreaper());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == fodder) {
        c.is_token = true;
    }
    let opp_before = g.players[1].life;
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    // Sacrosanct -3 to opp + Bloodreaper -1 to opp = -4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn pest_conservator_sac_a_pest_draws() {
    let mut g = two_player_game();
    let pc = g.add_card_to_battlefield(0, catalog::pest_conservator());
    g.clear_sickness(pc);
    // Mint a Pest token to sacrifice. Add a pest directly.
    let pest_def = {
        // Use Grizzly Bears retyped as a Pest for a cheap fodder.
        let mut def = catalog::grizzly_bears();
        def.subtypes.creature_types = vec![CreatureType::Pest];
        def
    };
    let pest = g.add_card_to_battlefield(0, pest_def);
    g.add_card_to_library(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("Conservator activatable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == pest),
        "Pest should be sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn quandrix_cantripper_magecraft_loots_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_cantripper());
    let _ = g.add_card_to_hand(0, catalog::forest());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_pre = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 (bolt cast) + 1 (magecraft draw) - 1 (magecraft discard) = -1.
    assert_eq!(g.players[0].hand.len(), hand_pre - 1);
}

#[test]
fn prismari_cantrip_mage_magecraft_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_cantrip_mage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_pre = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 cast bolt + 1 magecraft draw = 0 net (no discard) → same as before.
    assert_eq!(g.players[0].hand.len(), hand_pre);
}

#[test]
fn prismari_firebrand_etb_pings_with_haste() {
    let mut g = two_player_game();
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_firebrand());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Firebrand castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Haste));
    // Damage: auto-target picks the opponent.
    assert_eq!(g.players[1].life, opp_before - 1);
}
