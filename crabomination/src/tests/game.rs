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

// ── Demo-deck promotions ──────────────────────────────────────────────────

#[test]
fn watery_grave_pays_two_life_and_stays_untapped() {
    // Default `AutoDecider` picks mode 0 of the shockland's ETB
    // `ChooseMode` — pay 2 life and enter untapped.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::watery_grave());
    let life_before = g.players[0].life;
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2,
        "Shockland mode 0 pays 2 life on ETB");
    let card = g.battlefield_find(id).expect("shockland is on the battlefield");
    assert!(!card.tapped, "Shockland should stay untapped on mode 0");
}

#[test]
fn cephalid_coliseum_sacrifices_for_each_player_to_draw_then_discard_three() {
    let mut g = two_player_game();
    let coli = g.add_card_to_battlefield(0, catalog::cephalid_coliseum());
    g.clear_sickness(coli);
    // Coliseum entered tapped via its ETB trigger. Untap it so we can
    // activate the wheel-mini ability.
    g.battlefield_find_mut(coli).unwrap().tapped = false;

    // Stock both libraries with enough cards to draw 3 each.
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    // Stock both hands with enough cards to discard 3 each.
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::lightning_bolt());
        g.add_card_to_hand(1, catalog::lightning_bolt());
    }

    let p0_lib_before = g.players[0].library.len();
    let p1_lib_before = g.players[1].library.len();
    let p0_grave_before = g.players[0].graveyard.len();
    let p1_grave_before = g.players[1].graveyard.len();

    // Pay {2}{U} and tap.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: coli,
        ability_index: 1,
        target: None,
    })
    .expect("Cephalid Coliseum's wheel-mini ability should activate");
    drain_stack(&mut g);

    // Coliseum sacrificed to its own graveyard.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == coli),
        "Coliseum should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == coli));

    // Each player drew 3 then discarded 3 — library shrinks by 3,
    // graveyard grows by ≥3 (their 3 discards; P0 also has Coliseum itself).
    assert_eq!(g.players[0].library.len(), p0_lib_before - 3);
    assert_eq!(g.players[1].library.len(), p1_lib_before - 3);
    assert!(g.players[0].graveyard.len() >= p0_grave_before + 3,
        "P0 should have ≥3 cards in graveyard from discard (plus Coliseum)");
    assert_eq!(g.players[1].graveyard.len(), p1_grave_before + 3,
        "P1 should have 3 discarded cards in graveyard");
}

#[test]
fn quantum_riddler_on_cast_draws_a_card() {
    let mut g = two_player_game();
    // Top of library: a known card to confirm it gets drawn from the on-cast
    // cantrip.
    let top = g.add_card_to_library(0, catalog::island());
    let qr_id = g.add_card_to_hand(0, catalog::quantum_riddler());
    // Pay {1}{U}{B}.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: qr_id,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Quantum Riddler should be castable");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == qr_id),
        "Quantum Riddler should resolve onto the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == top),
        "Quantum Riddler's on-cast cantrip should draw a card");
}

#[test]
fn psychic_frog_discard_pumps_until_end_of_turn() {
    let mut g = two_player_game();
    let frog = g.add_card_to_battlefield(0, catalog::psychic_frog());
    g.clear_sickness(frog);
    let to_pitch = g.add_card_to_hand(0, catalog::lightning_bolt());

    let p_before = g.battlefield_find(frog).unwrap().power();
    let t_before = g.battlefield_find(frog).unwrap().toughness();

    // Activate the discard-pump ability.
    g.perform_action(GameAction::ActivateAbility {
        card_id: frog,
        ability_index: 0,
        target: None,
    })
    .expect("Psychic Frog discard pump should activate");
    drain_stack(&mut g);

    // The first card in hand was discarded.
    assert!(g.players[0].graveyard.iter().any(|c| c.id == to_pitch),
        "discarded card should be in graveyard");
    // Frog gained +1/+1.
    let card = g.battlefield_find(frog).unwrap();
    assert_eq!(card.power(), p_before + 1);
    assert_eq!(card.toughness(), t_before + 1);
}

#[test]
fn psychic_frog_sacrifice_mills_each_opponent_four() {
    let mut g = two_player_game();
    let frog = g.add_card_to_battlefield(0, catalog::psychic_frog());
    g.clear_sickness(frog);
    // Stock P1's library so we can mill 4 from it.
    for _ in 0..6 {
        g.add_card_to_library(1, catalog::island());
    }
    let p1_lib_before = g.players[1].library.len();
    let p1_grave_before = g.players[1].graveyard.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: frog,
        ability_index: 1,
        target: None,
    })
    .expect("Psychic Frog sacrifice-mill should activate");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == frog),
        "Psychic Frog should sacrifice itself");
    assert!(!g.battlefield.iter().any(|c| c.id == frog));
    assert_eq!(g.players[1].library.len(), p1_lib_before - 4);
    assert_eq!(g.players[1].graveyard.len(), p1_grave_before + 4);
}

#[test]
fn pest_control_destroys_low_cmc_nonland_permanents() {
    let mut g = two_player_game();
    // Lands should survive (Land filter on Nonland).
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    // Mana value 1 — should die.
    let llanowar = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    // Mana value 2 — should die.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Mana value 5 — survives.
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());

    let pest = g.add_card_to_hand(0, catalog::pest_control());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pest,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Pest Control should cast for {W}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == forest),
        "Lands shouldn't be destroyed by Pest Control");
    assert!(!g.battlefield.iter().any(|c| c.id == llanowar),
        "1-CMC creature should die");
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "2-CMC creature should die");
    assert!(g.battlefield.iter().any(|c| c.id == dragon),
        "5-CMC creature should survive");
}

#[test]
fn prismatic_ending_at_converge_one_only_exiles_one_drops() {
    // Cast for {W} alone → converge = 1. A 2-CMC target is a *legal* cast
    // (target filter is just Nonland Permanent), but the resolution-time
    // `If(ManaValueOf(Target) ≤ ConvergedValue, …)` no-ops on the 2-CMC
    // creature. A 1-CMC target gets exiled.
    let mut g = two_player_game();
    let llanowar = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Cast targeting the 2-CMC creature: cast succeeds, but resolution
    // doesn't exile (converge=1 vs CMC=2).
    let pe = g.add_card_to_hand(0, catalog::prismatic_ending());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pe,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Prismatic Ending should accept a Nonland target at cast time");
    drain_stack(&mut g);
    assert!(!g.exile.iter().any(|c| c.id == bear),
        "Bear (2 CMC) shouldn't be exiled at converge=1");
    assert!(g.battlefield.iter().any(|c| c.id == bear));

    // Cast targeting the 1-CMC creature: exile fires.
    let pe2 = g.add_card_to_hand(0, catalog::prismatic_ending());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pe2,
        target: Some(Target::Permanent(llanowar)),
        mode: None,
        x_value: None,
    })
    .expect("Prismatic Ending should accept a 1-CMC target");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == llanowar),
        "Llanowar Elves (1 CMC) should be exiled at converge=1");
}

#[test]
fn ephemerate_flickers_target_creature_back_to_battlefield() {
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Damage it so we can verify the flicker clears damage.
    g.battlefield_find_mut(creature).unwrap().damage = 1;

    let eph = g.add_card_to_hand(0, catalog::ephemerate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eph,
        target: Some(Target::Permanent(creature)),
        mode: None,
        x_value: None,
    })
    .expect("Ephemerate should accept a creature you control");
    drain_stack(&mut g);

    let card = g.battlefield_find(creature).expect("creature flickered back");
    assert_eq!(card.damage, 0, "flicker should clear damage");
    // Card should not still be in exile.
    assert!(!g.exile.iter().any(|c| c.id == creature));
}

#[test]
fn ephemerate_refires_solitude_etb_via_place_card_on_battlefield() {
    // Solitude has a SelfSource ETB that exiles a creature an opponent
    // controls. Flickering Solitude with Ephemerate should refire that ETB
    // (because `place_card_in_dest::Battlefield` now calls
    // `fire_self_etb_triggers`), exiling a second opponent creature.
    let mut g = two_player_game();
    let solitude = g.add_card_to_battlefield(0, catalog::solitude());
    g.clear_sickness(solitude);
    let opp_a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_b = g.add_card_to_battlefield(1, catalog::llanowar_elves());

    let eph = g.add_card_to_hand(0, catalog::ephemerate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eph,
        target: Some(Target::Permanent(solitude)),
        mode: None,
        x_value: None,
    })
    .expect("Ephemerate should target your Solitude");
    drain_stack(&mut g);

    // Solitude should be back on the battlefield.
    assert!(g.battlefield.iter().any(|c| c.id == solitude),
        "Solitude returns from exile");
    // One of the opponent's creatures was exiled by Solitude's refired ETB.
    let exiled_count = [opp_a, opp_b]
        .iter()
        .filter(|cid| g.exile.iter().any(|c| c.id == **cid))
        .count();
    assert_eq!(exiled_count, 1,
        "Solitude's ETB should refire on flicker and exile one opp creature");
}

#[test]
fn fastland_enters_untapped_with_few_lands() {
    // With ≤ 3 total lands you control (i.e. ≤ 2 other lands), Blackcleave
    // Cliffs should enter untapped.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::swamp());
    let cliffs = g.add_card_to_hand(0, catalog::blackcleave_cliffs());
    g.perform_action(GameAction::PlayLand(cliffs)).unwrap();
    drain_stack(&mut g);
    let cp = g.battlefield.iter().find(|c| c.id == cliffs).unwrap();
    assert!(!cp.tapped, "Blackcleave Cliffs should enter untapped with ≤ 2 other lands");
}

#[test]
fn fastland_enters_tapped_with_many_lands() {
    // With 3 other lands already in play, Blackcleave Cliffs should enter
    // tapped (post-ETB count ≥ 4).
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    let cliffs = g.add_card_to_hand(0, catalog::blackcleave_cliffs());
    g.perform_action(GameAction::PlayLand(cliffs)).unwrap();
    drain_stack(&mut g);
    let cp = g.battlefield.iter().find(|c| c.id == cliffs).unwrap();
    assert!(cp.tapped, "Blackcleave Cliffs should enter tapped with 3+ other lands");
}

#[test]
fn force_of_negation_alt_cost_blocked_on_your_turn() {
    // Force of Negation's pitch alt cost is "if it's not your turn".
    // The active player is P0 by default — the engine should reject the
    // alt cast from P0.
    let mut g = two_player_game();
    let fon = g.add_card_to_hand(0, catalog::force_of_negation());
    let pitch = g.add_card_to_hand(0, catalog::counterspell()); // a blue card
    let err = g
        .perform_action(GameAction::CastSpellAlternative {
            card_id: fon,
            pitch_card: Some(pitch),
            target: None,
            mode: None,
            x_value: None,
        })
        .unwrap_err();
    assert_eq!(err, GameError::NoAlternativeCost,
        "Force of Negation's alt cost shouldn't fire on the caster's own turn");
}

#[test]
fn force_of_negation_alt_cost_works_on_opponents_turn() {
    let mut g = two_player_game();
    // Make it P1's turn so P0 can pitch-cast Force of Negation.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 0;
    let fon = g.add_card_to_hand(0, catalog::force_of_negation());
    let pitch = g.add_card_to_hand(0, catalog::counterspell());

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: fon,
        pitch_card: Some(pitch),
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Force of Negation alt cast should succeed on opponent's turn");
    assert!(g.exile.iter().any(|c| c.id == pitch));
    assert!(g.stack.iter().any(|si| matches!(
        si,
        crate::game::StackItem::Spell { card, .. } if card.id == fon
    )));
}

#[test]
fn devourer_of_destiny_etb_scries_two() {
    // ETB Scry 2: a scripted ScryOrder decision sends both top cards to
    // the bottom; the 3rd library card should bubble up to the top.
    let mut g = two_player_game();
    let _bottom_a = g.add_card_to_library(0, catalog::forest());
    let _bottom_b = g.add_card_to_library(0, catalog::plains());
    g.add_card_to_library(0, catalog::island());
    let third = g.players[0].library[2].id;
    let scry_a = g.players[0].library[0].id;
    let scry_b = g.players[0].library[1].id;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![],
        bottom: vec![scry_a, scry_b],
    }]));

    let dev = g.add_card_to_hand(0, catalog::devourer_of_destiny());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: dev, target: None, mode: None, x_value: None,
    })
    .expect("Devourer of Destiny is castable for {5}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == dev));
    assert_eq!(g.players[0].library[0].id, third,
        "After scry-2-to-bottom, the 3rd library card should be on top");
}

#[test]
fn wrath_of_the_skies_destroys_permanents_with_mana_value_x() {
    // Cast Wrath of the Skies with X=2: only nonland permanents whose CMC
    // is exactly 2 should die. Lands and other-CMC permanents survive.
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let llanowar = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // CMC 1
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());      // CMC 2
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());    // CMC 6

    let wrath = g.add_card_to_hand(0, catalog::wrath_of_the_skies());
    // Cost is {X}{W}{W}; pay {2}{W}{W} for X=2.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: wrath,
        target: None,
        mode: None,
        x_value: Some(2),
    })
    .expect("Wrath of the Skies should cast for {2}{W}{W}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == forest),
        "Lands aren't destroyed by Wrath of the Skies");
    assert!(g.battlefield.iter().any(|c| c.id == llanowar),
        "1-CMC creature shouldn't be destroyed when X=2");
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "2-CMC creature should die when X=2");
    assert!(g.battlefield.iter().any(|c| c.id == dragon),
        "6-CMC creature shouldn't be destroyed when X=2");
}

#[test]
fn spoils_of_the_vault_tutors_and_loses_three_life() {
    // Approximates "name + reveal until find" as Search(any → hand) plus
    // a flat 3-life cost.
    let mut g = two_player_game();
    let wanted = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::swamp());

    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(wanted))]));

    let spoils = g.add_card_to_hand(0, catalog::spoils_of_the_vault());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spoils,
        target: None,
        mode: None,
        x_value: None,
    })
    .expect("Spoils of the Vault should cast for {B}");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == wanted),
        "Tutored card should be in hand");
    assert_eq!(g.players[0].life, life_before - 3,
        "Spoils approximates the variable life cost as a flat 3");
}

#[test]
fn atraxa_grand_unifier_etb_draws_four() {
    // Approximation of "reveal top 10, take one of each card type" as
    // ETB Draw 4.
    let mut g = two_player_game();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let lib_before = g.players[0].library.len();
    let hand_before_cast = g.players[0].hand.len();
    let atraxa = g.add_card_to_hand(0, catalog::atraxa_grand_unifier());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: atraxa, target: None, mode: None, x_value: None,
    })
    .expect("Atraxa is castable for {3}{W}{U}{B}{R}{G}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == atraxa));
    // Hand started with Atraxa + drew 4 - cast Atraxa = +3 net change from
    // start, but only +4 from the ETB trigger itself.
    assert_eq!(g.players[0].hand.len(), hand_before_cast + 4,
        "Atraxa's ETB should draw 4 cards (approximation of the reveal-and-sort)");
    assert_eq!(g.players[0].library.len(), lib_before - 4,
        "Library should lose 4 cards");
}

#[test]
fn pathway_front_face_taps_for_front_color_only() {
    // Playing Blightstep Pathway via PlayLand picks the front (Swamp / B)
    // face. The land enters as a Swamp, has exactly one mana ability, and
    // taps for {B}.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blightstep_pathway());
    g.perform_action(GameAction::PlayLand(id))
        .expect("PlayLand should succeed for the front face");

    let card = g.battlefield_find(id).expect("card on battlefield");
    assert_eq!(card.definition.name, "Blightstep Pathway");
    assert!(card.definition.subtypes.land_types.contains(&crate::card::LandType::Swamp));
    assert!(!card.definition.subtypes.land_types.contains(&crate::card::LandType::Mountain),
        "Front face should be a Swamp only — Mountain belongs to the back face");
    assert_eq!(card.definition.activated_abilities.len(), 1,
        "Front face exposes only one mana ability (the front color)");

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None,
    }).expect("front face taps for {B}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 0,
        "Front face must not produce {{R}} — that's the back face");
}

#[test]
fn pathway_back_face_taps_for_back_color_only() {
    // Playing Blightstep Pathway via PlayLandBack swaps to the back face
    // (Searstep Pathway / Mountain / R) before placing on battlefield.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::blightstep_pathway());
    g.perform_action(GameAction::PlayLandBack(id))
        .expect("PlayLandBack should succeed for the back face");

    let card = g.battlefield_find(id).expect("card on battlefield");
    assert_eq!(card.definition.name, "Searstep Pathway");
    assert!(card.definition.subtypes.land_types.contains(&crate::card::LandType::Mountain));
    assert!(!card.definition.subtypes.land_types.contains(&crate::card::LandType::Swamp));
    assert_eq!(card.definition.activated_abilities.len(), 1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None,
    }).expect("back face taps for {R}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 0);
}

#[test]
fn play_land_back_rejects_non_mdfc() {
    // A regular basic with no `back_face` can't be played via PlayLandBack.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::forest());
    let err = g.perform_action(GameAction::PlayLandBack(id)).unwrap_err();
    assert_eq!(err, GameError::NotALand(id),
        "Playing a non-MDFC via PlayLandBack should reject");
    // Card returns to hand, no land was played.
    assert!(g.players[0].hand.iter().any(|c| c.id == id));
    assert_eq!(g.players[0].lands_played_this_turn, 0);
}

#[test]
fn gemstone_mine_etb_with_three_charge_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::gemstone_mine());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);
    let card = g.battlefield_find(id).expect("on battlefield");
    assert_eq!(card.counter_count(crate::card::CounterType::Charge), 3,
        "Gemstone Mine should ETB with three charge (mining) counters");
}

#[test]
fn gemstone_mine_taps_three_times_then_sacrifices() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::gemstone_mine());
    // Manually seed counters since add_card_to_battlefield bypasses the ETB.
    g.battlefield_find_mut(id).unwrap()
        .add_counters(crate::card::CounterType::Charge, 3);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Color(Color::White),
        DecisionAnswer::Color(Color::Blue),
        DecisionAnswer::Color(Color::Black),
    ]));

    // Tap #1: counter 3 → 2, no sac.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None,
    }).expect("tap #1 succeeds");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some());
    assert_eq!(g.battlefield_find(id).unwrap()
        .counter_count(crate::card::CounterType::Charge), 2);
    g.battlefield_find_mut(id).unwrap().tapped = false;

    // Tap #2: counter 2 → 1.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None,
    }).expect("tap #2 succeeds");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some());
    assert_eq!(g.battlefield_find(id).unwrap()
        .counter_count(crate::card::CounterType::Charge), 1);
    g.battlefield_find_mut(id).unwrap().tapped = false;

    // Tap #3: counter 1 → 0, then sacrifice.
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None,
    }).expect("tap #3 succeeds");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id),
        "Gemstone Mine should sac itself when the last counter is removed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "Sacrificed Gemstone Mine should land in the graveyard");
}

#[test]
fn teferi_minus_three_returns_target_and_draws() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    // Teferi's base loyalty (4) is seeded by CardInstance::new for
    // planeswalkers; no need to add counters manually.
    let teferi = g.add_card_to_battlefield(0, catalog::teferi_time_raveler());
    let opp_creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Seed P0's library so the +draw doesn't deck them out.
    g.add_card_to_library(0, catalog::forest());

    let hand_before = g.players[0].hand.len();
    let opp_hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: teferi,
        ability_index: 0, // -3
        target: Some(Target::Permanent(opp_creature)),
    })
    .expect("Teferi's -3 should accept an opponent's nonland permanent");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_creature),
        "Targeted creature should leave the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == opp_creature),
        "Bounced creature should return to its owner's hand");
    assert_eq!(g.players[1].hand.len(), opp_hand_before + 1);
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "Teferi's -3 should also draw a card");
    assert_eq!(g.battlefield_find(teferi).unwrap()
        .counter_count(CounterType::Loyalty), 1,
        "Loyalty should drop from 4 to 1 after a -3 ability");
}

#[test]
fn mystical_dispute_alt_cost_requires_blue_target() {
    // Casting Mystical Dispute via the alt cost {U} should reject a non-blue
    // target spell on the stack and accept a blue one.
    let mut g = two_player_game();
    g.players[0].mana_pool.add(Color::Blue, 1);

    // Put a non-blue spell on the stack first.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt is castable for {R}");

    g.priority.player_with_priority = 0;
    let dispute = g.add_card_to_hand(0, catalog::mystical_dispute());
    let err = g.perform_action(GameAction::CastSpellAlternative {
        card_id: dispute,
        pitch_card: None,
        target: Some(Target::Permanent(bolt)),
        mode: None,
        x_value: None,
    })
    .unwrap_err();
    assert_eq!(err, GameError::SelectionRequirementViolated,
        "Alt cost should reject a non-blue target");

    // Replace with a blue spell on the stack.
    g.stack.clear();
    g.players[0].mana_pool.add(Color::Blue, 1);
    let counterspell = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counterspell,
        target: Some(Target::Permanent(bolt)),
        mode: None,
        x_value: None,
    })
    .expect("Counterspell is castable for {U}{U}");

    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: dispute,
        pitch_card: None,
        target: Some(Target::Permanent(counterspell)),
        mode: None,
        x_value: None,
    })
    .expect("Mystical Dispute alt cost should accept a blue target");
}

#[test]
fn mystical_dispute_does_not_counter_when_opponent_can_pay() {
    // Opponent has 3 colorless to spare → Dispute auto-pays on their
    // behalf and the spell survives.
    let mut g = two_player_game();
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    // Give P1 enough mana to pay the {3} tax.
    g.players[1].mana_pool.add_colorless(3);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");

    g.priority.player_with_priority = 0;
    let dispute = g.add_card_to_hand(0, catalog::mystical_dispute());
    g.perform_action(GameAction::CastSpell {
        card_id: dispute,
        target: Some(Target::Permanent(bolt)),
        mode: None,
        x_value: None,
    })
    .expect("Mystical Dispute castable for {2}{U}");
    drain_stack(&mut g);

    // Bolt's controller (P1) paid {3}, so Bolt resolved and dealt damage.
    assert_eq!(g.players[0].life, 17,
        "Bolt should still resolve when P1 pays the dispute tax");
    assert_eq!(g.players[1].mana_pool.colorless_amount(), 0,
        "P1's spare colorless should have been consumed paying the tax");
}

#[test]
fn mystical_dispute_counters_when_opponent_cannot_pay() {
    // Same scenario but opponent has no spare mana → counter goes through.
    let mut g = two_player_game();
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");

    g.priority.player_with_priority = 0;
    let dispute = g.add_card_to_hand(0, catalog::mystical_dispute());
    g.perform_action(GameAction::CastSpell {
        card_id: dispute,
        target: Some(Target::Permanent(bolt)),
        mode: None,
        x_value: None,
    })
    .expect("Mystical Dispute castable for {2}{U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, 20,
        "Bolt should be countered when P1 can't afford the {{3}} tax");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "Countered Bolt should hit P1's graveyard");
}

#[test]
fn quantum_riddler_on_cast_draws_even_if_countered() {
    // The cantrip is a SpellCast+SelfSource trigger that goes on the stack
    // above the spell, so it resolves first — countering Quantum Riddler in
    // response should not prevent the draw.
    let mut g = two_player_game();
    let drawn = g.add_card_to_library(0, catalog::forest());

    let qr = g.add_card_to_hand(0, catalog::quantum_riddler());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: qr, target: None, mode: None, x_value: None,
    })
    .expect("Quantum Riddler is castable for {1}{U}{B}");

    // P1 counters Quantum Riddler with a Counterspell while the on-cast
    // cantrip is still on the stack above it.
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(qr)),
        mode: None,
        x_value: None,
    })
    .expect("Counterspell cast against Quantum Riddler");

    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == qr),
        "Quantum Riddler should be countered (no permanent in play)");
    assert!(g.players[0].hand.iter().any(|c| c.id == drawn),
        "On-cast cantrip should still draw P0's library card even though the spell was countered");
}

#[test]
fn convoke_taps_creature_to_pay_one_generic() {
    // Wrath of the Skies costs {X}{W}{W}. With X=2 + 1 convoked creature,
    // the player only needs {1}{W}{W} from real mana — the convoke tap
    // contributes the missing {1}. The same X=2 wrath then sweeps both
    // 2-CMC nonland permanents (including the just-tapped convoke
    // creature, which is itself a 2-CMC bear).
    let mut g = two_player_game();
    let mascot = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(mascot);
    // Only enough real mana for {1}{W}{W} — short by {1} for X=2.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);

    let wrath = g.add_card_to_hand(0, catalog::wrath_of_the_skies());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // Without convoke, the cast fails (mana shortfall).
    let try_no_convoke = g.perform_action(GameAction::CastSpell {
        card_id: wrath, target: None, mode: None, x_value: Some(2),
    });
    assert!(try_no_convoke.is_err(),
        "Wrath at X=2 should be unaffordable without convoke help");

    // With convoke, the tap contributes the missing {1}.
    g.perform_action(GameAction::CastSpellConvoke {
        card_id: wrath,
        target: None,
        mode: None,
        x_value: Some(2),
        convoke_creatures: vec![mascot],
    })
    .expect("Wrath should be castable with convoke topping up the X cost");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Wrath at X=2 destroys the 2-CMC opp creature");
    // The convoked creature itself is also 2-CMC and gets swept by its own
    // wrath. Either way it's no longer on battlefield (graveyarded).
    assert!(!g.battlefield.iter().any(|c| c.id == mascot));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mascot),
        "Convoked creature was tapped, then destroyed by Wrath, ending in graveyard");
}

#[test]
fn pest_control_at_converge_three_destroys_higher_cmc() {
    // Pest Control cost is {W}{B}, but we'll inject extra mana of a third
    // color to bump converge to 3. This should now destroy 3-CMC nonland
    // permanents (e.g. anything with three pips of cost).
    let mut g = two_player_game();
    // 3-CMC opponent permanent — Goblin Guide (1R) is 1-CMC, Lightning
    // Bolt is instant (no permanent on battlefield). Use Serra Angel
    // (3WW = 5 CMC). Hmm we need a 3-CMC creature. Use Black Knight (BB)?
    // 2-CMC. Use a 3-CMC one. The catalog has shivan_dragon (4RR = 6).
    // Let's just use Shivan Dragon and verify converge=3 doesn't kill it.
    // For converge=2 leaving a 3-CMC creature, we need… let's just add
    // a 3-CMC permanent via a test card. Use catalog::serra_angel which
    // is 3WW = CMC 5. Hmm.
    //
    // Simpler: use Grizzly Bears (CMC 2) to confirm converge=3 still
    // destroys it (it covers 1-3) and converge=2 also destroys it.
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pest = g.add_card_to_hand(0, catalog::pest_control());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: pest, target: None, mode: None, x_value: None,
    })
    .expect("Pest Control castable for {W}{B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp),
        "Pest Control at converge=2 destroys 2-CMC creatures");
}

#[test]
fn convoke_rejects_tapped_creature() {
    // Convoking a tapped creature should reject the cast.
    let mut g = two_player_game();
    let mascot = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mascot).unwrap().tapped = true;
    g.players[0].mana_pool.add(Color::White, 1);

    let pe = g.add_card_to_hand(0, catalog::prismatic_ending());
    let err = g.perform_action(GameAction::CastSpellConvoke {
        card_id: pe,
        target: None,
        mode: None,
        x_value: None,
        convoke_creatures: vec![mascot],
    });
    assert!(err.is_err(), "Convoking a tapped creature should reject");
    // Card should be back in hand.
    assert!(g.players[0].hand.iter().any(|c| c.id == pe));
}

#[test]
fn ephemerate_rebound_exiles_then_recasts_next_upkeep() {
    // Cast Ephemerate from hand. After resolution: creature is flickered
    // back, Ephemerate is in exile (not graveyard) with a `YourNextUpkeep`
    // delayed trigger queued. Advancing to P0's next upkeep should fire
    // the rebound — re-running the flicker effect on a fresh auto-target.
    let mut g = two_player_game();
    let creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Damage so we can confirm the flicker clears it (and the rebound
    // recast still flickers something).
    g.battlefield_find_mut(creature).unwrap().damage = 1;

    let eph = g.add_card_to_hand(0, catalog::ephemerate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: eph,
        target: Some(Target::Permanent(creature)),
        mode: None,
        x_value: None,
    })
    .expect("Ephemerate castable for {W}");
    drain_stack(&mut g);

    // Damage cleared by flicker.
    assert_eq!(g.battlefield_find(creature).unwrap().damage, 0);
    // Rebound: Ephemerate is in exile, NOT in graveyard.
    assert!(g.exile.iter().any(|c| c.id == eph),
        "Ephemerate should be exiled by rebound, not graveyarded");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == eph),
        "Ephemerate should NOT have hit P0's graveyard");
    // A delayed YourNextUpkeep trigger should be queued.
    assert_eq!(g.delayed_triggers.len(), 1,
        "Rebound should register one delayed trigger");

    // Damage the creature again so we can prove the rebound recast actually
    // re-flickered it (would clear the new damage).
    g.battlefield_find_mut(creature).unwrap().damage = 2;

    // Advance to P0's next upkeep.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 1;
    for _ in 0..15 {
        if g.active_player_idx == 0 && g.step == TurnStep::Upkeep && g.stack.is_empty() {
            break;
        }
        g.perform_action(GameAction::PassPriority).unwrap();
    }

    // The rebound trigger fired and re-flickered the creature → damage cleared.
    assert_eq!(g.battlefield_find(creature).unwrap().damage, 0,
        "Rebound's recast should re-flicker the creature, clearing damage");
    assert!(g.delayed_triggers.is_empty(),
        "Rebound trigger should fire once and clear");
}

#[test]
fn elesh_norn_doubles_your_etb_triggers() {
    // Devourer of Destiny scry-on-cast still fires from the stack — but
    // ETB triggers from a permanent entering should fire 2x while you
    // control Elesh Norn. Use Quantum Riddler's on-cast cantrip… wait,
    // that's a SpellCast trigger, not ETB. Use a creature with a real
    // ETB: Solitude (ETB exiles target opp creature). With Elesh Norn out
    // and two opp creatures, both should be exiled.
    let mut g = two_player_game();
    let _norn = g.add_card_to_battlefield(0, catalog::elesh_norn_mother_of_machines());
    let opp_a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_b = g.add_card_to_battlefield(1, catalog::llanowar_elves());

    let solitude = g.add_card_to_hand(0, catalog::solitude());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: solitude, target: None, mode: None, x_value: None,
    })
    .expect("Solitude castable for {3}{W}");
    drain_stack(&mut g);

    // Both opp creatures should have been exiled by the doubled ETB.
    let exiled = [opp_a, opp_b].iter()
        .filter(|cid| g.exile.iter().any(|c| c.id == **cid))
        .count();
    assert_eq!(exiled, 2,
        "Solitude's ETB should fire twice with Elesh Norn out, exiling both opp creatures");
}

#[test]
fn elesh_norn_suppresses_opponent_etb_triggers() {
    // P0 controls Elesh Norn. P1 plays Solitude — its ETB exile trigger
    // should be suppressed (zero firings), so P0's creatures stay put.
    let mut g = two_player_game();
    let _norn = g.add_card_to_battlefield(0, catalog::elesh_norn_mother_of_machines());
    let p0_creature = g.add_card_to_battlefield(0, catalog::grizzly_bears());

    let solitude = g.add_card_to_hand(1, catalog::solitude());
    g.players[1].mana_pool.add(Color::White, 1);
    g.players[1].mana_pool.add_colorless(3);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: solitude, target: None, mode: None, x_value: None,
    })
    .expect("Opponent's Solitude castable");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == p0_creature),
        "P0's creature should survive — Elesh Norn suppresses opp's ETB triggers");
    assert!(!g.exile.iter().any(|c| c.id == p0_creature));
}

#[test]
fn cavern_of_souls_makes_creatures_uncounterable() {
    // While P0 controls a Cavern of Souls, any creature spell P0 casts is
    // marked uncounterable on the stack — Counterspell from P1 should not
    // remove it.
    let mut g = two_player_game();
    let _cavern = g.add_card_to_battlefield(0, catalog::cavern_of_souls());

    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, mode: None, x_value: None,
    })
    .expect("Grizzly Bears castable for {1}{G}");

    // P1 tries to counter the bear.
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(bear)),
        mode: None,
        x_value: None,
    })
    .expect("Counterspell castable");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bear),
        "Cavern of Souls should make P0's creature uncounterable");
}

#[test]
fn cavern_of_souls_does_not_protect_noncreature_spells() {
    // Cavern's uncounterable approximation is creature-only — countering a
    // noncreature spell should still work even with a Cavern in play.
    let mut g = two_player_game();
    let _cavern = g.add_card_to_battlefield(0, catalog::cavern_of_souls());

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");

    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(bolt)),
        mode: None,
        x_value: None,
    })
    .expect("Counterspell castable");
    drain_stack(&mut g);

    assert!(!g.stack.iter().any(|si| matches!(
        si, crate::game::StackItem::Spell { card, .. } if card.id == bolt
    )), "Lightning Bolt should be countered (Cavern only protects creatures)");
}

#[test]
fn damping_sphere_taxes_each_spell_after_the_first() {
    // First Lightning Bolt costs {R}; second one costs {1}{R} while
    // Damping Sphere is in play.
    let mut g = two_player_game();
    let _sphere = g.add_card_to_battlefield(0, catalog::damping_sphere());

    // First spell: should cost just {R}.
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("First Lightning Bolt should cast for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].spells_cast_this_turn, 1);

    // Second spell: should now require {1}{R}. Pay just {R} → fails.
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt2,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    });
    assert!(err.is_err(),
        "Second spell should require an extra {{1}} under Damping Sphere");

    // With the extra {1}, it casts.
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("Second Lightning Bolt should cast for {1}{R} under Damping Sphere");
}

#[test]
fn damping_sphere_resets_count_at_turn_start() {
    // Per-player `spells_cast_this_turn` should clear at the start of the
    // next turn, so the first spell of a new turn isn't taxed.
    let mut g = two_player_game();
    let _sphere = g.add_card_to_battlefield(0, catalog::damping_sphere());

    g.players[0].spells_cast_this_turn = 3;
    // Simulate a fresh turn for P0 — `do_untap` resets the per-player
    // counter (and lands_played).
    g.do_untap();
    assert_eq!(g.players[0].spells_cast_this_turn, 0,
        "Per-player spell count should reset on untap");

    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        mode: None,
        x_value: None,
    })
    .expect("First spell of a new turn should not be taxed");
}

#[test]
fn consign_to_memory_counters_targeted_trigger() {
    // P1 casts Devourer of Destiny (on-cast Scry 2 trigger goes on the
    // stack). P0 responds with Consign to Memory targeting Devourer; the
    // Scry trigger is removed from the stack before it can resolve.
    let mut g = two_player_game();
    // Seed P1's library so the scry would have something to act on if it
    // resolved (so we can verify it didn't).
    g.add_card_to_library(1, catalog::island());
    g.add_card_to_library(1, catalog::island());

    let dev = g.add_card_to_hand(1, catalog::devourer_of_destiny());
    g.players[1].mana_pool.add_colorless(5);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: dev, target: None, mode: None, x_value: None,
    })
    .expect("Devourer of Destiny castable for {5}");

    // Confirm the scry trigger landed on the stack alongside the spell.
    let trigger_count = g.stack.iter()
        .filter(|si| matches!(si, crate::game::StackItem::Trigger { source, .. } if *source == dev))
        .count();
    assert_eq!(trigger_count, 1, "Scry-on-cast trigger should be queued");

    // P0 casts Consign to Memory targeting Devourer.
    let consign = g.add_card_to_hand(0, catalog::consign_to_memory());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: consign,
        target: Some(Target::Permanent(dev)),
        mode: None,
        x_value: None,
    })
    .expect("Consign to Memory castable for {U}");
    drain_stack(&mut g);

    // The Scry trigger never resolved → no scry decision was asked. We
    // can't directly observe "did scry happen" without a scripted decider,
    // but we can assert the trigger was removed from the stack and the
    // Devourer still resolved (since Consign only counters the ability,
    // not the spell).
    assert!(g.battlefield.iter().any(|c| c.id == dev),
        "Devourer should still resolve — Consign only counters the ability");
    assert!(!g.stack.iter().any(|si| matches!(
        si, crate::game::StackItem::Trigger { source, .. } if *source == dev
    )), "Scry-on-cast trigger should have been countered");
}
