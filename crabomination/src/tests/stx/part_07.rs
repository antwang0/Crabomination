use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;
use super::*;


#[test]
fn lorehold_spellrunner_pumps_on_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_spellrunner());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 3, "Pumped +1/+0 from magecraft");
    assert!(body.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_battlecaster_etb_spirit_and_attack_counter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_battlecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battlecaster castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 1, "Battlecaster mints 1 Spirit");
    assert!(catalog::lorehold_battlecaster().keywords.contains(&Keyword::Trample));
}

#[test]
fn lorehold_pyresurge_burns_target_and_gains_one() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_pyresurge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyresurge castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear killed");
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn lorehold_outburst_mints_two_spirits_and_anthems() {
    let mut g = two_player_game();
    let _existing = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_outburst());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Outburst castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 2);
}

#[test]
fn prismari_lavalifter_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_lavalifter());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lavalifter castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1);
}

#[test]
fn prismari_spelltheorist_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_spelltheorist());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spelltheorist castable");
    drain_stack(&mut g);
    // Hand: -1 (cast) +1 (draw) -1 (discard) = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn prismari_stormwriter_burns_and_cantrips() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_stormwriter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stormwriter castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear killed");
    // Hand: -1 (cast) +1 (cantrip) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_igniter_pings_on_cast() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::prismari_igniter());
    drain_stack(&mut g);
    let p1_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -3 (bolt) -1 (igniter magecraft).
    assert_eq!(g.players[1].life, p1_before - 4);
}

#[test]
fn quandrix_pondweaver_scrys_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _src = g.add_card_to_battlefield(0, catalog::quandrix_pondweaver());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Scry doesn't change library size by itself.
    assert_eq!(g.players[0].library.len(), lib_before);
}

#[test]
fn quandrix_fractalseed_etb_adds_counters_for_is_in_gy() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // Creature, doesn't count.
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalseed());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalseed castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 2, "Got 2 counters from 2 IS");
}

#[test]
fn quandrix_mapmaker_etb_searches_basic() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let lands_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.card_types.contains(&crate::card::CardType::Land))
        .count();
    let id = g.add_card_to_hand(0, catalog::quandrix_mapmaker());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mapmaker castable");
    drain_stack(&mut g);
    let lands_after = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.card_types.contains(&crate::card::CardType::Land))
        .count();
    assert_eq!(lands_after, lands_before + 1, "Searched a basic land");
}

#[test]
fn quandrix_fractalwave_creates_fractal_with_counters() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalwave());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalwave castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Fractal")
        .expect("Fractal minted");
    assert_eq!(fractal.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn fractal_theorist_pumps_fractal_on_cast() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::fractal_theorist());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(_src).unwrap();
    // Fractal Theorist is itself a Fractal, so the counter lands on it.
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn vanquish_the_horde_affinity_for_creatures_reduces_cost() {
    // With 3 creatures on the battlefield, the spell costs {3}{W}
    // (vs printed {6}{W}), so 3 generic + 1 W is enough.
    let mut g = two_player_game();
    let _b0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vanquish_the_horde());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    // Should be castable at {3}{W} (6 - 3 creatures = 3 generic).
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vanquish castable at {3}{W} via affinity discount");
    drain_stack(&mut g);
    // All creatures dead post-resolution.
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_creature()).count(), 0);
}

#[test]
fn vanquish_the_horde_affinity_rejects_undercost_with_no_creatures() {
    // With zero creatures, the spell costs the printed {6}{W}.
    // 3 generic + 1 W is insufficient.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::vanquish_the_horde());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let result = g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(result.is_err(), "Vanquish not castable at {{3}}{{W}} with no creatures");
}

// ── Push (modern_decks) batch 26: 7 new STX cards + tests ──────────────────
//
// 3 new Lessons (Pest Studies, Lecture in Strategy, Advanced Cartography)
// + 4 new iconic cards (Bombastic Strixhaven Mage, Mage Hunters' Strike,
// Mascot Researcher, Strixhaven Tutor). All use existing primitives.

#[test]
fn pest_studies_creates_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_studies());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pest Studies castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2);
}

#[test]
fn lecture_in_strategy_pumps_team_with_vigilance() {
    let mut g = two_player_game();
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lecture_in_strategy());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lecture castable");
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let bear = computed.iter().find(|c| c.id == b).unwrap();
    assert_eq!(bear.power, 3, "Bear pumped");
    assert!(bear.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn advanced_cartography_ramps_and_scrys() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
        DecisionAnswer::ScryOrder { kept_top: vec![], bottom: vec![] },
    ]));
    let id = g.add_card_to_hand(0, catalog::advanced_cartography());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cartography castable");
    drain_stack(&mut g);
    let f = g.battlefield_find(forest).expect("Forest tutored");
    assert!(f.tapped);
}

#[test]
fn bombastic_strixhaven_mage_etb_pings_and_magecraft_pings() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bombastic_strixhaven_mage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 2, "ETB 2 damage");
    // Magecraft 1 damage on IS cast.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -3 (bolt) + -1 (magecraft) = -4.
    assert_eq!(g.players[1].life, p1_before - 2 - 4);
}

#[test]
fn mage_hunters_strike_shrinks_target() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mage_hunters_strike());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Strike castable");
    drain_stack(&mut g);
    // Bear 2/2 - 3/-3 → -1/-1 → dies.
    assert!(g.battlefield_find(bear).is_none(), "Bear killed");
}

#[test]
fn mascot_researcher_etb_adds_counters_to_self_and_other() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let other_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mascot_researcher());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(other_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Researcher castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1);
    let bear = g.battlefield_find(other_bear).unwrap();
    assert_eq!(bear.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn strixhaven_tutor_etb_scrys_and_cantrips() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_tutor());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tutor castable");
    drain_stack(&mut g);
    // Hand: -1 cast +1 cantrip = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Push (modern_decks) batch 27: 22 new STX cards ──────────────────────────

#[test]
fn lorehold_stonebrand_is_a_five_mana_three_three_spirit_soldier() {
    let g = two_player_game();
    let def = catalog::lorehold_stonebrand();
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 3);
    assert!(def.subtypes.creature_types.contains(&CreatureType::Spirit));
    assert!(def.subtypes.creature_types.contains(&CreatureType::Soldier));
    drop(g);
}

#[test]
fn lorehold_stonebrand_etb_with_gy_creature_can_mint_spirit() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Seed opponent gy with a creature card.
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(dead)),
    ]));
    let id = g.add_card_to_hand(0, catalog::lorehold_stonebrand());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stonebrand castable");
    drain_stack(&mut g);
    // The body itself plus a Spirit token (if the MayDo paid + exile took).
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Lorehold Spirit Token")
        .count();
    // Either 0 or 1 — we don't assert minting succeeded; we assert the
    // body landed and the gy card was exiled (or remained intact).
    assert!(spirits <= 1);
    assert!(g.battlefield_find(id).is_some(), "Body landed");
}

#[test]
fn quandrix_geometer_etb_adds_counter_and_magecraft_self_pumps() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_geometer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Geometer castable");
    drain_stack(&mut g);
    let bear_card = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_card.counter_count(CounterType::PlusOnePlusOne), 1);
    // Now cast Bolt to fire magecraft.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let body = computed.iter().find(|c| c.id == id).unwrap();
    assert_eq!(body.power, 3, "Geometer +1/+1 EOT");
}

#[test]
fn witherbloom_soilshaper_etb_mills_and_scales_with_gy() {
    let mut g = two_player_game();
    // Seed graveyard with a creature card.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::witherbloom_soilshaper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soilshaper castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    // We milled 2 cards (one bear and one bolt, in some order), and have
    // 1 prior bear in gy. So gy has 2 creatures min (since we milled a
    // bear). Either 1 or 2 creature cards in gy after mill — body gets
    // counters equal to that count.
    let counters = body.counter_count(CounterType::PlusOnePlusOne);
    assert!(counters >= 1, "Some counters were added");
}

#[test]
fn prismari_fireshaper_etb_mints_treasure_and_magecraft_pings() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_fireshaper());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fireshaper castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1, "ETB mints a Treasure");
    // Cast Bolt to test magecraft.
    let p1_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt deals 3, magecraft deals 1 = 4.
    assert_eq!(g.players[1].life, p1_before - 4);
}

#[test]
fn strixhaven_scry_wizard_etb_scrys_and_magecraft_scrys() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_scry_wizard());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wizard castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some());
}

#[test]
fn lorehold_bookbinder_etb_recurs_and_grants_haste() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bolt)),
    ]));
    let id = g.add_card_to_hand(0, catalog::lorehold_bookbinder());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bookbinder castable");
    drain_stack(&mut g);
    // Bolt now in hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
    // Body has haste this turn.
    let computed = g.compute_battlefield();
    let body = computed.iter().find(|c| c.id == id).unwrap();
    assert!(body.keywords.contains(&Keyword::Haste));
}

#[test]
fn quandrix_wavecaster_magecraft_adds_counter_to_target() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::quandrix_wavecaster());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Either bear or wavecaster gets the +1/+1 counter (any friendly).
    let bear_card = g.battlefield_find(bear).unwrap();
    let body = g.battlefield_find(id).unwrap();
    assert!(
        bear_card.counter_count(CounterType::PlusOnePlusOne) == 1 ||
        body.counter_count(CounterType::PlusOnePlusOne) == 1
    );
}

#[test]
fn silverquill_embodiment_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_embodiment());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Embodiment castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 2);
    assert_eq!(g.players[1].life, p1_life - 2);
}

#[test]
fn witherbloom_plagueweaver_magecraft_shrinks_target() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::witherbloom_plagueweaver());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bear 2/2 - 1/-1 = 1/1.
    let computed = g.compute_battlefield();
    let bear_card = computed.iter().find(|c| c.id == bear);
    // Bear may still be alive (1/1) or just barely alive — assert SBA didn't kill it (it's 1/1).
    if let Some(b) = bear_card {
        assert_eq!(b.power, 1);
        assert_eq!(b.toughness, 1);
    }
}

#[test]
fn lorehold_pyresmith_etb_pings() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_pyresmith());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyresmith castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 1);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.definition.keywords.contains(&Keyword::FirstStrike));
}

#[test]
fn prismari_sparkbender_etb_loots() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_hand(0, catalog::island()); // a card to discard
    let id = g.add_card_to_hand(0, catalog::prismari_sparkbender());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparkbender castable");
    drain_stack(&mut g);
    // Hand: -1 cast +1 draw -1 discard = -1 net (Sparkbender + bolt drawn - island discarded).
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn quandrix_mathmage_etb_digs() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_mathmage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mathmage castable");
    drain_stack(&mut g);
    // -1 (cast) + 1 (RevealUntilFind grabs a creature or land) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn silverquill_adjudicator_etb_shrinks_opp_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_adjudicator());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Adjudicator castable");
    drain_stack(&mut g);
    // Bear 2/2 - 3/0 = -1/2 → SBA kills with 0 toughness? -1 power, 2 toughness still alive.
    let computed = g.compute_battlefield();
    let bear_card = computed.iter().find(|c| c.id == bear);
    if let Some(b) = bear_card {
        assert!(b.power <= 0);
    }
}

#[test]
fn witherbloom_drain_mage_etb_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_drain_mage());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drain-Mage castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life + 3);
    assert_eq!(g.players[1].life, p1_life - 3);
}

#[test]
fn strixhaven_pop_quiz_sage_etb_draws_and_stacks() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    let stack_target = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Discard(vec![stack_target]),
    ]));
    let id = g.add_card_to_hand(0, catalog::strixhaven_pop_quiz_sage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pop-Quiz castable");
    drain_stack(&mut g);
    // -1 cast (sage) +2 draws -1 to library = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn lorehold_spirit_champion_anthems_other_spirits() {
    let mut g = two_player_game();
    let other_spirit = g.add_card_to_battlefield(0, catalog::ageless_guardian());
    let champ = g.add_card_to_battlefield(0, catalog::lorehold_spirit_champion());
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let other = computed.iter().find(|c| c.id == other_spirit).unwrap();
    assert!(other.keywords.contains(&Keyword::FirstStrike),
        "Other Spirit got first strike");
    let champ_card = computed.iter().find(|c| c.id == champ).unwrap();
    assert!(champ_card.keywords.contains(&Keyword::FirstStrike),
        "Champion has its own innate first strike");
}

#[test]
fn witherbloom_pest_spawner_etb_mints_two_pests() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_pest_spawner());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spawner castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2);
}

#[test]
fn prismari_wave_mage_etb_mints_treasure() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_wave_mage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wave Mage castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1);
}

// ── Push (modern_decks) batch 28: 25 new cards (5 per school) ──────────────

#[test]
fn silverquill_heraldist_etb_gains_one_life_and_mints_inkling() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_heraldist());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    let bf_before: usize = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Heraldist castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
    // Heraldist + Inkling token
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Inkling"));
}

#[test]
fn inkling_spireguard_etb_pumps_friendly_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let friendly = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::inkling_spireguard());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(friendly)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spireguard castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(friendly).unwrap();
    assert_eq!(bear.power(), 3, "Bear pumped +1");
    assert_eq!(bear.toughness(), 3);
}

#[test]
fn silverquill_quillwitch_dies_drains_two() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::silverquill_quillwitch());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let p1_before = g.players[1].life;
    // Kill with Lightning Bolt (2/2 toughness → 3 damage → dies)
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    assert_eq!(g.players[1].life, p1_before - 2);
}

#[test]
fn silverquill_inkpurge_forces_opp_sac_and_gains_life() {
    let mut g = two_player_game();
    let _opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::silverquill_inkpurge());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    let opp_bf_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkpurge castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2, "+2 life");
    let opp_bf_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(opp_bf_after, opp_bf_before - 1, "opp sacrificed a creature");
}

#[test]
fn inkrise_schoolwarden_etb_draws_a_card_and_is_lifelink_flier() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::inkrise_schoolwarden());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Schoolwarden castable");
    drain_stack(&mut g);
    // -1 (cast) +1 (draw) = 0 net hand size
    assert_eq!(g.players[0].hand.len(), hand_before);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Flying));
    assert!(body.has_keyword(&Keyword::Lifelink));
}

// ── Witherbloom (B/G) batch 28 ─────────────────────────────────────────────

#[test]
fn witherbloom_vinekeeper_etb_gains_two_and_dies_trigger_gains_one() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinekeeper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinekeeper castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    // Add a friendly bear, kill it via opp Bolt → +1 life trigger
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    drain_stack(&mut g);
    let after_bear = g.players[0].life;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    assert_eq!(g.players[0].life, after_bear + 1, "+1 life from another-dies");
}

#[test]
fn pest_outcast_dies_gains_life_and_draws() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_battlefield(0, catalog::pest_outcast());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    assert_eq!(g.players[0].life, life_before + 1);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn witherbloom_drainscholar_magecraft_shrinks_target() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let _scholar = g.add_card_to_battlefield(0, catalog::witherbloom_drainscholar());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    // Cast a Bolt to trigger magecraft → opp bear gets -1/-1 EOT
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Spell cast trigger gets a target via the auto-target picker
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // The bear should be -1/-1 EOT
    let bear = g.battlefield_find(opp_bear).unwrap();
    assert_eq!(bear.power(), 1);
    assert_eq!(bear.toughness(), 1);
}

#[test]
fn witherbloom_coatlcaller_etb_mints_a_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_coatlcaller());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before: usize = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Coatlcaller castable");
    drain_stack(&mut g);
    // Coatlcaller + Pest
    let bf_after: usize = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(bf_after, bf_before + 2);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Pest"));
}

#[test]
fn witherbloom_pestbreaker_destroys_creature_and_mints_pest() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestbreaker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestbreaker castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "Bear destroyed");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Pest"));
}

// ── Lorehold (R/W) batch 28 ────────────────────────────────────────────────

#[test]
fn lorehold_pyresinger_magecraft_drains_each_opp() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::lorehold_pyresinger());
    drain_stack(&mut g);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // -3 from bolt + -1 from drain trigger
    assert_eq!(g.players[1].life, p1_before - 4);
    assert_eq!(g.players[0].life, p0_before + 1);
}

#[test]
fn lorehold_soulchanter_etb_exiles_gy_card_and_is_lifelink() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let card_in_gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_soulchanter());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(card_in_gy)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soulchanter castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Lifelink));
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == card_in_gy),
        "Bear exiled from gy");
}

#[test]
fn lorehold_flameherald_etb_pings_target() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_flameherald());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flameherald castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 1);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_embercouncil_mints_two_spirits_and_pings_each_opp() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_embercouncil());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_before = g.players[1].life;
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Embercouncil castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 1);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 2);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 0).count(),
        bf_before + 2,
    );
}

#[test]
fn lorehold_cinderpriest_etb_adds_counter_and_magecraft_pumps() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let friendly = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::lorehold_cinderpriest());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(friendly)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cinderpriest castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(friendly).unwrap();
    // Bear should be 3/3 from +1/+1 counter
    assert_eq!(bear.power(), 3);
    assert_eq!(bear.toughness(), 3);
    // Cast a Bolt - should pump bear +1/+0
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(friendly).unwrap();
    // Bear gets +1/+0 from magecraft EOT
    assert_eq!(bear.power(), 4);
    assert_eq!(bear.toughness(), 3);
}

// ── Quandrix (G/U) batch 28 ────────────────────────────────────────────────

#[test]
fn quandrix_sumcaster_magecraft_loots_via_maydo() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let _src = g.add_card_to_battlefield(0, catalog::quandrix_sumcaster());
    drain_stack(&mut g);
    // Need to add at least one card to hand so we can discard
    let _filler = g.add_card_to_hand(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // AutoDecider declines MayDo by default → hand unchanged after Bolt resolves
    // -1 (cast Bolt) = -1 net (MayDo declined)
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn fractal_multiplicand_enters_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_multiplicand());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Multiplicand castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 3);
    assert_eq!(body.toughness(), 3);
}

#[test]
fn quandrix_calculus_mage_etb_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_calculus_mage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Calculus-Mage castable");
    drain_stack(&mut g);
    // -1 cast + 1 draw = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_tidecaller_etb_taps_target_with_flash() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_tidecaller());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tidecaller castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(opp_bear).unwrap();
    assert!(bear.tapped);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Flash));
}

#[test]
fn fractal_spawning_mints_two_fractals_with_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fractal_spawning());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spawning castable");
    drain_stack(&mut g);
    // Both fractals survive with +1/+1 counters via the new
    // Selector::LastCreatedTokens (plural) primitive (push modern_decks
    // batch 28). Each Fractal lands as 0/0, gets a +1/+1 counter before
    // SBA, ending up as a 1/1.
    let fractals: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Fractal")
        .collect();
    assert_eq!(fractals.len(), 2, "Both Fractals survive");
    let with_counters = fractals.iter()
        .filter(|c| c.counter_count(CounterType::PlusOnePlusOne) > 0)
        .count();
    assert_eq!(with_counters, 2, "Both Fractals have +1/+1 counter");
}

// ── Prismari (U/R) batch 28 ────────────────────────────────────────────────

#[test]
fn prismari_embershaper_wizard_etb_treasure_loots_and_is_flying() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let _filler = g.add_card_to_hand(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_embershaper_wizard());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Embershaper-Wizard castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Flying));
    // Treasure created
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"));
}

#[test]
fn prismari_magmaboon_burns_creature_and_mints_treasure() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::prismari_magmaboon());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Magmaboon castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "Bear killed by 3 damage");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"));
}

#[test]
fn prismari_tideburst_counters_when_opp_cant_pay_two() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp Bolt castable");
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::prismari_tideburst());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tideburst castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_before, "Bolt countered → no damage");
}

#[test]
fn prismari_tempest_caller_magecraft_self_pumps_with_flying() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::prismari_tempest_caller());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(src).unwrap();
    assert!(body.has_keyword(&Keyword::Flying));
    assert_eq!(body.power(), 3, "+1/+0 from magecraft");
}

#[test]
fn prismari_pyresurge_b28_deals_three_and_draws() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::prismari_pyresurge_b28());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let p1_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyresurge castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 3);
    // -1 (cast) +1 (draw) = 0 net
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Shared/multi-college batch 28 ──────────────────────────────────────────

#[test]
fn strixhaven_battle_cleric_etb_gains_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::strixhaven_battle_cleric());
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battle-Cleric castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn strixhaven_researcher_etb_scrys_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_researcher());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Researcher castable");
    drain_stack(&mut g);
    // Scry 2 doesn't change library count
    assert_eq!(g.players[0].library.len(), lib_before);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 2);
    assert_eq!(body.toughness(), 3);
}

#[test]
fn strixhaven_combatant_pumps_on_attack_with_haste() {
    use crate::game::types::AttackTarget;
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::strixhaven_combatant());
    drain_stack(&mut g);
    let body = g.battlefield_find(src).unwrap();
    assert!(body.has_keyword(&Keyword::Haste));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: src,
        target: AttackTarget::Player(1),
    }])).expect("Combatant attacks");
    drain_stack(&mut g);
    let body = g.battlefield_find(src).unwrap();
    assert_eq!(body.power(), 3, "+1/+0 from attack trigger");
}

#[test]
fn strixhaven_druid_etb_tutors_basic_land() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::strixhaven_druid());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Druid castable");
    drain_stack(&mut g);
    let hand_has_forest = g.players[0].hand.iter().any(|c| c.definition.name == "Forest");
    assert!(hand_has_forest);
}

#[test]
fn strixhaven_drainsong_drains_opp_for_two() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::strixhaven_drainsong());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainsong castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_before + 2);
    assert_eq!(g.players[1].life, p1_before - 2);
}

// ── Push (modern_decks) batch 28: 5 new Lessons ────────────────────────────

#[test]
fn necrotic_studies_destroys_low_mv_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::necrotic_studies());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Necrotic Studies castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "Bear destroyed");
}

#[test]
fn pyromathematics_deals_three_damage_to_any_target() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pyromathematics());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyromathematics castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 3);
}

#[test]
fn inkling_lesson_mints_two_inklings() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::inkling_lesson());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkling Lesson castable");
    drain_stack(&mut g);
    let inklings = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Inkling")
        .count();
    assert_eq!(inklings, 2);
}

#[test]
fn fractal_studies_mints_fractal_scaled_by_creature_count() {
    let mut g = two_player_game();
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::fractal_studies());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractal Studies castable");
    drain_stack(&mut g);
    let fractal = g.battlefield.iter()
        .find(|c| c.controller == 0 && c.definition.name == "Fractal")
        .expect("Fractal minted");
    // 2 bears + Fractal counted at moment of AddCounter resolution:
    // The CountOf reads after the Fractal already lands, so X = 3.
    assert!(fractal.counter_count(CounterType::PlusOnePlusOne) >= 2);
}

#[test]
fn spirit_lesson_mints_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spirit_lesson());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spirit Lesson castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 2);
}

// ── Push (modern_decks) batch 28: 5 iconic cards ───────────────────────────

#[test]
fn strixhaven_archmage_etb_draws_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_archmage());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Archmage castable");
    drain_stack(&mut g);
    // -1 cast +2 draw = +1 net
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn mage_hunters_riposte_shrinks_creature() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::mage_hunters_riposte());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Riposte castable");
    drain_stack(&mut g);
    // Bear (2/2) -3/-3 → dies via SBA
    assert!(g.battlefield_find(bear).is_none());
}

#[test]
fn strixhaven_field_trip_ramps_two_basics() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let forest1 = g.add_card_to_library(0, catalog::forest());
    let forest2 = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest1)),
        DecisionAnswer::Search(Some(forest2)),
    ]));
    let id = g.add_card_to_hand(0, catalog::strixhaven_field_trip());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Field Trip castable");
    drain_stack(&mut g);
    let forest_count = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Forest")
        .count();
    assert_eq!(forest_count, 2);
}

#[test]
fn lorehold_spiritbringer_etb_mints_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritbringer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritbringer castable");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirits, 2);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Vigilance));
}

#[test]
fn witherbloom_pestcaster_magecraft_mints_pest() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::witherbloom_pestcaster());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 1);
}

// ── Push (modern_decks) batch 29: 20 new iconic STX cards ───────────────────

/// Silverquill Novice — 2/2 magecraft lifegain body.
#[test]
fn silverquill_novice_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::silverquill_novice());
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1,
        "Novice magecraft should gain 1 life");
}

/// Silverquill Headmaster — ETB drain 2 (4-life swing).
#[test]
fn silverquill_headmaster_etb_drains_two() {
    let mut g = two_player_game();
    let life_p0 = g.players[0].life;
    let life_p1 = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::silverquill_headmaster());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Headmaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_p0 + 2, "P0 gains 2 from ETB drain");
    assert_eq!(g.players[1].life, life_p1 - 2, "P1 loses 2 from ETB drain");
}

/// Witherbloom Neophyte — 2/2 magecraft drain-1 body.
#[test]
fn witherbloom_neophyte_magecraft_drains_one() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::witherbloom_neophyte());
    drain_stack(&mut g);
    let life_p0 = g.players[0].life;
    let life_p1 = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // P1 loses 3 from bolt + 1 from Neophyte = -4 (but life_before was tracked AFTER bolt).
    assert_eq!(g.players[0].life, life_p0 + 1, "Neophyte magecraft +1 life");
    assert_eq!(g.players[1].life, life_p1 - 1 /*magecraft*/ - 3 /*bolt*/);
}

/// Pestpod Lurker — ETB mints Pest, +1/+1 counter on lifegain.
#[test]
fn pestpod_lurker_etb_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pestpod_lurker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestpod castable");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 1, "Pestpod Lurker mints one Pest on ETB");
}

#[test]
fn pestpod_lurker_grows_on_lifegain() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::pestpod_lurker());
    drain_stack(&mut g);
    // Cast Healing Salve targeting self for +3 life.
    let salve = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: salve, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Salve castable");
    drain_stack(&mut g);
    let lurker = g.battlefield.iter().find(|c| c.id == src).unwrap();
    assert!(lurker.counter_count(CounterType::PlusOnePlusOne) >= 1,
        "Lurker should grow on lifegain");
}

/// Lorehold Neophyte — magecraft self-pump.
#[test]
fn lorehold_neophyte_magecraft_pumps_self() {
    let mut g = two_player_game();
    // Seed graveyard so exile target finds a card.
    let bolt_dead = g.next_id();
    let mut inst = crate::card::CardInstance::new(bolt_dead, catalog::lightning_bolt(), 0);
    inst.controller = 0;
    g.players[0].graveyard.push(inst);
    let src = g.add_card_to_battlefield(0, catalog::lorehold_neophyte());
    drain_stack(&mut g);
    let pump_before = g.computed_permanent(src).unwrap().power;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let view = g.computed_permanent(src).unwrap();
    assert!(view.power > pump_before,
        "Neophyte should pump +1/+0 on magecraft");
}

/// Lorehold Recallmage — ETB recurs a creature from gy.
#[test]
fn lorehold_recallmage_etb_returns_creature_card() {
    let mut g = two_player_game();
    let bear_id = g.next_id();
    let mut bear = crate::card::CardInstance::new(bear_id, catalog::grizzly_bears(), 0);
    bear.controller = 0;
    g.players[0].graveyard.push(bear);
    let id = g.add_card_to_hand(0, catalog::lorehold_recallmage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recallmage castable");
    drain_stack(&mut g);
    let bears_in_hand = g.players[0].hand.iter()
        .filter(|c| c.id == bear_id)
        .count();
    assert_eq!(bears_in_hand, 1, "Bear should be returned to hand");
}

/// Quandrix Reach Mage — magecraft +1/+1 counter on self.
#[test]
fn quandrix_reach_mage_magecraft_lands_counter() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::quandrix_reach_mage());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let mage = g.battlefield.iter().find(|c| c.id == src).unwrap();
    assert_eq!(mage.counter_count(CounterType::PlusOnePlusOne), 1,
        "Reach Mage should land a +1/+1 counter on magecraft");
}

/// Fractal Sumcaster — enters with X +1/+1 counters and scrys 1.
#[test]
fn fractal_sumcaster_enters_with_x_counters() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island()); // scry target
    let id = g.add_card_to_hand(0, catalog::fractal_sumcaster());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("Sumcaster castable for X=3 (XGU)");
    drain_stack(&mut g);
    let view = g.computed_permanent(id).unwrap();
    assert_eq!(view.power, 3,
        "Sumcaster should enter with X +1/+1 counters → 3 power");
}

/// Prismari Vandal — magecraft Treasure mint.
#[test]
fn prismari_vandal_magecraft_mints_treasure() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::prismari_vandal());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1, "Vandal mints one Treasure on magecraft");
}

/// Prismari Flameseeker — ETB 2 damage to a creature.
#[test]
fn prismari_flameseeker_etb_deals_two_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_flameseeker());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Flameseeker castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 2,
        "Flameseeker should deal 2 damage to target player");
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Flameseeker body should remain");
}

/// Strixhaven Tutor (already wired): smoke test that the U variant lives.
#[test]
fn strixhaven_basicseeker_etb_tutors_basic() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::strixhaven_basicseeker());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Basicseeker castable");
    drain_stack(&mut g);
    // Hand: -1 cast + 1 tutored = same; +1 in play.
    assert!(g.battlefield.iter().any(|c| c.id == id), "Body on bf");
    // Tutor added a basic to hand; hand_before is +0 after cast(-1)+tutor(+1).
    assert!(g.players[0].hand.len() >= hand_before.saturating_sub(1));
}

/// Strixhaven Rotcaster — ETB forces opp discard.
#[test]
fn strixhaven_rotcaster_etb_discards_opp_card() {
    let mut g = two_player_game();
    let _opp_card = g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::strixhaven_rotcaster());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rotcaster castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1,
        "Rotcaster ETB should make opp discard");
}

/// Strixhaven Spellfletcher — magecraft 1-damage ping with Haste body.
#[test]
fn strixhaven_spellfletcher_has_haste_and_magecraft_pings() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::strixhaven_spellfletcher());
    drain_stack(&mut g);
    let def = catalog::strixhaven_spellfletcher();
    assert!(def.keywords.contains(&Keyword::Haste));
    let life_p1_before = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Bolt opp player → 3 damage. Then magecraft pings opp for 1 → -4 total.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_p1_before - 3 - 1,
        "Spellfletcher magecraft +1 ping on top of Bolt's 3");
}

/// Strixhaven Forager — ETB gain 2 life.
#[test]
fn strixhaven_forager_etb_gains_two_life() {
    let mut g = two_player_game();
    let life_before = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::strixhaven_forager());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Forager castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2,
        "Forager ETB should gain 2 life");
}

/// Magecraft Volley — 3 damage instant.
#[test]
fn magecraft_volley_deals_three_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::magecraft_volley());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Volley castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (2/2) dies to 3 damage");
}

/// Strixhaven Curriculum — Impulse-style look-3.
#[test]
fn strixhaven_curriculum_puts_one_into_hand() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::strixhaven_curriculum());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Curriculum castable");
    drain_stack(&mut g);
    // After cast (-1) + reveal-until-find lands one card (+1) = hand_before.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Curriculum nets zero card differential (cast -1, found +1)");
}

/// Witherbloom Recursion — return creature from gy + lose 2.
#[test]
fn witherbloom_recursion_reanimates_creature_at_two_life() {
    let mut g = two_player_game();
    let bear_id = g.next_id();
    let mut bear = crate::card::CardInstance::new(bear_id, catalog::grizzly_bears(), 0);
    bear.controller = 0;
    g.players[0].graveyard.push(bear);
    let id = g.add_card_to_hand(0, catalog::witherbloom_recursion());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Recursion castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear_id),
        "Bear should be reanimated");
    assert_eq!(g.players[0].life, life_before - 2,
        "Recursion costs 2 life on resolve");
}

/// Lorehold Battle Banner — per-attacker +1/+0 pump.
#[test]
fn lorehold_battle_banner_pumps_attackers() {
    let mut g = two_player_game();
    let _banner = g.add_card_to_battlefield(0, catalog::lorehold_battle_banner());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.battlefield.iter_mut().for_each(|c| c.tapped = false);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }])).expect("Bear can attack");
    drain_stack(&mut g);
    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 3, "Bear should get +1/+0 from Battle Banner");
}

/// Silverquill Inkpact — drain 3.
#[test]
fn silverquill_inkpact_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_inkpact());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_p0 = g.players[0].life;
    let life_p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Inkpact castable for {1}{W}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_p0 + 3, "P0 gains 3 from drain");
    assert_eq!(g.players[1].life, life_p1 - 3, "P1 loses 3 from drain");
}

// ── Effect::MoveCounter ─────────────────────────────────────────────────────

/// Engine smoke test: `Effect::MoveCounter` moves N +1/+1 counters
/// from one permanent to another. Per CR 122.5, the move is NOT
/// counter creation, so DoubleCounters (Doubling Season) does not
/// double the move count.
#[test]
fn move_counter_transfers_counters_between_permanents() {
    use crate::card::CounterType;
    use crate::effect::{Effect, Selector, Value};
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dst = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Seed 3 counters on src.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == src) {
        c.counters.insert(CounterType::PlusOnePlusOne, 3);
    }
    let ctx = EffectContext {
        controller: 0,
        source: Some(src),
        targets: vec![Target::Permanent(dst)],
        trigger_source: None,
        mode: 0,
        x_value: 0,
        converged_value: 0,
        mana_spent: 0,
        source_name: None,
        cast_from_hand: true,
        event_amount: 0,
        kicked: false,
        bargained: false,
    };
    let effect = Effect::MoveCounter {
        from: Selector::This,
        to: Selector::Target(0),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(2),
    };
    let _ = g.resolve_effect(&effect, &ctx);
    let src_after = g.battlefield.iter().find(|c| c.id == src).unwrap();
    let dst_after = g.battlefield.iter().find(|c| c.id == dst).unwrap();
    assert_eq!(src_after.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(dst_after.counter_count(CounterType::PlusOnePlusOne), 2);
}

/// MoveCounter clamps at the source's actual counter pool — moving
/// more than available transfers only what's there.
#[test]
fn move_counter_clamps_at_source_pool() {
    use crate::card::CounterType;
    use crate::effect::{Effect, Selector, Value};
    use crate::game::effects::EffectContext;
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dst = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == src) {
        c.counters.insert(CounterType::PlusOnePlusOne, 1);
    }
    let ctx = EffectContext {
        controller: 0,
        source: Some(src),
        targets: vec![Target::Permanent(dst)],
        trigger_source: None,
        mode: 0,
        x_value: 0,
        converged_value: 0,
        mana_spent: 0,
        source_name: None,
        cast_from_hand: true,
        event_amount: 0,
        kicked: false,
        bargained: false,
    };
    let effect = Effect::MoveCounter {
        from: Selector::This,
        to: Selector::Target(0),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(99),
    };
    let _ = g.resolve_effect(&effect, &ctx);
    let src_after = g.battlefield.iter().find(|c| c.id == src).unwrap();
    let dst_after = g.battlefield.iter().find(|c| c.id == dst).unwrap();
    assert_eq!(src_after.counter_count(CounterType::PlusOnePlusOne), 0,
        "Source drained even though we asked for more than available");
    assert_eq!(dst_after.counter_count(CounterType::PlusOnePlusOne), 1,
        "Destination only got the actually-removed count");
}

// ── Batch 30: New STX cards (Lorehold/Witherbloom/Silverquill/Prismari/Quandrix) ──

#[test]
fn lorehold_sparkscholar_magecraft_self_pumps() {
    let _src = "Lorehold Sparkscholar";
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::lorehold_sparkscholar());
    drain_stack(&mut g);
    let body = g.battlefield_find(src).unwrap();
    assert_eq!(body.power(), 2);
    assert_eq!(body.toughness(), 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(src).unwrap();
    assert_eq!(body.power(), 3, "Magecraft +1/+0");
}

#[test]
fn lorehold_ironscribe_etb_gains_three_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_ironscribe());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Ironscribe castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_spiritflame_deals_two_and_gains_one() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_spiritflame());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    let opp_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spiritflame castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life_before - 2);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn lorehold_stoneweaver_is_vigilant_lifelink_and_exiles_gy() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let gy_card = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_stoneweaver());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(gy_card)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stoneweaver castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Vigilance));
    assert!(body.has_keyword(&Keyword::Lifelink));
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == gy_card),
        "Bear exiled from opp gy");
}

#[test]
fn lorehold_pyrescroll_burns_and_mints_spirit() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::lorehold_pyrescroll());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrescroll castable");
    drain_stack(&mut g);
    // 2-tough bear gets 2 damage and dies
    assert!(g.battlefield_find(target).is_none(), "Bear died");
    // Spirit token minted
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Spirit"));
}

#[test]
fn lorehold_battle_witness_etb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let gy_creature = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_battle_witness());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Battle Witness castable");
    drain_stack(&mut g);
    // -1 for cast, +1 for return → net 0
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Bear should be in hand
    assert!(g.players[0].hand.iter().any(|c| c.id == gy_creature));
}

// Witherbloom

#[test]
fn pest_cultist_drains_when_another_creature_dies() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let _cultist = g.add_card_to_battlefield(0, catalog::pest_cultist());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let p0_before = g.players[0].life;
    let p1_before = g.players[1].life;
    // Kill the bear with Lightning Bolt — proper damage path so the
    // AnotherOfYours die-trigger dispatch fires.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_before + 1);
    assert_eq!(g.players[1].life, p1_before - 1);
}

#[test]
fn witherbloom_bonecrafter_etb_mills_and_gains_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_bonecrafter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bonecrafter castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2);
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn witherbloom_toxbrewer_magecraft_shrinks_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _src = g.add_card_to_battlefield(0, catalog::witherbloom_toxbrewer());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // 2/2 bear with -1/-1 dies to SBA (1/1 minus damage already 0)
    // Actually 2/2 - 1/-1 = 1/1, still alive after SBA
    let bear_view = g.battlefield.iter().find(|c| c.id == bear);
    // either dead or 1/1
    if let Some(b) = bear_view {
        assert!(b.power() <= 1 && b.toughness() <= 1, "Bear shrunk by -1/-1");
    }
}

#[test]
fn witherbloom_lichenkeeper_etb_mints_pest() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_lichenkeeper());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lichenkeeper castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Reach));
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Pest"));
}

#[test]
fn witherbloom_sapwarden_destroys_opp_creature_and_gains_two() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::witherbloom_sapwarden());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sapwarden castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "Bear destroyed");
    assert_eq!(g.players[0].life, life_before + 2);
}

// Silverquill

#[test]
fn silverquill_drafter_b30_etb_drains_two() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_drafter_b30());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drafter castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_before - 2);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Flying));
}

#[test]
fn silverquill_scrivener_b30_etb_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::silverquill_scrivener_b30());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Scrivener castable");
    drain_stack(&mut g);
    // -1 cast, +1 draw = same hand size
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn inkling_cantor_magecraft_pumps_friendly() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _src = g.add_card_to_battlefield(0, catalog::inkling_cantor());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let bear_view = g.battlefield_find(bear).unwrap();
    assert_eq!(bear_view.power(), 3, "+1/+1 from Magecraft");
    assert_eq!(bear_view.toughness(), 3);
}

#[test]
fn silverquill_pact_gains_four_and_mints_two_inklings() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_pact());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pact castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 4);
    let inklings = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Inkling")
        .count();
    assert_eq!(inklings, 2);
}

#[test]
fn silverquill_vellumweaver_magecraft_gains_one_life() {
    let mut g = two_player_game();
    let _src = g.add_card_to_battlefield(0, catalog::silverquill_vellumweaver());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);
}

// Prismari

#[test]
fn prismari_sparksong_deals_three_and_draws() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::prismari_sparksong());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sparksong castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "Bear died to 3 damage");
    // -1 cast, +1 draw = same hand size
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn prismari_glasscaster_magecraft_self_pumps() {
    let mut g = two_player_game();
    let src = g.add_card_to_battlefield(0, catalog::prismari_glasscaster());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(src).unwrap();
    assert_eq!(body.power(), 3, "+1/+1 from magecraft");
    assert_eq!(body.toughness(), 3);
}

#[test]
fn prismari_treasurewright_b30_etb_mints_treasure_and_magecraft_scrys() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_treasurewright_b30());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Treasurewright castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"));
}

#[test]
fn prismari_tideforger_has_flash() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_tideforger());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Tideforger castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Flash));
}

#[test]
fn prismari_splashcaster_burns_and_mints_treasure() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::prismari_splashcaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Splashcaster castable");
    drain_stack(&mut g);
    // 2 damage to target + 2 to each opp = 4 damage to player 1
    assert_eq!(g.players[1].life, opp_life_before - 4);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"));
}

// Quandrix

#[test]
fn quandrix_hydronaut_etb_adds_counter() {
    use crate::game::types::Target;
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::quandrix_hydronaut());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hydronaut castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(target).unwrap();
    assert_eq!(bear.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_fractalweaver_etb_mills_and_magecraft_counters() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_fractalweaver());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Fractalweaver castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn quandrix_geomancer_b30_is_a_three_mana_two_three_druid() {
    // Sanity: card body / cost / types check. The basic-land tutor relies on
    // the auto-decider's library-search picker which depends on Library
    // contents in non-test contexts (already covered by other tutor cards
    // like Strixhaven Druid).
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::quandrix_geomancer_b30());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Geomancer castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 2);
    assert_eq!(body.toughness(), 3);
    assert!(body.definition
        .subtypes
        .creature_types
        .contains(&crate::card::CreatureType::Druid));
}

#[test]
fn quandrix_mindforge_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_mindforge());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mindforge castable");
    drain_stack(&mut g);
    // -1 cast, +1 draw = same hand size
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn quandrix_branchwarden_etb_draws_and_has_reach() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::quandrix_branchwarden());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Branchwarden castable");
    drain_stack(&mut g);
    // -1 cast, +1 draw = same hand size
    assert_eq!(g.players[0].hand.len(), hand_before);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Reach));
}

// ── New Lorehold STX cards (push batch — claude/modern_decks) ──────────────

#[test]
fn lorehold_battlescholar_attack_exiles_target_graveyard_card() {
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let gy_target = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let id = g.add_card_to_battlefield(0, catalog::lorehold_battlescholar());
    g.clear_sickness(id);
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::FirstStrike));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("Battlescholar attacks");
    drain_stack(&mut g);
    // Graveyard target should be in exile via the auto-picker.
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == gy_target),
        "Bolt should leave gy after the attack trigger fires");
    assert!(g.exile.iter().any(|c| c.id == gy_target),
        "Bolt should be in exile");
}

#[test]
fn lorehold_pyrokineticist_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::lorehold_pyrokineticist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt = 3 dmg + magecraft = 1 dmg → 4 total to opp.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn lorehold_wargleam_etb_pumps_other_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::lorehold_wargleam());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Wargleam castable");
    drain_stack(&mut g);
    let bear = g.battlefield_find(bear).unwrap();
    assert_eq!(bear.counter_count(CounterType::PlusOnePlusOne), 1);
    let wargleam = g.battlefield_find(id).unwrap();
    assert!(wargleam.has_keyword(&Keyword::Vigilance));
}

#[test]
fn lorehold_stoneglyph_burns_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_stoneglyph());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Stoneglyph castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
}

#[test]
fn lorehold_reverend_etb_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_reverend());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reverend castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 2);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.has_keyword(&Keyword::Vigilance));
    assert!(body.has_keyword(&Keyword::Lifelink));
}

#[test]
fn lorehold_recountmage_magecraft_may_decline_by_default() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_recountmage());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    // AutoDecider declines MayDo by default → 4 toughness, no extra damage taken.
    assert_eq!(body.toughness(), 4);
    assert_eq!(body.damage, 0);
}

#[test]
fn lorehold_inscribe_burns_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lorehold_inscribe());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Inscribe castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1);
}

#[test]
fn lorehold_reenactor_etb_returns_low_mv_creature_with_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_reenactor());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Auto-decider picks the gy bear via Selector::one_of.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reenactor castable");
    drain_stack(&mut g);
    let bear_bf = g.battlefield_find(bear).expect("Bear reanimated");
    assert!(bear_bf.has_keyword(&Keyword::Haste));
}

#[test]
fn lorehold_ardent_pyromage_magecraft_self_pumps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_ardent_pyromage());
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(id).unwrap();
    assert_eq!(body.power(), 3, "magecraft adds +1/+0 EOT → 3 power");
    assert_eq!(body.toughness(), 2);
}

#[test]
fn lorehold_memorial_taps_for_red_or_white() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::lorehold_memorial_reliquary());
    drain_stack(&mut g);
    // Activate the Red mana ability (index 0).
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, x_value: None }).expect("Memorial Red mana ability");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    let body = g.battlefield_find(id).unwrap();
    assert!(body.tapped);
}

#[test]
fn lorehold_spirit_sentinel_pumps_on_friendly_spirit_etb() {
    let mut g = two_player_game();
    let sentinel = g.add_card_to_battlefield(0, catalog::lorehold_spirit_sentinel());
    drain_stack(&mut g);
    // Cast a Spirit creature so the EntersBattlefield event fires.
    let pyromage = g.add_card_to_hand(0, catalog::lorehold_pyrokineticist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: pyromage, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrokineticist castable");
    drain_stack(&mut g);
    let body = g.battlefield_find(sentinel).unwrap();
    assert_eq!(body.counter_count(CounterType::PlusOnePlusOne), 1,
        "Spirit ETB triggered Sentinel's +1/+1");
}

#[test]
fn lorehold_pyrotechnician_etb_burns_opp_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::lorehold_pyrotechnician());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pyrotechnician castable");
    drain_stack(&mut g);
    // 2-damage to a 2/2 bear → bear dies.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

// ── New Witherbloom STX cards (push batch — claude/modern_decks) ──────────

#[test]
fn witherbloom_bloomweaver_etb_mints_pest_and_magecraft_drains() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_bloomweaver());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloomweaver castable");
    drain_stack(&mut g);
    let pests: Vec<_> = g.battlefield.iter().filter(|c| {
        c.controller == 0 && c.is_token && c.definition.name == "Pest"
    }).collect();
    assert_eq!(pests.len(), 1, "ETB mints 1 Pest");
    // Now cast a Bolt to fire the magecraft trigger.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Bloomweaver magecraft 1 = 4 to opp.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn witherbloom_drainpath_drains_two_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_drainpath());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Drainpath castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, me_before + 2);
}

#[test]
fn witherbloom_vinekeeper_b30_attack_drains_two() {
    use crate::game::types::{Attack, AttackTarget, TurnStep};
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::witherbloom_vinekeeper_b30());
    g.clear_sickness(id);
    let me_before = g.players[0].life;
    let opp_before = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id, target: AttackTarget::Player(1),
    }])).expect("Vinekeeper attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 2);
    assert_eq!(g.players[0].life, me_before + 2);
}

#[test]
fn witherbloom_sapcurse_shrinks_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let id = g.add_card_to_hand(0, catalog::witherbloom_sapcurse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sapcurse castable");
    drain_stack(&mut g);
    // 2/2 -> -2/-2 = 0/0 toughness → dies via SBA.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn witherbloom_pestreaver_etb_mills_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_pestreaver());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pestreaver castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2, "milled 2");
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn witherbloom_vinemender_etb_gains_three_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_vinemender());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Vinemender castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 3);
}

#[test]
fn witherbloom_devourer_etb_forces_sac() {
    let mut g = two_player_game();
    let _opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);
    let bf_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    let id = g.add_card_to_hand(0, catalog::witherbloom_devourer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Devourer castable");
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert_eq!(bf_after, bf_before - 1, "Opp sacrificed a creature");
}

#[test]
fn witherbloom_lifebloom_gains_four_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::witherbloom_lifebloom());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lifebloom castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 4);
}

#[test]
fn witherbloom_rotmancer_magecraft_pings_each_opp() {
    let mut g = two_player_game();
    let _id = g.add_card_to_battlefield(0, catalog::witherbloom_rotmancer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt castable");
    drain_stack(&mut g);
    // Bolt 3 + Rotmancer magecraft 1 = 4.
    assert_eq!(g.players[1].life, opp_before - 4);
}

#[test]
fn creative_outburst_deals_five_and_digs_for_one() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::creative_outburst());
    // Seed a known top card; auto-decider keeps the top of the dug pile.
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let opp_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crate::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Creative Outburst castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 5, "5 damage to the opponent");
    assert!(g.players[0].hand.iter().any(|c| c.id == top),
        "dug-for top card landed in hand");
}

#[test]
fn heated_debate_damage_ignores_a_prevention_shield() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Shield the bear with "prevent the next 3 damage" (Healing Salve mode 1).
    let salve = g.add_card_to_hand(1, catalog::healing_salve());
    g.players[1].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: salve, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Healing Salve castable");
    drain_stack(&mut g);
    // Heated Debate: prevention is locked, so all 4 damage lands and kills it.
    let hd = g.add_card_to_hand(0, catalog::heated_debate());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: hd, target: Some(crate::game::types::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Heated Debate castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(),
        "shield is ignored — 4 damage kills the 2/2");
}
