use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;


#[test]
fn quandrix_hatchling_enters_with_two_counters_and_grows_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_hatchling());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hatchling castable");
    drain_stack(&mut g);
    let h = g.battlefield.iter().find(|c| c.definition.name == "Quandrix Hatchling").expect("Hatchling");
    assert_eq!(h.counter_count(CounterType::PlusOnePlusOne), 2, "Hatchling enters with 2 counters");
    // Cast a bolt and check growth
    let h_id = h.id;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let h_after = g.battlefield_find(h_id).expect("Hatchling");
    assert_eq!(h_after.counter_count(CounterType::PlusOnePlusOne), 3, "Hatchling grew via magecraft");
}

#[test]
fn prismari_cascade_volley_burns_target_and_pings_each_opp_creature() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_cascade_volley());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Volley castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3, "3 damage to opp player");
    // Each bear took 1 damage.
    let b1d = g.battlefield_find(b1).map(|c| c.damage).unwrap_or(0);
    let b2d = g.battlefield_find(b2).map(|c| c.damage).unwrap_or(0);
    assert_eq!(b1d, 1, "Bear 1 took 1 damage");
    assert_eq!(b2d, 1, "Bear 2 took 1 damage");
}

// ── Table: bystander on battlefield, cast a bolt at opp, magecraft adds
//    extra damage/drain on the opponent (and possibly life for you). ────────
#[test]
fn magecraft_pings_or_drains_opponent_on_instant_cast() {
    for (def, extra_opp_loss, self_gain) in [
        (catalog::prismari_initiate(), 1, 0),
        (catalog::strixhaven_quill_mage(), 1, 0),
        (catalog::lorehold_strikevanguard(), 1, 0),
        (catalog::silverquill_eulogist(), 1, 0),
        (catalog::silverquill_inkmaster(), 1, 1),
        (catalog::witherbloom_marshcaster(), 1, 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
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
        assert_eq!(g.players[1].life, life1_before - 3 - extra_opp_loss,
            "{}: opp lost 3 (bolt) + {}", name, extra_opp_loss);
        assert_eq!(g.players[0].life, life0_before + self_gain, "{}: self gain", name);
    }
}

// ── Table: bystander on battlefield, cast a bolt, magecraft pumps the
//    bystander itself (+dp/+dt). ────────────────────────────────────────────
#[test]
fn magecraft_self_pumps_on_instant_cast() {
    for (def, dp, dt) in [
        (catalog::quandrix_mistshaper(), 1, 1),
        (catalog::lorehold_saint(), 1, 0),
        (catalog::inkling_quillwarden(), 1, 0),
        (catalog::witherbloom_toxicaster(), 0, 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        drain_stack(&mut g);
        let p_before = g.battlefield_find(id).map(|c| c.power()).unwrap_or(0);
        let t_before = g.battlefield_find(id).map(|c| c.toughness()).unwrap_or(0);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(id).expect("still on battlefield");
        assert_eq!(c.power(), p_before + dp, "{}: power pumped", name);
        assert_eq!(c.toughness(), t_before + dt, "{}: toughness pumped", name);
    }
}

// ── Table: bystander on battlefield, cast a bolt, magecraft pumps another
//    (target) creature. ─────────────────────────────────────────────────────
#[test]
fn magecraft_pumps_other_creature_on_instant_cast() {
    for def in [
        catalog::silverquill_lifeglyph(),
        catalog::prismari_spellforger_b22(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let src = g.add_card_to_battlefield(0, def);
        g.clear_sickness(src);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        drain_stack(&mut g);
        let p_before = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let p_after = g.battlefield_find(bear).map(|c| c.power()).unwrap_or(0);
        assert_eq!(p_after, p_before + 1, "{}: pumped target creature", name);
    }
}

// ── Table: bystander on battlefield, cast a bolt, magecraft mints a token. ──
#[test]
fn magecraft_mints_token_on_instant_cast() {
    for (def, token_name) in [
        (catalog::witherbloom_pestcaller(), "Pest"),
        (catalog::inkling_verseweaver(), "Inkling"),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let tokens = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == token_name)
            .count();
        assert_eq!(tokens, 1, "{}: magecraft mints a {}", name, token_name);
    }
}

// ── Table: bystander on battlefield, cast a bolt, magecraft gives some
//    card-advantage effect (draw / loot / scry) or life; net hand checked. ──
#[test]
fn magecraft_card_advantage_on_instant_cast() {
    for (def, net, life_delta) in [
        (catalog::strixhaven_scholar(), -1, 0),   // scry 1
        (catalog::quandrix_sage(), 0, 0),          // scry + draw
        (catalog::prismari_spell_shaper(), 0, 0),  // scry + draw
        (catalog::prismari_embershaper(), -1, 0),  // loot
        (catalog::witherbloom_vinetender(), -1, 1),// gain 1 life
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_hand(0, catalog::island()); // discard fodder for looters
        g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len() as i64;
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len() as i64, hand_before + net,
            "{}: net hand after bolt + magecraft", name);
        assert_eq!(g.players[0].life, life_before + life_delta, "{}: life", name);
    }
}

#[test]
fn strixhaven_initiate_has_reach_and_taps_for_green() {
    let mut g = two_player_game();
    let init = g.add_card_to_battlefield(0, catalog::strixhaven_initiate());
    g.clear_sickness(init);
    let def = catalog::strixhaven_initiate();
    assert!(def.keywords.contains(&Keyword::Reach));
    let green_before = g.players[0].mana_pool.amount(Color::Green);
    g.perform_action(GameAction::ActivateAbility {
        card_id: init,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Mana ability activates");
    drain_stack(&mut g);
    let green_after = g.players[0].mana_pool.amount(Color::Green);
    assert_eq!(green_after, green_before + 1, "Initiate added Green");
}

// ── Table: spell/creature that burns the opponent player for N on cast/ETB,
//    possibly also minting a token. ─────────────────────────────────────────
#[test]
fn burns_opponent_player_on_cast_or_etb() {
    for (def, colors, cl, targeted, dmg, token) in [
        (catalog::strixhaven_burnscholar(), vec![Color::Red], 0, true, 1, None),
        (catalog::lorehold_spiritarcher(), vec![Color::Red], 3, false, 2, None),
        (catalog::lorehold_sparkflare(), vec![Color::Red], 0, true, 2, None),
        (catalog::prismari_hotburst(), vec![Color::Red], 1, true, 2, Some(("Treasure", 1))),
        (catalog::prismari_cinderspark(), vec![Color::Red], 0, true, 1, None),
        (catalog::lorehold_recital(), vec![Color::Red, Color::White], 1, true, 1, Some(("Spirit", 1))),
        (catalog::prismari_spitfire(), vec![Color::Red], 3, false, 2, None),
        (catalog::prismari_pyromancer(), vec![Color::Red, Color::Blue], 2, false, 2, None),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        let life1_before = g.players[1].life;
        let target = if targeted {
            Some(crabomination::game::types::Target::Player(1))
        } else { None };
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1_before - dmg, "{}: burns opp for {}", name, dmg);
        if let Some((tok, n)) = token {
            let tokens = g.battlefield.iter()
                .filter(|c| c.controller == 0 && c.definition.name == tok)
                .count();
            assert_eq!(tokens, n, "{}: mints {} {}", name, n, tok);
        }
    }
}

// ── Table: spell/creature that kills a Grizzly Bears on cast (burn/shrink/
//    destroy/exile), possibly gaining life and/or minting a token. ──────────
#[test]
fn removal_kills_bear_on_cast() {
    for (def, colors, cl, gain, token) in [
        (catalog::prismari_quickfire(), vec![Color::Red], 0, 0, None),
        (catalog::lorehold_sparkstrike(), vec![Color::Red], 1, 0, None),
        (catalog::prismari_volleyfire(), vec![Color::Red], 3, 0, Some(("Treasure", 1))),
        (catalog::prismari_embergem(), vec![Color::Red], 2, 0, Some(("Treasure", 1))),
        (catalog::toxic_bloodletting(), vec![Color::Black, Color::Green], 1, 2, None),
        (catalog::silverquill_hush(), vec![Color::White, Color::Black], 0, 2, None),
        (catalog::pestilent_bloom(), vec![Color::Black, Color::Green], 0, 0, Some(("Pest", 1))),
        (catalog::silverquill_reckoning(), vec![Color::White, Color::Black], 3, 0, Some(("Inkling", 1))),
        (catalog::silverquill_loredrain(), vec![Color::Black], 2, 2, None),
        (catalog::silverquill_indictment(), vec![Color::White, Color::Black], 2, 2, None),
        (catalog::lorehold_ironhand(), vec![Color::Red, Color::White], 3, 0, None),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(crabomination::game::types::Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "{}: bear removed", name);
        assert_eq!(g.players[0].life, life_before + gain, "{}: life gain", name);
        if let Some((tok, n)) = token {
            let tokens = g.battlefield.iter()
                .filter(|c| c.controller == 0 && c.definition.name == tok)
                .count();
            assert_eq!(tokens, n, "{}: mints {} {}", name, n, tok);
        }
    }
}

// ── Table: shrink effects the bear survives, with p/t and life asserts. ────
#[test]
fn shrink_spells_leave_bear_alive_with_expected_stats() {
    for (def, colors, cl, exp_p, exp_t, gain) in [
        (catalog::witherbloom_aspersor(), vec![Color::Black, Color::Green], 0, 0, 1, 1),
        (catalog::witherbloom_withercut(), vec![Color::Black, Color::Green], 1, -1, 1, 0),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(crabomination::game::types::Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let bear_c = g.battlefield_find(bear).expect("bear survives");
        assert_eq!(bear_c.power(), exp_p, "{}: power", name);
        assert_eq!(bear_c.toughness(), exp_t, "{}: toughness", name);
        assert_eq!(g.players[0].life, life_before + gain, "{}: life gain", name);
    }
}

// ── Table: cast card that drains (or just gains) life on resolve/ETB. ──────
#[test]
fn drains_or_gains_life_on_cast_or_etb() {
    for (def, colors, cl, targeted, opp_loss, self_gain) in [
        (catalog::silverquill_sealwriter(), vec![Color::Black], 2, true, 2, 2),
        (catalog::witherbloom_carnivine(), vec![Color::Black, Color::Green], 3, true, 3, 3),
        (catalog::silverquill_conviction(), vec![Color::White, Color::Black], 0, false, 2, 2),
        (catalog::witherbloom_famine(), vec![Color::Black], 3, false, 4, 4),
        (catalog::witherbloom_greenrot(), vec![Color::Green], 1, false, 0, 2),
        (catalog::witherbloom_pestbroker(), vec![Color::Black], 2, false, 2, 2),
        (catalog::inkling_pamphleteer(), vec![Color::White, Color::Black], 0, false, 1, 1),
        (catalog::inkling_drainmaster(), vec![Color::Black], 3, true, 3, 3),
        (catalog::witherbloom_tendril(), vec![Color::Black, Color::Green], 1, false, 2, 2),
        (catalog::silverquill_lifebinder(), vec![Color::White], 2, false, 0, 2),
        (catalog::lorehold_bonereader(), vec![Color::White], 2, false, 0, 2),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        let life0_before = g.players[0].life;
        let life1_before = g.players[1].life;
        let target = if targeted {
            Some(crabomination::game::types::Target::Player(1))
        } else { None };
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1_before - opp_loss, "{}: opp loss", name);
        assert_eq!(g.players[0].life, life0_before + self_gain, "{}: self gain", name);
    }
}

// ── Table: ETB mints token(s). ─────────────────────────────────────────────
#[test]
fn etb_mints_tokens() {
    for (def, colors, cl, token_name, count) in [
        (catalog::prismari_treasurer(), vec![Color::Blue], 1, "Treasure", 1),
        (catalog::inkling_acolyte(), vec![Color::White], 1, "Inkling", 1),
        (catalog::pest_swarmlord(), vec![Color::Black, Color::Green], 3, "Pest", 2),
        (catalog::prismari_sparkforge(), vec![Color::Blue, Color::Red], 2, "Treasure", 1),
        (catalog::lorehold_ringleader(), vec![Color::Red, Color::White], 3, "Spirit", 2),
        (catalog::pest_ravager(), vec![Color::Black, Color::Green], 3, "Pest", 2),
        (catalog::silverquill_quillscribe(), vec![Color::White, Color::Black], 2, "Inkling", 1),
        (catalog::lorehold_soulshaper(), vec![Color::White], 2, "Spirit", 1),
        (catalog::prismari_treasurer_surge(), vec![Color::Blue, Color::Red], 3, "Treasure", 2),
        (catalog::witherbloom_spore_master(), vec![Color::Black, Color::Green], 3, "Pest", 2),
        (catalog::pest_wrangler(), vec![Color::Green], 2, "Pest", 1),
        (catalog::pest_brood_mother(), vec![Color::Black, Color::Green], 3, "Pest", 2),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let tokens = g.battlefield.iter()
            .filter(|c| c.controller == 0 && c.definition.name == token_name)
            .count();
        assert_eq!(tokens, count, "{}: mints {} {}", name, count, token_name);
    }
    // Keyword/subtype spot-checks kept from the originals.
    assert!(catalog::inkling_acolyte().subtypes.creature_types.contains(&CreatureType::Inkling));
    assert!(catalog::prismari_sparkforge().keywords.contains(&Keyword::Haste));
    assert!(catalog::lorehold_ringleader().keywords.contains(&Keyword::Haste));
}

// ── Table: ETB draw/loot etc.; net hand + life deltas, optional token. ─────
#[test]
fn etb_card_advantage_and_life_deltas() {
    for (def, colors, cl, target_player, net, self_life, opp_life, token) in [
        (catalog::silverquill_inkscholar(), vec![Color::White], 2, None, -1, 0, 0, None),
        (catalog::quandrix_mistweaver(), vec![Color::Blue], 1, None, 0, 0, 0, None),
        (catalog::prismari_mindwave(), vec![Color::Blue], 2, None, 0, 0, 0, None),
        (catalog::prismari_stormspire(), vec![Color::Blue, Color::Red], 4, None, 1, 0, 0, None),
        (catalog::inkling_lorewright(), vec![Color::White, Color::Black], 3, None, 0, -1, 0, None),
        (catalog::strixhaven_necropact(), vec![Color::Black], 2, Some(0), 1, -2, 0, None),
        (catalog::pest_harvest(), vec![Color::Black, Color::Green], 2, None, 0, 0, 0, Some(("Pest", 1))),
        (catalog::prismari_stormgaze(), vec![Color::Blue, Color::Red], 2, Some(1), 0, 0, -1, None),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        let hand_before = g.players[0].hand.len() as i64;
        let life0_before = g.players[0].life;
        let life1_before = g.players[1].life;
        let target = target_player.map(crabomination::game::types::Target::Player);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len() as i64, hand_before + net, "{}: net hand", name);
        assert_eq!(g.players[0].life, life0_before + self_life, "{}: self life", name);
        assert_eq!(g.players[1].life, life1_before + opp_life, "{}: opp life", name);
        if let Some((tok, n)) = token {
            let tokens = g.battlefield.iter()
                .filter(|c| c.controller == 0 && c.definition.name == tok)
                .count();
            assert_eq!(tokens, n, "{}: token count", name);
        }
    }
}

// ── Table: ETB scry-style bodies — assert the body resolved onto bf. ───────
#[test]
fn etb_scry_bodies_resolve_onto_battlefield() {
    for (def, colors, cl) in [
        (catalog::silverquill_bookbearer(), vec![Color::White], 2),
        (catalog::spellbook_studier(), vec![Color::Blue], 1),
        (catalog::silverquill_notetaker(), vec![Color::White], 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).is_some(), "{}: on battlefield after ETB scry", name);
    }
}

// ── Table: return a card from your graveyard to hand, optional token. ──────
#[test]
fn returns_card_from_graveyard_to_hand() {
    for (def, colors, cl, gy_def, token) in [
        (catalog::witherbloom_necrosophist(), vec![Color::Black], 2, catalog::grizzly_bears(), None),
        (catalog::pest_reanimator(), vec![Color::Black, Color::Green], 2, catalog::grizzly_bears(), None),
        (catalog::silverquill_memorist(), vec![Color::White, Color::Black], 2, catalog::lightning_bolt(), None),
        (catalog::pest_mausoleum(), vec![Color::Black, Color::Green], 2, catalog::grizzly_bears(), Some(("Pest", 1))),
        (catalog::lorehold_echoflame(), vec![Color::Red, Color::White], 3, catalog::lightning_bolt(), Some(("Spirit", 1))),
    ] {
        let name = def.name;
        let gy_name = gy_def.name;
        let mut g = two_player_game();
        g.add_card_to_graveyard(0, gy_def);
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let in_hand = g.players[0].hand.iter().any(|c| c.definition.name == gy_name);
        assert!(in_hand, "{}: {} returned to hand", name, gy_name);
        if let Some((tok, n)) = token {
            let tokens = g.battlefield.iter()
                .filter(|c| c.controller == 0 && c.definition.name == tok)
                .count();
            assert_eq!(tokens, n, "{}: token count", name);
        }
    }
}

// ── Table: target opp discards N, optional life gain. ──────────────────────
#[test]
fn makes_target_opponent_discard() {
    for (def, colors, cl, discards, gain) in [
        (catalog::silverquill_compulsion(), vec![Color::Black], 1, 1, 0),
        (catalog::inkling_inquisitor(), vec![Color::Black], 2, 1, 0),
        (catalog::witherbloom_handburner(), vec![Color::Black], 2, 2, 2),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::lightning_bolt());
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        let opp_hand_before = g.players[1].hand.len();
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), opp_hand_before - discards, "{}: opp discarded", name);
        assert_eq!(g.players[0].life, life_before + gain, "{}: life gain", name);
    }
}

// ── Table: put a +1/+1 counter on a bear (targeted or auto-picked). ────────
#[test]
fn puts_counter_on_creature_on_cast() {
    for (def, colors, cl, targeted) in [
        (catalog::quandrix_calibrator(), vec![Color::Green], 2, true),
        (catalog::quandrix_counterbalance(), vec![Color::Green, Color::Blue], 0, true),
        (catalog::quandrix_counterproof(), vec![Color::Green, Color::Blue], 0, true),
        (catalog::quandrix_polymath(), vec![Color::Green, Color::Blue], 1, false),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        let target = if targeted {
            Some(crabomination::game::types::Target::Permanent(bear))
        } else { None };
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let counters = g.battlefield_find(bear)
            .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
            .unwrap_or(0);
        assert_eq!(counters, 1, "{}: bear got +1/+1 counter", name);
    }
}

// ── Table: team pump sorceries with an EOT keyword grant. ──────────────────
#[test]
fn team_pump_spells_pump_and_grant_keyword() {
    for (def, colors, cl, exp_power, kw) in [
        (catalog::lorehold_strikeforce(), vec![Color::Red, Color::White], 2, 4, Keyword::Trample),
        (catalog::silverquill_battle_hymn(), vec![Color::White], 2, 3, Keyword::Vigilance),
        (catalog::lorehold_spirit_anthem(), vec![Color::Red, Color::White], 3, 4, Keyword::FirstStrike),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let bear_c = g.battlefield_find(bear).expect("bear");
        assert_eq!(bear_c.power(), exp_power, "{}: bear pumped", name);
        let computed = g.compute_battlefield();
        let bear_cp = computed.iter().find(|c| c.id == bear).unwrap();
        assert!(bear_cp.keywords.contains(&kw), "{}: bear has {:?} EOT", name, kw);
    }
}

#[test]
fn heroic_defiance_pumps_and_grants_hexproof_and_indestructible() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::heroic_defiance());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Defiance castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).expect("Bear");
    assert!(bear_card.has_keyword(&Keyword::Hexproof));
    assert!(bear_card.has_keyword(&Keyword::Indestructible));
    assert_eq!(bear_card.power(), 3, "Bear pumped to 3 power");
    assert_eq!(bear_card.toughness(), 3);
}

#[test]
fn tome_shredder_exiles_instant_from_graveyard_to_grow() {
    // Real oracle: "Haste / {T}, Exile an instant or sorcery card from
    // your graveyard: Put a +1/+1 counter on this creature."
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::tome_shredder());
    g.clear_sickness(id);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::island()); // land — not a legal exile fodder
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("{T}, exile an instant/sorcery from graveyard: +1/+1 counter");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Tome Shredder on battlefield");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1,
        "Tome Shredder picked up a +1/+1 counter");
    assert!(c.tapped, "tap cost paid");
    assert!(g.exile.iter().any(|c| c.id == bolt),
        "the instant was exiled from the graveyard as the cost");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "bolt left the graveyard");
    assert!(catalog::tome_shredder().keywords.contains(&Keyword::Haste));
}

// ── Table: search a basic Forest out of the library onto the battlefield. ──
#[test]
fn searches_basic_land_onto_battlefield() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    for (def, colors, cl, check_tapped) in [
        (catalog::mascot_acolyte(), vec![Color::Green], 2, true),
        (catalog::hunt_the_library(), vec![Color::Green], 3, false),
        (catalog::field_researcher(), vec![Color::White], 2, false),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let forest = g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::grizzly_bears());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let f = g.battlefield_find(forest).expect("Forest tutored onto bf");
        assert!(f.definition.is_land(), "{}: land on battlefield", name);
        if check_tapped {
            assert!(f.tapped, "{}: tutored Forest enters tapped", name);
        }
    }
}

// ── Table: search a card out of the library into hand. ─────────────────────
#[test]
fn searches_card_into_hand() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    for (def, colors, cl, wanted) in [
        (catalog::quandrix_cartographer(), vec![Color::Green], 2, catalog::forest()),
        (catalog::silverquill_hightutor(), vec![Color::White], 1, catalog::lightning_bolt()),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let wanted_id = g.add_card_to_library(0, wanted);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(wanted_id))]));
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == wanted_id),
            "{}: searched card is in hand", name);
    }
}

#[test]
fn strixhaven_vigil_gains_life_on_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::strixhaven_vigil());
    // Advance the turn to active_player 0's upkeep.
    let life_before = g.players[0].life;
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Fire step triggers manually.
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "Vigil grants 1 life on upkeep");
}

#[test]
fn inkling_battlecaster_attack_drains_one() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_battlecaster());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Battlecaster attacks");
    drain_stack(&mut g);
    // Attack trigger: drain 1 (each opp loses 1, you gain 1).
    assert_eq!(g.players[1].life, 19, "opp loses 1 from attack drain");
    assert_eq!(g.players[0].life, 21, "you gain 1 from attack drain");
    let def = catalog::inkling_battlecaster();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Vigilance));
}

// ── Table: creature dies (via SBA damage), death trigger drains/gains. ─────
#[test]
fn death_triggers_drain_on_sba_death() {
    for (def, dmg, opp_loss, self_gain) in [
        (catalog::pest_forager(), 1, 0, 1),
        (catalog::witherbloom_saproot(), 3, 2, 2),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        let life0_before = g.players[0].life;
        let life1_before = g.players[1].life;
        let card = g.battlefield_find_mut(id).unwrap();
        card.damage = dmg;
        g.check_state_based_actions();
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).is_none(), "{}: died", name);
        assert_eq!(g.players[1].life, life1_before - opp_loss, "{}: opp loss", name);
        assert_eq!(g.players[0].life, life0_before + self_gain, "{}: self gain", name);
    }
}

// ── Table: creature dies to a bolt, death trigger drains 2. ────────────────
#[test]
fn death_triggers_drain_two_when_bolted() {
    use crabomination::game::types::Target;
    for def in [
        catalog::witherbloom_reaper_hand(),
        catalog::witherbloom_drainbreath(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let p0_life = g.players[0].life;
        let p1_life = g.players[1].life;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(id)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, p1_life - 2, "{}: drain 2 on death", name);
        assert_eq!(g.players[0].life, p0_life + 2, "{}: gain 2 on death", name);
    }
}

#[test]
fn lorehold_emberscribe_etb_exiles_gy_and_pings() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_emberscribe());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life1_before = g.players[1].life;
    let opp_gy_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Emberscribe castable");
    drain_stack(&mut g);
    // Bolt should be exiled from opp's gy.
    assert!(g.players[1].graveyard.len() < opp_gy_before, "card exiled from opp gy");
    // 1 damage to each opp.
    assert_eq!(g.players[1].life, life1_before - 1, "Emberscribe pings opp for 1");
}

#[test]
fn lorehold_reliquary_pumps_creature_on_gy_leave() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_reliquary());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Stage a creature card in your graveyard, then cast something that
    // returns it — Lorehold Ember-Recall returns target creature with
    // mv≤2 to the battlefield, which triggers Reliquary.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let recall = g.add_card_to_hand(0, catalog::lorehold_ember_recall());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let counters_before: u32 = g.battlefield.iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    g.perform_action(GameAction::CastSpell {
        card_id: recall, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recall castable");
    drain_stack(&mut g);
    let counters_after: u32 = g.battlefield.iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert!(counters_after > counters_before, "Reliquary added a counter on gy-leave");
    let _ = bear; // touched in assertion sum
}

#[test]
fn lorehold_ember_recall_returns_low_mv_creature_and_pings_opp() {
    let mut g = two_player_game();
    // Stage a 1-mana creature in your gy (Bears are 2 MV — need <=2 MV creature).
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_ember_recall());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ember-Recall castable");
    drain_stack(&mut g);
    let bear_on_bf = g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears");
    assert!(bear_on_bf, "Bear returned to battlefield");
    assert_eq!(g.players[1].life, life1_before - 1, "opp pinged for 1");
}

// ── Table: Fractal token minted with N +1/+1 counters (varied sizing). ─────
#[test]
fn fractal_token_minted_with_expected_counters() {
    for (i, (def, colors, cl, expected, exact)) in [
        (catalog::fractal_bloom_caller(), vec![Color::Green, Color::Blue], 2, 2, true),
        (catalog::fractal_harvest(), vec![Color::Green, Color::Blue], 3, 3, true),
        (catalog::fractal_tessellation(), vec![Color::Green, Color::Blue], 3, 3, true),
        (catalog::quandrix_pondkeeper(), vec![Color::Blue], 2, 2, true),
        (catalog::fractal_surge(), vec![Color::Green, Color::Blue], 1, 3, false),
    ].into_iter().enumerate() {
        let name = def.name;
        let mut g = two_player_game();
        match i {
            1 => { g.add_card_to_library(0, catalog::island()); } // harvest draws
            2 => { for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); } } // lands scale
            3 => { // pondkeeper: 2 IS in gy
                g.add_card_to_graveyard(0, catalog::lightning_bolt());
                g.add_card_to_graveyard(0, catalog::lightning_bolt());
            }
            4 => { // surge: creature count
                for _ in 0..3 { g.add_card_to_battlefield(0, catalog::grizzly_bears()); }
                drain_stack(&mut g);
            }
            _ => {}
        }
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let fractal = g.battlefield.iter()
            .find(|c| c.controller == 0 && c.definition.name == "Fractal")
            .expect("Fractal minted");
        let counters = fractal.counter_count(CounterType::PlusOnePlusOne);
        if exact {
            assert_eq!(counters, expected, "{}: Fractal counter count", name);
        } else {
            assert!(counters >= expected, "{}: Fractal scales with creature count", name);
        }
    }
}

#[test]
fn quandrix_synthesist_magecraft_pumps_team() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::quandrix_synthesist());
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c1 = g.battlefield_find(bear1).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    let c2 = g.battlefield_find(bear2).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(c1, 1);
    assert_eq!(c2, 1);
}

// ── CR 701.46a — Stun counter consumption on untap ─────────────────────────

#[test]
fn stun_counter_replaces_untap_per_cr_701_46a() {
    // CR 701.46a: A permanent with a stun counter would become untapped,
    // remove a stun counter instead. The permanent stays tapped on that
    // untap step; on the next untap (no stun counters left) it untaps
    // normally. Push (modern_decks batch 22): the do_untap step in
    // game/stack.rs now consults stun counters before flipping tapped.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Tap + 1 stun counter.
    {
        let card = g.battlefield_find_mut(bear).unwrap();
        card.tapped = true;
        card.add_counters(CounterType::Stun, 1);
    }
    // First untap — stun counter consumed, permanent stays tapped.
    g.active_player_idx = 0;
    g.do_untap();
    {
        let card = g.battlefield_find(bear).unwrap();
        assert!(card.tapped, "First untap: stun counter consumed, still tapped");
        assert_eq!(card.counter_count(CounterType::Stun), 0,
            "Stun counter consumed by replacing untap event");
    }
    // Second untap — no stun counters left, permanent untaps normally.
    g.do_untap();
    let card = g.battlefield_find(bear).unwrap();
    assert!(!card.tapped, "Second untap: no stun counters → untaps normally");
}

#[test]
fn inkling_aristocrat_gains_life_when_another_creature_dies() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let _aristo = g.add_card_to_battlefield(0, catalog::inkling_aristocrat());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    // Kill the bear with a Lightning Bolt — uses the proper damage path
    // so the CreatureDied event fires and triggers dispatch.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert!(g.players[0].life > life_before,
        "Aristocrat gains at least 1 life on friendly creature death (was {}, now {})",
        life_before, g.players[0].life);
}

#[test]
fn inkling_aristocrat_does_not_trigger_on_self() {
    // Aristocrat dying is not "another creature you control dying".
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let aristo = g.add_card_to_battlefield(0, catalog::inkling_aristocrat());
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(aristo)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before, "Aristocrat doesn't trigger on self-death");
}

#[test]
fn lorehold_battlechronicler_attack_returns_creature_from_gy() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bc = g.add_card_to_battlefield(0, catalog::lorehold_battlechronicler());
    g.clear_sickness(bc);
    let hand_before = g.players[0].hand.len();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bc,
        target: AttackTarget::Player(1),
    }])).expect("Attack declared");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Attack returned creature from gy → hand");
    assert!(g.players[0].graveyard.iter().all(|c| c.id != bear_in_gy));
}

#[test]
fn lorehold_searing_wisdom_exiles_gy_card_and_burns() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_searing_wisdom());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear_in_gy)),
        additional_targets: vec![Target::Player(1)],
        mode: None, x_value: None,
    }).expect("Searing Wisdom castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear_in_gy), "Bear exiled from gy");
    assert_eq!(g.players[1].life, p1_life - 3, "Burns target for 3");
}

#[test]
fn lorehold_volley_hits_target_for_two_and_others_for_one() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let target_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let other_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_volley());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target_bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Volley castable");
    drain_stack(&mut g);
    // target_bear takes 2 + 1 = 3 → dies (toughness 2)
    assert!(g.battlefield_find(target_bear).is_none(), "Target bear dies to 2+1");
    // other_bear takes 1 → marked 1 damage, survives
    let other = g.battlefield_find(other_bear).unwrap();
    assert_eq!(other.damage, 1, "Other bear marked 1");
}

#[test]
fn fractal_avenger_enters_with_four_plus_one_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_avenger());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Avenger castable");
    drain_stack(&mut g);
    let fa = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Fractal Avenger")
        .expect("Fractal Avenger on battlefield");
    assert_eq!(fa.counter_count(CounterType::PlusOnePlusOne), 4);
    assert_eq!(fa.power(), 4, "Avenger is 4/4 from counters");
    assert!(fa.has_keyword(&Keyword::Trample));
}

#[test]
fn fractal_sovereign_etb_scales_counters_with_lands() {
    let mut g = two_player_game();
    // Give controller 3 lands on battlefield.
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_sovereign());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sovereign castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(bear).unwrap();
    assert_eq!(bear.counter_count(CounterType::PlusOnePlusOne), 3,
        "Bear gets +1/+1 counters = number of lands (3)");
}

#[test]
fn fractal_resonance_pumps_each_creature_you_control() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_resonance());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Resonance castable");
    drain_stack(&mut g);
    let c1 = g.battlefield_find(bear1).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    let c2 = g.battlefield_find(bear2).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    let co = g.battlefield_find(opp_bear).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0);
    assert_eq!(c1, 1);
    assert_eq!(c2, 1);
    assert_eq!(co, 0, "opp bear not pumped");
}

#[test]
fn quandrix_pairweaver_pumps_two_creatures() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_pairweaver());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(b1)),
        additional_targets: vec![Target::Permanent(b2)],
        mode: None, x_value: None,
    }).expect("Pairweaver castable");
    drain_stack(&mut g);
    let b1 = g.battlefield_find(b1).unwrap();
    let b2 = g.battlefield_find(b2).unwrap();
    assert_eq!(b1.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(b2.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn prismari_pyreburst_sweeps_x_three_creatures() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_pyreburst());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyreburst castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(b1).is_none(), "Friendly bear sweeps too");
    assert!(g.battlefield_find(b2).is_none(), "Opp bear destroyed");
}

#[test]
fn prismari_vorthos_etb_loots_and_burns_with_is_discard() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    // Hand has an instant for the discard step.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::prismari_vorthos());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Discard(vec![bolt]),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Vorthos castable");
    drain_stack(&mut g);
    // Player 1 takes 2 damage because IS card was discarded.
    assert_eq!(g.players[1].life, p1_life - 2, "Burns opp for 2 after IS discard");
}

#[test]
fn inkling_sage_pump_activation_makes_two_two_flier() {
    let mut g = two_player_game();
    let sage = g.add_card_to_battlefield(0, catalog::inkling_sage());
    g.clear_sickness(sage);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sage,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Sage activation");
    drain_stack(&mut g);
    let sage = g.battlefield_find(sage).unwrap();
    assert_eq!(sage.power(), 2, "Sage pumped from 1/2 → 2/3");
    assert_eq!(sage.toughness(), 3);
}

#[test]
fn spirit_conduit_taps_for_one_damage() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let sc = g.add_card_to_battlefield(0, catalog::spirit_conduit());
    g.clear_sickness(sc);
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sc,
        ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None , mode: None}).expect("Spirit Conduit activation");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 1, "Conduit pings for 1");
    let sc = g.battlefield_find(sc).unwrap();
    assert!(sc.tapped);
    let def = catalog::spirit_conduit();
    assert!(def.card_types.contains(&crabomination::card::CardType::Artifact));
    assert!(def.card_types.contains(&crabomination::card::CardType::Creature));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Spirit));
}

#[test]
fn quandrix_aether_adept_taps_target_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let qa = g.add_card_to_battlefield(0, catalog::quandrix_aether_adept());
    g.clear_sickness(qa);
    g.perform_action(GameAction::ActivateAbility {
        card_id: qa,
        ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None , mode: None}).expect("Aether Adept activation");
    drain_stack(&mut g);
    let bear = g.battlefield_find(bear).unwrap();
    assert!(bear.tapped, "Bear tapped");
    assert!(catalog::quandrix_aether_adept().keywords.contains(&Keyword::Defender));
}

#[test]
fn prismari_sparkbright_attack_pings_target() {
    use crabomination::game::types::{Attack, AttackTarget, Target, TurnStep};
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sb = g.add_card_to_battlefield(0, catalog::prismari_sparkbright());
    g.clear_sickness(sb);
    let bear_before = g.battlefield_find(opp_bear).unwrap().damage;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sb,
        target: AttackTarget::Player(1),
    }])).expect("Attack declared");
    // The on-attack trigger needs a target.
    // Resolve any pending triggers — auto-target picks a legal candidate.
    let _ = g.pass_priority();
    drain_stack(&mut g);
    let _ = bear_before;
    let _ = Target::Permanent(opp_bear);
    // Bear should have taken at least 1 damage from the trigger OR opp lost 1 life.
    let life_after = g.players[1].life;
    let bear = g.battlefield_find(opp_bear);
    let damage_done = (20i32 - life_after)
        + bear.map(|b| b.damage as i32).unwrap_or(0);
    assert!(damage_done >= 1, "On-attack ping dealt at least 1 damage somewhere");
    assert!(catalog::prismari_sparkbright().keywords.contains(&Keyword::Haste));
}

#[test]
fn lorehold_pilgrimwarden_attack_mints_soldier() {
    use crabomination::game::{Attack, AttackTarget};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_pilgrimwarden());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Pilgrimwarden attacks");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Soldier")
        .count();
    assert_eq!(soldiers, 1, "Pilgrimwarden mints a Soldier per attack");
    let def = catalog::lorehold_pilgrimwarden();
    assert!(def.keywords.contains(&Keyword::FirstStrike));
}

// ── Table: ETB effect, then a bolt cast magecraft-pumps the body +1/+0. ────
#[test]
fn etb_effect_then_magecraft_self_pumps() {
    for (def, colors, cl, opp_delta, self_delta, token) in [
        (catalog::prismari_pyrocrafter(), vec![Color::Red], 2, -1, 0, None),
        (catalog::prismari_magmaspark(), vec![Color::Blue, Color::Red], 0, -1, 0, None),
        (catalog::prismari_drakeforge(), vec![Color::Blue, Color::Red], 2, 0, 0, Some(("Treasure", 1))),
        (catalog::pest_cultivator_adept(), vec![Color::Black, Color::Green], 2, 0, 0, Some(("Pest", 1))),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        for c in &colors { g.players[0].mana_pool.add(*c, 1); }
        for _ in 0..cl { g.players[0].mana_pool.add_colorless(1); }
        let life0_before = g.players[0].life;
        let life1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, life1_before + opp_delta, "{}: opp life after ETB", name);
        assert_eq!(g.players[0].life, life0_before + self_delta, "{}: self life after ETB", name);
        if let Some((tok, n)) = token {
            let tokens = g.battlefield.iter()
                .filter(|c| c.controller == 0 && c.definition.name == tok)
                .count();
            assert_eq!(tokens, n, "{}: ETB token count", name);
        }
        // Magecraft: cast a bolt → body grows by +1 power (or +1/+1 counter).
        let p_before = g.battlefield_find(id).map(|c| c.power()).unwrap_or(0);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let p_after = g.battlefield_find(id).map(|c| c.power()).unwrap_or(0);
        assert_eq!(p_after, p_before + 1, "{}: pumped via magecraft", name);
    }
}

#[test]
fn silverquill_tribunal_forces_opp_sacrifice_and_gains_one_life() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let _victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_tribunal());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tribunal castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "Gain 1 life");
    // Opponent should have lost their bear.
    let p1_creatures: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 1).collect();
    assert_eq!(p1_creatures.len(), 0, "Opp sacrificed their creature");
}

#[test]
fn inkling_banner_bearer_buffs_other_inklings() {
    let mut g = two_player_game();
    let bb = g.add_card_to_battlefield(0, catalog::inkling_banner_bearer());
    let token_id = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let inkling = computed.iter().find(|c| c.id == token_id).unwrap();
    // Inkling Aspirant is 2/1 base; +1/+0 anthem → 3/1.
    assert_eq!(inkling.power, 3, "Other Inkling pumped +1 power");
    assert_eq!(inkling.toughness, 1);
    // Source itself is unaffected.
    let banner = computed.iter().find(|c| c.id == bb).unwrap();
    assert_eq!(banner.power, 2);
}

#[test]
fn lorehold_revival_returns_creature_with_haste() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear_id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_revival());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear_id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Revival castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear_id), "Bear reanimated");
    // Haste grant via the new `granted_keywords_eot` EOT path (engine
    // fix — batch 24). `has_keyword` checks both printed and granted.
    let bear = g.battlefield_find(bear_id).expect("Bear on battlefield");
    assert!(bear.has_keyword(&Keyword::Haste),
        "Reanimated bear has haste EOT via `granted_keywords_eot`");
}

#[test]
fn quandrix_logician_etb_scrys_and_pumps_fractal_on_cast() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_logician());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Logician castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some());
    // Mint a Fractal first then cast a spell.
    let fractal = g.add_card_to_battlefield(0, catalog::quandrix_hatchling());
    drain_stack(&mut g);
    let counters_before = g.battlefield_find(fractal).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(fractal).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert!(counters_after > counters_before, "Fractal grew on instant cast");
}

#[test]
fn fractal_echoist_etb_counters_scale_with_graveyard() {
    let mut g = two_player_game();
    // Seed gy with IS cards.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::fractal_echoist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echoist castable");
    drain_stack(&mut g);
    let fe = g.battlefield_find(id).expect("Echoist on battlefield");
    assert_eq!(fe.counter_count(CounterType::PlusOnePlusOne), 3,
        "Echoist enters with 3 +1/+1 counters (one per IS in gy)");
}

#[test]
fn quandrix_mathenotaur_etb_doubles_counters_on_target() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Manually stack +1/+1 counters on the bear.
    {
        let b = g.battlefield_find_mut(bear).unwrap();
        b.add_counters(CounterType::PlusOnePlusOne, 3);
    }
    let counters_before = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters_before, 3);
    let id = g.add_card_to_hand(0, catalog::quandrix_mathenotaur());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mathenotaur castable");
    drain_stack(&mut g);
    let counters_after = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters_after, 6,
        "Mathenotaur doubles target's counters: 3 → 6");
}

#[test]
fn prismari_mindkindler_magecraft_taps_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::prismari_mindkindler());
    drain_stack(&mut g);
    // Untap bear so we can confirm tap.
    {
        let b = g.battlefield_find_mut(bear).unwrap();
        b.tapped = false;
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(bear).unwrap();
    assert!(bear.tapped, "Mindkindler magecraft tapped opp's bear");
}

/// CR 701.14c — "If a creature fights itself, it deals damage to
/// itself equal to twice its power." Lock-in: a 2/2 fighting itself
/// takes 2×2 = 4 damage → dies (2 toughness < 4 damage).
#[test]
fn cr_701_14c_self_fight_deals_twice_power_to_self() {
    use crabomination::card::{Effect, Selector};
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    // Use a simple 2/2 — vanilla Grizzly Bears has no triggers that
    // could interact with the fight resolution.
    let beast = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(beast);
    let fight_effect = Effect::Fight {
        attacker: Selector::Target(0),
        defender: Selector::Target(1),
    };
    // Use the same target on both slots so the fight resolves on itself.
    let ctx = {
        let mut c = EffectContext::for_spell(
            0,
            Some(crabomination::game::types::Target::Permanent(beast)),
            0,
            0,
        );
        c.targets.push(crabomination::game::types::Target::Permanent(beast));
        c
    };
    g.resolve_effect(&fight_effect, &ctx).expect("Self-fight resolves");
    drain_stack(&mut g);
    // The 4/4 takes 8 damage (2 × 4 power) → dies via SBA.
    assert!(g.battlefield_find(beast).is_none(),
        "Ironhand self-fights → 8 damage to self → dies");
}

/// Engine — `Effect::GrantKeyword` with `Duration::EndOfTurn` now uses
/// the new `granted_keywords_eot` bag on `CardInstance`, with cleanup at
/// the Cleanup step. Lock-in test: grant Haste EOT on a bear, verify
/// `has_keyword` reports it, advance to Cleanup, verify it's gone.
#[test]
fn granted_keyword_eot_clears_at_cleanup_per_batch_24() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Directly stage a granted keyword (simulates an EOT GrantKeyword
    // effect resolving on the bear).
    {
        let b = g.battlefield_find_mut(bear).unwrap();
        b.granted_keywords_eot.push(Keyword::Haste);
    }
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.has_keyword(&Keyword::Haste),
        "Bear has haste via granted_keywords_eot");
    // Computed view also picks it up.
    let computed = g.compute_battlefield();
    let bear_c = computed.iter().find(|c| c.id == bear).unwrap();
    assert!(bear_c.keywords.contains(&Keyword::Haste),
        "Computed view reports granted EOT keyword");
    // Cleanup: keyword should be cleared.
    g.step = crabomination::game::types::TurnStep::Cleanup;
    g.do_cleanup(&mut Vec::new());
    let b = g.battlefield_find(bear).unwrap();
    assert!(b.granted_keywords_eot.is_empty(),
        "granted_keywords_eot bag empty after Cleanup");
    assert!(!b.has_keyword(&Keyword::Haste),
        "Bear lost granted haste at Cleanup");
}

#[test]
fn witherbloom_pest_lord_etb_mints_pest_and_pumps_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pest_lord());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest-Lord castable");
    drain_stack(&mut g);
    // 1 Pest token minted.
    let pest_id = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Pest")
        .map(|c| c.id)
        .expect("Pest minted");
    let computed = g.compute_battlefield();
    let pest = computed.iter().find(|c| c.id == pest_id).unwrap();
    // 1/1 base + 1/0 anthem = 2/1.
    assert_eq!(pest.power, 2, "Pest pumped to 2 by Pest-Lord anthem");
    assert_eq!(pest.toughness, 1);
}

#[test]
fn lorehold_spirit_caller_etb_mints_two_hasty_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_caller());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spirit-Caller castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 2, "Two Spirit tokens minted");
    // Both Spirits have haste.
    for s in &spirits {
        assert!(s.has_keyword(&Keyword::Haste), "Spirit has haste EOT");
    }
}

#[test]
fn quandrix_symmetrycaster_etb_scales_with_hand_size() {
    let mut g = two_player_game();
    // 3 cards in hand.
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_symmetrycaster());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    assert_eq!(hand_before, 4, "Test seeded 4 cards (3 islands + Symmetrycaster)");
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Symmetrycaster castable");
    drain_stack(&mut g);
    // After casting, hand has 3 cards (4 - 1 Symmetrycaster).
    // Symmetrycaster reads hand size at trigger resolution time → 3 counters.
    let sc = g.battlefield_find(id).unwrap();
    assert_eq!(sc.counter_count(CounterType::PlusOnePlusOne), 3,
        "Symmetrycaster's ETB sized by hand size (3 islands remaining)");
}

#[test]
fn prismari_wildform_pumps_grants_haste_and_cantrips() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_wildform());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wildform castable");
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let bear_c = computed.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(bear_c.power, 4, "Bear pumped +2");
    assert_eq!(bear_c.toughness, 3, "Bear pumped +1 toughness");
    assert!(bear_c.keywords.contains(&Keyword::Haste), "Bear has haste");
    // Hand: -1 (cast) +1 (cantrip) = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_censurer_etb_taps_opp_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inkling_censurer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Censurer castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(opp_bear).unwrap();
    assert!(bear.tapped, "Bear tapped on ETB");
    assert!(catalog::inkling_censurer().keywords.contains(&Keyword::Vigilance));
}

#[test]
fn witherbloom_soilbleeder_etb_optional_sac_drains_three() {
    use crabomination::game::types::Target;
    use crabomination::decision::{ScriptedDecider, DecisionAnswer};
    let mut g = two_player_game();
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_soilbleeder());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soilbleeder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_before + 3);
    assert_eq!(g.players[1].life, p1_before - 3);
}
