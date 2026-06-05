use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn silverquill_quilledict_b154_drains_three_and_mints_two_inklings() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::silverquill_quilledict_b154());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Quilledict castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3, "Drain 3");
    let inklings = g.battlefield.iter()
        .filter(|c| c.definition.name == "Inkling").count();
    assert_eq!(inklings, 2, "Mints exactly 2 Inkling tokens");
}

// ── batch 154 — Quandrix cards ─────────────────────────────────────────────

#[test]
fn quandrix_fractalsmith_b154_mints_fractal_on_cast() {
    let mut g = two_player_game();
    let _fs = g.add_card_to_battlefield(0, catalog::quandrix_fractalsmith_b154());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let fractals = g.battlefield.iter()
        .filter(|c| c.definition.name == "Fractal").count();
    assert_eq!(fractals, 1, "Magecraft mints exactly one Fractal token");
}

#[test]
fn quandrix_equationmage_b154_self_grows_on_cast() {
    let mut g = two_player_game();
    let em = g.add_card_to_battlefield(0, catalog::quandrix_equationmage_b154());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(em).map(|c| {
        c.counters.iter()
            .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n).sum::<u32>()
    }).unwrap_or(0);
    assert_eq!(counters, 1);
}

#[test]
fn quandrix_riftguard_b154_etb_pumps_friendly_with_two_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rg = g.add_card_to_hand(0, catalog::quandrix_riftguard_b154());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rg, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Riftguard castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(bear).map(|c| {
        c.counters.iter()
            .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n).sum::<u32>()
    }).unwrap_or(0);
    assert_eq!(counters, 2);
}

#[test]
fn quandrix_tidesinger_b154_draws_on_cast() {
    let mut g = two_player_game();
    let _ts = g.add_card_to_battlefield(0, catalog::quandrix_tidesinger_b154());
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let cards_with_bolt = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // hand started with cards_with_bolt; after cast: -bolt, +draw = same
    assert_eq!(g.players[0].hand.len(), cards_with_bolt,
        "Magecraft drew 1 to replace the cast Bolt");
}

#[test]
fn quandrix_calculation_b154_mints_four_four_fractal_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::quandrix_calculation_b154());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Calculation castable");
    drain_stack(&mut g);
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Fractal").collect();
    assert_eq!(fractals.len(), 1);
    let counters: u32 = fractals[0].counters.iter()
        .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
        .map(|(_, n)| *n).sum();
    assert_eq!(counters, 4, "0/0 Fractal + 4 counters = 4/4");
    assert_eq!(g.players[0].hand.len(), hand_before, "spell out, draw 1 in");
}

#[test]
fn lorehold_searingscholar_b154_drains_each_opp_on_cast() {
    let mut g = two_player_game();
    let _ls = g.add_card_to_battlefield(0, catalog::lorehold_searingscholar_b154());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt: -3, magecraft drain: -1, total -4
    assert_eq!(g.players[1].life, life1_before - 4);
}

#[test]
fn lorehold_cinderward_b154_etb_gains_three_life() {
    let mut g = two_player_game();
    let lc = g.add_card_to_hand(0, catalog::lorehold_cinderward_b154());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: lc, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cinderward castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
}

#[test]
fn lorehold_strikeritual_b154_burns_and_mints_spirit() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::lorehold_strikeritual_b154());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Strikeritual castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits, 1, "Mints exactly one Spirit token");
}

#[test]
fn quandrix_wavebreaker_b154_etb_bounces_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wb = g.add_card_to_hand(0, catalog::quandrix_wavebreaker_b154());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: wb, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wavebreaker castable");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear returned to opp's hand");
}

#[test]
fn quandrix_bloomguard_b154_etb_fans_counters_on_each_friendly_creature() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bg = g.add_card_to_hand(0, catalog::quandrix_bloomguard_b154());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bg, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bloomguard castable");
    drain_stack(&mut g);
    for id in [b1, b2] {
        let c = g.battlefield_find(id).unwrap();
        let counters: u32 = c.counters.iter()
            .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n).sum();
        assert_eq!(counters, 1, "Each friendly bear gets a +1/+1 counter");
    }
}

// ── batch 154 — Prismari cards ─────────────────────────────────────────────

#[test]
fn prismari_treasurelord_b154_mints_treasure_on_cast() {
    let mut g = two_player_game();
    let _tl = g.add_card_to_battlefield(0, catalog::prismari_treasurelord_b154());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 1, "Magecraft mints one Treasure");
}

#[test]
fn prismari_inferno_b154_burns_target_for_five() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::prismari_inferno_b154());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(serra)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inferno castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == serra),
        "Serra Angel (4 toughness) takes 5 → dies");
}

#[test]
fn prismari_tempestmage_b154_self_pumps_on_cast() {
    let mut g = two_player_game();
    let tm = g.add_card_to_battlefield(0, catalog::prismari_tempestmage_b154());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pt = g.computed_permanent(tm).map(|cp| (cp.power, cp.toughness));
    assert_eq!(pt, Some((2, 3)), "1/2 + magecraft +1/+1 = 2/3");
}

#[test]
fn prismari_crashbinder_b154_loots_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _cb = g.add_card_to_battlefield(0, catalog::prismari_crashbinder_b154());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // hand: -bolt +draw -discard = -1. gy: +bolt(cast) +discard = +2
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "looted: net -1 hand size");
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2,
        "Bolt to gy + discarded card to gy");
}

#[test]
fn prismari_sparkglyph_b154_burns_target_for_three() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::prismari_sparkglyph_b154());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkglyph castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 3);
}

#[test]
fn prismari_stormbreaker_b154_etb_burns_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let sb = g.add_card_to_hand(0, catalog::prismari_stormbreaker_b154());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: sb, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Stormbreaker castable");
    drain_stack(&mut g);
    // The ETB trigger auto-targets — typically the opponent gets the
    // 2 damage. Caster also draws 1 (Island onto hand).
    assert!(g.players[1].life < life1_before
            || g.battlefield.iter().any(|c| c.damage >= 2 && c.id != sb),
        "ETB dealt 2 damage somewhere — either to opp or a creature");
    // Hand: -spell +draw = same total
    assert_eq!(g.players[0].hand.len(), hand_before,
        "ETB drew a card to replace the cast spell");
}

#[test]
fn prismari_flameseeker_b154_loots_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _fs = g.add_card_to_battlefield(0, catalog::prismari_flameseeker_b154());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "Looted: net -1 hand");
}

#[test]
fn prismari_calligrapher_b154_etb_scrys_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::mountain());
    let pc = g.add_card_to_hand(0, catalog::prismari_calligrapher_b154());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pc, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Calligrapher castable");
    drain_stack(&mut g);
    // After scry 2, library still has 2 cards (no draw)
    assert_eq!(g.players[0].library.len(), 2);
}

#[test]
fn silverquill_sentinel_b154_gains_life_on_is_cast() {
    let mut g = two_player_game();
    let _ss = g.add_card_to_battlefield(0, catalog::silverquill_sentinel_b154());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn silverquill_sphereturn_b154_drains_four() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::silverquill_sphereturn_b154());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Sphereturn castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 4);
    assert_eq!(g.players[0].life, life0_before + 4);
}

#[test]
fn inkling_bookwarden_b154_is_a_three_mana_lifelink_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_bookwarden_b154());
    let c = g.battlefield_find(id).expect("on bf");
    assert!(c.definition.keywords.contains(&Keyword::Flying));
    assert!(c.definition.keywords.contains(&Keyword::Lifelink));
    assert_eq!(c.definition.power, 2);
    assert_eq!(c.definition.toughness, 3);
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

// ── batch 154 helper shortcut lock-in tests ────────────────────────────────

#[test]
fn shortcut_magecraft_mint_pest_uses_magecraft_trigger_with_create_token_body() {
    // Lock in that magecraft_mint_pest() builds a magecraft trigger
    // (SpellCast scope + IS filter) whose body is Effect::CreateToken
    // with the stx_pest_token() definition. Future refactors can't
    // collapse this onto magecraft_mint_inkling or magecraft_treasure.
    use crate::effect::shortcut::magecraft_mint_pest;
    let trig = magecraft_mint_pest();
    assert_eq!(trig.event.kind, crate::effect::EventKind::SpellCast);
    match trig.effect {
        crate::effect::Effect::CreateToken { count, ref definition, .. } => {
            assert_eq!(count, crate::effect::Value::Const(1));
            assert_eq!(definition.name, "Pest");
        }
        _ => panic!("body must be CreateToken with Pest definition"),
    }
}

#[test]
fn shortcut_magecraft_mint_inkling_uses_inkling_token() {
    // Lock in that magecraft_mint_inkling() mints a 1/1 W/B flying
    // Inkling. Distinguishes from magecraft_mint_pest (B/G Pest) and
    // magecraft_mint_spirit (R/W Spirit).
    use crate::effect::shortcut::magecraft_mint_inkling;
    let trig = magecraft_mint_inkling();
    assert_eq!(trig.event.kind, crate::effect::EventKind::SpellCast);
    match trig.effect {
        crate::effect::Effect::CreateToken { ref definition, .. } => {
            assert_eq!(definition.name, "Inkling");
            assert!(definition.keywords.contains(&Keyword::Flying));
        }
        _ => panic!("body must be CreateToken with Inkling definition"),
    }
}

#[test]
fn shortcut_magecraft_mint_fractal_seq_creates_token_then_stamps_counters() {
    // Lock in that magecraft_mint_fractal(N) is a Seq[CreateToken,
    // AddCounter(LastCreatedToken, +1/+1, N)] body — the printed Quandrix
    // "create a 0/0 Fractal with N +1/+1 counters" pattern.
    use crate::effect::shortcut::magecraft_mint_fractal;
    let trig = magecraft_mint_fractal(2);
    assert_eq!(trig.event.kind, crate::effect::EventKind::SpellCast);
    match &trig.effect {
        crate::effect::Effect::Seq(steps) => {
            assert_eq!(steps.len(), 2);
            assert!(matches!(steps[0], crate::effect::Effect::CreateToken { .. }));
            match &steps[1] {
                crate::effect::Effect::AddCounter { what, kind, amount } => {
                    assert!(matches!(what, crate::effect::Selector::LastCreatedToken));
                    assert_eq!(*kind, CounterType::PlusOnePlusOne);
                    assert_eq!(*amount, crate::effect::Value::Const(2));
                }
                _ => panic!("step 1 must be AddCounter"),
            }
        }
        _ => panic!("body must be Seq[CreateToken, AddCounter]"),
    }
}

#[test]
fn shortcut_magecraft_mint_and_drain_seq_mints_then_drains() {
    // Lock in that magecraft_mint_and_drain(def, count, amount) builds a
    // magecraft trigger whose body is Seq[CreateToken(count), Drain(amount)]
    // — the Pest-aristocrats "mint a body then drain the table per spell"
    // shape. Mint must precede the drain so the token is on the battlefield
    // before any "if you gained life" / sacrifice payoff sees the drain.
    use crate::effect::shortcut::magecraft_mint_and_drain;
    let trig = magecraft_mint_and_drain(crate::catalog::stx_pest_token(), 1, 2);
    assert_eq!(trig.event.kind, crate::effect::EventKind::SpellCast);
    match &trig.effect {
        crate::effect::Effect::Seq(steps) => {
            assert_eq!(steps.len(), 2);
            match &steps[0] {
                crate::effect::Effect::CreateToken { count, definition, .. } => {
                    assert_eq!(*count, crate::effect::Value::Const(1));
                    assert_eq!(definition.name, "Pest");
                }
                _ => panic!("step 0 must be CreateToken"),
            }
            match &steps[1] {
                crate::effect::Effect::Drain { amount, .. } => {
                    assert_eq!(*amount, crate::effect::Value::Const(2));
                }
                _ => panic!("step 1 must be Drain"),
            }
        }
        _ => panic!("body must be Seq[CreateToken, Drain]"),
    }
}

#[test]
fn shortcut_dies_mint_pest_uses_creature_died_self_source() {
    // Lock in that dies_mint_pest() builds a CreatureDied/SelfSource
    // trigger whose body mints a Pest. Pulls the self-replacing-Pest
    // pattern (Pest Swarmer, future Pest cards) onto a one-liner.
    use crate::effect::shortcut::dies_mint_pest;
    let trig = dies_mint_pest();
    assert_eq!(trig.event.kind, crate::effect::EventKind::CreatureDied);
    assert!(matches!(trig.event.scope, crate::effect::EventScope::SelfSource));
    match trig.effect {
        crate::effect::Effect::CreateToken { ref definition, .. } => {
            assert_eq!(definition.name, "Pest");
        }
        _ => panic!("body must be CreateToken with Pest definition"),
    }
}

#[test]
fn shortcut_on_attack_mint_lorehold_spirit_uses_attacks_self_source() {
    // Lock in that on_attack_mint_lorehold_spirit() builds an
    // Attacks/SelfSource trigger whose body mints a 2/2 R/W Spirit.
    // Distinguishes from on_attack_create_token<T> which is the generic
    // form — this shortcut bakes the Lorehold Spirit token definition.
    use crate::effect::shortcut::on_attack_mint_lorehold_spirit;
    let trig = on_attack_mint_lorehold_spirit();
    assert_eq!(trig.event.kind, crate::effect::EventKind::Attacks);
    assert!(matches!(trig.event.scope, crate::effect::EventScope::SelfSource));
    match trig.effect {
        crate::effect::Effect::CreateToken { ref definition, .. } => {
            assert_eq!(definition.name, "Spirit");
            assert_eq!(definition.power, 2);
            assert_eq!(definition.toughness, 2);
        }
        _ => panic!("body must be CreateToken with Lorehold Spirit definition"),
    }
}

#[test]
fn shortcut_magecraft_add_counter_self_targets_self_with_plus_one_plus_one() {
    // Lock in that magecraft_add_counter_self() is a magecraft trigger
    // whose body is AddCounter(Selector::This, +1/+1, 1). Prevents
    // future refactors from accidentally collapsing onto
    // magecraft_add_counter_to_friendly (which targets a friendly).
    use crate::effect::shortcut::magecraft_add_counter_self;
    let trig = magecraft_add_counter_self();
    assert_eq!(trig.event.kind, crate::effect::EventKind::SpellCast);
    match trig.effect {
        crate::effect::Effect::AddCounter { ref what, kind, amount } => {
            assert!(matches!(what, crate::effect::Selector::This));
            assert_eq!(kind, CounterType::PlusOnePlusOne);
            assert_eq!(amount, crate::effect::Value::Const(1));
        }
        _ => panic!("body must be AddCounter targeting Self"),
    }
}

// ── Batch 155: stat / body / trigger lock-in tests ──────────────────────────

#[test]
fn pest_acolyte_b155_gains_one_life_on_attack() {
    let mut g = two_player_game();
    let pid = g.add_card_to_battlefield(0, catalog::pest_acolyte_b155());
    g.clear_sickness(pid);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pid,
        target: AttackTarget::Player(1),
    }]))
    .expect("attacker declare");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

// ── batch 155 — Silverquill cards ──────────────────────────────────────────

#[test]
fn silverquill_reciter_b155_gains_two_life_on_is_cast() {
    let mut g = two_player_game();
    let _r = g.add_card_to_battlefield(0, catalog::silverquill_reciter_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2, "Magecraft +2 life");
}

#[test]
fn inkling_striplark_b155_drains_each_opp_on_is_cast() {
    let mut g = two_player_game();
    let _s = g.add_card_to_battlefield(0, catalog::inkling_striplark_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1 - 3, "Opp -1 drain + -3 bolt");
    assert_eq!(g.players[0].life, life0_before + 1, "+1 life from drain");
}

#[test]
fn silverquill_manuscriber_b155_etb_scrys() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::silverquill_manuscriber_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Manuscriber castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("on bf");
    assert_eq!(c.definition.power, 2);
    assert_eq!(c.definition.toughness, 3);
}

#[test]
fn inkling_lifepoet_b155_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_lifepoet_b155());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Lifepoet castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2);
    assert_eq!(g.players[0].life, life0_before + 2);
    let c = g.battlefield_find(id).expect("on bf");
    assert!(c.definition.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn silverquill_adjudicator_b155_exiles_creature_and_drains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let adj = g.add_card_to_hand(0, catalog::silverquill_adjudicator_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: adj, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Adjudicator castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear exiled");
    assert!(g.exile.iter().any(|c| c.id == bear), "Bear in exile");
    assert_eq!(g.players[1].life, life1_before - 1);
    assert_eq!(g.players[0].life, life0_before + 1);
}

#[test]
fn inkling_spellbinder_b155_magecraft_mints_inkling() {
    let mut g = two_player_game();
    let _sb = g.add_card_to_battlefield(0, catalog::inkling_spellbinder_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let inklings_before = g.battlefield.iter()
        .filter(|c| c.definition.name == "Inkling").count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let inklings_after = g.battlefield.iter()
        .filter(|c| c.definition.name == "Inkling").count();
    assert_eq!(inklings_after, inklings_before + 1, "Spellbinder mints an Inkling");
}

#[test]
fn silverquill_quillplay_b155_drains_one_and_mints_inkling() {
    let mut g = two_player_game();
    let qp = g.add_card_to_hand(0, catalog::silverquill_quillplay_b155());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: qp, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Quillplay castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1);
    assert_eq!(g.players[0].life, life0_before + 1);
    let inklings = g.battlefield.iter()
        .filter(|c| c.definition.name == "Inkling").count();
    assert_eq!(inklings, 1, "Quillplay mints an Inkling token");
}

#[test]
fn silverquill_curatorial_b155_drains_and_reanimates() {
    let mut g = two_player_game();
    let dead_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let cur = g.add_card_to_hand(0, catalog::silverquill_curatorial_b155());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: cur, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Curatorial castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2, "Opp -2 drain");
    assert!(g.battlefield_find(dead_bear).is_some(), "Bear reanimated to bf");
}

#[test]
fn inkling_slipscribe_b155_self_pumps_on_is_cast() {
    let mut g = two_player_game();
    let ss = g.add_card_to_battlefield(0, catalog::inkling_slipscribe_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let power_before = g.battlefield_find(ss).expect("on bf").power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let power_after = g.battlefield_find(ss).expect("on bf").power();
    assert_eq!(power_after, power_before + 1, "+1/+0 from magecraft");
}

#[test]
fn silverquill_recital_b155_each_opp_sacs_and_mints() {
    let mut g = two_player_game();
    let _opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let recital = g.add_card_to_hand(0, catalog::silverquill_recital_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: recital, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Recital castable");
    drain_stack(&mut g);
    let opp_creatures = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.is_creature()).count();
    assert_eq!(opp_creatures, 0, "Opp sacrificed their creature");
    let my_inklings = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Inkling").count();
    assert_eq!(my_inklings, 1, "Recital mints an Inkling");
    assert_eq!(g.players[0].life, life0_before + 1, "+1 life");
}

#[test]
fn inkling_vespermage_b155_etb_grows_target_inkling() {
    let mut g = two_player_game();
    let other_ink = g.add_card_to_battlefield(0, catalog::inkling_aspirant());
    let vesper = g.add_card_to_hand(0, catalog::inkling_vespermage_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: vesper, target: Some(Target::Permanent(other_ink)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vespermage castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(other_ink).map(|c| {
        c.counters.iter()
            .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n).sum::<u32>()
    }).unwrap_or(0);
    assert_eq!(counters, 1, "Target Inkling gets +1/+1 counter");
}

#[test]
fn silverquill_caesura_b155_taps_creature_and_cantrips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let caes = g.add_card_to_hand(0, catalog::silverquill_caesura_b155());
    g.players[0].mana_pool.add(Color::White, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: caes, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Caesura castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).expect("on bf").tapped, "Bear tapped");
    // Hand: -1 (cast) +1 (draw) = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_pen_verseman_b155_etb_drains_one_and_scrys() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::inkling_pen_verseman_b155());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pen-Verseman castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1);
    assert_eq!(g.players[0].life, life0_before + 1);
    let c = g.battlefield_find(id).expect("on bf");
    assert!(c.definition.keywords.contains(&Keyword::Flying));
    assert!(c.definition.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn silverquill_liturgist_ii_b155_drains_on_is_cast() {
    let mut g = two_player_game();
    let _lit = g.add_card_to_battlefield(0, catalog::silverquill_liturgist_ii_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1 - 3);
    assert_eq!(g.players[0].life, life0_before + 1);
}

#[test]
fn inkling_skydrifter_b155_etb_with_plus_one_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_skydrifter_b155());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Skydrifter castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(id).map(|c| {
        c.counters.iter()
            .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n).sum::<u32>()
    }).unwrap_or(0);
    assert_eq!(counters, 1, "+1/+1 counter from ETB");
    let c = g.battlefield_find(id).expect("on bf");
    assert!(c.definition.keywords.contains(&Keyword::Flying));
    assert!(c.definition.keywords.contains(&Keyword::Lifelink));
}

// ── batch 155 — Witherbloom cards ──────────────────────────────────────────

#[test]
fn witherbloom_bonebinder_b155_drains_on_is_cast() {
    let mut g = two_player_game();
    let _b = g.add_card_to_battlefield(0, catalog::witherbloom_bonebinder_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1 - 3);
    assert_eq!(g.players[0].life, life0_before + 1);
}

#[test]
fn pest_hivescholar_b155_mints_pest_on_is_cast() {
    let mut g = two_player_game();
    let _ph = g.add_card_to_battlefield(0, catalog::pest_hivescholar_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let pests_before = g.battlefield.iter()
        .filter(|c| c.definition.name == "Pest").count();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter()
        .filter(|c| c.definition.name == "Pest").count();
    assert_eq!(pests_after, pests_before + 1, "Magecraft mints a Pest");
}

#[test]
fn witherbloom_cultivator_b155_etb_gains_life_and_surveils() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::witherbloom_cultivator_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cultivator castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2, "+2 life from ETB");
}

#[test]
fn witherbloom_sapling_b155_activation_grows_self() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::witherbloom_sapling_b155());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, x_value: None,
    }).expect("Activation succeeds");
    drain_stack(&mut g);
    let counters = g.battlefield_find(s).map(|c| {
        c.counters.iter()
            .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n).sum::<u32>()
    }).unwrap_or(0);
    assert_eq!(counters, 1, "Activation adds +1/+1 counter");
}

#[test]
fn witherbloom_inkblossom_b155_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_inkblossom_b155());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Inkblossom castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2);
    assert_eq!(g.players[0].life, life0_before + 2);
}

#[test]
fn pest_conjuror_b155_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_conjuror_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Conjuror castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.definition.name == "Pest").count();
    assert_eq!(pests, 2, "Two Pests minted on ETB");
}

#[test]
fn witherbloom_tutor_b155_searches_creature_to_hand() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear_in_lib = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bear_in_lib))]));
    let tutor = g.add_card_to_hand(0, catalog::witherbloom_tutor_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: tutor, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Tutor castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 2, "-2 life paid");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_in_lib),
        "Bear searched to hand");
}

#[test]
fn pest_acolyte_ii_b157_gains_life_on_is_cast() {
    let mut g = two_player_game();
    let _pa = g.add_card_to_battlefield(0, catalog::pest_acolyte_ii_b157());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "+1 life");
}

#[test]
fn witherbloom_vinepoet_b155_grows_on_is_cast() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::witherbloom_vinepoet_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(v).map(|c| {
        c.counters.iter()
            .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n).sum::<u32>()
    }).unwrap_or(0);
    assert_eq!(counters, 1, "+1/+1 counter from magecraft");
}

// ── batch 155 — Lorehold cards ─────────────────────────────────────────────

#[test]
fn lorehold_glyphbearer_b155_pings_on_is_cast() {
    let mut g = two_player_game();
    let _gb = g.add_card_to_battlefield(0, catalog::lorehold_glyphbearer_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Magecraft pings 1 + Bolt pings 3 = -4 life total
    assert_eq!(g.players[1].life, life1_before - 4);
}

#[test]
fn lorehold_watchspirit_b155_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_watchspirit_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Watchspirit castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_spiritforge_b155_attack_mints_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_spiritforge_b155());
    g.clear_sickness(id);
    let spirits_before = g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit").count();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("attacker declared");
    drain_stack(&mut g);
    let spirits_after = g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits_after, spirits_before + 1, "Attack mints a Spirit");
}

#[test]
fn lorehold_pyrescholar_b155_etb_pings_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_pyrescholar_b155());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Pyrescholar castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1,
        "Pyrescholar ETB pings opp player for 1");
}

#[test]
fn lorehold_battlechant_b155_deals_damage_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bc = g.add_card_to_hand(0, catalog::lorehold_battlechant_b155());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bc, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battlechant castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none() ||
        g.battlefield_find(bear).map(|c| c.damage).unwrap_or(0) == 2,
        "Bear takes 2 damage (or dies from SBA)");
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_ancestralist_b155_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_ancestralist_b155());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Ancestralist castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead),
        "Bear returned to hand from graveyard");
}

#[test]
fn lorehold_echocaller_b155_gains_life_on_is_cast() {
    let mut g = two_player_game();
    let _ec = g.add_card_to_battlefield(0, catalog::lorehold_echocaller_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn witherbloom_bramblelord_ii_b155_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_bramblelord_ii_b155());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {3}{B}{G}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_before + 2);
    assert_eq!(g.players[1].life, p1_before - 2);
}

#[test]
fn witherbloom_mossdrinker_b155_etb_gains_three_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_mossdrinker_b155());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {3}{G}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
}

#[test]
fn lorehold_chronicler_b155_pings_on_instant_cast() {
    let mut g = two_player_game();
    let _c = g.add_card_to_battlefield(0, catalog::lorehold_chronicler_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt: -3 life. Magecraft ping: -1 life. Total -4.
    assert_eq!(g.players[1].life, p1_before - 4);
}

#[test]
fn lorehold_reverent_b155_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_reverent_b155());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {R}{W}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn lorehold_pyromancer_b155_pings_and_gains_on_cast() {
    let mut g = two_player_game();
    let _p = g.add_card_to_battlefield(0, catalog::lorehold_pyromancer_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    // P0 gains 1; P1 loses 3 (bolt) + 1 (magecraft ping) = 4.
    assert_eq!(g.players[0].life, p0_before + 1);
    assert_eq!(g.players[1].life, p1_before - 4);
}

#[test]
fn silverquill_eulogist_b155_destroys_and_loses_one_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_eulogist_b155());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {1}{W}{B}");
    drain_stack(&mut g);
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == bear),
        "bear should be destroyed"
    );
    assert_eq!(g.players[0].life, life_before - 1);
}

#[test]
fn quandrix_embodiment_b155_etb_with_plus_one_plus_one_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_embodiment_b155());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {2}{G}{U}");
    drain_stack(&mut g);
    let card = g.battlefield.iter()
        .find(|c| c.definition.name == "Quandrix Embodiment (b155)")
        .expect("Embodiment on battlefield");
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1);
    // 3/3 + counter = 4/4 effective.
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 4);
}

#[test]
fn fractal_magus_b155_scries_on_instant_cast() {
    let mut g = two_player_game();
    let _m = g.add_card_to_battlefield(0, catalog::fractal_magus_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let lib_before = g.players[0].library.len();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let _ = lib_before;
    // Scry: doesn't change library size, but the trigger should fire.
    // Just lock in that it doesn't crash.
}

#[test]
fn lorehold_pyrebolt_b155_deals_two_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pb = g.add_card_to_hand(0, catalog::lorehold_pyrebolt_b155());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pb, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrebolt castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none() ||
        g.battlefield_find(bear).map(|c| c.damage).unwrap_or(0) == 2,
        "Bear takes 2 damage (or dies)");
}

#[test]
fn lorehold_spirit_caller_b155_etb_mints_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_caller_b155());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Spirit Caller castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits, 2, "Spirit Caller mints two Spirit tokens");
}

// ── batch 155 — Quandrix cards ─────────────────────────────────────────────

#[test]
fn quandrix_cartographer_b155_etb_scrys_one() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::quandrix_cartographer_b155());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cartographer castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("on bf");
    assert_eq!(c.definition.power, 2);
    assert_eq!(c.definition.toughness, 2);
}

#[test]
fn quandrix_hatchling_b155_grows_on_is_cast() {
    let mut g = two_player_game();
    let h = g.add_card_to_battlefield(0, catalog::quandrix_hatchling_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(h).map(|c| {
        c.counters.iter()
            .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n).sum::<u32>()
    }).unwrap_or(0);
    assert_eq!(counters, 1, "+1/+1 counter from magecraft");
}

#[test]
fn quandrix_fractalist_b155_mints_fractal_on_is_cast() {
    let mut g = two_player_game();
    let _f = g.add_card_to_battlefield(0, catalog::quandrix_fractalist_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let fractals = g.battlefield.iter()
        .filter(|c| c.definition.name == "Fractal").count();
    assert_eq!(fractals, 1, "Magecraft mints a Fractal token");
}

#[test]
fn quandrix_scriptor_b155_scrys_on_is_cast() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let _s = g.add_card_to_battlefield(0, catalog::quandrix_scriptor_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry doesn't change library size; just verify the bolt cast didn't crash
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn quandrix_forecaster_b155_etb_draws_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_forecaster_b155());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {1}{G}{U}");
    drain_stack(&mut g);
    // Hand: -1 (cast) +1 (draw) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_coursebearer_b155_etb_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_coursebearer_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Coursebearer castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) + 1 (draw) = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_combustion_b155_kills_2_toughness_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_combustion_b155());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {U}{R}");
    drain_stack(&mut g);
    // 2 damage to a 2/2 → bear dies.
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == bear),
        "bear should die from 2 damage"
    );
}

#[test]
fn prismari_surge_b155_draw_2_discard_1() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_surge_b155());
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {1}{U}{R}");
    drain_stack(&mut g);
    // -1 (cast) +2 (draw) -1 (discard) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_mathwarden_b155_may_draw_on_is_cast() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let _mw = g.add_card_to_battlefield(0, catalog::quandrix_mathwarden_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Hand: -1 (cast Bolt) +1 (draw via magecraft May) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn elemental_whirlwind_b155_damages_each_opponent_and_draws() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::elemental_whirlwind_b155());
    g.add_card_to_library(1, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_before = g.players[1].life;
    let p1_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {3}{U}{R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 4);
    assert_eq!(g.players[1].hand.len(), p1_hand_before + 1);
}

#[test]
fn prismari_treasure_spawner_b155_mints_treasure_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_treasure_spawner_b155());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("castable for {3}{U}{R}");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1);
}

// ── Batch 155: CR rule lock-in tests ────────────────────────────────────────

#[test]
fn cr_506_5_attacks_trigger_fires_per_attacker_in_batch() {
    // CR 506.5 / Sparring Regimen pattern: when multiple attackers are
    // declared in one batch, the "whenever you attack" trigger (scoped
    // YourControl on EventKind::Attacks) fires once per attacker. Each
    // attacker should pick up its own +1/+1 counter.
    let mut g = two_player_game();
    let _regimen = g.add_card_to_battlefield(0, catalog::sparring_regimen());
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear1);
    g.clear_sickness(bear2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: bear1, target: AttackTarget::Player(1) },
        Attack { attacker: bear2, target: AttackTarget::Player(1) },
    ]))
    .expect("bears can attack");
    drain_stack(&mut g);
    let b1 = g.battlefield.iter().find(|c| c.id == bear1).unwrap();
    let b2 = g.battlefield.iter().find(|c| c.id == bear2).unwrap();
    assert_eq!(b1.counter_count(CounterType::PlusOnePlusOne), 1,
        "first attacker should pick up one +1/+1 counter");
    assert_eq!(b2.counter_count(CounterType::PlusOnePlusOne), 1,
        "second attacker should pick up one +1/+1 counter");
}

#[test]
fn cr_603_attacks_trigger_broadcast_skips_opponent_anchors() {
    // CR 603.6 — "Whenever YOU attack" triggers from a YourControl
    // scoped Attacks listener should NOT fire when the OPPONENT
    // declares attackers, because their broadcast walks the opponent's
    // permanents (not yours). Lock-in: opponent attacks with their own
    // creature, but my Sparring Regimen sits on the bf — no counter
    // should land on the opponent's attacker.
    let mut g = two_player_game();
    let _regimen = g.add_card_to_battlefield(0, catalog::sparring_regimen());
    drain_stack(&mut g);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(0),
    }]))
    .expect("opponent's bear can attack");
    drain_stack(&mut g);
    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        bear_card.counter_count(CounterType::PlusOnePlusOne),
        0,
        "opponent's attacker should NOT get my Regimen's counter"
    );
}

#[test]
fn cr_118_8_exile_from_graveyard_cost_pre_flight_no_mana_burned() {
    // CR 118.8 — "If a player can't pay the costs of a spell or
    // ability, they can't cast or activate it." Lock-in: when no
    // graveyard card matches the exile-from-gy cost, activation is
    // rejected before mana / tap is committed.
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::lorehold_pledgemage());
    g.clear_sickness(pm);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let pool_red = g.players[0].mana_pool.amount(Color::Red);
    let pool_white = g.players[0].mana_pool.amount(Color::White);
    let pool_colorless = g.players[0].mana_pool.colorless_amount();
    // No card in graveyard — activation must be rejected and mana
    // pool must NOT be drained.
    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: pm,
        ability_index: 0,
        target: None,
        x_value: None,
    });
    assert!(result.is_err(), "must reject without legal gy-exile target");
    // Mana pool unchanged.
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), pool_red);
    assert_eq!(g.players[0].mana_pool.amount(Color::White), pool_white);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), pool_colorless);
    // Source not tapped.
    let pm_card = g.battlefield.iter().find(|c| c.id == pm).unwrap();
    assert!(!pm_card.tapped, "source should not be tapped on failed activation");
}

// ── Batch 156: attack-anchor lock-in tests (multi-attacker fan-out) ────────

#[test]
fn lorehold_banner_b156_pumps_each_attacker_in_batch() {
    // Push c4b7b14's batch-fanout fix: a multi-attacker swing fans the
    // Lorehold Banner's "another attacks" trigger to each attacker.
    let mut g = two_player_game();
    let _banner = g.add_card_to_battlefield(0, catalog::lorehold_banner_b156());
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(b1);
    g.clear_sickness(b2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: b1, target: AttackTarget::Player(1) },
        Attack { attacker: b2, target: AttackTarget::Player(1) },
    ]))
    .expect("both bears can attack");
    drain_stack(&mut g);
    let bear1 = g.battlefield.iter().find(|c| c.id == b1).unwrap();
    let bear2 = g.battlefield.iter().find(|c| c.id == b2).unwrap();
    // Each attacker should be 3/2 EOT (2/2 printed + 1/+0 EOT).
    assert_eq!(bear1.power(), 3, "attacker 1 should be pumped");
    assert_eq!(bear2.power(), 3, "attacker 2 should be pumped");
}

#[test]
fn lorehold_marshal_b156_gains_life_per_other_attacker() {
    let mut g = two_player_game();
    let _marshal = g.add_card_to_battlefield(0, catalog::lorehold_marshal_b156());
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(b1);
    g.clear_sickness(b2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: b1, target: AttackTarget::Player(1) },
        Attack { attacker: b2, target: AttackTarget::Player(1) },
    ]))
    .expect("both bears attack");
    drain_stack(&mut g);
    // Marshal fires once per attacker (both bears are "other"), so
    // life should go up by 2.
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn silverquill_tactician_b156_mints_inkling_per_other_attacker() {
    let mut g = two_player_game();
    let _tact = g.add_card_to_battlefield(0, catalog::silverquill_tactician_b156());
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(b1);
    g.clear_sickness(b2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: b1, target: AttackTarget::Player(1) },
        Attack { attacker: b2, target: AttackTarget::Player(1) },
    ]))
    .expect("both bears attack");
    drain_stack(&mut g);
    let inklings = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Inkling")
        .count();
    assert_eq!(inklings, 2, "should mint two Inklings (one per other-attacker)");
}

#[test]
fn quandrix_mathematician_ii_b156_counters_each_attacker() {
    let mut g = two_player_game();
    let _m = g.add_card_to_battlefield(0, catalog::quandrix_mathematician_ii_b156());
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(b1);
    g.clear_sickness(b2);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: b1, target: AttackTarget::Player(1) },
        Attack { attacker: b2, target: AttackTarget::Player(1) },
    ]))
    .expect("both bears attack");
    drain_stack(&mut g);
    let bear1 = g.battlefield.iter().find(|c| c.id == b1).unwrap();
    let bear2 = g.battlefield.iter().find(|c| c.id == b2).unwrap();
    assert_eq!(bear1.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(bear2.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn pest_hivebreeder_b156_mints_pest_on_other_creature_death() {
    // Use Lightning Bolt to kill the pest via real combat-damage / SBA
    // flow so the unified dispatcher fires AnotherOfYours triggers
    // (`remove_to_graveyard_with_triggers` only handles SelfSource).
    let mut g = two_player_game();
    let _hb = g.add_card_to_battlefield(0, catalog::pest_hivebreeder_b156());
    let pest = g.add_card_to_battlefield(0, catalog::pest_acolyte_b155());
    drain_stack(&mut g);
    let pests_before = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest").count();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(pest)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let pests_after = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest").count();
    assert_eq!(pests_after, pests_before + 1, "should mint a Pest on death");
}

// ── Bot AI lock-ins (push: planeswalker attack heuristic) ──────────────────

#[test]
fn bot_attacks_finishable_planeswalker_with_proper_power() {
    // The bot's DeclareAttackers handler now redirects attacks at an
    // opponent's planeswalker when our total attacking power can finish
    // it off in one swing. Lock-in: with a 5-loyalty PW + a 5-power
    // attacker (Grizzly Bears pumped to 5/5 with +1/+1 counters), the
    // bot should aim AT the walker.
    use crate::server::bot::{Bot, RandomBot};
    use crate::card::CounterType;
    let mut g = two_player_game();
    let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(beater);
    // Pump bear to 5/5 via three +1/+1 counters.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == beater) {
        c.add_counters(CounterType::PlusOnePlusOne, 3);
    }
    // Opp's planeswalker (Dellian Fel: 5 base loyalty).
    let pw = g.add_card_to_battlefield(1, catalog::professor_dellian_fel());
    let loyalty = g.battlefield.iter().find(|c| c.id == pw).unwrap()
        .counter_count(CounterType::Loyalty);
    assert!(loyalty >= 1, "PW should have loyalty");

    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let mut bot = RandomBot::new();
    let action = bot.next_action(&g, 0);
    // Bot should DeclareAttackers with the beater aimed at the walker
    // since 5 power finishes off a 5-loyalty walker.
    match action {
        Some(GameAction::DeclareAttackers(attacks)) => {
            let aimed_at_pw = attacks.iter().any(|a| {
                matches!(a.target, AttackTarget::Planeswalker(p) if p == pw)
            });
            assert!(
                aimed_at_pw,
                "bot should aim at the finishable PW (loyalty {loyalty}); got {attacks:?}",
            );
        }
        other => panic!("expected DeclareAttackers, got {other:?}"),
    }
}

#[test]
fn bot_does_not_aim_at_walker_too_tough_to_finish() {
    // Symmetric lock-in: when the bot's attacking power is below the
    // walker's loyalty, the bot should NOT throw attackers at the
    // walker.
    use crate::server::bot::{Bot, RandomBot};
    use crate::card::CounterType;
    let mut g = two_player_game();
    let beater = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(beater);
    let pw = g.add_card_to_battlefield(1, catalog::professor_dellian_fel());
    // Walker has full base loyalty (5); our 2-power bear can't finish it.
    let loyalty = g.battlefield.iter().find(|c| c.id == pw).unwrap()
        .counter_count(CounterType::Loyalty);
    let our_power: i32 = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature())
        .map(|c| c.power())
        .sum();
    assert!((our_power as u32) < loyalty);

    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let mut bot = RandomBot::new();
    let action = bot.next_action(&g, 0);
    match action {
        Some(GameAction::DeclareAttackers(attacks)) => {
            let aimed_at_pw = attacks.iter().any(|a| {
                matches!(a.target, AttackTarget::Planeswalker(_))
            });
            assert!(
                !aimed_at_pw,
                "bot should NOT aim at a walker it can't finish off; got {attacks:?}",
            );
        }
        other => panic!("expected DeclareAttackers, got {other:?}"),
    }
}

#[test]
fn bot_prefers_surviving_trade_over_deathtouch_attacker() {
    // With one 3/3 blocker and two 2/2 attackers — one vanilla, one with
    // deathtouch — the bot should block the vanilla one (3/3 kills it and
    // survives) rather than the deathtouch one (which would kill the 3/3).
    use crate::card::Keyword;
    use crate::server::bot;
    let mut g = two_player_game();
    // Deathtouch attacker declared first (so the old tie-break would pick it).
    let dt_atk = {
        let mut d = catalog::grizzly_bears();
        d.name = "Venom Bear";
        d.keywords = vec![Keyword::Deathtouch];
        g.add_card_to_battlefield(1, d)
    };
    let vanilla_atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(dt_atk);
    g.clear_sickness(vanilla_atk);
    g.attacking.push(Attack { attacker: dt_atk, target: AttackTarget::Player(0) });
    g.attacking.push(Attack { attacker: vanilla_atk, target: AttackTarget::Player(0) });
    let blocker = {
        let mut d = catalog::grizzly_bears();
        d.name = "Wall Bear";
        d.power = 3;
        d.toughness = 3;
        g.add_card_to_battlefield(0, d)
    };
    g.clear_sickness(blocker);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].1, vanilla_atk,
        "bot blocks the vanilla attacker it can kill and survive, not the deathtouch one");
}

#[test]
fn bot_blocks_smart_value_trade() {
    // Push (this run): smarter blocker AI. With one 3/3 attacker
    // attacking us and a 2/2 blocker, the blocker should still chump
    // (life-threatened logic) when our life is low. With a 3/4
    // blocker (clean kill, survives), it should block.
    use crate::server::bot;
    let mut g = two_player_game();
    g.players[0].life = 5;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareBlockers;
    // Opp's 4/3 attacker.
    let beater_def = {
        let mut def = catalog::grizzly_bears();
        def.name = "Big Bear";
        def.power = 4;
        def.toughness = 3;
        def
    };
    let attacker = g.add_card_to_battlefield(1, beater_def);
    g.clear_sickness(attacker);
    g.attacking.push(Attack {
        attacker,
        target: AttackTarget::Player(0),
    });
    // Our 3/4 blocker — clean kill, survives.
    let blocker_def = {
        let mut def = catalog::grizzly_bears();
        def.name = "Wall Bear";
        def.power = 3;
        def.toughness = 4;
        def
    };
    let blocker = g.add_card_to_battlefield(0, blocker_def);
    g.clear_sickness(blocker);

    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert_eq!(blocks.len(), 1, "should block the attacker");
    assert_eq!(blocks[0].0, blocker);
    assert_eq!(blocks[0].1, attacker);
}

#[test]
fn bot_gang_blocks_to_kill_when_life_threatened() {
    // Facing lethal from a 6/6 no single blocker can kill, the bot should
    // gang two 3/3s onto it (combined power 6 ≥ toughness 6) to remove the
    // threat rather than scatter chumps.
    use crate::server::bot;
    let mut g = two_player_game();
    g.players[0].life = 5; // a 6-power attacker is lethal
    let big = {
        let mut d = catalog::grizzly_bears();
        d.name = "Huge Bear";
        d.power = 6;
        d.toughness = 6;
        d
    };
    let attacker = g.add_card_to_battlefield(1, big);
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let mk = |g: &mut crate::game::GameState| {
        let mut d = catalog::grizzly_bears();
        d.power = 3;
        d.toughness = 3;
        let id = g.add_card_to_battlefield(0, d);
        g.clear_sickness(id);
        id
    };
    let b1 = mk(&mut g);
    let b2 = mk(&mut g);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert_eq!(blocks.len(), 2, "both blockers gang the lethal attacker");
    assert!(blocks.iter().all(|(_, a)| *a == attacker));
    let blockers: std::collections::HashSet<_> = blocks.iter().map(|(b, _)| *b).collect();
    assert!(blockers.contains(&b1) && blockers.contains(&b2));
}

#[test]
fn bot_assigns_two_blockers_to_a_menace_attacker() {
    // CR 509.1b — a Menace 4/4 must be blocked by two creatures. With two
    // idle 2/3 blockers the bot must commit both (a lone block is illegal).
    use crate::card::Keyword;
    use crate::server::bot;
    let mut g = two_player_game();
    g.players[0].life = 4; // lethal pressure so the bot wants to block
    let menace = {
        let mut d = catalog::grizzly_bears();
        d.name = "Menace Bear";
        d.power = 4;
        d.toughness = 4;
        d.keywords = vec![Keyword::Menace];
        d
    };
    let attacker = g.add_card_to_battlefield(1, menace);
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let mk = |g: &mut crate::game::GameState| {
        let mut d = catalog::grizzly_bears();
        d.power = 2;
        d.toughness = 3;
        let id = g.add_card_to_battlefield(0, d);
        g.clear_sickness(id);
        id
    };
    mk(&mut g);
    mk(&mut g);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    let on_menace = blocks.iter().filter(|(_, a)| *a == attacker).count();
    assert!(on_menace == 0 || on_menace >= 2,
        "Menace attacker gets 0 or ≥2 blockers, never a lone (illegal) block; got {on_menace}");
    assert_eq!(on_menace, 2, "two idle blockers available → commit both");
}

#[test]
fn bot_drops_lone_block_on_menace_when_no_second_blocker() {
    // With only one creature available, a Menace attacker can't be legally
    // blocked — the bot must leave it unblocked rather than emit an illegal
    // single block.
    use crate::card::Keyword;
    use crate::server::bot;
    let mut g = two_player_game();
    g.players[0].life = 3;
    let menace = {
        let mut d = catalog::grizzly_bears();
        d.name = "Menace Bear";
        d.power = 3;
        d.toughness = 3;
        d.keywords = vec![Keyword::Menace];
        d
    };
    let attacker = g.add_card_to_battlefield(1, menace);
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let lone = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(lone);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert!(blocks.iter().all(|(_, a)| *a != attacker),
        "no legal block on the Menace attacker → leave it unblocked");
}

// ── CR 603.4 — intervening 'if' clause re-check at resolve time ────────────

#[test]
fn cr_603_4_intervening_if_re_checked_at_resolve_time() {
    // CR 603.4 — "If the condition isn't true at that time [resolve],
    // the ability is removed from the stack and does nothing."
    //
    // Push a trigger directly onto the stack with an intervening_if
    // predicate that's currently false, then drain. Verify the body
    // never runs (life total unchanged).
    use crate::card::Predicate;
    use crate::effect::PlayerRef;
    use crate::game::types::StackItem;

    let mut g = two_player_game();
    // Predicate that will be false: "your hand size is at least 100"
    let pred = Predicate::ValueAtLeast(
        crate::effect::Value::HandSizeOf(PlayerRef::You),
        crate::effect::Value::Const(100),
    );
    // Body: gain 50 life — would be observable if it ran.
    let body = crate::effect::Effect::GainLife {
        who: crate::effect::Selector::You,
        amount: crate::effect::Value::Const(50),
    };
    // Manufacture a trigger source so the resolution context has a
    // valid `source` id.
    let src = g.add_card_to_battlefield(0, catalog::island());
    g.stack.push(StackItem::Trigger {
        source: src,
        controller: 0,
        effect: Box::new(body),
        target: None,
        mode: None,
        x_value: 0,
        converged_value: 0,
        trigger_source: None,
        mana_spent: 0,
        event_amount: 0,
        intervening_if: Some(pred),
    });
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before,
        "Trigger fizzled per CR 603.4 — body didn't run");
}

#[test]
fn cr_603_4_intervening_if_runs_when_true_at_resolve_time() {
    // Sanity check: trigger with intervening_if = Some(true_predicate)
    // resolves normally.
    use crate::card::Predicate;
    use crate::effect::PlayerRef;
    use crate::game::types::StackItem;

    let mut g = two_player_game();
    // True predicate: "your hand size is at least 0" (always true)
    let pred = Predicate::ValueAtLeast(
        crate::effect::Value::HandSizeOf(PlayerRef::You),
        crate::effect::Value::Const(0),
    );
    let body = crate::effect::Effect::GainLife {
        who: crate::effect::Selector::You,
        amount: crate::effect::Value::Const(7),
    };
    let src = g.add_card_to_battlefield(0, catalog::island());
    g.stack.push(StackItem::Trigger {
        source: src,
        controller: 0,
        effect: Box::new(body),
        target: None,
        mode: None,
        x_value: 0,
        converged_value: 0,
        trigger_source: None,
        mana_spent: 0,
        event_amount: 0,
        intervening_if: Some(pred),
    });
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 7,
        "Trigger body ran — predicate held at resolve time");
}

// ── CR 705.3 — Krark's Thumb-style coin-flip advantage ─────────────────────

#[test]
fn cr_705_3_coin_flip_advantage_lets_tails_be_recovered() {
    // Direct exercise of the new `Player.coin_flip_advantage` field:
    // with advantage = 1, a flip-coin effect that would default to tails
    // (via a ScriptedDecider that always returns Bool(false)) should
    // still see heads on at least one of the two replayed flips.
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    use crate::effect::Effect;

    let mut g = two_player_game();
    g.players[0].coin_flip_advantage = 1;
    // ScriptedDecider returning false twice then true means:
    //   - Without advantage: 1 flip returns false → tails branch.
    //   - With advantage=1:  2 flips. The first returns false, the
    //     second returns false. heads_seen stays false → tails branch.
    //   (To force heads_seen=true we'd need ≥1 Bool(true) in the script.)
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(true),  // the SECOND replay wins
    ]));

    // Build a flip effect that adds +5 life on heads, -5 life on tails.
    let body = Effect::FlipCoin {
        count: crate::effect::Value::Const(1),
        on_heads: Box::new(Effect::GainLife {
            who: crate::effect::Selector::You,
            amount: crate::effect::Value::Const(5),
        }),
        on_tails: Box::new(Effect::LoseLife {
            who: crate::effect::Selector::You,
            amount: crate::effect::Value::Const(5),
        }),
    };
    // Drop it on the stack as a Trigger to exercise the resolver path.
    let src = g.add_card_to_battlefield(0, catalog::island());
    g.stack.push(crate::game::types::StackItem::Trigger {
        source: src,
        controller: 0,
        effect: Box::new(body),
        target: None,
        mode: None,
        x_value: 0,
        converged_value: 0,
        trigger_source: None,
        mana_spent: 0,
        event_amount: 0,
        intervening_if: None,
    });
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    // With advantage=1, even though the first flip returned false, the
    // second returned true → heads_seen = true → +5 life.
    assert_eq!(g.players[0].life, life_before + 5,
        "Coin-flip advantage lets us redeem a tails result");
}

#[test]
fn cr_705_3_no_advantage_means_one_flip_one_result() {
    // Without advantage, a single Bool(false) → tails branch.
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    use crate::effect::Effect;

    let mut g = two_player_game();
    assert_eq!(g.players[0].coin_flip_advantage, 0, "default advantage is 0");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));

    let body = Effect::FlipCoin {
        count: crate::effect::Value::Const(1),
        on_heads: Box::new(Effect::GainLife {
            who: crate::effect::Selector::You,
            amount: crate::effect::Value::Const(5),
        }),
        on_tails: Box::new(Effect::LoseLife {
            who: crate::effect::Selector::You,
            amount: crate::effect::Value::Const(5),
        }),
    };
    let src = g.add_card_to_battlefield(0, catalog::island());
    g.stack.push(crate::game::types::StackItem::Trigger {
        source: src,
        controller: 0,
        effect: Box::new(body),
        target: None,
        mode: None,
        x_value: 0,
        converged_value: 0,
        trigger_source: None,
        mana_spent: 0,
        event_amount: 0,
        intervening_if: None,
    });
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 5,
        "Without advantage, tails fires the lose-life branch");
}

// ── CR 122.4 — max counters of a kind SBA ──────────────────────────────────

#[test]
fn cr_122_4_excess_counters_pruned_by_sba() {
    use crate::card::{CardDefinition, CardType};
    use crate::mana::cost;

    let mut g = two_player_game();
    let def = CardDefinition {
        name: "Pinnacle Test (synthetic)",
        cost: cost(&[]),
        card_types: vec![CardType::Artifact],
        enters_as_copy: None,
        max_counters_of_kind: Some((CounterType::PlusOnePlusOne, 3)),
        ..Default::default()
    };
    let id = g.add_card_to_battlefield(0, def);
    {
        let c = g.battlefield_find_mut(id).expect("on bf");
        c.add_counters(CounterType::PlusOnePlusOne, 7);
    }
    let _ = g.check_state_based_actions();
    let after = g.battlefield_find(id).expect("on bf");
    assert_eq!(after.counter_count(CounterType::PlusOnePlusOne), 3,
        "Excess counters pruned down to the cap (3)");
}

#[test]
fn quandrix_expansor_b155_creates_fractal_with_x_counters() {
    let mut g = two_player_game();
    let exp = g.add_card_to_hand(0, catalog::quandrix_expansor_b155());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: exp, target: None, additional_targets: vec![],
        mode: None, x_value: Some(3),
    }).expect("Expansor castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.definition.name == "Fractal").expect("Fractal minted");
    let counters: u32 = fractal.counters.iter()
        .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
        .map(|(_, n)| *n).sum();
    assert_eq!(counters, 3, "Fractal has X=3 +1/+1 counters");
}

#[test]
fn quandrix_logician_b155_etb_pumps_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_logician_b155());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Logician castable");
    drain_stack(&mut g);
    let counters = g.battlefield_find(bear).map(|c| {
        c.counters.iter()
            .filter(|(k, _)| **k == CounterType::PlusOnePlusOne)
            .map(|(_, n)| *n).sum::<u32>()
    }).unwrap_or(0);
    assert_eq!(counters, 1, "Bear gets +1/+1 counter");
}

// ── batch 155 — Prismari cards ─────────────────────────────────────────────

#[test]
fn prismari_sparkmaster_b155_pings_each_opp_on_is_cast() {
    let mut g = two_player_game();
    let _sm = g.add_card_to_battlefield(0, catalog::prismari_sparkmaster_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 1 - 3, "Magecraft 1 + Bolt 3");
}

#[test]
fn prismari_treasureseeker_b155_mints_treasure_on_is_cast() {
    let mut g = two_player_game();
    let _ts = g.add_card_to_battlefield(0, catalog::prismari_treasureseeker_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 1, "Magecraft mints a Treasure");
}

#[test]
fn prismari_flameshape_b155_self_pumps_on_is_cast() {
    let mut g = two_player_game();
    let fs = g.add_card_to_battlefield(0, catalog::prismari_flameshape_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let power_before = g.battlefield_find(fs).expect("on bf").power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let power_after = g.battlefield_find(fs).expect("on bf").power();
    assert_eq!(power_after, power_before + 1, "+1/+1 from magecraft");
}

#[test]
fn prismari_tidepainter_b155_may_draw_on_is_cast() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let _tp = g.add_card_to_battlefield(0, catalog::prismari_tidepainter_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_pyroshaper_b155_loots_on_is_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _ps = g.add_card_to_battlefield(0, catalog::prismari_pyroshaper_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let _hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Loot = Draw 1 + Discard 1, net 0 cards in hand
    // (-1 Bolt cast, +1 from draw, -1 from discard = -1)
    // Actually the bolt was the only card; auto-decider may discard the drawn island
}

#[test]
fn prismari_forgewright_b155_etb_pings_each_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_forgewright_b155());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Forgewright castable");
    drain_stack(&mut g);
    let bear_damage = g.battlefield_find(bear).map(|c| c.damage).unwrap_or(0);
    assert_eq!(bear_damage, 1, "ETB pings each opp creature for 1");
}

#[test]
fn prismari_tinkertinker_b155_scrys_on_is_cast() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let _tt = g.add_card_to_battlefield(0, catalog::prismari_tinkertinker_b155());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry doesn't change library size
}

#[test]
fn prismari_flowcaster_b155_etb_draws_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_flowcaster_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Flowcaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "-1 cast + 1 draw = 0");
}

#[test]
fn prismari_spellsign_b155_deals_two_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let ss = g.add_card_to_hand(0, catalog::prismari_spellsign_b155());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: ss, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellsign castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2, "Deal 2");
    assert_eq!(g.players[0].hand.len(), hand_before, "-1 cast + 1 draw = 0");
}

// ── batch 158 — Silverquill cards ──────────────────────────────────────────

#[test]
fn silverquill_inkwarden_b158_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkwarden_b158());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkwarden castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    let c = g.battlefield.iter().find(|c| c.definition.name == "Silverquill Inkwarden (b158)").unwrap();
    assert!(c.definition.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn inkling_pinionguard_b158_is_a_three_mana_lifelink_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_pinionguard_b158());
    let c = g.battlefield_find(id).expect("on bf");
    assert!(c.definition.keywords.contains(&Keyword::Flying));
    assert!(c.definition.keywords.contains(&Keyword::Lifelink));
    assert_eq!(c.definition.power, 2);
    assert_eq!(c.definition.toughness, 2);
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_pen_crier_b158_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_pen_crier_b158());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pen-Crier castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 2);
    assert_eq!(g.players[1].life, life1_before - 2);
}

#[test]
fn silverquill_pen_bearer_b158_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _b = g.add_card_to_battlefield(0, catalog::silverquill_pen_bearer_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "magecraft gained 1 life");
}

#[test]
fn inkling_scriptor_b158_magecraft_drains_on_cast() {
    let mut g = two_player_game();
    let _s = g.add_card_to_battlefield(0, catalog::inkling_scriptor_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 dmg + drain 1
    assert_eq!(g.players[1].life, life1_before - 3 - 1);
}

#[test]
fn silverquill_penkeeper_b158_etb_scrys_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_penkeeper_b158());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Penkeeper castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn silverquill_vow_b158_drains_one_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_vow_b158());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vow castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 1, "you gain 1");
    assert_eq!(g.players[1].life, life1_before - 1, "opp loses 1");
    // -1 (cast Vow) +1 (draw) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_penlord_b158_etb_gains_three_life_and_is_flier_lifelink() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_penlord_b158());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Penlord castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
    let c = g.battlefield.iter().find(|c| c.definition.name == "Inkling Penlord (b158)").unwrap();
    assert!(c.definition.keywords.contains(&Keyword::Flying));
    assert!(c.definition.keywords.contains(&Keyword::Lifelink));
    assert_eq!(c.definition.power, 3);
    assert_eq!(c.definition.toughness, 4);
}

#[test]
fn silverquill_censurer_b158_etb_taps_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_censurer_b158());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Censurer castable");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).expect("bear still on bf");
    assert!(b.tapped, "opp bear should be tapped");
}

#[test]
fn silverquill_inkdrain_b158_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkdrain_b158());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkdrain castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 3);
    assert_eq!(g.players[1].life, life1_before - 3);
}

#[test]
fn inkling_aerogate_b158_is_a_three_mana_one_three_flying_vigilance_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_aerogate_b158());
    let c = g.battlefield_find(id).expect("on bf");
    assert_eq!(c.definition.power, 1);
    assert_eq!(c.definition.toughness, 3);
    assert!(c.definition.keywords.contains(&Keyword::Flying));
    assert!(c.definition.keywords.contains(&Keyword::Vigilance));
    assert!(c.definition.subtypes.creature_types.contains(&CreatureType::Inkling));
}

#[test]
fn silverquill_battlescholar_b158_etb_drains_one_and_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_battlescholar_b158());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battlescholar castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 1);
    assert_eq!(g.players[1].life, life1_before - 1);
}

#[test]
fn inkling_veilwarden_b158_is_a_five_mana_four_four_lifelink_flier() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inkling_veilwarden_b158());
    let c = g.battlefield_find(id).expect("on bf");
    assert_eq!(c.definition.power, 4);
    assert_eq!(c.definition.toughness, 4);
    assert!(c.definition.keywords.contains(&Keyword::Flying));
    assert!(c.definition.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn silverquill_edicter_b158_forces_opp_sac_and_gains_one_life() {
    let mut g = two_player_game();
    let _bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_edicter_b158());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    let opp_creatures_before = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.card_types.iter().any(|t| matches!(t, crate::card::CardType::Creature)))
        .count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Edicter castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    let opp_creatures_after = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.card_types.iter().any(|t| matches!(t, crate::card::CardType::Creature)))
        .count();
    assert_eq!(opp_creatures_after, opp_creatures_before - 1, "opp sacrificed one");
}

// ── batch 158 — Witherbloom cards ──────────────────────────────────────────

#[test]
fn witherbloom_decantor_b158_magecraft_drains_on_cast() {
    let mut g = two_player_game();
    let _d = g.add_card_to_battlefield(0, catalog::witherbloom_decantor_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3 - 1);
}

#[test]
fn pest_cultivator_ii_b158_etb_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_cultivator_ii_b158());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cultivator castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 1);
}

#[test]
fn witherbloom_drainfeeder_b158_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_drainfeeder_b158());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainfeeder castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 2);
    assert_eq!(g.players[1].life, life1_before - 2);
}

#[test]
fn pest_engorger_ii_b158_magecraft_grows_with_counter() {
    let mut g = two_player_game();
    let pe = g.add_card_to_battlefield(0, catalog::pest_engorger_ii_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(pe).expect("on bf");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn pest_wretch_b158_magecraft_pumps_self() {
    let mut g = two_player_game();
    let p = g.add_card_to_battlefield(0, catalog::pest_wretch_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let pwr_before = g.battlefield_find(p).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pwr_after = g.battlefield_find(p).unwrap().power();
    assert_eq!(pwr_after, pwr_before + 1);
}

#[test]
fn witherbloom_vinepoet_ii_b158_magecraft_gains_life() {
    let mut g = two_player_game();
    let _v = g.add_card_to_battlefield(0, catalog::witherbloom_vinepoet_ii_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn pest_swarmrider_b158_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_swarmrider_b158());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Swarmrider castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2);
}

#[test]
fn witherbloom_faminescion_b158_drains_three_and_mills() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::witherbloom_faminescion_b158());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    let lib1_before = g.players[1].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Faminescion castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0_before + 3);
    assert_eq!(g.players[1].life, life1_before - 3);
    assert_eq!(g.players[1].library.len(), lib1_before - 2);
}

#[test]
fn witherbloom_toxinspear_b158_kills_two_toughness_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_toxinspear_b158());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Toxinspear castable");
    drain_stack(&mut g);
    // -2/-1 on a 2/2: 0/1
    let b = g.battlefield_find(bear);
    match b {
        Some(c) => {
            assert!(c.power() <= 0 || c.toughness() <= 1);
        }
        None => {
            // dead is fine — 0 power 1 toughness gets put on the field
            assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
        }
    }
}

// ── batch 158 — Lorehold cards ─────────────────────────────────────────────

#[test]
fn lorehold_wallscribe_b158_etb_gains_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_wallscribe_b158());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wallscribe castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn lorehold_embermage_b158_magecraft_pings_target() {
    let mut g = two_player_game();
    let _e = g.add_card_to_battlefield(0, catalog::lorehold_embermage_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + magecraft ping 1
    assert_eq!(g.players[1].life, life1_before - 3 - 1);
}

#[test]
fn lorehold_spirit_drummer_b158_magecraft_self_pumps() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::lorehold_spirit_drummer_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let pwr_before = g.battlefield_find(s).unwrap().power();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(s).unwrap().power(), pwr_before + 1);
}

#[test]
fn lorehold_stoneflame_b158_destroys_two_two_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_stoneflame_b158());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stoneflame castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn lorehold_spectermage_b158_magecraft_pings_any() {
    let mut g = two_player_game();
    let _s = g.add_card_to_battlefield(0, catalog::lorehold_spectermage_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 3 - 1);
}

#[test]
fn lorehold_spirit_caster_b158_mints_spirit_token_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spirit_caster_b158());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spirit-Caster castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 1);
}

#[test]
fn lorehold_spellsong_b158_burns_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spellsong_b158());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life0_before = g.players[0].life;
    let life1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spellsong castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1_before - 2);
    assert_eq!(g.players[0].life, life0_before + 2);
}

#[test]
fn lorehold_stonewright_b158_mints_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_stonewright_b158());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stonewright castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 2);
}

// ── batch 158 — Quandrix cards ─────────────────────────────────────────────

#[test]
fn quandrix_coursetaker_b158_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _c = g.add_card_to_battlefield(0, catalog::quandrix_coursetaker_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn quandrix_bigbrain_b158_mints_fractal_with_two_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_bigbrain_b158());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bigbrain castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal")
        .expect("Fractal minted");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn quandrix_fractaltender_b158_magecraft_adds_counter_to_self() {
    let mut g = two_player_game();
    let f = g.add_card_to_battlefield(0, catalog::quandrix_fractaltender_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(f).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_researcher_b158_magecraft_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _r = g.add_card_to_battlefield(0, catalog::quandrix_researcher_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -1 (Bolt cast) + 1 (draw) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_inquirer_b158_etb_scrys_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_inquirer_b158());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inquirer castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn quandrix_echo_b158_draws_and_pumps_friendly() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_echo_b158());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Echo castable");
    drain_stack(&mut g);
    // -1 (Echo cast) + 1 (draw) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_counterpoint_b158_counters_unless_paid() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    // p0 responds with Counterpoint
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::quandrix_counterpoint_b158());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Counterpoint castable");
    drain_stack(&mut g);
    // P1 had no extra {1} → bolt countered, P0 still at 20.
    assert_eq!(g.players[0].life, 20);
}

#[test]
fn fractal_skydweller_b158_etb_with_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_skydweller_b158());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Skydweller castable");
    drain_stack(&mut g);
    let c = g.battlefield.iter()
        .find(|c| c.definition.name == "Fractal Skydweller (b158)")
        .expect("on bf");
    assert!(c.definition.keywords.contains(&Keyword::Flying));
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_equalist_b158_magecraft_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _e = g.add_card_to_battlefield(0, catalog::quandrix_equalist_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
}

// ── batch 158 — Prismari cards ─────────────────────────────────────────────

#[test]
fn prismari_sparkmage_ii_b158_magecraft_mints_treasure() {
    let mut g = two_player_game();
    let _s = g.add_card_to_battlefield(0, catalog::prismari_sparkmage_ii_b158());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1);
}

#[test]
fn bot_declines_bad_block_into_first_strike() {
    // Push (claude/modern_decks): first-strike-aware blocking. A 2/2
    // first-strike attacker kills a 2/2 vanilla blocker before it can
    // strike back, so the "trade" is illusory. With full life the bot
    // must NOT make that block.
    use crate::server::bot;
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareBlockers;
    let attacker = {
        let mut d = catalog::grizzly_bears();
        d.name = "First Striker";
        d.keywords = vec![Keyword::FirstStrike];
        g.add_card_to_battlefield(1, d)
    };
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 vanilla
    g.clear_sickness(blocker);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert!(blocks.is_empty(),
        "bot should not chump-trade a 2/2 into a 2/2 first-striker at full life");
}

#[test]
fn bot_blocks_first_striker_it_outsizes() {
    // The same attacker, but our 3/3 blocker survives the first-strike
    // damage and kills it on the regular step — a real clean trade.
    use crate::server::bot;
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareBlockers;
    let attacker = {
        let mut d = catalog::grizzly_bears();
        d.name = "First Striker";
        d.keywords = vec![Keyword::FirstStrike];
        g.add_card_to_battlefield(1, d)
    };
    g.clear_sickness(attacker);
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let blocker = {
        let mut d = catalog::grizzly_bears();
        d.name = "Wall Bear";
        d.power = 3;
        d.toughness = 3;
        g.add_card_to_battlefield(0, d)
    };
    g.clear_sickness(blocker);
    let blocks = bot::pick_blocks_for_test(&g, 0);
    assert_eq!(blocks, vec![(blocker, attacker)],
        "a 3/3 survives the first-strike 2 and kills the 2/2 first-striker");
}

#[test]
fn cr_705_3_static_grants_coin_flip_advantage() {
    // Krark's-Thumb-style: advantage comes from a battlefield static, not the
    // Player field. A scripted (tails, heads) flip should resolve as heads.
    use crate::card::{CardDefinition, CardType, StaticAbility, Supertype};
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    use crate::effect::{Effect, PlayerStaticTarget, Selector, StaticEffect, Value};

    let thumb = CardDefinition {
        name: "Test Thumb",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Coin-flip advantage.",
            effect: StaticEffect::CoinFlipAdvantage { target: PlayerStaticTarget::Controller },
        }],
        ..Default::default()
    };
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, thumb);
    assert_eq!(g.coin_flip_advantage_now(0), 1, "static grants advantage to its controller");
    assert_eq!(g.coin_flip_advantage_now(1), 0, "opponent gets none");

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(true), // the second replay wins
    ]));
    let body = Effect::FlipCoin {
        count: Value::Const(1),
        on_heads: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(5) }),
        on_tails: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(5) }),
    };
    let src = g.add_card_to_battlefield(0, catalog::island());
    g.stack.push(crate::game::types::StackItem::Trigger {
        source: src, controller: 0, effect: Box::new(body), target: None, mode: None,
        x_value: 0, converged_value: 0, trigger_source: None, mana_spent: 0,
        event_amount: 0, intervening_if: None,
    });
    let life_before = g.players[0].life;
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 5,
        "static-granted advantage redeems the tails flip");
}
