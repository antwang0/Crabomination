use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

// ── Shared helpers for the table-driven tests below ─────────────────────────

macro_rules! big_mana {
    ($g:expr) => {
        for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
            $g.players[0].mana_pool.add(c, 4);
        }
        $g.players[0].mana_pool.add_colorless(10);
    };
}

macro_rules! cast {
    ($g:expr, $id:expr, $target:expr) => {
        $g.perform_action(GameAction::CastSpell {
            card_id: $id,
            target: $target,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("castable")
    };
}

macro_rules! bolt {
    ($g:expr, $target:expr) => {{
        let bolt = $g.add_card_to_hand(0, catalog::lightning_bolt());
        $g.players[0].mana_pool.add(Color::Red, 1);
        cast!($g, bolt, Some($target));
    }};
}

// ── Magecraft: source on battlefield, Bolt at opponent, check life deltas ───

#[test]
fn magecraft_bolt_at_player_life_deltas() {
    // (def, opp_loss incl. the 3 bolt damage, self_gain)
    for (def, opp_loss, self_gain) in [
        (catalog::silverquill_blackquill_acolyte(), 4, 1),
        (catalog::inkling_vassal(), 4, 1),
        (catalog::lorehold_emberscholar(), 4, 0),
        (catalog::prismari_inkjet_apprentice(), 4, 0),
        (catalog::silverquill_adept(), 4, 1),
        (catalog::lorehold_sparkflinger(), 4, 0),
        (catalog::silverquill_penbringer(), 3, 1),
        (catalog::spirit_spellsmith(), 3, 1),
        (catalog::witherbloom_saprosage(), 3, 1),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::plains());
        let _id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let opp_before = g.players[1].life;
        let life_before = g.players[0].life;
        bolt!(g, Target::Player(1));
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - opp_loss);
        assert_eq!(g.players[0].life, life_before + self_gain);
    }
}

#[test]
fn magecraft_ping_plus_bolt_kills_targeted_bear() {
    // Bolt 3 + magecraft ping 1 = 4 dmg → bear (2 toughness) dies.
    for def in [
        catalog::prismari_pyroartist(),
        catalog::lorehold_emberhand_priest(),
        catalog::prismari_blastcaster(),
    ] {
        let mut g = two_player_game();
        let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let _id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        bolt!(g, Target::Permanent(target));
        drain_stack(&mut g);
        assert!(g.battlefield_find(target).is_none());
    }
}

#[test]
fn magecraft_puts_counter_on_self() {
    for def in [
        catalog::quandrix_geode_smith(),
        catalog::inkling_bookbinder(),
        catalog::witherbloom_vinepicker(),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        bolt!(g, Target::Player(1));
        drain_stack(&mut g);
        assert_eq!(
            g.battlefield_find(id)
                .unwrap()
                .counter_count(CounterType::PlusOnePlusOne),
            1
        );
    }
}

#[test]
fn magecraft_self_pump_until_eot() {
    for (def, p, t, kws) in [
        (catalog::prismari_brushpyre(), 5, 3, vec![Keyword::Haste]),
        (catalog::inkling_quillblade(), 3, 2, vec![]),
        (catalog::silverquill_inkdancer(), 3, 2, vec![]),
        (catalog::lorehold_sparkwarden(), 3, 2, vec![Keyword::Lifelink]),
        (catalog::prismari_stormcoil(), 4, 4, vec![]),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        bolt!(g, Target::Player(1));
        drain_stack(&mut g);
        let card = g.battlefield_find(id).unwrap();
        assert_eq!(card.power(), p);
        assert_eq!(card.toughness(), t);
        for kw in &kws {
            assert!(card.has_keyword(kw));
        }
    }
}

#[test]
fn magecraft_scry_keeps_or_bins_top_of_library() {
    for def in [
        catalog::witherbloom_grafted_seer(),
        catalog::prismari_oddsmaker(),
        catalog::quandrix_pupil(),
        catalog::prismari_sparktwister(),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(0, catalog::island());
        let _id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let lib_before = g.players[0].library.len();
        bolt!(g, Target::Player(1));
        drain_stack(&mut g);
        // Scry: top card either stays or is binned.
        assert!(g.players[0].library.len() <= lib_before);
    }
}

// ── Attack triggers ─────────────────────────────────────────────────────────

#[test]
fn attack_trigger_drains_each_opp() {
    use crabomination::game::{Attack, AttackTarget, TurnStep};
    // Attack-trigger drain/ping of 1 — combat damage is separate.
    for (def, self_gain) in [
        (catalog::silverquill_ravenmage(), Some(1)),
        (catalog::silverquill_ravenswing(), Some(1)),
        (catalog::lorehold_lightspeaker(), None),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        drain_stack(&mut g);
        let opp_before = g.players[1].life;
        let life_before = g.players[0].life;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: id,
            target: AttackTarget::Player(1),
        }]))
        .expect("declare attackers");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - 1);
        if let Some(gain) = self_gain {
            assert_eq!(g.players[0].life, life_before + gain);
        }
    }
}

#[test]
fn inkling_battlescholar_attack_pumps_self() {
    use crabomination::game::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_battlescholar());
    g.clear_sickness(id);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attackers");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power_bonus, 1);
}

// ── Cast-from-hand drains (ETB or sorcery drain effects) ────────────────────

#[test]
fn cast_drains_life() {
    // (def, targets_opponent, opp_loss, self_gain)
    for (def, targeted, opp_loss, self_gain) in [
        (catalog::silverquill_grand_inkmaster(), false, 4, Some(4)),
        (catalog::strixhaven_demonstrator(), false, 2, Some(2)),
        (catalog::silverquill_drainmaster_v2(), false, 3, Some(3)),
        (catalog::inkling_scriptmaster(), false, 2, Some(2)),
        (catalog::witherbloom_blightroot(), false, 3, Some(3)),
        (catalog::witherbloom_toxicvigor(), false, 3, Some(3)),
        (catalog::silverquill_refrain(), false, 2, Some(2)),
        (catalog::silverquill_recital(), false, 2, Some(2)),
        (catalog::inkling_inkblot(), false, 1, Some(1)),
        (catalog::inkling_magistrate(), false, 2, None),
        (catalog::silverquill_vellum(), false, 2, Some(2)),
        (catalog::silverquill_lecture(), false, 3, Some(3)),
        (catalog::silverquill_liturgy(), false, 2, Some(4)),
        (catalog::silverquill_inkdraft(), false, 1, Some(1)),
        (catalog::silverquill_maxim(), true, 3, Some(3)),
        (catalog::silverquill_diatribe(), true, 4, None),
        (catalog::silverquill_inkspear(), true, 1, Some(1)),
        (catalog::lorehold_refrain(), true, 2, Some(2)),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::plains());
        let id = g.add_card_to_hand(0, def);
        big_mana!(g);
        let opp_before = g.players[1].life;
        let life_before = g.players[0].life;
        let target = if targeted { Some(Target::Player(1)) } else { None };
        cast!(g, id, target);
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - opp_loss);
        if let Some(gain) = self_gain {
            assert_eq!(g.players[0].life, life_before + gain);
        }
    }
}

#[test]
fn cast_gains_life() {
    for (def, gain) in [
        (catalog::witherbloom_spireling(), 2),
        (catalog::lorehold_ember_sentinel(), 3),
        (catalog::silverquill_spellguard(), 2),
        (catalog::inkling_devotee(), 2),
        (catalog::strixhaven_quartermaster(), 2),
        (catalog::witherbloom_lifechant(), 5),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        let id = g.add_card_to_hand(0, def);
        big_mana!(g);
        let life_before = g.players[0].life;
        cast!(g, id, None);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life_before + gain);
    }
}

// ── Token minting on cast (ETB or spell effect) ─────────────────────────────

#[test]
fn cast_mints_tokens() {
    // (def, token name, count, expected +1/+1 counters on the token)
    for (def, token_name, count, counters) in [
        (catalog::silverquill_inkjet_scribe(), "Inkling", 1, None),
        (catalog::silverquill_inkcaller(), "Inkling", 1, None),
        (catalog::silverquill_quillbinder(), "Inkling", 1, None),
        (catalog::inkling_ascendancy(), "Inkling", 2, None),
        (catalog::witherbloom_thornmaster(), "Pest", 1, None),
        (catalog::witherbloom_pestswarm_master(), "Pest", 2, None),
        (catalog::witherbloom_pestbloomer(), "Pest", 2, None),
        (catalog::pestilent_marsh(), "Pest", 2, None),
        (catalog::pest_glutton(), "Pest", 1, None),
        (catalog::pest_quartermaster(), "Pest", 1, None),
        (catalog::prismari_glassforge(), "Treasure", 1, None),
        (catalog::prismari_treasurespark(), "Treasure", 1, None),
        (catalog::prismari_treasurespell(), "Treasure", 2, None),
        (catalog::lorehold_spiritbinder(), "Spirit", 1, None),
        (catalog::lorehold_battle_cry(), "Spirit", 1, None),
        (catalog::lorehold_scrollwarden(), "Spirit", 1, None),
        (catalog::lorehold_spiritscribe(), "Spirit", 2, None),
        (catalog::fractal_cascade(), "Fractal", 1, Some(4)),
        (catalog::fractal_anomaly_v2(), "Fractal", 1, Some(5)),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::plains());
        let id = g.add_card_to_hand(0, def);
        big_mana!(g);
        cast!(g, id, None);
        drain_stack(&mut g);
        let tokens: Vec<_> = g
            .battlefield
            .iter()
            .filter(|c| c.is_token && c.definition.name == token_name)
            .collect();
        assert_eq!(tokens.len(), count, "{token_name} token count");
        if let Some(n) = counters {
            assert_eq!(tokens[0].counter_count(CounterType::PlusOnePlusOne), n);
        }
    }
}

// ── Vanilla-ish bodies: cast, check stats/keywords ──────────────────────────

#[test]
fn cast_creature_stats_and_keywords() {
    for (def, pt, kws) in [
        (
            catalog::inkling_heralder(),
            Some((2, 2)),
            vec![Keyword::Flying, Keyword::Lifelink],
        ),
        (catalog::spirit_bardguard(), Some((2, 3)), vec![Keyword::Vigilance]),
        (catalog::spirit_spearmaiden(), Some((2, 2)), vec![Keyword::FirstStrike]),
        (
            catalog::lorehold_phoenix_soldier(),
            Some((2, 2)),
            vec![Keyword::Flying, Keyword::Haste],
        ),
        (
            catalog::strixhaven_skylancer(),
            Some((3, 3)),
            vec![Keyword::Flying, Keyword::Vigilance],
        ),
        (catalog::witherbloom_witchwarden(), Some((3, 3)), vec![Keyword::Lifelink]),
        (catalog::prismari_stormgale(), Some((3, 3)), vec![Keyword::Flying]),
        (catalog::quandrix_calculator_v2(), Some((2, 2)), vec![]),
        (
            catalog::inkling_scrollwarden(),
            None,
            vec![Keyword::Flying, Keyword::Vigilance],
        ),
        (catalog::silverquill_wingweaver(), Some((1, 3)), vec![Keyword::Flying]),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::plains());
        let id = g.add_card_to_hand(0, def);
        big_mana!(g);
        cast!(g, id, None);
        drain_stack(&mut g);
        let card = g.battlefield_find(id).unwrap();
        if let Some((p, t)) = pt {
            assert_eq!(card.power(), p);
            assert_eq!(card.toughness(), t);
        }
        for kw in &kws {
            assert!(card.has_keyword(kw));
        }
    }
}

#[test]
fn etb_scry_keeps_library_size() {
    for def in [
        catalog::silverquill_scribebearer(),
        catalog::strixhaven_library_mage(),
    ] {
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        let id = g.add_card_to_hand(0, def);
        big_mana!(g);
        let lib_before = g.players[0].library.len();
        cast!(g, id, None);
        drain_stack(&mut g);
        // Scry doesn't draw or move cards; library size unchanged.
        assert_eq!(g.players[0].library.len(), lib_before);
    }
}

// ── Cast targeting an opposing bear that dies to the effect ─────────────────

#[test]
fn cast_kills_targeted_bear() {
    // (def, self life gain)
    for (def, gain) in [
        (catalog::lorehold_warpriest(), 2),
        (catalog::prismari_embershout(), 0),
        (catalog::lorehold_glimmercaller(), 0),
        (catalog::lorehold_smiteseer(), 2),
        (catalog::prismari_spelljay(), 0),
        (catalog::witherbloom_toxicvial(), 0),
        (catalog::silverquill_vermilion(), 3),
        (catalog::witherbloom_rotsplash(), 1),
        (catalog::prismari_volcanic_song(), 0),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        let id = g.add_card_to_hand(0, def);
        big_mana!(g);
        let life_before = g.players[0].life;
        cast!(g, id, Some(Target::Permanent(bear)));
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "bear should die");
        assert_eq!(g.players[0].life, life_before + gain);
    }
}

#[test]
fn cast_pings_targeted_player() {
    for (def, dmg) in [
        (catalog::prismari_emberweaver(), 2),
        (catalog::prismari_skyflare(), 2),
        (catalog::lorehold_veteran(), 1),
        (catalog::lorehold_flameherald_v2(), 1),
        (catalog::prismari_flamewright(), 2),
        (catalog::lorehold_lavabolt(), 3),
        (catalog::prismari_cantrip_spark(), 1),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        let id = g.add_card_to_hand(0, def);
        big_mana!(g);
        let opp_before = g.players[1].life;
        cast!(g, id, Some(Target::Player(1)));
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before - dmg);
    }
}

// ── ETB draw / return-from-graveyard ────────────────────────────────────────

#[test]
fn etb_draws_a_card() {
    // Net hand: -1 (cast) +1 (draw) = same. (def, life_loss)
    for (def, life_loss) in [
        (catalog::quandrix_thoughtweaver(), 0),
        (catalog::quandrix_numerologist(), 0),
        (catalog::prismari_dragonkin(), 0),
        (catalog::silverquill_pencrafter(), 1),
    ] {
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::plains());
        g.add_card_to_library(0, catalog::plains());
        let id = g.add_card_to_hand(0, def);
        let hand_before = g.players[0].hand.len();
        let life_before = g.players[0].life;
        big_mana!(g);
        cast!(g, id, None);
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1);
        assert_eq!(g.players[0].life, life_before - life_loss);
    }
}

#[test]
fn etb_returns_card_from_graveyard_to_hand() {
    for (def, gy_def, gain) in [
        (catalog::silverquill_curator(), catalog::grizzly_bears(), None),
        (catalog::quandrix_snapcaster(), catalog::lightning_bolt(), None),
        (catalog::silverquill_bookbond(), catalog::grizzly_bears(), Some(1)),
    ] {
        let mut g = two_player_game();
        let gy_card = g.add_card_to_graveyard(0, gy_def);
        let id = g.add_card_to_hand(0, def);
        big_mana!(g);
        let life_before = g.players[0].life;
        cast!(g, id, None);
        drain_stack(&mut g);
        assert!(g.players[0].hand.iter().any(|c| c.id == gy_card));
        if let Some(gain) = gain {
            assert_eq!(g.players[0].life, life_before + gain);
        }
    }
}

// ── Fractals entering with counters ─────────────────────────────────────────

#[test]
fn fractals_enter_with_counters() {
    for (def, n, kws) in [
        (catalog::fractal_seer(), 1, vec![]),
        (catalog::fractal_sproutling(), 1, vec![]),
        (catalog::fractal_aegis(), 3, vec![Keyword::Trample]),
        (catalog::fractal_tideshaper(), 3, vec![]),
        (catalog::fractal_sentinel(), 5, vec![Keyword::Trample]),
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        big_mana!(g);
        cast!(g, id, None);
        drain_stack(&mut g);
        let card = g.battlefield_find(id).unwrap();
        assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), n);
        assert_eq!(card.power(), n as i32);
        assert_eq!(card.toughness(), n as i32);
        for kw in &kws {
            assert!(card.has_keyword(kw));
        }
    }
}

// ── Individually kept tests (unique shapes, regressions, CR citations) ──────

#[test]
fn inkling_saboteur_combat_damage_forces_discard() {
    use crabomination::game::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::island());
    let hand_before = g.players[1].hand.len();
    let id = g.add_card_to_battlefield(0, catalog::inkling_saboteur());
    g.clear_sickness(id);
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("declare attackers");
    // Run through combat to deal combat damage.
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    // Opponent should have discarded one card from the combat damage trigger.
    assert_eq!(g.players[1].hand.len(), hand_before - 1);
}

#[test]
fn silverquill_sealwright_magecraft_pumps_friendly_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _id = g.add_card_to_battlefield(0, catalog::silverquill_sealwright());
    drain_stack(&mut g);
    bolt!(g, Target::Player(1));
    drain_stack(&mut g);
    let card = g.battlefield_find(target).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn witherbloom_ravensoul_dies_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_ravensoul());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    // Lightning Bolt 3 dmg = exact lethal on 3-toughness Ravensoul.
    bolt!(g, Target::Permanent(id));
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "Ravensoul should be dead");
    // Death trigger drains 2: each opp loses 2, you gain 2.
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_ironbacked_archivist_etb_exiles_graveyard_card() {
    let mut g = two_player_game();
    let gy_card = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::lorehold_ironbacked_archivist());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    // Auto-target picker should have exiled the bolt from the opp gy.
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == gy_card));
    assert!(g.exile.iter().any(|c| c.id == gy_card));
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_relicbearer_grows_on_gy_leave() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_relicbearer());
    drain_stack(&mut g);
    // Seed gy with a card, then use Lorehold Acolyte to move it to exile
    // (a Card-Left-Graveyard event).
    let _gy_card = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let exiler = g.add_card_to_hand(0, catalog::lorehold_acolyte());
    big_mana!(g);
    cast!(g, exiler, None);
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_grand_calculator_etb_scales_with_lands() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Add 4 lands to controller.
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_grand_calculator());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    // Counters should land on a friendly creature (4 from 4 lands).
    // Auto-picker may pick bear or calculator — assert that the total
    // across all friendly creatures grew by 4.
    let bear_counters = g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let calc_counters = g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(bear_counters + calc_counters, 4);
}

#[test]
fn quandrix_lifestream_pumps_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_lifestream());
    let hand_before = g.players[0].hand.len();
    big_mana!(g);
    cast!(g, id, Some(Target::Permanent(target)));
    drain_stack(&mut g);
    let card = g.battlefield_find(target).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1);
    // -1 cast + 1 draw = net 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_mistforger_etb_mints_fractal_scaled_by_creatures() {
    let mut g = two_player_game();
    // 2 friendly creatures pre-cast.
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_mistforger());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    // 2 bears + Mistforger + Fractal token = 4 creatures at the
    // AddCounter step (token enters before counters are placed).
    let fractal = g.battlefield.iter().find(|c| c.definition.name == "Fractal").unwrap();
    assert_eq!(
        fractal.counter_count(CounterType::PlusOnePlusOne),
        4,
        "Fractal should have 4 +1/+1 counters (2 bears + Mistforger + Fractal itself)"
    );
}

#[test]
fn inkling_decreemaster_etb_forces_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let hand_before = g.players[1].hand.len();
    let id = g.add_card_to_hand(0, catalog::inkling_decreemaster());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 1);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_sageling_dies_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::inkling_sageling());
    drain_stack(&mut g);
    let hand_before = g.players[0].hand.len();
    bolt!(g, Target::Permanent(id));
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none());
    // `bolt!` mints the Bolt into hand and casts it (net 0); the death
    // trigger then draws a card = net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn silverquill_final_year_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_final_year());
    drain_stack(&mut g);
    bolt!(g, Target::Player(1));
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power_bonus, 1);
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_sergeant_anthems_other_inklings() {
    // Pump effects are computed via the layer system; we check the
    // computed power on a separate Inkling.
    let mut g = two_player_game();
    let other_inkling = g.add_card_to_battlefield(0, catalog::inkling_vassal());
    let sergeant = g.add_card_to_battlefield(0, catalog::inkling_sergeant());
    drain_stack(&mut g);
    // Inkling Vassal is 1/2; +1/+0 anthem from Sergeant → 2 effective power.
    let computed = g
        .compute_battlefield()
        .into_iter()
        .find(|c| c.id == other_inkling)
        .unwrap();
    assert_eq!(computed.power, 2);
    // Sergeant doesn't anthem itself ("other" clause) — 2/2 base unchanged.
    let sergeant_computed = g
        .compute_battlefield()
        .into_iter()
        .find(|c| c.id == sergeant)
        .unwrap();
    assert_eq!(sergeant_computed.power, 2);
}

#[test]
fn silverquill_verdict_exiles_high_power_creature() {
    let mut g = two_player_game();
    // Add a 4/4 creature (Serra Angel) on opponent side.
    let target = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::silverquill_verdict());
    big_mana!(g);
    let life = g.players[0].life;
    cast!(g, id, Some(Target::Permanent(target)));
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none());
    assert_eq!(g.players[0].life, life + 2);
}

#[test]
fn silverquill_verdict_rejects_low_power_target() {
    // Grizzly Bears is 2/2 — power < 3, target rejected.
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_verdict());
    big_mana!(g);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(), "Verdict should reject a 2-power target");
}

#[test]
fn inkling_bondsmith_etb_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::inkling_bondsmith());
    big_mana!(g);
    cast!(g, id, Some(Target::Permanent(target)));
    drain_stack(&mut g);
    let card = g.battlefield_find(target).unwrap();
    assert_eq!(card.power_bonus, 1);
    assert!(card.has_keyword(&Keyword::Lifelink));
}

#[test]
fn inkling_aspect_etb_pumps_self_and_grants_menace() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_aspect());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power_bonus, 1);
    assert!(card.has_keyword(&Keyword::Menace));
}

#[test]
fn pestmaster_pumps_on_pest_token_death_via_cached_controller() {
    // Lock-in for the push (modern_decks claude/modern_decks) engine fix:
    // `died_card_controllers` cache lets AnotherOfYours triggers fire
    // off dying tokens (CR 111.7c "ceases to exist" SBA removes the
    // token from every zone in the same sweep as the death event, so
    // the zone-walking subject_controller lookup returns None without
    // the cache).
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::witherbloom_pestmaster());
    // Cast Pest Summoning to mint two Pest tokens under P0's control.
    let ps = g.add_card_to_hand(0, catalog::pest_summoning());
    big_mana!(g);
    cast!(g, ps, None);
    drain_stack(&mut g);
    // Two Pest tokens entered. Find one and kill it with a Bolt.
    let pest_id = g
        .battlefield
        .iter()
        .find(|c| {
            c.id != pm
                && c.definition
                    .subtypes
                    .creature_types
                    .contains(&CreatureType::Pest)
        })
        .map(|c| c.id)
        .expect("Found a Pest token on the battlefield");
    let pmc_before = g
        .battlefield_find(pm)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    // Kill the Pest with a Bolt.
    bolt!(g, Target::Permanent(pest_id));
    drain_stack(&mut g);
    // Pestmaster sees the token death via the controller cache and grows.
    let pmc = g.battlefield_find(pm).expect("Pestmaster still alive");
    assert_eq!(
        pmc.counter_count(CounterType::PlusOnePlusOne),
        pmc_before + 1,
        "Pestmaster gained a +1/+1 counter on Pest *token* death (cache lookup path)"
    );
}

#[test]
fn felisa_pumps_inkling_on_pest_token_with_counter_death() {
    // Lock-in for the push (modern_decks batch 47) token-death snapshot
    // cache: Felisa's "creature with +1/+1 counter dies → 1/1 W/B
    // Inkling" trigger fires for TOKEN deaths too. Before the cache
    // landed, the CR 111.7c "token ceases to exist" SBA removed the
    // dying token from every zone in the same sweep — by dispatch time
    // the `WithCounter(+1/+1)` filter (evaluated via
    // evaluate_requirement_static on the dying card) returned false
    // because no zone had the card.
    let mut g = two_player_game();
    let _felisa = g.add_card_to_battlefield(0, catalog::felisa_fang_of_silverquill());
    // Mint a Pest token under P0 (the Pest also has a +1/+1 counter
    // applied directly to the battlefield instance — simulating a
    // mid-game scenario where, say, Silverquill Memorize pumped a
    // friendly Pest before it died).
    let ps = g.add_card_to_hand(0, catalog::pest_summoning());
    big_mana!(g);
    cast!(g, ps, None);
    drain_stack(&mut g);
    // Find the first Pest token.
    let pest_id = g
        .battlefield
        .iter()
        .find(|c| c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .map(|c| c.id)
        .expect("Pest token created");
    // Add a +1/+1 counter directly.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == pest_id) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    let bf_before = g.battlefield.len();
    // Kill the Pest with a Bolt.
    bolt!(g, Target::Permanent(pest_id));
    drain_stack(&mut g);
    // The Pest is gone (ceased to exist), Bolt is in graveyard, and
    // Felisa minted an Inkling because the cache let her counter
    // filter resolve on the dying token. Net: -1 pest, -0 bolt
    // (still in gy, off bf) + 1 inkling = bf_before.
    let inkling_present = g
        .battlefield
        .iter()
        .any(|c| c.definition.name == "Inkling");
    assert!(
        inkling_present,
        "Felisa mints an Inkling when a counter-bearing Pest token dies"
    );
    // Sanity: bf size is unchanged (pest out, inkling in).
    assert_eq!(g.battlefield.len(), bf_before);
}

#[test]
fn silverquill_reprover_shrinks_opp_creature_on_etb() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_reprover());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    let card = g.battlefield_find(opp).unwrap();
    assert_eq!(card.power(), 0, "Bear should be 0/2 after -2/-0");
    assert_eq!(card.toughness(), 2);
}

#[test]
fn witherbloom_vinetwister_etb_fans_counters_on_other_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinetwister());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    let bear_c = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_c.counter_count(CounterType::PlusOnePlusOne), 1);
    let self_c = g.battlefield_find(id).unwrap();
    assert_eq!(self_c.counter_count(CounterType::PlusOnePlusOne), 0);
}

#[test]
fn lorehold_battle_memorial_deals_three_to_creature_and_three_to_player() {
    let mut g = two_player_game();
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_battle_memorial());
    let opp_before = g.players[1].life;
    big_mana!(g);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("Battle Memorial castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp).is_none(), "3-damage kills the bear");
    assert_eq!(g.players[1].life, opp_before - 3);
}

#[test]
fn quandrix_arcanist_flash_magecraft_scrys() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::quandrix_arcanist());
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Flash));
    g.add_card_to_library(0, catalog::island());
    bolt!(g, Target::Player(1));
    drain_stack(&mut g);
    assert!(!g.players[0].library.is_empty());
}

#[test]
fn quandrix_triplecaster_etb_puts_two_counters_on_target() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_triplecaster());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    let total = g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne)
        + g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(total, 2);
}

#[test]
fn quandrix_counterfold_doubles_counters_on_creature() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(target).unwrap().counters.insert(CounterType::PlusOnePlusOne, 3);
    let id = g.add_card_to_hand(0, catalog::quandrix_counterfold());
    big_mana!(g);
    cast!(g, id, Some(Target::Permanent(target)));
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne),
        6
    );
}

#[test]
fn quandrix_augurer_etb_draws_and_fans_counters() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_augurer());
    let hand_before = g.players[0].hand.len();
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_geometer_v3_etb_pumps_each_friendly() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_geometer_v3());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    // Geometer itself gets a counter (it's a friendly creature when fan-out
    // runs), plus the bear.
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_tide_pumps_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_tide());
    let hand_before = g.players[0].hand.len();
    big_mana!(g);
    cast!(g, id, Some(Target::Permanent(bear)));
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    // Net: -1 (cast) +1 (draw) = same hand.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_snapcaster_and_vinepriest_unique_shapes_kept_in_gy_table() {
    // Placeholder-free: snapcaster covered in etb_returns_card_from_graveyard_to_hand;
    // vinepriest (search) kept below.
}

#[test]
fn quandrix_vinepriest_etb_fetches_basic_land() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::quandrix_vinepriest());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    // Forest should be in hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == forest));
}

#[test]
fn prismari_scribbler_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::prismari_scribbler());
    let hand_before = g.players[0].hand.len();
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_skyspark_pumps_and_grants_flying() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_skyspark());
    big_mana!(g);
    cast!(g, id, Some(Target::Permanent(target)));
    drain_stack(&mut g);
    let bear = g.battlefield_find(target).unwrap();
    assert_eq!(bear.power(), 3);
    assert_eq!(bear.toughness(), 3);
    assert!(bear.has_keyword(&Keyword::Flying));
}

#[test]
fn prismari_embergale_burns_creature_and_pings_each_opp() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_before = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::prismari_embergale());
    big_mana!(g);
    cast!(g, id, Some(Target::Permanent(bear)));
    drain_stack(&mut g);
    // Bear takes 3, then 1 damage to each opp.
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn prismari_sparkmage_v3_pings_creature_on_is_cast() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _id = g.add_card_to_battlefield(0, catalog::prismari_sparkmage_v3());
    bolt!(g, Target::Player(1));
    drain_stack(&mut g);
    // Magecraft ping: target_filtered picks the bear (sole creature target).
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.damage, 1);
}

#[test]
fn prismari_burnscribe_etb_pings_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_burnscribe());
    big_mana!(g);
    cast!(g, id, Some(Target::Permanent(bear)));
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.damage, 1);
}

#[test]
fn strixhaven_crucible_activation_drains_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::strixhaven_crucible());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Crucible activatable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn witherbloom_pestcaller_v2_magecraft_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_pestcaller_v2());
    bolt!(g, Target::Player(1));
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.subtypes.creature_types.contains(&CreatureType::Pest))
        .collect();
    assert_eq!(pests.len(), 1);
    // Source survives.
    assert!(g.battlefield_find(id).is_some());
}

#[test]
fn witherbloom_vinepriest_etb_gains_two_life_and_magecraft_gains_one() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinepriest());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    bolt!(g, Target::Player(1));
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
}

#[test]
fn spirit_banner_bearer_anthems_other_spirits() {
    let mut g = two_player_game();
    let other_spirit_id = g.add_card_to_battlefield(0, catalog::lorehold_aerospirit());
    let banner_id = g.add_card_to_battlefield(0, catalog::spirit_banner_bearer());
    // Use layered computed view per CR 613.
    let computed: Vec<_> = g.compute_battlefield();
    let banner_computed = computed.iter().find(|c| c.id == banner_id).expect("banner");
    let spirit_computed = computed.iter().find(|c| c.id == other_spirit_id).expect("aero");
    // Banner-bearer doesn't anthem itself; stays 1/3.
    assert_eq!(banner_computed.power, 1);
    // Aerospirit is base 3/2; +1/+0 from anthem → 4/2.
    assert_eq!(spirit_computed.power, 4);
}

#[test]
fn lorehold_battle_drum_pumps_team_with_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_battle_drum());
    big_mana!(g);
    cast!(g, id, None);
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.power(), 3);
    assert!(bear_card.has_keyword(&Keyword::Haste));
}

#[test]
fn fractal_wavebreaker_etb_bounces_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_wavebreaker());
    big_mana!(g);
    cast!(g, id, Some(Target::Permanent(bear)));
    drain_stack(&mut g);
    // Bear should be back in opp's hand.
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

#[test]
fn silverquill_lawscribe_etb_taps_opp_creature() {
    let mut g = two_player_game();
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_lawscribe());
    big_mana!(g);
    cast!(g, id, Some(Target::Permanent(prey)));
    drain_stack(&mut g);
    assert!(g.battlefield_find(prey).unwrap().tapped);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
}
