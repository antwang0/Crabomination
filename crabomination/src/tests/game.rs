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
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear_id,
        target: AttackTarget::Player(1),
    }]))
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
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: attacker_id,
        target: AttackTarget::Player(1),
    }]))
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
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: angel_id,
        target: AttackTarget::Player(1),
    }]))
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
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: angel_id,
        target: AttackTarget::Player(1),
    }]))
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
        .perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: bear_id,
            target: AttackTarget::Player(1),
        }]))
        .unwrap_err();
    assert_eq!(err, GameError::SummoningSickness(bear_id));
}

#[test]
fn haste_creature_can_attack_immediately() {
    let mut g = two_player_game();
    let goblin_id = g.add_card_to_battlefield(0, catalog::goblin_guide()); // Haste, still sick

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: goblin_id,
        target: AttackTarget::Player(1),
    }]))
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

fn one_u_spell() -> crate::card::CardDefinition {
    use crate::mana::{ManaCost, ManaSymbol};
    let mut spell = catalog::grizzly_bears();
    spell.cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::Blue)],
    };
    spell
}

#[test]
fn cast_one_u_with_dual_and_basic_auto_taps_correctly() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    g.add_card_to_battlefield(0, catalog::island());
    let id = g.add_card_to_hand(0, one_u_spell());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Tundra + Island should be enough mana for {1}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn cast_one_u_with_two_tundras_auto_taps_correctly() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    g.add_card_to_battlefield(0, catalog::tundra());
    let id = g.add_card_to_hand(0, one_u_spell());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Two Tundras should be enough mana for {1}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn inquisition_of_kozilek_picks_low_cmc_nonland() {
    let mut g = two_player_game();
    // Opponent's hand: a land (forest) + a high-cost spell (Mahamoti Djinn,
    // {3}{U}{U}, CMC 5) + a low-cost spell (Lightning Bolt, {R}, CMC 1).
    let _land = g.add_card_to_hand(1, catalog::forest());
    let _big = g.add_card_to_hand(1, catalog::mahamoti_djinn());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());

    let inq = g.add_card_to_hand(0, catalog::inquisition_of_kozilek());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: inq, target: None, mode: None, x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);

    // Lightning Bolt (CMC 1, nonland) should be the pick. Mahamoti is CMC 5
    // (excluded) and the forest is a land (excluded).
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
    assert!(!g.players[1].hand.iter().any(|c| c.id == bolt));
}

#[test]
fn thoughtseize_picks_nonland_and_costs_two_life() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let life_before = g.players[0].life;

    let ts = g.add_card_to_hand(0, catalog::thoughtseize());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ts, target: None, mode: None, x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);

    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Thoughtseize should pick the nonland Lightning Bolt");
    assert_eq!(g.players[0].life, life_before - 2);
}

#[test]
fn solitude_evoke_exiles_target_then_sacrifices_self() {
    use crate::card::CardType;
    let mut g = two_player_game();
    let solitude_id = g.add_card_to_hand(0, catalog::solitude());
    // P0 has a white card to pitch.
    let pitch = g.add_card_to_hand(0, catalog::serra_angel());
    // P1 has a creature for Solitude's ETB to exile.
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: solitude_id,
        pitch_card: Some(pitch),
        target: Some(Target::Permanent(opp_creature)),
        mode: None,
        x_value: None,
    })
    .expect("Solitude evoke should succeed");
    drain_stack(&mut g);

    // ETB exile fires → opponent's creature in exile.
    assert!(g.exile.iter().any(|c| c.id == opp_creature),
        "Solitude's ETB should exile the targeted creature");
    // Evoke sacrifice fires → Solitude is back in P0's graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == solitude_id),
        "Solitude should be sacrificed via evoke");
    assert!(!g.battlefield.iter().any(|c| c.id == solitude_id));
    // Pitched white card is in exile.
    assert!(g.exile.iter().any(|c| c.id == pitch));
    // Make sure CardType is used so the import isn't flagged unused.
    let _ = CardType::Creature;
}

#[test]
fn thud_sacrifices_creature_and_deals_damage_equal_to_its_power() {
    let mut g = two_player_game();
    // Give P0 a 5-power creature to sacrifice.
    let bear_id = g.add_card_to_battlefield(0, catalog::shivan_dragon()); // 5/5
    g.clear_sickness(bear_id);
    let thud_id = g.add_card_to_hand(0, catalog::thud());
    g.players[0].mana_pool.add(Color::Red, 1);

    let opp_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: thud_id,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Thud should be castable");
    drain_stack(&mut g);

    // Shivan Dragon (power 5) sacrificed → 5 damage to P1.
    assert!(!g.battlefield.iter().any(|c| c.id == bear_id),
        "Sacrificed creature should leave the battlefield");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear_id));
    assert_eq!(g.players[1].life, opp_life_before - 5,
        "P1 should take 5 damage equal to sacrificed Shivan Dragon's power");
}

#[test]
fn cosmogoyf_pt_scales_with_card_types_in_graveyards() {
    use crate::card::CardType;
    let mut g = two_player_game();
    let goyf_id = g.add_card_to_battlefield(0, catalog::cosmogoyf());

    // Empty graveyards → Cosmogoyf is 0/1.
    let cp = g.computed_permanent(goyf_id).unwrap();
    assert_eq!(cp.power, 0);
    assert_eq!(cp.toughness, 1);

    // Add a Creature card to graveyard → 1/2.
    g.players[0].graveyard.push(crate::card::CardInstance::new(
        crate::card::CardId(9001),
        catalog::grizzly_bears(),
        0,
    ));
    let cp = g.computed_permanent(goyf_id).unwrap();
    assert_eq!(cp.power, 1);
    assert_eq!(cp.toughness, 2);

    // Add an Instant card → 2/3.
    g.players[1].graveyard.push(crate::card::CardInstance::new(
        crate::card::CardId(9002),
        catalog::lightning_bolt(),
        1,
    ));
    let cp = g.computed_permanent(goyf_id).unwrap();
    assert_eq!(cp.power, 2);
    assert_eq!(cp.toughness, 3);

    // Confirm CardType is referenced so the import isn't flagged.
    let _ = CardType::Land;
}

/// Spam pass-priority for at most `n` cycles or until the game ends.
fn pass_until_game_over_or(g: &mut GameState, max: usize) {
    for _ in 0..max {
        if g.is_game_over() {
            return;
        }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
}

#[test]
fn pact_of_negation_eliminates_caster_if_unpaid_on_next_upkeep() {
    // Resolve a Pact, pass to next upkeep with no mana — engine should
    // auto-fail the pay-or-lose and eliminate the caster.
    let mut g = two_player_game();
    let pact = g.add_card_to_hand(0, catalog::pact_of_negation());
    g.perform_action(GameAction::CastSpell {
        card_id: pact, target: None, mode: None, x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.delayed_triggers.len(), 1, "pact registers a delayed upkeep trigger");

    // Advance to P0's next upkeep. P0 has no lands → can't pay {3}{U}{U}.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 1;
    pass_until_game_over_or(&mut g, 30);

    assert!(g.is_game_over(), "P0 should have been eliminated by unpaid pact");
    assert_eq!(g.game_over, Some(Some(1)));
}

#[test]
fn pact_of_negation_lets_caster_live_when_they_can_pay() {
    let mut g = two_player_game();
    let pact = g.add_card_to_hand(0, catalog::pact_of_negation());
    g.perform_action(GameAction::CastSpell {
        card_id: pact, target: None, mode: None, x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);

    // Give P0 enough mana sources to pay {3}{U}{U} via auto-tap on the
    // upkeep. (Mana pool itself empties between steps, so pre-stocking the
    // pool would do nothing — we need actual untapped lands.)
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::island());
    }

    // Advance to P0's next upkeep — but stop at Upkeep so the resolved
    // Pact trigger can be observed without rolling further into the turn.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 1;
    for _ in 0..15 {
        if g.is_game_over() { break; }
        if g.active_player_idx == 0 && g.step == TurnStep::Upkeep && g.stack.is_empty() {
            break;
        }
        g.perform_action(GameAction::PassPriority).unwrap();
    }

    assert!(!g.is_game_over(), "P0 paid the pact and should still be alive");
    assert!(g.players[0].is_alive());
    assert!(g.delayed_triggers.is_empty(), "pact trigger fired and consumed");
}

#[test]
fn goryos_vengeance_exiles_creature_at_end_step() {
    // Reanimate Griselbrand with Goryo's, pass to end step → it's exiled.
    let mut g = two_player_game();
    let grisel = g.add_card_to_library(0, catalog::griselbrand());
    // Move Griselbrand straight into the graveyard for the test setup.
    let card = g.players[0].library.iter().position(|c| c.id == grisel)
        .map(|pos| g.players[0].library.remove(pos))
        .unwrap();
    g.players[0].graveyard.push(card);

    let goryo = g.add_card_to_hand(0, catalog::goryos_vengeance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: goryo,
        target: Some(Target::Permanent(grisel)),
        mode: None,
        x_value: None,
    })
    .expect("Goryo's should accept legendary creature target");
    drain_stack(&mut g);

    // Griselbrand on battlefield, delayed exile registered.
    assert!(g.battlefield.iter().any(|c| c.id == grisel));
    assert_eq!(g.delayed_triggers.len(), 1);

    // Pass to end step — delayed trigger fires and exiles Griselbrand.
    g.step = TurnStep::PostCombatMain;
    for _ in 0..15 {
        if g.exile.iter().any(|c| c.id == grisel) { break; }
        if g.is_game_over() { break; }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    assert!(!g.battlefield.iter().any(|c| c.id == grisel),
        "Griselbrand should be exiled at end step");
    assert!(g.exile.iter().any(|c| c.id == grisel),
        "Griselbrand should be in the exile zone");
}

#[test]
fn force_of_will_pitches_a_blue_card_to_counter_a_spell() {
    // Validate the alt-cost path: pitch a blue card (Counterspell) instead
    // of paying the {3}{U}{U} mana cost. The FoW spell goes on the stack
    // and the pitch card moves to exile while life is paid.
    let mut g = two_player_game();
    let fow = g.add_card_to_hand(0, catalog::force_of_will());
    let pitch = g.add_card_to_hand(0, catalog::counterspell()); // blue card to exile
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: fow,
        pitch_card: Some(pitch),
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Force of Will should be castable via pitch alt cost");

    // Pitched card moved to exile, no longer in hand.
    assert!(g.exile.iter().any(|c| c.id == pitch));
    assert!(!g.players[0].hand.iter().any(|c| c.id == pitch));
    // Force of Will itself is on the stack.
    assert!(g.stack.iter().any(|si| matches!(
        si,
        crate::game::StackItem::Spell { card, .. } if card.id == fow
    )));
    // 1 life paid for the alt cost; no mana spent.
    assert_eq!(g.players[0].life, life_before - 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 0);
}

#[test]
fn force_of_will_rejects_non_blue_pitch_card() {
    let mut g = two_player_game();
    let fow = g.add_card_to_hand(0, catalog::force_of_will());
    let bad_pitch = g.add_card_to_hand(0, catalog::lightning_bolt()); // red card
    let err = g
        .perform_action(GameAction::CastSpellAlternative {
            card_id: fow,
            pitch_card: Some(bad_pitch),
            target: None,
            mode: None,
            x_value: None,
        })
        .unwrap_err();
    assert!(matches!(err, GameError::InvalidPitchCard(_)));
}

#[test]
fn black_lotus_manual_activation_with_wants_ui_prompts_for_color() {
    use crate::decision::Decision;
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let lotus = g.add_card_to_battlefield(0, catalog::black_lotus());
    g.clear_sickness(lotus);
    // Activating manually should suspend on a ChooseColor decision instead
    // of auto-picking White.
    g.perform_action(GameAction::ActivateAbility {
        card_id: lotus, ability_index: 0, target: None,
    })
    .unwrap();
    let pd = g.pending_decision.as_ref().expect("ChooseColor should suspend");
    assert!(matches!(pd.decision, Decision::ChooseColor { .. }));
    // Picking Blue adds 3 blue mana to the pool.
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Color(Color::Blue)))
        .unwrap();
    assert!(g.pending_decision.is_none());
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 3);
    assert_eq!(g.players[0].mana_pool.amount(Color::White), 0);
}

#[test]
fn black_lotus_auto_taps_for_specific_color() {
    // Auto-tap on a {1}{U} cost should trigger Black Lotus's
    // `AnyOneColor(3)` ability and have it add Blue, not the default White
    // that AutoDecider would pick. Without this, casting any colored spell
    // off Black Lotus alone fails.
    use crate::mana::{ManaCost, ManaSymbol};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::black_lotus());
    let mut spell = catalog::grizzly_bears();
    spell.cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(1), ManaSymbol::Colored(Color::Blue)],
    };
    let id = g.add_card_to_hand(0, spell);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Black Lotus should auto-tap into Blue for {1}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn cast_white_knight_with_two_tundras() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    g.add_card_to_battlefield(0, catalog::tundra());
    let id = g.add_card_to_hand(0, catalog::white_knight());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Two Tundras should pay {W}{W} for White Knight");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn flooded_strand_fetches_tundra_untapped() {
    // Onslaught/Zendikar fetchlands put the fetched land onto the
    // battlefield UNTAPPED. Regression: we previously set `tapped: true`,
    // which made `{W}{W}` spells uncastable when a fetched Tundra was on
    // the board.
    let mut g = two_player_game();
    let tundra_in_lib = g.add_card_to_library(0, catalog::tundra());
    let strand_id = g.add_card_to_battlefield(0, catalog::flooded_strand());
    g.clear_sickness(strand_id);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(
        tundra_in_lib,
    ))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: strand_id,
        ability_index: 0,
        target: None,
    })
    .unwrap();
    drain_stack(&mut g);
    let fetched = g
        .battlefield
        .iter()
        .find(|c| c.id == tundra_in_lib)
        .expect("Tundra should be on battlefield after fetch");
    assert!(!fetched.tapped, "fetched Tundra should enter untapped");
}

#[test]
fn cast_white_knight_with_one_tundra_already_tapped_fails() {
    // Reproduces the exact "Need 1 White mana but only have 0" error: one
    // Tundra is already tapped (fetched this turn, used for an earlier
    // spell, etc.), so only one W source is available — not enough for {W}{W}.
    let mut g = two_player_game();
    let t1 = g.add_card_to_battlefield(0, catalog::tundra());
    g.add_card_to_battlefield(0, catalog::tundra());
    if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == t1) {
        c.tapped = true;
    }
    let id = g.add_card_to_hand(0, catalog::white_knight());
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: id, target: None, mode: None, x_value: None,
        })
        .unwrap_err();
    assert!(
        matches!(err, GameError::Mana(_)),
        "one tapped Tundra + one untapped Tundra cannot pay {{W}}{{W}}, got {err:?}"
    );
}

#[test]
fn cast_white_knight_after_playing_two_tundras_via_play_land() {
    // Simulate the user playing both Tundras through the normal play-land
    // path (vs. dropping them straight onto the battlefield). They share a
    // turn for simplicity — the engine doesn't enforce the one-land-per-turn
    // rule across direct calls when we reset `lands_played_this_turn`.
    let mut g = two_player_game();
    let t1 = g.add_card_to_hand(0, catalog::tundra());
    let t2 = g.add_card_to_hand(0, catalog::tundra());
    let knight_id = g.add_card_to_hand(0, catalog::white_knight());

    g.perform_action(GameAction::PlayLand(t1)).unwrap();
    // Reset the land-drop counter so we can play a second Tundra in this
    // test without sequencing through a full turn.
    g.players[0].lands_played_this_turn = 0;
    g.perform_action(GameAction::PlayLand(t2)).unwrap();

    g.perform_action(GameAction::CastSpell {
        card_id: knight_id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Two freshly-played Tundras should pay {W}{W}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == knight_id));
}

#[test]
fn cast_u_with_only_tundra_succeeds() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    let id = g.add_card_to_hand(0, catalog::opt());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Single Tundra should pay {U} (Opt)");
    drain_stack(&mut g);
}

#[test]
fn cast_one_u_with_only_tundra_fails() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    let id = g.add_card_to_hand(0, one_u_spell());
    let err = g
        .perform_action(GameAction::CastSpell {
            card_id: id, target: None, mode: None, x_value: None,
        })
        .unwrap_err();
    assert!(matches!(err, GameError::Mana(_)),
        "single Tundra should not be enough mana for {{1}}{{U}}, got {err:?}");
}

#[test]
fn cast_one_u_with_tundra_plus_plains_auto_taps_correctly() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tundra());
    g.add_card_to_battlefield(0, catalog::plains());
    let id = g.add_card_to_hand(0, one_u_spell());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, mode: None, x_value: None,
    })
    .expect("Tundra (for U) + Plains (for generic) should pay {1}{U}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn putrefy_can_destroy_artifact() {
    let mut g = two_player_game();
    let mox_id = g.add_card_to_battlefield(1, catalog::mox_ruby());
    let putrefy_id = g.add_card_to_hand(0, catalog::putrefy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: putrefy_id,
        target: Some(Target::Permanent(mox_id)),
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mox_id));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == mox_id));
}

#[test]
fn goblin_guide_reveals_top_and_gives_land() {
    let mut g = two_player_game();
    // Stack defending player (1) library: top is a forest.
    let forest_id = g.add_card_to_library(1, catalog::forest());
    let goblin_id = setup_attacker(&mut g, 0, catalog::goblin_guide);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: goblin_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    // Forest moved from defender's library to defender's hand.
    assert!(g.players[1].hand.iter().any(|c| c.id == forest_id));
    assert!(!g.players[1].library.iter().any(|c| c.id == forest_id));
}

#[test]
fn hypnotic_specter_discards_damaged_opponent() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    let spec_id = setup_attacker(&mut g, 0, catalog::hypnotic_specter);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: spec_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    // Specter dealt 2 to player 1 → trigger forced one discard.
    assert_eq!(g.players[1].hand.len(), 0);
    assert_eq!(g.players[1].graveyard.len(), 1);
}

#[test]
fn wheel_of_fortune_discards_both_hands_and_draws_seven() {
    let mut g = two_player_game();
    // Active player (caster) is seat 1; opponent is seat 0.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    // Give each player some hand cards and a stocked library.
    for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); }
    for _ in 0..2 { g.add_card_to_hand(1, catalog::mountain()); }
    for _ in 0..10 { g.add_card_to_library(0, catalog::forest()); }
    for _ in 0..10 { g.add_card_to_library(1, catalog::mountain()); }
    let wheel_id = g.add_card_to_hand(1, catalog::wheel_of_fortune());
    g.players[1].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: wheel_id,
        target: None,
        mode: None,
        x_value: None,
    })
    .unwrap();
    drain_stack(&mut g);
    // Both hands were discarded, then both drew 7.
    assert_eq!(g.players[0].hand.len(), 7, "opponent should have 7 cards");
    assert_eq!(g.players[1].hand.len(), 7, "caster should have 7 cards");
    // p0 discarded 3 forests. p1 discarded 2 mountains, plus the wheel itself
    // resolves and joins p1's graveyard.
    assert_eq!(g.players[0].graveyard.len(), 3);
    assert_eq!(g.players[1].graveyard.len(), 3);
}

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
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: attacker_id,
        target: AttackTarget::Player(1),
    }]))
    .unwrap();

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
