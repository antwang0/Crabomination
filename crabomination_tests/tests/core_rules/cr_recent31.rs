//! CR conformance for the RNA batch's engine work:
//! - CR 702.108c — an adapt *activated ability* fires "whenever you activate an
//!   adapt ability" triggers (Gyre Engineer untaps).
//! - CR 509.1b — a "can't be blocked by creatures with power N or less"
//!   restriction is enforced at block declaration (Enraged Ceratok).
//! - CR 602.5e — a creature whose activated abilities can't be activated
//!   (Lawmage's Binding) rejects activation attempts.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// CR 702.108c — activating an adapt ability triggers "whenever you activate an
/// adapt ability" abilities, even on a different creature.
#[test]
fn cr_702_108c_adapt_activation_triggers() {
    let mut g = two_player_game();
    let eng = g.add_card_to_battlefield(0, catalog::gyre_engineer());
    let eel = g.add_card_to_battlefield(0, catalog::skitter_eel()); // {5}{U}: Adapt 2
    g.clear_sickness(eng);
    g.clear_sickness(eel);
    g.battlefield_find_mut(eng).unwrap().tapped = true;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility { card_id: eel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("adapt");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(eng).unwrap().tapped, "Gyre Engineer untapped by the adapt-activation trigger");
}

/// CR 509.1b — a power-2 creature can't be declared as a blocker for a creature
/// with "can't be blocked by creatures with power 2 or less."
#[test]
fn cr_509_1b_cant_be_blocked_by_small_power() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::enraged_ceratok()); // 4/4
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: atk, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(small, atk)])).is_err(),
        "power-2 blocker is illegal against Enraged Ceratok");
}

/// CR 602.5e — Lawmage's Binding stops the enchanted creature from activating
/// its abilities.
#[test]
fn cr_602_5e_binding_locks_activation() {
    let mut g = two_player_game();
    // Devkarin Dissident has a {4}{G}: +2/+2 activated ability.
    let creature = g.add_card_to_battlefield(1, catalog::devkarin_dissident());
    g.clear_sickness(creature);
    // Attach Lawmage's Binding to it.
    let aura = g.add_card_to_hand(0, catalog::lawmages_binding());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell { card_id: aura, target: Some(Target::Permanent(creature)), additional_targets: vec![], mode: None, x_value: None }).expect("cast binding");
    drain_stack(&mut g);
    assert!(g.computed_permanent(creature).unwrap().keywords.contains(&Keyword::CantActivateAbilities), "binding grants CantActivateAbilities");
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(4);
    assert!(g.perform_action(GameAction::ActivateAbility { card_id: creature, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).is_err(),
        "bound creature can't activate its ability");
}
