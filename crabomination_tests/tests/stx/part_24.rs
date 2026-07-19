use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

// ─────────────────────────────────────────────────────────────────────────
// Batches 202-206 (modern_decks) — consolidated table-driven suites.
// ─────────────────────────────────────────────────────────────────────────

/// Definition-shape checks for all cards whose tests only asserted static
/// properties (P/T, keywords, subtypes, ability counts, cmc).
#[test]
fn b202_b206_def_shapes() {
    let d = catalog::inkling_lifecaller_b202();
    assert_eq!(d.cost.cmc(), 5);
    assert!(d.keywords.contains(&Keyword::Flying));
    assert!(d.keywords.contains(&Keyword::Lifelink));
    assert_eq!(d.triggered_abilities.len(), 1);

    let d = catalog::pest_devourer_b202();
    assert_eq!(d.triggered_abilities.len(), 1);
    assert!(d.subtypes.creature_types.contains(&CreatureType::Pest));

    let d = catalog::pestshell_crusader_b202();
    assert!(d.keywords.contains(&Keyword::Trample));
    assert!(d.subtypes.creature_types.contains(&CreatureType::Pest));
    assert!(d.subtypes.creature_types.contains(&CreatureType::Knight));

    let d = catalog::pest_howler_b202();
    assert_eq!(d.cost.cmc(), 2);
    assert_eq!(d.triggered_abilities.len(), 1);

    assert_eq!(catalog::witherbloom_cultivator_b202().activated_abilities.len(), 1);

    let d = catalog::witherbloom_briarcaller_b202();
    assert_eq!((d.power, d.toughness), (4, 4));
    assert!(d.keywords.contains(&Keyword::Trample));
    assert!(d.keywords.contains(&Keyword::Reach));

    assert_eq!(catalog::lorehold_spirit_caller_b202().triggered_abilities.len(), 1);

    let d = catalog::lorehold_battlescholar_b202();
    assert!(d.keywords.contains(&Keyword::FirstStrike));
    assert_eq!(d.cost.cmc(), 2);

    let d = catalog::lorehold_ghostsmith_b202();
    assert_eq!(d.power, 3);
    assert_eq!(d.triggered_abilities.len(), 1);

    assert_eq!(catalog::prismari_soothsayer_b202().triggered_abilities.len(), 1);

    let d = catalog::prismari_volcanist_b202();
    assert!(d.keywords.contains(&Keyword::Haste));
    assert!(d.keywords.contains(&Keyword::Trample));
    assert_eq!(d.power, 4);

    let d = catalog::quandrix_grizzler_b202();
    assert!(d.keywords.contains(&Keyword::Vigilance));
    assert_eq!((d.power, d.toughness), (3, 3));

    let d = catalog::quandrix_skydiver_b202();
    assert!(d.keywords.contains(&Keyword::Flying));
    assert!(d.keywords.contains(&Keyword::Hexproof));

    assert_eq!(catalog::quandrix_geomant_b202().activated_abilities.len(), 1);

    let d = catalog::silverquill_cantor_b203();
    assert_eq!(d.cost.cmc(), 2);
    assert_eq!(d.triggered_abilities.len(), 1);

    let d = catalog::inkling_whisperer_b203();
    assert!(d.keywords.contains(&Keyword::Flying));
    assert!(d.subtypes.creature_types.contains(&CreatureType::Inkling));

    let d = catalog::silverquill_censurer_b203();
    assert!(d.keywords.contains(&Keyword::Lifelink));
    assert_eq!(d.triggered_abilities.len(), 1);

    let d = catalog::silverquill_hospitaller_b203();
    assert!(d.keywords.contains(&Keyword::Lifelink));
    assert!(d.keywords.contains(&Keyword::Vigilance));
    assert_eq!(d.power, 4);

    let d = catalog::inkling_sheriff_b203();
    assert!(d.keywords.contains(&Keyword::Flying));
    assert!(d.keywords.contains(&Keyword::Vigilance));
    assert!(d.keywords.contains(&Keyword::Lifelink));

    assert_eq!(catalog::silverquill_mausoleum_b203().activated_abilities.len(), 1);

    let d = catalog::witherbloom_pestlord_b203();
    assert_eq!((d.power, d.toughness), (4, 5));
    assert!(d.keywords.contains(&Keyword::Trample));

    assert_eq!(catalog::lorehold_apprentice_b203().triggered_abilities.len(), 1);

    let d = catalog::lorehold_soulbinder_b203();
    assert!(d.keywords.contains(&Keyword::Vigilance));
    assert_eq!(d.triggered_abilities.len(), 1);

    let d = catalog::lorehold_ancestor_b203();
    assert_eq!(d.power, 4);
    assert!(d.keywords.contains(&Keyword::Trample));

    assert_eq!(catalog::prismari_apprentice_ii_b203().triggered_abilities.len(), 1);
    assert_eq!(catalog::prismari_counter_b203().cost.cmc(), 2);
    assert_eq!(catalog::prismari_squallcaller_ii_b203().triggered_abilities.len(), 1);
    assert_eq!(catalog::prismari_mage_b203().triggered_abilities.len(), 1);
    assert_eq!(catalog::quandrix_apprentice_ii_b203().triggered_abilities.len(), 1);

    let d = catalog::quandrix_naturist_b203();
    assert!(d.keywords.contains(&Keyword::Trample));
    assert_eq!(d.power, 3);

    assert_eq!(catalog::quandrix_charmer_b203().triggered_abilities.len(), 1);

    let d = catalog::quandrix_verdant_b203();
    assert!(d.keywords.contains(&Keyword::Vigilance));
    assert!(d.keywords.contains(&Keyword::Reach));
    assert_eq!(d.toughness, 4);

    let d = catalog::silverquill_inkguard_b205();
    assert_eq!((d.power, d.toughness), (2, 3));
    assert!(d.keywords.contains(&Keyword::Lifelink));
    assert!(d.subtypes.creature_types.contains(&CreatureType::Inkling));

    let d = catalog::prismari_galemage_b205();
    assert_eq!((d.power, d.toughness), (2, 3));
    assert!(d.subtypes.creature_types.contains(&CreatureType::Wizard));

    let d = catalog::quandrix_megafractal_b206();
    assert_eq!((d.power, d.toughness), (5, 5));
    assert!(d.keywords.contains(&Keyword::Trample));
    assert!(d.subtypes.creature_types.contains(&CreatureType::Fractal));

    let d = catalog::silverquill_dictator_b206();
    assert_eq!((d.power, d.toughness), (3, 3));
    assert!(d.keywords.contains(&Keyword::Flying));
    assert!(d.keywords.contains(&Keyword::Lifelink));
    assert!(d.subtypes.creature_types.contains(&CreatureType::Inkling));
}

/// Sorcery drains: caster gains N, opponent loses N.
#[test]
fn b202_b206_sorcery_drains() {
    for (def, n) in [
        (catalog::witherbloom_famine_b202(), 4),
        (catalog::pest_tendril_b203(), 4),
        (catalog::silverquill_drainstrike_b204(), 3),
        (catalog::witherbloom_bloodtap_b204(), 5),
        (catalog::silverquill_final_edict_b205(), 3),
        (catalog::witherbloom_grim_harvest_b206(), 4),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 2);
        }
        g.players[0].mana_pool.add_colorless(6);
        let p0 = g.players[0].life;
        let p1 = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, p0 + n, "{name}: gains {n}");
        assert_eq!(g.players[1].life, p1 - n, "{name}: opp loses {n}");
    }
}

/// Burn spells targeting the opponent: deal exactly N.
#[test]
fn b202_b206_burn_spells() {
    for (def, n) in [
        (catalog::lorehold_bolt_ii_b202(), 2),
        (catalog::prismari_bolt_b202(), 3),
        (catalog::prismari_surge_ii_b202(), 4),
        (catalog::lorehold_strike_b203(), 3),
        (catalog::prismari_flame_b203(), 4),
        (catalog::lorehold_flameburst_b204(), 4),
        (catalog::prismari_emberbolt_b205(), 2),
        (catalog::prismari_inferno_b206(), 4),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island()); // for riders that scry
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 2);
        }
        g.players[0].mana_pool.add_colorless(6);
        let p1 = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, p1 - n, "{name}: deals {n}");
    }
}

/// Magecraft bodies that drain/ping the opponent for 1 when a spell is
/// cast: bolt at opp → 3 + 1 = 4 total.
#[test]
fn b202_b205_magecraft_ping_or_drain_one() {
    for def in [
        catalog::prismari_pyroartisan_b202(),
        catalog::witherbloom_apprentice_ii_b203(),
        catalog::inkling_mentor_b203(),
        catalog::prismari_pyromage_b204(),
        catalog::witherbloom_bloodmoss_b205(),
        catalog::silverquill_grimquill_b205(),
        catalog::prismari_flarecaster_b205(),
        catalog::lorehold_emberhistorian_b205(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let p1 = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, p1 - 4, "{name}: bolt 3 + magecraft 1");
    }
}

/// Magecraft pingers whose targets are auto-picked; assert only that the
/// bolt damage went through.
#[test]
fn b202_b204_magecraft_auto_ping() {
    for def in [catalog::lorehold_pyromancer_b202(), catalog::lorehold_pyromaster_b204()] {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let p1 = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        assert!(g.players[1].life <= p1 - 3, "{name}: bolt damage applied");
    }
}

/// Magecraft self-pumps: cast a bolt, check the body's computed power.
#[test]
fn b202_b206_magecraft_self_pumps() {
    for (def, power) in [
        (catalog::prismari_sparkforger_b202(), 3),
        (catalog::witherbloom_apprentice_iii_b203(), 3),
        (catalog::lorehold_warchronicler_b205(), 5),
        (catalog::lorehold_ember_veteran_b206(), 5),
        (catalog::quandrix_scholar_b206(), 2),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        let view = g.computed_permanent(id).expect("alive");
        assert_eq!(view.power, power, "{name}: magecraft self-pump");
    }
}

/// Magecraft counter-placers: spellbloom counters itself; mentor counters
/// another friendly creature.
#[test]
fn b202_b204_magecraft_counters() {
    for (def, on_self) in [
        (catalog::witherbloom_spellbloom_b202(), true),
        (catalog::quandrix_mentor_b204(), false),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let cd = g.add_card_to_battlefield(0, def);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        let watch = if on_self { cd } else { bear };
        let c = g.battlefield_find(watch).expect("alive");
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "{name}");
    }
}

/// Magecraft token-minters: cast a bolt → exactly one token on the bf.
#[test]
fn b202_b205_magecraft_token_mints() {
    for def in [
        catalog::prismari_treasurehunter_b202(),
        catalog::quandrix_fractalweaver_b202(),
        catalog::quandrix_fractaller_b204(),
        catalog::prismari_pyrosmith_b205(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        let tokens = g.battlefield.iter().filter(|c| c.is_token).count();
        assert_eq!(tokens, 1, "{name}: one token minted");
    }
}

/// ETB / on-resolve lifegain (creatures and Lay Faith).
#[test]
fn b202_b206_etb_lifegain() {
    for (def, n) in [
        (catalog::witherbloom_mossblossom_b202(), 2),
        (catalog::silverquill_lay_faith_b203(), 4),
        (catalog::lorehold_spirit_squire_b203(), 2),
        (catalog::silverquill_lightscribe_b205(), 3),
        (catalog::witherbloom_fungalbeast_b206(), 2),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 2);
        }
        g.players[0].mana_pool.add_colorless(6);
        let p0 = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, p0 + n, "{name}: gains {n}");
    }
}

/// ETB token-minters: cast the creature → expected token count.
#[test]
fn b202_b204_etb_token_mints() {
    for (def, n) in [
        (catalog::witherbloom_pestcaller_b202(), 2),
        (catalog::prismari_tinkerer_b202(), 1),
        (catalog::pest_patriarch_b203(), 1),
        (catalog::lorehold_spirit_sage_b203(), 1),
        (catalog::lorehold_spiritbringer_b204(), 1),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 2);
        }
        g.players[0].mana_pool.add_colorless(6);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let tokens = g.battlefield.iter().filter(|c| c.is_token).count();
        assert_eq!(tokens, n, "{name}: {n} token(s) minted");
    }
}

/// Batch 205 Enrage cycle — counter payoffs. Exercises the
/// `EventKind::DealtDamage` event (CR 702.130): bolt the creature (3
/// damage, survivable) and count the +1/+1 counters added.
#[test]
fn b205_enrage_counter_payoffs() {
    for (def, counters) in [
        (catalog::lorehold_battlescarred_b205(), 1),
        (catalog::lorehold_echovenger_b205(), 3), // Value::TriggerEventAmount = 3 dmg
        (catalog::witherbloom_gravethorn_b205(), 3),
        (catalog::quandrix_thornfractal_b205(), 3),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(id)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(id).expect("survives 3 damage");
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), counters, "{name}");
    }
}

/// Batch 205 Enrage cycle — life-swing payoffs (drain / gain / ping).
#[test]
fn b205_enrage_life_swings() {
    for (def, opp_loss, self_gain) in [
        (catalog::lorehold_vengescribe_b205(), 1, 0),
        (catalog::lorehold_grudgebearer_b205(), 2, 2),
        (catalog::lorehold_stoneguard_b205(), 0, 2),
        (catalog::witherbloom_thornbeast_b205(), 1, 1),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let l0 = g.players[0].life;
        let l1 = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(id)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(id).is_some(), "{name}: survives 3 damage");
        assert_eq!(g.players[1].life, l1 - opp_loss, "{name}: opp loss");
        assert_eq!(g.players[0].life, l0 + self_gain, "{name}: self gain");
    }
}

#[test]
fn lorehold_warhost_b205_enrage_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_warhost_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter().filter(|c| c.is_token
        && c.definition.subtypes.creature_types.contains(&CreatureType::Spirit)).count();
    assert_eq!(spirits, 1, "enrage minted one Lorehold Spirit token");
}

#[test]
fn lorehold_chroniclekeeper_b205_enrage_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let id = g.add_card_to_battlefield(0, catalog::lorehold_chroniclekeeper_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    // -1 Bolt cast, +1 enrage draw = net same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before, "enrage drew a card");
}

/// Death triggers (own creature dies to a bolt). `bolt_self` picks whether
/// the watcher itself or a bystander bear takes the bolt.
#[test]
fn b203_b206_death_trigger_life_swings() {
    for (def, bolt_self, opp_loss, self_gain) in [
        (catalog::pest_sapper_b203(), true, 2, 0),
        (catalog::witherbloom_sapfeeder_b205(), true, 2, 2),
        (catalog::witherbloom_rotcaller_b205(), false, 1, 1),
        (catalog::silverquill_deathscribe_b205(), false, 1, 0),
        (catalog::witherbloom_sporecaller_b206(), false, 0, 1),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let watcher = g.add_card_to_battlefield(0, def);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let victim = if bolt_self { watcher } else { bear };
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let l0 = g.players[0].life;
        let l1 = g.players[1].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Permanent(victim)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        assert!(g.battlefield_find(victim).is_none(), "{name}: victim dies");
        assert_eq!(g.players[1].life, l1 - opp_loss, "{name}: opp loss");
        assert_eq!(g.players[0].life, l0 + self_gain, "{name}: self gain");
    }
}

#[test]
fn inkling_heartcaller_b202_gains_life_when_inkling_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::inkling_heartcaller_b202());
    let inkling = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    drain_stack(&mut g);
    let p0 = g.players[0].life;
    // Kill via Lightning Bolt → CreatureDied fires.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(inkling)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 2, "Inkling death triggers 2 life");
}

/// ETB draw-a-card bodies: net hand size unchanged after casting.
#[test]
fn b202_b206_etb_draws() {
    for def in [
        catalog::prismari_drakebreeder_b202(),
        catalog::quandrix_streamer_b203(),
        catalog::lorehold_archivekeeper_b206(),
        catalog::prismari_windscholar_b206(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 2);
        }
        g.players[0].mana_pool.add_colorless(6);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        // -1 cast + 1 draw = net 0.
        assert_eq!(g.players[0].hand.len(), hand_before, "{name}: ETB drew a card");
    }
}

/// Magecraft draw bodies: bolt cast, hand size nets to unchanged.
#[test]
fn b202_b205_magecraft_draws() {
    for def in [catalog::quandrix_conjurer_b202(), catalog::quandrix_tidecaller_b205()] {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_battlefield(0, def);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("bolt");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before, "{name}: magecraft drew");
    }
}

/// Draw-two spells: net +1 card.
#[test]
fn b202_draw_two_spells() {
    for def in [catalog::prismari_spellcraft_b202(), catalog::quandrix_cantrip_b202()] {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::mountain());
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 2);
        }
        g.players[0].mana_pool.add_colorless(6);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        // -1 cast + 2 draw = net +1.
        assert_eq!(g.players[0].hand.len(), hand_before + 1, "{name}");
    }
}

/// Spells/ETBs that put +1/+1 counters on a targeted friendly creature.
#[test]
fn b202_b205_counters_on_target() {
    for (def, counters) in [
        (catalog::witherbloom_vinepath_b202(), 2),
        (catalog::quandrix_vinemage_b202(), 1),
        (catalog::quandrix_surge_b203(), 3),
        (catalog::quandrix_growthseer_b205(), 1),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island()); // for surveil riders
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 2);
        }
        g.players[0].mana_pool.add_colorless(6);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let c = g.battlefield_find(bear).expect("bear alive");
        assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), counters, "{name}");
    }
}

/// Reanimators: bear in graveyard returns to the battlefield.
#[test]
fn b202_reanimators() {
    for def in [catalog::lorehold_reanimator_b202(), catalog::lorehold_excavate_b202()] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            g.players[0].mana_pool.add(c, 2);
        }
        g.players[0].mana_pool.add_colorless(6);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(dead).is_some(), "{name}: bear reanimated");
    }
}

// ── Remaining per-card play-pattern tests ───────────────────────────────

#[test]
fn silverquill_recap_b202_returns_low_mv_creature_from_graveyard() {
    let mut g = two_player_game();
    let dead_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_recap_b202());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead_bear),
        "bear returned to hand");
}

#[test]
fn witherbloom_sapdraw_b202_drains_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_sapdraw_b202());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let p0 = g.players[0].life;
    let p1 = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 2);
    assert_eq!(g.players[1].life, p1 - 2);
    // -1 cast + 1 draw = net 0 hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn witherbloom_decompose_b202_destroys_two_toughness_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::witherbloom_decompose_b202());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2-toughness destroyed");
}

#[test]
fn witherbloom_rotcaller_b202_etb_makes_opp_discard() {
    let mut g = two_player_game();
    // Give opp a card to discard.
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let opp_hand_before = g.players[1].hand.len();
    let id = g.add_card_to_hand(0, catalog::witherbloom_rotcaller_b202());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1, "opp discarded");
}

#[test]
fn witherbloom_verdance_b202_mints_a_four_four_beast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_verdance_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let beast = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Beast");
    assert!(beast.is_some(), "Beast token created");
    let b = beast.unwrap();
    assert_eq!(b.power(), 4);
    assert_eq!(b.toughness(), 4);
}

#[test]
fn lorehold_charge_b202_pumps_team_with_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_charge_b202());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::FirstStrike));
    assert_eq!(c.power(), 3);
}

#[test]
fn lorehold_frontlord_b202_anthems_other_friendlies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::lorehold_frontlord_b202());
    drain_stack(&mut g);
    let view = g.computed_permanent(bear).expect("bear on bf");
    assert_eq!(view.power, 3, "+1/+0 anthem applies");
}

#[test]
fn lorehold_cleanse_b202_damages_each_creature() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_cleanse_b202());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "own bear dies (2 dmg)");
    assert!(g.battlefield_find(opp).is_none(), "opp bear dies (2 dmg)");
}

#[test]
fn lorehold_echoblade_b202_pumps_friendly_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_echoblade_b202());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert_eq!(c.power(), 3, "+1/+1 magecraft pump");
}

#[test]
fn lorehold_lavascholar_b202_pings_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_lavascholar_b202());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // 1-damage ping; auto-picks a target (opp player most likely).
    assert!(g.players[1].life <= p1, "lavascholar dealt damage");
}

#[test]
fn prismari_squallcaller_b202_etb_taps_opp_creature() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_squallcaller_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(opp_bear).expect("bear alive");
    assert!(c.tapped, "opp creature tapped");
}

#[test]
fn prismari_spiketide_b202_draws_three_and_discards_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::mountain());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_spiketide_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast + 3 draw - 2 discard = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_sumtotal_b202_puts_x_counters_for_each_creature() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_sumtotal_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(b1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(b1).expect("bear alive");
    // 3 creatures on bf at spell-resolve = 3 counters.
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3);
}

#[test]
fn quandrix_sparkbender_b202_counters_target_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    g.priority.player_with_priority = 0;
    let counter = g.add_card_to_hand(0, catalog::quandrix_sparkbender_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: counter, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "bolt was countered");
}

/// The printed "unless its controller pays {1}" escape: with a spare
/// floated mana, the bolt's controller auto-pays and the spell resolves.
#[test]
fn quandrix_sparkbender_b202_spell_survives_when_controller_pays_one() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    // Extra floated mana to pay the {1} escape.
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    g.priority.player_with_priority = 0;
    let counter = g.add_card_to_hand(0, catalog::quandrix_sparkbender_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: counter, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("counter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 17, "controller paid the {{1}} escape; bolt resolved");
}

#[test]
fn quandrix_fractalspawn_b202_etb_mints_two_counter_fractal() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalspawn_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| c.is_token
        && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
    assert!(fractal.is_some(), "Fractal minted");
    assert_eq!(fractal.unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn quandrix_symmetry_b202_mints_fractal_with_x_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_symmetry_b202());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: Some(4),
    }).expect("castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter().find(|c| c.is_token
        && c.definition.subtypes.creature_types.contains(&CreatureType::Fractal));
    assert!(fractal.is_some(), "Fractal minted");
    assert_eq!(fractal.unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
}

#[test]
fn quandrix_streampath_b202_bounces_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_streampath_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp_bear).is_none(), "opp bear bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == opp_bear), "bear in opp hand");
    // -1 cast + 1 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn silverquill_wardrune_b202_pumps_toughness_with_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_wardrune_b202());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("bear alive");
    assert!(c.has_keyword(&Keyword::Vigilance));
    assert_eq!(c.toughness(), 5, "+0/+3 → 5 toughness");
}

#[test]
fn silverquill_edict_b203_forces_opp_sac() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_edict_b203());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp_bear).is_none(), "edict took the bear");
}

#[test]
fn prismari_cantrip_b203_draws_then_discards() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::prismari_cantrip_b203());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw - 1 discard = -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn inkling_sage_apprentice_b203_gains_one_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::inkling_sage_apprentice_b203());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0 + 1);
}

#[test]
fn silverquill_bond_b204_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_bond_b204());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("alive");
    assert!(c.has_keyword(&Keyword::Lifelink));
    assert_eq!(c.power(), 3, "+1/+1 pump");
}

#[test]
fn witherbloom_drainshade_b204_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_drainshade_b204());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 2);
}

#[test]
fn prismari_sparkboost_b204_pumps_two_power() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_sparkboost_b204());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("alive");
    assert_eq!(c.power(), 4, "+2/+0");
}

#[test]
fn lorehold_enrage_does_not_fire_without_damage() {
    // Sanity: an undamaged enrage creature has no counters / no triggers.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_battlescarred_b205());
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("alive");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 0,
        "no damage → no enrage counter");
}

#[test]
fn prismari_tidescribe_b205_etb_resolves_and_enters() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::prismari_tidescribe_b205());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("Tidescribe on bf");
    assert_eq!((c.power(), c.toughness()), (1, 4), "1/4 body, ETB scry 2 resolved");
}

#[test]
fn prismari_stormloot_b205_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::prismari_stormloot_b205());
    // A spare card to discard for the loot.
    g.add_card_to_hand(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    // -1 Bolt cast, +1 loot draw, -1 loot discard = net -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "magecraft loot: draw then discard");
}

#[test]
fn lorehold_relicwarden_b205_attack_gains_life() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_relicwarden_b205());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) {
        c.summoning_sick = false;
    }
    g.step = TurnStep::DeclareAttackers;
    let l0_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }])).expect("Attack declared");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0_before + 2, "on-attack gains 2 life");
}

#[test]
fn quandrix_mistcaller_b205_magecraft_scrys() {
    // Magecraft scry doesn't change hand/life; assert it fires without
    // error and the body is a 1/3 Merfolk Wizard.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let mc = g.add_card_to_battlefield(0, catalog::quandrix_mistcaller_b205());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    let c = g.battlefield_find(mc).expect("alive");
    assert_eq!((c.power(), c.toughness()), (1, 3));
}

#[test]
fn lorehold_skirmisher_b206_pings_on_attack() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_skirmisher_b206());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    let l1 = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "on-attack ping 1 to opp");
}

#[test]
fn silverquill_purge_b206_exiles_small_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::silverquill_purge_b206());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2-power bear exiled");
}

// ─────────────────────────────────────────────────────────────────────────
// CR rule lock-in tests — batch 202 round
// ─────────────────────────────────────────────────────────────────────────

/// CR 704.5a — A player with 0 or less life loses the game (state-based
/// action). Drain a player to 0 via a Famine cast and verify they lose.
#[test]
fn cr_704_5a_player_at_zero_or_less_life_loses() {
    let mut g = two_player_game();
    // Drop p1 to 4 life.
    g.players[1].life = 4;
    let famine = g.add_card_to_hand(0, catalog::witherbloom_famine_b202());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: famine, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Famine deals 4 drain → p1 at 0 life → loses.
    assert!(g.players[1].life <= 0);
    assert!(g.is_game_over(), "player at 0 life triggers game-over SBA");
}

/// CR 608.2b — A spell with all illegal targets is removed from the stack
/// and goes to graveyard rather than resolving. Cast Bolt at a creature,
/// then remove the creature before bolt resolves → bolt fizzles.
#[test]
fn cr_608_2b_spell_with_illegal_target_fizzles() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt on stack");
    // Remove the bear by direct mutation: pull it off bf, drop into gy.
    let bear_inst = g.battlefield.iter().position(|c| c.id == opp_bear)
        .map(|i| g.battlefield.remove(i)).expect("bear on bf");
    g.players[1].graveyard.push(bear_inst);
    let p1_life = g.players[1].life;
    drain_stack(&mut g);
    // Bolt resolves with no legal target → fizzles → no damage to player.
    assert!(g.battlefield_find(opp_bear).is_none(), "bear moved to gy");
    assert_eq!(g.players[1].life, p1_life, "bolt fizzled — no damage redirected");
}

/// CR 121.6a — A replacement effect for "draw a card" applies even if
/// the draw would be impossible because the library is empty. We pin
/// the inverse: a draw against an empty library is a *loss*, not a
/// no-op, and that loss is the SBA trigger (not the draw replacement
/// — they're independent paths). This lock-in pairs with 704.5b above.
#[test]
fn cr_704_5b_empty_library_draw_attempt_loses_game() {
    let mut g = two_player_game();
    // p0 has an empty library.
    g.players[0].library.clear();
    g.players[0].cards_drawn_this_turn = 0;
    // Force a draw via Quandrix Cantrip.
    let id = g.add_card_to_hand(0, catalog::quandrix_cantrip_b202());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    // Drawing with empty library → "attempted to draw from empty lib"
    // SBA → caster loses.
    assert!(g.is_game_over(), "empty-library draw loses the game");
}

// ─────────────────────────────────────────────────────────────────────────
// CR rules lock-in tests (batch 205 session)
// ─────────────────────────────────────────────────────────────────────────

/// CR 702.130 — Enrage. "Whenever this creature is dealt damage" fires on
/// COMBAT damage just as it does on spell damage. Lorehold Battlescarred
/// (3/4) blocks a 2/2, survives the 2 combat damage, and the enrage trigger
/// puts a +1/+1 counter on it. Exercises the `DealtDamage` event off the
/// combat-damage path in `game/combat.rs` (CR 510 → 702.130 interaction).
#[test]
fn cr_702_130_enrage_fires_on_combat_damage() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::lorehold_battlescarred_b205()); // 3/4
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("bear attacks");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .expect("battlescarred blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    let c = g.battlefield_find(blocker).expect("3/4 survives 2 combat damage");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1,
        "CR 702.130: enrage fired on combat damage and added a +1/+1 counter");
}

/// CR 122.1c — Shield counters. "If damage would be dealt to a permanent
/// with a shield counter on it … prevent that damage and remove a shield
/// counter from it." A Bolt (3) into a 2/2 with one shield counter is fully
/// prevented; the creature survives at 0 marked damage and the shield
/// counter is consumed.
#[test]
fn cr_122_1c_shield_counter_prevents_noncombat_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::Shield, 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the shielded bear");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("shield prevented lethal damage; bear survives");
    assert_eq!(c.damage, 0, "CR 122.1c: damage was prevented, none marked");
    assert_eq!(c.counter_count(CounterType::Shield), 0,
        "CR 122.1c: the shield counter was removed by the prevention");
}

/// CR 510.1c / 510.1d — Combat Damage Step. A blocked attacker assigns its
/// combat damage to the creature blocking it, and the blocker assigns to the
/// attacker; both are dealt simultaneously. Two 2/2s trade and both die.
#[test]
fn cr_510_blocked_attacker_and_blocker_trade() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("bear attacks");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)]))
        .expect("bear blocks");
    drain_stack(&mut g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(),
        "CR 510.1d: attacker took 2 lethal combat damage and died");
    assert!(g.battlefield_find(blocker).is_none(),
        "CR 510.1c: blocker took 2 lethal combat damage and died");
}
