//! CR conformance for rules exercised by this run's DGM gap wave 3:
//! CR 606.3 (one loyalty ability per turn — Ral Zarek), CR 117.7c (cost
//! reduction is generic-only — Council of the Absolute's chosen-name discount),
//! and CR 510 (a "deals combat damage to you" trigger identifies the dealing
//! creature — Teysa, Envoy of Ghosts).

use crabomination::card::CardInstance;
use crabomination::catalog;
use crabomination::game::actions::cost_reduction_for_spell;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game, GameError};
use crabomination::mana::Color;

/// CR 606.3 — a player may activate only one loyalty ability of a given
/// permanent each turn; the second activation is rejected.
#[test]
fn cr_606_3_one_loyalty_ability_per_turn() {
    let mut g = two_player_game();
    let ral = g.add_card_to_battlefield(0, catalog::ral_zarek());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ral, ability_index: 1, target: Some(Target::Player(1)), x_value: None,
    }).expect("first loyalty ability");
    drain_stack(&mut g);
    let err = g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ral, ability_index: 0, target: Some(Target::Permanent(ral)), x_value: None,
    });
    assert!(matches!(err, Err(GameError::LoyaltyAbilityAlreadyUsed(_))), "second is illegal");
}

/// CR 117.7c — a cost reduction lowers only the generic portion of a cost.
/// Council of the Absolute's chosen-name discount shaves {2} off Punish the
/// Enemy's {4}{R}, so {2}{R} (three mana) casts it; the colored {R} is never
/// reduced away.
#[test]
fn cr_117_7c_named_cost_reduction_is_generic_only() {
    let mut g = two_player_game();
    let council = g.add_card_to_battlefield(0, catalog::council_of_the_absolute());
    g.battlefield_find_mut(council).unwrap().named_card = Some("Punish the Enemy".into());
    let named = CardInstance::new(g.next_id(), catalog::punish_the_enemy(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &named, None), 2, "chosen name → {{2}} off");

    // {2}{R} pays the discounted cost; the base {4}{R} would need five.
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::punish_the_enemy());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![Target::Permanent(foe)], mode: None, x_value: None,
    }).expect("cast with the {2} discounted away");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "resolved: the 2/2 took 3");
}

/// CR 510 — a "whenever a creature deals combat damage to you" trigger can act
/// on the creature that dealt the damage (the dealer is bound as its source).
#[test]
fn cr_510_combat_damage_to_you_identifies_dealer() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::teysa_envoy_of_ghosts());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack player 0");
    drain_stack(&mut g);
    let mut guard = 0;
    while g.step != TurnStep::CombatDamage && guard < 40 {
        g.perform_action(GameAction::PassPriority).expect("pass");
        guard += 1;
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "the dealing creature was destroyed");
}
