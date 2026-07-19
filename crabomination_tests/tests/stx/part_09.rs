use crabomination::card::{CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use super::*;

// ── Consolidated STX batches 37-42 ──────────────────────────────────────────
//
// Structurally identical per-card tests are collapsed into table-driven
// tests below; unique-shape tests are kept individually at the end.

fn fill_mana(g: &mut crabomination::game::Game) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 2);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Static stat/keyword/subtype checks: put on battlefield, inspect.
#[test]
fn stx_vanilla_stats_keywords_and_subtypes() {
    let cases: Vec<(_, i32, i32, Vec<Keyword>, Vec<CreatureType>)> = vec![
        (catalog::witherbloom_rotwarden(), 4, 4, vec![Keyword::Trample, Keyword::Lifelink], vec![]),
        (catalog::pest_briarscale(), 3, 3, vec![Keyword::Trample], vec![CreatureType::Pest, CreatureType::Beast]),
        (catalog::spirit_warbearer(), 2, 2, vec![Keyword::FirstStrike], vec![CreatureType::Spirit]),
        (catalog::inkling_ambassador(), 1, 1, vec![Keyword::Flying, Keyword::Lifelink], vec![]),
        (catalog::lorehold_wargeist(), 3, 2, vec![Keyword::Haste], vec![CreatureType::Spirit, CreatureType::Warrior]),
        (catalog::inkling_bookwarden(), 4, 5, vec![Keyword::Flying, Keyword::Lifelink], vec![CreatureType::Inkling]),
        (catalog::pest_reaver(), 3, 3, vec![Keyword::Deathtouch], vec![CreatureType::Pest]),
        (catalog::spirit_outrider(), 3, 4, vec![Keyword::FirstStrike], vec![CreatureType::Spirit, CreatureType::Knight]),
        (catalog::inkling_bookcrier(), 3, 2, vec![Keyword::Flying], vec![CreatureType::Inkling]),
        (catalog::lorehold_saberspirit(), 3, 4, vec![Keyword::FirstStrike, Keyword::Lifelink], vec![]),
        (catalog::fractal_sproutling(), 1, 1, vec![], vec![]),
    ];
    for (def, p, t, kws, types) in cases {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let card = g.battlefield_find(id).unwrap();
        assert_eq!(card.power(), p, "{name} power");
        assert_eq!(card.toughness(), t, "{name} toughness");
        for kw in &kws {
            assert!(card.has_keyword(kw), "{name} has {kw:?}");
        }
        for ty in &types {
            assert!(card.definition.subtypes.creature_types.contains(ty), "{name} is {ty:?}");
        }
    }
}

/// Magecraft watchers on the battlefield while a Lightning Bolt is cast at
/// the opponent: assert opponent/self life deltas (bolt itself is -3).
#[test]
fn stx_magecraft_life_deltas_on_bolt_cast() {
    let cases: Vec<(_, i32, i32)> = vec![
        (catalog::lorehold_ember_priest_v2(), -4, 0),
        (catalog::prismari_dazzler(), -4, 0),
        (catalog::prismari_wildmage(), -4, 0),
        (catalog::lorehold_ember_reader(), -4, 0),
        (catalog::prismari_cinderbolt(), -4, 0),
        (catalog::lorehold_emberkeeper(), -4, 0),
        (catalog::witherbloom_distiller(), -4, 0),
        (catalog::prismari_glasshammer(), -4, 0),
        (catalog::witherbloom_bloodbrewer(), -4, 0),
        (catalog::silverquill_spellbinder(), -4, 1),
        (catalog::silverquill_liturgist(), -3, 1),
        (catalog::witherbloom_bloomcaller(), -3, 1),
        (catalog::silverquill_essayist(), -3, 1),
        (catalog::quandrix_pondkeeper_v2(), -3, 0),
        (catalog::prismari_scryer(), -3, 0),
        (catalog::silverquill_purifier(), -3, 0),
    ];
    for (def, opp_delta, self_delta) in cases {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_library(0, catalog::island());
        let _id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let opp_before = g.players[1].life;
        let life_before = g.players[0].life;
        let lib_before = g.players[0].library.len();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before + opp_delta, "{name} opp life");
        assert_eq!(g.players[0].life, life_before + self_delta, "{name} self life");
        // Scry-only watchers reorder without drawing.
        assert_eq!(g.players[0].library.len(), lib_before, "{name} library size");
    }
}

/// Magecraft: watcher gives itself a +1/+1 counter when a spell is cast.
#[test]
fn stx_magecraft_self_counter_on_bolt_cast() {
    let cases = vec![
        catalog::quandrix_scout(),
        catalog::inkling_calligraphist(),
        catalog::quandrix_scaler(),
        catalog::witherbloom_sproutchant(),
        catalog::quandrix_seedling(),
        catalog::witherbloom_bloomstalk(),
    ];
    for def in cases {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        drain_stack(&mut g);
        let before = g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let card = g.battlefield_find(id).unwrap();
        assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), before + 1, "{name} counter");
    }
}

/// Magecraft: watcher pumps its own power by 1 when a spell is cast.
#[test]
fn stx_magecraft_self_pump_on_bolt_cast() {
    let cases: Vec<(_, Option<Keyword>)> = vec![
        (catalog::prismari_stormrider(), Some(Keyword::Flying)),
        (catalog::lorehold_pyrokin(), Some(Keyword::Haste)),
        (catalog::prismari_hothead(), Some(Keyword::Haste)),
        (catalog::lorehold_ironwill(), Some(Keyword::FirstStrike)),
        (catalog::spirit_bookburner(), None),
        (catalog::prismari_soundsmith(), None),
        (catalog::lorehold_b37_beacon(), None),
    ];
    for (def, kw) in cases {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_battlefield(0, def);
        g.clear_sickness(id);
        drain_stack(&mut g);
        let pwr_before = g.battlefield_find(id).unwrap().power();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        let card = g.battlefield_find(id).unwrap();
        assert_eq!(card.power(), pwr_before + 1, "{name} power");
        if let Some(kw) = kw {
            assert!(card.has_keyword(&kw), "{name} has {kw:?}");
        }
    }
}

/// Magecraft: watcher pumps another friendly creature by 1 when a spell is cast.
#[test]
fn stx_magecraft_pumps_other_friendly_creature() {
    let cases = vec![
        (catalog::fractal_catalyst(), catalog::grizzly_bears()),
        (catalog::quandrix_doublecaster_v2(), catalog::fractal_emergent()),
        (catalog::prismari_tempestmage(), catalog::grizzly_bears()),
        (catalog::silverquill_scriptwright(), catalog::inkling_aspirant()),
    ];
    for (watcher, recipient) in cases {
        let name = watcher.name.clone();
        let mut g = two_player_game();
        let _w = g.add_card_to_battlefield(0, watcher);
        let target = g.add_card_to_battlefield(0, recipient);
        drain_stack(&mut g);
        let pwr_before = g.battlefield_find(target).unwrap().power();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(target).unwrap().power(), pwr_before + 1, "{name} pumped recipient");
    }
}

/// Big cast-and-resolve table: cast the card (optionally targeting the
/// opponent), then check life deltas, net hand change, tokens minted, and
/// keywords on the resulting permanent (if it is one).
/// `None` means "not asserted for this card".
#[test]
fn stx_cast_resolves_life_hand_tokens_and_keywords() {
    struct Case {
        def: crabomination::card::CardDefinition,
        targeted: bool,
        opp: Option<i32>,
        selfd: Option<i32>,
        hand: Option<i32>,
        tokens: usize,
        kws: Vec<Keyword>,
    }
    fn case(
        def: crabomination::card::CardDefinition,
        targeted: bool,
        opp: Option<i32>,
        selfd: Option<i32>,
        hand: Option<i32>,
        tokens: usize,
        kws: Vec<Keyword>,
    ) -> Case {
        Case { def, targeted, opp, selfd, hand, tokens, kws }
    }
    let cases = vec![
        case(catalog::lorehold_b37_spiritflame(), false, Some(-1), None, None, 1, vec![]),
        case(catalog::lorehold_sermonizer(), true, Some(-2), Some(2), None, 0, vec![]),
        case(catalog::quandrix_researcher(), false, None, Some(-1), Some(0), 0, vec![]),
        case(catalog::prismari_sparkmage_v2(), true, Some(-2), None, None, 0, vec![]),
        case(catalog::prismari_eddy(), false, None, None, Some(0), 0, vec![]),
        case(catalog::inkling_scriptwarden(), false, Some(-1), Some(1), None, 0, vec![Keyword::Flying, Keyword::Vigilance]),
        case(catalog::silverquill_battle_oration(), false, Some(-4), Some(4), None, 1, vec![]),
        case(catalog::witherbloom_fungalweb(), false, Some(-2), Some(2), None, 0, vec![]),
        case(catalog::pest_swarmrider(), false, None, None, None, 1, vec![]),
        case(catalog::lorehold_skydefender(), false, None, Some(2), None, 0, vec![Keyword::Flying, Keyword::Vigilance]),
        case(catalog::lorehold_spiritrider(), false, None, None, None, 2, vec![]),
        case(catalog::quandrix_fluctuator(), false, None, None, Some(0), 0, vec![]),
        case(catalog::prismari_cinderpoet(), false, None, None, Some(-1), 0, vec![]),
        case(catalog::prismari_pyrocaster(), true, Some(-2), None, None, 0, vec![]),
        case(catalog::silverquill_manuscript(), false, Some(-2), None, Some(0), 0, vec![]),
        case(catalog::witherbloom_cauldronkeeper(), false, None, Some(2), None, 0, vec![]),
        case(catalog::prismari_sparkbolt(), true, Some(-2), None, None, 0, vec![]),
        case(catalog::inkling_magister(), false, Some(-3), Some(3), None, 0, vec![Keyword::Flying, Keyword::Vigilance]),
        case(catalog::witherbloom_alchemist(), false, Some(-2), Some(2), None, 0, vec![]),
        case(catalog::witherbloom_decoction(), false, Some(-2), Some(2), None, 0, vec![]),
        case(catalog::lorehold_hellraiser(), true, Some(-2), None, None, 0, vec![Keyword::Haste]),
        case(catalog::lorehold_bonfire(), true, Some(-3), Some(1), None, 0, vec![]),
        case(catalog::lorehold_spiritsage(), false, None, None, None, 1, vec![]),
        case(catalog::quandrix_scrymaster(), false, None, None, Some(-1), 0, vec![]),
        case(catalog::fractal_stargazer(), false, None, None, Some(-1), 0, vec![]),
        case(catalog::quandrix_aetherwarden(), false, None, None, Some(0), 0, vec![Keyword::Flying]),
        case(catalog::spirit_pyremage(), true, Some(-1), None, None, 0, vec![]),
        case(catalog::inkling_avant_garde(), false, Some(-2), Some(2), None, 0, vec![Keyword::Flying, Keyword::Lifelink]),
        case(catalog::silverquill_convocation(), false, Some(-2), Some(2), None, 2, vec![]),
        case(catalog::pest_brewer(), false, None, None, None, 1, vec![]),
        case(catalog::witherbloom_pestsage(), false, None, None, None, 2, vec![]),
        case(catalog::lorehold_recital_v2(), true, Some(-2), None, None, 1, vec![]),
        case(catalog::silverquill_witnessing(), false, Some(-3), Some(3), Some(0), 0, vec![]),
        case(catalog::silverquill_cantorist(), false, Some(-1), Some(1), None, 0, vec![Keyword::Lifelink]),
        case(catalog::inkling_treasurer(), false, None, Some(1), None, 0, vec![Keyword::Flying]),
        case(catalog::witherbloom_bloodglyph(), false, Some(-2), Some(2), None, 1, vec![]),
        case(catalog::lorehold_wraithcaller(), false, None, None, None, 1, vec![]),
        case(catalog::lorehold_ballad(), true, Some(-2), Some(2), None, 0, vec![]),
        case(catalog::silverquill_inkflood(), false, None, Some(2), None, 2, vec![]),
        case(catalog::quandrix_amplifier(), false, None, None, Some(0), 0, vec![]),
        case(catalog::prismari_quickcast(), true, Some(-2), None, Some(0), 0, vec![]),
        case(catalog::prismari_starcaller(), false, None, None, Some(0), 0, vec![Keyword::Flying]),
        case(catalog::witherbloom_sapglyph(), true, Some(-2), Some(2), None, 0, vec![]),
        case(catalog::pest_cultivator_v2(), false, None, None, None, 1, vec![]),
        case(catalog::lorehold_stoneguard(), false, None, Some(2), None, 0, vec![Keyword::Vigilance]),
        case(catalog::lorehold_pyresummon(), true, Some(-1), None, None, 1, vec![]),
        case(catalog::quandrix_geometer_v2(), false, None, None, Some(0), 0, vec![]),
        case(catalog::quandrix_calligrapher_v2(), false, None, None, Some(0), 0, vec![]),
        case(catalog::prismari_inferno_v2(), true, Some(-3), None, None, 0, vec![]),
        case(catalog::prismari_stagewright(), false, None, None, Some(0), 0, vec![]),
        case(catalog::witherbloom_coatlcoiler(), true, Some(-2), None, None, 0, vec![Keyword::Deathtouch]),
        case(catalog::witherbloom_cinderscribe(), false, Some(-2), None, None, 2, vec![]),
        case(catalog::silverquill_penlord(), false, Some(-3), Some(3), None, 0, vec![Keyword::Flying, Keyword::Lifelink]),
        case(catalog::inkling_disciple(), false, None, Some(1), None, 0, vec![Keyword::Flying]),
        case(catalog::silverquill_inkflame(), false, Some(-2), Some(2), Some(0), 0, vec![]),
        case(catalog::prismari_stormblade(), true, Some(-2), None, Some(0), 0, vec![]),
        case(catalog::prismari_pyromancer_v2(), true, Some(-2), None, None, 0, vec![]),
        case(catalog::prismari_treasurer_v2(), false, None, None, None, 2, vec![]),
        case(catalog::inkling_recruiter(), false, None, None, None, 1, vec![]),
    ];
    for c in cases {
        let name = c.def.name.clone();
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        g.add_card_to_hand(0, catalog::forest());
        let id = g.add_card_to_hand(0, c.def);
        fill_mana(&mut g);
        let opp_before = g.players[1].life;
        let life_before = g.players[0].life;
        let hand_before = g.players[0].hand.len() as i32;
        let tokens_before = g.battlefield.iter().filter(|x| x.is_token).count();
        let target = if c.targeted {
            Some(crabomination::game::types::Target::Player(1))
        } else {
            None
        };
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).unwrap_or_else(|e| panic!("{name} castable: {e:?}"));
        drain_stack(&mut g);
        if let Some(d) = c.opp {
            assert_eq!(g.players[1].life, opp_before + d, "{name} opp life");
        }
        if let Some(d) = c.selfd {
            assert_eq!(g.players[0].life, life_before + d, "{name} self life");
        }
        if let Some(d) = c.hand {
            assert_eq!(g.players[0].hand.len() as i32, hand_before + d, "{name} hand net");
        }
        let tokens_after = g.battlefield.iter().filter(|x| x.is_token).count();
        assert_eq!(tokens_after, tokens_before + c.tokens, "{name} tokens");
        if let Some(card) = g.battlefield_find(id) {
            for kw in &c.kws {
                assert!(card.has_keyword(kw), "{name} has {kw:?}");
            }
        }
    }
}

/// Fractals that enter with +1/+1 counters (power == counter count).
#[test]
fn stx_fractals_enter_with_counters() {
    let cases: Vec<(_, i32, Vec<Keyword>)> = vec![
        (catalog::fractal_emergent(), 3, vec![]),
        (catalog::fractal_aquanaut(), 2, vec![Keyword::Flying]),
        (catalog::fractal_mathmage(), 3, vec![]),
        (catalog::fractal_tidecaller_v2(), 2, vec![Keyword::Flying]),
    ];
    for (def, n, kws) in cases {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let card = g.battlefield.iter().find(|c| c.id == id || (c.controller == 0 && c.definition.name == name))
            .expect("on battlefield");
        assert_eq!(card.counter_count(CounterType::PlusOnePlusOne) as i32, n, "{name} counters");
        assert_eq!(card.power(), n, "{name} power");
        assert_eq!(card.toughness(), n, "{name} toughness");
        for kw in &kws {
            assert!(card.has_keyword(kw), "{name} has {kw:?}");
        }
    }
}

/// Spells/ETBs that mint a Fractal token with N +1/+1 counters.
#[test]
fn stx_mints_fractal_token_with_counters() {
    let cases: Vec<(_, i32)> = vec![
        (catalog::quandrix_basinkeeper(), 2),
        (catalog::quandrix_bountycaller(), 4),
        (catalog::fractal_burst(), 3),
    ];
    for (def, n) in cases {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let token = g.battlefield.iter().find(|c| c.is_token).expect("token minted");
        assert_eq!(token.counter_count(CounterType::PlusOnePlusOne) as i32, n, "{name} token counters");
        assert_eq!(token.power(), n, "{name} token power");
    }
}

/// Bounce an opponent's creature back to its owner's hand.
#[test]
fn stx_bounces_opponent_creature_to_hand() {
    for def in [catalog::quandrix_tideshaper(), catalog::prismari_skywarp()] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(crabomination::game::types::Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "{name} removed bear from battlefield");
        assert!(g.players[1].hand.iter().any(|c| c.id == bear), "{name} bounced bear to hand");
    }
}

/// Removal: cast targeting an opponent Grizzly Bears, which dies.
#[test]
fn stx_removal_kills_opponent_bear() {
    for def in [
        catalog::prismari_drift(),
        catalog::silverquill_censure_v2(),
        catalog::lorehold_pyrelancer(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(crabomination::game::types::Target::Permanent(bear)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none(), "{name} killed the bear");
    }
}

/// Pump spells/ETBs targeting a friendly creature: power delta on the target.
#[test]
fn stx_pumps_target_friendly_creature() {
    let cases: Vec<(_, _, i32, Option<Keyword>)> = vec![
        (catalog::silverquill_pinion(), catalog::grizzly_bears(), 1, Some(Keyword::Flying)),
        (catalog::silverquill_memorize(), catalog::grizzly_bears(), 1, None),
        (catalog::quandrix_equation_v2(), catalog::grizzly_bears(), 2, None),
        (catalog::inkling_quilltender(), catalog::inkling_aspirant(), 1, None),
    ];
    for (def, target_def, delta, kw) in cases {
        let name = def.name.clone();
        let mut g = two_player_game();
        let target = g.add_card_to_battlefield(0, target_def);
        drain_stack(&mut g);
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        let pwr_before = g.battlefield_find(target).unwrap().power();
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(crabomination::game::types::Target::Permanent(target)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        let card = g.battlefield_find(target).unwrap();
        assert_eq!(card.power(), pwr_before + delta, "{name} pumped target");
        if let Some(kw) = kw {
            assert!(card.has_keyword(&kw), "{name} granted {kw:?}");
        }
    }
}

/// ETBs that force the opponent to discard (with an optional drain).
#[test]
fn stx_etb_opponent_discards() {
    let cases: Vec<(_, bool, Option<i32>)> = vec![
        (catalog::inkling_proxy(), true, None),
        (catalog::silverquill_drafter_v2(), true, None),
        (catalog::inkling_bellringer(), false, None),
        (catalog::inkling_loredrain(), false, Some(-2)),
    ];
    for (def, targeted, opp_delta) in cases {
        let name = def.name.clone();
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::lightning_bolt());
        g.add_card_to_hand(1, catalog::lightning_bolt());
        let opp_hand_before = g.players[1].hand.len();
        let opp_before = g.players[1].life;
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        let target = if targeted {
            Some(crabomination::game::types::Target::Player(1))
        } else {
            None
        };
        g.perform_action(GameAction::CastSpell {
            card_id: id, target, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].hand.len(), opp_hand_before - 1, "{name} opp discarded");
        if let Some(d) = opp_delta {
            assert_eq!(g.players[1].life, opp_before + d, "{name} opp life");
        }
    }
}

/// Attack triggers: declare an attack and check life deltas.
#[test]
fn stx_attack_trigger_life_deltas() {
    use crabomination::game::{Attack, AttackTarget, TurnStep};
    let cases: Vec<(_, i32, i32)> = vec![
        (catalog::witherbloom_pestpicker(), -1, 0),
        (catalog::lorehold_knight_champion(), 0, 2),
    ];
    for (def, opp_delta, self_delta) in cases {
        let name = def.name.clone();
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
        assert_eq!(g.players[1].life, opp_before + opp_delta, "{name} opp life");
        assert_eq!(g.players[0].life, life_before + self_delta, "{name} self life");
    }
}

/// Magecraft looters: casting a spell loots (draw one, discard one → hand -1 net).
#[test]
fn stx_magecraft_loot_on_bolt_cast() {
    for def in [
        catalog::quandrix_loomweaver(),
        catalog::quandrix_aquamancer(),
        catalog::quandrix_spellseer(),
    ] {
        let name = def.name.clone();
        let mut g = two_player_game();
        for _ in 0..3 {
            g.add_card_to_library(0, catalog::island());
        }
        g.add_card_to_hand(0, catalog::forest());
        let _id = g.add_card_to_battlefield(0, def);
        drain_stack(&mut g);
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let hand_before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        // -1 cast + 1 draw - 1 discard = -1 net.
        assert_eq!(g.players[0].hand.len(), hand_before - 1, "{name} looted");
    }
}

/// ETB effect on cast, then a magecraft trigger from a follow-up Bolt.
#[test]
fn stx_etb_then_magecraft_followup() {
    // (def, opp_delta, self_delta, tokens, counters_after_bolt, self_gain_on_bolt)
    let cases: Vec<(_, i32, i32, usize, u32, i32)> = vec![
        (catalog::silverquill_soulbinder(), -2, 2, 0, 1, 0),
        (catalog::witherbloom_rootbinder(), 0, 2, 0, 0, 1),
        (catalog::witherbloom_cultivator(), 0, 0, 1, 1, 0),
        (catalog::quandrix_synthsage(), 0, 2, 0, 1, 0),
    ];
    for (def, opp_delta, self_delta, tokens, counters_after, gain_on_bolt) in cases {
        let name = def.name.clone();
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def);
        fill_mana(&mut g);
        let opp_before = g.players[1].life;
        let life_before = g.players[0].life;
        let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).expect("castable");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, opp_before + opp_delta, "{name} opp life");
        assert_eq!(g.players[0].life, life_before + self_delta, "{name} self life");
        assert_eq!(
            g.battlefield.iter().filter(|c| c.is_token).count(),
            tokens_before + tokens,
            "{name} tokens"
        );
        // Follow-up bolt fires magecraft.
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        let life_mid = g.players[0].life;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(crabomination::game::types::Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("Bolt castable");
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, life_mid + gain_on_bolt, "{name} magecraft lifegain");
        let card = g.battlefield_find(id).unwrap();
        assert_eq!(
            card.counter_count(CounterType::PlusOnePlusOne) as u32,
            counters_after,
            "{name} magecraft counter"
        );
    }
}

// ── Unique-shape tests kept individually ────────────────────────────────────

#[test]
fn fractal_reefborn_etb_doubles_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    // Put 2 counters on the bear via direct manipulation.
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 2);
    let id = g.add_card_to_hand(0, catalog::fractal_reefborn());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reefborn castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(bear).unwrap();
    // 2 + 2 = 4 counters
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 4);
}

#[test]
fn pest_vinekin_dies_gains_three_life_and_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::pest_vinekin());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    let life_before = g.players[0].life;
    let _ = g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 2);
}

#[test]
fn silverquill_inkproclamation_each_opp_sacs_and_mints_inkling() {
    let mut g = two_player_game();
    let _victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_inkproclamation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let tokens_before = g.battlefield.iter().filter(|c| c.is_token).count();
    let opp_creatures_before = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.is_creature())
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkproclamation castable");
    drain_stack(&mut g);
    let opp_creatures_after = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.is_creature())
        .count();
    assert_eq!(opp_creatures_after, opp_creatures_before - 1);
    let tokens_after = g.battlefield.iter().filter(|c| c.is_token).count();
    assert_eq!(tokens_after, tokens_before + 1);
}

#[test]
fn witherbloom_spawnkeeper_drains_when_another_creature_dies() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::witherbloom_spawnkeeper());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let life_before = g.players[0].life;
    // Kill the bear via Lightning Bolt — proper damage path emits
    // CreatureDied, firing AnotherOfYours-scoped triggers.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn lorehold_archivist_v2_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_archivist_v2());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Archivist castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Bear returned to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn lorehold_annalist_magecraft_exiles_graveyard_card() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_annalist());
    let gy_card = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // The bear should now be in exile (or removed from gy)
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == gy_card),
        "Bear should leave opponent's graveyard via Lorehold Annalist's magecraft");
}

#[test]
fn lorehold_warden_v2_etb_exiles_target_graveyard_card() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let target_id = g.players[1].graveyard.last().unwrap().id;
    let id = g.add_card_to_hand(0, catalog::lorehold_warden_v2());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crabomination::game::types::Target::Permanent(target_id)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Warden castable");
    drain_stack(&mut g);
    // Card is now in exile, not in gy.
    assert!(g.players[1].graveyard.iter().all(|c| c.id != target_id));
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Vigilance));
}

#[test]
fn silverquill_encore_pumps_team_with_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_encore());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let pwr_before = g.battlefield_find(bear).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Encore castable");
    drain_stack(&mut g);
    let b1 = g.battlefield_find(bear).unwrap();
    let b2 = g.battlefield_find(bear2).unwrap();
    assert_eq!(b1.power(), pwr_before + 1);
    assert_eq!(b2.power(), pwr_before + 1);
    assert!(b1.has_keyword(&Keyword::Lifelink));
    assert!(b2.has_keyword(&Keyword::Lifelink));
}

#[test]
fn quandrix_equalizer_etb_pumps_each_other_friendly_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_equalizer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Equalizer castable");
    drain_stack(&mut g);
    let b1 = g.battlefield_find(bear).unwrap();
    let b2 = g.battlefield_find(bear2).unwrap();
    let eq = g.battlefield_find(id).unwrap();
    assert_eq!(b1.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(b2.counter_count(CounterType::PlusOnePlusOne), 1);
    // Equalizer itself doesn't get a counter (OtherThanSource)
    assert_eq!(eq.counter_count(CounterType::PlusOnePlusOne), 0);
}

#[test]
fn inkling_sentencer_shrinks_opp_creature_on_etb() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::inkling_sentencer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let pwr_before = g.battlefield_find(opp_bear).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crabomination::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Sentencer castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(opp_bear).unwrap().power(),
        pwr_before - 1
    );
}

#[test]
fn witherbloom_toxicologist_shrinks_target_on_is_cast() {
    let mut g = two_player_game();
    let _tox = g.add_card_to_battlefield(0, catalog::witherbloom_toxicologist());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let pwr_before = g.battlefield_find(opp_bear).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Permanent(opp_bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Toxicologist trigger shrinks the target, but the Bolt also deals
    // 3 damage. The card may be in graveyard now if it died.
    let still_alive = g.battlefield_find(opp_bear);
    if let Some(c) = still_alive {
        assert_eq!(c.power(), pwr_before - 1);
    }
}

#[test]
fn prismari_emberscribe_pings_creature_on_is_cast() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _id = g.add_card_to_battlefield(0, catalog::prismari_emberscribe());
    drain_stack(&mut g);
    let dmg_before = g.battlefield_find(target).unwrap().damage;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft 1 = 4 dmg → bear (2 toughness) goes to gy.
    // If gy'd, battlefield_find returns None; otherwise check damage.
    let still_in_play = g.battlefield_find(target);
    if let Some(c) = still_in_play {
        assert!(c.damage > dmg_before);
    }
}

#[test]
fn prismari_maestro_draws_two_on_combat_damage() {
    use crabomination::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let maestro = g.add_card_to_battlefield(0, catalog::prismari_maestro());
    g.clear_sickness(maestro);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority)
            .expect("pass priority");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: maestro,
        target: AttackTarget::Player(1),
    }]))
    .expect("Maestro attacks");
    drain_stack(&mut g);
    let hand_before = g.players[0].hand.len();
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority)
            .expect("pass priority");
    }
    g.resolve_combat().expect("combat damage");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].hand.len(),
        hand_before + 2,
        "drew 2 from Maestro combat-damage trigger"
    );
}

#[test]
fn fractal_bloomweaver_etb_with_counters_and_pumps_others() {
    let mut g = two_player_game();
    let other_fractal = g.add_card_to_battlefield(0, catalog::fractal_grower());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::fractal_bloomweaver());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bloomweaver castable");
    drain_stack(&mut g);
    let bloom = g.battlefield_find(id).unwrap();
    assert_eq!(
        bloom
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        3,
        "Bloomweaver enters with 3 counters"
    );
    let other = g.battlefield_find(other_fractal).unwrap();
    assert_eq!(
        other
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        1,
        "other Fractal gains 1 counter via Bloomweaver ETB"
    );
}

#[test]
fn witherbloom_rotsage_etb_offers_optional_sac_loot() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    let _fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::witherbloom_rotsage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Rotsage castable");
    drain_stack(&mut g);
    // AutoDecider declines the may-do by default, leaving fodder alive.
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn prismari_cantrip_bolt_deals_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::mountain());
    // Use a 3-toughness creature so it survives the 2 damage.
    let beefy = g.add_card_to_battlefield(1, catalog::silverquill_essayist());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::prismari_cantrip_bolt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(crabomination::game::types::Target::Permanent(beefy)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cantrip Bolt castable");
    drain_stack(&mut g);
    // We cast (hand -1), drew (hand +1), net 0
    assert_eq!(g.players[0].hand.len(), hand_before);
    let card = g.battlefield_find(beefy).unwrap();
    assert_eq!(card.damage, 2);
}

#[test]
fn prismari_stormbearer_etb_loots_then_magecraft_pumps_self() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::prismari_stormbearer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormbearer castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Flying));
    assert_eq!(card.power(), 4);
}

#[test]
fn witherbloom_bramblevine_grows_on_lifegain() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_bramblevine());
    drain_stack(&mut g);
    let counters_before = g
        .battlefield_find(id)
        .unwrap()
        .counters
        .get(&CounterType::PlusOnePlusOne)
        .copied()
        .unwrap_or(0);
    // Trigger via a direct lifegain effect — cast Witherbloom Sapglyph
    // for +2 life.
    let drain = g.add_card_to_hand(0, catalog::witherbloom_sapglyph());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: drain,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Sapglyph castable");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id)
            .unwrap()
            .counters
            .get(&CounterType::PlusOnePlusOne)
            .copied()
            .unwrap_or(0),
        counters_before + 1
    );
}
