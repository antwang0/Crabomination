//! CR conformance for this run's engine work:
//! - CR 109.2 / 109.5 — "creature" means a battlefield permanent; "you" on a
//!   triggered ability is its controller, and "that player may" routes to the
//!   named seat (`Effect::MayDoBy`).
//! - CR 204 — an object with a colour indicator is each colour it denotes.
//! - CR 733.1 — an illegal action is reversed whole; payments already made
//!   are cancelled.

use crabomination::card::CardType;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(10);
}

/// CR 109.5 — "you" on a triggered ability is the ability's controller, not
/// the player whose action fired it: Jeweled Torque's owner gains the life
/// when an *opponent* casts a spell of the chosen colour.
#[test]
fn cr_109_5_trigger_you_is_the_abilitys_controller() {
    let mut g = two_player_game();
    let torque = g.add_card_to_battlefield(0, catalog::jeweled_torque());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Color(Color::Red),
        DecisionAnswer::Bool(true),
    ]));
    g.fire_self_etb_triggers(torque, 0);
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 0);
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22, "the Torque's controller gains");
}

/// CR 109.5 sibling — "that player may" routes the choice to the named seat
/// (Ley Line's each-upkeep counter belongs to the active player).
#[test]
fn cr_109_5_that_player_may_routes_to_the_named_seat() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ley_line());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// CR 204.2 — a token minted with a colour indicator is that colour even
/// though it has no mana cost.
#[test]
fn cr_204_2_color_indicator_defines_color() {
    let mut g = two_player_game();
    let cage = g.add_card_to_battlefield(0, catalog::monkey_cage());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cage).is_none());
    let monkey = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Monkey")
        .expect("minted");
    assert!(monkey.definition.cost.symbols.is_empty(), "tokens have no mana cost");
    assert_eq!(g.computed_permanent(monkey.id).unwrap().colors, vec![Color::Green]);
}

/// CR 733.1 — a declaration the acting player can't pay for is reversed whole:
/// no attackers are declared and the mana already floated is untouched.
#[test]
fn cr_733_1_unpayable_attack_declaration_is_reversed_whole() {
    let mut g = two_player_game();
    let tax = g.add_card_to_battlefield(1, catalog::war_tax());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tax,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(5),
    })
    .expect("activate");
    drain_stack(&mut g);

    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.players[0].mana_pool.add(Color::Red, 2); // short of the {5} toll
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])
            .is_err()
    );
    assert!(g.attacking.is_empty(), "no attacker stuck");
    assert_eq!(g.players[0].mana_pool.total(), 2, "payment cancelled");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "attack tap reversed");
}

/// CR 109.2 — "creature" with no zone word means a permanent on the
/// battlefield: Cowardice's trigger doesn't fire off a creature *card* being
/// targeted in a graveyard.
#[test]
fn cr_109_2_creature_means_a_battlefield_permanent() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::cowardice());
    let gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(crabomination::game::types::Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == gy), "graveyard card untouched");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear));
    assert!(!g.players[1].hand.iter().any(|c| c.definition.card_types == vec![CardType::Instant]));
}
