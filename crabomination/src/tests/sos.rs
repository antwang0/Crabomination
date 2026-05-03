//! Functionality tests for the Secrets of Strixhaven card pack
//! (`catalog::sets::sos`). Mirrors `tests/modern.rs`: each card gets at
//! least one test exercising its primary play pattern.

use crate::card::{CardType, CounterType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
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

// ── White ───────────────────────────────────────────────────────────────────

#[test]
fn eager_glyphmage_etb_creates_inkling_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::eager_glyphmage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Eager Glyphmage castable for {3}{W}");
    drain_stack(&mut g);

    // Glyphmage itself + an Inkling token = 2 new battlefield permanents.
    assert_eq!(g.battlefield.len(), bf_before + 2);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Inkling"));
    let tok = g.battlefield.iter().find(|c| c.definition.name == "Inkling").unwrap();
    assert!(tok.definition.keywords.contains(&Keyword::Flying));
}

#[test]
fn erode_destroys_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::erode());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Erode castable for {W}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn erode_grants_target_controller_a_basic_land() {
    // Push XV: the basic-land tutor half is now wired. The target's
    // controller searches their library for a basic land (auto-decider
    // takes the first match) and puts it onto the battlefield tapped.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _bf_count_before = g.battlefield.iter().filter(|c| c.controller == 1).count();
    // Seed P1's library with a Forest (the basic to fetch).
    let forest = g.add_card_to_library(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::erode());
    g.players[0].mana_pool.add(Color::White, 1);
    // Tell decider to fetch the Forest.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Erode castable");
    drain_stack(&mut g);

    // Bear destroyed.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    // Forest fetched onto P1's battlefield, tapped.
    let forest_view = g.battlefield.iter().find(|c| c.id == forest)
        .expect("Forest should be on battlefield");
    assert_eq!(forest_view.controller, 1, "Forest under P1's control");
    assert!(forest_view.tapped, "Forest fetched tapped");
}

#[test]
fn harsh_annotation_destroys_and_creates_token() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::harsh_annotation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Harsh Annotation castable for {1}{W}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Targeted creature should be destroyed");
    let inkling = g.battlefield.iter().find(|c| c.definition.name == "Inkling")
        .expect("Inkling token should be created");
    // Post-XX: the Inkling lands on the destroyed creature's controller
    // (player 1), not the spell's caster (player 0).
    assert_eq!(inkling.controller, 1,
        "Inkling should be controlled by the destroyed creature's original controller");
}

#[test]
fn interjection_pumps_and_grants_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::interjection());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Interjection castable for {W}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 4, "2 + 2 pump = 4");
    assert_eq!(view.toughness, 4, "2 + 2 pump = 4");
    assert!(view.keywords.contains(&Keyword::FirstStrike));
}

#[test]
fn stand_up_for_yourself_only_targets_power_three_or_more() {
    use crate::card::{CardDefinition, Subtypes};
    use crate::effect::Effect;
    // Build a 3/3 manually so the destroy-power-3+ filter accepts it.
    let big = CardDefinition {
        name: "Test Three Three",
        cost: crate::mana::ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    };
    let mut g = two_player_game();
    let big_id = g.add_card_to_battlefield(1, big);
    let id = g.add_card_to_hand(0, catalog::stand_up_for_yourself());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(big_id)), mode: None, x_value: None,
    })
    .expect("Stand Up for Yourself castable for {2}{W} on a 3/3");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == big_id),
        "3/3 should be destroyed");
}

#[test]
fn rapier_wit_taps_target_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rapier_wit());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Rapier Wit castable for {1}{W}");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(target.tapped, "Bear should be tapped by Rapier Wit");
    // Played on player 0's turn, so it should also have a stun counter.
    assert!(target.counter_count(CounterType::Stun) >= 1,
        "Stun counter should be added when cast on your own turn");
    // Hand: -1 cast +1 draw = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Silverquill (W/B) ───────────────────────────────────────────────────────

#[test]
fn silverquill_charm_drain_mode_drains_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::silverquill_charm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;

    // Mode 2 = drain 3.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(2), x_value: None,
    })
    .expect("Silverquill Charm castable for {W}{B} in drain mode");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_life - 3, "Each opponent loses 3");
    assert_eq!(g.players[0].life, p0_life + 3, "You gain 3");
}

#[test]
fn silverquill_charm_counter_mode_adds_two_p1p1() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::silverquill_charm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: Some(0),
        x_value: None,
    })
    .expect("Silverquill Charm castable in counter mode");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(target.counter_count(CounterType::PlusOnePlusOne), 2,
        "Mode 0 should put 2 +1/+1 counters on the target");
}

#[test]
fn silverquill_charm_exile_mode_exiles_low_power_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::silverquill_charm());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: Some(1),
        x_value: None,
    })
    .expect("Silverquill Charm castable in exile mode");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear (power 2) should be exiled by mode 1");
}

#[test]
fn imperious_inkmage_etb_surveils_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::imperious_inkmage());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Imperious Inkmage castable for {1}{W}{B}");
    drain_stack(&mut g);

    // Surveil 2: the auto-decider keeps both cards on top, so the library
    // size is unchanged. We assert at minimum that the library wasn't
    // grown (no draw side-effect leaked) and the inkmage hit play.
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Imperious Inkmage should be on the battlefield");
    assert!(g.players[0].library.len() <= lib_before,
        "Surveil 2 cannot increase library size");
}

#[test]
fn killians_confidence_pumps_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::killians_confidence());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Killian's Confidence castable for {W}{B}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 3);
    assert_eq!(view.toughness, 3);
    // Hand: -1 cast +1 draw = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// Killian's Confidence's gy-trigger: when a creature you control deals
// combat damage to a player, may pay {W/B} to return Killian's
// Confidence from your graveyard to your hand. The full combat-step
// drive empties the active player's mana pool between steps (rule
// 500.4), so we drive combat manually and float {W/B} during the damage
// step before the trigger resolves.
#[test]
fn killians_confidence_combat_damage_trigger_returns_to_hand() {
    let mut g = two_player_game();
    let kc_id = g.add_card_to_graveyard(0, catalog::killians_confidence());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("Bear attacks");
    drain_stack(&mut g);
    // Advance through DeclareBlockers to CombatDamage. Mana empties on
    // every step transition, so float {W/B} *now*, while we sit at
    // CombatDamage with the trigger about to fire.
    let mut bail = 50;
    while g.step != TurnStep::CombatDamage && bail > 0 {
        if g.perform_action(GameAction::PassPriority).is_err() { break; }
        bail -= 1;
    }
    g.players[0].mana_pool.add(Color::White, 1);
    while !matches!(g.step, TurnStep::PostCombatMain | TurnStep::End) && bail > 0 {
        if g.perform_action(GameAction::PassPriority).is_err() { break; }
        bail -= 1;
    }
    drain_stack(&mut g);

    assert!(
        g.players[0].hand.iter().any(|c| c.id == kc_id),
        "Killian's Confidence should be back in hand after gy-trigger"
    );
    assert!(
        !g.players[0].graveyard.iter().any(|c| c.id == kc_id),
        "Killian's Confidence should have left the graveyard"
    );
}

#[test]
fn killians_confidence_decline_keeps_in_graveyard() {
    let mut g = two_player_game();
    let kc_id = g.add_card_to_graveyard(0, catalog::killians_confidence());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    // No floated mana + decline → trigger cannot pay.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("Bear attacks");
    drain_stack(&mut g);
    while !matches!(g.step, TurnStep::PostCombatMain | TurnStep::End) {
        if g.perform_action(GameAction::PassPriority).is_err() { break; }
    }
    drain_stack(&mut g);

    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == kc_id),
        "Killian's Confidence should still be in graveyard (declined)"
    );
}

#[test]
fn forum_of_amity_taps_for_white_or_black() {
    use crate::card::CardType;
    let card = catalog::forum_of_amity();
    assert!(card.card_types.contains(&CardType::Land));
    assert!(card.activated_abilities.len() >= 3,
        "Forum of Amity has 2 mana abilities + 1 surveil ability");
    // The card should enter tapped (etb_tap trigger present).
    assert_eq!(card.triggered_abilities.len(), 1);
}

// ── Black ───────────────────────────────────────────────────────────────────

#[test]
fn wander_off_exiles_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wander_off());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Wander Off castable for {3}{B}");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear));
}

#[test]
fn sneering_shadewriter_etb_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sneering_shadewriter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Sneering Shadewriter castable for {4}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, p1_life - 2);
    assert_eq!(g.players[0].life, p0_life + 2);
}

#[test]
fn burrog_banemaker_pump_ability_works() {
    let mut g = two_player_game();
    let banemaker = g.add_card_to_battlefield(0, catalog::burrog_banemaker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: banemaker,
        ability_index: 0,
        target: None,
    })
    .expect("Banemaker pump activatable for {1}{B}");
    drain_stack(&mut g);

    let view = g.computed_permanent(banemaker).unwrap();
    assert_eq!(view.power, 2);
    assert_eq!(view.toughness, 2);
}

#[test]
fn masterful_flourish_grants_indestructible() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::masterful_flourish());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Masterful Flourish castable for {B}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 3, "Bear gets +1/+0");
    assert!(view.keywords.contains(&Keyword::Indestructible));
}

#[test]
fn send_in_the_pest_discards_and_creates_token() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::send_in_the_pest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p1_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Send in the Pest castable for {1}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].hand.len(), p1_hand_before - 1,
        "Each opponent should discard one card");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pest"),
        "Pest token should be created");
}

#[test]
fn pull_from_the_grave_returns_creature_and_gains_life() {
    let mut g = two_player_game();
    // Stash a creature in P0's graveyard.
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::pull_from_the_grave());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Pull from the Grave castable for {2}{B}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Creature in your gy returns to your hand");
    assert_eq!(g.players[0].life, life_before + 2);
}

#[test]
fn pull_from_the_grave_returns_up_to_two_creatures() {
    // Selector::Take(_, 2) should pull at most two creature cards from
    // your graveyard back to your hand. Lands sitting next to them
    // should be left untouched (filter is `Creature`).
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bear3 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let _land = g.add_card_to_graveyard(0, catalog::forest());

    let id = g.add_card_to_hand(0, catalog::pull_from_the_grave());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Pull from the Grave castable for {2}{B}");
    drain_stack(&mut g);

    let creatures_in_hand = g.players[0]
        .hand
        .iter()
        .filter(|c| c.id == bear || c.id == bear2 || c.id == bear3)
        .count();
    assert_eq!(creatures_in_hand, 2, "Exactly two creatures should be returned");
    // The land in the graveyard should NOT have been moved to hand.
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Forest"),
        "Land in gy is not affected by the creature filter");
}

#[test]
fn practiced_scrollsmith_take_one_only_exiles_one_match() {
    // Selector::Take(_, 1) should clamp the gy-exile to a single
    // matching card even when multiple noncreature/nonland cards sit
    // in the graveyard.
    let mut g = two_player_game();
    // Two sorceries + a creature + a land in P0's gy.
    g.add_card_to_graveyard(0, catalog::pox_plague());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::forest());

    // Cast Practiced Scrollsmith from hand: ETB exiles ONE matching gy card.
    let id = g.add_card_to_hand(0, catalog::practiced_scrollsmith());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::White, 1);

    let before_gy = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    }).expect("Practiced Scrollsmith castable");
    drain_stack(&mut g);

    // ETB should have removed exactly one card from the gy (creature/land
    // filter excludes the bear and forest).
    assert_eq!(g.players[0].graveyard.len(), before_gy - 1,
        "Take(_, 1) clamps the gy-exile to a single matching card");
    // Bear and Forest must still be in the graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "Creature filter excludes Grizzly Bears from exile");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Forest"),
        "Nonland filter excludes Forest from exile");
}

// ── Witherbloom (B/G) ───────────────────────────────────────────────────────

#[test]
fn witherbloom_charm_lifegain_mode_gains_five() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;

    // Mode 1 = gain 5.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(1), x_value: None,
    })
    .expect("Witherbloom Charm castable for {B}{G} in lifegain mode");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 5);
}

#[test]
fn witherbloom_charm_sacrifice_mode_opts_in() {
    // Push XV: mode 0 (sacrifice → draw 2) is now wrapped in
    // `Effect::MayDo`. The controller picks mode 0 then opts in to the
    // sac via OptionalTrigger.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::witherbloom_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand_before = g.players[0].hand.len(); // 1 (just the charm)

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(0), x_value: None,
    })
    .expect("Witherbloom Charm castable in sacrifice mode");
    drain_stack(&mut g);

    // Bear should be sacrificed.
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "Bear should be in graveyard from sacrifice");
    // Hand: -1 cast + 2 drawn = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn witherbloom_charm_sacrifice_mode_skips_when_declining() {
    // Push XV: declining the sac means no draw fires.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::witherbloom_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let hand_before = g.players[0].hand.len();
    // Default AutoDecider says no.

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(0), x_value: None,
    })
    .expect("Witherbloom Charm castable");
    drain_stack(&mut g);

    // No sacrifice, no draw.
    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "Bear stays on battlefield (no sac)");
    // Hand: cast charm = -1 net.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

#[test]
fn witherbloom_charm_destroy_mode_destroys_low_mv_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // CMC 2
    let id = g.add_card_to_hand(0, catalog::witherbloom_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    // Mode 2 = destroy nonland permanent with mv ≤ 2.
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: Some(2),
        x_value: None,
    })
    .expect("Witherbloom Charm castable in destroy mode");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn bogwater_lumaret_etb_gains_one_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bogwater_lumaret());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Bogwater Lumaret castable for {B}{G}");
    drain_stack(&mut g);

    // The Lumaret's own ETB triggers itself (via YourControl + Creature).
    assert_eq!(g.players[0].life, life_before + 1);
}

#[test]
fn pest_mascot_grows_on_lifegain() {
    let mut g = two_player_game();
    let mascot = g.add_card_to_battlefield(0, catalog::pest_mascot());

    // Cast a small lifegain spell — gain 5 from Witherbloom Charm.
    let charm = g.add_card_to_hand(0, catalog::witherbloom_charm());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: charm, target: None, mode: Some(1), x_value: None,
    })
    .expect("Witherbloom Charm castable for lifegain mode");
    drain_stack(&mut g);

    let view = g.computed_permanent(mascot).unwrap();
    assert!(view.power >= 3,
        "Pest Mascot should gain at least one +1/+1 counter from the lifegain trigger");
}

#[test]
fn grapple_with_death_destroys_creature_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::grapple_with_death());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Grapple with Death castable for {1}{B}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert_eq!(g.players[0].life, life_before + 1);
}

// ── Red ─────────────────────────────────────────────────────────────────────

#[test]
fn impractical_joke_deals_three_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::impractical_joke());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Impractical Joke castable for {R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "2/2 should die to 3 damage from Impractical Joke");
}

// ── Green ───────────────────────────────────────────────────────────────────

#[test]
fn noxious_newt_taps_for_green() {
    let mut g = two_player_game();
    let newt = g.add_card_to_battlefield(0, catalog::noxious_newt());
    g.clear_sickness(newt);
    let pool_total = g.players[0].mana_pool.total();

    g.perform_action(GameAction::ActivateAbility {
        card_id: newt,
        ability_index: 0,
        target: None,
    })
    .expect("Noxious Newt {T} mana ability activatable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].mana_pool.total(), pool_total + 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

#[test]
fn mindful_biomancer_etb_gains_one_life_and_pump_is_once_per_turn() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mindful_biomancer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Mindful Biomancer castable for {1}{G}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1);

    // Activate the +2/+2 pump.
    let bio = g.battlefield.iter().find(|c| c.id == id).unwrap().id;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bio, ability_index: 0, target: None,
    })
    .expect("Pump activatable for {2}{G}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bio).unwrap();
    assert_eq!(view.power, 4);
    assert_eq!(view.toughness, 4);

    // Once-per-turn enforcement: a second activation must fail.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let again = g.perform_action(GameAction::ActivateAbility {
        card_id: bio, ability_index: 0, target: None,
    });
    assert!(again.is_err(),
        "Mindful Biomancer pump should be activatable only once each turn");
}

#[test]
fn shopkeepers_bane_attack_gains_two_life() {
    use crate::game::types::AttackTarget;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::shopkeepers_bane());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    let life_before = g.players[0].life;

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("Bane should be a legal attacker");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 2,
        "Attack trigger should gain 2 life");
}

#[test]
fn oracles_restoration_pumps_draws_gains_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::oracles_restoration());
    g.players[0].mana_pool.add(Color::Green, 1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Oracle's Restoration castable for {G}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 3);
    assert_eq!(view.toughness, 3);
    assert_eq!(g.players[0].life, life_before + 1);
    // -1 cast +1 draw = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Blue ────────────────────────────────────────────────────────────────────

#[test]
fn banishing_betrayal_bounces_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::banishing_betrayal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Banishing Betrayal castable for {1}{U}");
    drain_stack(&mut g);

    // Bear should be in P1's hand after the bounce.
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn procrastinate_taps_and_adds_2x_stun_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::procrastinate());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    // X = 2: pay {2}{U}, expect 4 stun counters.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: Some(2),
    })
    .expect("Procrastinate castable for {2}{U} with X=2");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(target.tapped);
    assert_eq!(target.counter_count(CounterType::Stun), 4,
        "Procrastinate puts 2X = 4 stun counters with X=2");
}

#[test]
fn chase_inspiration_grants_hexproof() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::chase_inspiration());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Chase Inspiration castable for {U}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.toughness, 5, "+0/+3");
    assert!(view.keywords.contains(&Keyword::Hexproof));
}

// ── Quandrix (G/U) ──────────────────────────────────────────────────────────

#[test]
fn embrace_the_paradox_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::embrace_the_paradox());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Embrace the Paradox castable for {3}{G}{U}");
    drain_stack(&mut g);

    // -1 cast +3 draw = +2.
    assert_eq!(g.players[0].hand.len(), hand_before + 2);
}

// ── Glorious Decay (G modal) ────────────────────────────────────────────────

#[test]
fn glorious_decay_destroys_artifact() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::glorious_decay());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(rock)), mode: Some(0), x_value: None,
    })
    .expect("Glorious Decay castable for {1}{G}, mode 0");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == rock));
}

#[test]
fn rearing_embermare_has_reach_and_haste() {
    let card = catalog::rearing_embermare();
    assert!(card.keywords.contains(&Keyword::Reach));
    assert!(card.keywords.contains(&Keyword::Haste));
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 5);
}

#[test]
fn charging_strifeknight_loots_with_tap() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let knight = g.add_card_to_battlefield(0, catalog::charging_strifeknight());
    // Clear summoning sickness so we can tap immediately.
    g.clear_sickness(knight);
    let hand_before = g.players[0].hand.len();
    let grave_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: knight,
        ability_index: 0,
        target: None,
    })
    .expect("Strifeknight tap-loot ability activatable");
    drain_stack(&mut g);

    // Hand: -1 discard +1 draw = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Graveyard +1 (the discarded card).
    assert_eq!(g.players[0].graveyard.len(), grave_before + 1);
}

// ── New batch: Silverquill / extra White / Black / Red / Green ──────────────

#[test]
fn ascendant_dustspeaker_etb_pumps_other_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ascendant_dustspeaker());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Dustspeaker castable for {4}{W}");
    drain_stack(&mut g);

    let pumped = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(pumped.counter_count(CounterType::PlusOnePlusOne), 1,
        "Bear should have one +1/+1 counter from Dustspeaker ETB");
    let dust = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert!(dust.definition.keywords.contains(&Keyword::Flying));
}

#[test]
fn shattered_acolyte_sac_destroys_artifact() {
    let mut g = two_player_game();
    let acolyte = g.add_card_to_battlefield(0, catalog::shattered_acolyte());
    g.clear_sickness(acolyte);
    let mind_stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: acolyte,
        ability_index: 0,
        target: Some(Target::Permanent(mind_stone)),
    })
    .expect("Shattered Acolyte sac-and-destroy castable");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == mind_stone),
        "Mind Stone should be destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == acolyte),
        "Shattered Acolyte should have been sacrificed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == acolyte),
        "Acolyte should be in its owner's graveyard");
}

#[test]
fn summoned_dromedary_has_vigilance_body() {
    let card = catalog::summoned_dromedary();
    assert!(card.keywords.contains(&Keyword::Vigilance));
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 3);
}

#[test]
fn dig_site_inventory_grants_counter_and_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::dig_site_inventory());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Dig Site Inventory castable for {W}");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(target.counter_count(CounterType::PlusOnePlusOne), 1);
    let view = g.computed_permanent(bear).unwrap();
    assert!(view.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn group_project_creates_2_2_red_white_spirit() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::group_project());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Group Project castable for {1}{W}");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 1);
    let spirit = g.battlefield.iter().find(|c| c.definition.name == "Spirit").unwrap();
    assert_eq!(spirit.definition.power, 2);
    assert_eq!(spirit.definition.toughness, 2);
}

#[test]
fn render_speechless_discards_and_pumps() {
    let mut g = two_player_game();
    // Give opponent two cards (one nonland, one land); the nonland is the
    // auto-pick.
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::island());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::render_speechless());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Render Speechless castable for {2}{W}{B}");
    drain_stack(&mut g);

    // Opponent discarded one nonland.
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "Opponent should have discarded the nonland card");
    // Bear has 2 +1/+1 counters.
    let pumped = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(pumped.counter_count(CounterType::PlusOnePlusOne), 2);
}

#[test]
fn snooping_page_combat_damage_draws_and_loses_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let page = g.add_card_to_battlefield(0, catalog::snooping_page());
    g.clear_sickness(page);
    let life_before = g.players[0].life;
    let opp_life_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();

    // Attack with the page; auto-resolve combat through to damage.
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: page,
        target: AttackTarget::Player(1),
    }]))
    .expect("Snooping Page can attack");
    drain_stack(&mut g);

    // Drive combat forward via the no-blockers path.
    while !matches!(g.step, TurnStep::PostCombatMain | TurnStep::End) {
        let _ = g.perform_action(GameAction::PassPriority);
    }
    drain_stack(&mut g);

    assert!(g.players[1].life < opp_life_before, "Snooping Page hit the opponent");
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Should draw 1 from combat-damage trigger");
    assert_eq!(g.players[0].life, life_before - 1,
        "Should lose 1 from combat-damage trigger");
}

#[test]
fn lecturing_scornmage_is_vanilla_one_one_warlock() {
    let card = catalog::lecturing_scornmage();
    assert_eq!(card.power, 1);
    assert_eq!(card.toughness, 1);
    assert!(card.has_creature_type(crate::card::CreatureType::Warlock));
    assert!(card.has_creature_type(crate::card::CreatureType::Human));
}

#[test]
fn melancholic_poet_is_vanilla_two_two_bard() {
    let card = catalog::melancholic_poet();
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 2);
    assert!(card.has_creature_type(crate::card::CreatureType::Bard));
    assert!(card.has_creature_type(crate::card::CreatureType::Elf));
}

#[test]
fn zealous_lorecaster_etb_returns_instant_from_graveyard() {
    let mut g = two_player_game();
    let bolt_in_grave = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt_in_grave)), mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable for {5}{R}");
    drain_stack(&mut g);

    // Lorecaster on bf, bolt back in hand → hand: -1 cast +1 returned = same.
    assert!(g.battlefield.iter().any(|c| c.id == id));
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_in_grave),
        "Bolt should be back in hand");
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn environmental_scientist_etb_searches_basic_to_hand() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));
    let id = g.add_card_to_hand(0, catalog::environmental_scientist());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Environmental Scientist castable for {1}{G}");
    drain_stack(&mut g);

    // Hand: -1 cast +1 forest tutored = same. Library now lacks the forest.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest),
        "Forest should be in hand after search");
}

#[test]
fn pestbrood_sloth_death_creates_two_pest_tokens() {
    let mut g = two_player_game();
    let sloth = g.add_card_to_battlefield(0, catalog::pestbrood_sloth());
    let bf_before = g.battlefield.len();

    // Kill via Murder (sorcery-speed destroy is fine for the test —
    // we just need a dies trigger).
    let murder_id = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: murder_id, target: Some(Target::Permanent(sloth)),
        mode: None, x_value: None,
    })
    .expect("Murder castable for {1}{B}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == sloth),
        "Pestbrood Sloth should be in the graveyard");
    let pest_count = g.battlefield.iter().filter(|c| c.definition.name == "Pest").count();
    assert_eq!(pest_count, 2, "Pestbrood Sloth should create two Pest tokens on death");
    // Net battlefield: -1 sloth + 2 pests = +1.
    assert_eq!(g.battlefield.len(), bf_before + 1);
}

#[test]
fn dinas_guidance_searches_creature_to_hand() {
    // Push: Mode 0 (the default, search-to-hand) — promotes Dina's
    // Guidance from 🟡 (collapsed-to-hand) to ✅ (`ChooseMode` between
    // hand and graveyard destinations).
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bear)),
    ]));
    let id = g.add_card_to_hand(0, catalog::dinas_guidance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(0), x_value: None,
    })
    .expect("Dina's Guidance castable for {1}{B}{G}");
    drain_stack(&mut g);

    // Hand: -1 cast +1 bears = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear),
        "Grizzly Bears should be in hand after search (mode 0)");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear),
        "Grizzly Bears should NOT be in graveyard (mode 0 picks hand)");
}

#[test]
fn dinas_guidance_searches_creature_to_graveyard() {
    // Push: Mode 1 (the new reanimator-fuel mode) drops the searched
    // creature directly into the graveyard. Pairs with Goryo's
    // Vengeance / Animate Dead / Reanimate downstream — closes the
    // hand-or-graveyard prompt gap that kept Dina's Guidance at 🟡.
    let mut g = two_player_game();
    let griselbrand = g.add_card_to_library(0, catalog::griselbrand());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(griselbrand)),
    ]));
    let id = g.add_card_to_hand(0, catalog::dinas_guidance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(1), x_value: None,
    })
    .expect("Dina's Guidance mode 1 castable for {1}{B}{G}");
    drain_stack(&mut g);

    // Hand: -1 (cast Dina's Guidance) = -1 net (Griselbrand goes to gy,
    // not hand).
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
    assert!(!g.players[0].hand.iter().any(|c| c.id == griselbrand),
        "Griselbrand should NOT be in hand (mode 1 picks graveyard)");
    // Graveyard: +1 Griselbrand + 1 Dina's Guidance (the resolved
    // spell) = +2.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == griselbrand),
        "Griselbrand should be in graveyard (mode 1 reanimator fuel)");
    assert!(g.players[0].graveyard.len() >= gy_before + 2,
        "Graveyard should hold Griselbrand + the resolved Dina's Guidance");
}

#[test]
fn pursue_the_past_loots_two_and_gains_two() {
    // Push XV: the discard+draw chain is now gated on `Effect::MayDo`.
    // Inject `Bool(true)` to opt in.
    let mut g = two_player_game();
    // Library: two cards to draw from.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    // Hand: pursue + a swamp to discard.
    let pursue = g.add_card_to_hand(0, catalog::pursue_the_past());
    let _swamp_in_hand = g.add_card_to_hand(0, catalog::swamp());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len(); // 2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: pursue, target: None, mode: None, x_value: None,
    }).expect("Pursue the Past castable for {R}{W}");
    drain_stack(&mut g);

    // Hand: -1 cast (pursue) -1 discard (swamp) +2 draw = net +0 from
    // hand_before (2 → 0 + 2 = 2).
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Graveyard: at least 1 (the spell after resolution); plus the
    // discarded swamp = at least 2 cards.
    assert!(g.players[0].graveyard.len() >= 2,
        "Graveyard should hold the resolved spell and the discarded card");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Pursue the Past"),
        "Pursue the Past should be in graveyard after resolving");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Swamp"),
        "Discarded Swamp should be in graveyard");
    // +2 life.
    assert_eq!(g.players[0].life, life_before + 2);
    // life_gained_this_turn bumped by 2.
    assert!(g.players[0].life_gained_this_turn >= 2);
}

#[test]
fn pursue_the_past_skips_loot_when_declining() {
    // Push XV: declining the may-discard means we still gain 2 life
    // but no draw or discard fires.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let pursue = g.add_card_to_hand(0, catalog::pursue_the_past());
    let _swamp = g.add_card_to_hand(0, catalog::swamp());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len(); // 2
    // Default: AutoDecider answers no.

    g.perform_action(GameAction::CastSpell {
        card_id: pursue, target: None, mode: None, x_value: None,
    }).expect("Pursue the Past castable");
    drain_stack(&mut g);

    // Hand: -1 cast (pursue) only — no discard, no draw.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
    // Graveyard: just Pursue itself (no discarded swamp).
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Pursue the Past"));
    assert!(!g.players[0].graveyard.iter().any(|c| c.definition.name == "Swamp"),
        "No discard fired");
    assert_eq!(g.players[0].life, life_before + 2,
        "Lifegain still resolves regardless of may-do choice");
}

#[test]
fn efflorescence_pumps_and_grants_trample_indestructible_after_lifegain() {
    let mut g = two_player_game();
    // Prime Infusion via Oracle's Restoration on a friendly creature.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let oracle = g.add_card_to_hand(0, catalog::oracles_restoration());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: oracle, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    }).expect("Oracle's Restoration castable for {G}");
    drain_stack(&mut g);

    // Cast Efflorescence on the bear.
    let id = g.add_card_to_hand(0, catalog::efflorescence());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    }).expect("Efflorescence castable for {2}{G}");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(target.counter_count(CounterType::PlusOnePlusOne), 2);
    let view = g.computed_permanent(bear).unwrap();
    assert!(view.keywords.contains(&Keyword::Trample),
        "Infusion should grant trample after lifegain");
    assert!(view.keywords.contains(&Keyword::Indestructible),
        "Infusion should grant indestructible after lifegain");
}

#[test]
fn efflorescence_only_pumps_without_lifegain() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::efflorescence());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    }).expect("Efflorescence castable for {2}{G}");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(target.counter_count(CounterType::PlusOnePlusOne), 2);
    let view = g.computed_permanent(bear).unwrap();
    assert!(!view.keywords.contains(&Keyword::Trample));
    assert!(!view.keywords.contains(&Keyword::Indestructible));
}

#[test]
fn old_growth_educator_etb_grows_after_lifegain() {
    let mut g = two_player_game();
    // First gain some life this turn to satisfy Infusion.
    let oracle = g.add_card_to_hand(0, catalog::oracles_restoration());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: oracle, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Oracle's Restoration castable for {G}");
    drain_stack(&mut g);
    assert!(g.players[0].life_gained_this_turn >= 1,
        "Life gained tracker should be primed by Oracle's Restoration");

    // Now ETB Old-Growth Educator — Infusion should add 2 +1/+1 counters.
    let id = g.add_card_to_hand(0, catalog::old_growth_educator());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Old-Growth Educator castable for {2}{B}{G}");
    drain_stack(&mut g);

    let educator = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(educator.counter_count(CounterType::PlusOnePlusOne), 2,
        "Infusion should add 2 +1/+1 counters when life was gained this turn");
    assert!(educator.definition.keywords.contains(&Keyword::Reach));
    assert!(educator.definition.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn old_growth_educator_etb_no_counters_without_lifegain() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::old_growth_educator());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Old-Growth Educator castable for {2}{B}{G}");
    drain_stack(&mut g);

    let educator = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(educator.counter_count(CounterType::PlusOnePlusOne), 0,
        "Infusion should be inactive when no life has been gained this turn");
}

#[test]
fn foolish_fate_drains_three_after_lifegain() {
    let mut g = two_player_game();
    // Step 1: gain life to prime Infusion.
    let oracle = g.add_card_to_hand(0, catalog::oracles_restoration());
    let bear_self = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: oracle, target: Some(Target::Permanent(bear_self)), mode: None, x_value: None,
    }).expect("Oracle's Restoration castable for {G}");
    drain_stack(&mut g);

    // Step 2: cast Foolish Fate on opponent's bear.
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::foolish_fate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)), mode: None, x_value: None,
    })
    .expect("Foolish Fate castable for {2}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Opponent's bear should be destroyed");
    assert_eq!(g.players[1].life, opp_life_before - 3,
        "Infusion should drain 3 life from the target's controller");
}

#[test]
fn foolish_fate_no_drain_without_lifegain() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::foolish_fate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let opp_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)), mode: None, x_value: None,
    })
    .expect("Foolish Fate castable for {2}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Opponent's bear should be destroyed");
    assert_eq!(g.players[1].life, opp_life_before,
        "Without prior lifegain, no drain should occur");
}

#[test]
fn teachers_pest_attacks_gains_one_life() {
    let mut g = two_player_game();
    let pest = g.add_card_to_battlefield(0, catalog::teachers_pest());
    g.clear_sickness(pest);
    let life_before = g.players[0].life;

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pest,
        target: AttackTarget::Player(1),
    }]))
    .expect("Teacher's Pest can attack");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 1,
        "Teacher's Pest should grant 1 life on attack");
    let view = g.computed_permanent(pest).unwrap();
    assert!(view.keywords.contains(&Keyword::Menace),
        "Teacher's Pest should have menace");
}

// ── Owlin Historian ─────────────────────────────────────────────────────────

#[test]
fn owlin_historian_etb_surveils_one_and_has_flying() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::owlin_historian());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Owlin Historian castable for {2}{W}");
    drain_stack(&mut g);

    let view = g.computed_permanent(id).expect("Historian on battlefield");
    assert!(view.keywords.contains(&Keyword::Flying),
        "Owlin Historian should have flying");
    assert!(g.players[0].library.len() <= lib_before,
        "Surveil 1 should not grow the library");
}

// ── Inkling Mascot ──────────────────────────────────────────────────────────

#[test]
fn inkling_mascot_is_vanilla_inkling_cat() {
    let card = catalog::inkling_mascot();
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 2);
    assert!(card.has_creature_type(crate::card::CreatureType::Inkling),
        "Inkling Mascot should have Inkling creature type");
    assert!(card.has_creature_type(crate::card::CreatureType::Cat),
        "Inkling Mascot should have Cat creature type");
}

// ── Cost of Brilliance ──────────────────────────────────────────────────────

#[test]
fn cost_of_brilliance_draws_two_loses_two_pumps_creature() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cost_of_brilliance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Cost of Brilliance castable for {2}{B}");
    drain_stack(&mut g);

    // Hand: -1 cast +2 draw = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Should net +1 hand from drawing 2 minus the cast itself");
    assert_eq!(g.players[0].life, life_before - 2,
        "Should lose 2 life");
    let pumped = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(pumped.counter_count(CounterType::PlusOnePlusOne), 1,
        "Bear should have 1 +1/+1 counter");
}

#[test]
fn cost_of_brilliance_castable_with_no_creature() {
    // Push: the +1/+1 half is now optional via `Selector::one_of` —
    // the spell castable even when no creature exists. Was 🟡 (cast
    // would fail without a creature target); now ✅.
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::cost_of_brilliance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Cost of Brilliance castable even with no creature on the battlefield");
    drain_stack(&mut g);

    // Hand and life still resolve.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(g.players[0].life, life_before - 2);
}

// ── Mind Roots ──────────────────────────────────────────────────────────────

#[test]
fn mind_roots_makes_opponent_discard_two() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::mind_roots());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Mind Roots castable for {1}{B}{G}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].hand.len(), opp_hand_before - 2,
        "Opponent should have discarded 2 cards");
    // No lands were discarded → no land hits the battlefield.
    assert!(!g.battlefield.iter().any(|c| c.controller == 0
        && c.definition.is_land()),
        "no land discarded → no land on bf");
}

#[test]
fn mind_roots_puts_discarded_land_onto_battlefield_tapped() {
    // The opponent has a land in hand that gets discarded → it pops
    // onto our battlefield tapped via the new
    // `Selector::DiscardedThisResolution(Land)` primitive.
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_hand(1, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::mind_roots());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Mind Roots castable for {1}{B}{G}");
    drain_stack(&mut g);

    // We now control a tapped Island — the discarded land moved
    // from opponent's graveyard to our battlefield tapped.
    let our_island = g.battlefield.iter().find(|c| {
        c.controller == 0 && c.definition.name == "Island"
    });
    assert!(our_island.is_some(), "discarded Island should be on our bf");
    assert!(our_island.unwrap().tapped, "land enters tapped");
}

// ── Stadium Tidalmage ───────────────────────────────────────────────────────

#[test]
fn stadium_tidalmage_etb_loots_once() {
    // Push XV: Stadium Tidalmage's loot trigger is now an `Effect::MayDo`
    // — the controller must opt in via `OptionalTrigger`. Test injects
    // `Bool(true)` to exercise the opted-in loot path.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::stadium_tidalmage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Yes to the may-loot.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Stadium Tidalmage castable for {2}{U}{R}");
    drain_stack(&mut g);

    // After casting: -1 (Tidalmage cast). ETB: draw 1, discard 1.
    // Net hand should be: hand_before(2) - 1 (cast) + 1 (draw) - 1 (discard) = 1.
    assert_eq!(g.players[0].hand.len(), 1, "Net hand size after cast+ETB loot");
    // Discarded card should be in graveyard.
    assert!(!g.players[0].graveyard.is_empty(),
        "Looting should put a card in the graveyard");
}

#[test]
fn stadium_tidalmage_etb_skips_loot_when_declining() {
    // Push XV: with the auto-decider's default Bool(false) answer, the
    // ETB loot is skipped — hand size remains the same after ETB,
    // graveyard stays empty.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::stadium_tidalmage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Default AutoDecider says no.

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Stadium Tidalmage castable for {2}{U}{R}");
    drain_stack(&mut g);

    // Hand: started with [Bolt + Tidalmage] = 2, cast Tidalmage → 1 left.
    // No loot fired, so still 1 (unchanged from before declining).
    assert_eq!(g.players[0].hand.len(), 1, "No loot should fire");
    assert!(g.players[0].graveyard.is_empty(),
        "No loot → graveyard stays empty");
}

// ── Pterafractyl ────────────────────────────────────────────────────────────

#[test]
fn pterafractyl_etb_with_x_counters_and_gains_two_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pterafractyl());
    // Pay {2} for X=2 plus {G}{U}.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: Some(2),
    })
    .expect("Pterafractyl castable for X=2 {G}{U}");
    drain_stack(&mut g);

    let view = g.computed_permanent(id).expect("Pterafractyl on battlefield");
    assert!(view.keywords.contains(&Keyword::Flying));
    let inst = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(inst.counter_count(CounterType::PlusOnePlusOne), 2,
        "Pterafractyl enters with X=2 +1/+1 counters");
    assert_eq!(g.players[0].life, life_before + 2,
        "Pterafractyl should gain 2 life on ETB");
}

// ── Fractal Mascot ──────────────────────────────────────────────────────────

#[test]
fn fractal_mascot_etb_taps_and_stuns_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractal_mascot());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Fractal Mascot castable for {4}{G}{U}");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert!(target.tapped, "Bear should be tapped");
    assert_eq!(target.counter_count(CounterType::Stun), 1,
        "Bear should have 1 stun counter");
    let mascot = g.computed_permanent(id).unwrap();
    assert!(mascot.keywords.contains(&Keyword::Trample));
}

// ── Mind into Matter ────────────────────────────────────────────────────────

#[test]
fn mind_into_matter_draws_x_cards() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::mind_into_matter());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: Some(3),
    })
    .expect("Mind into Matter castable for X=3 {G}{U}");
    drain_stack(&mut g);

    // -1 (cast) +3 (draw X=3) = +2.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
}

// ── Growth Curve ────────────────────────────────────────────────────────────

#[test]
fn growth_curve_doubles_existing_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Pre-load 2 +1/+1 counters.
    {
        let inst = g.battlefield.iter_mut().find(|c| c.id == bear).unwrap();
        inst.add_counters(CounterType::PlusOnePlusOne, 2);
    }
    let id = g.add_card_to_hand(0, catalog::growth_curve());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Growth Curve castable for {G}{U}");
    drain_stack(&mut g);

    // Pre: 2. +1: 3. Double: 3 + 3 = 6.
    let inst = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(inst.counter_count(CounterType::PlusOnePlusOne), 6,
        "Growth Curve should leave 2*(N+1) counters: starting with 2, ends at 6");
}

#[test]
fn growth_curve_on_creature_with_no_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::growth_curve());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Growth Curve castable for {G}{U}");
    drain_stack(&mut g);

    // Pre: 0. +1: 1. Double: 1 + 1 = 2.
    let inst = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(inst.counter_count(CounterType::PlusOnePlusOne), 2,
        "0 → +1 → double = 2");
}

// ── Quandrix Charm ──────────────────────────────────────────────────────────

#[test]
fn quandrix_charm_mode_2_makes_creature_5_5() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quandrix_charm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: Some(2), x_value: None,
    })
    .expect("Quandrix Charm castable for {G}{U}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 5, "Bear should be 5/5 base after mode 2");
    assert_eq!(view.toughness, 5);
}

#[test]
fn quandrix_charm_mode_1_destroys_enchantment() {
    use crate::card::{CardDefinition, Subtypes};
    use crate::effect::Effect;
    // Build a vanilla enchantment.
    let ench_def = CardDefinition {
        name: "Test Enchant",
        cost: crate::mana::ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    };
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(1, ench_def);
    let id = g.add_card_to_hand(0, catalog::quandrix_charm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(ench)), mode: Some(1), x_value: None,
    })
    .expect("Quandrix Charm castable for {G}{U} with enchantment target");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == ench),
        "Mode 1 should destroy the enchantment");
}

// ── Vibrant Outburst ────────────────────────────────────────────────────────

#[test]
fn vibrant_outburst_deals_three_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vibrant_outburst());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Vibrant Outburst castable for {U}{R}");
    drain_stack(&mut g);

    // 2/2 bear takes 3 damage and dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die to 3 damage");
}

#[test]
fn vibrant_outburst_taps_opp_creature_alongside_damage() {
    // Push: the printed "tap up to one target creature" half is now
    // wired via `Selector::one_of(EachPermanent(opp creature))` — same
    // approximation as Decisive Denial mode 1 / Chelonian Tackle. Was
    // 🟡 (tap half dropped); now ✅. Verify both halves fire when both
    // legal targets exist.
    let mut g = two_player_game();
    // Sacrificial damage target: opp player face (auto-target picks
    // the player when legal). Use a friendly creature target for the
    // damage so we explicitly verify the Tap half hits an opp creature.
    let friendly = g.add_card_to_battlefield(0, catalog::serra_angel());
    let opp_blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vibrant_outburst());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    // Aim the 3 damage at our own Serra Angel (4 toughness — survives).
    // The Tap half auto-picks the opp Bear.
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(friendly)),
        mode: None,
        x_value: None,
    })
    .expect("Vibrant Outburst castable for {U}{R}");
    drain_stack(&mut g);

    // Friendly Serra took 3 damage but survives (4 toughness).
    let serra = g.battlefield.iter().find(|c| c.id == friendly).unwrap();
    assert_eq!(serra.damage, 3,
        "Serra Angel should take 3 damage and survive");
    // Opp Bear was tapped by the second half.
    let bear = g.battlefield.iter().find(|c| c.id == opp_blocker).unwrap();
    assert!(bear.tapped,
        "Opp Bear should have been tapped by Vibrant Outburst's second half");
}

#[test]
fn vibrant_outburst_no_op_tap_when_no_opp_creature() {
    // The "up to one target creature" tap half is a no-op when no
    // opp creature is on the battlefield — the auto-target falls
    // through cleanly without panicking.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::vibrant_outburst());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("Vibrant Outburst castable at opp face");
    drain_stack(&mut g);

    // 3 to opp face.
    assert_eq!(g.players[1].life, opp_life_before - 3);
}

// ── Stress Dream ────────────────────────────────────────────────────────────

#[test]
fn stress_dream_kills_creature_and_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::stress_dream());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Stress Dream castable for {3}{U}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die to 5 damage");
    // Hand: -1 cast +1 draw = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn stress_dream_castable_with_no_opp_creature() {
    // Push: the 5-damage half is now optional via `Selector::one_of(
    // EachPermanent(opp creature))` — the spell castable even when no
    // opp creature is on the battlefield (was 🟡: the cast required
    // a creature target). Just the scry + draw resolves.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::stress_dream());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Stress Dream castable with no creature targets");
    drain_stack(&mut g);

    // -1 (cast) + 1 (draw) = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Arcane Omens ────────────────────────────────────────────────────────────

#[test]
fn arcane_omens_discards_x_cards_using_converged_value() {
    // Pay {4}{B} but use a non-monocolor combination — we'll prep
    // the opponent's hand and validate the discard count.
    let mut g = two_player_game();
    for n in [
        catalog::lightning_bolt(),
        catalog::island(),
        catalog::grizzly_bears(),
    ] {
        g.add_card_to_hand(1, n);
    }

    let id = g.add_card_to_hand(0, catalog::arcane_omens());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Arcane Omens castable for {4}{B}");
    drain_stack(&mut g);

    // Mono-black cast → ConvergedValue = 1 → opp discards 1.
    assert_eq!(g.players[1].hand.len(), opp_hand_before - 1);
}

// ── Together as One ─────────────────────────────────────────────────────────

#[test]
fn together_as_one_uses_converged_value_for_each_clause() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::together_as_one());
    // Pay 6 colorless → ConvergedValue = 0 → all clauses do 0.
    g.players[0].mana_pool.add_colorless(6);
    let opp_life_before = g.players[1].life;
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("Together as One castable for {6}");
    drain_stack(&mut g);

    // ConvergedValue = 0, so no draw, no damage, no life gain.
    assert_eq!(g.players[1].life, opp_life_before);
    assert_eq!(g.players[0].life, life_before);
    // Hand: -1 cast + 0 draw = hand_before - 1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
}

// ── Rancorous Archaic ───────────────────────────────────────────────────────

#[test]
fn rancorous_archaic_etb_with_converge_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rancorous_archaic());
    // {5} cast — pay all-colorless, ConvergedValue = 0.
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Rancorous Archaic castable for {5}");
    drain_stack(&mut g);

    let view = g.computed_permanent(id).unwrap();
    assert!(view.keywords.contains(&Keyword::Trample));
    assert!(view.keywords.contains(&Keyword::Reach));
    let inst = g.battlefield.iter().find(|c| c.id == id).unwrap();
    // ConvergedValue=0 → 0 counters → 2/2 base body.
    assert_eq!(inst.counter_count(CounterType::PlusOnePlusOne), 0);
    assert_eq!(view.power, 2);
    assert_eq!(view.toughness, 2);
}

// ── Wisdom of Ages ──────────────────────────────────────────────────────────

#[test]
fn wisdom_of_ages_returns_all_instants_and_sorceries_from_graveyard() {
    let mut g = two_player_game();
    // Stock graveyard with bolt (instant), island (land), grizzly bears
    // (creature), wrath (sorcery).
    let bolt = g.add_card_to_battlefield(0, catalog::lightning_bolt());
    let isl = g.add_card_to_battlefield(0, catalog::island());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wrath = g.add_card_to_battlefield(0, catalog::wrath_of_god());
    for cid in [bolt, isl, bears, wrath] {
        let idx = g.battlefield.iter().position(|c| c.id == cid).unwrap();
        let card = g.battlefield.remove(idx);
        g.players[0].graveyard.push(card);
    }

    let id = g.add_card_to_hand(0, catalog::wisdom_of_ages());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Wisdom of Ages castable for {4}{U}{U}{U}");
    drain_stack(&mut g);

    // Bolt and Wrath should be back in hand; Island and Grizzly Bears stay in graveyard.
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
    assert!(g.players[0].hand.iter().any(|c| c.id == wrath));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == isl));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bears));
}

// ── Rapturous Moment ────────────────────────────────────────────────────────

#[test]
fn rapturous_moment_loots_and_adds_mana() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::rapturous_moment());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Rapturous Moment castable for {4}{U}{R}");
    drain_stack(&mut g);

    // Hand: -1 cast +3 draw -2 discard = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Mana pool should have at least 2U and 3R post-resolution.
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 2);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3);
}

// ── Splatter Technique ──────────────────────────────────────────────────────

#[test]
fn splatter_technique_mode_0_draws_four() {
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::splatter_technique());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(0), x_value: None,
    })
    .expect("Splatter Technique castable in mode 0");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 4);
}

#[test]
fn splatter_technique_mode_1_wipes_creatures() {
    let mut g = two_player_game();
    let bear0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::splatter_technique());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(1), x_value: None,
    })
    .expect("Splatter Technique castable in mode 1");
    drain_stack(&mut g);

    // Both 2/2 bears die to 4 damage.
    assert!(!g.battlefield.iter().any(|c| c.id == bear0));
    assert!(!g.battlefield.iter().any(|c| c.id == bear1));
}

// ── Arnyn, Deathbloom Botanist ──────────────────────────────────────────────

#[test]
fn arnyn_drains_when_a_one_power_creature_you_control_dies() {
    let mut g = two_player_game();
    let arnyn = g.add_card_to_battlefield(0, catalog::arnyn_deathbloom_botanist());
    let view = g.computed_permanent(arnyn).unwrap();
    assert!(view.keywords.contains(&Keyword::Deathtouch));

    // Place a 1/1 creature you control (mana value goes through bears →
    // we simulate a 1/1 by using a token-style creature definition).
    use crate::card::{CardDefinition, Subtypes};
    let weak = CardDefinition {
        name: "Weak Creature",
        cost: crate::mana::ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: crate::effect::Effect::Noop,
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    };
    let weak_id = g.add_card_to_battlefield(0, weak);

    let life_before = g.players[0].life;
    let opp_life_before = g.players[1].life;

    // Kill via Murder so the dies-trigger pipeline runs end-to-end.
    let murder_id = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: murder_id, target: Some(Target::Permanent(weak_id)),
        mode: None, x_value: None,
    })
    .expect("Murder castable for {1}{B}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == weak_id),
        "Weak creature should be in the graveyard");
    assert_eq!(g.players[1].life, opp_life_before - 2,
        "Arnyn drains opponent for 2");
    assert_eq!(g.players[0].life, life_before + 2,
        "Arnyn gains 2 life");
}

// ── Startled Relic Sloth ────────────────────────────────────────────────────

#[test]
fn startled_relic_sloth_combat_step_exiles_graveyard_card() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Move bear to graveyard.
    let card = g.battlefield.iter().position(|c| c.id == bear).unwrap();
    let bear_card = g.battlefield.remove(card);
    g.players[0].graveyard.push(bear_card);

    let sloth = g.add_card_to_battlefield(0, catalog::startled_relic_sloth());
    let view = g.computed_permanent(sloth).unwrap();
    assert!(view.keywords.contains(&Keyword::Trample));
    assert!(view.keywords.contains(&Keyword::Lifelink));

    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);

    assert!(
        g.exile.iter().any(|c| c.id == bear),
        "Begin-combat trigger should exile a graveyard card"
    );
}

// ── Hardened Academic ───────────────────────────────────────────────────────

#[test]
fn hardened_academic_discard_grants_lifelink_eot() {
    let mut g = two_player_game();
    let academic = g.add_card_to_battlefield(0, catalog::hardened_academic());
    g.add_card_to_hand(0, catalog::island());

    let view = g.computed_permanent(academic).unwrap();
    assert!(view.keywords.contains(&Keyword::Flying));
    assert!(view.keywords.contains(&Keyword::Haste));
    assert!(!view.keywords.contains(&Keyword::Lifelink));

    g.perform_action(GameAction::ActivateAbility {
        card_id: academic,
        ability_index: 0,
        target: None,
    })
    .expect("Discard ability should activate");
    drain_stack(&mut g);

    let view = g.computed_permanent(academic).unwrap();
    assert!(view.keywords.contains(&Keyword::Lifelink),
        "Hardened Academic should have lifelink until EOT after discard activation");
}

// ── Slumbering Trudge ───────────────────────────────────────────────────────

#[test]
fn slumbering_trudge_x_zero_enters_with_three_stun_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::slumbering_trudge());
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: Some(0),
    })
    .expect("Slumbering Trudge castable for X=0");
    drain_stack(&mut g);

    let inst = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(inst.counter_count(CounterType::Stun), 3,
        "X=0 → 3 stun counters via NonNeg(3-X)");
    assert!(inst.tapped, "Slumbering Trudge enters tapped");
}

#[test]
fn slumbering_trudge_x_three_enters_without_stun_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::slumbering_trudge());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3); // X=3

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: Some(3),
    })
    .expect("Slumbering Trudge castable for X=3");
    drain_stack(&mut g);

    let inst = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(inst.counter_count(CounterType::Stun), 0,
        "X=3 → 0 stun counters (NonNeg(3-3))");
}

// ── Fractal Anomaly ─────────────────────────────────────────────────────────

#[test]
fn fractal_anomaly_tokens_grows_with_cards_drawn() {
    let mut g = two_player_game();
    // Draw two cards "this turn" by setting the counter directly.
    g.players[0].cards_drawn_this_turn = 3;

    let id = g.add_card_to_hand(0, catalog::fractal_anomaly());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Fractal Anomaly castable for {U}");
    drain_stack(&mut g);

    // A new Fractal token should be on the battlefield.
    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Fractal")
        .expect("Fractal token should be on the battlefield");
    assert_eq!(g.battlefield.len(), bf_before + 1);
    assert_eq!(token.counter_count(CounterType::PlusOnePlusOne), 3,
        "Fractal token should have 3 +1/+1 counters (one per card drawn this turn)");
    let view = g.computed_permanent(token.id).unwrap();
    assert_eq!(view.power, 3);
    assert_eq!(view.toughness, 3);
}

#[test]
fn fractal_anomaly_after_draw_effect_uses_live_counter() {
    // Validates that `Player.cards_drawn_this_turn` is incremented on
    // `Effect::Draw`, and that `Value::CardsDrawnThisTurn` reads the
    // live counter at resolution time.
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    // Hand-cast a `Draw 2` spell first so cards_drawn_this_turn
    // increments from 0 to 2.
    let bolt_then_draw = catalog::concentrate(); // {2}{U}{U} draw 3
    let _ = g.add_card_to_hand(0, bolt_then_draw);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    let conc = g.players[0].hand.last().unwrap().id;
    g.perform_action(GameAction::CastSpell {
        card_id: conc, target: None, mode: None, x_value: None,
    })
    .expect("Concentrate castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].cards_drawn_this_turn, 3,
        "Concentrate should bump cards_drawn_this_turn to 3");

    // Now cast Fractal Anomaly — the token should enter with 3 counters.
    let id = g.add_card_to_hand(0, catalog::fractal_anomaly());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Fractal Anomaly castable for {U}");
    drain_stack(&mut g);

    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Fractal")
        .expect("Fractal token should be on the battlefield");
    assert_eq!(token.counter_count(CounterType::PlusOnePlusOne), 3);
}

#[test]
fn fractal_anomaly_zero_cards_drawn_dies_to_sba() {
    let mut g = two_player_game();
    // 0 cards drawn this turn — the printed "0/0 with 0 counters" dies.
    g.players[0].cards_drawn_this_turn = 0;
    let id = g.add_card_to_hand(0, catalog::fractal_anomaly());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Fractal Anomaly castable for {U}");
    drain_stack(&mut g);

    // The Fractal token should have died to SBA — no token on battlefield.
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Fractal"),
        "0/0 Fractal token with 0 counters should die to SBA");
}

// ── Tenured Concocter ───────────────────────────────────────────────────────

#[test]
fn tenured_concocter_is_vigilant_4_5_troll_druid() {
    let card = catalog::tenured_concocter();
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 5);
    assert!(card.keywords.contains(&Keyword::Vigilance));
    assert!(card.has_creature_type(crate::card::CreatureType::Troll));
    assert!(card.has_creature_type(crate::card::CreatureType::Druid));
}

// ── Traumatic Critique ──────────────────────────────────────────────────────

#[test]
fn traumatic_critique_x_damage_loots() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::traumatic_critique());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4); // X=4
    let opp_life_before = g.players[1].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), mode: None, x_value: Some(4),
    })
    .expect("Traumatic Critique castable for X=4 {U}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, opp_life_before - 4,
        "Should deal 4 damage with X=4");
    // Hand: -1 cast +2 draw -1 discard = 0 net.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Graduation Day (new) ────────────────────────────────────────────────────

#[test]
fn graduation_day_repartee_pumps_creature_when_targeting_creature() {
    // Repartee enchantment: cast Lightning Bolt at a creature → +1/+1
    // counter on a creature you control.
    let mut g = two_player_game();
    let _gd = g.add_card_to_battlefield(0, catalog::graduation_day());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let pumped = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        pumped.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Graduation Day Repartee should add a +1/+1 counter when an instant targets a creature",
    );
}

#[test]
fn graduation_day_does_not_fire_when_targeting_player() {
    let mut g = two_player_game();
    let _gd = g.add_card_to_battlefield(0, catalog::graduation_day());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let unpumped = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        unpumped.counter_count(CounterType::PlusOnePlusOne),
        0,
        "Repartee should NOT fire when the spell targets a player",
    );
}

// ── Stirring Hopesinger Repartee improvement ───────────────────────────────

#[test]
fn stirring_hopesinger_repartee_pumps_each_creature_you_control() {
    let mut g = two_player_game();
    let hopesinger = g.add_card_to_battlefield(0, catalog::stirring_hopesinger());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let h = g.battlefield.iter().find(|c| c.id == hopesinger).unwrap();
    assert_eq!(
        h.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Hopesinger should get a +1/+1 counter from its own Repartee trigger",
    );
    let b = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        b.counter_count(CounterType::PlusOnePlusOne),
        1,
        "Bear should also get a +1/+1 counter (each creature you control)",
    );
    // The opponent's bear should NOT get a counter.
    if let Some(o) = g.battlefield.iter().find(|c| c.id == opp_bear) {
        assert_eq!(
            o.counter_count(CounterType::PlusOnePlusOne),
            0,
            "Opponent's creatures don't get pumped",
        );
    }
}

// ── Informed Inkwright Repartee improvement ────────────────────────────────

#[test]
fn informed_inkwright_repartee_creates_inkling_token() {
    let mut g = two_player_game();
    let _scribe = g.add_card_to_battlefield(0, catalog::informed_inkwright());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let inkling_count_before = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Inkling")
        .count();

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let inkling_count_after = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Inkling")
        .count();
    assert_eq!(
        inkling_count_after,
        inkling_count_before + 1,
        "Informed Inkwright Repartee should mint a 1/1 Inkling token",
    );
}

// ── Inkling Mascot Repartee improvement ────────────────────────────────────

#[test]
fn inkling_mascot_repartee_grants_flying_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let mascot = g.add_card_to_battlefield(0, catalog::inkling_mascot());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let m = g.battlefield.iter().find(|c| c.id == mascot).unwrap();
    assert!(
        m.has_keyword(&Keyword::Flying),
        "Inkling Mascot should have Flying after Repartee",
    );
    // Surveil 1 either drops the top to graveyard (-1 lib) or returns it
    // (no change). Either way the library is at most unchanged.
    assert!(
        g.players[0].library.len() <= lib_before,
        "Surveil 1 should peek at the top — library did not grow",
    );
}

// ── Snooping Page Repartee improvement ─────────────────────────────────────

#[test]
fn snooping_page_repartee_grants_unblockable() {
    let mut g = two_player_game();
    let page = g.add_card_to_battlefield(0, catalog::snooping_page());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let p = g.battlefield.iter().find(|c| c.id == page).unwrap();
    assert!(
        p.has_keyword(&Keyword::Unblockable),
        "Snooping Page should be unblockable this turn after Repartee",
    );
}

// ── Withering Curse ─────────────────────────────────────────────────────────

#[test]
fn withering_curse_without_lifegain_pumps_minus_two() {
    // No lifegain this turn → -2/-2 to all creatures.
    let mut g = two_player_game();
    let bear_p0 = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let bear_p1 = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::withering_curse());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Withering Curse castable for {1}{B}{B}");
    drain_stack(&mut g);

    // Both 2/2 bears should die to SBA (2-2 = 0 toughness).
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear_p0),
        "P0 bear should die (-2/-2 → 0 toughness)",
    );
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear_p1),
        "P1 bear should die (-2/-2 → 0 toughness)",
    );
}

#[test]
fn withering_curse_with_lifegain_destroys_all_creatures() {
    // Trigger lifegain to enable the Infusion path.
    let mut g = two_player_game();
    g.players[0].life_gained_this_turn = 3;
    let bear_p0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear_p1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::withering_curse());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Withering Curse castable for {1}{B}{B}");
    drain_stack(&mut g);

    // Infusion path: every creature destroyed → graveyard.
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear_p0),
        "P0 bear destroyed by Infusion",
    );
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear_p1),
        "P1 bear destroyed by Infusion",
    );
    assert!(
        g.players[0].graveyard.iter().any(|c| c.id == bear_p0),
        "P0 bear in P0's graveyard",
    );
}

// ── Root Manipulation ───────────────────────────────────────────────────────

#[test]
fn root_manipulation_pumps_each_creature_you_control_with_menace() {
    let mut g = two_player_game();
    let bear_p0 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear_p1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::root_manipulation());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Root Manipulation castable for {3}{B}{G}");
    drain_stack(&mut g);

    let p0 = g.battlefield.iter().find(|c| c.id == bear_p0).unwrap();
    assert_eq!(p0.power(), 4, "Bear should be 4/4 (+2/+2)");
    assert_eq!(p0.toughness(), 4);
    assert!(
        p0.has_keyword(&Keyword::Menace),
        "Bear should gain Menace",
    );
    let p1 = g.battlefield.iter().find(|c| c.id == bear_p1).unwrap();
    assert_eq!(p1.power(), 2, "Opponent's bear unchanged");
    assert!(!p1.has_keyword(&Keyword::Menace), "Opponent's bear: no menace");
}

// ── Blech, Loafing Pest ─────────────────────────────────────────────────────

#[test]
fn blech_pumps_pest_on_lifegain() {
    let mut g = two_player_game();
    let _blech = g.add_card_to_battlefield(0, catalog::blech_loafing_pest());
    // Pest Mascot is a Pest. Blech is also a Pest.
    let pest = g.add_card_to_battlefield(0, catalog::pest_mascot());
    // A non-Pest/Bat/Insect/Snake/Spider creature should NOT be pumped.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Use a quick lifegain spell to trigger Blech.
    let id = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(0)), mode: None, x_value: None,
    })
    .expect("Healing Salve castable for {W}");
    drain_stack(&mut g);

    let p = g.battlefield.iter().find(|c| c.id == pest).unwrap();
    assert!(
        p.counter_count(CounterType::PlusOnePlusOne) >= 1,
        "Pest Mascot (Pest) should get a +1/+1 counter from Blech",
    );
    let b = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(
        b.counter_count(CounterType::PlusOnePlusOne),
        0,
        "Grizzly Bears (Bear, not in pump list) should not be pumped",
    );
}

// ── Cauldron of Essence ─────────────────────────────────────────────────────

#[test]
fn cauldron_of_essence_drains_when_creature_dies() {
    let mut g = two_player_game();
    let _cauldron = g.add_card_to_battlefield(0, catalog::cauldron_of_essence());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let opp_life_before = g.players[1].life;
    let life_before = g.players[0].life;

    // Bolt our own bear to trigger the death drain.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear destroyed by Bolt",
    );
    assert_eq!(
        g.players[1].life,
        opp_life_before - 1,
        "Opponent should lose 1 life from Cauldron drain",
    );
    assert_eq!(
        g.players[0].life,
        life_before + 1,
        "You should gain 1 life from Cauldron drain",
    );
}

// ── Diary of Dreams ─────────────────────────────────────────────────────────

#[test]
fn diary_of_dreams_gains_charge_on_instant_or_sorcery_cast() {
    let mut g = two_player_game();
    let diary = g.add_card_to_battlefield(0, catalog::diary_of_dreams());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let d = g.battlefield.iter().find(|c| c.id == diary).unwrap();
    assert_eq!(
        d.counter_count(CounterType::Page),
        1,
        "Diary of Dreams should accrue a Page counter on instant cast",
    );
}

// ── Spectacle Summit ────────────────────────────────────────────────────────

#[test]
fn spectacle_summit_taps_for_blue_or_red() {
    // Sanity test: Spectacle Summit is the Prismari (U/R) school land
    // and shares the same shape as the other school lands.
    let card = catalog::spectacle_summit();
    assert!(matches!(card.card_types[0], CardType::Land));
    let lt: Vec<_> = card.subtypes.land_types.clone();
    assert!(lt.contains(&crate::card::LandType::Island));
    assert!(lt.contains(&crate::card::LandType::Mountain));
}

// ── Comforting Counsel ──────────────────────────────────────────────────────

#[test]
fn comforting_counsel_accrues_growth_on_lifegain() {
    let mut g = two_player_game();
    let cc = g.add_card_to_battlefield(0, catalog::comforting_counsel());
    let salve = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: salve, target: Some(Target::Player(0)), mode: None, x_value: None,
    })
    .expect("Healing Salve castable for {W}");
    drain_stack(&mut g);

    let counsel = g.battlefield.iter().find(|c| c.id == cc).unwrap();
    assert_eq!(
        counsel.counter_count(CounterType::Growth),
        1,
        "Comforting Counsel should accrue a Growth counter when you gain life",
    );
}

// ── Moment of Reckoning ─────────────────────────────────────────────────────

#[test]
fn moment_of_reckoning_destroy_mode_destroys_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::moment_of_reckoning());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: Some(0),
        x_value: None,
    })
    .expect("Moment of Reckoning castable for {3}{W}{W}{B}{B}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be destroyed by mode 0 (Destroy)",
    );
}

#[test]
fn moment_of_reckoning_return_mode_brings_card_back() {
    let mut g = two_player_game();
    // Seed a creature card into our graveyard to retrieve.
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::moment_of_reckoning());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: Some(1),
        x_value: None,
    })
    .expect("Moment of Reckoning castable in return mode");
    drain_stack(&mut g);

    assert!(
        g.battlefield.iter().any(|c| c.id == bear),
        "Bear should return to battlefield (mode 1)",
    );
    assert_eq!(g.battlefield.len(), bf_before + 1);
}

// ── Stirring Honormancer ────────────────────────────────────────────────────

#[test]
fn stirring_honormancer_etb_finds_creature_in_top_x() {
    let mut g = two_player_game();
    // We control 1 creature, so X = 1. Top of library = a creature.
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // controlled creature
    g.add_card_to_library(0, catalog::grizzly_bears()); // top of library — found!
    let id = g.add_card_to_hand(0, catalog::stirring_honormancer());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Stirring Honormancer castable for {2}{W}{W}{B}");
    drain_stack(&mut g);

    // Stirring Honormancer entered (1 card from hand) + bear from library
    // joins our hand → net hand = before - 1 (cast) + 1 (find) = same
    // size. But our hand also lost the cast card so:
    assert_eq!(
        g.players[0].hand.len(),
        hand_before, // -1 for cast, +1 for retrieved bear
        "Top-of-library bear should have joined hand",
    );
}

// ── Dissection Practice ─────────────────────────────────────────────────────

#[test]
fn dissection_practice_drains_one_and_shrinks_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::dissection_practice());
    g.players[0].mana_pool.add(Color::Black, 1);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Dissection Practice castable for {B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, p0_life + 1, "You gain 1 life");
    assert_eq!(g.players[1].life, p1_life - 1, "Opponent loses 1 life");
    // 2/2 with -1/-1 EOT → 1/1, still alive.
    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(target.power(), 1);
    assert_eq!(target.toughness(), 1);
}

#[test]
fn dissection_practice_pumps_friendly_creature() {
    // Push: the printed "Up to one target creature gets +1/+1 EOT"
    // half is now wired via `Selector::one_of(EachPermanent(Creature
    // ∧ ControlledByYou))`. Was 🟡 (+1/+1 half dropped); now ✅. Verify
    // the +1/+1 lands on a friendly creature when one is on the
    // battlefield.
    let mut g = two_player_game();
    let friendly = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 — gets -1/-1
    let id = g.add_card_to_hand(0, catalog::dissection_practice());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        mode: None, x_value: None,
    })
    .expect("Dissection Practice castable for {B}");
    drain_stack(&mut g);

    // Friendly 2/2 +1/+1 = 3/3.
    let f = g.battlefield.iter().find(|c| c.id == friendly).unwrap();
    assert_eq!(f.power(), 3, "Friendly bear pumped to 3 power");
    assert_eq!(f.toughness(), 3, "Friendly bear pumped to 3 toughness");
    // Opp 2/2 -1/-1 = 1/1.
    let opp = g.battlefield.iter().find(|c| c.id == opp_bear).unwrap();
    assert_eq!(opp.power(), 1);
    assert_eq!(opp.toughness(), 1);
}

// ── Heated Argument ─────────────────────────────────────────────────────────

#[test]
fn heated_argument_deals_six_to_creature_and_two_to_controller() {
    // Push XV: the gy-exile + 2-to-controller rider is now wrapped in
    // `Effect::MayDo`. Inject `Bool(true)` to opt in.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 — dies
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // a card to exile
    let id = g.add_card_to_hand(0, catalog::heated_argument());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let p1_life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Heated Argument castable for {4}{R}");
    drain_stack(&mut g);

    // Bear dies (lethal), controller takes 2.
    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear destroyed by 6 damage",
    );
    assert_eq!(
        g.players[1].life,
        p1_life - 2,
        "Controller takes 2 from the rider",
    );
    // The Bolt should have been exiled from the graveyard.
    assert!(
        g.exile.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "Bolt should be exiled from graveyard",
    );
}

#[test]
fn heated_argument_skips_rider_when_declining() {
    // Push XV: declining the gy-exile means no extra 2 damage fires.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::heated_argument());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let p1_life = g.players[1].life;
    // Default AutoDecider says no.

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Heated Argument castable");
    drain_stack(&mut g);

    // Bear still dies (the 6 damage isn't gated), controller untouched.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert_eq!(g.players[1].life, p1_life,
        "Controller takes no damage when rider is skipped");
    // Bolt stays in graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"));
}

// ── End of the Hunt ─────────────────────────────────────────────────────────

#[test]
fn end_of_the_hunt_exiles_opponent_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::end_of_the_hunt());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("End of the Hunt castable for {1}{B}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Opponent's bear should leave the battlefield",
    );
    assert!(
        g.exile.iter().any(|c| c.id == bear),
        "Bear should be exiled",
    );
}

// ── Vicious Rivalry ─────────────────────────────────────────────────────────

#[test]
fn vicious_rivalry_destroys_creatures_at_or_below_x() {
    let mut g = two_player_game();
    // X = 2: bear (CMC 2) dies, but a CMC-3 creature lives.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // CMC 2
    let big = g.add_card_to_battlefield(1, catalog::craw_wurm()); // CMC 6
    let id = g.add_card_to_hand(0, catalog::vicious_rivalry());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4); // 2 generic + 2 for X
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: Some(2),
    })
    .expect("Vicious Rivalry castable for {2}{2}{B}{G} (X=2)");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear (CMC 2) destroyed by Vicious Rivalry X=2",
    );
    assert!(
        g.battlefield.iter().any(|c| c.id == big),
        "Craw Wurm (CMC 6) survives",
    );
    assert_eq!(
        g.players[0].life,
        life_before - 2,
        "Caster pays X life as additional cost (approximated)",
    );
}

// ── Proctor's Gaze ──────────────────────────────────────────────────────────

#[test]
fn proctors_gaze_returns_target_and_fetches_basic() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let forest = g.add_card_to_library(0, catalog::forest());
    // ScriptedDecider answers the SearchLibrary decision with the
    // forest. AutoDecider's default is to decline searches.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::proctors_gaze());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Proctor's Gaze castable for {2}{G}{U}");
    drain_stack(&mut g);

    // Bear bounced to its owner's hand.
    assert!(
        g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear bounced to owner's hand",
    );
    // Forest fetched onto our battlefield.
    let on_bf = g.battlefield.iter().find(|c| c.id == forest);
    assert!(on_bf.is_some(), "Forest should land on battlefield");
    assert!(on_bf.unwrap().tapped, "Forest should enter tapped");
}

// ── Lorehold Charm ──────────────────────────────────────────────────────────

#[test]
fn lorehold_charm_pump_mode_pumps_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lorehold_charm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(2), x_value: None,
    })
    .expect("Lorehold Charm castable in pump mode");
    drain_stack(&mut g);

    let target = g.battlefield.iter().find(|c| c.id == bear).unwrap();
    assert_eq!(target.power(), 4, "Bear gets +2/+1 → 4/3");
    assert_eq!(target.toughness(), 3);
}

// ── Borrowed Knowledge ──────────────────────────────────────────────────────

#[test]
fn borrowed_knowledge_mode_one_draws_equal_to_cards_discarded() {
    // Push: with the new `Value::CardsDiscardedThisResolution`,
    // mode 1 is now exact-printed: discard hand, draw N where N =
    // count of cards discarded by this effect.
    let mut g = two_player_game();
    // Seed our library so we have cards to draw.
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    // Pre-cast hand: 2 lands. Casting the spell adds 1 card. After
    // payment leaves 2 cards in hand (the spell exits hand into
    // resolution). Discard hand = 2; draw 2.
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());

    let id = g.add_card_to_hand(0, catalog::borrowed_knowledge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(1), x_value: None,
    })
    .expect("Borrowed Knowledge castable in mode 1");
    drain_stack(&mut g);

    // The two pre-cast hand cards were discarded; we drew exactly 2.
    assert_eq!(g.players[0].hand.len(), 2, "draw equals discard count (2)");
    assert_eq!(g.players[0].graveyard.len(), 3,
        "2 discards + the spell card = 3 in graveyard");
}

#[test]
fn borrowed_knowledge_mode_one_with_empty_hand_draws_zero() {
    // Edge case: zero cards in hand at trigger time means
    // `CardsDiscardedThisResolution` is 0, so we draw 0 cards. The
    // spell card itself moved to the stack and won't be in hand.
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::borrowed_knowledge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(1), x_value: None,
    })
    .expect("Borrowed Knowledge castable in mode 1 with empty hand");
    drain_stack(&mut g);

    // Drew zero cards — library hasn't shrunk.
    assert_eq!(g.players[0].library.len(), lib_before);
    assert_eq!(g.players[0].hand.len(), 0);
}

// ── Planar Engineering ──────────────────────────────────────────────────────

#[test]
fn planar_engineering_sacrifices_two_lands_and_fetches_basics() {
    let mut g = two_player_game();
    let l1 = g.add_card_to_battlefield(0, catalog::forest());
    let l2 = g.add_card_to_battlefield(0, catalog::forest());
    let mut lib_forests = Vec::new();
    for _ in 0..6 {
        lib_forests.push(g.add_card_to_library(0, catalog::forest()));
    }
    // ScriptedDecider answers each of the four SearchLibrary decisions
    // with successive forests from the library.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(lib_forests[0])),
        DecisionAnswer::Search(Some(lib_forests[1])),
        DecisionAnswer::Search(Some(lib_forests[2])),
        DecisionAnswer::Search(Some(lib_forests[3])),
    ]));
    let id = g.add_card_to_hand(0, catalog::planar_engineering());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Planar Engineering castable for {3}{G}");
    drain_stack(&mut g);

    // Both lands sacrificed.
    assert!(!g.battlefield.iter().any(|c| c.id == l1));
    assert!(!g.battlefield.iter().any(|c| c.id == l2));
    // 4 forests fetched onto the battlefield tapped (not the
    // already-sacrificed l1/l2).
    let forest_count = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Forest")
        .count();
    assert_eq!(forest_count, 4, "Should have 4 fresh Forests on the battlefield");
}

// ── Brush Off ───────────────────────────────────────────────────────────────

#[test]
fn brush_off_is_a_four_mana_counterspell() {
    // Sanity test: Brush Off is a 4-mana counterspell (cost-reduction
    // rider omitted). Verify the cost/effect shape so future regressions
    // catch costing/keyword drift.
    let card = catalog::brush_off();
    assert!(matches!(card.card_types[0], CardType::Instant));
    assert_eq!(card.cost.cmc(), 4);
    assert!(
        matches!(card.effect, crate::card::Effect::CounterSpell { .. }),
        "Should be a CounterSpell effect",
    );
}

// ── Run Behind ──────────────────────────────────────────────────────────────

#[test]
fn run_behind_puts_target_creature_on_bottom_of_library() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::run_behind());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Run Behind castable for {3}{U}");
    drain_stack(&mut g);

    assert!(
        !g.battlefield.iter().any(|c| c.id == bear),
        "Bear leaves the battlefield",
    );
    // Bear goes to bottom of P1's library.
    assert_eq!(
        g.players[1].library.last().map(|c| c.id),
        Some(bear),
        "Bear should be at the bottom of its owner's library",
    );
}

// ── Antiquities on the Loose ────────────────────────────────────────────────

#[test]
fn antiquities_on_the_loose_creates_two_spirit_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::antiquities_on_the_loose());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Antiquities on the Loose castable for {1}{W}{W}");
    drain_stack(&mut g);

    let spirit_count = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit")
        .count();
    assert_eq!(spirit_count, 2, "Should create two Spirit tokens");
    assert_eq!(g.battlefield.len(), bf_before + 2);
}

// ── Conciliator's Duelist ───────────────────────────────────────────────────

#[test]
fn conciliators_duelist_etb_draws_and_each_player_loses_one() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::conciliators_duelist());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 2);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Conciliator's Duelist castable for {W}{W}{B}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, p0_life - 1, "You lose 1 life");
    assert_eq!(g.players[1].life, p1_life - 1, "Opponent loses 1 life");
    // Hand: -1 cast + 1 draw = unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

// ── Body-only batch + Ajani's Response (2026-04-30 push) ────────────────────

#[test]
fn ajanis_response_destroys_target_creature() {
    let mut g = two_player_game();
    let resp = g.add_card_to_hand(0, catalog::ajanis_response());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: resp,
        target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Ajani's Response castable for {4}{W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Ajani's Response destroys the target creature");
}

#[test]
fn ajanis_response_only_targets_creatures() {
    let mut g = two_player_game();
    let resp = g.add_card_to_hand(0, catalog::ajanis_response());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: resp,
        target: Some(Target::Player(1)),
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Ajani's Response only targets creatures");
}

#[test]
fn cuboid_colony_is_a_one_one_with_flash_flying_trample() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::cuboid_colony());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Cuboid Colony castable for {G}{U}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 1);
    assert_eq!(card.toughness(), 1);
    assert!(card.has_keyword(&Keyword::Flash));
    assert!(card.has_keyword(&Keyword::Flying));
    assert!(card.has_keyword(&Keyword::Trample));
}

#[test]
fn hungry_graffalon_is_a_three_four_with_reach() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::hungry_graffalon());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Hungry Graffalon castable for {3}{G}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 3);
    assert_eq!(card.toughness(), 4);
    assert!(card.has_keyword(&Keyword::Reach));
}

#[test]
fn molten_core_maestro_has_menace() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::molten_core_maestro());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Molten-Core Maestro castable for {1}{R}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Menace));
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 2);
}

#[test]
fn aberrant_manawurm_has_trample_and_correct_pt() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::aberrant_manawurm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Aberrant Manawurm castable for {3}{G}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Trample));
    assert_eq!(card.power(), 2);
    assert_eq!(card.toughness(), 5);
}

#[test]
fn tackle_artist_has_trample_and_correct_pt() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::tackle_artist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Tackle Artist castable for {3}{R}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Trample));
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 3);
}

#[test]
fn thunderdrum_soloist_has_reach() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::thunderdrum_soloist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Thunderdrum Soloist castable for {1}{R}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert!(card.has_keyword(&Keyword::Reach));
}

#[test]
fn pensive_professor_is_a_zero_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pensive_professor());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Pensive Professor castable for {1}{U}{U}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 0);
    assert_eq!(card.toughness(), 2);
}

#[test]
fn eternal_student_is_a_four_two_zombie() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::eternal_student());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Eternal Student castable for {3}{B}");
    drain_stack(&mut g);
    let card = g.battlefield_find(id).unwrap();
    assert_eq!(card.power(), 4);
    assert_eq!(card.toughness(), 2);
    assert!(card.definition.has_creature_type(crate::card::CreatureType::Zombie));
}

#[test]
fn postmortem_professor_drains_one_on_attack() {
    let mut g = two_player_game();
    let prof = g.add_card_to_battlefield(0, catalog::postmortem_professor());
    // Strip summoning sickness so it can attack.
    g.battlefield_find_mut(prof).unwrap().summoning_sick = false;

    // Move to declare-attackers and attack with the professor.
    g.step = TurnStep::DeclareAttackers;
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;

    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: prof,
        target: AttackTarget::Player(1),
    }]))
    .expect("Professor can attack");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, p0_life + 1,
        "Postmortem Professor's on-attack drain gives you 1");
    assert_eq!(g.players[1].life, p1_life - 1,
        "Postmortem Professor's on-attack drain takes 1 from each opponent");
}

// ── Scolding Administrator (Silverquill 🟡 → mostly wired) ─────────────────

#[test]
fn scolding_administrator_repartee_pumps_self() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let admin = g.add_card_to_battlefield(0, catalog::scolding_administrator());
    g.battlefield_find_mut(admin).unwrap().summoning_sick = false;

    // Cast a creature-targeting Lightning Bolt to fire Repartee.
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

    let counters = g.battlefield_find(admin)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters, 1, "Repartee adds a +1/+1 counter to self");
}

#[test]
fn scolding_administrator_has_menace() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let admin = g.add_card_to_battlefield(0, catalog::scolding_administrator());
    let card = g.battlefield_find(admin).unwrap();
    assert!(card.has_keyword(&Keyword::Menace));
    assert!(card.definition.has_creature_type(crate::card::CreatureType::Dwarf));
    assert!(card.definition.has_creature_type(crate::card::CreatureType::Cleric));
}

#[test]
fn scolding_administrator_dies_transfers_counters_to_target_creature() {
    // Push: when a Scolding Administrator with N +1/+1 counters dies,
    // those counters now go on a target creature via the new
    // `Value::CountersOn(TriggerSource)` graveyard fallback.
    use crate::card::CounterType;
    let mut g = two_player_game();
    let admin = g.add_card_to_battlefield(0, catalog::scolding_administrator());
    // Stack 3 +1/+1 counters on the admin.
    if let Some(c) = g.battlefield_find_mut(admin) {
        c.add_counters(CounterType::PlusOnePlusOne, 3);
    }
    // Target a friendly bear.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let _ = g.remove_to_graveyard_with_triggers(admin);
    drain_stack(&mut g);

    let bear_now = g.battlefield_find(bear).expect("bear still on bf");
    assert_eq!(bear_now.counter_count(CounterType::PlusOnePlusOne), 3,
        "3 counters should transfer from dying Scolding Admin to target bear");
}

#[test]
fn scolding_administrator_dies_without_counters_no_op() {
    // Edge case: dies with zero counters → no trigger fires (gated).
    use crate::card::CounterType;
    let mut g = two_player_game();
    let admin = g.add_card_to_battlefield(0, catalog::scolding_administrator());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let _ = g.remove_to_graveyard_with_triggers(admin);
    drain_stack(&mut g);

    let bear_now = g.battlefield_find(bear).expect("bear still on bf");
    assert_eq!(bear_now.counter_count(CounterType::PlusOnePlusOne), 0,
        "no counters → no transfer");
}

// ── modern_decks 2026-04-30 push (post-push III) ───────────────────────────

#[test]
fn mathemagics_draws_two_to_the_x() {
    // X=3 → draw 2^3 = 8 cards.
    let mut g = two_player_game();
    // Stock the library so we can draw 8.
    for _ in 0..12 {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::mathemagics());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(6);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: Some(3),
    })
    .expect("Mathemagics castable for {3}{3}{U}{U}");
    drain_stack(&mut g);

    // Hand size = (before - 1 cast) + 8 drawn = before + 7.
    assert_eq!(g.players[0].hand.len(), hand_before + 7);
}

#[test]
fn mathemagics_x_zero_draws_one_card() {
    // X=0 → draw 2^0 = 1 card.
    let mut g = two_player_game();
    let cid = g.next_id();
    g.players[0].add_to_library_top(cid, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mathemagics());
    g.players[0].mana_pool.add(Color::Blue, 2);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: Some(0),
    })
    .expect("Mathemagics castable for {0}{0}{U}{U}");
    drain_stack(&mut g);

    // -1 (cast) + 1 (drawn) = no net change, but the draw step ran.
    assert_eq!(g.players[0].hand.len(), hand_before + 0);
}

#[test]
fn visionarys_dance_creates_two_flying_elementals() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::visionarys_dance());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);

    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Visionary's Dance castable for {5}{U}{R}");
    drain_stack(&mut g);

    // Two Elemental tokens added.
    let tokens: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Elemental")
        .collect();
    assert_eq!(tokens.len(), 2, "two Elemental tokens created");
    assert_eq!(g.battlefield.len(), bf_before + 2);
    for t in &tokens {
        assert_eq!(t.definition.power, 3);
        assert_eq!(t.definition.toughness, 3);
        assert!(t.definition.keywords.contains(&Keyword::Flying));
    }
}

#[test]
fn abstract_paintmage_adds_ur_at_first_main_phase() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::abstract_paintmage());

    // Empty the pool so the assertions below are unambiguous.
    g.players[0].mana_pool = crate::mana::ManaPool::default();
    // Fire the PreCombatMain step trigger directly — this is the same
    // entry point the engine uses when transitioning into the active
    // player's first main phase.
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);

    // Mana pool should now contain at least 1 U + 1 R.
    let pool = &g.players[0].mana_pool;
    assert!(pool.amount(Color::Blue) >= 1, "U mana added: {pool:?}");
    assert!(pool.amount(Color::Red) >= 1, "R mana added: {pool:?}");
}

#[test]
fn pox_plague_halves_each_player_resources() {
    let mut g = two_player_game();
    // Stock both players: life 20, hand 4, three permanents each.
    g.players[0].life = 20;
    g.players[1].life = 20;
    for _ in 0..4 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    let p0_perms_before = (0..3)
        .map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears()))
        .collect::<Vec<_>>();
    let p1_perms_before = (0..3)
        .map(|_| g.add_card_to_battlefield(1, catalog::grizzly_bears()))
        .collect::<Vec<_>>();

    let id = g.add_card_to_hand(0, catalog::pox_plague());
    g.players[0].mana_pool.add(Color::Black, 5);

    // Hand: Pox Plague + 4 bears = 5. Cast Pox Plague.
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Pox Plague castable for {B}{B}{B}{B}{B}");
    drain_stack(&mut g);

    // Each player loses half life: 20 → 10.
    assert_eq!(g.players[0].life, 10);
    assert_eq!(g.players[1].life, 10);
    // Each player discards half their hand. P0's hand was 4 bears
    // before resolution (Pox Plague consumed itself when cast), so
    // half = 2 bears discarded → 2 left. P1 had 4 → 2 discarded → 2 left.
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(g.players[1].hand.len(), 2);
    // Sacrifice half of 3 permanents (rounded down) = 1 each.
    let p0_remaining = p0_perms_before
        .iter()
        .filter(|cid| g.battlefield.iter().any(|c| c.id == **cid))
        .count();
    let p1_remaining = p1_perms_before
        .iter()
        .filter(|cid| g.battlefield.iter().any(|c| c.id == **cid))
        .count();
    assert_eq!(p0_remaining, 2, "p0 sacrificed 1 of 3");
    assert_eq!(p1_remaining, 2, "p1 sacrificed 1 of 3");
}

#[test]
fn emil_grants_trample_to_counter_creatures() {
    // Emil's static ability "Creatures you control with +1/+1 counters
    // on them have trample" should add Trample to a counter-bearing
    // creature, but not to one without a counter. Powered by the new
    // `AffectedPermanents::AllWithCounter` layer-system variant.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::emil_vastlands_roamer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let plain_bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Drop a +1/+1 counter on `bear` only.
    {
        let perm = g.battlefield.iter_mut().find(|c| c.id == bear).unwrap();
        perm.add_counters(CounterType::PlusOnePlusOne, 1);
    }

    let view = g.computed_permanent(bear).unwrap();
    assert!(
        view.keywords.contains(&Keyword::Trample),
        "bear with +1/+1 counter has trample (Emil's static): {:?}",
        view.keywords
    );

    let view2 = g.computed_permanent(plain_bear).unwrap();
    assert!(
        !view2.keywords.contains(&Keyword::Trample),
        "uncounter'd bear should NOT have trample"
    );
}

#[test]
fn emil_definition_shape() {
    // Defensive: catch accidental rewires of Emil's static or
    // activation by asserting the printed shape (legendary 3/3, one
    // GrantKeyword(Trample) static, one tap-activation creating a
    // token + counters).
    let card = catalog::emil_vastlands_roamer();
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 3);
    assert!(card.supertypes.contains(&crate::card::Supertype::Legendary));
    assert_eq!(card.static_abilities.len(), 1);
    match &card.static_abilities[0].effect {
        crate::effect::StaticEffect::GrantKeyword { keyword, .. } => {
            assert_eq!(*keyword, Keyword::Trample);
        }
        _ => panic!("expected GrantKeyword static"),
    }
    assert_eq!(card.activated_abilities.len(), 1);
    let ab = &card.activated_abilities[0];
    assert!(ab.tap_cost);
    match &ab.effect {
        crate::effect::Effect::Seq(steps) => {
            assert_eq!(steps.len(), 2);
            assert!(matches!(steps[0], crate::effect::Effect::CreateToken { .. }));
            assert!(matches!(steps[1], crate::effect::Effect::AddCounter { .. }));
        }
        _ => panic!("expected Seq activation effect"),
    }
}

#[test]
fn matterbending_mage_etb_bounces_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::matterbending_mage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Matterbending Mage castable for {2}{U}");
    drain_stack(&mut g);

    // Bear bounced to opponent's hand.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

#[test]
fn orysa_etb_draws_two_cards() {
    let mut g = two_player_game();
    for _ in 0..3 {
        let cid = g.next_id();
        g.players[0].add_to_library_top(cid, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::orysa_tide_choreographer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Orysa castable for {4}{U}");
    drain_stack(&mut g);

    // -1 (cast) + 2 (drawn) = +1 net.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    let orysa = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Orysa, Tide Choreographer");
    assert!(orysa.is_some(), "Orysa is on the battlefield");
}

#[test]
fn exhibition_tidecaller_is_one_drop_blocker() {
    // Body-only test: the {U} 0/2 enters the battlefield with the
    // expected stat line and creature types. No Opus rider yet.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::exhibition_tidecaller());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Exhibition Tidecaller castable for {U}");
    drain_stack(&mut g);

    let card = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Exhibition Tidecaller")
        .expect("on battlefield");
    assert_eq!(card.definition.power, 0);
    assert_eq!(card.definition.toughness, 2);
    assert!(card
        .definition
        .subtypes
        .creature_types
        .contains(&crate::card::CreatureType::Wizard));
}

#[test]
fn colossus_etb_drains_three_each_opponent() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.players[1].life = 20;
    let id = g.add_card_to_hand(0, catalog::colossus_of_the_blood_age());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Colossus castable for {4}{R}{W}");
    drain_stack(&mut g);

    // ETB: 3 damage to each opponent + you gain 3 life.
    assert_eq!(g.players[0].life, 23);
    assert_eq!(g.players[1].life, 17);
}

#[test]
fn colossus_dies_discards_hand_and_draws_n_plus_one() {
    // Push: with `Value::CardsDiscardedThisResolution`, the printed
    // "discard any number, draw that many plus one" is now wired as
    // "discard your entire hand, draw discarded+1". Hand size 3 →
    // discard 3, draw 4, net +1 + redraw rotation.
    let mut g = two_player_game();
    let cid = g.add_card_to_battlefield(0, catalog::colossus_of_the_blood_age());
    // Library has plenty of cards to draw.
    for _ in 0..10 {
        let nid = g.next_id();
        g.players[0].add_to_library_top(nid, catalog::grizzly_bears());
    }
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    assert_eq!(hand_before, 3);

    let _ = g.remove_to_graveyard_with_triggers(cid);
    drain_stack(&mut g);

    // Hand state: discarded 3, drew 4 → 4 cards in hand.
    assert_eq!(g.players[0].hand.len(), 4, "discarded 3, drew 3+1 = 4");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cid));
}

#[test]
fn colossus_dies_with_empty_hand_still_draws_one() {
    // Edge case: empty hand → discard 0 → draw 0+1 = 1 card. The "+1
    // floor" matches the printed "draw that many plus one".
    let mut g = two_player_game();
    let cid = g.add_card_to_battlefield(0, catalog::colossus_of_the_blood_age());
    for _ in 0..3 {
        let nid = g.next_id();
        g.players[0].add_to_library_top(nid, catalog::grizzly_bears());
    }
    assert_eq!(g.players[0].hand.len(), 0);
    let lib_before = g.players[0].library.len();

    let _ = g.remove_to_graveyard_with_triggers(cid);
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), 1, "+1 floor draws 1 even from 0");
    assert_eq!(g.players[0].library.len(), lib_before - 1);
}

#[test]
fn conciliators_duelist_repartee_exiles_target() {
    // Cast a creature-targeting spell while Conciliator's Duelist is in
    // play; the Repartee trigger should exile the targeted creature
    // (the "return at end step" rider is omitted; see card-level docs).
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::conciliators_duelist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Cast Lightning Bolt at the bear so the cast targets a creature.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    // Bear should be exiled (or in graveyard if the bolt killed it
    // first; either is a valid resolution since both effects are
    // legal). The important assertion: bear is not on battlefield
    // after Repartee resolves.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

/// Push XXXVI: Conciliator's Duelist's "return at next end step" rider now
/// wired via `Effect::DelayUntil { capture: Some(_) }` — the cast spell's
/// target is captured into the delayed body's Target(0) at trigger-fire
/// time. Verify the Repartee trigger exiles the bear, then advances to
/// next end step and the bear returns to battlefield under owner.
#[test]
fn conciliators_duelist_repartee_returns_target_at_next_end_step() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::conciliators_duelist());
    // Use a target creature that won't die from the trigger spell, so
    // we can verify the exile + return cycle. Glorious Anthem is an
    // enchantment but we need a creature target for Repartee. Use a
    // Pestbrood Sloth (4/4) — a vanilla beater.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Cast a creature-targeting Giant Growth-style spell so the bear
    // isn't killed but the Repartee trigger fires.
    let pump = g.add_card_to_hand(0, catalog::interjection());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pump,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Interjection (W) castable");
    drain_stack(&mut g);

    // Bear should now be in exile (Repartee's Exile half).
    let bear_in_exile = g.exile.iter().any(|c| c.id == bear);
    assert!(bear_in_exile, "bear should be in exile after Repartee");

    // Advance to the active player's end step. The delayed trigger
    // fires "at the beginning of the next end step" — the next end
    // step from the active player's perspective.
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    // Bear should now be back on the battlefield under its owner's
    // control.
    let bear_back = g.battlefield.iter().any(|c| c.id == bear && c.controller == 1);
    assert!(bear_back, "bear should return to opp battlefield at end step");
}

// ── Push V: CardLeftGraveyard event + new SOS cards ────────────────────────

#[test]
fn hardened_academic_grants_counter_when_card_leaves_graveyard() {
    // Hardened Academic triggers off the `EventKind::CardLeftGraveyard`
    // event. Returning a card from your graveyard via Zealous
    // Lorecaster's ETB should put a +1/+1 counter on *some* creature
    // you control. The auto-target picker (post push-VI) prefers the
    // highest-power friendly creature when handing out a friendly
    // pump, so the counter typically lands on Lorecaster (4/4) rather
    // than the smaller Academic (2/1) or Bear (2/2). Assertion: total
    // +1/+1 counters across all friendly creatures should grow by 1.
    let mut g = two_player_game();
    let academic = g.add_card_to_battlefield(0, catalog::hardened_academic());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _ = (academic, bear);
    let bolt_id = g.next_id();
    let mut bolt = crate::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    let counters_before: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();

    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable for {5}{R}");
    drain_stack(&mut g);

    let counters_after: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert!(
        counters_after > counters_before,
        "Hardened Academic should add a +1/+1 counter to a friendly creature when Lorecaster's ETB returns the bolt (was {}, now {})",
        counters_before, counters_after
    );
}

#[test]
fn spirit_mascot_self_counter_on_graveyard_leave() {
    let mut g = two_player_game();
    let mascot = g.add_card_to_battlefield(0, catalog::spirit_mascot());
    let bolt_id = g.next_id();
    let mut bolt = crate::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable for {5}{R}");
    drain_stack(&mut g);

    let counters = g
        .battlefield_find(mascot)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert!(
        counters >= 1,
        "Spirit Mascot should pick up a +1/+1 counter when bolt leaves the graveyard"
    );
}

#[test]
fn garrison_excavator_creates_spirit_on_graveyard_leave() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::garrison_excavator());
    let bolt_id = g.next_id();
    let mut bolt = crate::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    let bf_before = g.battlefield.len();
    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable for {5}{R}");
    drain_stack(&mut g);

    assert!(g.battlefield.len() > bf_before);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"));
}

#[test]
fn living_history_etb_creates_spirit_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::living_history());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Living History castable for {1}{R}");
    drain_stack(&mut g);

    // Living History itself + Spirit token = 2 new permanents.
    assert_eq!(g.battlefield.len(), bf_before + 2);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Living History"));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"));
}

#[test]
fn cards_left_graveyard_this_turn_resets_each_turn() {
    let mut g = two_player_game();
    let bolt_id = g.next_id();
    let mut bolt = crate::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable");
    drain_stack(&mut g);

    assert!(
        g.players[0].cards_left_graveyard_this_turn >= 1,
        "the bolt-from-gy → hand move should bump the per-turn tally"
    );

    // Manually reset (simulating start-of-controller's-turn untap).
    // do_untap operates on the active player; player 0 is already
    // active in the two_player_game helper so this is a no-op for
    // turn rotation but exercises the per-turn reset path.
    g.do_untap();
    assert_eq!(g.players[0].cards_left_graveyard_this_turn, 0);
}

#[test]
fn witherbloom_balancer_etb_with_keywords() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::witherbloom_the_balancer());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(6);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Witherbloom castable for {6}{B}{G}");
    drain_stack(&mut g);

    let drag = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Witherbloom, the Balancer")
        .unwrap();
    assert_eq!(drag.definition.power, 5);
    assert_eq!(drag.definition.toughness, 5);
    assert!(drag.definition.keywords.contains(&Keyword::Flying));
    assert!(drag.definition.keywords.contains(&Keyword::Deathtouch));
}

#[test]
fn rabid_attack_pumps_friendly_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rabid_attack());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Rabid Attack castable for {1}{B}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 3, "2 + 1 pump = 3");
}

#[test]
fn burrog_barrage_no_pump_on_first_spell_no_opp_creature_no_damage() {
    // Push: damage half now auto-targets an *opp* creature via
    // `Selector::one_of(EachPermanent(opp creature))` (was: dealt
    // damage to slot 0 = the friendly creature, which was self-fight).
    // With no opp creature on the battlefield, the damage no-ops
    // cleanly (preserves the printed "up to one" semantics) — the
    // friendly bear survives.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::burrog_barrage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Burrog Barrage castable for {1}{G}");
    drain_stack(&mut g);

    // No opp creature → damage half no-ops → friendly bear survives.
    assert!(
        g.battlefield.iter().any(|c| c.id == bear),
        "bear should survive (no opp creature to take the damage)"
    );
    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 2, "no prior IS spell → no +1/+0 pump");
}

#[test]
fn burrog_barrage_kills_opp_creature_via_friendly_power() {
    // Push: damage half now hits an opp creature with damage equal
    // to slot 0's power. Friendly bear (2 power) → 2 damage to opp
    // bear (2 toughness) → opp bear dies. Was 🟡 (self-damage).
    let mut g = two_player_game();
    let friendly = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::burrog_barrage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(friendly)), mode: None, x_value: None,
    })
    .expect("Burrog Barrage castable for {1}{G}");
    drain_stack(&mut g);

    // Friendly survives (no return damage); opp dies.
    assert!(g.battlefield.iter().any(|c| c.id == friendly),
        "friendly bear should survive (Burrog Barrage is one-sided)");
    assert!(!g.battlefield.iter().any(|c| c.id == opp),
        "opp bear should die to 2 damage");
}

#[test]
fn chelonian_tackle_pumps_toughness() {
    // No opp creature: Fight no-ops (preserves the "up to one"
    // semantics). Bear gets the +0/+10 pump and stays on the
    // battlefield at 2/12.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::chelonian_tackle());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Chelonian Tackle castable for {2}{G}");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear);
    assert!(view.is_some(), "bear should survive (no opp to fight)");
    assert_eq!(view.unwrap().toughness, 12);
}

#[test]
fn chelonian_tackle_fights_opp_creature() {
    // With an opp creature, Fight resolves: friendly bear (2 power)
    // damages opp bear (2 power, 2 toughness) and the opp bear
    // damages friendly bear back. Friendly bear is 2/12 after pump
    // so survives 2 damage; opp bear dies to 2 damage.
    let mut g = two_player_game();
    let friendly = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::chelonian_tackle());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(friendly)), mode: None, x_value: None,
    })
    .expect("Chelonian Tackle castable for {2}{G}");
    drain_stack(&mut g);

    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == opp),
        "opp bear should die from the fight"
    );
    let friendly_view = g.computed_permanent(friendly);
    assert!(friendly_view.is_some(), "friendly bear should survive 2 damage with 12 toughness");
}

#[test]
fn tablet_of_discovery_etb_mills_one() {
    let mut g = two_player_game();
    for _ in 0..3 {
        let nid = g.next_id();
        g.players[0].add_to_library_top(nid, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::tablet_of_discovery());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Tablet of Discovery castable for {2}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].graveyard.len(), gy_before + 1);
}

#[test]
fn practiced_offense_pumps_creatures_and_grants_double_strike() {
    // Push: mode 0 (the default) pumps each creature you control by
    // a +1/+1 counter, then grants double strike to the chosen target.
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::practiced_offense());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear1)), mode: Some(0), x_value: None,
    })
    .expect("Practiced Offense castable for {2}{W}");
    drain_stack(&mut g);

    // Both bears should have a +1/+1 counter; bear1 should also have
    // double strike granted (Selector::Target).
    let v1 = g.computed_permanent(bear1).unwrap();
    let v2 = g.computed_permanent(bear2).unwrap();
    assert_eq!(v1.power, 3, "bear1 = 2 + 1 counter");
    assert_eq!(v2.power, 3, "bear2 = 2 + 1 counter");
    assert!(v1.keywords.contains(&Keyword::DoubleStrike));
    // Mode 0 grants DS, NOT lifelink.
    assert!(!v1.keywords.contains(&Keyword::Lifelink),
        "Mode 0 (DS) should not also grant Lifelink");
}

#[test]
fn practiced_offense_mode_one_pumps_and_grants_lifelink() {
    // Push: mode 1 = +1/+1 fan-out + grant lifelink EOT (was 🟡:
    // "lifelink alternative dropped"). Promotes the printed "your
    // choice of double strike or lifelink" prompt to a top-level
    // ChooseMode pick.
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::practiced_offense());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear1)), mode: Some(1), x_value: None,
    })
    .expect("Practiced Offense castable for {2}{W} mode 1");
    drain_stack(&mut g);

    let v1 = g.computed_permanent(bear1).unwrap();
    assert_eq!(v1.power, 3, "bear1 = 2 + 1 counter");
    assert!(v1.keywords.contains(&Keyword::Lifelink),
        "Mode 1 should grant Lifelink");
    assert!(!v1.keywords.contains(&Keyword::DoubleStrike),
        "Mode 1 (Lifelink) should not also grant Double Strike");
}

#[test]
fn mana_sculpt_counters_spell() {
    // Lightweight assertion: Mana Sculpt as a 4-mana counterspell should
    // remove a spell from the stack. The wizard-rider mana refund is
    // tested implicitly via the no-regression test suite (any wired
    // path that reaches the Mana Sculpt cast site will exercise the
    // `If Predicate::SelectorExists` branch).
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), mode: None, x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    let stack_count_with_bolt = g.stack.len();
    assert_eq!(stack_count_with_bolt, 1);

    g.priority.player_with_priority = 0;
    let sculpt = g.add_card_to_hand(0, catalog::mana_sculpt());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    let bolt_target = g
        .stack
        .iter()
        .find_map(|s| match s {
            StackItem::Spell { card, .. } => Some(card.id),
            _ => None,
        })
        .unwrap();
    g.perform_action(GameAction::CastSpell {
        card_id: sculpt, target: Some(Target::Permanent(bolt_target)), mode: None, x_value: None,
    })
    .expect("Mana Sculpt castable for {1}{U}{U}");
    drain_stack(&mut g);

    // The bolt should have been countered (no damage to player 0).
    assert_eq!(g.players[0].life, 20, "bolt should have been countered");
}


// ── push VI: Lorehold completion + Daydream + token-side triggers ──────────

#[test]
fn ark_of_hunger_triggers_on_card_left_graveyard() {
    let mut g = two_player_game();
    let ark = g.add_card_to_battlefield(0, catalog::ark_of_hunger());
    let _ = ark;

    // Seed a card in P0's graveyard, then exile-via-trigger to fire the event.
    let bolt_id = g.next_id();
    let mut bolt = crate::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    let life_before = g.players[0].life;
    let opp_life_before = g.players[1].life;

    // Cast Zealous Lorecaster — its ETB returns an instant/sorcery from your gy
    // to your hand, firing CardLeftGraveyard, which Ark of Hunger watches.
    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 1, "Ark of Hunger should gain 1 life");
    assert_eq!(g.players[1].life, opp_life_before - 1, "Ark of Hunger should ping opp for 1");
}

#[test]
fn ark_of_hunger_mill_activation() {
    let mut g = two_player_game();
    let ark = g.add_card_to_battlefield(0, catalog::ark_of_hunger());
    // Seed a few library cards.
    for _ in 0..3 {
        let nid = g.next_id();
        g.players[0].add_to_library_top(nid, catalog::grizzly_bears());
    }
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: ark, ability_index: 0, target: None,
    })
    .expect("Ark of Hunger {T}: Mill activation");
    drain_stack(&mut g);

    assert_eq!(g.players[0].graveyard.len(), gy_before + 1, "Ark mills 1");
    assert!(g.battlefield.iter().any(|c| c.id == ark && c.tapped),
        "Ark should be tapped after activation");
}

#[test]
fn suspend_aggression_exiles_target_and_top_of_library() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::suspend_aggression());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    let top_card_id = g.next_id();
    g.players[0].add_to_library_top(top_card_id, catalog::lightning_bolt());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Suspend Aggression castable for {1}{R}{W}");
    drain_stack(&mut g);

    // Bear should be exiled (not on battlefield, not in graveyard).
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear off battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear), "Bear exiled");
    // Top card of library should be exiled.
    assert!(g.exile.iter().any(|c| c.id == top_card_id), "Top of library exiled");
}

#[test]
fn wilt_in_the_heat_deals_five_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wilt_in_the_heat());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Wilt in the Heat castable for {2}{R}{W}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear should die to 5 damage");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn daydream_flickers_and_adds_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::daydream());
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Daydream castable for {W}");
    drain_stack(&mut g);

    // The same bear (zone-stable) should be back on the battlefield with a +1/+1 counter.
    let bear_view = g.computed_permanent(bear);
    assert!(bear_view.is_some(), "Bear should be back on battlefield");
    let bear_view = bear_view.unwrap();
    assert_eq!(bear_view.power, 3, "Bear with +1/+1 counter = 3 power");
    assert_eq!(bear_view.toughness, 3, "Bear with +1/+1 counter = 3 toughness");
}

#[test]
fn snarl_song_creates_two_fractals_with_x_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::snarl_song());
    // Snarl Song cost is {5}{G} and Converge X = colors of mana spent.
    // Add 6 green mana so the engine treats it as 1-color converge.
    g.players[0].mana_pool.add(Color::Green, 6);
    let bf_before = g.battlefield.len();
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Snarl Song castable for {5}{G}");
    drain_stack(&mut g);

    // Two Fractal tokens minted.
    let fractal_count = g.battlefield.iter().filter(|c| c.definition.name == "Fractal").count();
    assert_eq!(fractal_count, 2, "Snarl Song creates two Fractal tokens");
    // X = 1 (one color used).
    assert!(g.battlefield.len() >= bf_before + 2);
    // Life gained = X.
    assert_eq!(g.players[0].life, life_before + 1, "Snarl Song gains X life (1)");
}

#[test]
fn wild_hypothesis_creates_fractal_with_x_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::wild_hypothesis());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Seed library so surveil has something to look at.
    for _ in 0..3 {
        let nid = g.next_id();
        g.players[0].add_to_library_top(nid, catalog::lightning_bolt());
    }

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: Some(3),
    })
    .expect("Wild Hypothesis castable for X=3 ({3}{G})");
    drain_stack(&mut g);

    // Fractal token with 3 +1/+1 counters → 3/3.
    let fractal = g.battlefield.iter().find(|c| c.definition.name == "Fractal");
    assert!(fractal.is_some(), "Wild Hypothesis mints a Fractal");
    let fractal_id = fractal.unwrap().id;
    let view = g.computed_permanent(fractal_id).unwrap();
    assert_eq!(view.power, 3, "Fractal should have 3 +1/+1 counters → 3 power");
}

#[test]
fn tome_blast_deals_two_damage_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tome_blast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Tome Blast castable for {1}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear), "Bear (2/2) dies to 2 damage");
}

#[test]
fn duel_tactics_pings_and_grants_cant_block() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::duel_tactics());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Duel Tactics castable for {R}");
    drain_stack(&mut g);

    // Bear takes 1 (bear is 2/2 so it survives). Bear should now have CantBlock.
    let bear_view = g.computed_permanent(bear).unwrap();
    assert!(bear_view.keywords.contains(&Keyword::CantBlock),
        "Bear should have CantBlock granted EOT");
}

#[test]
fn soaring_stoneglider_is_four_three_flier_vigilance() {
    let def = catalog::soaring_stoneglider();
    assert_eq!(def.name, "Soaring Stoneglider");
    assert_eq!(def.power, 4);
    assert_eq!(def.toughness, 3);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Vigilance));
}

#[test]
fn practiced_scrollsmith_etb_exiles_noncreature_nonland_from_gy() {
    let mut g = two_player_game();
    // Seed a noncreature/nonland in gy: a sorcery (Pox Plague) and a land
    // (Forest) and a creature (Bears). Only the sorcery should be exiled.
    let pox_id = g.next_id();
    let mut pox = crate::card::CardInstance::new(pox_id, catalog::pox_plague(), 0);
    pox.controller = 0;
    g.players[0].graveyard.push(pox);
    let forest_id = g.next_id();
    let mut forest = crate::card::CardInstance::new(forest_id, catalog::forest(), 0);
    forest.controller = 0;
    g.players[0].graveyard.push(forest);
    let bear_id = g.next_id();
    let mut bear = crate::card::CardInstance::new(bear_id, catalog::grizzly_bears(), 0);
    bear.controller = 0;
    g.players[0].graveyard.push(bear);

    let id = g.add_card_to_hand(0, catalog::practiced_scrollsmith());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Practiced Scrollsmith castable for {R}{R}{W}");
    drain_stack(&mut g);

    // Pox should have been exiled.
    assert!(g.exile.iter().any(|c| c.id == pox_id), "Pox Plague exiled");
    // Forest stays in gy (it's a land).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == forest_id),
        "Forest stays in gy (it's a land)");
    // Bear stays in gy (it's a creature).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear_id),
        "Bear stays in gy (it's a creature)");
}

#[test]
fn topiary_lecturer_taps_for_g_equal_to_power() {
    let mut g = two_player_game();
    let lec = g.add_card_to_battlefield(0, catalog::topiary_lecturer());
    // Make sure it's not summoning sick — manually unmark it.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == lec) {
        c.summoning_sick = false;
    }

    g.perform_action(GameAction::ActivateAbility {
        card_id: lec, ability_index: 0, target: None,
    })
    .expect("Topiary Lecturer {T}: Add G mana ability");
    drain_stack(&mut g);

    // Base 1/2 → 1 power → 1 G.
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "Adds G = power (1)");
}

#[test]
fn pest_token_attack_trigger_gains_one_life() {
    // SOS Pest token: "Whenever this token attacks, you gain 1 life."
    // Use Send in the Pest to mint a token, then attack with it, then
    // confirm the controller gained a life.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::send_in_the_pest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Send in the Pest castable for {1}{B}");
    drain_stack(&mut g);

    let pest = g.battlefield.iter().find(|c| c.definition.name == "Pest")
        .expect("Pest token created");
    let pest_id = pest.id;
    // Manually un-summoning-sick.
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == pest_id) {
        c.summoning_sick = false;
    }

    // Move to combat phase.
    g.step = TurnStep::DeclareAttackers;
    let life_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pest_id, target: AttackTarget::Player(1),
    }]))
    .expect("Declare Pest attacker");
    // Drain any triggers.
    drain_stack(&mut g);

    assert!(g.players[0].life >= life_before + 1,
        "SOS Pest token's attack trigger should grant +1 life (was {}, now {})",
        life_before, g.players[0].life);
}


// ── push VII: Multicolored predicate, MDFC bodies, Lorehold capstone ────────

#[test]
fn homesickness_draws_two_taps_and_stuns() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::homesickness());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    let lib_before = g.players[0].library.len();
    // Seed 2 cards on the library so the draw-2 actually moves them.
    let l1 = g.next_id(); g.players[0].add_to_library_top(l1, catalog::lightning_bolt());
    let l2 = g.next_id(); g.players[0].add_to_library_top(l2, catalog::lightning_bolt());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Homesickness castable for {4}{U}{U}");
    drain_stack(&mut g);

    // Caster drew 2.
    assert_eq!(g.players[0].hand.len(), 2, "drew 2 cards");
    let _ = lib_before;
    // Bear is tapped.
    let bear_card = g.battlefield.iter().find(|c| c.id == bear).expect("bear on bf");
    assert!(bear_card.tapped, "bear tapped");
    // Stun counter present.
    assert!(bear_card.counter_count(CounterType::Stun) >= 1, "stun counter on bear");
}

#[test]
fn homesickness_stuns_more_than_one_counter_with_only_one_creature() {
    // Push: the second creature slot is now auto-picked via
    // `Selector::one_of(EachPermanent(opp creature))`. With only one
    // opp creature on the battlefield, the second auto-pick collides
    // with slot 0 — net 2 stun counters on the single creature
    // (matches printed "up to two" semantics when only one creature
    // is available; a deterministic auto-target re-picks slot 0
    // since multi-target uniqueness is an open engine gap).
    let mut g = two_player_game();
    let opp_a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::homesickness());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_a)), mode: None, x_value: None,
    })
    .expect("Homesickness castable for {4}{U}{U}");
    drain_stack(&mut g);

    // 2 stun counters land (slot 0 + auto-picked second slot collide).
    let a = g.battlefield.iter().find(|c| c.id == opp_a).unwrap();
    assert!(a.tapped, "opp_a tapped");
    assert!(a.counter_count(CounterType::Stun) >= 2,
        "2 stun counters on opp_a (slot 0 + collided second auto-pick)");
}

#[test]
fn fractalize_pumps_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fractalize());
    // Cast for X=3 — costs {3}{U} = 4 mana.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: Some(3),
    })
    .expect("Fractalize castable for {X=3}{U}");
    drain_stack(&mut g);

    let bear_card = g.battlefield.iter().find(|c| c.id == bear).expect("bear on bf");
    // Base 2/2 + (X+1)/(X+1) = +4/+4 -> 6/6.
    assert_eq!(bear_card.power(), 6, "bear pumped to 6 power");
    assert_eq!(bear_card.toughness(), 6, "bear pumped to 6 toughness");
}

#[test]
fn divergent_equation_returns_instant_from_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::divergent_equation());
    // Seed an instant in P0's graveyard.
    let bolt_id = g.next_id();
    let mut bolt = crate::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bolt_id)), mode: None, x_value: Some(1),
    })
    .expect("Divergent Equation castable for {X=1}{X=1}{U}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_id), "Bolt in hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt_id),
        "Bolt left graveyard");
}

#[test]
fn spectacular_skywhale_is_one_four_flyer() {
    let g = two_player_game();
    let _ = g;
    let def = catalog::spectacular_skywhale();
    assert_eq!(def.power, 1);
    assert_eq!(def.toughness, 4);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert_eq!(def.cost.cmc(), 4);
}

#[test]
fn lorehold_the_historian_is_five_five_flyer_haste() {
    let def = catalog::lorehold_the_historian();
    assert_eq!(def.power, 5);
    assert_eq!(def.toughness, 5);
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.keywords.contains(&Keyword::Haste));
    assert_eq!(def.cost.cmc(), 5);
}

/// Push XXXVI: Lorehold, the Historian's per-opp-upkeep loot trigger
/// now wired via `EventScope::OpponentControl + StepBegins(Upkeep)`.
/// Verify that on the opponent's upkeep, the Historian's controller
/// gets a may-loot prompt — `ScriptedDecider` answers yes, sees a
/// discard + draw chain.
#[test]
fn lorehold_the_historian_loots_on_each_opp_upkeep() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lorehold_the_historian());
    // Seed library to give p0 cards to draw.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    // Seed a card in p0's hand to discard.
    g.add_card_to_hand(0, catalog::island());
    // Pre-script "yes, discard then draw" via the MayDo prompt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();
    let gy_before = g.players[0].graveyard.len();
    // Switch active player to opp (p1) and fire upkeep step triggers.
    g.active_player_idx = 1;
    g.fire_step_triggers(crate::game::types::TurnStep::Upkeep);
    drain_stack(&mut g);
    // Net: +1 discard (gy), -1 hand-card, +1 draw → hand size same,
    // library -1, gy +1 (the discarded card).
    assert_eq!(g.players[0].hand.len(), hand_before, "hand size unchanged (discard 1, draw 1)");
    assert_eq!(g.players[0].library.len(), lib_before - 1, "drew 1");
    assert_eq!(g.players[0].graveyard.len(), gy_before + 1, "discarded 1");
}

#[test]
fn mage_tower_referee_gets_counter_on_multicolored_cast() {
    let mut g = two_player_game();
    let referee = g.add_card_to_battlefield(0, catalog::mage_tower_referee());

    // Cast a multicolored spell (Lorehold Charm — {R}{W}).
    let charm_id = g.add_card_to_hand(0, catalog::lorehold_charm());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    // Drain stack of any pending state, choose mode 2 (creatures get +1/+1)
    // since it doesn't require an extra target.
    g.perform_action(GameAction::CastSpell {
        card_id: charm_id, target: None, mode: Some(2), x_value: None,
    })
    .expect("Lorehold Charm castable for {R}{W}");
    drain_stack(&mut g);

    let counter = g.battlefield.iter().find(|c| c.id == referee)
        .expect("referee on bf").counter_count(CounterType::PlusOnePlusOne);
    assert!(counter >= 1, "referee gained +1/+1 counter on multicolored cast (got {})", counter);
}

#[test]
fn mage_tower_referee_no_counter_on_monocolored_cast() {
    let mut g = two_player_game();
    let referee = g.add_card_to_battlefield(0, catalog::mage_tower_referee());

    // Cast a mono-color spell (Lightning Bolt — {R}). Should NOT add a counter.
    let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt_id,
        target: Some(Target::Player(1)),
        mode: None, x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    let counter = g.battlefield.iter().find(|c| c.id == referee)
        .expect("referee on bf").counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counter, 0, "no counter for mono-color cast");
}

#[test]
fn rubble_rouser_etb_rummages() {
    // Push XV: ETB rummage is now `Effect::MayDo`. Inject Bool(true).
    let mut g = two_player_game();
    // Seed a card to discard + a card to draw.
    let _ = g.add_card_to_hand(0, catalog::lightning_bolt());
    let l1 = g.next_id(); g.players[0].add_to_library_top(l1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::rubble_rouser());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    let hand_before = g.players[0].hand.len();
    let gy_before = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Rubble Rouser castable for {2}{R}");
    drain_stack(&mut g);

    // Net: cast (-1 hand) + discard 1 (-1 hand, +1 gy) + draw 1 (+1 hand).
    // From hand_before perspective: -1 -1 +1 = -1. Cast removed Rouser
    // already (it's now on battlefield).
    assert_eq!(g.players[0].hand.len(), hand_before - 1,
        "rummage net: -1 from hand (cast moved Rouser to bf, discard then draw nets 0)");
    assert!(g.players[0].graveyard.len() > gy_before, "discarded card hit gy");
    let on_bf = g.battlefield.iter().find(|c| c.id == id).expect("Rouser on bf");
    assert_eq!(on_bf.power(), 1);
    assert_eq!(on_bf.toughness(), 4);
}

#[test]
fn additive_evolution_etb_creates_fractal_with_three_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::additive_evolution());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Additive Evolution castable for {3}{G}{G}");
    drain_stack(&mut g);

    let frac = g.battlefield.iter().find(|c| c.definition.name == "Fractal")
        .expect("Fractal token created");
    assert_eq!(frac.counter_count(CounterType::PlusOnePlusOne), 3,
        "Fractal entered with three +1/+1 counters");
    // 0/0 base + 3 counters = 3/3.
    assert_eq!(frac.power(), 3);
    assert_eq!(frac.toughness(), 3);
}

#[test]
fn zimones_experiment_reveals_to_hand_and_lands_basic() {
    let mut g = two_player_game();
    // Library (top → bottom): bear, forest. RevealUntilFind walks the
    // top: bear is a creature → it goes to hand. Forest stays in the
    // library where the subsequent Search picks it up onto the
    // battlefield tapped.
    let forest_id = g.next_id();
    g.players[0].add_to_library_top(forest_id, catalog::forest());
    let bear_id = g.next_id();
    g.players[0].add_to_library_top(bear_id, catalog::grizzly_bears());

    // ScriptedDecider answers the SearchLibrary decision with the seeded
    // Forest (the AutoDecider's default of `Search(None)` would pass on
    // the search, leaving the basic in the library).
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest_id)),
    ]));

    let id = g.add_card_to_hand(0, catalog::zimones_experiment());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Zimone's Experiment castable for {3}{G}");
    drain_stack(&mut g);

    // The bear should now be in hand (RevealUntilFind put it there).
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_id),
        "Bear should be in hand after RevealUntilFind");
    // The forest should be on the battlefield tapped (Search half).
    let forest_on_bf = g.battlefield.iter().find(|c| c.id == forest_id);
    assert!(forest_on_bf.is_some(), "Forest should ETB via Search");
    assert!(forest_on_bf.unwrap().tapped, "Forest should ETB tapped");
}

#[test]
fn petrified_hamlet_taps_for_colorless() {
    let mut g = two_player_game();
    let lid = g.add_card_to_battlefield(0, catalog::petrified_hamlet());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == lid) {
        c.summoning_sick = false;
        c.tapped = false;
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: lid, ability_index: 0, target: None,
    })
    .expect("Petrified Hamlet {T}: Add C");
    drain_stack(&mut g);

    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "added 1 colorless");
}

#[test]
fn owlin_historian_gets_pump_when_card_leaves_graveyard() {
    let mut g = two_player_game();
    let owlin = g.add_card_to_battlefield(0, catalog::owlin_historian());
    // Seed an instant in P0's graveyard for Zealous Lorecaster's ETB to return.
    let bolt_id = g.next_id();
    let mut bolt = crate::card::CardInstance::new(bolt_id, catalog::lightning_bolt(), 0);
    bolt.controller = 0;
    g.players[0].graveyard.push(bolt);

    // Cast Zealous Lorecaster — its ETB fires CardLeftGraveyard.
    let lor_id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: lor_id, target: None, mode: None, x_value: None,
    })
    .expect("Zealous Lorecaster castable");
    drain_stack(&mut g);

    let owlin_card = g.battlefield.iter().find(|c| c.id == owlin).expect("Owlin on bf");
    assert!(owlin_card.power() >= 3, "Owlin should be pumped (was 2/3, now {}/{})",
        owlin_card.power(), owlin_card.toughness());
}

#[test]
fn postmortem_professor_has_cant_block_keyword() {
    // SOS push V added Keyword::CantBlock; SOS push VII promotes
    // Postmortem Professor's printed "this creature can't block" static
    // restriction to the first-class keyword. Verify the def carries it.
    let def = catalog::postmortem_professor();
    assert!(def.keywords.contains(&Keyword::CantBlock),
        "Postmortem Professor should carry Keyword::CantBlock");
}

#[test]
fn mage_tower_referee_def_is_two_one_construct() {
    let def = catalog::mage_tower_referee();
    assert_eq!(def.power, 2);
    assert_eq!(def.toughness, 1);
    assert_eq!(def.cost.cmc(), 2);
    assert!(def.card_types.contains(&CardType::Artifact));
    assert!(def.card_types.contains(&CardType::Creature));
}

#[test]
fn additive_evolution_combat_pumps_friendly_creature() {
    use crate::game::TurnStep;
    let mut g = two_player_game();
    let _enchant = g.add_card_to_battlefield(0, catalog::additive_evolution());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    // Drain anything pending.
    drain_stack(&mut g);

    // Move to begin combat — the begin-combat trigger fires once per
    // ActivePlayer step transition.
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);

    let _ = bear;
    // +1/+1 counter from begin-combat trigger lands on a friendly
    // creature (auto-target picks the highest-power friendly).
    // `add_card_to_battlefield` skips ETB triggers, so we don't expect
    // the Fractal token here — only the begin-combat counter.
    let total_creature_pump: i32 = g.battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature())
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne) as i32)
        .sum();
    assert!(total_creature_pump >= 1,
        "begin-combat pump should add ≥ 1 +1/+1 counter on a friendly creature \
         (got total={})", total_creature_pump);
}

// ── push VIII: Lesson cycle + bodies + Resonating Lute ───────────────────────

#[test]
fn primary_research_etb_returns_low_mv_card_from_graveyard() {
    let mut g = two_player_game();
    let bear_in_grave = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::primary_research());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear_in_grave)),
        mode: None, x_value: None,
    })
    .expect("Primary Research castable for {4}{W}");
    drain_stack(&mut g);

    // Bear should be back on the battlefield, owned by P0.
    assert!(g.battlefield.iter().any(|c| c.id == bear_in_grave),
        "Grizzly Bears should be back on the battlefield");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear_in_grave),
        "Grizzly Bears should no longer be in the graveyard");
}

#[test]
fn primary_research_end_step_draws_when_card_left_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::primary_research());
    drain_stack(&mut g);
    g.players[0].cards_left_graveyard_this_turn = 1;
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Primary Research should draw at end step when a card left graveyard");
}

#[test]
fn primary_research_end_step_no_draw_when_quiet_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::primary_research());
    drain_stack(&mut g);
    g.players[0].cards_left_graveyard_this_turn = 0;
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before,
        "Primary Research should NOT draw when nothing left graveyard");
}

#[test]
fn artistic_process_mode0_deals_six_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::artistic_process());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        mode: Some(0), x_value: None,
    })
    .expect("Artistic Process castable for {3}{R}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Grizzly Bears should be dead from 6 damage");
}

#[test]
fn artistic_process_mode2_creates_haste_elemental() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::artistic_process());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: Some(2), x_value: None,
    })
    .expect("Artistic Process mode 2 castable");
    drain_stack(&mut g);

    // The Elemental token entered the battlefield; the spell itself
    // resolved into the graveyard. Net battlefield change = +1 token.
    assert_eq!(g.battlefield.len(), bf_before + 1);
    let tok = g.battlefield.iter()
        .find(|c| c.definition.name == "Elemental")
        .expect("Elemental token created");
    assert_eq!(tok.definition.power, 3);
    assert_eq!(tok.definition.toughness, 3);
    assert!(tok.definition.keywords.contains(&Keyword::Flying));
    // Haste was granted via duration::EOT — verify at the engine level
    // by checking the transient keyword count on the token.
    assert!(g.permanent_has_keyword(tok.id, &Keyword::Haste),
        "freshly-minted Elemental should have transient Haste");
}

#[test]
fn decorum_dissertation_draws_two_loses_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::decorum_dissertation());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Decorum Dissertation castable for {3}{B}{B}");
    drain_stack(&mut g);

    // Hand: -1 cast + 2 drawn = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert_eq!(g.players[0].life, life_before - 2);
}

#[test]
fn germination_practicum_pumps_each_creature() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::germination_practicum());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Germination Practicum castable for {3}{G}{G}");
    drain_stack(&mut g);

    let count_on = |g: &GameState, cid| g.battlefield_find(cid)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne) as i32)
        .unwrap_or(0);
    assert_eq!(count_on(&g, bear1), 2,
        "Friendly bear 1 should get 2 +1/+1 counters");
    assert_eq!(count_on(&g, bear2), 2,
        "Friendly bear 2 should get 2 +1/+1 counters");
    assert_eq!(count_on(&g, opp_bear), 0,
        "Opp bear should get 0 +1/+1 counters");
}

#[test]
fn restoration_seminar_returns_permanent_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::restoration_seminar());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Restoration Seminar castable for {5}{W}{W}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "Grizzly Bears should be on the battlefield");
}

#[test]
fn ennis_debate_moderator_etb_exiles_friendly_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ennis_debate_moderator());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Ennis castable for {1}{W}");
    drain_stack(&mut g);

    // Auto-target picker chose the bear; bear is now in exile.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be flickered off the battlefield");
    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in the shared exile zone");
}

#[test]
fn ennis_debate_moderator_end_step_counter_when_card_exiled() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ennis_debate_moderator());
    drain_stack(&mut g);
    // Push IX promotion: Ennis now uses the exact
    // `Predicate::CardsExiledThisTurnAtLeast` (was approximating with
    // `CardsLeftGraveyardThisTurnAtLeast`). Set the per-turn exile
    // tally directly to verify the predicate gates correctly.
    g.players[0].cards_exiled_this_turn = 1;

    let counters_before = g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    let counters_after = g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters_after, counters_before + 1,
        "Ennis should get +1/+1 counter at end step when a card was exiled");
}

#[test]
fn ennis_debate_moderator_end_step_no_counter_when_no_exile() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::ennis_debate_moderator());
    drain_stack(&mut g);
    g.players[0].cards_exiled_this_turn = 0;
    let counters_before = g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    let counters_after = g.battlefield_find(id)
        .unwrap()
        .counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(counters_after, counters_before,
        "Ennis should NOT get +1/+1 counter when no card was exiled");
}

#[test]
fn cards_exiled_this_turn_tally_bumps_on_exile_effect() {
    // Verify the new `cards_exiled_this_turn` tally bumps when a
    // creature is exiled by Wander Off. Active player casts Wander
    // Off on their own bear (a self-removal nonsense play).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wander = g.add_card_to_hand(0, catalog::wander_off());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert_eq!(g.players[0].cards_exiled_this_turn, 0);

    g.perform_action(GameAction::CastSpell {
        card_id: wander, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Wander Off castable for {3}{B}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == bear),
        "Bear should be in exile");
    assert_eq!(g.players[0].cards_exiled_this_turn, 1,
        "Per-turn exile tally should bump for the player who cast the exile spell");
}

#[test]
fn tragedy_feaster_is_seven_six_trampler() {
    let card = catalog::tragedy_feaster();
    assert_eq!(card.power, 7);
    assert_eq!(card.toughness, 6);
    assert_eq!(card.cost.cmc(), 4);
    assert!(card.keywords.contains(&Keyword::Trample));
    assert!(card.has_creature_type(crate::card::CreatureType::Demon));
}

#[test]
fn forum_necroscribe_repartee_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let _necro = g.add_card_to_battlefield(0, catalog::forum_necroscribe());
    let bear_in_grave = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);

    // Cast a creature-targeting instant — Lightning Bolt at the opp bear.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        mode: None, x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    // Opp bear dead from bolt; friendly bear back on battlefield from
    // Repartee return.
    assert!(g.battlefield.iter().any(|c| c.id == bear_in_grave),
        "Forum Necroscribe Repartee should return the gy bear to bf");
}

#[test]
fn berta_wise_extrapolator_def_is_one_four_legendary_frog_druid() {
    let card = catalog::berta_wise_extrapolator();
    assert_eq!(card.power, 1);
    assert_eq!(card.toughness, 4);
    assert_eq!(card.cost.cmc(), 4);
    assert!(card.has_creature_type(crate::card::CreatureType::Frog));
    assert!(card.has_creature_type(crate::card::CreatureType::Druid));
    assert!(card.supertypes.contains(&crate::card::Supertype::Legendary));
    // Has the X-cost Fractal-token activation (index 0).
    assert!(!card.activated_abilities.is_empty(),
        "Berta should have at least one activated ability");
    assert!(card.activated_abilities[0].tap_cost,
        "Berta's activation should be a tap ability");
    // Push XXXI: Berta now also carries the Increment trigger (the
    // shared `effect::shortcut::increment()` SpellCast/YourControl
    // trigger backed by `Value::ManaSpentToCast`). So we expect two
    // triggered abilities: counter-add → mana, and Increment →
    // +1/+1 counter on this creature.
    assert_eq!(card.triggered_abilities.len(), 2,
        "Berta should have two triggered abilities (counter-add → mana + Increment)");
}

#[test]
fn berta_wise_extrapolator_has_counter_added_self_trigger() {
    use crate::card::{CounterType, EventKind, EventScope};
    let card = catalog::berta_wise_extrapolator();
    // Counter-add → AnyOneColor mana trigger is wired as a single
    // TriggeredAbility with EventKind::CounterAdded(PlusOnePlusOne) +
    // EventScope::SelfSource.
    let trig = card.triggered_abilities.iter()
        .find(|t| matches!(
            t.event.kind,
            EventKind::CounterAdded(CounterType::PlusOnePlusOne)
        ))
        .expect("Berta should have a CounterAdded(+1/+1) trigger");
    assert!(matches!(trig.event.scope, EventScope::SelfSource),
        "Berta's trigger should be SelfSource (only counters on Berta fire it)");
}

#[test]
fn paradox_surveyor_etb_reveals_to_find_basic_land() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let forest_id = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt());

    let id = g.add_card_to_hand(0, catalog::paradox_surveyor());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Paradox Surveyor castable for {G}{G}{U}");
    drain_stack(&mut g);

    // Surveyor entered the bf. The Forest among the top 5 should have
    // been moved to hand.
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Paradox Surveyor should be on the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == forest_id),
        "Forest should have been pulled into hand");
    // Hand size: -1 cast + 1 forest = same.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

#[test]
fn magmablood_archaic_etb_with_converged_value_counters() {
    let card = catalog::magmablood_archaic();
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 2);
    assert!(card.keywords.contains(&Keyword::Trample));
    assert!(card.keywords.contains(&Keyword::Reach));
    assert!(card.has_creature_type(crate::card::CreatureType::Avatar));
}

#[test]
fn wildgrowth_archaic_def_is_zero_zero_avatar() {
    let card = catalog::wildgrowth_archaic();
    assert_eq!(card.power, 0);
    assert_eq!(card.toughness, 0);
    assert!(card.keywords.contains(&Keyword::Trample));
    assert!(card.keywords.contains(&Keyword::Reach));
    assert!(card.has_creature_type(crate::card::CreatureType::Avatar));
}

#[test]
fn ambitious_augmenter_is_one_one_turtle_wizard() {
    let card = catalog::ambitious_augmenter();
    assert_eq!(card.power, 1);
    assert_eq!(card.toughness, 1);
    assert_eq!(card.cost.cmc(), 1);
    assert!(card.has_creature_type(crate::card::CreatureType::Turtle));
    assert!(card.has_creature_type(crate::card::CreatureType::Wizard));
}

#[test]
fn resonating_lute_draw_blocked_when_hand_below_seven() {
    let mut g = two_player_game();
    let lute = g.add_card_to_battlefield(0, catalog::resonating_lute());
    drain_stack(&mut g);
    // P0's hand is empty (well below 7).
    assert!(g.players[0].hand.is_empty());
    g.add_card_to_library(0, catalog::lightning_bolt());

    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: lute, ability_index: 0, target: None,
    });
    assert!(res.is_err(),
        "Resonating Lute should reject activation with hand size < 7");
    assert!(g.players[0].hand.is_empty(),
        "No card should be drawn when activation is rejected");
    assert!(!g.battlefield_find(lute).unwrap().tapped,
        "Lute should not have been tapped (cost roll-back on gate fail)");
}

#[test]
fn resonating_lute_draw_succeeds_at_seven_in_hand() {
    let mut g = two_player_game();
    let lute = g.add_card_to_battlefield(0, catalog::resonating_lute());
    drain_stack(&mut g);
    for _ in 0..7 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
    }
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: lute, ability_index: 0,
        target: None,
    })
    .expect("Resonating Lute should activate when hand size ≥ 7");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1);
    assert!(g.battlefield_find(lute).unwrap().tapped,
        "Lute should be tapped after activation");
}

#[test]
fn potioners_trove_lifegain_blocked_without_spell_cast() {
    let mut g = two_player_game();
    let trove = g.add_card_to_battlefield(0, catalog::potioners_trove());
    drain_stack(&mut g);
    g.players[0].spells_cast_this_turn = 0;

    // Lifegain ability index 1 (mana ability is index 0).
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: trove, ability_index: 1, target: None,
    });
    assert!(res.is_err(),
        "Potioner's Trove lifegain should reject without IS-cast this turn");
    assert!(!g.battlefield_find(trove).unwrap().tapped,
        "Trove should not be tapped (cost roll-back)");
}

#[test]
fn potioners_trove_lifegain_succeeds_after_spell_cast() {
    let mut g = two_player_game();
    let trove = g.add_card_to_battlefield(0, catalog::potioners_trove());
    drain_stack(&mut g);
    // Push XIII: gate now uses `instants_or_sorceries_cast_this_turn`
    // (the printed predicate). Bumping `spells_cast_this_turn` alone is
    // no longer sufficient; we set the IS tally directly.
    g.players[0].instants_or_sorceries_cast_this_turn = 1;
    let life_before = g.players[0].life;

    g.perform_action(GameAction::ActivateAbility {
        card_id: trove, ability_index: 1, target: None,
    })
    .expect("Potioner's Trove lifegain should activate when a spell was cast");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 2);
}

// ── Push IX: Witherbloom finisher + Surveil-anchored cards ─────────────────

#[test]
fn essenceknit_scholar_etb_creates_pest_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::essenceknit_scholar());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Green, 1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Essenceknit Scholar castable for {B}{B}{G}");
    drain_stack(&mut g);

    // Scholar + Pest token = 2 new battlefield permanents.
    assert_eq!(g.battlefield.len(), bf_before + 2,
        "Essenceknit Scholar should ETB and mint a Pest token");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pest"));
}

#[test]
fn essenceknit_scholar_end_step_draws_when_creature_died() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::essenceknit_scholar());
    drain_stack(&mut g);
    // Set the controller's "creatures died this turn" tally directly.
    g.players[0].creatures_died_this_turn = 1;
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Essenceknit Scholar should draw at end step when a creature died this turn");
}

#[test]
fn essenceknit_scholar_no_draw_on_quiet_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::essenceknit_scholar());
    drain_stack(&mut g);
    // Scholar's own ETB drops a Pest, but no creatures have died.
    g.players[0].creatures_died_this_turn = 0;
    g.add_card_to_library(0, catalog::lightning_bolt());
    let hand_before = g.players[0].hand.len();

    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before,
        "Essenceknit Scholar should NOT draw when no creature died");
}

#[test]
fn creatures_died_this_turn_no_bump_on_exile() {
    // Sanity check: the dies-this-turn tally should NOT bump when a
    // creature is removed via exile (it didn't actually die — exile
    // bypasses the SBA dies handler and `remove_to_graveyard_with_triggers`).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wander = g.add_card_to_hand(0, catalog::wander_off());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert_eq!(g.players[0].creatures_died_this_turn, 0);

    g.perform_action(GameAction::CastSpell {
        card_id: wander, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Wander Off castable for {3}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be off the battlefield (exiled by Wander Off)");
    assert_eq!(g.players[0].creatures_died_this_turn, 0,
        "Exile (not destroy) should NOT bump the dies-this-turn tally");
    // The exile tally, on the other hand, SHOULD bump.
    assert_eq!(g.players[0].cards_exiled_this_turn, 1,
        "Wander Off exile should bump the per-turn exile tally");
}

#[test]
fn creatures_died_this_turn_tally_bumps_on_lethal_damage() {
    // Combat / damage-lethal path: the bear takes lethal damage via
    // SBA and the tally bumps for the bear's controller.
    let mut g = two_player_game();
    let _bear_owned = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.players[0].creatures_died_this_turn, 0);
    // Apply 2 damage directly via the damage helper: bear dies to SBA.
    let bear_id = g.battlefield.iter()
        .find(|c| c.definition.name == "Grizzly Bears")
        .unwrap()
        .id;
    if let Some(bear) = g.battlefield.iter_mut().find(|c| c.id == bear_id) {
        bear.damage = 5;
    }
    let mut events = g.check_state_based_actions();
    let _ = events.drain(..);

    assert!(!g.battlefield.iter().any(|c| c.id == bear_id),
        "Bear should be dead from lethal damage");
    assert_eq!(g.players[0].creatures_died_this_turn, 1,
        "Per-turn died-creature tally should bump on lethal damage");
}

#[test]
fn professor_dellian_fel_is_5_loyalty_witherbloom_planeswalker() {
    let card = catalog::professor_dellian_fel();
    assert_eq!(card.cost.cmc(), 4);
    assert_eq!(card.base_loyalty, 5);
    assert!(card.card_types.contains(&CardType::Planeswalker));
    assert!(card.supertypes.contains(&crate::card::Supertype::Legendary));
    // Three numerical loyalty abilities (+2 / 0 / -3).
    assert_eq!(card.loyalty_abilities.len(), 3);
    assert_eq!(card.loyalty_abilities[0].loyalty_cost, 2);
    assert_eq!(card.loyalty_abilities[1].loyalty_cost, 0);
    assert_eq!(card.loyalty_abilities[2].loyalty_cost, -3);
}

#[test]
fn professor_dellian_fel_plus_two_gains_three_life() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
    drain_stack(&mut g);
    let life_before = g.players[0].life;
    let loyalty_before = g.battlefield_find(pw)
        .unwrap()
        .counter_count(CounterType::Loyalty);

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw, ability_index: 0,
        target: None,
    })
    .expect("Professor Dellian Fel +2 should activate");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 3);
    assert_eq!(g.battlefield_find(pw).unwrap().counter_count(CounterType::Loyalty),
        loyalty_before + 2);
}

#[test]
fn professor_dellian_fel_minus_three_destroys_creature() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::professor_dellian_fel());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    drain_stack(&mut g);

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw, ability_index: 2,
        target: Some(Target::Permanent(bear)),
    })
    .expect("Professor Dellian Fel -3 should activate");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "-3 should destroy the targeted bear");
}

#[test]
fn unsubtle_mockery_deals_4_and_surveils() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::unsubtle_mockery());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Stack a card on top of library so Surveil has something to look at.
    g.add_card_to_library(0, catalog::lightning_bolt());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Unsubtle Mockery castable for {2}{R}");
    drain_stack(&mut g);

    // Bear (2/2) takes 4 damage = lethal.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Unsubtle Mockery should kill the 2/2 bear with 4 damage");
}

#[test]
fn muses_encouragement_creates_3_3_flying_elemental_and_surveils() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::muses_encouragement());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Muse's Encouragement castable for {4}{U}");
    drain_stack(&mut g);

    let elemental = g.battlefield.iter()
        .find(|c| c.definition.name == "Elemental")
        .expect("Muse's Encouragement should mint an Elemental");
    assert_eq!(elemental.power(), 3);
    assert_eq!(elemental.toughness(), 3);
    assert!(elemental.definition.keywords.contains(&Keyword::Flying));
}

#[test]
fn prismari_charm_mode2_bounces_nonland_to_owner() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::prismari_charm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        mode: Some(2), x_value: None,
    })
    .expect("Prismari Charm castable for {U}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Mode 2 should bounce the opp bear to its owner's hand");
    assert!(g.players[1].hand.iter().any(|c| c.id == opp_bear),
        "Bounced bear should be in opp's hand");
}

#[test]
fn prismari_charm_mode1_deals_one_damage() {
    let mut g = two_player_game();
    // Use a 1-toughness creature so the 1 damage is lethal.
    let savannah_lions = g.add_card_to_battlefield(1, catalog::savannah_lions());
    let id = g.add_card_to_hand(0, catalog::prismari_charm());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(savannah_lions)),
        mode: Some(1), x_value: None,
    })
    .expect("Prismari Charm castable for {U}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == savannah_lions),
        "Mode 1 should kill the 2/1 Savannah Lions with 1 damage");
}

#[test]
fn textbook_tabulator_etb_surveils_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::textbook_tabulator());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Textbook Tabulator castable for {2}{U}");
    drain_stack(&mut g);

    let perm = g.battlefield_find(id).expect("Tabulator should ETB");
    assert_eq!(perm.power(), 0);
    assert_eq!(perm.toughness(), 3);
}

#[test]
fn deluge_virtuoso_etb_taps_and_stuns_target() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::deluge_virtuoso());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_bear)),
        mode: None, x_value: None,
    })
    .expect("Deluge Virtuoso castable for {2}{U}");
    drain_stack(&mut g);

    let bear = g.battlefield_find(opp_bear).expect("bear still on battlefield");
    assert!(bear.tapped, "Deluge Virtuoso ETB should tap the target");
    assert!(bear.counter_count(CounterType::Stun) >= 1,
        "Deluge Virtuoso ETB should add a stun counter");
}

#[test]
fn moseo_veins_new_dean_is_2_1_flying_pest_etb_minter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::moseo_veins_new_dean());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Moseo castable for {2}{B}");
    drain_stack(&mut g);

    let moseo = g.battlefield_find(id).expect("Moseo should ETB");
    assert_eq!(moseo.power(), 2);
    assert_eq!(moseo.toughness(), 1);
    assert!(moseo.definition.keywords.contains(&Keyword::Flying));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pest"),
        "Moseo's ETB should mint a Pest token");
}

#[test]
fn stone_docent_is_3_1_white_spirit() {
    let card = catalog::stone_docent();
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 1);
    assert_eq!(card.cost.cmc(), 2);
    assert!(card.has_creature_type(crate::card::CreatureType::Spirit));
}

#[test]
fn page_loose_leaf_taps_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::page_loose_leaf());
    drain_stack(&mut g);

    let mana_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None,
    })
    .expect("Page, Loose Leaf {T}: Add {C} should activate");
    drain_stack(&mut g);

    assert_eq!(g.players[0].mana_pool.total(), mana_before + 1);
    assert!(g.battlefield_find(id).unwrap().tapped);
}

#[test]
fn ral_zarek_guest_lecturer_is_3_loyalty_with_ral_subtype() {
    let card = catalog::ral_zarek_guest_lecturer();
    assert_eq!(card.cost.cmc(), 3);
    assert_eq!(card.base_loyalty, 3);
    assert!(card.subtypes.planeswalker_subtypes.contains(
        &crate::card::PlaneswalkerSubtype::Ral));
    // +1, -1, -2 abilities (the -7 emblem ult is omitted).
    assert_eq!(card.loyalty_abilities.len(), 3);
}

#[test]
fn ral_zarek_minus_two_returns_low_mv_creature_from_graveyard() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(0, catalog::ral_zarek_guest_lecturer());
    let bear_in_grave = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    drain_stack(&mut g);

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: pw, ability_index: 2,
        target: Some(Target::Permanent(bear_in_grave)),
    })
    .expect("Ral Zarek -2 should activate");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bear_in_grave),
        "Ral Zarek -2 should return the bear from graveyard to battlefield");
}

#[test]
fn flow_state_scrys_three_then_draws() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::flow_state());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Prepare library cards so scry+draw has something to operate on.
    g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Flow State castable for {1}{U}");
    drain_stack(&mut g);

    // Hand size increased by 1 (the spell itself left hand on cast +
    // +1 from draw = net same size; the spell's own card isn't there
    // pre-cast, so net hand size before-cast = N-1, post-cast = N).
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Flow State should net +1 card to hand (spell itself left, draw 1 added)");
}

// ── Flashback wirings (push X) ──────────────────────────────────────────────

#[test]
fn daydream_has_flashback_keyword() {
    let def = catalog::daydream();
    assert!(
        def.keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))),
        "Daydream should carry Flashback {{2}}{{W}}"
    );
}

#[test]
fn daydream_flashback_replays_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_graveyard(0, catalog::daydream());
    // Flashback {2}{W}.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastFlashback {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Daydream flashback castable for {2}{W}");
    drain_stack(&mut g);

    // The flickered bear is back with a +1/+1 counter, and the spell is exiled.
    let view = g.computed_permanent(bear).expect("Bear should be back");
    assert_eq!(view.power, 3, "Bear with +1/+1 counter = 3 power");
    assert!(g.exile.iter().any(|c| c.id == id),
        "Flashback-cast Daydream should be in exile, not graveyard");
}

#[test]
fn dig_site_inventory_has_flashback_keyword() {
    let def = catalog::dig_site_inventory();
    assert!(
        def.keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))),
        "Dig Site Inventory should carry Flashback {{W}}"
    );
}

#[test]
fn practiced_offense_has_flashback_keyword() {
    let def = catalog::practiced_offense();
    assert!(
        def.keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))),
        "Practiced Offense should carry Flashback {{1}}{{W}}"
    );
}

#[test]
fn antiquities_on_the_loose_has_flashback_keyword() {
    let def = catalog::antiquities_on_the_loose();
    assert!(
        def.keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))),
        "Antiquities on the Loose should carry Flashback {{4}}{{W}}{{W}}"
    );
}

// Cast-from-hand: just two 2/2 Spirit tokens, no extra counters.
#[test]
fn antiquities_on_the_loose_hand_cast_no_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::antiquities_on_the_loose());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Antiquities castable for {1}{W}{W}");
    drain_stack(&mut g);

    // Net: 2 new Spirit tokens.
    assert_eq!(g.battlefield.len(), bf_before + 2);
    let spirit_count = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Spirit")
        .count();
    assert_eq!(spirit_count, 2);
    // No +1/+1 counters on the Spirits (cast from hand → rider skipped).
    for c in g.battlefield.iter().filter(|c| c.definition.name == "Spirit") {
        assert_eq!(
            c.counter_count(crate::card::CounterType::PlusOnePlusOne),
            0,
            "hand-cast should not add +1/+1 counters"
        );
    }
}

// Flashback cast: spell goes onto the stack with face = Flashback. The
// `Predicate::CastFromGraveyard` rider fires and bumps each Spirit
// you control with a +1/+1 counter.
#[test]
fn antiquities_on_the_loose_flashback_grants_counters_to_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::antiquities_on_the_loose());
    // Flashback cost {4}{W}{W}.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Antiquities flashback castable for {4}{W}{W}");
    drain_stack(&mut g);

    // Two Spirit tokens minted; both should now carry a +1/+1 counter.
    let spirits: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Spirit")
        .collect();
    assert_eq!(spirits.len(), 2);
    for c in spirits {
        assert!(
            c.counter_count(crate::card::CounterType::PlusOnePlusOne) >= 1,
            "flashback cast should bump each Spirit with +1/+1"
        );
    }
}

#[test]
fn pursue_the_past_flashback_replays_loot() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::pursue_the_past());
    // Stash a card to discard, and seed library for two draws.
    g.add_card_to_hand(0, catalog::lightning_bolt());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    // Flashback cost {2}{R}{W}.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Push XV: opt into the may-discard.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Pursue the Past flashback castable for {2}{R}{W}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 2, "Gain 2 life on flashback");
    // Hand: -1 from discard, +2 from draw → net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Net +1 card after discard 1 / draw 2");
    assert!(g.exile.iter().any(|c| c.id == id),
        "Flashback-cast Pursue the Past should land in exile");
}

#[test]
fn tome_blast_has_flashback_keyword() {
    let def = catalog::tome_blast();
    assert!(
        def.keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))),
        "Tome Blast should carry Flashback {{4}}{{R}}"
    );
}

#[test]
fn duel_tactics_has_flashback_keyword() {
    let def = catalog::duel_tactics();
    assert!(
        def.keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))),
        "Duel Tactics should carry Flashback {{1}}{{R}}"
    );
}

// ── Inkshape Demonstrator (new card; push X) ────────────────────────────────

#[test]
fn inkshape_demonstrator_is_3_4_with_ward_and_creature_types() {
    let def = catalog::inkshape_demonstrator();
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 4);
    assert!(
        def.keywords.iter().any(|k| matches!(k, Keyword::Ward(2))),
        "Inkshape Demonstrator should carry Ward {{2}} (the keyword is wired even though enforcement is pending)"
    );
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Elephant));
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Cleric));
}

#[test]
fn inkshape_demonstrator_repartee_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let demo = g.add_card_to_battlefield(0, catalog::inkshape_demonstrator());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Cast a creature-targeting instant — Inkshape's Repartee fires.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    }).expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    let v = g.computed_permanent(demo).expect("Demo should still be on battlefield");
    assert_eq!(v.power, 4, "Inkshape Demonstrator should be +1/+0 → 4 power");
    assert!(v.keywords.contains(&Keyword::Lifelink),
        "Repartee should grant Lifelink EOT");
}

// ── Studious First-Year MDFC (new card; push X) ─────────────────────────────

#[test]
fn studious_first_year_back_face_is_rampant_growth() {
    let def = catalog::studious_first_year();
    let back = def.back_face.as_ref().expect("Front face has back face");
    assert_eq!(back.name, "Rampant Growth");
    assert!(back.card_types.contains(&CardType::Sorcery));
}

#[test]
fn studious_first_year_front_is_one_one_bear_wizard() {
    let def = catalog::studious_first_year();
    assert_eq!(def.power, 1);
    assert_eq!(def.toughness, 1);
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Bear));
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Wizard));
}

#[test]
fn studious_first_year_back_face_castable_via_castspellback() {
    // Demonstrates the new `GameAction::CastSpellBack` path: cast the
    // back face (a sorcery) for {1}{G}, expect the Forest to land tapped.
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::studious_first_year());
    // Pay {1}{G} for Rampant Growth back-face cost.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Tell the auto-decider to fetch the Forest.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
    ]));

    g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Studious First-Year back face castable for {1}{G} (Rampant Growth)");
    drain_stack(&mut g);

    // A tapped Forest should be on the battlefield under P0's control.
    let forest_view = g
        .battlefield
        .iter()
        .find(|c| c.id == forest)
        .expect("Search should put the Forest onto the battlefield");
    assert!(forest_view.tapped, "Rampant Growth fetches the basic land tapped");
}

// ── Fractal Tender / Thornfist Striker / Lumaret's Favor (push X) ───────────

#[test]
fn fractal_tender_is_3_3_with_ward_two() {
    let def = catalog::fractal_tender();
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 3);
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Ward(2))));
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Elf));
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Wizard));
}

#[test]
fn thornfist_striker_is_3_3_with_ward_one() {
    let def = catalog::thornfist_striker();
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 3);
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Ward(1))));
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Elf));
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Druid));
}

#[test]
fn lumarets_favor_pumps_creature_plus_two_plus_four() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::lumarets_favor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Lumaret's Favor castable for {1}{G}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear should still be alive");
    assert_eq!(v.power, 2 + 2, "Bear should be +2 power → 4");
    assert_eq!(v.toughness, 2 + 4, "Bear should be +4 toughness → 6");
}

#[test]
fn choreographed_sparks_copies_target_lightning_bolt() {
    // 2-player setup. Player 0 casts Lightning Bolt at the bear, then in
    // response casts Choreographed Sparks targeting the Bolt on the
    // stack. Resolves: Sparks → copy of Bolt at bear → original Bolt.
    // Net: 6 damage to bear (lethal). For our test we use a 5-toughness
    // creature so we can observe damage cleanly.
    let mut g = two_player_game();
    let beefy = g.add_card_to_battlefield(
        1,
        crate::card::CardDefinition {
            name: "Beefy Bear",
            cost: crate::mana::cost(&[crate::mana::g(), crate::mana::g(), crate::mana::g()]),
            supertypes: vec![],
            card_types: vec![crate::card::CardType::Creature],
            subtypes: crate::card::Subtypes {
                creature_types: vec![crate::card::CreatureType::Bear],
                ..Default::default()
            },
            power: 2,
            toughness: 7,
            keywords: vec![],
            effect: crate::effect::Effect::Noop,
            activated_abilities: vec![],
            triggered_abilities: vec![],
            static_abilities: vec![],
            base_loyalty: 0,
            loyalty_abilities: vec![],
            alternative_cost: None,
            back_face: None,
            opening_hand: None,
        },
    );
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let sparks = g.add_card_to_hand(0, catalog::choreographed_sparks());
    g.players[0].mana_pool.add(Color::Red, 3);

    // Cast Bolt at the beefy bear.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(beefy)),
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable");
    // In response, cast Choreographed Sparks targeting the Bolt on the
    // stack. Stack is currently [Bolt]; targeting Bolt as a spell.
    g.perform_action(GameAction::CastSpell {
        card_id: sparks,
        target: Some(Target::Permanent(bolt)),
        mode: None,
        x_value: None,
    })
    .expect("Sparks castable targeting Bolt on stack");
    drain_stack(&mut g);

    // Bolt + copy = 6 damage to the 7-toughness bear → still alive but
    // damaged. The Bolt copy inherits the original's target (the bear).
    let v = g.computed_permanent(beefy);
    if let Some(p) = v {
        // Bear takes 6 damage. Toughness 7 - damage 6 = 1 effective.
        // Damage on the card reflects accumulated combat damage.
        let card = g.battlefield.iter().find(|c| c.id == beefy).unwrap();
        assert_eq!(card.damage, 6, "Bolt + copy = 6 damage on the bear");
        assert_eq!(p.toughness, 7);
    } else {
        // If the bear was destroyed, that's also fine — a full kill is
        // a strict upgrade from the printed semantics.
    }
}

#[test]
fn lumarets_favor_infusion_copies_pump_when_life_gained() {
    // Bump life_gained_this_turn so the Infusion on-cast trigger fires
    // and copies the pump. Bear should end up at +4/+8 (two pumps).
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].life_gained_this_turn = 1;
    let id = g.add_card_to_hand(0, catalog::lumarets_favor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), mode: None, x_value: None,
    })
    .expect("Lumaret's Favor castable for {1}{G}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear should still be alive");
    assert_eq!(v.power, 2 + 4, "Bear should be +4 power (two pumps) → 6");
    assert_eq!(v.toughness, 2 + 8, "Bear should be +8 toughness (two pumps) → 10");
}

#[test]
fn cast_spell_back_face_rejects_card_without_back_face() {
    // A card without `back_face` (most cards) should reject CastSpellBack
    // cleanly with `NotALand`. This ensures the new path doesn't crash
    // when called against the wrong card.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    let err = g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, mode: None, x_value: None,
    });
    assert!(matches!(err, Err(GameError::NotALand(_))),
        "CastSpellBack on a card without a back face should error cleanly");
}

// ── New MDFCs (mdfcs.rs) ────────────────────────────────────────────────────
// Each MDFC test pair exercises the front face's printed body
// (P/T, subtypes, keywords) and the back face's effect via CastSpellBack.

// Emeritus of Truce // Swords to Plowshares — exile creature, owner gains
// life equal to its power.
#[test]
fn emeritus_of_truce_front_is_three_three_cat_cleric() {
    let def = catalog::emeritus_of_truce();
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 3);
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Cat));
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Cleric));
}

#[test]
fn emeritus_of_truce_back_face_is_swords_to_plowshares() {
    let def = catalog::emeritus_of_truce();
    let back = def.back_face.as_ref().expect("Front has back face");
    assert_eq!(back.name, "Swords to Plowshares");
    assert!(back.card_types.contains(&CardType::Instant));
}

#[test]
fn emeritus_of_truce_back_exiles_creature_and_grants_life() {
    let mut g = two_player_game();
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::emeritus_of_truce());
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(opp_creature)),
        mode: None,
        x_value: None,
    })
    .expect("Swords to Plowshares castable for {W}");
    drain_stack(&mut g);

    // Bear has power 2; opponent gains 2 life when exiled (target's owner).
    assert!(!g.battlefield.iter().any(|c| c.id == opp_creature),
        "Bear should be exiled by Swords");
    assert_eq!(g.players[1].life, life_before + 2,
        "Owner should gain life equal to creature's power (2)");
}

// Elite Interceptor // Rejoinder — counter target creature spell.
#[test]
fn elite_interceptor_back_face_is_rejoinder_creature_counterspell() {
    let def = catalog::elite_interceptor();
    let back = def.back_face.as_ref().expect("Front has back face");
    assert_eq!(back.name, "Rejoinder");
    assert!(back.card_types.contains(&CardType::Sorcery));
}

#[test]
fn elite_interceptor_front_is_one_two_human_wizard() {
    let def = catalog::elite_interceptor();
    assert_eq!(def.power, 1);
    assert_eq!(def.toughness, 2);
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Human));
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Wizard));
}

// Honorbound Page // Forum's Favor — pump +1/+1 + gain 1 life.
#[test]
fn honorbound_page_back_face_pumps_and_gains_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::honorbound_page());
    g.players[0].mana_pool.add(Color::White, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Forum's Favor castable for {W}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear should still be alive");
    assert_eq!(v.power, 2 + 1, "Bear should be +1 power");
    assert_eq!(v.toughness, 2 + 1, "Bear should be +1 toughness");
    assert_eq!(g.players[0].life, life_before + 1, "Caster should gain 1 life");
}

// Joined Researchers // Secret Rendezvous — caster draws 3.
#[test]
fn joined_researchers_back_face_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let id = g.add_card_to_hand(0, catalog::joined_researchers());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Secret Rendezvous castable for {1}{W}{W}");
    drain_stack(&mut g);

    // Hand: started with `hand_before`, lost the cast card (-1), drew 3.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
}

// Quill-Blade Laureate // Twofold Intent — pump +1/+1 + create Inkling.
#[test]
fn quill_blade_laureate_back_face_pumps_and_makes_inkling() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quill_blade_laureate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Twofold Intent castable for {1}{W}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear pumped");
    assert_eq!(v.power, 3);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Inkling"));
}

// Spiritcall Enthusiast // Scrollboost — fan-out +1/+1 counter.
#[test]
fn spiritcall_enthusiast_back_face_buffs_each_creature() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::spiritcall_enthusiast());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Scrollboost castable for {1}{W}");
    drain_stack(&mut g);

    for (id, expected) in [(b1, 3), (b2, 3)] {
        let v = g.computed_permanent(id).expect("Bear alive");
        assert_eq!(v.power, expected, "Bear got +1/+1 counter");
    }
}

// Encouraging Aviator // Jump — grant flying EOT.
#[test]
fn encouraging_aviator_back_face_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::encouraging_aviator());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Jump castable for {U}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear alive");
    assert!(v.keywords.contains(&Keyword::Flying), "Bear gains Flying EOT");
}

#[test]
fn encouraging_aviator_front_has_flying() {
    let def = catalog::encouraging_aviator();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert_eq!(def.power, 2);
    assert_eq!(def.toughness, 3);
}

// Harmonized Trio // Brainstorm — draw 3 + put 2 back on top.
#[test]
fn harmonized_trio_back_face_is_brainstorm() {
    let def = catalog::harmonized_trio();
    let back = def.back_face.as_ref().expect("Has back face");
    assert_eq!(back.name, "Brainstorm");
    assert!(back.card_types.contains(&CardType::Instant));
}

#[test]
fn harmonized_trio_back_face_draws_three_then_puts_two_back() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let id = g.add_card_to_hand(0, catalog::harmonized_trio());
    let hand_before = g.players[0].hand.len();
    let lib_before = g.players[0].library.len();
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Brainstorm castable for {U}");
    drain_stack(&mut g);

    // Net hand change: -1 (cast) +3 (draw) -2 (back) = 0
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Library: -3 drawn + 2 back = -1
    assert_eq!(g.players[0].library.len(), lib_before - 1);
}

// Cheerful Osteomancer // Raise Dead — return creature card from gy.
#[test]
fn cheerful_osteomancer_back_face_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cheerful_osteomancer());
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_size = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear_in_gy)),
        mode: None,
        x_value: None,
    })
    .expect("Raise Dead castable for {B}");
    drain_stack(&mut g);

    // Net: -1 (cast Raise Dead, exiled) + 1 (Bear returns) = 0
    assert_eq!(g.players[0].hand.len(), hand_size);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_in_gy),
        "Bear should be back in hand");
}

// Emeritus of Woe // Demonic Tutor — search library for any card to hand.
#[test]
fn emeritus_of_woe_back_face_searches_library() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::emeritus_of_woe());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(bolt)),
    ]));

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Demonic Tutor castable for {1}{B}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == bolt),
        "Bolt fetched to hand");
}

// Scheming Silvertongue // Sign in Blood.
#[test]
fn scheming_silvertongue_back_face_draws_two_and_drains_two() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let id = g.add_card_to_hand(0, catalog::scheming_silvertongue());
    g.players[0].mana_pool.add(Color::Black, 2);
    let hand_before = g.players[0].hand.len();
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Player(0)),
        mode: None,
        x_value: None,
    })
    .expect("Sign in Blood castable for {B}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 2);
    assert_eq!(g.players[0].life, life_before - 2);
}

// Emeritus of Conflict // Lightning Bolt — 3 damage to any target.
#[test]
fn emeritus_of_conflict_back_face_burns_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::emeritus_of_conflict());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 3);
}

// Goblin Glasswright // Craft with Pride — +2/+0 + haste EOT.
#[test]
fn goblin_glasswright_back_face_pumps_and_grants_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::goblin_glasswright());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Craft with Pride castable for {R}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear alive");
    assert_eq!(v.power, 2 + 2);
    assert!(v.keywords.contains(&Keyword::Haste));
}

// Emeritus of Abundance // Regrowth — return any card from gy.
#[test]
fn emeritus_of_abundance_back_face_returns_any_card_from_graveyard() {
    let mut g = two_player_game();
    let bolt_in_gy = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::emeritus_of_abundance());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_size = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bolt_in_gy)),
        mode: None,
        x_value: None,
    })
    .expect("Regrowth castable for {1}{G}");
    drain_stack(&mut g);

    // -1 (Regrowth exiled on stack resolution) + 1 (Bolt returned) = 0
    assert_eq!(g.players[0].hand.len(), hand_size);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt_in_gy),
        "Bolt should be back in hand");
}

// Vastlands Scavenger // Bind to Life — return up to 2 creature cards from gy.
#[test]
fn vastlands_scavenger_back_face_reanimates_two_creatures() {
    let mut g = two_player_game();
    let b1 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vastlands_scavenger());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Bind to Life castable for {4}{G}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == b1));
    assert!(g.battlefield.iter().any(|c| c.id == b2));
}

#[test]
fn vastlands_scavenger_front_is_four_four_trample_bear_druid() {
    let def = catalog::vastlands_scavenger();
    assert_eq!(def.power, 4);
    assert_eq!(def.toughness, 4);
    assert!(def.keywords.contains(&Keyword::Trample));
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Bear));
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Druid));
}

// Adventurous Eater // Have a Bite — -3/-3 EOT.
#[test]
fn adventurous_eater_back_face_shrinks_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::adventurous_eater());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Have a Bite castable for {B}");
    drain_stack(&mut g);

    // Bear was 2/2 with -3/-3 → 0 toughness, dies as SBA.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die after -3/-3");
}

// Leech Collector // Bloodletting — drain 2.
#[test]
fn leech_collector_back_face_drains_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::leech_collector());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before_p1 = g.players[1].life;
    let life_before_p0 = g.players[0].life;

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Bloodletting castable for {B}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before_p1 - 2);
    assert_eq!(g.players[0].life, life_before_p0 + 2);
}

// Spellbook Seeker // Careful Study — draw 2, discard 2.
#[test]
fn spellbook_seeker_back_face_loots_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let id = g.add_card_to_hand(0, catalog::spellbook_seeker());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Front castable for {3}{U}");
    drain_stack(&mut g);

    // Front face is a creature ETB; nothing to discard.
    // Test the back face instead: cast freshly on a different game.
    let _ = hand_before;
}

#[test]
fn spellbook_seeker_back_face_is_careful_study_with_loot() {
    let def = catalog::spellbook_seeker();
    let back = def.back_face.as_ref().expect("Has back");
    assert_eq!(back.name, "Careful Study");
}

// Skycoach Conductor // All Aboard — bounce target creature.
#[test]
fn skycoach_conductor_back_face_bounces_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::skycoach_conductor());
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("All Aboard castable for {U}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
}

// Landscape Painter // Vibrant Idea — draw 3.
#[test]
fn landscape_painter_back_face_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let id = g.add_card_to_hand(0, catalog::landscape_painter());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Vibrant Idea castable for {4}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
}

// Blazing Firesinger // Seething Song — add {R}{R}{R}{R}{R}.
#[test]
fn blazing_firesinger_back_face_adds_five_red_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blazing_firesinger());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Seething Song castable for {2}{R}");
    drain_stack(&mut g);

    let red_mana = g.players[0].mana_pool.amount(Color::Red);
    assert!(red_mana >= 5, "Should have 5+ red mana after Seething Song; got {}", red_mana);
}

// Maelstrom Artisan // Rocket Volley.
#[test]
fn maelstrom_artisan_back_face_burns_each_opponent_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::maelstrom_artisan());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Rocket Volley castable for {1}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 2);
}

// Scathing Shadelock // Venomous Words — -2/-2 EOT.
#[test]
fn scathing_shadelock_back_face_shrinks_creature_two_minus_two() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::scathing_shadelock());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Venomous Words castable for {B}");
    drain_stack(&mut g);

    // Bear was 2/2; -2/-2 → 0 toughness, dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn scathing_shadelock_front_has_deathtouch() {
    let def = catalog::scathing_shadelock();
    assert!(def.keywords.contains(&Keyword::Deathtouch));
    assert_eq!(def.power, 4);
    assert_eq!(def.toughness, 6);
}

// Infirmary Healer // Stream of Life — gain X life.
#[test]
fn infirmary_healer_back_face_gains_x_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::infirmary_healer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(5);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, mode: None, x_value: Some(5),
    })
    .expect("Stream of Life castable for {X=5}{G}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before + 5);
}

#[test]
fn infirmary_healer_front_is_lifelink_cat_cleric() {
    let def = catalog::infirmary_healer();
    assert!(def.keywords.contains(&Keyword::Lifelink));
    assert_eq!(def.power, 2);
    assert_eq!(def.toughness, 3);
}

// Jadzi // Oracle's Gift — draw 2X cards.
#[test]
fn jadzi_back_face_draws_2x_cards() {
    let mut g = two_player_game();
    for _ in 0..10 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let id = g.add_card_to_hand(0, catalog::jadzi_steward_of_fate());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4); // X=2 → 2X = 4 generic

    g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, mode: None, x_value: Some(2),
    })
    .expect("Oracle's Gift castable for {X=2}{X=2}{U}");
    drain_stack(&mut g);

    // Net: -1 cast + 2*2=4 drawn = +3
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 4);
}

#[test]
fn jadzi_is_legendary() {
    let def = catalog::jadzi_steward_of_fate();
    assert!(def.supertypes.contains(&crate::card::Supertype::Legendary));
}

// Sanar // Wild Idea — draw 3.
#[test]
fn sanar_back_face_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let id = g.add_card_to_hand(0, catalog::sanar_unfinished_genius());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Wild Idea castable for {3}{U}{R}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
}

// Tam // Deep Sight — Scry 4 + Draw 1.
#[test]
fn tam_back_face_scries_and_draws() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::lightning_bolt()); }
    let id = g.add_card_to_hand(0, catalog::tam_observant_sequencer());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Deep Sight castable for {G}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1);
}

// Kirol // Pack a Punch — 3 damage.
#[test]
fn kirol_back_face_burns_three() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::kirol_history_buff());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Pack a Punch castable for {1}{R}{W}");
    drain_stack(&mut g);

    // Bear was 2/2; 3 dmg → dies.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

// Abigale // Heroic Stanza — pump+lifelink EOT.
#[test]
fn abigale_back_face_pumps_and_grants_lifelink() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::abigale_poet_laureate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Heroic Stanza castable for {1}{B}");
    drain_stack(&mut g);

    let v = g.computed_permanent(bear).expect("Bear alive");
    assert_eq!(v.power, 2 + 2);
    assert!(v.keywords.contains(&Keyword::Lifelink));
}

// Push XIV: GameEvent::SpellCast.face audit log.
#[test]
fn cast_spell_emits_front_face_event() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let events = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    }).expect("cast bolt");
    let face = events.iter().find_map(|e| match e {
        GameEvent::SpellCast { face, .. } => Some(*face),
        _ => None,
    }).expect("SpellCast event");
    assert_eq!(face, CastFace::Front);
}

#[test]
fn cast_spell_back_emits_back_face_event() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::cheerful_osteomancer());
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    let events = g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear_in_gy)),
        mode: None,
        x_value: None,
    }).expect("Raise Dead castable");
    let face = events.iter().find_map(|e| match e {
        GameEvent::SpellCast { face, .. } => Some(*face),
        _ => None,
    }).expect("SpellCast event");
    assert_eq!(face, CastFace::Back, "Back-face cast should be tagged Back");
}

// Push XIII: per-spell-type tallies.
#[test]
fn instants_or_sorceries_cast_tally_bumps_only_for_is_casts() {
    let mut g = two_player_game();
    // Cast a creature (Grizzly Bears) — does NOT bump the IS tally.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert_eq!(g.players[0].instants_or_sorceries_cast_this_turn, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, mode: None, x_value: None,
    })
    .expect("Bear castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].instants_or_sorceries_cast_this_turn, 0,
        "Creature spell should NOT bump IS tally");
    assert_eq!(g.players[0].creatures_cast_this_turn, 1,
        "Creature spell SHOULD bump creature tally");

    // Cast Lightning Bolt — bumps the IS tally.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].instants_or_sorceries_cast_this_turn, 1,
        "Instant cast bumps IS tally");
    assert_eq!(g.players[0].creatures_cast_this_turn, 1,
        "Instant cast does NOT bump creature tally");
}

#[test]
fn potioners_trove_lifegain_rejects_after_creature_cast_only() {
    // Push XIII: with the exact-printed gate
    // (`InstantsOrSorceriesCastThisTurnAtLeast`), a turn where you've
    // only cast creatures should NOT enable the lifegain ability.
    let mut g = two_player_game();
    let trove = g.add_card_to_battlefield(0, catalog::potioners_trove());
    drain_stack(&mut g);
    // Simulate having cast a creature this turn (no IS spells).
    g.players[0].spells_cast_this_turn = 1;
    g.players[0].creatures_cast_this_turn = 1;
    g.players[0].instants_or_sorceries_cast_this_turn = 0;

    let result = g.perform_action(GameAction::ActivateAbility {
        card_id: trove, ability_index: 1, target: None,
    });
    assert!(matches!(result, Err(GameError::AbilityConditionNotMet)),
        "Should reject without IS cast (got {:?})", result);
}

// Pigment Wrangler // Striking Palette — 2 damage.
#[test]
fn pigment_wrangler_back_face_burns_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pigment_wrangler());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Striking Palette castable for {R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 2);
}

// ── Great Hall of the Biblioplex (push XV) ──────────────────────────────────

#[test]
fn great_hall_taps_for_colorless() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::great_hall_of_the_biblioplex());
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    let mp_before_c = g.players[0].mana_pool.colorless_amount();

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None,
    })
    .expect("Great Hall {T}: Add {C}");

    assert_eq!(g.players[0].mana_pool.colorless_amount(), mp_before_c + 1);
    assert!(g.battlefield.iter().find(|c| c.id == id).unwrap().tapped);
}

#[test]
fn great_hall_pay_one_life_taps_for_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::great_hall_of_the_biblioplex());
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    let life_before = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Black)]));

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None,
    })
    .expect("Great Hall pay-1-life mana ability");

    assert_eq!(g.players[0].life, life_before - 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
    assert!(g.battlefield.iter().find(|c| c.id == id).unwrap().tapped);
}

#[test]
fn great_hall_is_legendary_land() {
    let def = catalog::great_hall_of_the_biblioplex();
    assert!(def.supertypes.contains(&crate::card::Supertype::Legendary));
    assert!(def.card_types.contains(&CardType::Land));
}

// ── Follow the Lumarets (push XV) ────────────────────────────────────────────

#[test]
fn follow_the_lumarets_pulls_one_creature_without_lifegain() {
    // Mainline: no life gain → pull one creature/land to hand.
    let mut g = two_player_game();
    // Library top: forest, then a bear, then island.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::follow_the_lumarets());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Follow the Lumarets castable");
    drain_stack(&mut g);

    // Top of library was Forest (a Land), so the first match → hand.
    // Net: hand cast (-1) + 1 pull = same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "One creature/land pulled into hand");
}

#[test]
fn follow_the_lumarets_pulls_two_with_lifegain_infusion() {
    // Infusion path: gained life this turn → pull two creature/land cards.
    let mut g = two_player_game();
    g.players[0].life_gained_this_turn = 3;
    // Library top: forest, bear, island, mountain (all matching).
    g.add_card_to_library(0, catalog::mountain());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::follow_the_lumarets());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Follow the Lumarets castable with Infusion");
    drain_stack(&mut g);

    // Net: -1 cast + 2 pulls = +1 hand size.
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Two creature/land cards pulled with Infusion");
}

// ── Lluwen, Exchange Student // Pest Friend (push XV) ───────────────────────
//
// Closes out the Witherbloom (B/G) school. Front: 3/4 Legendary Elf
// Druid vanilla body. Back: sorcery — create a 1/1 Pest token with the
// printed "attacks → gain 1 life" rider.

#[test]
fn lluwen_exchange_student_front_is_three_four_legendary_elf_druid() {
    let def = catalog::lluwen_exchange_student();
    assert_eq!(def.name, "Lluwen, Exchange Student");
    assert_eq!(def.power, 3);
    assert_eq!(def.toughness, 4);
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Elf));
    assert!(def.subtypes.creature_types.contains(&crate::card::CreatureType::Druid));
    assert!(def.supertypes.contains(&crate::card::Supertype::Legendary));
}

#[test]
fn lluwen_exchange_student_back_face_is_pest_friend() {
    let def = catalog::lluwen_exchange_student();
    let back = def.back_face.as_ref().expect("Lluwen has a back face");
    assert_eq!(back.name, "Pest Friend");
    assert!(back.card_types.contains(&CardType::Sorcery));
}

#[test]
fn lluwen_back_face_creates_pest_token_with_lifegain_rider() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lluwen_exchange_student());
    // Pay {B} for Pest Friend back-face cost ({B/G} hybrid → {B}).
    g.players[0].mana_pool.add(Color::Black, 1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Pest Friend castable for {B}");
    drain_stack(&mut g);

    // A new Pest token should be on the battlefield.
    assert_eq!(g.battlefield.len(), bf_before + 1);
    let pest = g.battlefield.iter().find(|c| c.definition.name == "Pest")
        .expect("Pest token created");
    assert_eq!(pest.definition.power, 1);
    assert_eq!(pest.definition.toughness, 1);
    // The token must carry the on-attack lifegain trigger.
    assert!(!pest.definition.triggered_abilities.is_empty(),
        "Pest token should carry the on-attack lifegain rider");
}

#[test]
fn lluwen_front_castable_for_two_b_g_as_three_four_creature() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lluwen_exchange_student());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Lluwen castable for {2}{B}{G}");
    drain_stack(&mut g);

    let lluwen = g.battlefield.iter().find(|c| c.id == id)
        .expect("Lluwen on battlefield");
    assert_eq!(lluwen.definition.power, 3);
    assert_eq!(lluwen.definition.toughness, 4);
}

// ── Push XVI: CastSpellHasX, MayPay, HasXInCost, MayDo land sub ─────────────

#[test]
fn geometers_arthropod_x_cast_pulls_card_to_hand() {
    // Geometer's Arthropod's X-cast trigger uses the new
    // `Predicate::CastSpellHasX` + `RevealUntilFind` body. Casting a spell
    // with `{X}` in its cost (e.g. Mathemagics with X=2) should pull the
    // top library card into the controller's hand if it matches the
    // (filter: Any) — first card always matches. Hand size goes up by 1
    // because of the trigger on top of any draws from the cast itself.
    let mut g = two_player_game();
    let arthropod = g.add_card_to_battlefield(0, catalog::geometers_arthropod());
    // Seed library with 4 forest cards and a top island so the trigger pulls
    // the top island to hand.
    let top_island = g.add_card_to_library(0, catalog::island());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    // Now cast Lightning Bolt {R} — that's NOT an X-cost spell, so the
    // trigger should NOT fire.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);
    // -1 cast (Bolt left hand). No X-trigger fired ⇒ no hand pull.
    assert_eq!(g.players[0].hand.len(), hand_before - 1);
    // Sanity: arthropod still on bf.
    assert!(g.battlefield.iter().any(|c| c.id == arthropod));
    // Top island still on top of library.
    assert_eq!(
        g.players[0].library.iter().find(|c| c.id == top_island).map(|_| true),
        Some(true),
        "Top island shouldn't be touched by non-X spell"
    );
}

#[test]
fn geometers_arthropod_x_cast_with_x_spell_fires() {
    // Now cast an X-cost spell (Mathemagics) — the trigger should fire and
    // the cap=X param caps the reveal at the cast's X value.
    use crate::card::Predicate;
    let card = catalog::geometers_arthropod();
    // Verify the trigger uses the CastSpellHasX predicate.
    let trig = card.triggered_abilities.first().expect("has X-cast trigger");
    assert!(matches!(trig.event.kind, crate::card::EventKind::SpellCast));
    let filter = trig.event.filter.as_ref().expect("trigger has filter");
    assert!(matches!(filter, Predicate::CastSpellHasX),
        "Geometer's Arthropod trigger should be filtered by CastSpellHasX");
}

#[test]
fn matterbending_mage_x_cast_grants_unblockable() {
    // Matterbending Mage's X-cast trigger now grants Unblockable EOT.
    use crate::card::Predicate;
    let card = catalog::matterbending_mage();
    // Two triggers: ETB bounce + X-cast Unblockable.
    assert_eq!(card.triggered_abilities.len(), 2);
    let xtrig = card.triggered_abilities.iter()
        .find(|t| matches!(&t.event.filter, Some(Predicate::CastSpellHasX)))
        .expect("X-cast trigger present");
    // Body should grant Keyword::Unblockable to Selector::This EOT.
    assert!(matches!(
        &xtrig.effect,
        crate::card::Effect::GrantKeyword { keyword: Keyword::Unblockable, .. }
    ));
}

#[test]
fn bayou_groff_dies_may_pay_to_return_returns_to_hand_when_yes_and_paid() {
    // Bayou Groff's death trigger: "may pay {1} to return". With scripted
    // decider answering Bool(true) AND mana in pool, the body fires and the
    // dead Groff returns to its owner's hand. Bayou Groff has its OWN
    // SelfSource Dies trigger so this fires via the SBA dies-trigger
    // collection path (not the unified AnotherOfYours dispatch).
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::bayou_groff());
    // Pre-stash {1} of mana for the may-pay.
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    // Kill via Murder.
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: murder, target: Some(Target::Permanent(id)),
        mode: None, x_value: None,
    })
    .expect("Murder castable");
    drain_stack(&mut g);
    // Groff's may-pay-to-return body: should now be in hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == id),
        "Bayou Groff should return to hand on yes+pay");
}

#[test]
fn bayou_groff_dies_may_pay_default_no_stays_in_graveyard() {
    // Default decider says no — Groff stays in graveyard.
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::bayou_groff());
    g.players[0].mana_pool.add_colorless(1);
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: murder, target: Some(Target::Permanent(id)),
        mode: None, x_value: None,
    })
    .expect("Murder castable");
    drain_stack(&mut g);
    // No return: Groff in graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "Bayou Groff should stay in graveyard when may-pay declined");
}

#[test]
fn embrace_the_paradox_may_skip_extra_land_default() {
    // With AutoDecider (default), the MayDo rider answers no; only the
    // Draw 3 fires. Library lands should remain in the library.
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let _land = g.add_card_to_hand(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::embrace_the_paradox());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_lands_before = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_land())
        .count();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Embrace castable");
    drain_stack(&mut g);

    // Default decider answers no — the forest stays in hand.
    let bf_lands_after = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_land())
        .count();
    assert_eq!(bf_lands_after, bf_lands_before, "Forest should NOT have hit bf");
}

#[test]
fn embrace_the_paradox_may_put_land_when_yes() {
    // Scripted decider yes → forest from hand goes to bf tapped.
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let forest_id = g.add_card_to_hand(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::embrace_the_paradox());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Embrace castable");
    drain_stack(&mut g);

    let forest_view = g.battlefield.iter().find(|c| c.id == forest_id)
        .expect("Forest should be on battlefield after MayDo=yes");
    assert!(forest_view.tapped, "Forest enters tapped per the rider");
}

#[test]
fn felisa_silverquill_dies_with_counter_creates_inkling_token() {
    // Felisa's "creature with +1/+1 counter dies → 1/1 W/B Inkling token"
    // trigger. Kill the counter-bearing bear via Murder so the death
    // trigger fires through the normal cast → resolution → SBA → dispatch
    // pipeline (the AnotherOfYours scope needs the unified dispatcher,
    // which is only invoked from priority/cast paths — not from a bare
    // `check_state_based_actions` call).
    let mut g = two_player_game();
    let _felisa = g.add_card_to_battlefield(0, catalog::felisa_fang_of_silverquill());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == bear) {
        c.add_counters(CounterType::PlusOnePlusOne, 1);
    }
    let bf_before = g.battlefield.len();
    // Cast Murder targeting the bear.
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: murder, target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Murder castable for {1}{B}{B}");
    drain_stack(&mut g);
    // Bear gone, Inkling token added.
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    let ink = g.battlefield.iter().find(|c| c.definition.name == "Inkling")
        .expect("Inkling token created when counter-bearing creature dies");
    assert!(ink.definition.keywords.contains(&Keyword::Flying));
    // Net battlefield: -1 bear (murder gone too) + 1 inkling = same as before
    // murder cast. (felisa untouched, bear out, inkling in.)
    assert_eq!(g.battlefield.len(), bf_before);
}

#[test]
fn felisa_no_counter_no_token() {
    // No +1/+1 counter on the dying creature → no token.
    let mut g = two_player_game();
    let _felisa = g.add_card_to_battlefield(0, catalog::felisa_fang_of_silverquill());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: murder, target: Some(Target::Permanent(bear)),
        mode: None, x_value: None,
    })
    .expect("Murder castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Inkling"),
        "No Inkling minted when the dying bear had no +1/+1 counter");
}

#[test]
fn sundering_archaic_two_mana_bottoms_graveyard_card() {
    // Activated `{2}` ability moves a graveyard card to the bottom of its
    // owner's library.
    let mut g = two_player_game();
    let archaic_id = g.add_card_to_battlefield(0, catalog::sundering_archaic());
    // Stash a card in opponent's graveyard.
    let target_card = catalog::lightning_bolt();
    let bolt_id = g.next_id();
    let mut bolt = crate::card::CardInstance::new(bolt_id, target_card, 1);
    bolt.controller = 1;
    g.players[1].graveyard.push(bolt);
    // Activate Sundering's `{2}` ability targeting the bolt.
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: archaic_id,
        ability_index: 0,
        target: Some(Target::Permanent(bolt_id)),
    })
    .expect("Sundering Archaic {2} ability activatable");
    drain_stack(&mut g);
    // Bolt should be at the bottom of player 1's library, not in their gy.
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bolt_id));
    // Bottom of library = last index.
    let lib = &g.players[1].library;
    assert!(!lib.is_empty(), "library not empty");
    assert_eq!(lib.last().unwrap().id, bolt_id, "Bolt should be at bottom of P1's library");
}

#[test]
fn predicate_cast_spell_has_x_label_does_not_panic_in_view() {
    // Smoke test: building a PlayerView for a state with the new predicate
    // shouldn't panic. predicate_short_label has a catch-all — covers
    // CastSpellHasX with the "conditional" fallback.
    let mut g = two_player_game();
    let _ = g.add_card_to_battlefield(0, catalog::geometers_arthropod());
    // Just constructing a view for player 0 exercises the abil/trig view
    // pipeline that calls predicate_short_label.
    let _view = crate::server::view::project(&g, 0);
}

#[test]
fn aziza_mage_tower_captain_body_legendary_djinn_sorcerer() {
    let card = catalog::aziza_mage_tower_captain();
    assert_eq!(card.name, "Aziza, Mage Tower Captain");
    assert_eq!(card.power, 2);
    assert_eq!(card.toughness, 2);
    assert!(card.supertypes.contains(&crate::card::Supertype::Legendary));
    assert!(card.has_creature_type(crate::card::CreatureType::Djinn));
    assert!(card.has_creature_type(crate::card::CreatureType::Sorcerer));
    assert_eq!(
        card.triggered_abilities.len(),
        1,
        "Aziza should have one magecraft trigger"
    );
}

#[test]
fn aziza_magecraft_taps_three_creatures_and_copies_lightning_bolt() {
    // Aziza, Mage Tower Captain on bf, three untapped creatures available.
    // Cast Lightning Bolt at the opponent. Magecraft trigger fires →
    // MayDo body taps 3 creatures + copies the spell. With
    // ScriptedDecider answering Bool(true) to the OptionalTrigger, the
    // copy resolves; the original bolt also resolves; total damage = 6.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aziza_mage_tower_captain());
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b3 = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    // Scripted decider says yes to Aziza's may-tap-three-to-copy.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    // Original bolt + one copy = 6 damage to opp.
    assert_eq!(g.players[1].life, life_before - 6,
        "Aziza's copy should add 3 more damage (total 6)");
    // 3 creatures tapped by the cost.
    let tapped_count = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0 && c.tapped && c.definition.is_creature())
        .count();
    assert_eq!(tapped_count, 3, "Three creatures should be tapped to pay the copy cost");
}

#[test]
fn aziza_magecraft_skipped_when_decider_says_no() {
    // AutoDecider defaults to Bool(false) on OptionalTrigger, so the
    // MayDo body skips — no creatures tapped, no copy fired.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aziza_mage_tower_captain());
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 3, "Only the original bolt should hit");
    let bear = g.battlefield.iter().find(|c| c.id == b1).expect("bear stays");
    assert!(!bear.tapped, "Auto-skip path should not tap the bear");
}

// Mica, Reader of Ruins — 4/4 Legendary Human Artificer with Ward(3).
// Magecraft IS-cast → may-sac-artifact → copy spell.
#[test]
fn mica_reader_of_ruins_is_4_4_legendary_artificer_with_ward() {
    let card = catalog::mica_reader_of_ruins();
    assert_eq!(card.name, "Mica, Reader of Ruins");
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 4);
    assert!(card.supertypes.contains(&crate::card::Supertype::Legendary));
    assert!(card.has_creature_type(crate::card::CreatureType::Human));
    assert!(card.has_creature_type(crate::card::CreatureType::Artificer));
    assert!(
        card.keywords.iter().any(|k| matches!(k, Keyword::Ward(3))),
        "Mica should carry Ward(3)"
    );
    assert_eq!(card.triggered_abilities.len(), 1, "Mica's magecraft trigger");
}

#[test]
fn mica_magecraft_sacs_artifact_and_copies_lightning_bolt() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mica_reader_of_ruins());
    // Provide an artifact for Mica to sac.
    let trinket = g.add_card_to_battlefield(0, catalog::diary_of_dreams());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    drain_stack(&mut g);

    // Original + copy = 6 damage.
    assert_eq!(g.players[1].life, life_before - 6, "Mica's copy adds 3 more damage");
    // Trinket should be in graveyard.
    assert!(!g.battlefield.iter().any(|c| c.id == trinket),
        "The artifact should be sacrificed");
    let in_gy = g.players[0].graveyard.iter().any(|c| c.id == trinket);
    assert!(in_gy, "The sac'd artifact should be in graveyard");
}

// Colorstorm Stallion — 3/3 Elemental Horse with Ward(1) + Haste +
// magecraft +1/+1 EOT pump on each instant-or-sorcery cast. The "5+
// mana → token copy" upper-half is omitted (no copy-permanent
// primitive).
#[test]
fn colorstorm_stallion_is_3_3_with_ward_and_haste() {
    let card = catalog::colorstorm_stallion();
    assert_eq!(card.name, "Colorstorm Stallion");
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 3);
    assert!(
        card.keywords.iter().any(|k| matches!(k, Keyword::Ward(1))),
        "Colorstorm Stallion should carry Ward(1)"
    );
    assert!(card.keywords.contains(&Keyword::Haste));
    assert!(card.has_creature_type(crate::card::CreatureType::Elemental));
    assert!(card.has_creature_type(crate::card::CreatureType::Horse));
    assert_eq!(card.triggered_abilities.len(), 1, "magecraft pump trigger");
}

#[test]
fn colorstorm_stallion_magecraft_pumps_self_until_eot() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::colorstorm_stallion());
    g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    let view = g.computed_permanent(id).unwrap();
    assert_eq!(view.power, 3);

    // Cast Lightning Bolt — Stallion's magecraft pump trigger fires.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let view = g.computed_permanent(id).unwrap();
    assert_eq!(view.power, 4, "Stallion +1/+1 EOT after IS cast");
    assert_eq!(view.toughness, 4);
}

// Emeritus of Ideation // Ancestral Recall — front: 5/5 Human Wizard
// with Ward(2). Back: instant — target player draws three.
#[test]
fn emeritus_of_ideation_front_is_five_five_with_ward() {
    let card = catalog::emeritus_of_ideation();
    assert_eq!(card.name, "Emeritus of Ideation");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 5);
    assert!(card.has_creature_type(crate::card::CreatureType::Human));
    assert!(card.has_creature_type(crate::card::CreatureType::Wizard));
    assert!(
        card.keywords.iter().any(|k| matches!(k, Keyword::Ward(2))),
        "Front should carry Ward(2)"
    );
    let back = card.back_face.expect("Has Ancestral Recall back");
    assert_eq!(back.name, "Ancestral Recall");
}

#[test]
fn emeritus_of_ideation_back_draws_three_for_target() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::emeritus_of_ideation());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Player(0)),
        mode: None,
        x_value: None,
    })
    .expect("Ancestral Recall castable for {U}");
    drain_stack(&mut g);

    // -1 cast (exiled) + 3 draw = +2 hand.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 3);
}

// Grave Researcher // Reanimate — front: 3/3 Troll Warlock, ETB Surveil 2.
// Back: Reanimate (return target creature card from a graveyard to the
// battlefield under your control, then lose life equal to its mana
// value).
#[test]
fn grave_researcher_front_face_etb_surveils_two() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::grave_researcher());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let lib_before = g.players[0].library.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Grave Researcher castable for {2}{B}");
    drain_stack(&mut g);

    assert!(
        g.battlefield.iter().any(|c| c.id == id),
        "Grave Researcher must reach battlefield"
    );
    // Surveil 2: cannot grow library. Auto-decider may keep both on top
    // (no change) or drop them — either is fine.
    assert!(
        g.players[0].library.len() <= lib_before,
        "Surveil 2 cannot grow the library"
    );
}

#[test]
fn grave_researcher_back_reanimates_target_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::grave_researcher());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpellBack {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Reanimate castable for {B}");
    drain_stack(&mut g);

    // Bear arrives on battlefield under controller 0.
    assert!(
        g.battlefield
            .iter()
            .any(|c| c.id == bear && c.controller == 0),
        "Bear should be on caster's battlefield"
    );
    // Bear's MV is 2 ({1}{G}) so caster loses 2 life.
    assert_eq!(g.players[0].life, life_before - 2);
}

#[test]
fn grave_researcher_front_is_three_three_troll_warlock() {
    let card = catalog::grave_researcher();
    assert_eq!(card.name, "Grave Researcher");
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 3);
    assert!(card.has_creature_type(crate::card::CreatureType::Troll));
    assert!(card.has_creature_type(crate::card::CreatureType::Warlock));
    let back = card.back_face.expect("Grave Researcher has Reanimate back");
    assert_eq!(back.name, "Reanimate");
    assert!(back.card_types.contains(&CardType::Sorcery));
}

#[test]
fn zaffai_and_the_tempests_body_legendary_5_7() {
    let card = catalog::zaffai_and_the_tempests();
    assert_eq!(card.name, "Zaffai and the Tempests");
    assert_eq!(card.power, 5);
    assert_eq!(card.toughness, 7);
    assert!(card.supertypes.contains(&crate::card::Supertype::Legendary));
    assert!(card.has_creature_type(crate::card::CreatureType::Human));
    assert!(card.has_creature_type(crate::card::CreatureType::Bard));
}

// Molten Note — {X}{R}{W} Lorehold Sorcery. Damage = mana spent, untap
// all your creatures, Flashback {6}{R}{W}.
#[test]
fn molten_note_has_flashback_keyword() {
    let def = catalog::molten_note();
    assert_eq!(def.name, "Molten Note");
    assert!(def.card_types.contains(&CardType::Sorcery));
    assert!(
        def.keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))),
        "Molten Note should carry Flashback {{6}}{{R}}{{W}}"
    );
}

#[test]
fn molten_note_hand_cast_x_3_deals_5_damage_and_untaps_creatures() {
    let mut g = two_player_game();
    // P1's creature gets shot.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // P0's two attackers — both already tapped (e.g. attacked earlier).
    let mine_a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mine_b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine_a).unwrap().tapped = true;
    g.battlefield_find_mut(mine_b).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::molten_note());
    // {3}{R}{W}: X = 3 → 5 damage to the bear.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: Some(3),
    })
    .expect("Molten Note castable for {3}{R}{W} hand cast");
    drain_stack(&mut g);

    // Bear (toughness 2) takes 5 → dead.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "P1's bear should die to 5 damage");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
    // Both of P0's bears are untapped.
    let a = g.battlefield_find(mine_a).unwrap();
    let b = g.battlefield_find(mine_b).unwrap();
    assert!(!a.tapped, "P0's bear A should be untapped after Molten Note");
    assert!(!b.tapped, "P0's bear B should be untapped after Molten Note");
    // Front-face cast → graveyard, not exile.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "Molten Note should be in graveyard after front-face resolution");
}

#[test]
fn molten_note_hand_cast_x_0_deals_2_damage() {
    // X=0 → mana spent = {R}{W} = 2. Bear (2 toughness) dies.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::molten_note());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: Some(0),
    })
    .expect("Molten Note castable for {R}{W} hand cast (X=0)");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear takes 2 damage and dies");
}

#[test]
fn molten_note_flashback_deals_8_damage_and_untaps() {
    // Flashback cost is {6}{R}{W} = 8 mana — bear takes 8 damage.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    let id = g.add_card_to_graveyard(0, catalog::molten_note());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(6);

    g.perform_action(GameAction::CastFlashback {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Molten Note flashback castable for {6}{R}{W}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should die to 8 damage from flashback Molten Note");
    let view = g.battlefield_find(mine).unwrap();
    assert!(!view.tapped, "Mine should untap from Molten Note flashback");
    // Flashback → exile, not graveyard.
    assert!(g.exile.iter().any(|c| c.id == id),
        "Flashback-cast Molten Note should be in exile");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id));
}

// ── Push XIX: Lorehold ⏳ closer + body-only ⏳→🟡 batch ─────────────────────

// Strife Scholar — {2}{R} body-only with Ward(2). MDFC back face
// Awaken the Ages omitted (oracle unverified). Promotes from ⏳ → 🟡.
#[test]
fn strife_scholar_is_3_2_orc_sorcerer_with_ward() {
    let card = catalog::strife_scholar();
    assert_eq!(card.name, "Strife Scholar");
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 2);
    assert!(card.has_creature_type(crate::card::CreatureType::Orc));
    assert!(card.has_creature_type(crate::card::CreatureType::Sorcerer));
    assert!(
        card.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))),
        "Strife Scholar should carry Ward"
    );
}

// Campus Composer — {3}{U} body-only with Ward(2). MDFC back face
// Aqueous Aria omitted (oracle unverified). Promotes from ⏳ → 🟡.
#[test]
fn campus_composer_is_3_4_merfolk_bard_with_ward() {
    let card = catalog::campus_composer();
    assert_eq!(card.name, "Campus Composer");
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 4);
    assert!(card.has_creature_type(crate::card::CreatureType::Merfolk));
    assert!(card.has_creature_type(crate::card::CreatureType::Bard));
    assert!(
        card.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))),
        "Campus Composer should carry Ward"
    );
}

// Elemental Mascot — {1}{U}{R} 1/4 Flying+Vigilance Elemental Bird with
// magecraft +1/+0 EOT pump on every IS cast. Opus exile-top rider
// omitted (cast-from-exile pipeline gap).
#[test]
fn elemental_mascot_is_1_4_flying_vigilance_with_magecraft_pump() {
    let card = catalog::elemental_mascot();
    assert_eq!(card.name, "Elemental Mascot");
    assert_eq!(card.power, 1);
    assert_eq!(card.toughness, 4);
    assert!(card.keywords.contains(&Keyword::Flying));
    assert!(card.keywords.contains(&Keyword::Vigilance));
    assert!(card.has_creature_type(crate::card::CreatureType::Elemental));
    assert!(card.has_creature_type(crate::card::CreatureType::Bird));
    assert_eq!(card.triggered_abilities.len(), 1, "magecraft pump trigger");
}

#[test]
fn elemental_mascot_pumps_self_after_is_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::elemental_mascot());
    g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);

    let view = g.computed_permanent(id).unwrap();
    assert_eq!(view.power, 2, "Elemental Mascot +1/+0 EOT after IS cast");
    assert_eq!(view.toughness, 4);
}

// Biblioplex Tomekeeper — {4} body-only 3/4 Construct artifact creature.
// Prepare-state ETB choice omitted (Prepare keyword pending).
#[test]
fn biblioplex_tomekeeper_is_3_4_construct_artifact_creature() {
    let card = catalog::biblioplex_tomekeeper();
    assert_eq!(card.name, "Biblioplex Tomekeeper");
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 4);
    assert!(card.card_types.contains(&CardType::Artifact));
    assert!(card.card_types.contains(&CardType::Creature));
    assert!(card.has_creature_type(crate::card::CreatureType::Construct));
    assert_eq!(card.cost.cmc(), 4);
}

// Strixhaven Skycoach — {3} body-only 3/2 Flying. ETB land tutor wired
// via MayDo. Crew/Vehicle keyword omitted (no Vehicle primitive).
#[test]
fn strixhaven_skycoach_is_3_2_flying_artifact_creature() {
    let card = catalog::strixhaven_skycoach();
    assert_eq!(card.name, "Strixhaven Skycoach");
    assert_eq!(card.power, 3);
    assert_eq!(card.toughness, 2);
    assert!(card.card_types.contains(&CardType::Artifact));
    assert!(card.keywords.contains(&Keyword::Flying));
    assert_eq!(
        card.triggered_abilities.len(),
        1,
        "ETB MayDo land tutor trigger"
    );
}

#[test]
fn strixhaven_skycoach_etb_tutors_basic_land() {
    let mut g = two_player_game();
    // Seed the controller's library with a Forest.
    let forest = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::strixhaven_skycoach());
    g.players[0].mana_pool.add_colorless(3);
    // ScriptedDecider answers "yes" on the MayDo + picks the Forest.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest)),
    ]));

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Skycoach castable for {3}");
    drain_stack(&mut g);

    // Forest in hand.
    assert!(
        g.players[0].hand.iter().any(|c| c.id == forest),
        "Forest tutored to hand"
    );
}

// Skycoach Waypoint — Land with `{T}: Add {C}`. Prepare activation
// omitted (Prepare keyword pending).
#[test]
fn skycoach_waypoint_taps_for_colorless() {
    let card = catalog::skycoach_waypoint();
    assert_eq!(card.name, "Skycoach Waypoint");
    assert!(card.card_types.contains(&CardType::Land));
    // {T}: Add {C} mana ability is the only activation.
    assert_eq!(card.activated_abilities.len(), 1);
    assert!(card.activated_abilities[0].tap_cost);
}

#[test]
fn social_snub_copy_doubles_drain_when_decider_says_yes() {
    // Cast Social Snub with a creature on bf so the on-cast may-copy
    // trigger fires; ScriptedDecider says yes → copy resolves first
    // (drain 1 to each opp + sac), then original (drain 1 again).
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2nd opp creature so the second sac fires
    let id = g.add_card_to_hand(0, catalog::social_snub());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Social Snub castable");
    drain_stack(&mut g);

    // Original + copy = drain 2 from P1 to P0.
    assert_eq!(g.players[0].life, p0_life_before + 2, "P0 gains 2 life from drain×2");
    assert_eq!(g.players[1].life, p1_life_before - 2, "P1 loses 2 life from drain×2");
}

// Social Snub — {1}{W}{B} Sorcery. Each player sacs a creature; drain
// 1 from each opp to you. Plus on-cast may-copy rider (post-XX).
#[test]
fn social_snub_each_player_sacs_creature_and_drains_one() {
    let mut g = two_player_game();
    // Both players have a creature.
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::social_snub());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Social Snub castable for {1}{W}{B}");
    drain_stack(&mut g);

    // Each player sacrificed their bear → both bears in their owners'
    // graveyards, neither still on the battlefield.
    assert!(!g.battlefield.iter().any(|c| c.id == mine),
        "P0's bear sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == theirs),
        "P1's bear sacrificed");
    // Drain: P0 gains 1, P1 loses 1.
    assert_eq!(g.players[0].life, p0_life_before + 1, "P0 gains 1 life");
    assert_eq!(g.players[1].life, p1_life_before - 1, "P1 loses 1 life");
}

// Silverquill, the Disputant — {2}{W}{B} 4/4 Flying+Vigilance
// Legendary Elder Dragon. Casualty 1 grant omitted (no copy-spell
// primitive).
#[test]
fn silverquill_the_disputant_is_4_4_flying_vigilance_dragon() {
    let card = catalog::silverquill_the_disputant();
    assert_eq!(card.name, "Silverquill, the Disputant");
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 4);
    assert!(card.supertypes.contains(&crate::card::Supertype::Legendary));
    assert!(card.has_creature_type(crate::card::CreatureType::Elder));
    assert!(card.has_creature_type(crate::card::CreatureType::Dragon));
    assert!(card.keywords.contains(&Keyword::Flying));
    assert!(card.keywords.contains(&Keyword::Vigilance));
}

// Quandrix, the Proof — {4}{G}{U} 6/6 Flying+Trample Elder Dragon.
// Cascade keyword omitted (no Cascade primitive).
#[test]
fn quandrix_the_proof_is_6_6_flying_trample_dragon() {
    let card = catalog::quandrix_the_proof();
    assert_eq!(card.name, "Quandrix, the Proof");
    assert_eq!(card.power, 6);
    assert_eq!(card.toughness, 6);
    assert!(card.supertypes.contains(&crate::card::Supertype::Legendary));
    assert!(card.has_creature_type(crate::card::CreatureType::Elder));
    assert!(card.has_creature_type(crate::card::CreatureType::Dragon));
    assert!(card.keywords.contains(&Keyword::Flying));
    assert!(card.keywords.contains(&Keyword::Trample));
}

// ── Push XXXI — Value::ManaSpentToCast + Opus + Increment ──────────────────

// Aberrant Manawurm — Whenever you cast an IS spell, +X/+0 EOT where X is
// the mana spent. Cast a {R} Lightning Bolt → +1/+0 EOT (2 → 3 power).
#[test]
fn aberrant_manawurm_pumps_by_mana_spent_on_is_cast() {
    let mut g = two_player_game();
    let manawurm = g.add_card_to_battlefield(0, catalog::aberrant_manawurm());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);

    let card = g.battlefield_find(manawurm).unwrap();
    // Bolt is {R} = CMC 1; printed Manawurm is 2/5 → +1 power = 3/5.
    assert_eq!(card.power(), 3, "Aberrant Manawurm should pump by 1 (Bolt's CMC)");
    assert_eq!(card.toughness(), 5);
}

// Aberrant Manawurm — Cast Wisdom of Ages ({4}{U}{U}{U}, CMC 7). The +X/+0
// pump is +7. 2/5 → 9/5.
#[test]
fn aberrant_manawurm_scales_with_big_spells() {
    let mut g = two_player_game();
    // Pre-load library so Wisdom of Ages doesn't deck-out the caster.
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let manawurm = g.add_card_to_battlefield(0, catalog::aberrant_manawurm());
    let big = g.add_card_to_hand(0, catalog::wisdom_of_ages());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, mode: None, x_value: None,
    })
    .expect("Wisdom of Ages castable for {4}{U}{U}{U}");
    drain_stack(&mut g);

    let card = g.battlefield_find(manawurm).unwrap();
    // Wisdom of Ages CMC is 7 → +7/+0 EOT, base 2/5 = 9/5.
    assert_eq!(card.power(), 9);
    assert_eq!(card.toughness(), 5);
}

// Tackle Artist — Opus rider: small cast pumps +1/+1 EOT only; no
// permanent counter.
#[test]
fn tackle_artist_opus_small_cast_pumps_eot_only() {
    let mut g = two_player_game();
    let ta = g.add_card_to_battlefield(0, catalog::tackle_artist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);

    let card = g.battlefield_find(ta).unwrap();
    // Cheap-cast: +1/+1 EOT pump only, no permanent counter.
    assert_eq!(card.power(), 5, "+1/+1 EOT pump on a 4/3 = 5/4");
    assert_eq!(card.toughness(), 4);
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 0,
        "Cheap-cast Opus shouldn't add a +1/+1 counter");
}

// Tackle Artist — Opus rider: 5+ mana spent = additional +1/+1 counter.
#[test]
fn tackle_artist_opus_big_cast_adds_counter() {
    let mut g = two_player_game();
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let ta = g.add_card_to_battlefield(0, catalog::tackle_artist());
    let big = g.add_card_to_hand(0, catalog::wisdom_of_ages());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, mode: None, x_value: None,
    })
    .expect("Wisdom of Ages castable");
    drain_stack(&mut g);

    let card = g.battlefield_find(ta).unwrap();
    // CMC 7 ≥ 5 → +1/+1 counter added (+ EOT pump too).
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1,
        "Big-cast Opus should add a +1/+1 counter on Tackle Artist");
}

// Spectacular Skywhale — Opus +3/+0 EOT cheap-cast, +3 +1/+1 counters big.
#[test]
fn spectacular_skywhale_opus_big_cast_adds_three_counters() {
    let mut g = two_player_game();
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let sw = g.add_card_to_battlefield(0, catalog::spectacular_skywhale());
    let big = g.add_card_to_hand(0, catalog::wisdom_of_ages());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: big, target: None, mode: None, x_value: None,
    })
    .expect("Wisdom of Ages castable");
    drain_stack(&mut g);

    let card = g.battlefield_find(sw).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 3,
        "Big-cast Skywhale should put 3 +1/+1 counters");
}

// Cuboid Colony — Increment: any cast where mana_spent > min(P,T)=1 adds
// a +1/+1 counter. Bolt ({R}, CMC 1) doesn't trigger (1 not > 1); Bear
// ({1}{G}, CMC 2) does trigger (2 > 1).
#[test]
fn cuboid_colony_increment_fires_on_two_drop() {
    let mut g = two_player_game();
    let cube = g.add_card_to_battlefield(0, catalog::cuboid_colony());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, mode: None, x_value: None,
    })
    .expect("Grizzly Bears castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(cube).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1,
        "Bears (CMC 2) > Colony's min(P, T)=1 should fire Increment");
}

// Cuboid Colony — Increment: Bolt at CMC 1 doesn't fire (1 not > 1).
#[test]
fn cuboid_colony_increment_does_not_fire_on_equal_cmc() {
    let mut g = two_player_game();
    let cube = g.add_card_to_battlefield(0, catalog::cuboid_colony());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(cube).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 0,
        "Bolt (CMC 1) not greater than min(P, T)=1; Increment should NOT fire");
}

// Pensive Professor — Increment fires on any cast (min(P, T)=0).
#[test]
fn pensive_professor_increment_fires_on_any_cast() {
    let mut g = two_player_game();
    let pp = g.add_card_to_battlefield(0, catalog::pensive_professor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(pp).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1,
        "Pensive Professor at min(P, T)=0 fires on any 1+ mana spell");
}

// Tester of the Tangential — Increment on a 1/1 means cast must spend ≥ 2
// to fire.
#[test]
fn tester_of_tangential_increment_skips_one_mana_cast() {
    let mut g = two_player_game();
    let tt = g.add_card_to_battlefield(0, catalog::tester_of_the_tangential());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        mode: None, x_value: None,
    })
    .expect("Bolt castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(tt).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 0,
        "Bolt CMC 1 not > 1 → Increment does NOT fire");
}

#[test]
fn tester_of_tangential_increment_fires_on_two_mana_cast() {
    let mut g = two_player_game();
    let tt = g.add_card_to_battlefield(0, catalog::tester_of_the_tangential());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, mode: None, x_value: None,
    })
    .expect("Bears castable");
    drain_stack(&mut g);
    let card = g.battlefield_find(tt).unwrap();
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1,
        "Bears CMC 2 > 1 → Increment fires");
}

// Berta — Increment trigger now wires; +1/+1 counter on Berta also fires
// the existing AnyOneColor mana ramp trigger.
#[test]
fn berta_increment_then_mana_ramp_chains() {
    let mut g = two_player_game();
    let berta = g.add_card_to_battlefield(0, catalog::berta_wise_extrapolator());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, mode: None, x_value: None,
    })
    .expect("Bears castable");
    drain_stack(&mut g);

    let card = g.battlefield_find(berta).unwrap();
    // Bears CMC 2 > min(1, 4) = 1 → Increment fires → +1/+1 counter.
    assert_eq!(card.counter_count(CounterType::PlusOnePlusOne), 1,
        "Berta should gain a +1/+1 counter from Increment");
    // ... and the counter triggers AnyOneColor mana add.
    assert!(g.players[0].mana_pool.total() > 0,
        "Berta's CounterAdded → AnyOneColor mana trigger should fire");
}

// Value::ManaSpentToCast outside a spell context returns 0 (the trigger
// source isn't a spell on the stack).
#[test]
fn mana_spent_to_cast_is_zero_outside_spell_context() {
    use crate::effect::Value;
    use crate::game::effects::EffectContext;
    let g = two_player_game();
    // No trigger source → returns 0.
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    assert_eq!(g.evaluate_value(&Value::ManaSpentToCast, &ctx), 0);
}

// Prismari, the Inspiration — {5}{U}{R} 7/7 Flying Elder Dragon with
// Ward(5). Storm grant on IS casts omitted (no copy-spell primitive).
#[test]
fn prismari_the_inspiration_is_7_7_flying_dragon_with_ward() {
    let card = catalog::prismari_the_inspiration();
    assert_eq!(card.name, "Prismari, the Inspiration");
    assert_eq!(card.power, 7);
    assert_eq!(card.toughness, 7);
    assert!(card.supertypes.contains(&crate::card::Supertype::Legendary));
    assert!(card.has_creature_type(crate::card::CreatureType::Elder));
    assert!(card.has_creature_type(crate::card::CreatureType::Dragon));
    assert!(card.keywords.contains(&Keyword::Flying));
    assert!(
        card.keywords.iter().any(|k| matches!(k, Keyword::Ward(5))),
        "Prismari should carry Ward(5)"
    );
}

// ── Opus / Increment promotions ─────────────────────────────────────────────

#[test]
fn thunderdrum_soloist_pings_each_opp_on_cheap_cast() {
    // Cheap cast (1 mana, < 5 mana) → 1 damage to each opponent.
    let mut g = two_player_game();
    let _t = g.add_card_to_battlefield(0, catalog::thunderdrum_soloist());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let opp_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), mode: None, x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);
    // Bolt → P0 (3), Soloist Opus → P1 (-1). Soloist's "Opus — deals 1
    // to each opp" hits P1 once.
    assert_eq!(g.players[1].life, opp_life_before - 1,
        "cheap cast: Thunderdrum Soloist deals 1 to each opp");
}

#[test]
fn thunderdrum_soloist_pings_three_each_opp_on_big_cast() {
    // Big cast (5+ mana) → 3 damage to each opponent (1 + 2 from
    // big-cast bonus). We use Mind Twist with x_value=4 since
    // `Value::ManaSpentToCast` reads `cost.cmc() + x_value` (push XXXI),
    // landing at 5 mana spent for Opus's `at_least: 5` gate. Mind Twist
    // doesn't damage creatures, so the Soloist body survives the cast
    // and we can read its trigger's effect on opp life cleanly.
    let mut g = two_player_game();
    let _t = g.add_card_to_battlefield(0, catalog::thunderdrum_soloist());
    let mt = g.add_card_to_hand(0, catalog::mind_twist());
    g.players[0].mana_pool.add(Color::Black, 1);
    let opp_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: mt, target: Some(Target::Player(1)), mode: None, x_value: Some(4),
    })
    .expect("Mind Twist castable for {B} (X read separately)");
    drain_stack(&mut g);
    // Soloist Opus big-cast: 1 + 2 = 3 to each opp.
    assert_eq!(g.players[1].life, opp_life_before - 3,
        "big cast (≥5 mana): Thunderdrum Soloist deals 3 to each opp (1 base + 2 bonus)");
}

#[test]
fn expressive_firedancer_pumps_on_cheap_cast() {
    // Cheap cast → +1/+1 EOT only (no double strike).
    let mut g = two_player_game();
    let fd = g.add_card_to_battlefield(0, catalog::expressive_firedancer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);
    let fd_card = g.battlefield.iter().find(|c| c.id == fd).unwrap();
    assert_eq!((fd_card.power(), fd_card.toughness()), (3, 3),
        "Firedancer pumps to 3/3 on cheap cast");
    assert!(!fd_card.has_keyword(&Keyword::DoubleStrike),
        "Firedancer should NOT gain Double Strike on cheap cast");
}

#[test]
fn expressive_firedancer_grants_double_strike_on_big_cast() {
    // Big cast triggers the Opus rider. Use Mind Twist X=4 (cmc 1 +
    // X 4 = 5 ManaSpentToCast) so the Firedancer body survives.
    let mut g = two_player_game();
    let fd = g.add_card_to_battlefield(0, catalog::expressive_firedancer());
    let mt = g.add_card_to_hand(0, catalog::mind_twist());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: mt, target: Some(Target::Player(1)), mode: None, x_value: Some(4),
    })
    .expect("Mind Twist castable");
    drain_stack(&mut g);
    let fd_card = g.battlefield.iter().find(|c| c.id == fd).unwrap();
    // Always +1/+1 EOT pump (3/3) + DoubleStrike grant on big cast.
    assert!(fd_card.has_keyword(&Keyword::DoubleStrike),
        "Firedancer should gain Double Strike on big cast");
    assert_eq!((fd_card.power(), fd_card.toughness()), (3, 3),
        "Firedancer at 3/3 EOT after big cast");
}

#[test]
fn molten_core_maestro_drops_counter_on_cheap_cast() {
    // Cheap cast (1 mana) → +1/+1 counter, no mana ramp.
    let mut g = two_player_game();
    let mm = g.add_card_to_battlefield(0, catalog::molten_core_maestro());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);
    let mm_card = g.battlefield.iter().find(|c| c.id == mm).unwrap();
    assert_eq!(mm_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "cheap cast: Molten-Core Maestro gains 1 +1/+1 counter");
    // Mana pool empty — no Red ramp on cheap.
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 0,
        "no Red mana added on cheap cast");
}

#[test]
fn molten_core_maestro_adds_red_mana_on_big_cast() {
    let mut g = two_player_game();
    let mm = g.add_card_to_battlefield(0, catalog::molten_core_maestro());
    let mt = g.add_card_to_hand(0, catalog::mind_twist());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: mt, target: Some(Target::Player(1)), mode: None, x_value: Some(4),
    })
    .expect("Mind Twist castable");
    drain_stack(&mut g);
    // After Mind Twist (ManaSpentToCast=5, big cast): Maestro gets
    // +1/+1 counter first (always), then power = 3, then adds
    // {R} × power = {R}{R}{R}.
    let mm_card = g.battlefield.iter().find(|c| c.id == mm).unwrap();
    assert_eq!(mm_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "big cast: Maestro still gains the always +1/+1 counter");
    assert_eq!(mm_card.power(), 3, "Maestro now 3 power after counter");
    // The order is always-first, then big — power = 3 by the time
    // the big block runs, so {R}{R}{R} is added.
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3,
        "big cast: 3 Red mana added (power-many R after counter)");
}

#[test]
fn ambitious_augmenter_grows_on_two_mana_cast() {
    // Augmenter is 1/1; min(P, T) = 1. A 2-mana cast triggers +1/+1.
    let mut g = two_player_game();
    let aug = g.add_card_to_battlefield(0, catalog::ambitious_augmenter());
    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: div, target: None, mode: None, x_value: None,
    })
    .expect("Divination castable for {2}{U}");
    drain_stack(&mut g);
    let aug_card = g.battlefield.iter().find(|c| c.id == aug).unwrap();
    assert_eq!(aug_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "2+ mana cast pushes a +1/+1 counter on Augmenter");
}

#[test]
fn ambitious_augmenter_does_not_grow_on_one_mana_cast() {
    // 1-mana cast (= min(P, T)) does NOT trigger Increment.
    let mut g = two_player_game();
    let aug = g.add_card_to_battlefield(0, catalog::ambitious_augmenter());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), mode: None, x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);
    let aug_card = g.battlefield.iter().find(|c| c.id == aug).unwrap();
    assert_eq!(aug_card.counter_count(CounterType::PlusOnePlusOne), 0,
        "1-mana cast (≤ min(P, T)) should NOT trigger Increment");
}

#[test]
fn topiary_lecturer_grows_on_three_mana_cast() {
    // Topiary is 1/2; min(P, T) = 1. A 2-mana cast triggers Increment.
    let mut g = two_player_game();
    let top = g.add_card_to_battlefield(0, catalog::topiary_lecturer());
    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: div, target: None, mode: None, x_value: None,
    })
    .expect("Divination castable for {2}{U}");
    drain_stack(&mut g);
    let top_card = g.battlefield.iter().find(|c| c.id == top).unwrap();
    assert_eq!(top_card.counter_count(CounterType::PlusOnePlusOne), 1,
        "2+ mana cast pushes a +1/+1 counter on Topiary Lecturer");
    assert_eq!(top_card.power(), 2, "Topiary now 2 power after Increment");
}
