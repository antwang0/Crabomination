use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

// ── Table-driven: cast an untargeted spell/creature and check its printed
//    payload (drain/gain/draw/mill/tokens/keywords/P-T). One row per card. ──

#[test]
fn table_cast_untargeted_primary_payload() {
    // Columns: (def, opp_loss, my_gain, draws, tok_name, tok_type,
    //           tok_count, opp_mill, keywords, pt)
    for (def, opp_loss, my_gain, draws, tok_name, tok_type, tok_count, mill, kws, pt) in [
        (catalog::witherbloom_drainherald(), 2i64, Some(2i64), false, None, None, 0usize, 0usize, &[Keyword::Lifelink][..], None),
        (catalog::silverquill_ledgerkeeper(), 2, Some(2), false, None, None, 0, 0, &[Keyword::Flying][..], None),
        (catalog::witherbloom_toxicpath_b103(), 2, Some(2), false, None, None, 0, 0, &[][..], None),
        (catalog::witherbloom_bloodgrafter_b122(), 2, Some(2), false, None, None, 0, 0, &[][..], None),
        (catalog::witherbloom_sapdrainer_b122(), 2, None, false, None, None, 0, 0, &[Keyword::Lifelink][..], Some((4i64, 3i64))),
        (catalog::witherbloom_saprooter_b120(), 2, Some(2), false, None, Some(CreatureType::Pest), 1, 0, &[][..], None),
        (catalog::inkling_battlescribe_b120(), 1, Some(1), false, None, None, 0, 0, &[Keyword::Flying, Keyword::Lifelink][..], None),
        (catalog::silverquill_scriptdrain(), 3, Some(3), false, None, None, 0, 0, &[][..], None),
        (catalog::silverquill_quillsweep_b119(), 3, Some(3), true, None, None, 0, 0, &[][..], None),
        (catalog::witherbloom_reapdrain(), 2, Some(2), true, None, None, 0, 0, &[][..], None),
        (catalog::silverquill_inkdiplomat(), 0, Some(1), true, None, None, 0, 0, &[][..], None),
        (catalog::silverquill_loresmith_b119(), 0, Some(2), false, None, None, 0, 0, &[Keyword::Lifelink, Keyword::Vigilance][..], None),
        (catalog::silverquill_reverence_b122(), 1, Some(1), true, None, None, 0, 0, &[][..], None),
        (catalog::pest_spawnmother(), 0, None, false, Some("Pest"), None, 3, 0, &[][..], None),
        (catalog::prismari_sparkbearer(), 0, None, false, Some("Treasure"), None, 1, 0, &[][..], None),
        (catalog::prismari_brewbinder(), 0, None, false, Some("Treasure"), None, 1, 0, &[][..], None),
        (catalog::inkling_aerospread(), 0, None, false, Some("Inkling"), None, 1, 0, &[][..], None),
        (catalog::lorehold_embertusk(), 1, None, false, None, Some(CreatureType::Spirit), 1, 0, &[][..], None),
        (catalog::witherbloom_pestbrood_b104(), 0, None, false, None, Some(CreatureType::Pest), 2, 0, &[Keyword::Deathtouch][..], None),
        (catalog::pest_brewmaster_b122(), 0, None, false, None, Some(CreatureType::Pest), 2, 0, &[][..], None),
        (catalog::witherbloom_cradlemage_b119(), 0, None, false, None, Some(CreatureType::Pest), 1, 2, &[][..], None),
        (catalog::pest_swarmcaller_b122(), 2, Some(2), false, None, Some(CreatureType::Pest), 2, 0, &[][..], None),
        (catalog::witherbloom_mireseer_b104(), 0, Some(1), false, None, None, 0, 2, &[][..], None),
        (catalog::witherbloom_cultmaster_b104(), 0, None, true, None, Some(CreatureType::Pest), 1, 3, &[][..], None),
        (catalog::quandrix_numeromancer(), 0, None, true, None, None, 0, 0, &[][..], None),
        (catalog::prismari_elementalist_b104(), 0, None, true, Some("Treasure"), None, 1, 0, &[][..], None),
        (catalog::inkling_scrollwarden_b68(), 0, None, false, None, None, 0, 0, &[Keyword::Flying, Keyword::Vigilance][..], Some((4, 4))),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        for _ in 0..4 {
            g.add_card_to_library(0, catalog::island());
            g.add_card_to_library(1, catalog::island());
        }
        for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(color, 3);
        }
        g.players[0].mana_pool.add_colorless(8);
        let id = g.add_card_to_hand(0, def);
        let l0 = g.players[0].life as i64;
        let l1 = g.players[1].life as i64;
        let hand_before = g.players[0].hand.len();
        let lib1_before = g.players[1].library.len();
        cast(&mut g, id);
        assert_eq!(g.players[1].life as i64, l1 - opp_loss, "{cname}: opp life");
        if let Some(gain) = my_gain {
            assert_eq!(g.players[0].life as i64, l0 + gain, "{cname}: my life");
        }
        if draws {
            // -1 (cast) +1 (draw) = net 0.
            assert_eq!(g.players[0].hand.len(), hand_before, "{cname}: cantrip");
        }
        assert_eq!(g.players[1].library.len(), lib1_before - mill, "{cname}: opp mill");
        if tok_name.is_some() || tok_type.is_some() {
            let count = g.battlefield.iter()
                .filter(|c| c.is_token && c.controller == 0 && match (&tok_name, &tok_type) {
                    (Some(n), _) => c.definition.name == *n,
                    (None, Some(ct)) => c.definition.subtypes.creature_types.contains(ct),
                    (None, None) => false,
                })
                .count();
            assert_eq!(count, tok_count, "{cname}: token count");
        }
        if !kws.is_empty() || pt.is_some() {
            let c = g.battlefield_find(id).expect("caster on battlefield");
            for kw in kws {
                assert!(c.has_keyword(kw), "{cname}: missing {kw:?}");
            }
            if let Some((p, t)) = pt {
                assert_eq!(c.power() as i64, p, "{cname}: power");
                assert_eq!(c.toughness() as i64, t, "{cname}: toughness");
            }
        }
    }
}

// ── Table-driven: magecraft "when you cast a spell, drain/ping the
//    opponent" cards. Cast a Lightning Bolt at P1 and check total loss. ──

#[test]
fn table_magecraft_ping_or_drain_opponent() {
    // Columns: (def, total opp loss incl. the 3 bolt damage, my gain, keywords)
    for (def, opp_loss, my_gain, kws) in [
        (catalog::inkling_glyphkeeper(), 4i64, 1i64, &[][..]),
        (catalog::witherbloom_toxinsage(), 4, 1, &[][..]),
        (catalog::silverquill_confessor(), 4, 1, &[][..]),
        (catalog::silverquill_inkblade_b104(), 4, 1, &[Keyword::Lifelink][..]),
        (catalog::prismari_stormcaller_b68(), 4, 0, &[][..]),
        (catalog::lorehold_pyrescholar_b103(), 4, 0, &[][..]),
        (catalog::prismari_sparkpoet(), 4, 0, &[][..]),
        (catalog::lorehold_pyromancer_b104(), 4, 0, &[][..]),
        (catalog::lorehold_spelldrake_b119(), 5, 0, &[Keyword::Flying][..]),
        (catalog::prismari_flamescholar_b119(), 4, 0, &[][..]),
        (catalog::silverquill_devotee_b120(), 5, 0, &[][..]),
        (catalog::prismari_pyrocaster_b120(), 4, 0, &[][..]),
        (catalog::lorehold_loreseeker_b120(), 4, 0, &[][..]),
        (catalog::lorehold_pyroscholar_b122(), 4, 0, &[][..]),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        let l0 = g.players[0].life as i64;
        let l1 = g.players[1].life as i64;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life as i64, l1 - opp_loss, "{cname}: opp life");
        assert_eq!(g.players[0].life as i64, l0 + my_gain, "{cname}: my life");
        let c = g.battlefield_find(id).expect("on bf");
        for kw in kws {
            assert!(c.has_keyword(kw), "{cname}: missing {kw:?}");
        }
    }
}

// ── Table-driven: magecraft self-pump (P/T until EOT) cards. ──

#[test]
fn table_magecraft_self_pump() {
    // Columns: (def, power delta, toughness delta (None = don't assert), keywords)
    for (def, dp, dt, kws) in [
        (catalog::witherbloom_vinescholar(), 1i64, Some(1i64), &[][..]),
        (catalog::lorehold_embertenured(), 1, None, &[Keyword::Vigilance][..]),
        (catalog::silverquill_brushmage(), 1, Some(1), &[][..]),
        (catalog::prismari_sparkcaller_b104(), 1, None, &[Keyword::Haste][..]),
        (catalog::lorehold_battlescribe_b119(), 1, None, &[Keyword::FirstStrike][..]),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        let p_before = g.battlefield_find(id).unwrap().power() as i64;
        let t_before = g.battlefield_find(id).unwrap().toughness() as i64;
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(id).expect("on bf");
        assert_eq!(c.power() as i64, p_before + dp, "{cname}: power");
        if let Some(dt) = dt {
            assert_eq!(c.toughness() as i64, t_before + dt, "{cname}: toughness");
        }
        for kw in kws {
            assert!(c.has_keyword(kw), "{cname}: missing {kw:?}");
        }
    }
}

// ── Table-driven: magecraft "+1/+1 counter on self" cards. ──

#[test]
fn table_magecraft_self_counter() {
    for def in [
        catalog::quandrix_symmetrybard(),
        catalog::quandrix_polymath_b119(),
        catalog::quandrix_apprentice_b120(),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        let before = g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
            before + 1, "{cname}: counter added");
    }
}

// ── Table-driven: cards that mint one Fractal token carrying N +1/+1
//    counters on resolution. ──

#[test]
fn table_mints_fractal_with_counters() {
    for (def, counters) in [
        (catalog::quandrix_aetherist_b103(), 2u32),
        (catalog::quandrix_mathematician_b104(), 2),
        (catalog::fractal_spawnmaster_b119(), 3),
        (catalog::fractal_bloomwright_b120(), 4),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(color, 3);
        }
        g.players[0].mana_pool.add_colorless(8);
        let id = g.add_card_to_hand(0, def);
        cast(&mut g, id);
        let fractals: Vec<_> = g.battlefield.iter()
            .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
            .collect();
        assert_eq!(fractals.len(), 1, "{cname}: one Fractal minted");
        assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne) as u32, counters,
            "{cname}: Fractal counters");
    }
}

// ── Table-driven: targeted removal that kills/exiles the victim outright,
//    with optional token/gain/cantrip riders. ──

#[test]
fn table_targeted_removal_kills_victim() {
    // Columns: (def, victim, my gain, draws, tok_name, tok_type, tok_count)
    for (def, victim, my_gain, draws, tok_name, tok_type, tok_count) in [
        (catalog::witherbloom_toxinbinder(), catalog::grizzly_bears(), 0i64, false, None, None, 0usize),
        (catalog::lorehold_sparkshrine(), catalog::grizzly_bears(), 0, false, Some("Spirit"), None, 1),
        (catalog::lorehold_pyrebinder(), catalog::grizzly_bears(), 0, false, None, None, 0),
        (catalog::lorehold_lecturer(), catalog::grizzly_bears(), 0, false, None, Some(CreatureType::Spirit), 1),
        (catalog::prismari_crackleburst_b104(), catalog::savannah_lions(), 0, false, Some("Treasure"), None, 1),
        (catalog::silverquill_censurer_b120(), catalog::savannah_lions(), 2, false, None, None, 0),
        (catalog::silverquill_verdict_b120(), catalog::savannah_lions(), 0, false, None, Some(CreatureType::Inkling), 1),
        (catalog::silverquill_verdict_b122(), catalog::serra_angel(), 4, false, None, None, 0),
        (catalog::prismari_inferno_b122(), catalog::serra_angel(), 0, true, None, None, 0),
        (catalog::prismari_tempest_b120(), catalog::serra_angel(), 0, true, None, None, 0),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(color, 3);
        }
        g.players[0].mana_pool.add_colorless(8);
        let target = g.add_card_to_battlefield(1, victim);
        g.clear_sickness(target);
        let l0 = g.players[0].life as i64;
        let id = g.add_card_to_hand(0, def);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(target).is_none(), "{cname}: victim removed");
        assert_eq!(g.players[0].life as i64, l0 + my_gain, "{cname}: my life");
        if draws {
            // -1 (cast) +1 (draw) = net -1 relative to pre-cast hand count.
            assert_eq!(g.players[0].hand.len(), hand_before, "{cname}: cantrip");
        }
        if tok_name.is_some() || tok_type.is_some() {
            let count = g.battlefield.iter()
                .filter(|c| c.is_token && c.controller == 0 && match (&tok_name, &tok_type) {
                    (Some(n), _) => c.definition.name == *n,
                    (None, Some(ct)) => c.definition.subtypes.creature_types.contains(ct),
                    (None, None) => false,
                })
                .count();
            assert_eq!(count, tok_count, "{cname}: token count");
        }
    }
}

// ── Table-driven: targeted damage that marks damage on a surviving Serra
//    Angel (4/4), with optional Spirit rider. ──

#[test]
fn table_targeted_damage_marks_angel() {
    for (def, dmg, spirits) in [
        (catalog::prismari_magmaweaver_b119(), 2u32, 0usize),
        (catalog::lorehold_bondbreaker_b120(), 3, 1),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(color, 3);
        }
        g.players[0].mana_pool.add_colorless(8);
        let target = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.clear_sickness(target);
        let id = g.add_card_to_hand(0, def);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(target).expect("angel alive");
        assert_eq!(c.damage, dmg, "{cname}: damage marked");
        let count = g.battlefield.iter()
            .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
            .count();
        assert_eq!(count, spirits, "{cname}: spirit count");
    }
}

// ── Table-driven: targeted -X/-X on a surviving Serra Angel (4/4). ──

#[test]
fn table_targeted_shrink_angel_survives() {
    for (def, dp, dt, kws) in [
        (catalog::witherbloom_spinecaster_b122(), -1i64, -1i64, &[][..]),
        (catalog::inkling_quillstrike_b122(), -2, -2, &[Keyword::Flying][..]),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(color, 3);
        }
        g.players[0].mana_pool.add_colorless(8);
        let target = g.add_card_to_battlefield(1, catalog::serra_angel());
        g.clear_sickness(target);
        let id = g.add_card_to_hand(0, def);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let angel = g.battlefield_find(target).expect("angel alive");
        assert_eq!(angel.power() as i64, 4 + dp, "{cname}: power");
        assert_eq!(angel.toughness() as i64, 4 + dt, "{cname}: toughness");
        if !kws.is_empty() {
            let c = g.battlefield_find(id).expect("caster on bf");
            for kw in kws {
                assert!(c.has_keyword(kw), "{cname}: missing {kw:?}");
            }
        }
    }
}

// ── Table-driven: spells/ETBs targeting a player for N damage, with
//    optional keyword/token/cantrip riders. ──

#[test]
fn table_targets_player_for_damage() {
    // Columns: (def, dmg, keywords on the (surviving) caster, tok_name,
    //           tok_type, tok_count, draws)
    for (def, dmg, kws, tok_name, tok_type, tok_count, draws) in [
        (catalog::lorehold_fireseer_b104(), 1i64, &[][..], None, None, 0usize, false),
        (catalog::lorehold_sparkstrike_b104(), 3, &[][..], None, Some(CreatureType::Spirit), 1, false),
        (catalog::lorehold_skirmisher_b119(), 1, &[Keyword::Haste][..], None, None, 0, false),
        (catalog::lorehold_flameherald_b120(), 1, &[Keyword::Haste][..], None, None, 0, false),
        (catalog::prismari_stormburst_b104(), 3, &[][..], None, None, 0, true),
        (catalog::prismari_ember_surge(), 3, &[][..], None, None, 0, true),
        (catalog::prismari_crucible_b120(), 2, &[][..], Some("Treasure"), None, 1, false),
        (catalog::prismari_lecturer(), 2, &[][..], None, None, 0, false),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(color, 3);
        }
        g.players[0].mana_pool.add_colorless(8);
        let id = g.add_card_to_hand(0, def);
        let l1 = g.players[1].life as i64;
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life as i64, l1 - dmg, "{cname}: opp life");
        if draws {
            assert_eq!(g.players[0].hand.len(), hand_before, "{cname}: cantrip");
        }
        if !kws.is_empty() {
            let c = g.battlefield_find(id).expect("caster on bf");
            for kw in kws {
                assert!(c.has_keyword(kw), "{cname}: missing {kw:?}");
            }
        }
        if tok_name.is_some() || tok_type.is_some() {
            let count = g.battlefield.iter()
                .filter(|c| c.is_token && c.controller == 0 && match (&tok_name, &tok_type) {
                    (Some(n), _) => c.definition.name == *n,
                    (None, Some(ct)) => c.definition.subtypes.creature_types.contains(ct),
                    (None, None) => false,
                })
                .count();
            assert_eq!(count, tok_count, "{cname}: token count");
        }
    }
}

// ── Table-driven: "sacrifice another creature: payoff" activated abilities
//    (sac_other_filter family). All rows sacrifice a Savannah Lions fodder
//    and check the payoff; the activator itself must survive. ──

#[test]
fn table_sac_other_activated_payoffs() {
    // Columns: (def, my life delta, opp life delta, draws, self power delta, keywords)
    for (def, dl0, dl1, draws, self_dp, kws) in [
        (catalog::pest_cultmaster_b121(), 0i64, 0i64, true, 0i64, &[][..]),
        (catalog::witherbloom_sapdrinker_b121(), 0, 0, false, 2, &[][..]),
        (catalog::pest_ringleader_b121(), 2, -2, false, 0, &[][..]),
        (catalog::pest_cultcaller_b122(), 1, -1, false, 0, &[][..]),
        (catalog::witherbloom_composter_b122(), -1, 0, true, 0, &[][..]),
        (catalog::witherbloom_cultivator_b120(), 1, -1, false, 0, &[][..]),
        (catalog::witherbloom_harvester_b119(), 0, 0, true, 0, &[][..]),
        (catalog::witherbloom_reaper_b121(), 0, 0, false, 0, &[Keyword::Indestructible][..]),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(color, 3);
        }
        g.players[0].mana_pool.add_colorless(8);
        let src = g.add_card_to_battlefield(0, def);
        g.clear_sickness(src);
        let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
        g.clear_sickness(fodder);
        let l0 = g.players[0].life as i64;
        let l1 = g.players[1].life as i64;
        let hand_before = g.players[0].hand.len();
        let p_before = g.battlefield_find(src).unwrap().power() as i64;
        g.perform_action(GameAction::ActivateAbility {
            card_id: src, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        }).expect("activation");
        drain_stack(&mut g);
        assert!(g.battlefield_find(fodder).is_none(), "{cname}: fodder sacrificed");
        let c = g.battlefield_find(src).expect("activator survives");
        assert_eq!(g.players[0].life as i64, l0 + dl0, "{cname}: my life");
        assert_eq!(g.players[1].life as i64, l1 + dl1, "{cname}: opp life");
        if draws {
            assert_eq!(g.players[0].hand.len(), hand_before + 1, "{cname}: drew a card");
        }
        assert_eq!(c.power() as i64, p_before + self_dp, "{cname}: self power");
        for kw in kws {
            assert!(c.has_keyword(kw), "{cname}: missing {kw:?}");
        }
    }
}

// ── Table-driven: targeted buffs on a friendly Savannah Lions (pump,
//    counters, keyword grants, optional cantrip). ──

#[test]
fn table_targeted_friendly_buff() {
    for (def, dp, dt, kws, draws) in [
        (catalog::quandrix_calculus_b119(), 1i64, Some(1i64), &[][..], true),
        (catalog::quandrix_equation_b120(), 1, Some(1), &[][..], true),
        (catalog::fractal_multiplier_b122(), 1, None, &[][..], false),
        (catalog::spirit_glyphbinder(), 1, Some(1), &[][..], false),
        (catalog::silverquill_anointment_b104(), 1, Some(1), &[Keyword::Indestructible][..], false),
        (catalog::silverquill_embolden_b119(), 2, Some(2), &[Keyword::Lifelink][..], false),
        (catalog::silverquill_bookmark(), 0, Some(2), &[Keyword::Lifelink][..], false),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        for color in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(color, 3);
        }
        g.players[0].mana_pool.add_colorless(8);
        let target = g.add_card_to_battlefield(0, catalog::savannah_lions());
        g.clear_sickness(target);
        let p_before = g.battlefield_find(target).unwrap().power() as i64;
        let t_before = g.battlefield_find(target).unwrap().toughness() as i64;
        let id = g.add_card_to_hand(0, def);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(target).expect("target alive");
        assert_eq!(c.power() as i64, p_before + dp, "{cname}: power");
        if let Some(dt) = dt {
            assert_eq!(c.toughness() as i64, t_before + dt, "{cname}: toughness");
        }
        for kw in kws {
            assert!(c.has_keyword(kw), "{cname}: missing {kw:?}");
        }
        if draws {
            assert_eq!(g.players[0].hand.len(), hand_before, "{cname}: cantrip");
        }
    }
}

// ── Table-driven: vanilla-ish stat/keyword/subtype checks. ──

#[test]
fn table_battlefield_stats_and_keywords() {
    for (def, p, t, kws, ctype) in [
        (catalog::pest_nightswarm(), 2i64, 2i64, &[Keyword::Flying][..], Some(CreatureType::Pest)),
        (catalog::lorehold_heroic_sage(), 2, 2, &[Keyword::FirstStrike, Keyword::Lifelink][..], None),
        (catalog::inkling_glyphwarden_b122(), 2, 4, &[Keyword::Flying, Keyword::Lifelink][..], Some(CreatureType::Inkling)),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let c = g.battlefield_find(id).expect("on bf");
        assert_eq!(c.power() as i64, p, "{cname}: power");
        assert_eq!(c.toughness() as i64, t, "{cname}: toughness");
        for kw in kws {
            assert!(c.has_keyword(kw), "{cname}: missing {kw:?}");
        }
        if let Some(ct) = ctype {
            assert!(c.definition.subtypes.creature_types.contains(&ct), "{cname}: subtype");
        }
    }
}

// ── Table-driven: magecraft looters (draw a card, then discard a card). ──

#[test]
fn table_magecraft_loots() {
    for def in [
        catalog::quandrix_mistshaper_b68(),
        catalog::prismari_tidemage(),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_hand(0, catalog::island()); // a card to discard
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // Cast bolt: -1, magecraft draw 1: +1, discard 1: -1 → net -1.
        assert_eq!(g.players[0].hand.len(), hand_before - 1, "{cname}: loot net");
    }
}

// ── Table-driven: "whenever this attacks, drain N" cards. ──

#[test]
fn table_attack_drains_opponent() {
    use crabomination::game::types::AttackTarget;
    for (def, n) in [
        (catalog::inkling_glaivemaster(), 1i64),
        (catalog::inkling_loremaster_b104(), 2),
    ] {
        let cname = def.name;
        let mut g = two_player_game();
        let attacker = g.add_card_to_battlefield(0, def);
        g.clear_sickness(attacker);
        g.step = crabomination::game::types::TurnStep::DeclareAttackers;
        let l0 = g.players[0].life as i64;
        let l1 = g.players[1].life as i64;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker, target: AttackTarget::Player(1),
        }])).expect("attacker declared");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life as i64, l0 + n, "{cname}: you gain");
        assert_eq!(g.players[1].life as i64, l1 - n, "{cname}: opp loses");
    }
}

// ── Unique / combined / regression tests kept as-is below. ──────────────────

#[test]
fn quandrix_streamwarden_magecraft_pumps_target_fractal() {
    let mut g = two_player_game();
    let fractal_id = g.add_card_to_battlefield(0, catalog::fractal_pondling());
    let _ = g.add_card_to_battlefield(0, catalog::quandrix_streamwarden());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.battlefield_find(fractal_id).expect("Fractal on bf");
    // Fractal grew from 1/1 to 2/2
    assert_eq!(view.power(), 2);
    assert_eq!(view.toughness(), 2);
}

#[test]
fn quandrix_sumstride_mints_fractal_scaling_with_creatures() {
    let mut g = two_player_game();
    // 3 creatures on the battlefield
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _ = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_sumstride());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sumstride castable");
    drain_stack(&mut g);
    // 3 creatures + the spell-resolution creates fractal token (+0). Fractal
    // should have 4 +1/+1 counters since after token creation there are 4
    // creatures (3 bears + 1 fractal). Verify the fractal exists & has some
    // counters (4/4 expected).
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
    assert!(fractal.is_some(), "Fractal token should exist");
    let view = g.battlefield_find(fractal.unwrap().id).expect("Fractal on bf");
    assert_eq!(view.power(), 4);
    assert_eq!(view.toughness(), 4);
}

#[test]
fn velomachus_attack_exiles_is_card_from_top_of_library_and_grants_may_play() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    // Seed library: 2 Forests (MV 0) + 1 Lightning Bolt (MV 1) on top.
    // RevealUntilFind walks until it hits the Bolt; misses go to
    // bottom of library randomized.
    use crabomination::card::CardInstance;
    let mut bolt = CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    let bolt_id = bolt.id;
    let mut top: Vec<CardInstance> = vec![
        CardInstance::new(g.next_id(), catalog::forest(), 0),
        CardInstance::new(g.next_id(), catalog::forest(), 0),
        bolt,
    ];
    for c in top.iter_mut() { c.controller = 0; }
    for c in top.into_iter().rev() {
        g.players[0].library.insert(0, c);
    }
    let velo = g.add_card_to_battlefield(0, catalog::velomachus_lorehold());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == velo) {
        c.summoning_sick = false; c.tapped = false;
    }

    // Move to combat + declare attack.
    g.step = crabomination::game::TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: velo,
        target: AttackTarget::Player(1),
    }])).expect("Velomachus can attack");
    drain_stack(&mut g);

    // Bolt should now be in exile with may_play permission to P0.
    let exiled = g.exile.iter().find(|c| c.id == bolt_id)
        .expect("Bolt exiled by Velomachus's attack trigger");
    assert!(
        exiled.may_play_until.is_some(),
        "Bolt has may_play permission stamped",
    );
}

#[test]
fn mavinda_activation_exiles_gy_is_card_and_grants_may_play() {
    // Mavinda's printed {0} activation: target IS card in your gy moves
    // to exile with may_play_until + exile_after + pay-own-cost stamped,
    // plus the {8}-unless-targets-your-creature surcharge rider.
    // Once-per-turn gate enforced.
    let mut g = two_player_game();
    let mavinda = g.add_card_to_battlefield(0, catalog::mavinda_students_advocate());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == mavinda) {
        c.summoning_sick = false; c.tapped = false;
    }
    // Printed cost is {0} — no mana floated.
    // Seed a Lightning Bolt in P0's graveyard.
    let mut bolt = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    let bolt_id = bolt.id;
    g.players[0].graveyard.push(bolt);

    g.perform_action(GameAction::ActivateAbility {
        card_id: mavinda, ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(bolt_id)), additional_targets: Vec::new(), x_value: None }).expect("Mavinda activation (printed {0})");
    drain_stack(&mut g);

    let exiled = g.exile.iter().find(|c| c.id == bolt_id)
        .expect("Bolt moved to exile by Mavinda");
    let perm = exiled.may_play_until.expect("may_play stamped");
    assert!(perm.exile_after, "Mavinda's permission has exile_after=true");
    assert_eq!(perm.player, 0, "permission goes to Mavinda's controller");
    // Pay-own-cost: the may-play cast isn't free — Bolt's own {R} is stamped.
    assert_eq!(exiled.granted_alt_cast_cost_eot.as_ref().map(|c| c.cmc()), Some(1),
        "cast-this-way pays the spell's own cost");
    assert!(exiled.granted_cast_surcharge_eot.is_some(),
        "the {{8}}-unless-targets-your-creature surcharge is stamped");

    // Second activation in the same turn → rejected (once-per-turn).
    let mut bolt2 = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    bolt2.controller = 0;
    let bolt2_id = bolt2.id;
    g.players[0].graveyard.push(bolt2);
    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: mavinda, ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(bolt2_id)), additional_targets: Vec::new(), x_value: None });
    assert!(result.is_err(),
        "Second Mavinda activation in same turn should be rejected (once-per-turn)");
}

#[test]
fn witherbloom_necromage_etb_creates_pest_and_dies_drains() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_necromage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1, "One Pest minted on ETB");

    // Kill the necromage to trigger the death-drain.
    let life1_before = g.players[1].life;
    let life0_before = g.players[0].life;
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2, "opp loses 2 on death");
    assert_eq!(g.players[0].life, life0_before + 2, "you gain 2 on death");
}

#[test]
fn lorehold_battlemage_b103_etb_pings_and_magecraft_creates_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_battlemage_b103());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Cast Battlemage; ETB-ping auto-targets the opponent's face for 2.
    let life1_before = g.players[1].life;
    cast(&mut g, id);
    assert_eq!(g.players[1].life, life1_before - 2, "ETB pings for 2");

    // Cast a Bolt to trigger magecraft and verify Spirit token.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1, "Magecraft minted one Spirit token");
}

#[test]
fn quandrix_cycloid_etb_pumps_each_your_creature() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_cycloid());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let b1c = g.battlefield_find(b1).unwrap();
    let b2c = g.battlefield_find(b2).unwrap();
    let cycloid = g.battlefield_find(id).unwrap();
    assert_eq!(b1c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(b2c.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(cycloid.counter_count(CounterType::PlusOnePlusOne), 1,
        "cycloid pumps itself too");
}

#[test]
fn quandrix_lecturer_creates_fractal_with_creature_counters() {
    let mut g = two_player_game();
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_lecturer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 1, "Fractal minted");
    // Counter count runs after the Fractal joins the battlefield, so
    // the count covers 2 bears + the new Fractal token = 3.
    assert_eq!(fractals[0].counter_count(CounterType::PlusOnePlusOne), 3);
}

#[test]
fn inkling_sigilbearer_b103_pumps_each_inkling() {
    let mut g = two_player_game();
    // Use existing Inkling minter (Eager Glyphmage ETB → 1/1 inkling token).
    let glyph = g.add_card_to_hand(0, catalog::eager_glyphmage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, glyph);
    // One Inkling minted. Now play Sigilbearer which puts +1/+1 on each
    // Inkling (including itself).
    let id = g.add_card_to_hand(0, catalog::inkling_sigilbearer_b103());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    // The newly-minted Inkling token + the Sigilbearer (which is also
    // Inkling) both have a counter.
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .collect();
    let with_counters = inklings.iter()
        .filter(|c| c.counter_count(CounterType::PlusOnePlusOne) == 1)
        .count();
    assert!(with_counters >= 2,
        "At least 2 Inklings got +1/+1 (token + sigilbearer)");
}

#[test]
fn pest_bannerlord_pumps_each_pest_on_etb() {
    let mut g = two_player_game();
    // Cast Witherbloom Pestcaller (batch 103) to mint a Pest token.
    let caller = g.add_card_to_hand(0, catalog::witherbloom_pestcaller_b103());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, caller);
    let pests_before: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .map(|c| c.id)
        .collect();
    assert!(!pests_before.is_empty(), "Pestcaller minted a Pest");
    // Now play Pest Bannerlord which puts +1/+1 on each Pest.
    let id = g.add_card_to_hand(0, catalog::pest_bannerlord());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    for p_id in &pests_before {
        let p = g.battlefield_find(*p_id).expect("pest alive");
        assert_eq!(p.counter_count(CounterType::PlusOnePlusOne), 1,
            "Each pre-existing Pest got +1/+1 counter");
    }
}

#[test]
fn spirit_of_counterpoint_pumps_each_spirit_on_etb() {
    let mut g = two_player_game();
    // Play Lorehold Apprentice (a Spirit-tribal aside) — actually we
    // need a Spirit. Use Spirit of Counterpoint and assert it pumps
    // itself if no other Spirits exist.
    let id = g.add_card_to_hand(0, catalog::spirit_of_counterpoint());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let sp = g.battlefield_find(id).expect("Spirit on bf");
    // The Spirit of Counterpoint is itself a Spirit, so it should
    // pump itself.
    assert_eq!(sp.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn silverquill_maelstrom_drains_four_and_makes_opp_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_maelstrom());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let opp_hand_before = g.players[1].hand.len();
    cast(&mut g, id);
    assert_eq!(g.players[1].life, life1_before - 4);
    assert_eq!(g.players[0].life, life0_before + 4);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
}

#[test]
fn quandrix_calculator_b103_draws_and_pumps_creatures_on_etb() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_calculator_b103());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    cast(&mut g, id);
    // Cast (-1) + Draw (+1) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    let b = g.battlefield_find(b1).expect("bear alive");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn fractal_conductor_pumps_each_fractal_on_etb() {
    let mut g = two_player_game();
    // First play Quandrix Summoner to mint a Fractal.
    let summoner = g.add_card_to_hand(0, catalog::quandrix_summoner());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, summoner);
    let fractals_before: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .map(|c| c.id)
        .collect();
    assert!(!fractals_before.is_empty(), "Quandrix Summoner minted a Fractal");
    // Now play Fractal Conductor to pump each Fractal.
    let id = g.add_card_to_hand(0, catalog::fractal_conductor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let counters_before: Vec<_> = fractals_before.iter()
        .map(|fid| g.battlefield_find(*fid).unwrap().counter_count(CounterType::PlusOnePlusOne))
        .collect();
    cast(&mut g, id);
    for (fid, c_before) in fractals_before.iter().zip(counters_before.iter()) {
        let f = g.battlefield_find(*fid).expect("Fractal alive");
        assert_eq!(f.counter_count(CounterType::PlusOnePlusOne), c_before + 1,
            "Fractal got +1 +1/+1 counter from Conductor's ETB");
    }
}

#[test]
fn silverquill_anthemcaster_b104_mints_two_inklings_and_pumps_team() {
    let mut g = two_player_game();
    // Pre-existing creature to verify the anthem applies to it.
    let bear = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(bear);
    let p_before = g.battlefield_find(bear).unwrap().power();
    let id = g.add_card_to_hand(0, catalog::silverquill_anthemcaster_b104());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Inkling")
        .collect();
    assert_eq!(inklings.len(), 2);
    // The pre-existing bear got +1/+1 EOT.
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.power(), p_before + 1);
}

#[test]
fn pest_bloodscribe_b104_pumps_self_on_sacrifice() {
    let mut g = two_player_game();
    let scribe = g.add_card_to_battlefield(0, catalog::pest_bloodscribe_b104());
    g.clear_sickness(scribe);
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == fodder) {
        c.is_token = true;
    }
    let p_before = g.battlefield_find(scribe).unwrap().power();
    // Trigger sacrifice via Witherbloom Sacrosanct (printed sac-and-drain).
    let sac = g.add_card_to_hand(0, catalog::witherbloom_sacrosanct());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: sac, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sacrosanct castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(scribe).expect("scribe alive");
    assert_eq!(c.power(), p_before + 1, "+1 power from sacrifice trigger");
}

#[test]
fn spirit_of_the_archive_b104_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::savannah_lions());
    let id = g.add_card_to_hand(0, catalog::spirit_of_the_archive_b104());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "graveyard creature returned to hand");
    let s = g.battlefield_find(id).expect("spirit on bf");
    assert!(s.has_keyword(&Keyword::Flying));
    assert!(s.has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_battlecaster_b104_etb_mints_spirit_and_magecraft_pumps_self() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_battlecaster_b104());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1, "Spirit minted on ETB");

    // Magecraft self-pump from an instant cast.
    g.clear_sickness(id);
    let p_before = g.battlefield_find(id).unwrap().power();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().power(), p_before + 1);
}

#[test]
fn prismari_pyromage_b104_burns_creature_and_scrys_on_is_cast() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::prismari_pyromage_b104());
    g.clear_sickness(mage);
    // Big-toughness target so the magecraft 1 dmg doesn't drop it.
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(big);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Target the player with the Bolt; the magecraft trigger auto-targets
    // the only creature on the battlefield for its 1-damage payload.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(big).expect("angel alive");
    assert_eq!(b.damage, 1, "magecraft pinged the angel for 1");
}

#[test]
fn quandrix_theorist_b104_pumps_each_friendly_fractal_on_is_cast() {
    let mut g = two_player_game();
    let theorist = g.add_card_to_battlefield(0, catalog::quandrix_theorist_b104());
    g.clear_sickness(theorist);
    // Mint a Fractal via Body of Research-style helper card; use Quandrix Mathematician.
    let math = g.add_card_to_hand(0, catalog::quandrix_mathematician_b104());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, math);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .expect("Fractal exists")
        .id;
    let counters_before = g.battlefield_find(fractal).unwrap().counter_count(CounterType::PlusOnePlusOne);
    // Now cast a bolt — Quandrix Theorist's magecraft should pump the Fractal.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(fractal).unwrap().counter_count(CounterType::PlusOnePlusOne),
        counters_before + 1);
}

#[test]
fn fractal_bloom_b104_creates_two_fractals_with_three_total_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_bloom_b104());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .collect();
    assert_eq!(fractals.len(), 2, "Two Fractals minted");
    let total_counters: u32 = fractals.iter()
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(total_counters, 3, "Three +1/+1 counters distributed");
}

#[test]
fn quandrix_symmetrist_b104_doubles_counters_on_target() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(target);
    // Manually stamp 2 +1/+1 counters on the bear.
    g.battlefield_find_mut(target).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let id = g.add_card_to_hand(0, catalog::quandrix_symmetrist_b104());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Started with 2 counters, ETB adds 2 more (= "doubles" to 4 total).
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

// ── modern_decks batch 105 (helper shortcuts) tests ───────────────────────

#[test]
fn shortcut_mint_inklings_creates_w_b_flying_tokens() {
    use crabomination::effect::shortcut::mint_inklings;
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&mint_inklings(2), &ctx).expect("mint_inklings resolves");
    drain_stack(&mut g);
    let inklings: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .collect();
    assert_eq!(inklings.len(), 2);
    assert!(inklings.iter().all(|c| c.has_keyword(&Keyword::Flying)),
        "Inkling tokens fly");
}

#[test]
fn shortcut_mint_lorehold_spirits_creates_r_w_spirits() {
    use crabomination::effect::shortcut::mint_lorehold_spirits;
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&mint_lorehold_spirits(1), &ctx).expect("mint_lorehold_spirits resolves");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1);
    assert_eq!(spirits[0].power(), 2, "Lorehold Spirit is 2/2");
    assert_eq!(spirits[0].toughness(), 2);
}

// ── modern_decks batch 107 (etb_drain_each_opp shortcut) test ───────────────

#[test]
fn shortcut_etb_drain_each_opp_drains_only_opponents() {
    // Asymmetric drain helper: opponents lose N life, you do NOT gain
    // any. Locks in the asymmetric body so a future refactor can't
    // accidentally swap it back to the symmetric `etb_drain` shape.
    use crabomination::effect::shortcut::etb_drain_each_opp;
    use crabomination::effect::{PlayerRef, Selector, Value};
    let trig = etb_drain_each_opp(3);
    // The body must be a LoseLife on EachOpponent of amount 3 — NOT a
    // Drain (which would also include the you-gain half).
    match trig.effect {
        Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) } => {}
        ref other => panic!("expected LoseLife on EachOpponent of 3, got {other:?}"),
    }
}

#[test]
fn inkling_vanguard_b119_anthems_other_inklings_but_not_self() {
    let mut g = two_player_game();
    // Mint an Inkling token under our control.
    use crabomination::effect::shortcut::mint_inklings;
    use crabomination::game::effects::EffectContext;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&mint_inklings(1), &ctx).expect("mint inklings");
    drain_stack(&mut g);
    let inkling_token = g.battlefield.iter()
        .find(|c| c.is_token && c.controller == 0
            && c.definition.subtypes.creature_types.contains(&CreatureType::Inkling))
        .map(|c| c.id)
        .expect("inkling token exists");
    // Now drop the Vanguard.
    let vanguard = g.add_card_to_battlefield(0, catalog::inkling_vanguard_b119());
    // The Inkling token (1/1 base) gets +1/+0 → 2/1 via the static anthem.
    // Static modifications land via the layer system — read through
    // `compute_battlefield` rather than `battlefield_find`.
    let computed_token = g.compute_battlefield().into_iter()
        .find(|c| c.id == inkling_token)
        .expect("token in computed");
    assert_eq!(computed_token.power, 2);
    assert_eq!(computed_token.toughness, 1);
    // Vanguard itself stays 3/4 (anthem excludes the source).
    let computed_vanguard = g.compute_battlefield().into_iter()
        .find(|c| c.id == vanguard)
        .expect("vanguard in computed");
    assert_eq!(computed_vanguard.power, 3);
    assert_eq!(computed_vanguard.toughness, 4);
}

#[test]
fn pest_hivewatcher_b119_gains_life_when_another_creature_dies() {
    let mut g = two_player_game();
    let _watcher = g.add_card_to_battlefield(0, catalog::pest_hivewatcher_b119());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(fodder);
    let life_before = g.players[0].life;
    // Kill the fodder via Lightning Bolt — routes through SBA, which
    // emits the CreatureDied event that the AnotherOfYours scope picks up.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder is dead");
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn pest_hivewatcher_b119_does_not_gain_life_when_only_self_dies() {
    let mut g = two_player_game();
    let watcher = g.add_card_to_battlefield(0, catalog::pest_hivewatcher_b119());
    g.clear_sickness(watcher);
    let life_before = g.players[0].life;
    // Bolt the watcher itself — its own death should NOT fire its trigger
    // (AnotherOfYours scope excludes the source).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(watcher)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(watcher).is_none());
    assert_eq!(g.players[0].life, life_before);
}

#[test]
fn witherbloom_mulchcaster_b119_mills_target_and_gains_two_life() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::witherbloom_mulchcaster_b119());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib1_before = g.players[1].library.len();
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mulchcaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib1_before - 4);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_reliquary_b119_reanimates_creature_and_mints_spirit() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::savannah_lions());
    let id = g.add_card_to_hand(0, catalog::lorehold_reliquary_b119());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    // Bear is back on the battlefield.
    assert!(g.battlefield_find(bear).is_some(), "bear reanimated");
    // Spirit minted too.
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit))
        .collect();
    assert_eq!(spirits.len(), 1);
}

#[test]
fn spirit_battlecry_b119_pumps_all_friendly_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    let pa = g.battlefield_find(a).unwrap().power();
    let pb = g.battlefield_find(b).unwrap().power();
    let id = g.add_card_to_hand(0, catalog::spirit_battlecry_b119());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert_eq!(g.battlefield_find(a).unwrap().power(), pa + 1);
    assert_eq!(g.battlefield_find(b).unwrap().power(), pb + 1);
}

#[test]
fn prismari_inferno_b119_burns_creature_and_player() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::savannah_lions());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::prismari_inferno_b119());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    }).expect("Inferno castable with two targets");
    drain_stack(&mut g);
    // Bear (2 toughness) dies from 2 damage.
    assert!(g.battlefield_find(bear).is_none());
    // Player 1 loses 2.
    assert_eq!(g.players[1].life, life1_before - 2);
}

#[test]
fn prismari_reshape_b119_bounces_and_scrys() {
    let mut g = two_player_game();
    let opp_creature = g.add_card_to_battlefield(1, catalog::savannah_lions());
    g.clear_sickness(opp_creature);
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_reshape_b119());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_creature)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // opp_creature returned to opp's hand.
    assert!(g.battlefield_find(opp_creature).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == opp_creature));
}

#[test]
fn quandrix_druid_b119_etb_pumps_each_friendly_fractal() {
    let mut g = two_player_game();
    // Use Quandrix Mathematician (batch 104) to mint a Fractal with 2 +1/+1
    // counters — 0/0 base + 2 counters survives state-based actions.
    let math = g.add_card_to_hand(0, catalog::quandrix_mathematician_b104());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, math);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal))
        .map(|c| c.id)
        .collect();
    assert_eq!(fractals.len(), 1, "one fractal minted with +1/+1 counters");
    let counters_before = g.battlefield_find(fractals[0]).unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    let id = g.add_card_to_hand(0, catalog::quandrix_druid_b119());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(g.battlefield_find(fractals[0]).unwrap()
        .counter_count(CounterType::PlusOnePlusOne), counters_before + 1);
}

// ── batch 119 helper shortcut lock-in test ──────────────────────────────────

#[test]
fn shortcut_on_other_dies_uses_another_of_yours_scope() {
    // Lock in that the new on_other_dies helper builds a
    // `CreatureDied/AnotherOfYours` event spec so future refactors
    // can't accidentally regress it to SelfSource (which would silently
    // make every "whenever another creature dies" trigger ignore other
    // deaths).
    use crabomination::effect::EventScope;
    use crabomination::effect::shortcut::on_other_dies;
    let trig = on_other_dies(Effect::GainLife {
        who: crabomination::effect::Selector::You,
        amount: crabomination::effect::Value::Const(1),
    });
    assert_eq!(trig.event.kind, crabomination::effect::EventKind::CreatureDied);
    assert!(matches!(trig.event.scope, EventScope::AnotherOfYours));
}

#[test]
fn fractal_hatchling_b119_grows_via_activated_ability() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::fractal_hatchling_b119());
    g.clear_sickness(id);
    let counters_before = g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    }).expect("activation");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne),
        counters_before + 1);
}

#[test]
fn witherbloom_apprentice_b120_magecraft_pumps_target_friendly() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(target);
    let app = g.add_card_to_battlefield(0, catalog::witherbloom_apprentice_b120());
    g.clear_sickness(app);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let lion = g.battlefield_find(target).expect("lion alive");
    assert_eq!(lion.power(), 3, "Lion 2/1 base +1/+1 magecraft = 3 power");
}

#[test]
fn pest_brooder_b120_dies_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_brooder_b120());
    g.clear_sickness(id);
    // Murder it manually.
    let card = g.battlefield_find_mut(id).unwrap();
    card.damage = (card.toughness() as u32) + 10;
    g.check_state_based_actions();
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 2, "two Pests minted on death");
}

#[test]
fn prismari_apprentice_b120_magecraft_scrys() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::prismari_apprentice_b120());
    g.clear_sickness(app);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::mountain());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry doesn't reduce library count.
    assert_eq!(g.players[0].library.len(), lib_before);
}

// ── batch 120 helper shortcut lock-in tests ─────────────────────────────────

#[test]
fn shortcut_drain_and_draw_drains_and_draws() {
    use crabomination::effect::shortcut::drain_and_draw;
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&drain_and_draw(2), &ctx).expect("drain_and_draw resolves");
    assert_eq!(g.players[0].life, life0_before + 2);
    assert_eq!(g.players[1].life, life1_before - 2);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn shortcut_drain_and_scry_drains_and_scrys() {
    use crabomination::effect::shortcut::drain_and_scry;
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let lib_before = g.players[0].library.len();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&drain_and_scry(3, 1), &ctx).expect("drain_and_scry resolves");
    assert_eq!(g.players[0].life, life0_before + 3);
    assert_eq!(g.players[1].life, life1_before - 3);
    // Scry doesn't draw — library count is unchanged.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn shortcut_drain_and_surveil_drains_and_surveils() {
    use crabomination::effect::shortcut::drain_and_surveil;
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&drain_and_surveil(1, 1), &ctx).expect("drain_and_surveil resolves");
    assert_eq!(g.players[0].life, life0_before + 1);
    assert_eq!(g.players[1].life, life1_before - 1);
}

#[test]
fn witherbloom_cultivator_b120_rejects_activation_without_fodder() {
    // No fodder creature on the battlefield — only Cultivator itself.
    // The pre-flight gate should reject the activation cleanly (no mana
    // burned, no tap consumed since this ability has no tap cost).
    let mut g = two_player_game();
    let cult = g.add_card_to_battlefield(0, catalog::witherbloom_cultivator_b120());
    g.clear_sickness(cult);
    g.players[0].mana_pool.add_colorless(1);
    let mana_before = g.players[0].mana_pool.colorless_amount();
    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: cult,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    });
    assert!(result.is_err(), "Activation rejected with no fodder");
    // Mana not consumed — clean rejection.
    assert_eq!(g.players[0].mana_pool.colorless_amount(), mana_before);
}

#[test]
fn witherbloom_bloodgrafter_b122_grows_on_sacrifice() {
    let mut g = two_player_game();
    let bg = g.add_card_to_battlefield(0, catalog::witherbloom_bloodgrafter_b122());
    g.clear_sickness(bg);
    // Use Cultcaller to sacrifice fodder, triggering Bloodgrafter's payoff.
    let cult = g.add_card_to_battlefield(0, catalog::pest_cultcaller_b122());
    g.clear_sickness(cult);
    let fodder = g.add_card_to_battlefield(0, catalog::savannah_lions());
    g.clear_sickness(fodder);
    g.players[0].mana_pool.add(Color::Black, 1);
    let p_before = g.battlefield_find(bg).unwrap().power();
    g.perform_action(GameAction::ActivateAbility {
        card_id: cult, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Cultcaller activation");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(bg).expect("Bloodgrafter alive").power();
    assert_eq!(p_after, p_before + 1, "Bloodgrafter grew by +1/+1 on sacrifice");
}

#[test]
fn witherbloom_necrotutor_b122_magecraft_returns_creature_to_top() {
    let mut g = two_player_game();
    let nt = g.add_card_to_battlefield(0, catalog::witherbloom_necrotutor_b122());
    g.clear_sickness(nt);
    let gy_creature = g.add_card_to_graveyard(0, catalog::savannah_lions());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before + 1,
        "creature returned to library top");
    // The top of library should be our creature
    let top = g.players[0].library.last().expect("library nonempty");
    assert_eq!(top.id, gy_creature, "Lions on top of library");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == gy_creature),
        "Lions no longer in graveyard");
}

#[test]
fn pest_brewmaster_b122_gains_life_on_other_pest_death() {
    // Place Brewmaster on the battlefield, then place another Pest. Use
    // Bonechanter (b121) — which sacs another creature to shrink a
    // target — to kill a Pest. Brewmaster's "another Pest dies → +1
    // life" trigger should fire.
    let mut g = two_player_game();
    let bw = g.add_card_to_battlefield(0, catalog::pest_brewmaster_b122());
    g.clear_sickness(bw);
    // Buff Brewmaster so the sac_other_filter prefers a Pest token over
    // the source's own Brewmaster.
    g.battlefield_find_mut(bw).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 5);
    // Now use Swarmcaller to mint 2 Pest tokens (and drain 2).
    let ps = g.add_card_to_hand(0, catalog::pest_swarmcaller_b122());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: ps, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Swarmcaller castable");
    drain_stack(&mut g);
    // Activate Bonechanter — it'll sac a Pest token (lowest-power
    // non-source creature) to shrink the opponent's creature.
    let bone = g.add_card_to_battlefield(0, catalog::witherbloom_bonechanter_b121());
    g.clear_sickness(bone);
    let opp = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.clear_sickness(opp);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bone, ability_index: 0,
        target: Some(Target::Permanent(opp)), additional_targets: Vec::new(), x_value: None,
    }).expect("Bonechanter activation");
    drain_stack(&mut g);
    // A Pest token died: Brewmaster's payoff (+1 life) + the token's
    // own die-trigger (+1 life) = +2 life.
    assert_eq!(g.players[0].life, l_before + 2,
        "Brewmaster + Pest-die trigger both fired");
}

#[test]
fn silverquill_mentor_b122_etb_gains_two_life_and_magecraft_pumps_friend() {
    let mut g = two_player_game();
    let sm = g.add_card_to_hand(0, catalog::silverquill_mentor_b122());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let l_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mentor castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l_before + 2, "ETB gained 2 life");

    // Cast a Bolt at the opponent → magecraft pumps Mentor +1/+1 EOT.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p_before = g.battlefield_find(sm).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let p_after = g.battlefield_find(sm).unwrap().power();
    assert_eq!(p_after, p_before + 1, "Mentor pumped via magecraft");
}

#[test]
fn lorehold_reliquaer_b122_mints_two_spirits_and_pings_opp() {
    let mut g = two_player_game();
    let lr = g.add_card_to_hand(0, catalog::lorehold_reliquaer_b122());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let opp_l = g.players[1].life;
    let bf_before = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: lr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reliquaer castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter()
        .filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2);
    assert_eq!(g.players[1].life, opp_l - 1, "1 damage to opp");
}

#[test]
fn lorehold_battlescryer_b122_is_haste_three_three_with_attack_trigger() {
    let mut g = two_player_game();
    let lb = g.add_card_to_battlefield(0, catalog::lorehold_battlescryer_b122());
    let c = g.battlefield_find(lb).unwrap();
    assert!(c.has_keyword(&Keyword::Haste));
    assert_eq!(c.power(), 3);
    assert_eq!(c.toughness(), 3);
    // Declare it as an attacker — the on-attack trigger pings opp for 1
    // (auto-targeted).
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let opp_l = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: lb,
        target: AttackTarget::Player(1),
    }])).expect("Battlescryer attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_l - 1, "1 damage from on-attack ping");
}

#[test]
fn prismari_loresage_b122_etb_loots() {
    let mut g = two_player_game();
    let pl = g.add_card_to_hand(0, catalog::prismari_loresage_b122());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island()); // for the discard
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pl, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Loresage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 1, "drew a card");
}

#[test]
fn prismari_sparkmage_b122_magecraft_pings_creature() {
    let mut g = two_player_game();
    let ps = g.add_card_to_battlefield(0, catalog::prismari_sparkmage_b122());
    g.clear_sickness(ps);
    let target = g.add_card_to_battlefield(1, catalog::savannah_lions()); // 2/1
    g.clear_sickness(target);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft auto-targets a creature (Lions: 2/1 → 1 damage marked, but
    // toughness 1 → dies via SBA).
    assert!(g.battlefield_find(target).is_none(),
        "Lions died from 1 damage on 1-toughness");
}

/// Velomachus's reveal cap reads its LIVE power: with a +2/+2 pump
/// (power 7), a 6-MV sorcery is now inside the "MV ≤ power" gate.
#[test]
fn velomachus_cap_follows_live_power() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let velo = g.add_card_to_battlefield(0, catalog::velomachus_lorehold());
    g.clear_sickness(velo);
    // +2/+2 in counters → power 7.
    g.battlefield_find_mut(velo).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    // Top of library: a 7-MV sorcery — inside the live power-7 cap,
    // outside the printed-power-5 cap the old wiring used.
    let big = g.add_card_to_library(0, catalog::moment_of_reckoning()); // {3}{W}{W}{B}{B} = MV 7
    let _ = big;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: velo,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.id == big && c.may_play_until.is_some()),
        "7-MV sorcery within the live-power cap was exiled with may-play"
    );
}

/// Mavinda's surcharge: the granted cast costs {8} more unless it
/// targets a creature you control (audit fix — was a flat {2}).
#[test]
fn mavinda_surcharge_applies_unless_targeting_own_creature() {
    use crabomination::game::types::Target;
    // Case 1: granted Giant Growth aimed at YOUR creature — costs just {G}.
    let mut g = two_player_game();
    let mavinda = g.add_card_to_battlefield(0, catalog::mavinda_students_advocate());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == mavinda) {
        c.summoning_sick = false;
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut gg = crabomination::card::CardInstance::new(g.next_id(), catalog::giant_growth(), 0);
    gg.controller = 0;
    let gg_id = gg.id;
    g.players[0].graveyard.push(gg);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mavinda, ability_index: 0,
        target: Some(Target::Permanent(gg_id)), additional_targets: Vec::new(), x_value: None,
    }).expect("Mavinda activation");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: gg_id,
        target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("targets your creature: no surcharge, {G} suffices");

    // Case 2: aimed at the OPPONENT's creature — {G} alone is rejected
    // (needs {G} + {8}).
    let mut g = two_player_game();
    let mavinda = g.add_card_to_battlefield(0, catalog::mavinda_students_advocate());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == mavinda) {
        c.summoning_sick = false;
    }
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut gg = crabomination::card::CardInstance::new(g.next_id(), catalog::giant_growth(), 0);
    gg.controller = 0;
    let gg_id = gg.id;
    g.players[0].graveyard.push(gg);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mavinda, ability_index: 0,
        target: Some(Target::Permanent(gg_id)), additional_targets: Vec::new(), x_value: None,
    }).expect("Mavinda activation");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Green, 1);
    assert!(g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: gg_id,
        target: Some(Target::Permanent(enemy)), additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "opponent-creature target: G alone can't cover the +8");
    // With {8} more floated the cast goes through.
    g.players[0].mana_pool.add_colorless(8);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: gg_id,
        target: Some(Target::Permanent(enemy)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("with 8 more the surcharged cast resolves");
}
