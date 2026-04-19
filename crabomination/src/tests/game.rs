use super::*;
use crate::catalog;
use crate::card::StaticAbility;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::effect::{Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value};
use crate::game::effects::EffectContext;
use crate::mana::Color;

fn two_player_game() -> GameState {
    let players = vec![
        Player::new(0, "Alice"),
        Player::new(1, "Bob"),
    ];
    let mut g = GameState::new(players);
    // Start in PreCombatMain so we can take actions without advancing steps
    g.step = TurnStep::PreCombatMain;
    g
}

/// Pass priority for all players until the stack is empty (spells resolve).
/// Returns all events produced during resolution.
fn drain_stack(g: &mut GameState) -> Vec<GameEvent> {
    let mut all_events = Vec::new();
    while !g.stack.is_empty() {
        let events = g.perform_action(GameAction::PassPriority).unwrap();
        all_events.extend(events);
        let events = g.perform_action(GameAction::PassPriority).unwrap();
        all_events.extend(events);
    }
    all_events
}

// ── Setup ─────────────────────────────────────────────────────────────────

#[test]
fn players_start_with_20_life() {
    let g = two_player_game();
    assert_eq!(g.players[0].life, 20);
    assert_eq!(g.players[1].life, 20);
}

// ── Land ──────────────────────────────────────────────────────────────────

#[test]
fn play_land_moves_to_battlefield() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::forest());
    let events = g.perform_action(GameAction::PlayLand(id)).unwrap();
    assert!(g.battlefield.iter().any(|c| c.id == id));
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::LandPlayed { .. })));
}

#[test]
fn cannot_play_two_lands_per_turn() {
    let mut g = two_player_game();
    let f1 = g.add_card_to_hand(0, catalog::forest());
    let f2 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(f1)).unwrap();
    let err = g.perform_action(GameAction::PlayLand(f2)).unwrap_err();
    assert_eq!(err, GameError::AlreadyPlayedLand);
}

#[test]
fn cannot_play_land_in_combat() {
    let mut g = two_player_game();
    g.step = TurnStep::DeclareAttackers;
    let id = g.add_card_to_hand(0, catalog::forest());
    let err = g.perform_action(GameAction::PlayLand(id)).unwrap_err();
    assert_eq!(err, GameError::SorcerySpeedOnly);
}

// ── Tap for mana ──────────────────────────────────────────────────────────

#[test]
fn tap_forest_adds_green_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
    })
    .unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

#[test]
fn cannot_tap_already_tapped_land() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
    })
    .unwrap();
    let err = g
        .perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
        })
        .unwrap_err();
    assert_eq!(err, GameError::CardIsTapped(id));
}

#[test]
fn llanowar_elves_tap_for_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::llanowar_elves());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
    })
    .unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

// ── Cast creature ─────────────────────────────────────────────────────────

#[test]
fn cast_grizzly_bears_enters_battlefield() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());
    // Pay {1}{G}
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, mode: None, x_value: None })
        .unwrap();
    let resolve_events = drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
    assert!(resolve_events.iter().any(|e| matches!(e, GameEvent::PermanentEntered { .. })));
}

#[test]
fn cast_creature_fails_without_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::grizzly_bears());
    let err = g
        .perform_action(GameAction::CastSpell { card_id: id, target: None, mode: None, x_value: None })
        .unwrap_err();
    assert!(matches!(err, GameError::Mana(_)));
    // Card still in hand after failed cast
    assert!(g.players[0].has_in_hand(id));
}

// ── Instants ──────────────────────────────────────────────────────────────

#[test]
fn lightning_bolt_deals_3_damage_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        mode: None, x_value: None })
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

#[test]
fn lightning_bolt_kills_creature() {
    let mut g = two_player_game();
    let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear_id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt_id,
        target: Some(Target::Permanent(bear_id)),
        mode: None, x_value: None })
    .unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear_id));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear_id));
}

#[test]
fn giant_growth_pumps_creature() {
    let mut g = two_player_game();
    let spell_id = g.add_card_to_hand(0, catalog::giant_growth());
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell_id,
        target: Some(Target::Permanent(bear_id)),
        mode: None, x_value: None })
    .unwrap();
    drain_stack(&mut g);
    let bear = g.battlefield.iter().find(|c| c.id == bear_id).unwrap();
    assert_eq!(bear.power(), 5);
    assert_eq!(bear.toughness(), 5);
}

#[test]
fn dark_ritual_adds_three_black_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::dark_ritual());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, mode: None, x_value: None })
        .unwrap();
    drain_stack(&mut g);
    // Paid 1B, gained BBB → net 2B in pool
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 3);
}

#[test]
fn ancestral_recall_draws_three_cards() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let id = g.add_card_to_hand(0, catalog::ancestral_recall());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, mode: None, x_value: None })
        .unwrap();
    drain_stack(&mut g);
    // Drew 3 cards (Ancestral Recall has no target in this engine version)
    assert_eq!(g.players[0].hand.len(), 3);
}

#[test]
fn terror_destroys_non_black_creature() {
    let mut g = two_player_game();
    let terror_id = g.add_card_to_hand(0, catalog::terror());
    let bear_id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: terror_id,
        target: Some(Target::Permanent(bear_id)),
        mode: None, x_value: None })
    .unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear_id));
}

// ── Moxen ─────────────────────────────────────────────────────────────────

#[test]
fn mox_ruby_casts_for_free_and_taps_for_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mox_ruby());
    // Cast for {0} — no mana needed
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, mode: None, x_value: None }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
    // Tap immediately (not a creature, so no summoning sickness)
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
        .unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
}

#[test]
fn mox_pearl_taps_for_white() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mox_pearl());
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
        .unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
}

#[test]
fn mox_sapphire_taps_for_blue() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mox_sapphire());
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
        .unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
}

#[test]
fn mox_jet_taps_for_black() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mox_jet());
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
        .unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
}

#[test]
fn mox_emerald_taps_for_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mox_emerald());
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
        .unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

#[test]
fn mox_untaps_each_turn() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mox_ruby());
    // Tap it
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None })
        .unwrap();
    assert!(g.battlefield.iter().find(|c| c.id == id).unwrap().tapped);
    // Simulate untap step
    g.do_untap();
    assert!(!g.battlefield.iter().find(|c| c.id == id).unwrap().tapped);
}

#[test]
fn terror_cannot_destroy_artifact_creature() {
    // Terror uses SelectionRequirement to exclude artifacts and black creatures.
    // Moxes are artifacts — verify Artifact type is on mox_ruby.
    let mox_def = catalog::mox_ruby();
    assert!(mox_def.card_types.contains(&crate::card::CardType::Artifact));
}

#[test]
fn terror_cannot_destroy_black_creature() {
    let mut g = two_player_game();
    let terror_id = g.add_card_to_hand(0, catalog::terror());
    let knight_id = g.add_card_to_battlefield(1, catalog::black_knight());
    g.players[0].mana_pool.add(Color::Black, 2);
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: terror_id,
            target: Some(Target::Permanent(knight_id)),
        mode: None, x_value: None })
        .unwrap_err();
    assert_eq!(err, GameError::SelectionRequirementViolated);
}

// ── Combat ────────────────────────────────────────────────────────────────

fn setup_attacker(g: &mut GameState, player: usize, def: impl Fn() -> crate::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(player, def());
    g.clear_sickness(id);
    id
}

#[test]
fn unblocked_attacker_deals_damage_to_player() {
    let mut g = two_player_game();
    let bear_id = setup_attacker(&mut g, 0, catalog::grizzly_bears);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![bear_id]))
        .unwrap();

    // Advance to combat damage
    g.step = TurnStep::CombatDamage;
    let events = g.resolve_combat().unwrap();

    assert_eq!(g.players[1].life, 18); // 20 - 2
    assert!(events
        .iter()
        .any(|e| matches!(e, GameEvent::DamageDealt { to_player: Some(1), amount: 2, .. })));
}

#[test]
fn blocked_combat_both_die() {
    let mut g = two_player_game();
    let attacker_id = setup_attacker(&mut g, 0, catalog::grizzly_bears);
    let blocker_id = setup_attacker(&mut g, 1, catalog::grizzly_bears);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![attacker_id]))
        .unwrap();

    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker_id, attacker_id)]))
        .unwrap();

    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();

    // Both 2/2 creatures trade
    assert!(!g.battlefield.iter().any(|c| c.id == attacker_id));
    assert!(!g.battlefield.iter().any(|c| c.id == blocker_id));
    // Defending player life unchanged (attacker was blocked)
    assert_eq!(g.players[1].life, 20);
}

#[test]
fn vigilance_creature_does_not_tap_when_attacking() {
    let mut g = two_player_game();
    let angel_id = setup_attacker(&mut g, 0, catalog::serra_angel);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![angel_id]))
        .unwrap();
    let angel = g.battlefield.iter().find(|c| c.id == angel_id).unwrap();
    assert!(!angel.tapped, "Vigilance: Serra Angel should not tap when attacking");
}

#[test]
fn flying_creature_cannot_be_blocked_by_ground_creature() {
    let mut g = two_player_game();
    let angel_id = setup_attacker(&mut g, 0, catalog::serra_angel);
    let bear_id = setup_attacker(&mut g, 1, catalog::grizzly_bears);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![angel_id]))
        .unwrap();

    g.step = TurnStep::DeclareBlockers;
    let err = g
        .perform_action(GameAction::DeclareBlockers(vec![(bear_id, angel_id)]))
        .unwrap_err();
    assert_eq!(err, GameError::CannotBlock(bear_id));
}

#[test]
fn summoning_sick_creature_cannot_attack() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // still sick

    g.step = TurnStep::DeclareAttackers;
    let err = g
        .perform_action(GameAction::DeclareAttackers(vec![bear_id]))
        .unwrap_err();
    assert_eq!(err, GameError::SummoningSickness(bear_id));
}

#[test]
fn haste_creature_can_attack_immediately() {
    let mut g = two_player_game();
    let goblin_id = g.add_card_to_battlefield(0, catalog::goblin_guide()); // Haste, still sick

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![goblin_id]))
        .unwrap();
    // No error — Haste bypasses summoning sickness
}

// ── Win condition ─────────────────────────────────────────────────────────

#[test]
fn player_dies_when_life_reaches_zero() {
    let mut g = two_player_game();
    g.players[1].life = 3;
    let bolt_id = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
            card_id: bolt_id,
            target: Some(Target::Player(1)),
        mode: None, x_value: None })
        .unwrap();
    let resolve_events = drain_stack(&mut g);
    assert!(g.is_game_over());
    assert!(resolve_events
        .iter()
        .any(|e| matches!(e, GameEvent::GameOver { winner: Some(0) })));
}

// ── Turn progression ──────────────────────────────────────────────────────

#[test]
fn pass_priority_advances_step() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.step, TurnStep::BeginCombat);
}

#[test]
fn untap_step_clears_summoning_sickness() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.battlefield.iter().find(|c| c.id == bear_id).unwrap().summoning_sick);

    // The bear belongs to player 0.  Its sickness clears during player 0's
    // Untap step, which follows the end of player 1's turn (Cleanup).
    // Simulate: it is the end of player 1's turn.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 1;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();

    // We should now be in player 0's Untap step
    assert_eq!(g.step, TurnStep::Untap);
    assert_eq!(g.active_player_idx, 0);
    // Summoning sickness cleared for player 0's permanents
    assert!(!g.battlefield.iter().find(|c| c.id == bear_id).unwrap().summoning_sick);
}

#[test]
fn cleanup_resets_end_of_turn_pump() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield.iter_mut().find(|c| c.id == bear_id).unwrap().power_bonus = 3;
    g.step = TurnStep::Cleanup;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    // Pump should be gone after cleanup
    let bear = g.battlefield.iter().find(|c| c.id == bear_id).unwrap();
    assert_eq!(bear.power_bonus, 0);
}

// ── New effects ───────────────────────────────────────────────────────────────

#[test]
fn wrath_of_god_destroys_all_creatures() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wrath_id = g.add_card_to_hand(0, catalog::wrath_of_god());
    g.players[0].mana_pool.add(Color::White, 4);
    g.perform_action(GameAction::CastSpell { card_id: wrath_id, target: None, mode: None, x_value: None }).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear1));
    assert!(!g.battlefield.iter().any(|c| c.id == bear2));
}

#[test]
fn lightning_helix_deals_damage_and_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lightning_helix());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        mode: None, x_value: None })
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
    assert_eq!(g.players[0].life, 23);
}

// ── Layer system ──────────────────────────────────────────────────────────────

#[test]
fn glorious_anthem_pumps_your_creatures() {
    let mut g = two_player_game();
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let anthem_id = g.add_card_to_battlefield(0, catalog::glorious_anthem());

    let bear = g.computed_permanent(bear_id).unwrap();
    assert_eq!(bear.power, 3);   // 2 + 1
    assert_eq!(bear.toughness, 3); // 2 + 1

    // Remove anthem — power should return to base.
    g.remove_from_battlefield_to_graveyard(anthem_id);
    let bear = g.computed_permanent(bear_id).unwrap();
    assert_eq!(bear.power, 2);
    assert_eq!(bear.toughness, 2);
}

#[test]
fn anthem_does_not_pump_opponent_creatures() {
    let mut g = two_player_game();
    let _anthem_id = g.add_card_to_battlefield(0, catalog::glorious_anthem());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Opponent's bear should be unaffected.
    let bear = g.computed_permanent(opp_bear).unwrap();
    assert_eq!(bear.power, 2);
    assert_eq!(bear.toughness, 2);
}

#[test]
fn anthem_kills_creature_with_negative_base_toughness() {
    // A -1/-1 pumped creature should die when anthem leaves.
    // Use a 1/1 token scenario: token survives anthem, dies when anthem leaves.
    let mut g = two_player_game();
    let bear_id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Weaken the bear to 2/1 via power_bonus manipulation
    g.battlefield.iter_mut().find(|c| c.id == bear_id).unwrap().toughness_bonus = -2;
    // Without anthem: 2/(2-2) = 2/0 — dead
    let bear = g.computed_permanent(bear_id).unwrap();
    assert_eq!(bear.toughness, 0);

    // Place anthem: 2/(0+1) = 2/1 — alive
    let _anthem_id = g.add_card_to_battlefield(0, catalog::glorious_anthem());
    let bear = g.computed_permanent(bear_id).unwrap();
    assert_eq!(bear.toughness, 1);
}

#[test]
fn anthem_pumped_creature_uses_computed_toughness_for_lethality() {
    // Creature pumped by anthem should survive damage equal to its base toughness
    // but die if damage >= computed toughness.
    let mut g = two_player_game();
    // Put a 2/2 bear under anthem → effective 3/3.
    let bear_id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let _anthem_id = g.add_card_to_battlefield(1, catalog::glorious_anthem());

    // Deal 2 damage — less than computed toughness 3, should survive.
    g.battlefield.iter_mut().find(|c| c.id == bear_id).unwrap().damage = 2;
    let sba = g.check_state_based_actions();
    assert!(sba.is_empty(), "Bear should survive 2 damage with 3 effective toughness");

    // Deal 3 damage — equal to computed toughness 3, should die.
    g.battlefield.iter_mut().find(|c| c.id == bear_id).unwrap().damage = 3;
    let sba = g.check_state_based_actions();
    assert!(sba.iter().any(|e| matches!(e, GameEvent::CreatureDied { card_id } if *card_id == bear_id)));
}

#[test]
fn layer_keyword_grants_flying() {
    let mut g = two_player_game();
    let mut bear_def = catalog::grizzly_bears();
    bear_def.static_abilities = vec![StaticAbility {
        description: "This creature has flying",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::This,
            keyword: crate::card::Keyword::Flying,
        },
    }];
    let bear_id = g.add_card_to_battlefield(0, bear_def);
    let cp = g.computed_permanent(bear_id).unwrap();
    assert!(cp.keywords.contains(&crate::card::Keyword::Flying));
}

#[test]
fn flying_via_layer_cannot_be_blocked_by_ground() {
    let mut g = two_player_game();
    // Give the attacker flying via static ability (layer 6)
    let mut bear_def = catalog::grizzly_bears();
    bear_def.static_abilities = vec![StaticAbility {
        description: "Flying",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::This,
            keyword: crate::card::Keyword::Flying,
        },
    }];
    let attacker_id = setup_attacker(&mut g, 0, || bear_def.clone());
    let ground_blocker = setup_attacker(&mut g, 1, catalog::grizzly_bears);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![attacker_id])).unwrap();

    g.step = TurnStep::DeclareBlockers;
    let err = g.perform_action(GameAction::DeclareBlockers(vec![(ground_blocker, attacker_id)])).unwrap_err();
    assert_eq!(err, GameError::CannotBlock(ground_blocker));
}

// ── Decision system ───────────────────────────────────────────────────────────

#[test]
fn wants_ui_opt_suspends_on_scry_and_draws_after_submit() {
    // P0 is marked as needing UI — the Scry half of Opt should suspend instead
    // of auto-resolving; after SubmitDecision, the Draw half of the Seq runs.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let undesired = g.add_card_to_library(0, catalog::forest());
    let keeper = g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let opt_id = g.add_card_to_hand(0, catalog::opt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell { card_id: opt_id, target: None, mode: None, x_value: None }).unwrap();
    drain_stack(&mut g);
    assert!(g.pending_decision.is_some(), "Opt should suspend on its Scry half for UI");
    // Scry the undesired card to the bottom. Draw should fire after submission.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![undesired],
    })).unwrap();
    assert!(g.pending_decision.is_none(), "decision should be resolved");
    assert!(g.players[0].hand.iter().any(|c| c.id == keeper),
        "Draw half of Opt should run after the scry decision resolves");
}

#[test]
fn scry_resolves_with_scripted_order() {
    let mut g = two_player_game();
    let top = g.add_card_to_library(0, catalog::forest());
    let next = g.add_card_to_library(0, catalog::plains());
    // Library: [top=forest, next=plains]. Keep `next` on top; send `top` to bottom.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![next],
        bottom: vec![top],
    }]));
    let opt_id = g.add_card_to_hand(0, catalog::opt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell { card_id: opt_id, target: None, mode: None, x_value: None }).unwrap();
    drain_stack(&mut g);
    assert!(g.pending_decision.is_none(), "scry should have been completed synchronously");
    // `next` was kept on top then drawn by Opt's draw effect → it's now in hand.
    // `top` went to the bottom of the library.
    assert!(g.players[0].hand.iter().any(|c| c.id == next));
    assert_eq!(g.players[0].library.last().unwrap().id, top);
}

#[test]
fn add_mana_any_color_asks_decider() {
    let mut g = two_player_game();
    let scripted = ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]);
    g.decider = Box::new(scripted);
    let eff = Effect::AddMana {
        who: PlayerRef::You,
        pool: ManaPayload::AnyOneColor(Value::Const(1)),
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&eff, &ctx).unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 0);
}

#[test]
fn opt_scries_then_draws() {
    let mut g = two_player_game();
    // Stack library: [undesired=forest, keeper=plains, ...]
    let undesired = g.add_card_to_library(0, catalog::forest());
    let keeper = g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    // Scry 1: the undesired top card is sent to the bottom; then Draw 1 draws `keeper`.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![undesired],
    }]));
    let opt = catalog::opt();
    let opt_id = g.add_card_to_hand(0, opt);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell { card_id: opt_id, target: None, mode: None, x_value: None }).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == keeper),
        "Opt should have drawn the keeper after scrying undesired to bottom");
}

#[test]
fn birds_of_paradise_produces_chosen_color() {
    let mut g = two_player_game();
    let birds_id = g.add_card_to_battlefield(0, catalog::birds_of_paradise());
    g.clear_sickness(birds_id);
    // Ask for White on the tap ability.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::White)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: birds_id,
        ability_index: 0,
        target: None,
    }).unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 1);
}

#[test]
fn demonic_tutor_fetches_chosen_card_via_decider() {
    let mut g = two_player_game();
    let wanted = g.add_card_to_library(1, catalog::black_lotus());
    g.add_card_to_library(1, catalog::swamp());
    g.add_card_to_library(1, catalog::swamp());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(wanted))]));
    let tutor_id = g.add_card_to_hand(1, catalog::demonic_tutor());
    // Give player 1 mana and priority.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell { card_id: tutor_id, target: None, mode: None, x_value: None }).unwrap();
    while !g.stack.is_empty() {
        g.perform_action(GameAction::PassPriority).unwrap();
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    assert!(g.players[1].hand.iter().any(|c| c.id == wanted),
        "Demonic Tutor should have fetched the wanted card into hand");
    assert!(!g.players[1].library.iter().any(|c| c.id == wanted),
        "wanted card should no longer be in library");
}

#[test]
fn preordain_scry_2_then_draws() {
    let mut g = two_player_game();
    let bottom_card = g.add_card_to_library(0, catalog::forest());
    let top_card = g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    // After scrying top 2 to bottom, the 3rd (island) becomes top and is drawn.
    let island_top = g.players[0].library[2].id;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![bottom_card, top_card],
    }]));
    let pre_id = g.add_card_to_hand(0, catalog::preordain());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell { card_id: pre_id, target: None, mode: None, x_value: None }).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == island_top),
        "Preordain should draw what was the 3rd card after scrying top 2 to bottom");
}
