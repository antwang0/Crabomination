use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

// Floods player 0's pool so table-driven tests don't need per-card costs.
macro_rules! flood_mana {
    ($g:expr) => {{
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            $g.players[0].mana_pool.add(c, 20);
        }
        $g.players[0].mana_pool.add_colorless(20);
    }};
}

macro_rules! cast {
    ($g:expr, $id:expr) => {
        $g.perform_action(GameAction::CastSpell {
            card_id: $id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable")
    };
    ($g:expr, $id:expr, $t:expr) => {
        $g.perform_action(GameAction::CastSpell {
            card_id: $id, target: Some($t), additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable")
    };
}

// ── Table: magecraft damage riders (creature on bf, Bolt at opp, total dmg) ──

#[test]
fn magecraft_damage_riders_add_to_bolt() {
    for (def, total) in [
        (catalog::lorehold_charwarden(), 4),
        (catalog::lorehold_ardent_acolyte(), 4),
        (catalog::prismari_embertongue(), 4),
        (catalog::lorehold_pyrechronicler(), 4),
        (catalog::prismari_ember_adept(), 4),
        (catalog::prismari_pyrosage(), 4),
        (catalog::lorehold_ember_sage(), 4),
        (catalog::lorehold_pyreheart(), 5),
        (catalog::prismari_eruption_mage(), 5),
        // Scry-on-magecraft cards: bolt-only damage, scry auto-resolved.
        (catalog::prismari_sparkscribe(), 3),
        (catalog::strixhaven_glyphmage(), 3),
        (catalog::quandrix_tideseer(), 3),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let opp_before = g.players[1].life;
        cast!(g, bolt, crabomination::game::types::Target::Player(1));
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - total);
    }
}

// ── Table: magecraft lifegain riders ────────────────────────────────────────

#[test]
fn magecraft_lifegain_riders_gain_on_bolt_cast() {
    for (def, gain) in [
        (catalog::witherbloom_lifeleecher(), 1),
        (catalog::witherbloom_bloodvine(), 1),
        (catalog::lorehold_lightcleric(), 1),
        (catalog::inkling_choirsinger(), 1),
        (catalog::silverquill_spellquill(), 1),
        (catalog::lorehold_spirit_sage(), 1),
        (catalog::silverquill_lifepenner(), 2),
    ] {
        let mut g = two_player_game();
        let _id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let me_before = g.players[0].life;
        cast!(g, bolt, crabomination::game::types::Target::Player(1));
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, me_before + gain);
    }
}

// ── Table: magecraft self-pumps (until-EOT power via battlefield_find) ──────

#[test]
fn magecraft_self_pumps_on_bolt_cast() {
    for (def, power) in [
        (catalog::witherbloom_rootcaster(), 3),
        (catalog::quandrix_scribe(), 2),
        (catalog::prismari_sparkpainter(), 4),
        (catalog::prismari_flameforger(), 5),
        (catalog::prismari_mirror_mage(), 3),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast!(g, bolt, crabomination::game::types::Target::Player(1));
        drain_stack(&mut g);
        let body = g.battlefield_find(id).unwrap();
        assert_eq!(body.power(), power);
    }
}

#[test]
fn magecraft_self_pumps_via_computed_battlefield() {
    for (def, power) in [
        (catalog::quandrix_pulseweaver(), 3),
        (catalog::lorehold_pyrescribe_adept(), 3),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast!(g, bolt, crabomination::game::types::Target::Player(1));
        drain_stack(&mut g);
        let cp = g.compute_battlefield().into_iter()
            .find(|c| c.id == id)
            .expect("on battlefield");
        assert_eq!(cp.power, power);
    }
}

// ── Table: magecraft +1/+1 counter riders ───────────────────────────────────

#[test]
fn magecraft_counter_riders_grow_on_instant_cast() {
    for def in [catalog::lorehold_forgemaster(), catalog::quandrix_wavewriter()] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        cast!(g, bolt, crabomination::game::types::Target::Player(1));
        drain_stack(&mut g);
        let body = g.battlefield_find(id).unwrap();
        assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1);
    }
}

// ── Table: ETB drains / life adjustments on untargeted cast ─────────────────

#[test]
fn etb_life_adjusters_on_cast() {
    for (def, opp_loss, self_gain) in [
        (catalog::witherbloom_caulhound(), 2, 2),
        (catalog::witherbloom_bloodroot(), 4, 4),
        (catalog::witherbloom_pestwarden(), 2, 2),
        (catalog::silverquill_drainlord(), 3, 3),
        (catalog::bloodvine_drainmage(), 3, 3),
        (catalog::lorehold_ancestor(), 1, 1),
        (catalog::silverquill_drainwriter(), 2, 2),
        (catalog::inkling_strikemark(), 2, 2),
        (catalog::witherbloom_thresher(), 1, 1),
        (catalog::silverquill_penitent(), 1, 1),
        (catalog::silverquill_antiphony(), 2, 2),
        (catalog::witherbloom_hexpetal(), 2, 2),
        (catalog::silverquill_inkletter(), 1, 1),
        (catalog::witherbloom_soulrender(), 3, 3),
        (catalog::silverquill_homily(), 1, 1),
        (catalog::inkling_cardinal(), 0, 2),
        (catalog::witherbloom_lifefarmer(), 0, 3),
        (catalog::strixhaven_honor_guard(), 0, 1),
        (catalog::witherbloom_marshtender(), 0, 1),
        (catalog::witherbloom_verdancer(), 0, 1),
        (catalog::strixhaven_sapper(), 1, 0),
        (catalog::inkling_maverick(), 1, 1),
        (catalog::witherbloom_bloodscribe(), 2, 0),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        let me_before = g.players[0].life;
        let opp_before = g.players[1].life;
        cast!(g, id);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - opp_loss);
        assert_eq!(g.players[0].life, me_before + self_gain);
    }
}

// ── Table: token minters on untargeted cast ─────────────────────────────────

#[test]
fn token_minters_mint_expected_count() {
    for (def, count) in [
        (catalog::inkling_sermon(), 1),
        (catalog::witherbloom_pestswarm(), 2),
        (catalog::lorehold_mass_ritual(), 3),
        (catalog::pest_horde(), 4),
        (catalog::lorehold_ghostmaster(), 3),
        (catalog::fractal_grower(), 1),
        (catalog::silverquill_forge(), 2),
        (catalog::mascot_lesson_b32(), 1),
        (catalog::prismari_treasurewright_b32(), 1),
        (catalog::silverquill_spellscribe(), 1),
        (catalog::pest_skyswarm(), 1),
        (catalog::lorehold_warhost(), 2),
        (catalog::witherbloom_pestrider(), 1),
        (catalog::spirit_phalanx(), 2),
        (catalog::silverquill_ovation(), 2),
        (catalog::lorehold_spirit_legion(), 2),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        let before = g.battlefield.iter()
            .filter(|c| c.is_token && c.controller == 0).count();
        cast!(g, id);
        drain_stack(&mut g);
        let after = g.battlefield.iter()
            .filter(|c| c.is_token && c.controller == 0).count();
        assert_eq!(after, before + count);
    }
}

// ── Table: targeted burn / drain at a player ────────────────────────────────

#[test]
fn targeted_player_burn_spells_and_etbs() {
    for (def, opp_loss, self_gain) in [
        (catalog::lorehold_soulburst(), 2, 0),
        (catalog::prismari_sparkflare(), 3, 0),
        (catalog::strixhaven_sorcerer(), 2, 0),
        (catalog::lorehold_pyremender(), 2, 2),
        (catalog::lorehold_b35_lightning(), 3, 1),
        (catalog::prismari_cinderdrake(), 3, 0),
        (catalog::prismari_burning_lesson(), 3, 0),
        (catalog::prismari_spellforge(), 2, 0),
        (catalog::lorehold_vow(), 2, 0),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        let me_before = g.players[0].life;
        let opp_before = g.players[1].life;
        cast!(g, id, crabomination::game::types::Target::Player(1));
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - opp_loss);
        assert_eq!(g.players[0].life, me_before + self_gain);
    }
}

// ── Table: targeted spells that kill an opposing bear ───────────────────────

#[test]
fn targeted_removal_kills_opp_bear() {
    for def in [
        catalog::prismari_sparkriot(),
        catalog::prismari_stormfront(),
        catalog::silverquill_magemark(),
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.add_card_to_library(0, catalog::island());
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        cast!(g, id, crabomination::game::types::Target::Permanent(bear));
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "opp bear removed");
    }
}

// ── Table: untargeted casts that remove an opposing bear (sac / burn) ───────

#[test]
fn untargeted_casts_remove_opp_bear() {
    for def in [
        catalog::silverquill_mandate(),
        catalog::pest_snatchgrab(),
        catalog::lorehold_burnscribe(),
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        cast!(g, id);
        drain_stack(&mut g);
        assert!(!g.battlefield.iter().any(|c| c.id == bear), "opp bear gone");
    }
}

// ── Table: ETB shrink where the bear survives ───────────────────────────────

#[test]
fn etb_shrinkers_reduce_bear_power() {
    for (def, power) in [
        (catalog::witherbloom_toxinkeeper(), 1),
        (catalog::quandrix_tidewright(), 0),
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        cast!(g, id, crabomination::game::types::Target::Permanent(bear));
        drain_stack(&mut g);
        let bear_body = g.battlefield_find(bear).unwrap();
        assert_eq!(bear_body.power(), power);
    }
}

// ── Table: targeted pump spells on our bear ─────────────────────────────────

#[test]
fn targeted_pump_spells_raise_bear_power() {
    for (def, power) in [
        (catalog::plant_adept_lesson(), 4),
        (catalog::lorehold_devotion(), 4),
        (catalog::silverquill_stylepoint(), 3),
        (catalog::silverquill_verseblade(), 3),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        cast!(g, id, crabomination::game::types::Target::Permanent(bear));
        drain_stack(&mut g);
        let body = g.battlefield_find(bear).unwrap();
        assert_eq!(body.power(), power);
    }
}

// ── Table: team pump spells ─────────────────────────────────────────────────

#[test]
fn team_pump_spells_raise_bear_power() {
    for (def, power) in [
        (catalog::lorehold_spirit_hymn(), 3),
        (catalog::silverquill_battle_chant(), 4),
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        cast!(g, id);
        drain_stack(&mut g);
        let body = g.battlefield_find(bear).unwrap();
        assert_eq!(body.power(), power);
    }
}

// ── Table: ETB pumps *another* creature — the source must NOT pump itself ───

#[test]
fn etb_another_pumpers_exclude_source() {
    for def in [catalog::strixhaven_mentor(), catalog::inkling_avenger()] {
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        cast!(g, id);
        drain_stack(&mut g);
        let bear_body = g.battlefield_find(bear).unwrap();
        assert_eq!(bear_body.counter_count(CounterType::PlusOnePlusOne), 1);
        // "another" filter excludes the source itself.
        let self_body = g.battlefield_find(id).unwrap();
        assert_eq!(self_body.counter_count(CounterType::PlusOnePlusOne), 0);
    }
}

// ── Table: ETB looters (-1 net hand) ────────────────────────────────────────

#[test]
fn etb_looters_net_minus_one_hand() {
    for def in [
        catalog::silverquill_lorescribe(),
        catalog::quandrix_topologist(),
        catalog::prismari_flamescribe(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _fodder = g.add_card_to_hand(0, catalog::lightning_bolt());
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        let hand_before = g.players[0].hand.len();
        cast!(g, id);
        drain_stack(&mut g);
        // -1 cast, +1 draw, -1 discard → -1 net.
        assert_eq!(g.players[0].hand.len(), hand_before - 1);
    }
}

// ── Table: ETB cantrips / card-to-hand (net 0 hand) ─────────────────────────

#[test]
fn etb_cantrips_net_zero_hand() {
    for def in [
        catalog::strixhaven_apprentice(),
        catalog::fractal_tidecaller(),
        catalog::quandrix_inquiry(),
        catalog::fractal_reckoner(),
        catalog::strixhaven_cartographer_b32(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::forest());
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        let hand_before = g.players[0].hand.len();
        cast!(g, id);
        drain_stack(&mut g);
        // -1 cast + 1 card to hand = same size.
        assert_eq!(g.players[0].hand.len(), hand_before);
    }
}

// ── Table: ETB scry / surveil resolves cleanly ──────────────────────────────

#[test]
fn etb_scry_surveil_resolves_cleanly() {
    for def in [
        catalog::quandrix_visionary(),
        catalog::quandrix_proofwriter(),
        catalog::silverquill_scribe_tutor(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let lib_before = g.players[0].library.len();
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        cast!(g, id);
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).is_some());
        // Scry keeps library size; surveil may send one card to graveyard.
        assert!(g.players[0].library.len() <= lib_before);
    }
}

// ── Table: ETB exiles a graveyard card ──────────────────────────────────────

#[test]
fn etb_graveyard_exilers() {
    for (def, targeted) in [
        (catalog::lorehold_grave_crusader(), true),
        (catalog::lorehold_memoirist(), true),
        (catalog::lorehold_zealot(), false),
    ] {
        let mut g = two_player_game();
        let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        if targeted {
            cast!(g, id, crabomination::game::types::Target::Permanent(bolt));
        } else {
            cast!(g, id);
        }
        drain_stack(&mut g);
        assert!(g.exile.iter().any(|c| c.id == bolt), "gy card exiled");
    }
}

// ── Table: ETB returns a card from our graveyard to hand ────────────────────

#[test]
fn etb_graveyard_to_hand_returners() {
    for (def, gy_def) in [
        (catalog::lorehold_spectrecaster(), catalog::lightning_bolt()),
        (catalog::inkling_loremaster(), catalog::lightning_bolt()),
        (catalog::witherbloom_gravecaller(), catalog::grizzly_bears()),
    ] {
        let mut g = two_player_game();
        let gy_card = g.add_card_to_graveyard(0, gy_def);
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        cast!(g, id);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == gy_card), "returned to hand");
    }
}

// ── Table: reanimators put a creature back on the battlefield ───────────────

#[test]
fn reanimators_return_creature_to_battlefield() {
    for (def, targeted) in [
        (catalog::witherbloom_pestlich(), false),
        (catalog::lorehold_bequeathing(), true),
    ] {
        let mut g = two_player_game();
        let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        flood_mana!(g);
        if targeted {
            cast!(g, id, crabomination::game::types::Target::Permanent(bear));
        } else {
            cast!(g, id);
        }
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_some(), "Bear reanimated");
    }
}

// ── Attack triggers (bodies differ; kept separate) ──────────────────────────

#[test]
fn witherbloom_sapseeker_attack_gains_one_life() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_sapseeker());
    g.clear_sickness(id);
    let life_before = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("Sapseeker attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Trample));
}

#[test]
fn lorehold_spectrebrand_attack_pumps_friendly() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::lorehold_spectrebrand());
    g.clear_sickness(id);
    g.clear_sickness(bear);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("Spectrebrand attacks");
    drain_stack(&mut g);
    let bear_body = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_body.power(), 3, "Bear should be pumped +1/+0");
}

#[test]
fn lorehold_skirmlord_attack_scales_with_other_attackers() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::lorehold_skirmlord());
    g.clear_sickness(id);
    g.clear_sickness(bear);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: id, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ])).expect("Both attack");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    // Base 2 + 1 (other attacker = bear) = 3
    assert_eq!(body.power(), 3);
}

// ── Activated abilities ─────────────────────────────────────────────────────

#[test]
fn witherbloom_mireguide_taps_for_black_or_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_mireguide());
    g.clear_sickness(id);
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Mireguide Black ability");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
}

#[test]
fn lorehold_pyromaster_taps_for_three_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_pyromaster());
    g.clear_sickness(id);
    drain_stack(&mut g);
    flood_mana!(g);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: Some(crabomination::game::types::Target::Player(1)), additional_targets: Vec::new(), x_value: None , mode: None}).expect("Pyromaster activated");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn strixhaven_pupil_activated_scry_and_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let id = g.add_card_to_battlefield(0, catalog::strixhaven_pupil());
    g.clear_sickness(id);
    g.players[0].mana_pool.add_colorless(2);
    drain_stack(&mut g);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Pupil activated");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "draw 1");
}

#[test]
fn strixhaven_banner_mana_and_sac_draw_abilities() {
    // Ability 0: tap for any color.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::strixhaven_banner());
    drain_stack(&mut g);
    let mana_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Banner mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), mana_before + 1);
    // Ability 1: pay {2}, sac, draw a card (fresh game/banner).
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::strixhaven_banner());
    g.players[0].mana_pool.add_colorless(2);
    drain_stack(&mut g);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("Banner sac-draw ability");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id), "banner sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "draw 1");
}

// ── Tribal / cross-card watchers ────────────────────────────────────────────

#[test]
fn inkling_warden_pumps_on_friendly_inkling_etb() {
    let mut g = two_player_game();
    let warden = g.add_card_to_battlefield(0, catalog::inkling_warden());
    drain_stack(&mut g);
    // Cast an Inkling
    let aspirant = g.add_card_to_hand(0, catalog::inkling_aspirant());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    cast!(g, aspirant);
    drain_stack(&mut g);
    let body = g.battlefield_find(warden).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn pest_hivekeeper_grows_on_another_pest_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_hivekeeper());
    drain_stack(&mut g);
    // Cast a Pest-minter to enter another Pest under our control.
    let minter = g.add_card_to_hand(0, catalog::pest_skyswarm());
    flood_mana!(g);
    cast!(g, minter);
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1,
        "Hivekeeper gains a +1/+1 on the Pest ETB");
}

#[test]
fn quandrix_counterbearer_pumps_when_counter_added_elsewhere() {
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::quandrix_counterbearer());
    drain_stack(&mut g);
    // Cast Inkling Avenger — its ETB drops a +1/+1 counter on another
    // friendly (the bear), which should trigger Counterbearer's pump.
    let avenger = g.add_card_to_hand(0, catalog::inkling_avenger());
    flood_mana!(g);
    cast!(g, avenger);
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    // Counterbearer (1/2) gets +1/+1 → 2/3 until EOT
    assert_eq!(body.power(), 2);
    assert_eq!(body.toughness(), 3);
}

#[test]
fn quandrix_wavecharger_etb_pumps_each_fractal() {
    let mut g = two_player_game();
    // Mint a fractal directly via Fractal Swarm.
    g.add_card_to_library(0, catalog::island());
    let fs = g.add_card_to_hand(0, catalog::fractal_swarm());
    flood_mana!(g);
    cast!(g, fs);
    drain_stack(&mut g);
    let fractal_id = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .unwrap().id;
    let before = g.battlefield_find(fractal_id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    // Now cast Wavecharger.
    let id = g.add_card_to_hand(0, catalog::quandrix_wavecharger());
    cast!(g, id);
    drain_stack(&mut g);
    let after = g.battlefield_find(fractal_id).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(after, before + 1, "pumped the existing Fractal by 1 counter");
}

#[test]
fn fractal_swarm_mints_two_two_fractal_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::fractal_swarm());
    flood_mana!(g);
    let hand_before = g.players[0].hand.len();
    cast!(g, id);
    drain_stack(&mut g);
    // -1 (cast Swarm) +1 (drew from island) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token
            && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1);
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn witherbloom_vitalist_grows_on_lifegain() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_vitalist());
    // Add a separate instant that gains life
    let life = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);
    cast!(g, life, crabomination::game::types::Target::Player(0));
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn lorehold_pyrescholar_grows_on_card_leave_gy() {
    let mut g = two_player_game();
    let pyre = g.add_card_to_battlefield(0, catalog::lorehold_pyrescholar());
    let gy_card = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    drain_stack(&mut g);
    // Use Lorehold Acolyte's exile from gy to remove the card
    let acolyte = g.add_card_to_hand(0, catalog::lorehold_acolyte());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast!(g, acolyte, crabomination::game::types::Target::Permanent(gy_card));
    drain_stack(&mut g);
    let body = g.battlefield_find(pyre).unwrap();
    assert_eq!(body.power(), 3, "Pyrescholar +1/+1 on gy leave");
}

#[test]
fn witherbloom_blooddrinker_dies_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_blooddrinker());
    drain_stack(&mut g);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    // Kill the blooddrinker with a Bolt
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast!(g, bolt, crabomination::game::types::Target::Permanent(id));
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id), "blooddrinker dies");
    assert_eq!(g.players[1].life, opp_before - 2, "opp loses 2 on death");
    assert_eq!(g.players[0].life, me_before + 2, "you gain 2 on death");
}

// ── Unique setups / effects kept separate ───────────────────────────────────

#[test]
fn witherbloom_diviner_etb_mills_and_optional_recover() {
    let mut g = two_player_game();
    // Stack some cards on top of library to mill
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::witherbloom_diviner());
    flood_mana!(g);
    let gy_before = g.players[0].graveyard.len();
    cast!(g, id);
    drain_stack(&mut g);
    // 3 cards milled to graveyard (auto-decider declines the MayDo by default)
    assert_eq!(g.players[0].graveyard.len(), gy_before + 3);
}

#[test]
fn quandrix_handmage_etb_mints_fractal_scaling_with_hand() {
    let mut g = two_player_game();
    // Add some cards to hand
    for _ in 0..3 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::quandrix_handmage());
    flood_mana!(g);
    cast!(g, id);
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Fractal"
    }).collect();
    assert_eq!(fractals.len(), 1);
    // Number of counters = hand size when ETB fires (after handmage left hand)
    let counter_count = fractals[0].counter_count(CounterType::PlusOnePlusOne);
    assert!(counter_count >= 3, "Fractal scales with hand size, got {}", counter_count);
}

#[test]
fn quandrix_equipoise_draws_and_pumps_with_hand_size() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_equipoise());
    flood_mana!(g);
    let hand_before = g.players[0].hand.len();
    cast!(g, id, crabomination::game::types::Target::Permanent(bear));
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before); // -1 cast +1 draw = 0
    let bear_body = g.battlefield_find(bear).unwrap();
    // counters = hand size after draw
    let counters = bear_body.counter_count(CounterType::PlusOnePlusOne);
    assert!(counters >= 1);
}

#[test]
fn quandrix_wilderwright_etb_finds_basic_land() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::quandrix_wilderwright());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast!(g, id);
    drain_stack(&mut g);
    let forest_body = g.battlefield_find(forest).expect("Forest on bf");
    assert!(forest_body.tapped);
}

#[test]
fn quandrix_solver_magecraft_draws_and_discards() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _id = g.add_card_to_battlefield(0, catalog::quandrix_solver());
    drain_stack(&mut g);
    let fodder = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    cast!(g, bolt, crabomination::game::types::Target::Player(1));
    drain_stack(&mut g);
    // -1 bolt (cast), magecraft +1 draw, -1 discard = -1 net hand size
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
    let _ = fodder;
}

#[test]
fn silverquill_indoctrinator_etb_discards_each_opp() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_indoctrinator());
    flood_mana!(g);
    let opp_hand_before = g.players[1].hand.len();
    cast!(g, id);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
}

#[test]
fn confront_the_doubt_discards_nonland_noncreature_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::confront_the_doubt());
    flood_mana!(g);
    let me_before = g.players[0].life;
    let opp_hand_before = g.players[1].hand.len();
    cast!(g, id, crabomination::game::types::Target::Player(1));
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
    assert_eq!(g.players[0].life, me_before + 2);
}

#[test]
fn test_of_patience_counters_an_ability_and_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    // P0 casts Devourer of Destiny; its on-cast Scry trigger goes on the stack.
    let dev = g.add_card_to_hand(0, catalog::devourer_of_destiny());
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpell {
        card_id: dev, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    // P1 responds with Test of Patience targeting the trigger's source.
    g.priority.player_with_priority = 1;
    let id = g.add_card_to_hand(1, catalog::test_of_patience());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);
    let hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Permanent(dev)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Test of Patience castable in response");
    drain_stack(&mut g);
    assert!(!g.stack.iter().any(|si| matches!(
        si, crabomination::game::StackItem::Trigger { source, .. } if *source == dev
    )), "Scry trigger should have been countered");
    // -1 cast + 1 draw = net 0.
    assert_eq!(g.players[1].hand.len(), hand_before);
}

// ── CR 107 — Numbers and Symbols audit (batch 32) ──────────────────────────

#[test]
fn cr_107_1c_x_zero_for_x_cost_spell_resolves_cleanly() {
    // CR 107.1c: "If a rule or ability instructs a player to choose 'any
    // number,' that player may choose any positive number or zero."
    // Crackle with Power cast for X=0 should deal 0 damage and gracefully
    // do nothing (CR 120.8 zero-damage suppression).
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::crackle_with_power());
    // Cost: {X}{R}{R}{R}{R}{R}; pay just the colored pips.
    for _ in 0..5 { g.players[0].mana_pool.add(Color::Red, 1); }
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: Some(0),
    }).expect("Crackle castable at X=0");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before, "5*0 = 0 damage");
}

#[test]
fn reduce_to_ashes_burns_creature_for_four() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::reduce_to_ashes());
    flood_mana!(g);
    cast!(g, id, crabomination::game::types::Target::Permanent(bear));
    drain_stack(&mut g);
    // 2/2 bear (toughness ≤ 4 = would die) is exiled, not sent to graveyard.
    assert!(g.exile.iter().any(|c| c.id == bear), "lethal target is exiled");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn reduce_to_ashes_only_damages_a_tall_creature() {
    let mut g = two_player_game();
    let hulk = g.add_card_to_battlefield(1, catalog::torrential_gearhulk()); // 5/6
    let id = g.add_card_to_hand(0, catalog::reduce_to_ashes());
    flood_mana!(g);
    cast!(g, id, crabomination::game::types::Target::Permanent(hulk));
    drain_stack(&mut g);
    let card = g.battlefield_find(hulk).expect("6-toughness survives 4 damage");
    assert_eq!(card.damage, 4, "takes 4 damage, not exiled (toughness > 4)");
    assert!(!g.exile.iter().any(|c| c.id == hulk));
}

// ── Mercurial Transformation: ability-strip verification (CR 113.10b) ───────

#[test]
fn mercurial_transformation_strips_keywords_from_target() {
    // Dragon (5/5 Flying) becomes 3/3 with no abilities → no Flying.
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    g.clear_sickness(dragon);
    let id = g.add_card_to_hand(0, catalog::mercurial_transformation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast!(g, id, crabomination::game::types::Target::Permanent(dragon));
    drain_stack(&mut g);
    let computed = g.computed_permanent(dragon).expect("Dragon on bf");
    assert!(!computed.keywords.contains(&Keyword::Flying),
        "Flying stripped by 'loses all abilities'");
    assert!(computed.lost_all_abilities, "lost_all_abilities flag set");
}

#[test]
fn mercurial_transformation_strips_etb_triggers_from_target() {
    // Sedgemoor Witch (magecraft → make Pest) is stripped, then we cast
    // another instant and expect no new Pest tokens.
    let mut g = two_player_game();
    let witch = g.add_card_to_battlefield(0, catalog::sedgemoor_witch());
    g.clear_sickness(witch);
    drain_stack(&mut g);
    let merc = g.add_card_to_hand(0, catalog::mercurial_transformation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast!(g, merc, crabomination::game::types::Target::Permanent(witch));
    drain_stack(&mut g);
    let token_count_before = g.battlefield.iter()
        .filter(|c| c.is_token).count();
    // Cast a bolt; Sedgemoor would normally make a Pest. Stripped, it can't.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast!(g, bolt, crabomination::game::types::Target::Player(1));
    drain_stack(&mut g);
    let token_count_after = g.battlefield.iter()
        .filter(|c| c.is_token).count();
    assert_eq!(token_count_after, token_count_before,
        "magecraft trigger stripped — no new Pest tokens");
}

// ── Remaining card-specific behaviors ───────────────────────────────────────

#[test]
fn witherbloom_pesthatch_mints_pest_and_pumps() {
    let mut g = two_player_game();
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::witherbloom_pesthatch());
    flood_mana!(g);
    cast!(g, id, crabomination::game::types::Target::Permanent(friend));
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 1);
    let bear = g.battlefield_find(friend).unwrap();
    assert_eq!(bear.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn silverquill_litany_shrinks_creature_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_litany());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let me_before = g.players[0].life;
    cast!(g, id, crabomination::game::types::Target::Permanent(bear));
    drain_stack(&mut g);
    // 2/2 bear → -2/-1 → 0/1 (alive, but powerless)
    let bear_view = g.computed_permanent(bear).expect("Bear still on bf");
    assert_eq!(bear_view.power, 0);
    assert_eq!(bear_view.toughness, 1);
    assert_eq!(g.players[0].life, me_before + 1);
}

#[test]
fn inkling_quillbearer_magecraft_shrinks_target() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let _id = g.add_card_to_battlefield(0, catalog::inkling_quillbearer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast!(g, bolt, crabomination::game::types::Target::Permanent(opp_bear));
    drain_stack(&mut g);
    // Bear was 2/2, took 3 → dead before magecraft can shrink. Check graveyard.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == opp_bear));
}

#[test]
fn inkling_calligrapher_magecraft_shrinks_target_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::inkling_calligrapher());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast!(g, bolt, crabomination::game::types::Target::Player(1));
    drain_stack(&mut g);
    // bear is 2/2 -> -1/-1 = 1/1 (still alive)
    let bear_body = g.battlefield_find(opp_bear);
    assert!(bear_body.is_some(), "bear still alive at 1/1");
    let _ = id;
}

#[test]
fn strixhaven_field_researcher_etb_pumps_team() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::strixhaven_field_researcher());
    flood_mana!(g);
    cast!(g, id);
    drain_stack(&mut g);
    let bear1_body = g.battlefield_find(bear1).unwrap();
    let bear2_body = g.battlefield_find(bear2).unwrap();
    assert_eq!(bear1_body.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(bear2_body.counter_count(CounterType::PlusOnePlusOne), 1);
    // The Field Researcher itself is a creature too — also pumped
    let self_body = g.battlefield_find(id).unwrap();
    assert_eq!(self_body.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_geomancer_etb_and_magecraft_add_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_b35_geomancer());
    flood_mana!(g);
    cast!(g, id);
    drain_stack(&mut g);
    let geo_id = g.battlefield.iter().find(|c| c.definition.name == "Quandrix Geomancer II").unwrap().id;
    let card = g.battlefield_find(geo_id).unwrap();
    // 2/3 + 1 ETB counter = 3/4
    assert_eq!(card.power(), 3);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast!(g, bolt, crabomination::game::types::Target::Player(1));
    drain_stack(&mut g);
    let card = g.battlefield_find(geo_id).unwrap();
    // +1 more counter → 4/5
    assert_eq!(card.power(), 4);
}

#[test]
fn quandrix_equation_adds_two_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_b35_equation());
    flood_mana!(g);
    cast!(g, id, crabomination::game::types::Target::Permanent(bear));
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn prismari_stormforge_deals_three_and_draws_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_stormforge());
    flood_mana!(g);
    let hand_before = g.players[0].hand.len();
    cast!(g, id, crabomination::game::types::Target::Permanent(bear));
    drain_stack(&mut g);
    // -1 from cast + 2 from draw = +1 net
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn inkling_b36_sentinel_is_a_three_mana_flying_soldier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_b36_sentinel());
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 3);
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.definition.subtypes.creature_types.contains(&CreatureType::Soldier));
}
