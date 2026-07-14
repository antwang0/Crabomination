//! Functionality tests for `catalog::sets::decks::recent208`.

use crate::card::Keyword;
use crate::catalog;
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::game::{drain_stack, two_player_game};
use crate::mana::Color;

/// Vanilla bodies carry their printed keyword set (Gleaming Barrier = Defender).
#[test]
fn vanillas_and_wall() {
    let mut g = two_player_game();
    let hb = g.add_card_to_battlefield(0, catalog::highborn_vampire());
    let sg = g.add_card_to_battlefield(0, catalog::swab_goblin());
    let gb = g.add_card_to_battlefield(0, catalog::gleaming_barrier());
    assert_eq!((g.computed_permanent(hb).unwrap().power, g.computed_permanent(hb).unwrap().toughness), (4, 3));
    assert_eq!((g.computed_permanent(sg).unwrap().power, g.computed_permanent(sg).unwrap().toughness), (2, 2));
    assert!(g.computed_permanent(gb).unwrap().keywords.contains(&Keyword::Defender));
}

/// Gleaming Barrier leaves a Treasure when it dies.
#[test]
fn gleaming_barrier_dies_into_treasure() {
    let mut g = two_player_game();
    let gb = g.add_card_to_battlefield(0, catalog::gleaming_barrier());
    g.battlefield_find_mut(gb).unwrap().counters.insert(crate::card::CounterType::MinusOneMinusOne, 4);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 1);
}

/// Storm Fleet Spy draws only if you attacked this turn (Raid).
#[test]
fn storm_fleet_spy_raid_draw() {
    // No attack this turn → no draw.
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let h0 = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::storm_fleet_spy());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0, "no attack → no Raid draw");

    // With an attack recorded, it draws.
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }]).expect("attack");
    g.add_card_to_library(0, catalog::island());
    let h1 = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::storm_fleet_spy());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h1 + 1, "attacked → Raid draws");
}

/// Battle-Rattle Shaman pumps a target at the start of your combat.
#[test]
fn battle_rattle_shaman_begin_combat_pump() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::battle_rattle_shaman());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
        crate::decision::DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0 from the shaman");
}

/// Wildheart Invoker gives +5/+5 and trample for {8}.
#[test]
fn wildheart_invoker_overruns_one() {
    let mut g = two_player_game();
    let inv = g.add_card_to_battlefield(0, catalog::wildheart_invoker());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(8);
    g.perform_action(GameAction::ActivateAbility {
        card_id: inv, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Devout Decree exiles a red creature and scries.
#[test]
fn devout_decree_exiles_red() {
    let mut g = two_player_game();
    let goblin = g.add_card_to_battlefield(1, catalog::swab_goblin()); // red
    g.add_card_to_library(0, catalog::island());
    let s = g.add_card_to_hand(0, catalog::devout_decree());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: Some(Target::Permanent(goblin)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(goblin).is_none(), "red creature exiled");
    assert!(g.exile.iter().any(|c| c.id == goblin), "in exile, not graveyard");
}
