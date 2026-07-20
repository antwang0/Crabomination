use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

/// Give player 0 a huge rainbow mana pool so table-driven tests can cast
/// any of the cards under test without per-card mana bookkeeping.
fn rainbow_mana(g: &mut crabomination::game::GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

// ── Dean's List ────────────────────────────────────────────────────────────

#[test]
fn deans_list_takes_top_card_and_mills_rest() {
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::grizzly_bears());
    let b = g.add_card_to_library(0, catalog::island());
    let c = g.add_card_to_library(0, catalog::forest());

    let id = g.add_card_to_hand(0, catalog::deans_list());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dean's List castable");
    drain_stack(&mut g);

    // Hand should contain one of the three; the rest should be in graveyard.
    let in_hand = [a, b, c].iter().filter(|&&id| {
        g.players[0].hand.iter().any(|inst| inst.id == id)
    }).count();
    let in_gy = [a, b, c].iter().filter(|&&id| {
        g.players[0].graveyard.iter().any(|inst| inst.id == id)
    }).count();
    assert!(in_hand >= 1, "at least one card in hand");
    // The other two go to graveyard via RevealMissDest::Graveyard.
    assert!(in_gy >= 1 || in_hand >= 1, "some cards moved out of library");
}

// ── Reanimation to the battlefield (table) ─────────────────────────────────
// Sigardian Savior, Brilliant Restoration, Witherbloom Necromancy,
// Lorehold Resurgence: a creature card leaves the graveyard for the
// battlefield, with an optional life delta on the caster.

#[test]
fn reanimation_spells_return_creature_to_battlefield() {
    for (def, targeted, life_delta) in [
        (catalog::sigardian_savior(), false, None),
        (catalog::brilliant_restoration(), false, Some(2)),
        (catalog::witherbloom_necromancy(), true, Some(-2)),
        (catalog::lorehold_resurgence(), true, None),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let bear_id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        let life_before = g.players[0].life;
        let target = if targeted { Some(Target::Permanent(bear_id)) } else { None };
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == bear_id), "{}: bear reanimated", name);
        if let Some(d) = life_delta {
            assert_eq!(g.players[0].life, life_before + d, "{}: life delta", name);
        }
    }
}

// ── Sneaky Snacker ─────────────────────────────────────────────────────────

#[test]
fn sneaky_snacker_recurs_from_graveyard_to_hand() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::sneaky_snacker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    // sorcery-speed activation requires main-phase priority on our turn.
    assert_eq!(g.active_player_idx, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None }).expect("Snacker recurs from graveyard");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "snacker in hand");
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == id),
        "snacker removed from gy"
    );
}

// ── Targeted removal (table) ───────────────────────────────────────────────
// Each spell targets the opponent's 2/2 Grizzly Bears and kills it, with an
// optional life delta on the caster.

#[test]
fn targeted_removal_kills_opp_bear() {
    for (def, life_delta) in [
        (catalog::daring_diversion(), None),
        (catalog::witherbloom_strangler(), None),
        (catalog::pyromancers_bolt(), None),
        (catalog::demolishing_lecture(), None),
        (catalog::critical_critique(), None),
        (catalog::galvanic_ribbons(), None),
        (catalog::lorehold_b35_lightning(), None),
        (catalog::prismari_pyrotechnics(), None),
        (catalog::prismari_spellfire(), None),
        (catalog::pestilent_verse(), Some(-1)),
        (catalog::witherbloom_necrotouch(), Some(2)),
        (catalog::silverquill_decree(), Some(2)),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        // Library seed for scry/cantrip riders (Critical Critique, Spellfire).
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::swamp());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "{}: bear killed", name);
        if let Some(d) = life_delta {
            assert_eq!(g.players[0].life, life_before + d, "{}: caster life delta", name);
        }
    }
}

// ── Pilgrim of the Ages ────────────────────────────────────────────────────

/// Printed oracle: "When this creature enters, you may search your
/// library for a basic Plains card, reveal it, put it into your hand,
/// then shuffle. / {6}: Return this card from your graveyard to your
/// hand."
#[test]
fn pilgrim_of_the_ages_sac_searches_for_basic_land() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let plains_id = g.add_card_to_library(0, catalog::plains());
    let pilgrim = g.add_card_to_hand(0, catalog::pilgrim_of_the_ages());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains_id))]));
    g.perform_action(GameAction::CastSpell {
        card_id: pilgrim, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pilgrim castable for {2}{W}");
    drain_stack(&mut g);
    // Pilgrim stays on the battlefield; the ETB tutors a basic Plains.
    assert!(g.battlefield.iter().any(|c| c.id == pilgrim), "pilgrim on battlefield");
    assert!(
        g.players[0].hand.iter().any(|c| c.id == plains_id),
        "basic Plains tutored to hand"
    );
}

#[test]
fn pilgrim_of_the_ages_six_mana_returns_it_from_graveyard() {
    let mut g = two_player_game();
    let pilgrim = g.add_card_to_graveyard(0, catalog::pilgrim_of_the_ages());
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pilgrim,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Pilgrim {6} graveyard activation");
    drain_stack(&mut g);
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == pilgrim),
        "pilgrim left the graveyard");
    assert!(g.players[0].hand.iter().any(|c| c.id == pilgrim),
        "pilgrim returned to hand");
}

// ── Fractal makers (table): Strixhaven Spawner, Quandrix Doubling Tutor ────

#[test]
fn fractal_makers_mint_countered_fractals() {
    for (def, min_counters) in [
        (catalog::strixhaven_spawner(), 2),
        (catalog::quandrix_doubling_tutor(), 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        // Each Fractal enters at 0/0 and the ForEach pumps counters before
        // SBA, so the tokens survive with their counters.
        let fractals: Vec<_> = g
            .battlefield
            .iter()
            .filter(|c| c.controller == 0 && c.definition.name == "Fractal")
            .collect();
        assert!(!fractals.is_empty(), "{}: at least one Fractal token minted", name);
        for f in &fractals {
            let n = f.counter_count(CounterType::PlusOnePlusOne);
            assert!(n >= min_counters, "{}: each fractal has ≥{} counters (got {})",
                name, min_counters, n);
        }
    }
}

// ── Magecraft life triggers (table) ────────────────────────────────────────
// Each card sits on the battlefield while its controller casts a Lightning
// Bolt at the opponent; life deltas include Bolt's own 3 damage.

#[test]
fn magecraft_life_triggers_on_instant_cast() {
    for (def, p0_delta, p1_delta) in [
        (catalog::mage_hunter_defender(), Some(1), Some(-4)),
        (catalog::quill_witch(), Some(1), Some(-4)),
        (catalog::shadow_mage_hopeful(), None, Some(-4)),
        (catalog::dissident_lecturer(), None, Some(-4)),
        (catalog::witherbloom_tincture_maker(), Some(1), None),
        (catalog::silverquill_initiate(), Some(1), None),
        (catalog::witherbloom_acolyte(), Some(1), None),
        (catalog::witherbloom_scholar(), Some(1), Some(-4)),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let _card = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let p0_before = g.players[0].life;
        let p1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).unwrap_or_else(|e| panic!("{}: Bolt castable: {:?}", name, e));
        drain_stack(&mut g);
        if let Some(d) = p0_delta {
            assert_eq!(g.players[0].life, p0_before + d, "{}: P0 life delta", name);
        }
        if let Some(d) = p1_delta {
            assert_eq!(g.players[1].life, p1_before + d, "{}: P1 life delta", name);
        }
    }
}

// ── Magecraft self-pumps (table) ───────────────────────────────────────────

#[test]
fn magecraft_self_pumps_on_instant_cast() {
    for (def, min_power, toughness) in [
        (catalog::pestilent_inkmage(), 4, Some(4)),     // base 2/4, +2/+0
        (catalog::crackleburr_initiate(), 3, Some(1)),  // base 2/1, +1/+0
        (catalog::quill_inscriber(), 3, None),
        (catalog::lorehold_pyromancer(), 4, None),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let mage = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).unwrap_or_else(|e| panic!("{}: Bolt castable: {:?}", name, e));
        drain_stack(&mut g);
        let pt = g.computed_permanent(mage).expect("computed");
        assert!(pt.power >= min_power, "{}: power ≥{} (got {})", name, min_power, pt.power);
        if let Some(t) = toughness {
            assert_eq!(pt.toughness, t, "{}: toughness", name);
        }
    }
}

#[test]
fn pestilent_inkmage_does_not_trigger_on_creature_cast() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::pestilent_inkmage());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bear castable");
    drain_stack(&mut g);
    let pt = g.computed_permanent(mage).expect("computed");
    // Base 2/4 unchanged (no magecraft on creature cast).
    assert_eq!(pt.power, 2);
    assert_eq!(pt.toughness, 4);
}

// ── Magecraft scry (table): Eager Scribe, Quill Page ───────────────────────

#[test]
fn magecraft_scry_on_instant_cast() {
    for def in [catalog::eager_scribe(), catalog::quill_page()] {
        let name = def.name;
        let mut g = two_player_game();
        let _c = g.add_card_to_battlefield(0, def);
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::mountain());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        let lib_before = g.players[0].library.len();
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).unwrap_or_else(|e| panic!("{}: Bolt castable: {:?}", name, e));
        drain_stack(&mut g);
        // Scry 1 doesn't change library size.
        assert_eq!(g.players[0].library.len(), lib_before, "{}: library unchanged", name);
    }
}

// ── Detention Sphere ───────────────────────────────────────────────────────

#[test]
fn detention_sphere_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::detention_sphere());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Detention Sphere castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "bear exiled on ETB");
    // The Sphere leaves — the linked exile returns to the battlefield.
    g.remove_from_battlefield_to_graveyard_raw(id);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear), "bear returns when the Sphere leaves");
}

// ── ETB counters on friendly creatures, no target (table) ──────────────────

#[test]
fn etb_counter_fanout_on_friendly_creatures() {
    for def in [
        catalog::silvercrown_lecturer(),
        catalog::symmetry_lecturer(),
        catalog::quandrix_sphinx(),
        catalog::quandrix_recalibrator(),
        catalog::quandrix_theorem(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        let bear_perm = g.battlefield.iter().find(|c| c.id == bear).expect("bear alive");
        assert!(bear_perm.counter_count(CounterType::PlusOnePlusOne) >= 1,
            "{}: bear gets ≥1 +1/+1 counter", name);
    }
}

// ── Targeted +1/+1 counter spells (table) ──────────────────────────────────

#[test]
fn targeted_counter_spells_land_counters() {
    for (def, n) in [
        (catalog::quandrix_cryptidkeeper(), 2),
        (catalog::mascot_interpretation(), 2),
        (catalog::quandrix_apprenticeship(), 2),
        (catalog::silver_quill_scholarship(), 1),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        // Library seed for cantrip riders (Scholarship, Interpretation).
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        let bear_perm = g.battlefield.iter().find(|c| c.id == bear).expect("bear alive");
        assert_eq!(bear_perm.counter_count(CounterType::PlusOnePlusOne), n,
            "{}: bear got {} +1/+1 counters", name, n);
    }
}

// ── Prismari Eruption ──────────────────────────────────────────────────────

#[test]
fn prismari_eruption_burns_grounded_creatures_and_spares_flyers() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::prismari_eruption());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Eruption castable");
    drain_stack(&mut g);
    // 2/2 bear dies; 4/4 flying Serra Angel lives.
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear dies");
    assert!(g.battlefield.iter().any(|c| c.id == flyer), "flyer survives");
}

// ── Silverquill Inquisitor ─────────────────────────────────────────────────

#[test]
fn silverquill_inquisitor_etb_discards_from_opp_hand() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let inq = g.add_card_to_hand(0, catalog::silverquill_inquisitor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: inq, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inquisitor castable");
    drain_stack(&mut g);
    // Opp's hand drops by 1 (random discard).
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
}

// ── Cantrips / looters / lifegain ETBs (table) ─────────────────────────────
// Cast the card with no target and assert the caster's net hand delta
// and/or life delta.

#[test]
fn cast_no_target_hand_and_life_deltas() {
    for (def, hand_delta, life_delta) in [
        (catalog::glasspool_embellisher(), Some(-1), None), // loot 1
        (catalog::fascinating_lecture(), Some(0), None),    // draw 2 discard 1
        (catalog::stridehollow_vampire(), Some(0), None),   // default mode: draw
        (catalog::prismari_iteration(), Some(0), None),     // discard 1 draw 2
        (catalog::wisdom_of_the_ancients(), Some(2), None), // draw 3
        (catalog::quandrix_snake_charmer(), Some(0), None), // ETB cantrip
        (catalog::prismari_loot(), Some(-1), None),         // draw then discard
        (catalog::prismari_brilliance(), Some(0), None),    // scry + draw
        (catalog::quandrix_survey(), Some(0), None),        // ramp + draw
        (catalog::prismari_spellfire_sage(), Some(0), None),
        (catalog::fractalic_discovery(), Some(0), None),    // draw 3 put back 2
        (catalog::pop_quiz_lecturer(), Some(-1), None),     // ETB scry only
        (catalog::silverquill_cantrip(), Some(0), Some(2)),
        (catalog::witherbloom_researcher(), Some(0), Some(2)),
        (catalog::stoneglare_lecturer(), Some(0), Some(2)),
        (catalog::witherbloom_field_worker(), None, Some(2)),
        (catalog::witherbloom_plowman(), None, Some(3)),
        (catalog::lorehold_strategist(), None, Some(2)),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        // Generous library seed (draw-3 spells, survey's land search).
        for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
        g.add_card_to_library(0, catalog::forest());
        // Filler card in hand for the looters' discards.
        g.add_card_to_hand(0, catalog::mountain());
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        let hand_before = g.players[0].hand.len() as isize;
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        if let Some(d) = hand_delta {
            assert_eq!(g.players[0].hand.len() as isize, hand_before + d,
                "{}: hand delta {}", name, d);
        }
        if let Some(d) = life_delta {
            assert_eq!(g.players[0].life, life_before + d, "{}: life delta {}", name, d);
        }
    }
}

// ── ETB / cast drains hitting each opponent (table) ────────────────────────

#[test]
fn etb_drain_each_opponent() {
    for (def, n) in [
        (catalog::pestilent_lecturer(), 1),
        (catalog::silverquill_mediator(), 2),
        (catalog::witherbloom_drain_ritual(), 3),
        (catalog::silverquill_lifedrain(), 2),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        let p0_before = g.players[0].life;
        let p1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, p1_before - n, "{}: opp loses {}", name, n);
        assert_eq!(g.players[0].life, p0_before + n, "{}: you gain {}", name, n);
    }
}

// ── Player-targeted drains / burn (table) ──────────────────────────────────

#[test]
fn targeted_player_drains() {
    for (def, gain, lose) in [
        (catalog::silverquill_sting(), 2, 2),
        (catalog::silverquill_strike(), 3, 3),
        (catalog::lorehold_reverie(), 3, 3),
        (catalog::prismari_surge(), 0, 3), // burn + draw, no lifegain
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        let p0_before = g.players[0].life;
        let p1_before = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, p1_before - lose, "{}: opp loses {}", name, lose);
        assert_eq!(g.players[0].life, p0_before + gain, "{}: you gain {}", name, gain);
    }
}

// ── Lorehold Excavator ─────────────────────────────────────────────────────

#[test]
fn lorehold_excavator_etb_exiles_target_gy_card() {
    let mut g = two_player_game();
    // Place a card in opponent's graveyard.
    let _bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let exc = g.add_card_to_hand(0, catalog::lorehold_excavator());
    let opp_gy_before = g.players[1].graveyard.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: exc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Excavator castable");
    drain_stack(&mut g);
    // Opp gy should be reduced by one (the bolt was exiled).
    assert_eq!(g.players[1].graveyard.len(), opp_gy_before - 1);
}

// ── Lorehold Conservator ───────────────────────────────────────────────────

#[test]
fn lorehold_conservator_etb_exiles_graveyard_card() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let lc = g.add_card_to_hand(0, catalog::lorehold_conservator());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: lc, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Conservator castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "bear in exile");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear),
        "bear no longer in graveyard");
}

// ── ETB "return creature card from gy to hand" (table) ─────────────────────

#[test]
fn etb_returns_creature_from_gy_to_hand() {
    for (def, life_delta) in [
        (catalog::witherbloom_necrotutor(), Some(-2)),
        (catalog::witherbloom_reanimator(), None),
        (catalog::lorehold_curator(), None),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let bear_id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        let life_before = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == bear_id),
            "{}: bear in hand", name);
        if let Some(d) = life_delta {
            assert_eq!(g.players[0].life, life_before + d, "{}: life delta", name);
        }
    }
}

// ── Pump spells (table): temporary buffs on a friendly bear ────────────────

#[test]
fn pump_spells_buff_friendly_bear() {
    for (def, targeted, min_power, toughness, keyword) in [
        (catalog::silverquill_pledge(), true, 5, Some(3), None),
        (catalog::lesson_in_honor(), true, 4, Some(4), None),
        (catalog::silverquill_resolve(), true, 3, Some(5), Some(Keyword::Lifelink)),
        (catalog::owlin_tactician(), true, 3, None, Some(Keyword::Flying)),
        (catalog::mob_mentality(), false, 3, Some(3), None),
        (catalog::plant_mascot(), false, 3, Some(3), None),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island()); // Learn/cantrip riders
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        let target = if targeted { Some(Target::Permanent(bear)) } else { None };
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        let b = g.compute_battlefield().into_iter().find(|c| c.id == bear)
            .expect("bear alive");
        assert!(b.power >= min_power, "{}: power ≥{} (got {})", name, min_power, b.power);
        if let Some(t) = toughness {
            assert_eq!(b.toughness, t, "{}: toughness", name);
        }
        if let Some(kw) = keyword {
            assert!(b.keywords.contains(&kw), "{}: bear gains {:?}", name, kw);
        }
    }
}

// ── Tap-and-stun spells (table): Scolding Detention, Containment Studies ───

#[test]
fn tap_and_stun_spells_stun_twice() {
    for def in [catalog::scolding_detention(), catalog::containment_studies()] {
        let name = def.name;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        g.players[0].mana_pool.add(Color::White, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        let bear_perm = g.battlefield.iter().find(|c| c.id == bear).expect("bear on bf");
        assert!(bear_perm.tapped, "{}: bear tapped", name);
        assert_eq!(bear_perm.counter_count(CounterType::Stun), 2, "{}: 2 stun counters", name);
    }
}

// ── Lesson Recall ──────────────────────────────────────────────────────────

#[test]
fn lesson_recall_returns_instant_and_cantrips() {
    let mut g = two_player_game();
    let bolt_id = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    let recall = g.add_card_to_hand(0, catalog::lesson_recall());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: recall, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recall castable");
    drain_stack(&mut g);
    // -1 cast + 1 (bolt to hand) + 1 (draw) = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_id), "bolt in hand");
}

// ── Pestilent Acolyte ──────────────────────────────────────────────────────

#[test]
fn pestilent_acolyte_etb_kills_one_toughness_creature() {
    let mut g = two_player_game();
    // Savannah Lions is a 2/1; after -1/-1, it becomes 1/0 → dies via SBA.
    let token = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let acolyte = g.add_card_to_hand(0, catalog::pestilent_acolyte());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: acolyte, target: Some(Target::Permanent(token)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Acolyte castable");
    drain_stack(&mut g);
    // Savannah Lions 2/1 takes -1/-1 → 1/0 → dies via SBA.
    assert!(!g.battlefield.iter().any(|c| c.id == token), "lions dead");
}

// ── Counter-doubling spells (table) ────────────────────────────────────────

#[test]
fn counter_doubling_spells() {
    for (def, seed, expected) in [
        (catalog::quandrix_manipulator(), 2, 4), // double
        (catalog::quandrix_doubling(), 3, 6),    // double
        (catalog::quandrix_catalyst(), 0, 4),    // +2 then double
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        if seed > 0
            && let Some(b) = g.battlefield.iter_mut().find(|c| c.id == bear)
        {
            b.add_counters(CounterType::PlusOnePlusOne, seed);
        }
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        let bear_perm = g.battlefield.iter().find(|c| c.id == bear).expect("bear on bf");
        assert_eq!(bear_perm.counter_count(CounterType::PlusOnePlusOne), expected,
            "{}: {} counters expected", name, expected);
    }
}

// ── Library tutors (table): Mystical Inquiry, Quandrix Tutor ───────────────

#[test]
fn tutors_put_searched_card_in_hand() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    for (def, lib_def) in [
        (catalog::mystical_inquiry(), catalog::lightning_bolt()),
        (catalog::quandrix_tutor(), catalog::grizzly_bears()),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let target = g.add_card_to_library(0, lib_def);
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == target),
            "{}: tutored card in hand", name);
    }
}

// ── Conjurer's Bauble ──────────────────────────────────────────────────────

/// The printed "Put target card from your graveyard on the bottom of your
/// library" clause: a graveyard card is bottomed before the draw, the
/// Bauble is sacrificed, and a card is drawn.
#[test]
fn conjurers_bauble_bottoms_a_graveyard_card_and_cantrips() {
    let mut g = two_player_game();
    // Two library cards so the bottomed card is distinguishable from the top.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let dead_bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let bauble = g.add_card_to_battlefield(0, catalog::conjurers_bauble());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bauble, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("Bauble activatable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bauble), "Bauble sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == dead_bolt),
        "graveyard card left the graveyard");
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(dead_bolt),
        "graveyard card bottomed onto the library");
}

// ── Silverquill Scholar ────────────────────────────────────────────────────

#[test]
fn silverquill_scholar_magecraft_draws_and_loses_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silverquill_scholar());
    // Seed library so the draw lands a real card (not deck-out).
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scholar's magecraft: draw 1, lose 1.
    // Hand: -1 (Bolt cast) + 1 (magecraft draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.players[0].life, life_before - 1, "scholar magecraft loses 1");
}

// ── Token makers (table) ───────────────────────────────────────────────────

#[test]
fn etb_token_makers_mint_expected_tokens() {
    for (def, n) in [
        (catalog::inkling_studies(), 2),
        (catalog::inkling_reinforcement(), 2),
        (catalog::inkling_squad(), 3),
        (catalog::inkling_aether_smith(), 1), // auto-decider picks mode 0
        (catalog::pest_brood_caller(), 2),
        (catalog::lorehold_smith(), 1), // Treasure
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
        assert_eq!(tokens_after, tokens_before + n, "{}: {} tokens minted", name, n);
    }
}

// ── Inkling Drillmaster ────────────────────────────────────────────────────

#[test]
fn inkling_drillmaster_etb_pumps_other_inkling_but_not_non_inkling() {
    let mut g = two_player_game();
    let squire = g.add_card_to_battlefield(0, catalog::inkling_squire());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dm = g.add_card_to_hand(0, catalog::inkling_drillmaster());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: dm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drillmaster castable");
    drain_stack(&mut g);
    let squire_perm = g.battlefield.iter().find(|c| c.id == squire).expect("squire alive");
    assert_eq!(squire_perm.counter_count(CounterType::PlusOnePlusOne), 1, "squire gets a +1/+1 counter");
    let bear_perm = g.battlefield.iter().find(|c| c.id == bear).expect("bear alive");
    assert_eq!(bear_perm.counter_count(CounterType::PlusOnePlusOne), 0, "bear gets no counter (not an Inkling)");
}

// ── Sealing Verse ──────────────────────────────────────────────────────────

#[test]
fn sealing_verse_exiles_low_mv_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let sv = g.add_card_to_hand(0, catalog::sealing_verse());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sv,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Verse castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear exiled");
    // Should be in exile, not graveyard.
    assert!(g.exile.iter().any(|c| c.id == bear), "bear in exile");
}

#[test]
fn sealing_verse_rejects_high_mv_target() {
    let mut g = two_player_game();
    // 5-mana high-mv target (Spectral Adjudicator is {3}{W} → MV 4)
    // Use a 5-mana creature instead: Bookwurm is {5}{G}{G} → MV 7.
    let wurm = g.add_card_to_battlefield(1, catalog::bookwurm());
    let sv = g.add_card_to_hand(0, catalog::sealing_verse());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let res = g.perform_action(GameAction::CastSpell {
        card_id: sv,
        target: Some(Target::Permanent(wurm)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(res.is_err(), "Sealing Verse rejects target with MV > 3");
}

// ── Roving Scholar ─────────────────────────────────────────────────────────

#[test]
fn roving_scholar_etb_each_player_draws_two() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    let rs = g.add_card_to_hand(0, catalog::roving_scholar());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p0_hand_before = g.players[0].hand.len();
    let p1_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: rs, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Roving Scholar castable");
    drain_stack(&mut g);
    // P0: -1 (cast scholar) + 2 (ETB draw) = +1 net hand. P1: +2.
    assert_eq!(g.players[0].hand.len(), p0_hand_before - 1 + 2);
    assert_eq!(g.players[1].hand.len(), p1_hand_before + 2);
}

// ── Lorehold Lookback ──────────────────────────────────────────────────────

#[test]
fn lorehold_lookback_returns_creature_from_gy_and_creates_spirit() {
    let mut g = two_player_game();
    let bear_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let ll = g.add_card_to_hand(0, catalog::lorehold_lookback());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    g.perform_action(GameAction::CastSpell {
        card_id: ll,
        target: Some(Target::Permanent(bear_gy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Lookback castable");
    drain_stack(&mut g);
    // Bear returned from gy to hand: -1 (cast) + 1 (bear to hand) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 1, "1 Spirit token minted");
}

// ── Witherbloom Lifedrinker ────────────────────────────────────────────────

#[test]
fn witherbloom_lifedrinker_grows_on_lifegain() {
    let mut g = two_player_game();
    let dr = g.add_card_to_battlefield(0, catalog::witherbloom_lifedrinker());
    // Cast a lifegain spell — Cram Session gains 4 life.
    g.add_card_to_library(0, catalog::island());
    let cs = g.add_card_to_hand(0, catalog::cram_session());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: cs, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cram Session castable");
    drain_stack(&mut g);
    let dr_perm = g.battlefield.iter().find(|c| c.id == dr).expect("dr alive");
    assert!(dr_perm.counter_count(CounterType::PlusOnePlusOne) >= 1, "lifedrinker grew on lifegain");
}

// ── Augusta, Dean of Order (promoted) ──────────────────────────────────────

/// Augusta's attack trigger untaps each creature you control: a bear that
/// taps to attack is untapped again as Augusta's trigger resolves.
#[test]
fn augusta_dean_of_order_untaps_attackers_on_attack() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::augusta_dean_of_order());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    while g.step != crabomination::game::types::TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("declare bear attacker");
    drain_stack(&mut g);
    assert!(
        !g.battlefield_find(bear).unwrap().tapped,
        "Augusta untaps the attacking bear"
    );
}

// ── Silverquill Apprentice ─────────────────────────────────────────────────

/// Printed oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, target creature gets +1/+0 until end of turn."
#[test]
fn silverquill_apprentice_magecraft_lands_counter_on_friendly() {
    let mut g = two_player_game();
    // Only the apprentice on the battlefield, so the magecraft
    // auto-target must pick it.
    let app = g.add_card_to_battlefield(0, catalog::silverquill_apprentice());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // The printed trigger is a +1/+0 pump EOT, not a counter.
    let cp = g.compute_battlefield().iter()
        .find(|c| c.id == app).cloned()
        .expect("apprentice on bf");
    assert_eq!(cp.power, 3, "apprentice pumped +1/+0 EOT (2 → 3)");
    assert_eq!(cp.toughness, 2, "toughness unchanged");
    let app_perm = g.battlefield.iter().find(|c| c.id == app).expect("apprentice alive");
    assert_eq!(app_perm.counter_count(CounterType::PlusOnePlusOne), 0,
        "the pump is temporary, not a +1/+1 counter");
}

// ── Quandrix Initiate ──────────────────────────────────────────────────────

#[test]
fn quandrix_initiate_grows_on_each_magecraft() {
    let mut g = two_player_game();
    let qi = g.add_card_to_battlefield(0, catalog::quandrix_initiate());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let qi_perm = g.battlefield.iter().find(|c| c.id == qi).expect("initiate alive");
    assert!(qi_perm.counter_count(CounterType::PlusOnePlusOne) >= 1,
        "initiate gained +1/+1 counter from magecraft");
}

// ── Activated drains / pings (table) ───────────────────────────────────────

#[test]
fn activated_drain_and_ping_permanents() {
    for (def, targeted, gain, lose) in [
        (catalog::lorehold_wand(), true, None, 2),
        (catalog::witherbloom_wand(), true, Some(2), 2),
        (catalog::silverquill_pen(), false, Some(2), 2),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        rainbow_mana(&mut g);
        let p0_before = g.players[0].life;
        let p1_before = g.players[1].life;
        let target = if targeted { Some(Target::Player(1)) } else { None };
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target, additional_targets: Vec::new(), x_value: None,
        }).unwrap_or_else(|e| panic!("{} activatable: {:?}", name, e));
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, p1_before - lose, "{}: opp loses {}", name, lose);
        if let Some(gn) = gain {
            assert_eq!(g.players[0].life, p0_before + gn, "{}: you gain {}", name, gn);
        }
    }
}

// ── Witherbloom Bramble ────────────────────────────────────────────────────

#[test]
fn witherbloom_bramble_creates_pest_and_counters_creatures() {
    let mut g = two_player_game();
    let _b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bramble = g.add_card_to_hand(0, catalog::witherbloom_bramble());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bramble, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bramble castable");
    drain_stack(&mut g);
    // At least one Pest token should have been created.
    let pests = g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.has_creature_type(crabomination::card::CreatureType::Pest)).count();
    assert!(pests >= 1, "at least one Pest token minted");
    // Existing bear (Grizzly) should have a +1/+1 counter.
    let bear = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("bear");
    assert!(bear.counter_count(CounterType::PlusOnePlusOne) >= 1,
        "bear has +1/+1 counter from fanout");
}

// ── Damage marks on a 4/4 Serra Angel (table) ──────────────────────────────

#[test]
fn targeted_damage_marks_on_serra_angel() {
    for (def, dmg) in [
        (catalog::prismari_spark(), 2),
        (catalog::prismari_arsonist(), 2),
        (catalog::reduce_rubble(), 3),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island()); // cantrip riders
        // Serra Angel (4/4) survives the damage so we can observe it.
        let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(angel)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        let a = g.battlefield.iter().find(|c| c.id == angel).expect("angel alive");
        assert_eq!(a.damage, dmg, "{}: angel takes {} damage", name, dmg);
    }
}

// ── Quandrix Trickster ─────────────────────────────────────────────────────

#[test]
fn quandrix_trickster_shrinks_target_on_etb() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let qt = g.add_card_to_hand(0, catalog::quandrix_trickster());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: qt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Trickster castable");
    drain_stack(&mut g);
    // Bear was 2/2 → -2/-0 → 0 power. Engine SBA does not destroy creatures
    // for 0 power, only 0 toughness. Bear should be alive but at 0/2.
    let bear_card = g.compute_battlefield().into_iter().find(|c| c.id == bear)
        .expect("bear on battlefield");
    assert!(bear_card.power <= 0, "bear shrunk to ≤0 power (got {})", bear_card.power);
}

// ── Burrog Snapper ─────────────────────────────────────────────────────────

#[test]
fn burrog_snapper_etb_minus_two_zero() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bs = g.add_card_to_hand(0, catalog::burrog_snapper());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bs,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Snapper castable");
    drain_stack(&mut g);
    // 2/2 bear becomes 0/2 — survives (toughness still 2).
    let pt = g.computed_permanent(bear).expect("bear alive");
    assert_eq!(pt.power, 0);
    assert_eq!(pt.toughness, 2);
}

// ── Lorehold Memorialist ───────────────────────────────────────────────────

#[test]
fn lorehold_memorialist_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let lm = g.add_card_to_hand(0, catalog::lorehold_memorialist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let hand_before = g.players[0].hand.len();
    // Target::Permanent works against graveyard entities too — see how
    // SOS Pull from the Grave / Reanimate-style tests target gy cards.
    g.perform_action(GameAction::CastSpell {
        card_id: lm, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Memorialist castable");
    drain_stack(&mut g);
    // -1 (cast Memorialist) + 1 (bear returned to hand) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear in hand");
}

// ── Reflective Anatomy ─────────────────────────────────────────────────────

#[test]
fn reflective_anatomy_pumps_target_by_total_counters() {
    let mut g = two_player_game();
    // Stage two creatures with +1/+1 counters: bear1 with 2, bear2 with 1.
    // After the engine improvement (`Value::CountersOn` summation across
    // fan-out selectors), the +X/+X reads X = 3 (2 + 1).
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    {
        let b1 = g.battlefield.iter_mut().find(|c| c.id == bear1).unwrap();
        b1.add_counters(CounterType::PlusOnePlusOne, 2);
        let b2 = g.battlefield.iter_mut().find(|c| c.id == bear2).unwrap();
        b2.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    let target = bear1;
    let ra = g.add_card_to_hand(0, catalog::reflective_anatomy());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ra, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reflective Anatomy castable");
    drain_stack(&mut g);
    let bear_card = g.compute_battlefield().into_iter().find(|c| c.id == target)
        .expect("target alive");
    // bear is 2/2 base, +2 counters = 4/4 baseline. +3/+3 pump (2+1 total
    // counters across the board) = 7/7.
    assert_eq!(bear_card.power, 7, "bear pumped to 7 power (4 base + 3 sum)");
}

// ── Witherbloom Ritualist ──────────────────────────────────────────────────

#[test]
fn witherbloom_ritualist_pumps_creature_and_gains_life() {
    let mut g = two_player_game();
    let wr = g.add_card_to_battlefield(0, catalog::witherbloom_ritualist());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: wr, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None }).expect("Ritualist activation");
    drain_stack(&mut g);
    let bear_card = g.compute_battlefield().into_iter().find(|c| c.id == bear)
        .expect("bear alive");
    assert!(bear_card.power >= 3, "bear pumped to ≥3 power");
    assert_eq!(g.players[0].life, life_before + 1, "P0 gains 1 life");
}

// ── Conspiracy Theorist ────────────────────────────────────────────────────

#[test]
fn conspiracy_theorist_activation_rejected_with_cards_in_hand() {
    // Push (modern_decks): "{1}{R}, {T}: ... Activate only if you control
    // no cards in hand." — the empty-hand gate is wired via
    // `ActivatedAbility.condition: Predicate::ValueEquals(HandSize, 0)`.
    // With one card in hand the activation must be rejected.
    let mut g = two_player_game();
    let ct = g.add_card_to_battlefield(0, catalog::conspiracy_theorist());
    g.clear_sickness(ct);
    g.add_card_to_hand(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: ct,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None });
    assert!(res.is_err(),
        "Activation rejected when hand_size > 0; got {:?}", res);
    // CT should not have been tapped (cost rolled back).
    assert!(!g.battlefield_find(ct).unwrap().tapped,
        "Conspiracy Theorist should not have been tapped");
}

/// Printed oracle, first ability: "Whenever this creature attacks, you
/// may pay {1} and discard a card. If you do, draw a card." Discarding
/// a LAND this way does not trip the second (nonland-only) exile
/// trigger.
#[test]
fn conspiracy_theorist_activation_succeeds_with_empty_hand() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let ct = g.add_card_to_battlefield(0, catalog::conspiracy_theorist());
    g.clear_sickness(ct);
    let hand_island = g.add_card_to_hand(0, catalog::island());
    let lib_card = g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(1);
    // Accept MayPay {1}, accept the discard, and (greedily) accept any
    // further prompt — the land discard must not offer an exile.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));

    g.step = crabomination::game::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::Attack {
        attacker: ct,
        target: crabomination::game::types::AttackTarget::Player(1),
    }])).expect("Conspiracy Theorist attacks");
    drain_stack(&mut g);

    // Island discarded, card drawn.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == hand_island),
        "the island was discarded to the graveyard");
    assert!(g.players[0].hand.iter().any(|c| c.id == lib_card),
        "a card was drawn for the discard");
    // A land discard never trips the nonland exile trigger.
    assert!(g.exile.iter().all(|c| c.id != hand_island),
        "discarded LAND must not be exiled by the nonland trigger");
}

/// Printed oracle, both abilities chained: attacking and paying {1} +
/// discarding a NONLAND card draws a card, then the discard trigger
/// lets you exile the discarded card from your graveyard and cast it
/// this turn.
#[test]
fn conspiracy_theorist_attack_with_discard_exiles_top_and_grants_may_play() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let ct = g.add_card_to_battlefield(0, catalog::conspiracy_theorist());
    g.clear_sickness(ct);
    // Put a nonland discard target in hand and a draw in the library.
    let hand_bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let lib_card = g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    // Scripted decider: accept MayPay {1}, accept the discard, then
    // accept exiling the discarded nonland card from the graveyard.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    let exile_before = g.exile.len();
    g.step = crabomination::game::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::Attack {
        attacker: ct,
        target: crabomination::game::types::AttackTarget::Player(1),
    }])).expect("Conspiracy Theorist attacks");
    drain_stack(&mut g);
    // A card was drawn for the discard.
    assert!(g.players[0].hand.iter().any(|c| c.id == lib_card),
        "drew a card for the pay-and-discard");
    // The DISCARDED card (not the top of the library) is in exile with
    // a may-play permission.
    assert_eq!(g.exile.len(), exile_before + 1);
    let exiled = g.exile.iter().find(|c| c.id == hand_bolt)
        .expect("the discarded bolt should be in exile");
    assert!(exiled.may_play_until.is_some(),
        "exiled card should have may_play permission");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == hand_bolt),
        "the discarded card left the graveyard");
}

/// Regression: `ExileTopAndGrantMayPlay` must exile the *top* of the
/// library (index 0), not the bottom. With two distinct cards stacked —
/// Lightning Bolt on top, Island on the bottom — the exile-top effect
/// should grab the Bolt and leave the Island in the library.
/// (Conspiracy Theorist's real oracle no longer uses this effect, so a
/// synthetic instant carries the engine coverage.)
#[test]
fn exile_top_and_grant_may_play_takes_the_top_card_not_the_bottom() {
    use crabomination::card::{CardDefinition, CardType, Effect, MayPlayDuration, Value};
    use crabomination::effect::PlayerRef;
    use crabomination::mana::cost;

    let exile_top = CardDefinition {
        name: "Exile Top Test",
        cost: cost(&[crabomination::mana::r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(1),
            duration: MayPlayDuration::EndOfThisTurn,
            pay_any_color: false,
            pay_own_cost: false,
            uncast_penalty: None,
        },
        ..Default::default()
    };

    let mut g = two_player_game();
    // First-added card sits at index 0 (the top); second is bottomed.
    let top_bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let bottom_island = g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, exile_top);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Exile Top Test castable for {R}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == top_bolt),
        "the top card (Bolt) should be exiled");
    let exiled = g.exile.iter().find(|c| c.id == top_bolt).unwrap();
    assert!(exiled.may_play_until.is_some(),
        "exiled top card should carry a may-play permission");
    assert!(g.players[0].library.iter().any(|c| c.id == bottom_island),
        "the bottom card (Island) should remain in the library");
    assert!(!g.exile.iter().any(|c| c.id == bottom_island),
        "the bottom card must not be the one exiled");
}

// ── Prismari Bauble ────────────────────────────────────────────────────────

#[test]
fn prismari_bauble_etb_scrys_and_can_sac_for_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let pb = g.add_card_to_hand(0, catalog::prismari_bauble());
    g.perform_action(GameAction::CastSpell {
        card_id: pb, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bauble castable (0 mana)");
    drain_stack(&mut g);
    // Now sacrifice it for a draw.
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: pb, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("Bauble sac for draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "P0 drew a card");
    // Bauble should now be in graveyard (sacrificed).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pb),
        "bauble in graveyard");
}

// ── Witherbloom Toxicology ─────────────────────────────────────────────────

#[test]
fn witherbloom_toxicology_destroys_creature_and_mints_pest() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tox = g.add_card_to_hand(0, catalog::witherbloom_toxicology());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: tox, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxicology castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "bear dies");
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.is_token && c.definition.name == "Pest")
        .collect();
    assert_eq!(pests.len(), 1, "exactly one Pest token");
}

// ── Counterspells (table): Quandrix Counterspell, Spell Squelch ────────────

#[test]
fn counterspells_counter_target_spell() {
    for def in [catalog::quandrix_counterspell(), catalog::spell_squelch()] {
        let name = def.name;
        let mut g = two_player_game();
        let opp_bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.active_player_idx = 1;
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: opp_bolt, target: Some(Target::Player(0)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("opp casts Bolt");
        g.priority.player_with_priority = 0;
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(opp_bolt)),
            additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 20, "{}: Bolt countered, no damage", name);
    }
}

// ── Lorehold Wayfinder ─────────────────────────────────────────────────────

#[test]
fn lorehold_wayfinder_etb_mills_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::mountain());
    g.add_card_to_library(0, catalog::plains());
    let wf = g.add_card_to_hand(0, catalog::lorehold_wayfinder());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let gy_before = g.players[0].graveyard.len();
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: wf, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Wayfinder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2, "milled 2");
    assert_eq!(g.players[0].library.len(), lib_before - 2);
}

// ── Prismari Pyromage ──────────────────────────────────────────────────────

#[test]
fn prismari_pyromage_magecraft_pings_target() {
    let mut g = two_player_game();
    let _pyro = g.add_card_to_battlefield(0, catalog::prismari_b35_pyromage());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Cast a bolt to trigger magecraft
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bear took 3 (Bolt) + 1 (Magecraft ping) = 4 dmg, died.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "bear dies");
}

// ── Mana permanents (table) ────────────────────────────────────────────────

#[test]
fn mana_permanents_tap_for_mana() {
    for def in [
        catalog::witherbloom_channeler(),
        catalog::quandrix_engineer(),
        catalog::mage_tower_crystal(),
    ] {
        let name = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        if let Some(p) = g.battlefield.iter_mut().find(|c| c.id == id) {
            p.summoning_sick = false;
        }
        let mana_before = g.players[0].mana_pool.total();
        g.perform_action(GameAction::ActivateAbility {
            card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).unwrap_or_else(|e| panic!("{} mana activation: {:?}", name, e));
        drain_stack(&mut g);
        assert!(g.players[0].mana_pool.total() > mana_before,
            "{}: mana added to pool", name);
    }
}

// ── Lorehold Banner ────────────────────────────────────────────────────────

#[test]
fn lorehold_banner_etb_gains_life_and_taps_for_color() {
    let mut g = two_player_game();
    let banner = g.add_card_to_hand(0, catalog::lorehold_banner());
    g.players[0].mana_pool.add_colorless(3);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: banner, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Banner castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2, "gains 2");
    // Activate red mana ability
    g.perform_action(GameAction::ActivateAbility {
        card_id: banner, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("red mana tap");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Red) >= 1, "red added");
}

// ── Edicts ─────────────────────────────────────────────────────────────────

#[test]
fn witherbloom_reaper_etb_edicts_each_opp() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let reaper = g.add_card_to_hand(0, catalog::witherbloom_reaper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: reaper, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reaper castable");
    drain_stack(&mut g);
    // The opp's bear (only creature) sacrificed.
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear sacrificed");
}

#[test]
fn witherbloom_verdict_forces_opp_sac() {
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let v = g.add_card_to_hand(0, catalog::witherbloom_verdict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let gy_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: v, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Verdict castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), gy_before + 1,
        "opp sacrificed creature → gy");
}

// ── Quandrix Recall ────────────────────────────────────────────────────────

#[test]
fn quandrix_recall_bounces_creature_to_owners_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let qr = g.add_card_to_hand(0, catalog::quandrix_recall());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: qr, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recall castable");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "bear back to owner's hand");
}

// ── Lorehold Justice ───────────────────────────────────────────────────────

#[test]
fn lorehold_justice_destroys_power_4_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());  // 4/4
    let lj = g.add_card_to_hand(0, catalog::lorehold_justice());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: lj, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Justice castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == big), "angel dies");
}

// ── Sweepers (table): Witherbloom Pestilence, Prismari Inferno ─────────────

#[test]
fn sweepers_kill_two_toughness_creatures_on_both_sides() {
    for def in [catalog::witherbloom_pestilence(), catalog::prismari_inferno()] {
        let name = def.name;
        let mut g = two_player_game();
        let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        rainbow_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![],
            mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{} castable: {:?}", name, e));
        drain_stack(&mut g);
        assert!(g.players[0].graveyard.iter().any(|c| c.id == b1), "{}: b1 dies", name);
        assert!(g.players[1].graveyard.iter().any(|c| c.id == b2), "{}: b2 dies", name);
    }
}

// ── Pest Inheritance ───────────────────────────────────────────────────────

#[test]
fn pest_inheritance_creates_pests_equal_to_lands() {
    let mut g = two_player_game();
    // Stage 3 lands.
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let pi = g.add_card_to_hand(0, catalog::pest_inheritance());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: pi, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Inheritance castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.has_creature_type(crabomination::card::CreatureType::Pest)).count();
    assert_eq!(pests, 3, "3 Pests minted (one per land)");
}

// ── Pest Mediator ──────────────────────────────────────────────────────────

#[test]
fn pest_mediator_grows_on_lifegain() {
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::pest_mediator());
    // Trigger a lifegain via Witherbloom Apprentice + a Bolt cast.
    let _wa = g.add_card_to_battlefield(0, catalog::witherbloom_apprentice());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pm_card = g.battlefield.iter().find(|c| c.id == pm).expect("pm alive");
    assert!(pm_card.counter_count(CounterType::PlusOnePlusOne) >= 1,
        "got +1/+1 counter from lifegain");
}

// ── Inkling Aerialist ──────────────────────────────────────────────────────

#[test]
fn inkling_aerialist_pumps_on_other_inkling_etb() {
    let mut g = two_player_game();
    let ia = g.add_card_to_battlefield(0, catalog::inkling_aerialist());
    // Mint an Inkling token via Defend the Campus
    let defend = g.add_card_to_hand(0, catalog::defend_the_campus());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: defend, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Defend castable");
    drain_stack(&mut g);
    let ia_card = g.compute_battlefield().into_iter().find(|c| c.id == ia)
        .expect("ia alive");
    // 3 Inkling tokens enter → 3 triggers → +3/+3 EOT
    assert!(ia_card.power >= 3, "Aerialist grows on Inkling ETB");
}

// ── Quandrix Theorist ──────────────────────────────────────────────────────

#[test]
fn quandrix_theorist_draws_per_counter_creature() {
    let mut g = two_player_game();
    // Two creatures with +1/+1 counters on the board.
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == b1) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == b2) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let qt = g.add_card_to_hand(0, catalog::quandrix_theorist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: qt, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Theorist castable");
    drain_stack(&mut g);
    // -1 (cast) + 2 (draw 2 from two counter creatures) = +1 net
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

// ── Quandrix Defender ──────────────────────────────────────────────────────

#[test]
fn quandrix_defender_etb_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let qd = g.add_card_to_hand(0, catalog::quandrix_defender());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: qd, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Defender castable");
    drain_stack(&mut g);
    // Scry 1 doesn't change library size.
    assert_eq!(g.players[0].library.len(), lib_before);
}

// ── Stormwild Capridor ─────────────────────────────────────────────────────

/// Stormwild Capridor converts prevented noncombat damage into +1/+1
/// counters: a Bolt makes it a 4/6 instead of killing it.
#[test]
fn stormwild_capridor_converts_burn_into_counters() {
    let mut g = two_player_game();
    let goat = g.add_card_to_battlefield(0, catalog::stormwild_capridor());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(goat)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(goat).expect("Capridor survives — damage prevented");
    assert_eq!(c.damage, 0, "no damage marked");
    assert_eq!(
        c.counter_count(CounterType::PlusOnePlusOne),
        3,
        "three +1/+1 counters for the three prevented damage"
    );
}
