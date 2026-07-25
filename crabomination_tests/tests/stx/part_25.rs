use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;


// ─────────────────────────────────────────────────────────────────────────
// Table-driven: magecraft "each cast pings/drains" creatures. Cast a bolt at
// the opponent; they lose 3 (bolt) + opp_loss (trigger); we gain self_gain.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn magecraft_ping_drain_on_instant_cast() {
    for (def, opp_loss, self_gain) in [
        (catalog::lorehold_cinderscribe_b207(), 1, 0),
        (catalog::prismari_pyrologist_b207(), 1, 0),
        (catalog::silverquill_inkbinder_b207(), 1, 1),
        (catalog::professor_onyx(), 2, 2),
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
        assert_eq!(g.players[1].life, p1_life - 3 - opp_loss, "bolt 3 + magecraft loss");
        assert_eq!(g.players[0].life, p0_life + self_gain, "magecraft gain");
    }
}

// Table-driven: magecraft "+1 power to self" creatures.
#[test]
fn magecraft_self_pump_on_instant_cast() {
    for def in [
        catalog::prismari_galeblaster_b207(),
        catalog::carving_cherub(),
        catalog::eager_first_year(),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let pwr = g.battlefield_find(id).unwrap().power();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(id).unwrap().power(), pwr + 1, "magecraft power +1");
    }
}

// Table-driven: ETB token minters — cast the creature, count named tokens.
#[test]
fn etb_token_minters_mint_named_tokens() {
    for (def, token, count) in [
        (catalog::lorehold_soulkindler_b207(), "Spirit", 1),
        (catalog::witherbloom_rotcaller_b207(), "Pest", 2),
        (catalog::prismari_goldsmith_b207(), "Treasure", 2),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 20);
        }
        g.players[0].mana_pool.add_colorless(20);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("token minter castable");
        drain_stack(&mut g);
        let tokens = g.battlefield.iter().filter(|c| c.definition.name == token).count();
        assert_eq!(tokens, count, "ETB mints the expected tokens");
    }
}

#[test]
fn witherbloom_toxicult_b207_etb_mints_pest_and_drains_on_magecraft() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxicult_b207());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxicult castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pest"),
        "ETB mints a Pest token");
    // Magecraft drain on instant cast.
    let p1_life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 4, "bolt 3 + magecraft drain 1");
}

#[test]
fn prismari_goldcaster_b207_magecraft_mints_treasure() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prismari_goldcaster_b207());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "magecraft mints a Treasure token");
}

// Table-driven: removal spells that kill/exile a targeted permanent (with an
// optional lifegain rider). expect_exiled: destroyed vs exiled distinction.
#[test]
fn removal_spells_remove_target_permanent() {
    for (def, victim_def, gain, expect_exiled) in [
        (catalog::prismari_firebolt_ii_b207(), catalog::grizzly_bears(), 0, false),
        (catalog::prismari_scorchmage_b208(), catalog::grizzly_bears(), 0, false),
        (catalog::electrickery(), catalog::savannah_lions(), 0, false),
        (catalog::fracture(), catalog::mind_stone(), 0, false),
        (catalog::silverquill_sanction_b207(), catalog::grizzly_bears(), 2, true),
    ] {
        let mut g = two_player_game();
        let victim = g.add_card_to_battlefield(1, victim_def);
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 20);
        }
        g.players[0].mana_pool.add_colorless(20);
        let p0_life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("removal castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "target removed from battlefield");
        if expect_exiled {
            assert!(!g.players[1].graveyard.iter().any(|c| c.id == victim),
                "exiled, not destroyed");
        }
        assert_eq!(g.players[0].life, p0_life + gain, "lifegain rider");
    }
}

// Table-driven: casts that damage/drain a player on ETB or resolution.
#[test]
fn etb_damage_and_drain_spells() {
    for (def, target, opp_loss, self_gain) in [
        (catalog::lorehold_emberbolt_b207(), Some(Target::Player(1)), 3, 0),
        (catalog::lorehold_pyrohistorian_b208(), Some(Target::Player(1)), 2, 0),
        (catalog::silverquill_eulogist_b207(), None, 2, 2),
        (catalog::lorehold_skydefender_b208(), None, 0, 2),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 20);
        }
        g.players[0].mana_pool.add_colorless(20);
        let p1_life = g.players[1].life;
        let p0_life = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, p1_life - opp_loss, "opponent loss");
        assert_eq!(g.players[0].life, p0_life + self_gain, "our gain");
    }
}

// Table-driven: card-flow casts — net hand delta after casting (draws minus
// the cast card and any discards).
#[test]
fn card_flow_casts_net_hand_delta() {
    for (def, lib_adds, delta) in [
        (catalog::quandrix_currentweaver_b207(), 3, 0),
        (catalog::prismari_stormloot_ii_b207(), 4, 0),
        (catalog::quandrix_tidecantor_b208(), 3, 1),
    ] {
        let mut g = two_player_game();
        for _ in 0..lib_adds { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 20);
        }
        g.players[0].mana_pool.add_colorless(20);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before + delta, "net hand delta");
    }
}

// Table-driven: Fractal minters — the token enters with the expected +1/+1
// counters (X-scaled for Fractalsurge).
#[test]
fn quandrix_fractal_minters_counter_counts() {
    for (def, x, counters) in [
        (catalog::quandrix_tidecaller_b207(), None, 2),
        (catalog::quandrix_fractalsurge_b207(), Some(3), 3),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 20);
        }
        g.players[0].mana_pool.add_colorless(20);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: x,
        }).expect("fractal minter castable");
        drain_stack(&mut g);
        let fractal = g.battlefield.iter().find(|c| c.definition.name == "Fractal")
            .expect("Fractal token minted");
        assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), counters,
            "Fractal counter count");
    }
}

// Table-driven: stat/keyword/subtype checks for vanilla-ish bodies.
#[test]
fn statline_and_keyword_checks() {
    for (def, p, t, kws, ctype) in [
        (catalog::lorehold_vanguard_b207(), 4, 3, vec![Keyword::Haste],
            Some(CreatureType::Spirit)),
        (catalog::quandrix_bigmind_b207(), 4, 5, vec![Keyword::Trample], None),
        (catalog::inkling_highflier_b207(), 2, 3, vec![Keyword::Flying, Keyword::Vigilance],
            Some(CreatureType::Inkling)),
        (catalog::disciplined_duelist(), 2, 1, vec![Keyword::DoubleStrike], None),
        (catalog::codespell_cleric(), 1, 1, vec![Keyword::Vigilance], None),
        (catalog::inkling_ambusher(), 2, 2, vec![Keyword::Flash, Keyword::Flying], None),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let c = g.battlefield_find(id).unwrap();
        assert_eq!((c.power(), c.toughness()), (p, t));
        for kw in &kws {
            assert!(c.has_keyword(kw), "expected keyword {kw:?}");
        }
        if let Some(ct) = ctype {
            assert!(c.definition.subtypes.creature_types.contains(&ct));
        }
    }
}

// Table-driven: "creatures died this turn" payoff drains — kill our/their
// bears, then cast and check the life swing.
#[test]
fn death_count_payoff_drains() {
    for (def, kill_own, kill_theirs, opp_loss, self_gain) in [
        // Total deaths this turn = 2 → opponent loses 2, we gain 2.
        (catalog::witherbloom_gravecaller_b207(), 1, 1, 2, 2),
        // 2 life per creature that died (1 died).
        (catalog::witherbloom_bloodfeast_b207(), 1, 0, 0, 2),
    ] {
        let mut g = two_player_game();
        for _ in 0..kill_own {
            let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            bolt_own_creature(&mut g, 0, c);
        }
        for _ in 0..kill_theirs {
            let c = g.add_card_to_battlefield(1, catalog::grizzly_bears());
            bolt_own_creature(&mut g, 0, c);
        }
        let p1_life = g.players[1].life;
        let p0_life = g.players[0].life;
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 20);
        }
        g.players[0].mana_pool.add_colorless(20);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("death payoff castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, p1_life - opp_loss, "drain scales with deaths");
        assert_eq!(g.players[0].life, p0_life + self_gain, "we gain that much");
    }
}

#[test]
fn witherbloom_reaping_b207_draws_per_creature_died_this_turn() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
    // Two of our creatures die this turn.
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    bolt_own_creature(&mut g, 0, b1);
    bolt_own_creature(&mut g, 0, b2);
    assert_eq!(g.players[0].creatures_died_this_turn, 2, "two creatures died");

    let id = g.add_card_to_hand(0, catalog::witherbloom_reaping_b207());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reaping castable");
    drain_stack(&mut g);
    // -1 for casting Reaping, +2 drawn = net +1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2,
        "Reaping draws one per creature that died this turn");
}

// Table-driven: "another creature you control dies" watchers.
#[test]
fn dies_trigger_watchers() {
    for (def, counter_delta, life_delta) in [
        (catalog::witherbloom_saplinglord_b207(), 1, 0),
        (catalog::silverquill_coursemate_b207(), 0, 1),
    ] {
        let mut g = two_player_game();
        let watcher = g.add_card_to_battlefield(0, def);
        let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let p0_life = g.players[0].life;
        bolt_own_creature(&mut g, 0, fodder);
        let c = g.battlefield_find(watcher).expect("watcher alive");
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), counter_delta,
            "counters when another creature dies");
        assert_eq!(g.players[0].life, p0_life + life_delta,
            "life when another creature dies");
    }
}

#[test]
fn lorehold_battlecaller_b207_mints_spirit_on_attack() {
    let mut g = two_player_game();
    let bc = g.add_card_to_battlefield(0, catalog::lorehold_battlecaller_b207());
    g.clear_sickness(bc);
    while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bc, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits, 1, "attacking mints a Spirit");
}

#[test]
fn lorehold_relicsmith_b207_returns_low_mv_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_relicsmith_b207());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Relicsmith castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "low-MV creature returned to hand");
}

#[test]
fn lorehold_charge_ii_b207_pumps_and_grants_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_charge_ii_b207());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let pwr = g.battlefield_find(bear).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Charge castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.power(), pwr + 1, "+1/+0 team pump");
    assert!(c.has_keyword(&Keyword::FirstStrike), "granted first strike");
}

#[test]
fn quandrix_theorist_b207_magecraft_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::quandrix_theorist_b207());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    // -1 bolt + 1 magecraft draw = net same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before, "magecraft scry+draw netted a card");
}

#[test]
fn quandrix_studymate_b207_grows_with_cards_drawn_this_turn() {
    let mut g = two_player_game();
    // Two cards drawn this turn so the ETB sees the tally.
    g.players[0].cards_drawn_this_turn = 2;
    let id = g.add_card_to_hand(0, catalog::quandrix_studymate_b207());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Studymate castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter().find(|c| c.definition.name == "Quandrix Studymate (b207)")
        .expect("Studymate on bf");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2,
        "+1/+1 counter per card drawn this turn");
}

#[test]
fn silverquill_edict_ii_b207_makes_opp_sacrifice_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::silverquill_edict_ii_b207());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Edict castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "opponent sacrificed their creature");
    // -1 cast + 1 draw = net same as hand_before.
    assert_eq!(g.players[0].hand.len(), hand_before, "drew a card");
}

// ─────────────────────────────────────────────────────────────────────────
// CR rule lock-in tests (batch 207 session).
// ─────────────────────────────────────────────────────────────────────────

/// CR 702.83 (Exalted): a creature that attacks alone gets +1/+1 until end
/// of turn. New `Predicate::AttackingAlone` + `exalted()` shortcut.
/// Table-driven over both Exalted 2/2s in these batches.
#[test]
fn cr_702_83_exalted_pumps_lone_attacker() {
    for def in [
        catalog::silverquill_duelmaster_b207(),
        catalog::lorehold_vanguard_captain_b208(),
    ] {
        let mut g = two_player_game();
        let duel = g.add_card_to_battlefield(0, def);
        g.clear_sickness(duel);
        while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: duel, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        let c = g.battlefield_find(duel).unwrap();
        assert_eq!((c.power(), c.toughness()), (3, 3),
            "CR 702.83a: attacking alone grants Exalted +1/+1");
    }
}

/// CR 702.83b: Exalted does NOT trigger when more than one creature
/// attacks (`Predicate::AttackingAlone` is false).
#[test]
fn cr_702_83b_exalted_silent_when_not_alone() {
    let mut g = two_player_game();
    let duel = g.add_card_to_battlefield(0, catalog::silverquill_duelmaster_b207());
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(duel);
    g.clear_sickness(buddy);
    while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: duel, target: AttackTarget::Player(1) },
        Attack { attacker: buddy, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    let c = g.battlefield_find(duel).unwrap();
    assert_eq!((c.power(), c.toughness()), (2, 2),
        "CR 702.83b: no Exalted pump when not attacking alone");
}

/// CR 702.83b: multiple Exalted abilities each trigger on a single lone
/// attacker. Two Akrasan Squires pump an attacking bear +1/+1 each → +2/+2.
#[test]
fn cr_702_83b_multiple_exalted_stack_on_lone_attacker() {
    let mut g = two_player_game();
    let _s1 = g.add_card_to_battlefield(0, catalog::akrasan_squire());
    let _s2 = g.add_card_to_battlefield(0, catalog::aven_squire());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    // The bear attacks alone (the squires hold back) → both Exalteds fire.
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!((c.power(), c.toughness()), (4, 4),
        "two Exalted sources each add +1/+1 to the lone attacker");
}

/// CR 702.92 (Battle cry): when the source attacks, each *other* attacking
/// creature gets +1/+0 — but the source itself does not.
#[test]
fn cr_702_92_battle_cry_pumps_other_attackers_only() {
    let mut g = two_player_game();
    let driver = g.add_card_to_battlefield(0, catalog::goblin_wardriver());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(driver);
    g.clear_sickness(bear);
    while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: driver, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    let other = g.battlefield_find(bear).unwrap();
    assert_eq!((other.power(), other.toughness()), (3, 2),
        "battle cry pumps the other attacker +1/+0");
    let src = g.battlefield_find(driver).unwrap();
    assert_eq!((src.power(), src.toughness()), (2, 2),
        "battle cry does NOT pump its own source");
}

/// CR 702.15 (Lifelink): combat damage dealt by a creature with lifelink
/// causes its controller to gain that much life. Table-driven with
/// Witherbloom Sapsiphon, whose combat-damage trigger gains a fixed 2.
#[test]
fn cr_702_15_lifelink_combat_damage_gains_life() {
    for (def, fixed_gain) in [
        // Anthemwriter is a 4/4 flying lifelink finisher (gain == power).
        (catalog::silverquill_anthemwriter(), None),
        (catalog::witherbloom_sapsiphon_b207(), Some(2)),
    ] {
        let mut g = two_player_game();
        let ll = g.add_card_to_battlefield(0, def);
        g.clear_sickness(ll);
        let power = g.battlefield_find(ll).unwrap().power();
        while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ll, target: AttackTarget::Player(1),
        }])).expect("attack");
        drain_stack(&mut g);
        let p0_life = g.players[0].life;
        let p1_life = g.players[1].life;
        while g.step != crabomination::game::types::TurnStep::CombatDamage {
            g.perform_action(GameAction::PassPriority).expect("pass priority");
        }
        g.resolve_combat().expect("combat damage");
        drain_stack(&mut g);
        let gain = fixed_gain.unwrap_or(power);
        assert_eq!(g.players[1].life, p1_life - power, "opponent took combat damage");
        assert_eq!(g.players[0].life, p0_life + gain, "CR 702.15: gained on combat damage");
    }
}

/// CR 510.1c: a trampling attacker blocked by a single creature assigns
/// lethal to the blocker and tramples the rest to the defending player.
#[test]
fn cr_510_1c_trample_overflow_to_player() {
    let mut g = two_player_game();
    // 4/4 trampler vs a 2/2 blocker → 2 lethal to blocker, 2 tramples.
    let atk = g.add_card_to_battlefield(0, catalog::quandrix_bigmind_b207()); // 4/5 Trample
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != crabomination::game::types::TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)]))
        .expect("block");
    drain_stack(&mut g);
    let p1_life = g.players[1].life;
    while g.step != crabomination::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(blk).is_none(), "blocker took lethal");
    // 4 power - 2 lethal to the 2/2 = 2 trample to player.
    assert_eq!(g.players[1].life, p1_life - 2, "CR 510.1c: 2 damage tramples over");
}

/// CR 510.1c-d — a `wants_ui` attacking player chooses, via interactive
/// `pending_decision`s, the order its blockers take damage and how its power
/// is divided. Here a 3/3 splits across two 2/2 blockers: by ordering the
/// second-declared blocker first, the player kills *it* (and leaves the
/// first alive), which the engine's default CardId-order split would not do.
#[test]
fn cr_510_1c_ui_player_orders_and_assigns_combat_damage() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let beater = CardDefinition {
        name: "Three Three",
        card_types: vec![CardType::Creature],
        power: 3,
        toughness: 3,
        ..Default::default()
    };
    let atk = g.add_card_to_battlefield(0, beater);
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // lower CardId
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // higher CardId
    g.clear_sickness(atk);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(b1, atk), (b2, atk)]))
        .expect("double block");
    drain_stack(&mut g);

    // Pass priority into the combat damage step — it suspends on the order
    // decision instead of auto-splitting.
    while g.pending_decision.is_none() {
        g.perform_action(GameAction::PassPriority).expect("pass");
        assert!(!g.is_game_over());
    }
    let pd = g.pending_decision.as_ref().expect("combat suspends on ordering");
    let Decision::CombatDamageOrder { attacker, blockers } = &pd.decision else {
        panic!("expected CombatDamageOrder, got {:?}", pd.decision);
    };
    assert_eq!(*attacker, atk);
    assert_eq!(blockers.len(), 2);
    // Order b2 (declared second) ahead of b1 so it takes lethal first.
    g.submit_decision(DecisionAnswer::DamageOrder(vec![b2, b1]))
        .expect("order accepted");

    // Now it suspends on the assignment decision.
    let pd = g.pending_decision.as_ref().expect("combat suspends on assignment");
    let Decision::AssignCombatDamage { attacker_power, blockers, .. } = &pd.decision else {
        panic!("expected AssignCombatDamage, got {:?}", pd.decision);
    };
    assert_eq!(*attacker_power, 3);
    assert_eq!(blockers.first().map(|(id, _, _)| *id), Some(b2),
        "the chosen order puts b2 first");
    // Assign lethal (2) to b2, the remaining 1 to b1.
    g.submit_decision(DecisionAnswer::CombatDamageAssignment(vec![(b2, 2), (b1, 1)]))
        .expect("assignment accepted");
    drain_stack(&mut g);

    assert!(g.pending_decision.is_none(), "combat fully resolved");
    assert!(g.battlefield_find(b2).is_none(), "b2 was assigned lethal and died");
    assert!(g.battlefield_find(b1).is_some(), "b1 only took 1 and survived");
}

/// CR 509.2 / 510.1c — Banding: when an attacker is blocked by a band that
/// includes a creature with banding, the *defending* player (not the
/// attacker) announces the attacker's damage order and assignment. Here the
/// bot attacks and the human defender, holding a banding blocker, is the one
/// prompted — and assigns all 3 to one bear, sparing the other.
#[test]
fn cr_509_2_banding_blocker_lets_defender_assign_damage() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::game::types::{ResumeContext, TurnStep};
    let mut g = two_player_game();
    // Defender (P1) drives the UI; the attacking bot (P0) does not.
    g.players[1].wants_ui = true;
    let beater = CardDefinition {
        name: "Three Three",
        card_types: vec![CardType::Creature],
        power: 3,
        toughness: 3,
        ..Default::default()
    };
    let atk = g.add_card_to_battlefield(0, beater);
    let hero = g.add_card_to_battlefield(1, catalog::benalish_hero()); // 1/1 banding
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(atk);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(hero, atk), (bear, atk)]))
        .expect("band-block with the banding hero");
    drain_stack(&mut g);

    while g.pending_decision.is_none() {
        g.perform_action(GameAction::PassPriority).expect("pass");
        assert!(!g.is_game_over());
    }
    // The order decision is routed to the *defending* player (P1).
    let pd = g.pending_decision.as_ref().expect("combat suspends on ordering");
    assert!(matches!(pd.resume, ResumeContext::CombatDamage { player: 1, .. }),
        "banding routes the assignment to the defending player, got {:?}", pd.resume);
    assert!(matches!(pd.decision, Decision::CombatDamageOrder { .. }));
    g.submit_decision(DecisionAnswer::DamageOrder(vec![bear, hero])).expect("defender orders");
    // Assignment also goes to the defender: dump all 3 into the bear, none on
    // the hero, so the 1/1 banding hero survives.
    g.submit_decision(DecisionAnswer::CombatDamageAssignment(vec![(bear, 3), (hero, 0)]))
        .expect("defender assigns");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear took all 3 and died");
    assert!(g.battlefield_find(hero).is_some(), "banding hero spared by the defender");
}

/// CR 702.85b (Cascade): the exile walk stops at the first nonland card
/// with mana value *strictly less* than the cascading spell's. A card whose
/// MV equals the cascade MV is not a valid hit — it's exiled past and
/// bottomed.
#[test]
fn cr_702_85_cascade_skips_equal_mana_value_cards() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Top of library: Brightglass Gearhulk (MV 4 — equals cascade MV, must
    // be skipped), then Grizzly Bears (MV 2 — the legal hit).
    let gearhulk = g.add_card_to_library(0, catalog::brightglass_gearhulk());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let elf = g.add_card_to_hand(0, catalog::bloodbraid_elf()); // cascade(4)
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, elf);

    assert!(g.battlefield.iter().any(|c| c.id == bears),
        "CR 702.85b: the MV-2 card is the legal cascade hit");
    assert!(!g.battlefield.iter().any(|c| c.id == gearhulk),
        "the MV-4 card (== cascade MV) is NOT cast");
    assert!(g.players[0].library.iter().any(|c| c.id == gearhulk),
        "the skipped equal-MV card is bottomed back into the library");
}

/// CR 702.52e (Dredge): dredging *replaces* the draw — the player gains no
/// net card from the draw event (the dredged card returns to hand, but the
/// would-be draw never happens). Net hand size change is +1 (the dredge
/// card) and the library shrinks only by the dredge count (the mill).
#[test]
fn cr_702_52_dredge_replaces_the_draw_event() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
    let thug = g.add_card_to_graveyard(0, catalog::golgari_thug()); // Dredge 4
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let lib_before = g.players[0].library.len();
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    // No card was drawn from the top — the library shrank only by the mill.
    assert_eq!(g.players[0].library.len(), lib_before - 4,
        "CR 702.52e: dredge mills 4 and skips the draw (library -4, not -5)");
    assert!(g.players[0].hand.iter().any(|c| c.id == thug),
        "the dredged card returns to hand");
    assert!(!events.iter().any(|e| matches!(e, GameEvent::CardDrawn { .. })),
        "no CardDrawn event — the draw was replaced");
}

/// CR 702.2c (Deathtouch): any nonzero combat damage a deathtouch creature
/// deals is lethal. Here a 1/2 deathtouch *blocker* (Stinkweed Imp) kills a
/// 6/4 attacker (Craw Wurm) by dealing it 1 damage.
#[test]
fn cr_702_2c_deathtouch_blocker_destroys_larger_attacker() {
    let mut g = two_player_game();
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm()); // 6/4
    let imp = g.add_card_to_battlefield(1, catalog::stinkweed_imp()); // 1/2 deathtouch
    g.clear_sickness(wurm);
    while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: wurm, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != crabomination::game::types::TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(imp, wurm)])).expect("block");
    drain_stack(&mut g);
    while g.step != crabomination::game::types::TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_none(),
        "CR 702.2c: 1 point of deathtouch damage is lethal to the 6/4");
}

/// CR 302.6 (Summoning sickness): a creature can't attack on the turn it
/// comes under its controller's control unless it has haste.
#[test]
fn cr_302_6_summoning_sick_creature_cannot_attack() {
    let mut g = two_player_game();
    // No clear_sickness → the creature is summoning sick this turn.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    let res = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }]));
    assert!(res.is_err(), "CR 302.6: summoning-sick creature can't be declared as attacker");
    // A haste creature (Lorehold Vanguard) is exempt.
    let haste = g.add_card_to_battlefield(0, catalog::lorehold_vanguard_b207());
    let ok = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: haste, target: AttackTarget::Player(1),
    }]));
    assert!(ok.is_ok(), "CR 702.10b: Haste exempts a freshly-entered creature");
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 208 (modern_decks) — cross-school follow-ups.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn prismari_scholar_adept_b208_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::prismari_scholar_adept_b208());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Just verify the cast resolves with the magecraft trigger present.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(g.stack.is_empty(), "magecraft scry resolved");
}

#[test]
fn quandrix_rootmage_b208_etb_counters_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_rootmage_b208());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rootmage castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

// ════════════════════════════════════════════════════════════════════════════
// Coverage backfill (claude/modern_decks): functionality tests for STX cards
// that were wired but lacked a dedicated test.
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn dina_soul_steeper_drains_on_lifegain() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    // "Whenever you gain life, each opponent loses 1 life."
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dina_soul_steeper());
    // Gain 5 life via Witherbloom Charm's lifegain mode.
    let charm = g.add_card_to_hand(0, catalog::witherbloom_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    g.perform_action(GameAction::CastSpell {
        card_id: charm, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("Witherbloom Charm castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "Dina drains 1 from the opponent on lifegain");
}

#[test]
fn professor_onyx_plus_one_loses_one_and_digs_three() {
    // Real +1: "You lose 1 life. Look at the top three cards of your
    // library. Put one of them into your hand and the rest into your
    // graveyard." (An earlier synthesized +1 drained 2.)
    let mut g = two_player_game();
    let onyx = g.add_card_to_battlefield(0, catalog::professor_onyx());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: onyx, ability_index: 0, target: None,
    })
    .expect("Onyx +1 activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 - 1, "you lose 1 life");
    assert_eq!(g.players[1].life, p1, "opponent untouched by the +1");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "one of the top three to hand");
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2, "the other two to the graveyard");
    let pw = g.battlefield_find(onyx).unwrap();
    assert_eq!(pw.counter_count(CounterType::Loyalty), 6, "5 base + 1 from the +1 ability");
}

#[test]
fn strixhaven_pondkeeper_etb_scries_and_has_flash() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::strixhaven_pondkeeper());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Pondkeeper castable for {1}{U}");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Pondkeeper on battlefield");
    assert_eq!((c.definition.power, c.definition.toughness), (2, 1));
    assert!(c.definition.keywords.contains(&Keyword::Flash));
}

#[test]
fn zimone_quandrix_prodigy_puts_land_and_scales_draw() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::zimone_quandrix_prodigy());
    g.clear_sickness(id);
    let land = g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![land])]));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("Zimone land-drop ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "land from hand, tapped");

    // Second ability: eight lands → draw two.
    for _ in 0..7 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    g.battlefield_find_mut(id).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(4);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("Zimone draw ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 2, "eight lands → draw two");
}

#[test]
fn academic_probation_mode0_locks_opponent_from_casting_named_spell() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::academic_probation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Name "Grizzly Bears" — opponent then can't cast their copy.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard(
        "Grizzly Bears".to_string(),
    )]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("Academic Probation mode 0 castable");
    drain_stack(&mut g);
    let opp_bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    // Hand priority to the opponent so they attempt the cast.
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: opp_bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect_err("named spell can't be cast by the opponent");
    assert!(matches!(err, crabomination::game::GameError::SpellNameLocked), "got {err:?}");
}

#[test]
fn unwilling_ingredient_exiles_from_graveyard_to_draw_and_lose_one() {
    // Real oracle: "Menace / {2}{B}, Exile this card from your graveyard:
    // You draw a card and you lose 1 life." (An earlier synthesized body
    // had a "dies → may pay {2}{B} to draw" trigger instead.)
    let mut g = two_player_game();
    let ingredient = g.add_card_to_graveyard(0, catalog::unwilling_ingredient());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ingredient, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("{2}{B}, exile from graveyard: draw 1, lose 1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    assert_eq!(g.players[0].life, life_before - 1, "lost 1 life");
    assert!(g.exile.iter().any(|c| c.id == ingredient),
        "Unwilling Ingredient exiled itself as the cost");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == ingredient),
        "no longer in the graveyard");
    assert!(catalog::unwilling_ingredient().keywords.contains(&Keyword::Menace));
}

#[test]
fn cr_122_1d_stun_counter_persists_through_untap() {
    // End-to-end CR 122.1d via Containment Studies: it taps a creature and
    // gives it two stun counters; each of the controller's untap steps
    // consumes one counter instead of untapping, untapping only once both
    // are gone.
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::containment_studies());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Containment Studies castable for {2}{W}");
    drain_stack(&mut g);
    {
        let bear = g.battlefield_find(opp_bear).unwrap();
        assert!(bear.tapped);
        assert_eq!(bear.counter_count(CounterType::Stun), 2);
    }
    g.active_player_idx = 1;
    g.do_untap();
    {
        let bear = g.battlefield_find(opp_bear).unwrap();
        assert!(bear.tapped, "stun keeps the creature tapped through one untap");
        assert_eq!(bear.counter_count(CounterType::Stun), 1, "one stun counter consumed");
    }
    g.do_untap();
    {
        let bear = g.battlefield_find(opp_bear).unwrap();
        assert!(bear.tapped, "still tapped with one stun counter left");
        assert_eq!(bear.counter_count(CounterType::Stun), 0, "second stun counter consumed");
    }
    g.do_untap();
    assert!(!g.battlefield_find(opp_bear).unwrap().tapped,
        "untaps normally once the stun counters are gone");
}

#[test]
fn diviners_wand_equips_for_three_and_buffs() {
    let mut g = two_player_game();
    let wand = g.add_card_to_battlefield(0, catalog::diviners_wand());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: wand, target: bear })
        .expect("Diviner's Wand equips for {3}");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+1 over a 2/2 bear");
    assert!(cp.keywords.contains(&Keyword::Flying), "grants flying");
}

#[test]
fn opposition_taps_a_creature_to_tap_a_permanent() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(0, catalog::opposition());
    let _ = opp;
    // A creature to pay the tap cost (clear sickness so it's a valid tapper).
    let tapper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(tapper);
    // Opponent's land to tap down.
    let target = g.add_card_to_battlefield(1, catalog::island());
    let opp_id = g.battlefield.iter().find(|c| c.definition.name == "Opposition").unwrap().id;
    g.perform_action(GameAction::ActivateAbility {
        card_id: opp_id, ability_index: 0,
        target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None,
    }).expect("Opposition activates by tapping a creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tapper).unwrap().tapped, "creature tapped to pay cost");
    assert!(g.battlefield_find(target).unwrap().tapped, "target permanent tapped");
}

/// CR 602.5b — a `wants_ui` activator chooses *which* of their creatures to
/// tap for Opposition's "Tap an untapped creature you control" cost, rather
/// than the engine auto-tapping the weakest. Activation suspends on a
/// `ChooseTarget`; the chosen creature is the one tapped.
#[test]
fn opposition_ui_activator_chooses_creature_to_tap() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let opp = g.add_card_to_battlefield(0, catalog::opposition());
    let tapper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let keep = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(tapper);
    g.clear_sickness(keep);
    let target = g.add_card_to_battlefield(1, catalog::island());

    g.perform_action(GameAction::ActivateAbility {
        card_id: opp, ability_index: 0,
        target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None,
    })
    .expect("activation suspends for the tap-cost choice");

    let pd = g.pending_decision.as_ref().expect("a tap-cost choice is pending");
    assert_eq!(pd.acting_player(), 0);
    match &pd.decision {
        crabomination::decision::Decision::ChooseTarget { legal, .. } => {
            assert_eq!(legal.len(), 2, "both creatures are tap-cost options");
        }
        other => panic!("expected ChooseTarget, got {other:?}"),
    }

    g.perform_action(GameAction::SubmitDecision(crabomination::decision::DecisionAnswer::Target(
        Target::Permanent(tapper),
    )))
    .expect("submit the tap-cost choice");

    assert!(g.battlefield_find(tapper).unwrap().tapped, "chosen creature tapped to pay cost");
    assert!(!g.battlefield_find(keep).unwrap().tapped, "unchosen creature stays untapped");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).unwrap().tapped, "target permanent tapped on resolve");
}

#[test]
fn omniscience_casts_hand_spells_free() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::omniscience());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    // Empty mana pool — only Omniscience makes this castable.
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Omniscience casts Bolt for free");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "Bolt dealt 3 with no mana paid");
    // Spell goes to graveyard, not exile (Omniscience doesn't exile).
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"));
}

#[test]
fn academic_dispute_forces_block_if_able() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    // Academic Dispute's rider: must be blocked if able.
    g.battlefield_find_mut(attacker).unwrap()
        .granted_keywords_eot.push(Keyword::MustBeBlocked);
    g.step = crabomination::game::types::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = crabomination::game::types::TurnStep::DeclareBlockers;
    // Leaving it unblocked while opp has an idle able blocker is illegal.
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![])).is_err(),
        "must-be-blocked attacker can't be left unblocked");
    // Assigning the idle blocker is legal.
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(opp, attacker)])).is_ok(),
        "blocking it satisfies the requirement");
}

#[test]
fn blade_historian_double_strike_deals_combat_damage_twice() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::blade_historian());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    // CR 702.4: double striker deals in the first-strike step…
    g.step = TurnStep::FirstStrikeDamage;
    g.resolve_first_strike_damage().expect("fs damage");
    assert_eq!(g.players[1].life, 18, "first-strike hit for 2");
    // …and again in the regular step.
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("regular damage");
    assert_eq!(g.players[1].life, 16, "regular hit for another 2");
}

#[test]
fn lorehold_mentor_buffs_lesser_power_attacker() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let mentor = g.add_card_to_battlefield(0, catalog::lorehold_mentor()); // 3 power
    let smaller = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 < 3
    g.clear_sickness(mentor);
    g.clear_sickness(smaller);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: mentor, target: AttackTarget::Player(1) },
        Attack { attacker: smaller, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(smaller).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "lesser-power attacker gains a Mentor counter");
}

// ─────────────────────────────────────────────────────────────────────────
// Coverage backfill (modern_decks): functionality tests for previously
// untested-by-name STX cards. All cards already wired; tests only.
// ─────────────────────────────────────────────────────────────────────────

#[test]
fn closing_statement_destroys_and_puts_counter_on_own_creature() {
    // Real oracle: "Destroy target creature or planeswalker you don't
    // control. Put a +1/+1 counter on up to one target creature you
    // control." (An earlier synthesized body exiled and gained X life.)
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::closing_statement());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(mine)], mode: None, x_value: None,
    }).expect("Closing Statement castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "opposing creature destroyed");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "destroyed, not exiled — it lands in its owner's graveyard");
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "own creature picked up the +1/+1 counter");
}

#[test]
fn devastating_mastery_destroys_all_nonland_permanents() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::devastating_mastery());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Devastating Mastery castable for {4}{W}{W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mine), "own creature destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == theirs), "opp creature destroyed");
}

#[test]
fn quandrix_apprentice_magecraft_digs_three_for_a_land() {
    // "Magecraft — ... look at the top three cards of your library. You may
    // reveal a land card from among them and put that card into your hand.
    // Put the rest on the bottom of your library in any order."
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::quandrix_apprentice());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    let lands_before = g.players[0].hand.iter()
        .filter(|c| c.definition.is_land()).count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 1,
        "one land taken to hand, two bottomed");
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.is_land()).count(),
        lands_before + 1, "magecraft impulsed a land into hand");
}

#[test]
fn blustersquall_taps_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::blustersquall());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Blustersquall castable for {U}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "target creature tapped");
}

#[test]
fn multiple_choice_casts_and_resolves() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::multiple_choice());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Multiple Choice castable for {1}{U}{U}");
    drain_stack(&mut g);
    // The modal sorcery resolves and is put into the graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Multiple Choice"),
        "Multiple Choice resolved to the graveyard");
}

/// Professor Onyx −8 — seven Punisher rounds: the opponent discards a
/// card per round while able, then loses 3 per round they can't cover.
/// With 3 cards in hand: 3 discards + 4 × 3 life lost.
#[test]
fn professor_onyx_ultimate_discard_or_lose_three_seven_times() {
    let mut g = two_player_game();
    let onyx = g.add_card_to_battlefield(0, catalog::professor_onyx());
    g.battlefield_find_mut(onyx).unwrap().add_counters(CounterType::Loyalty, 3); // 5+3=8
    for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
    let p1 = g.players[1].life;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        x_value: None,
        card_id: onyx, ability_index: 2, target: None,
    }).expect("Onyx -8 activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "all three cards discarded");
    assert_eq!(g.players[1].life, p1 - 12, "loses 3 for each of the 4 uncovered rounds");
}

/// Mila's loyalty trigger fires only when a PLANESWALKER you control is
/// attacked — attacks on the player don't add loyalty (audit fix).
#[test]
fn mila_loyalty_only_on_planeswalker_attacks() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mila_crafty_companion());
    let onyx = g.add_card_to_battlefield(0, catalog::professor_onyx());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;

    // Attack the PLAYER: no loyalty trigger.
    let before = g.battlefield_find(onyx).unwrap().counter_count(CounterType::Loyalty);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(0),
    }])).expect("attack player");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(onyx).unwrap().counter_count(CounterType::Loyalty),
        before, "no loyalty from an attack on the player");

    // New combat: attack the planeswalker — loyalty trigger fires.
    g.set_attacking(vec![]);
    g.battlefield_find_mut(bear).unwrap().tapped = false;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Planeswalker(onyx),
    }])).expect("attack planeswalker");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(onyx).unwrap().counter_count(CounterType::Loyalty),
        before + 1, "each planeswalker you control gets a loyalty counter");
}
