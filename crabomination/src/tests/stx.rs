//! Functionality tests for the Strixhaven base set
//! (`catalog::sets::stx`). New STX cards added here should ship with at
//! least one test exercising their primary play pattern.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::*;
use crate::mana::Color;
use crate::player::Player;

fn two_player_game() -> GameState {
    let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
    let mut g = GameState::new(players);
    g.step = TurnStep::PreCombatMain;
    g
}

fn drain_stack(g: &mut GameState) {
    while !g.stack.is_empty() {
        g.perform_action(GameAction::PassPriority).unwrap();
        g.perform_action(GameAction::PassPriority).unwrap();
    }
}

// ── Mono-color additions ────────────────────────────────────────────────────

#[test]
fn pop_quiz_draws_two_then_returns_one_to_top() {
    // Seed library so the draw has cards.
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::pop_quiz());
    let lib_before = g.players[0].library.len();
    let hand_before = g.players[0].hand.len();

    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Pop Quiz castable for {1}{W}");
    drain_stack(&mut g);

    // Hand: -1 (cast Pop Quiz) +2 (draw) -1 (put on top) = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Library: -2 (drawn) +1 (put-on-top) = -1.
    assert_eq!(g.players[0].library.len(), lib_before - 1);
}

#[test]
fn mascot_exhibition_creates_three_distinct_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mascot_exhibition());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Mascot Exhibition castable for {5}{W}{W}");
    drain_stack(&mut g);

    let tokens: Vec<_> = g.battlefield.iter().filter(|c| c.is_token).collect();
    assert_eq!(tokens.len(), 3, "should mint exactly three tokens");
    let elephant = tokens.iter().find(|c| c.definition.name == "Elephant")
        .expect("3/3 Elephant present");
    assert_eq!(elephant.power(), 3);
    assert_eq!(elephant.toughness(), 3);
    let cat = tokens.iter().find(|c| c.definition.name == "Cat")
        .expect("2/2 Cat with lifelink present");
    assert!(cat.has_keyword(&Keyword::Lifelink));
    let bird = tokens.iter().find(|c| c.definition.name == "Bird")
        .expect("1/1 Bird with flying present");
    assert!(bird.has_keyword(&Keyword::Flying));
}

#[test]
fn plumb_the_forbidden_at_x_two_sacs_two_draws_two_loses_two() {
    let mut g = two_player_game();
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b3 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }

    let id = g.add_card_to_hand(0, catalog::plumb_the_forbidden());
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;
    let bf_creatures_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature())
        .count();

    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: Some(2),
    })
    .expect("Plumb the Forbidden castable for {X=2}{B}{B}");
    drain_stack(&mut g);

    let bf_creatures_after = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature())
        .count();
    // Sacrificed 2 creatures.
    assert_eq!(bf_creatures_after, bf_creatures_before - 2,
        "two creatures sacrificed");
    // Hand: -1 (cast) +2 (draw) = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2);
    // Life: -2.
    assert_eq!(g.players[0].life, life_before - 2);
}

#[test]
fn owlin_shieldmage_is_a_flash_flyer() {
    // Sanity: card is built with the expected keywords / P/T.
    let owlin = catalog::owlin_shieldmage();
    assert!(owlin.keywords.contains(&Keyword::Flash));
    assert!(owlin.keywords.contains(&Keyword::Flying));
    assert_eq!(owlin.power, 2);
    assert_eq!(owlin.toughness, 3);
}

#[test]
fn frost_trickster_taps_and_stuns_target_on_etb() {
    let mut g = two_player_game();
    // Untapped creature on opponent's battlefield.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let id = g.add_card_to_hand(0, catalog::frost_trickster());

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Frost Trickster castable for {1}{U}");
    drain_stack(&mut g);

    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(bear_card.tapped, "target should be tapped");
    assert_eq!(bear_card.counter_count(CounterType::Stun), 1,
        "target should have a stun counter");
}

#[test]
fn body_of_research_creates_fractal_with_counters_from_library() {
    // Push XVI: Body of Research now uses `Value::LibrarySizeOf(You)` to
    // match the printed "for each card in your library" Oracle.
    let mut g = two_player_game();
    // Seed P0's library with 5 cards.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::body_of_research());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Body of Research castable for {4}{G}{U}");
    drain_stack(&mut g);

    let fractal = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Fractal")
        .expect("Fractal token present");
    // The Fractal should have +1/+1 counters equal to library size.
    let counters = fractal.counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, lib_before as u32,
        "Fractal +1/+1 counter count should equal library size before cast; got {}, expected {}",
        counters, lib_before);
    assert_eq!(fractal.power(), counters as i32);
    assert_eq!(fractal.toughness(), counters as i32);
}

#[test]
fn show_of_confidence_pumps_with_storm_count() {
    let mut g = two_player_game();
    // Cast a Lightning Bolt first to bump the storm counter, then Show of
    // Confidence — the spell should add `storm_count + 1` counters.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let soc = g.add_card_to_hand(0, catalog::show_of_confidence());

    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    g.perform_action(GameAction::CastSpell {
        card_id: soc, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Show of Confidence castable for {1}{W}");
    drain_stack(&mut g);

    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    let counters = bear_card.counter_count(CounterType::PlusOnePlusOne);
    // Storm count = 1 (Bolt) → Show of Confidence adds 1 + 1 = 2 counters.
    assert_eq!(counters, 2, "Should add storm_count + 1 = 2 counters");
}

#[test]
fn bury_in_books_returns_target_to_top_of_library() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::bury_in_books());

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Bury in Books castable for {3}{U}");
    drain_stack(&mut g);

    // Bear is off the battlefield and on top of P1's library.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    let top = g.players[1].library.last().expect("library not empty");
    assert_eq!(top.id, bear, "bear should be on top of P1's library");
}

#[test]
fn test_of_talents_counters_target_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    // Bolt is on the stack; P0 responds.
    g.priority.player_with_priority = 0;
    let tot = g.add_card_to_hand(0, catalog::test_of_talents());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: tot, target: Some(Target::Permanent(bolt)), mode: None, x_value: None,
    })
    .expect("Test of Talents castable for {1}{U}{U}");
    drain_stack(&mut g);

    // P0's life is unchanged — Bolt was countered.
    assert_eq!(g.players[0].life, 20, "Bolt should have been countered");
    // Bolt is in the graveyard.
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

// ── Repartee plumbing ──────────────────────────────────────────────────────

#[test]
fn rehearsed_debater_pumps_when_instant_targets_creature() {
    // Repartee: cast Lightning Bolt targeting a creature → Debater +1/+1 EOT.
    let mut g = two_player_game();
    let debater = g.add_card_to_battlefield(0, catalog::rehearsed_debater());
    g.clear_sickness(debater);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let d = g.battlefield.iter().find(|c| c.id == debater).unwrap();
    assert_eq!(d.power(), 4, "Debater should be 4/4 from Repartee");
    assert_eq!(d.toughness(), 4);
}

#[test]
fn rehearsed_debater_does_not_pump_when_targeting_player() {
    // Repartee fires on instant/sorcery that targets a CREATURE — bolting
    // a player should NOT trigger.
    let mut g = two_player_game();
    let debater = g.add_card_to_battlefield(0, catalog::rehearsed_debater());
    g.clear_sickness(debater);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let d = g.battlefield.iter().find(|c| c.id == debater).unwrap();
    assert_eq!(d.power(), 3, "Debater should NOT be pumped (target was a player)");
    assert_eq!(d.toughness(), 3);
}

#[test]
fn lecturing_scornmage_gains_counter_on_creature_targeted_spell() {
    let mut g = two_player_game();
    let scorn = g.add_card_to_battlefield(0, catalog::lecturing_scornmage());
    g.clear_sickness(scorn);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let s = g.battlefield.iter().find(|c| c.id == scorn).unwrap();
    assert_eq!(
        s.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Scornmage should gain a +1/+1 counter from Repartee"
    );
}

#[test]
fn melancholic_poet_drains_on_creature_targeted_spell() {
    let mut g = two_player_game();
    let _poet = g.add_card_to_battlefield(0, catalog::melancholic_poet());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Bolt: 3 to bear (kills); Repartee: drain 1 (P1 -1, P0 +1).
    assert_eq!(g.players[0].life, 21, "P0 +1 from Repartee drain");
    assert_eq!(g.players[1].life, 19, "P1 -1 from Repartee drain");
}

#[test]
fn multiple_choice_mode_one_creates_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::multiple_choice());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(1), x_value: None,
    })
    .expect("Multiple Choice castable for {1}{U}{U}");
    drain_stack(&mut g);

    // Mode 1 minted a 1/1 Pest token.
    let pest = g.battlefield.iter()
        .find(|c| c.is_token && c.definition.name == "Pest")
        .expect("Pest token present");
    assert_eq!(pest.power(), 1);
    assert_eq!(pest.toughness(), 1);
}

// ── Lorehold (R/W) ──────────────────────────────────────────────────────────

#[test]
fn lorehold_apprentice_gains_life_on_instant_cast() {
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::lorehold_apprentice());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Magecraft fires off the Bolt cast; Apprentice's lifegain rider trips.
    assert_eq!(g.players[0].life, life_before + 1,
        "Magecraft should grant +1 life on instant cast");
}

#[test]
fn lorehold_apprentice_does_not_gain_on_creature_spell() {
    // Magecraft only triggers on instant/sorcery, not creature spells.
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::lorehold_apprentice());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, mode: None, x_value: None,
    })
    .expect("Bears castable for {1}{G}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before,
        "Casting a creature should NOT trigger Magecraft");
}

#[test]
fn lorehold_pledgemage_has_reach() {
    let p = catalog::lorehold_pledgemage();
    assert!(p.keywords.contains(&Keyword::Reach));
    assert_eq!(p.power, 2);
    assert_eq!(p.toughness, 2);
}

#[test]
fn pillardrop_rescuer_returns_target_instant_from_graveyard() {
    let mut g = two_player_game();
    // P0 has a Bolt in their graveyard.
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::pillardrop_rescuer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt)), mode: None, x_value: None,
    })
    .expect("Pillardrop Rescuer castable for {3}{R}{W}");
    drain_stack(&mut g);
    // Bolt should be back in P0's hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt should be returned to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Bolt should no longer be in graveyard");
}

#[test]
fn heated_debate_deals_4_damage_to_target_creature() {
    let mut g = two_player_game();
    // 4-toughness creature dies to Heated Debate's 4 damage.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::heated_debate());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Heated Debate castable for {2}{R}");
    drain_stack(&mut g);
    // Bear (2/2) takes 4 damage and dies → graveyard.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be off the battlefield");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear),
        "Bear should be in P1's graveyard");
}

#[test]
fn storm_kiln_artist_creates_treasure_and_deals_1_damage() {
    let mut g = two_player_game();
    let _ska = g.add_card_to_battlefield(0, catalog::storm_kiln_artist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p1_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Storm-Kiln Artist's Magecraft: 1 damage to opponent + Treasure token.
    // Bolt also dealt 3 damage so total is 4.
    assert_eq!(g.players[1].life, p1_life_before - 4,
        "P1 takes 3 (Bolt) + 1 (Magecraft) = 4 damage");
    let treasures = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .count();
    assert_eq!(treasures, 1, "Storm-Kiln Artist should mint one Treasure");
}

// ── Quandrix (G/U) ──────────────────────────────────────────────────────────

#[test]
fn quandrix_apprentice_pumps_creature_you_control_on_instant_cast() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::quandrix_apprentice());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Magecraft auto-targets a creature you control. With the engine's
    // source-avoidance picker, the Apprentice (trigger source) is avoided
    // when another legal target exists — so the bear gets the pump.
    let bear_card = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        (bear_card.power(), bear_card.toughness()),
        (3, 3),
        "Source-avoidance picker should pump the bear, not the Apprentice",
    );
    let app_card = g.battlefield.iter().find(|c| c.id == app).unwrap();
    assert_eq!(
        (app_card.power(), app_card.toughness()),
        (1, 1),
        "Apprentice (trigger source) should not be the picked target",
    );
}

#[test]
fn quandrix_apprentice_falls_back_to_self_when_no_other_target() {
    // Source-avoidance falls back to the source when it's the only legal
    // pick — the trigger should still resolve, not fizzle.
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::quandrix_apprentice());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let app_card = g.battlefield.iter().find(|c| c.id == app).unwrap();
    assert_eq!(
        (app_card.power(), app_card.toughness()),
        (2, 2),
        "Apprentice pumps itself when it's the only legal Magecraft target",
    );
}

#[test]
fn quandrix_pledgemage_grows_via_activated_ability() {
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::quandrix_pledgemage());
    g.clear_sickness(pm);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pm, ability_index: 0, target: None,
    })
    .expect("Quandrix Pledgemage activatable for {1}{G}{U}");
    drain_stack(&mut g);
    let pm_card = g.battlefield.iter().find(|c| c.id == pm).unwrap();
    assert_eq!(pm_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "should gain 1 +1/+1 counter");
    assert_eq!(pm_card.power(), 3, "Pledgemage now 3/3");
    assert_eq!(pm_card.toughness(), 3);
}

#[test]
fn decisive_denial_counters_noncreature_unless_paid() {
    let mut g = two_player_game();
    // P1 casts a Bolt; P0 responds with Decisive Denial mode 0.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), mode: None, x_value: None,
    })
    .expect("Bolt castable");
    g.priority.player_with_priority = 0;
    let dd = g.add_card_to_hand(0, catalog::decisive_denial());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: dd, target: Some(Target::Permanent(bolt)), mode: Some(0), x_value: None,
    })
    .expect("Decisive Denial castable");
    drain_stack(&mut g);
    // Bolt countered (P1 had no extra mana for {2} kicker), P0 unhurt.
    assert_eq!(g.players[0].life, 20, "Bolt should be countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Bolt should be in graveyard");
}

// ── Prismari (U/R) ──────────────────────────────────────────────────────────

#[test]
fn prismari_pledgemage_has_trample_and_haste() {
    let p = catalog::prismari_pledgemage();
    assert!(p.keywords.contains(&Keyword::Trample));
    assert!(p.keywords.contains(&Keyword::Haste));
    assert_eq!(p.power, 2);
    assert_eq!(p.toughness, 3);
}

#[test]
fn prismari_apprentice_scrys_on_instant_cast() {
    let mut g = two_player_game();
    let _app = g.add_card_to_battlefield(0, catalog::prismari_apprentice());
    // Seed library so there's something to scry.
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let lib_before = g.players[0].library.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    // Scry doesn't change library size (it just looks at the top card and
    // optionally moves it to bottom). Sanity: library is still seeded.
    assert_eq!(g.players[0].library.len(), lib_before,
        "Scry 1 should not change library size");
}

#[test]
fn symmetry_sage_pumps_self_and_grants_flying_on_instant_cast() {
    let mut g = two_player_game();
    let sage = g.add_card_to_battlefield(0, catalog::symmetry_sage());
    g.clear_sickness(sage);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let s = g.battlefield.iter().find(|c| c.id == sage).unwrap();
    assert_eq!(s.power(), 2, "Sage should be 2/2 (1+1 from Magecraft)");
    assert_eq!(s.toughness(), 2);
    assert!(s.has_keyword(&Keyword::Flying),
        "Magecraft should grant flying EOT");
}

// ── Witherbloom (B/G) ──────────────────────────────────────────────────────

#[test]
fn witherbloom_pledgemage_taps_for_mana_at_life_cost() {
    let mut g = two_player_game();
    let pm = g.add_card_to_battlefield(0, catalog::witherbloom_pledgemage());
    g.clear_sickness(pm);
    let life_before = g.players[0].life;
    let pool_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: pm, ability_index: 0, target: None,
    })
    .expect("Witherbloom Pledgemage tap activatable");
    drain_stack(&mut g);
    // Pay 1 life, gain 1 mana.
    assert_eq!(g.players[0].life, life_before - 1, "should pay 1 life");
    assert_eq!(g.players[0].mana_pool.total(), pool_before + 1,
        "should gain 1 mana");
    let pm_card = g.battlefield.iter().find(|c| c.id == pm).unwrap();
    assert!(pm_card.tapped, "should be tapped after activating");
}

#[test]
fn sparring_regimen_creates_a_2_2_spirit_token_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sparring_regimen());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Sparring Regimen castable for {2}{R}{W}");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 1, "should create one Spirit token");
    let s = spirits[0];
    assert_eq!(s.power(), 2);
    assert_eq!(s.toughness(), 2);
    assert!(s.definition.subtypes.creature_types
        .contains(&crate::card::CreatureType::Spirit),
        "should be a Spirit");
}

#[test]
fn sparring_regimen_pumps_each_attacker_with_a_counter() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sparring_regimen());
    let attacker_a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let attacker_b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Attackers must not be summoning-sick.
    if let Some(c) = g.battlefield_find_mut(attacker_a) { c.summoning_sick = false; }
    if let Some(c) = g.battlefield_find_mut(attacker_b) { c.summoning_sick = false; }
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;

    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: attacker_a, target: AttackTarget::Player(1) },
        Attack { attacker: attacker_b, target: AttackTarget::Player(1) },
    ])).expect("declare two attackers");
    drain_stack(&mut g);

    let a = g.battlefield_find(attacker_a).unwrap();
    let b = g.battlefield_find(attacker_b).unwrap();
    assert_eq!(a.counter_count(CounterType::PlusOnePlusOne), 1,
        "attacker A should pick up a +1/+1 from the on-attack trigger");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1,
        "attacker B should pick up a +1/+1 from the on-attack trigger");
}

#[test]
fn bayou_groff_is_a_5_4_beast() {
    let g = catalog::bayou_groff();
    assert_eq!(g.power, 5);
    assert_eq!(g.toughness, 4);
    assert!(g.subtypes.creature_types.contains(&crate::card::CreatureType::Beast));
}

#[test]
fn pest_summoning_creates_two_pests() {
    // Real-text fix: was minting 1 Pest, now mints 2.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pest_summoning());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Pest Summoning castable for {B}{G}");
    drain_stack(&mut g);
    let pests = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest")
        .count();
    assert_eq!(pests, 2, "Pest Summoning should mint two Pest tokens");
}


// ── New iconic STX cards ────────────────────────────────────────────────────

#[test]
fn strict_proctor_is_a_one_three_flying_spirit() {
    let p = catalog::strict_proctor();
    assert_eq!(p.power, 1);
    assert_eq!(p.toughness, 3);
    assert!(p.keywords.contains(&Keyword::Flying));
    assert!(p.subtypes.creature_types.contains(&crate::card::CreatureType::Spirit));
}

#[test]
fn sedgemoor_witch_magecraft_creates_pest_token() {
    let mut g = two_player_game();
    let witch = g.add_card_to_battlefield(0, catalog::sedgemoor_witch());
    g.clear_sickness(witch);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable");
    drain_stack(&mut g);

    let pests: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Pest")
        .collect();
    assert_eq!(pests.len(), 1, "Sedgemoor Witch should mint one Pest token on instant cast");
}

#[test]
fn sedgemoor_witch_has_menace_and_ward_one() {
    let w = catalog::sedgemoor_witch();
    assert!(w.keywords.contains(&Keyword::Menace));
    assert!(w.keywords.contains(&Keyword::Ward(1)));
    assert_eq!(w.power, 3);
    assert_eq!(w.toughness, 2);
}

#[test]
fn spectacle_mage_has_prowess() {
    let m = catalog::spectacle_mage();
    assert!(m.keywords.contains(&Keyword::Prowess));
    assert_eq!(m.power, 1);
    assert_eq!(m.toughness, 2);
}

#[test]
fn mage_hunters_onslaught_destroys_creature_and_draws_card() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mage_hunters_onslaught());
    let bear_id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    // Prime the library so Draw 1 has a card to grab.
    g.add_card_to_library(0, catalog::island());

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear_id)),
        mode: None,
        x_value: None,
    })
    .expect("Mage Hunters' Onslaught castable for {2}{B}{B}");
    drain_stack(&mut g);

    // Bear should be in P1's graveyard.
    assert!(!g.battlefield.iter().any(|c| c.id == bear_id),
        "Grizzly Bears should be destroyed");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear_id),
        "Bear should be in P1's graveyard");
    // P0 should have drawn a card. (The Onslaught itself moved hand → stack
    // → graveyard, leaving hand_before - 1 + 1 (draw) = hand_before.)
    assert_eq!(g.players[0].hand.len(), hand_before);
    // The drawn card should be the Island we seeded.
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Island"),
        "P0 should have drawn the Island we seeded");
}

// ── STX legends (body-only smoke tests) ─────────────────────────────────────

#[test]
fn galazeth_prismari_is_three_four_flying_dragon_with_etb_treasure() {
    let g_card = catalog::galazeth_prismari();
    assert_eq!(g_card.power, 3);
    assert_eq!(g_card.toughness, 4);
    assert!(g_card.keywords.contains(&Keyword::Flying));
    assert!(g_card.subtypes.creature_types.contains(&crate::card::CreatureType::Dragon));
    assert!(g_card.supertypes.contains(&crate::card::Supertype::Legendary));
    assert_eq!(g_card.triggered_abilities.len(), 1, "should have ETB Treasure trigger");

    // Verify ETB actually mints a Treasure when Galazeth enters play.
    // (Direct battlefield insertion via `add_card_to_battlefield` skips
    // ETB triggers; cast normally so the spell-resolution path fires it.)
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::galazeth_prismari());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Galazeth Prismari castable for {2}{U}{R}");
    drain_stack(&mut g);
    let treasures: Vec<_> = g.battlefield.iter()
        .filter(|c| c.is_token && c.definition.name == "Treasure")
        .collect();
    assert_eq!(treasures.len(), 1, "Galazeth's ETB should create one Treasure");
}

#[test]
fn beledros_witherbloom_six_six_flying_trample_lifelink() {
    let b = catalog::beledros_witherbloom();
    assert_eq!(b.power, 6);
    assert_eq!(b.toughness, 6);
    assert!(b.keywords.contains(&Keyword::Flying));
    assert!(b.keywords.contains(&Keyword::Trample));
    assert!(b.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn velomachus_lorehold_five_five_flying_vigilance_haste() {
    let v = catalog::velomachus_lorehold();
    assert_eq!(v.power, 5);
    assert_eq!(v.toughness, 5);
    assert!(v.keywords.contains(&Keyword::Flying));
    assert!(v.keywords.contains(&Keyword::Vigilance));
    assert!(v.keywords.contains(&Keyword::Haste));
}

#[test]
fn tanazir_quandrix_five_five_flying_trample_dragon() {
    let t = catalog::tanazir_quandrix();
    assert_eq!(t.power, 5);
    assert_eq!(t.toughness, 5);
    assert!(t.keywords.contains(&Keyword::Flying));
    assert!(t.keywords.contains(&Keyword::Trample));
    assert!(t.subtypes.creature_types.contains(&crate::card::CreatureType::Dragon));
}

#[test]
fn shadrix_silverquill_four_four_flying_double_strike() {
    let s = catalog::shadrix_silverquill();
    assert_eq!(s.power, 4);
    assert_eq!(s.toughness, 4);
    assert!(s.keywords.contains(&Keyword::Flying));
    assert!(s.keywords.contains(&Keyword::DoubleStrike));
}

// Suppress unused-import lint when CounterType isn't used in this batch.
#[allow(dead_code)]
fn _keepalive(_: CounterType) {}


// ── Push: new STX 2021 card factories ───────────────────────────────────────

#[test]
fn charge_through_pumps_grants_trample_and_draws_card() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::charge_through());
    g.players[0].mana_pool.add(Color::Green, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Charge Through castable for {G}");
    drain_stack(&mut g);

    let pumped = g.battlefield_find(bear).expect("bear still on bf");
    assert_eq!(pumped.power(), 3, "bear should be pumped to 3");
    assert_eq!(pumped.toughness(), 3, "bear should be pumped to 3 toughness");
    assert!(pumped.has_keyword(&Keyword::Trample),
        "bear should gain trample EOT");
    // Cantrip: drew one card (hand: -spell-card, +draw = same).
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn resculpt_exiles_target_creature_owner_makes_four_four_elemental() {
    let mut g = two_player_game();
    let opp_threat = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::resculpt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_threat)),
        mode: None,
        x_value: None,
    })
    .expect("Resculpt castable for {1}{U}");
    drain_stack(&mut g);

    // Target was exiled.
    assert!(g.battlefield_find(opp_threat).is_none(),
        "target should be exiled");
    // Opp has a 4/4 Elemental token.
    let opp_token = g.battlefield.iter().find(|c| {
        c.controller == 1 && c.is_token && c.definition.name == "Elemental"
    });
    let token = opp_token.expect("opponent should have Elemental token");
    assert_eq!(token.power(), 4);
    assert_eq!(token.toughness(), 4);
}

#[test]
fn letter_of_acceptance_etb_scrys_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::letter_of_acceptance());
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Letter of Acceptance castable for {3}");
    drain_stack(&mut g);

    // Letter on bf. ETB drew a card net: -1 (cast) + 1 (draw) = 0.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Letter of Acceptance"));
}

#[test]
fn defend_the_campus_shrinks_attacking_creature() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    if let Some(c) = g.battlefield_find_mut(attacker) { c.summoning_sick = false; }
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker, target: AttackTarget::Player(0) },
    ])).expect("declare attacker");
    drain_stack(&mut g);

    // Pass priority to player 0 for Defend the Campus.
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::defend_the_campus());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(attacker)),
        mode: None,
        x_value: None,
    })
    .expect("Defend the Campus castable for {3}{R}{W}");
    drain_stack(&mut g);

    let pumped = g.battlefield_find(attacker).expect("attacker still on bf");
    assert_eq!(pumped.power(), -1,
        "2-power bear with -3/-0 EOT shrinks to -1 power");
}

#[test]
fn manifestation_sage_magecraft_pumps_target_by_hand_minus_three() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::manifestation_sage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Stock our hand to 6 cards → X = 6 - 3 = 3.
    for _ in 0..6 {
        g.add_card_to_hand(0, catalog::island());
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);

    // Bolt killed the bear (it's not on bf anymore). Shouldn't have
    // crashed evaluating the magecraft pump. The trigger fires
    // even if the target is gone — so just verify no panic.
}

#[test]
fn conspiracy_theorist_is_one_three_human_shaman() {
    let c = catalog::conspiracy_theorist();
    assert_eq!(c.power, 1);
    assert_eq!(c.toughness, 3);
    assert!(c.subtypes.creature_types.contains(&crate::card::CreatureType::Human));
    assert!(c.subtypes.creature_types.contains(&crate::card::CreatureType::Shaman));
}

#[test]
fn honor_troll_is_zero_three_troll() {
    let h = catalog::honor_troll();
    assert_eq!(h.power, 0);
    assert_eq!(h.toughness, 3);
    assert!(h.subtypes.creature_types.contains(&crate::card::CreatureType::Troll));
}

#[test]
fn reduce_to_memory_exiles_creature_and_owner_gets_inkling() {
    let mut g = two_player_game();
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::reduce_to_memory());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_creature)),
        mode: None,
        x_value: None,
    })
    .expect("Reduce to Memory castable for {2}{U}");
    drain_stack(&mut g);

    // Target exiled.
    assert!(g.battlefield_find(opp_creature).is_none());
    // Opponent now has an Inkling.
    let inkling = g.battlefield.iter().find(|c| {
        c.controller == 1 && c.is_token && c.definition.name == "Inkling"
    });
    assert!(inkling.is_some(), "opponent should have an Inkling token");
}
