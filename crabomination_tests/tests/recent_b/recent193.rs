//! Functionality tests for `catalog::sets::decks::recent193` (BLB/FDN gaps
//! riding mana-value-vs-trigger-event filters).

use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Jackdaw Savior: when another flying creature you control dies, reanimate a
/// lesser-mana-value creature card from your graveyard.
#[test]
fn jackdaw_savior_reanimates_lesser_mv() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::jackdaw_savior()); // surviving 3/1 flyer
    let victim = g.add_card_to_battlefield(0, catalog::jackdaw_savior()); // MV 3 flyer dies
    let target = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 < 3
    g.battlefield_find_mut(victim).unwrap().damage = 1; // lethal on the 3/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_some(), "grizzly returned to the battlefield");
}

/// Jackdaw's own death also triggers: the self-death path now threads the dying
/// creature's mana value so the lesser-MV reanimation finds a target.
#[test]
fn jackdaw_savior_self_death_reanimates() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let jackdaw = g.add_card_to_battlefield(0, catalog::jackdaw_savior()); // MV 3, dies
    let target = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 < 3
    g.battlefield_find_mut(jackdaw).unwrap().damage = 1; // lethal on the 3/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_some(), "grizzly returned on Jackdaw's own death");
}

/// Clement's enter trigger bounces a lesser-mana-value creature you control.
#[test]
fn clement_bounces_lesser_mv_on_enter() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2
    let clement = g.add_card_to_hand(0, catalog::clement_the_worrywort()); // MV 3
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: clement,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Clement");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "lesser-MV bear bounced to hand");
}

/// Soul-Shackled Zombie: exiling a creature card from a graveyard drains the
/// opponent for 2 and gains you 2.
#[test]
fn soul_shackled_zombie_creature_exile_drains() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let victim = g.add_card_to_graveyard(1, catalog::grizzly_bears()); // creature card
    g.players[0].life = 20;
    g.players[1].life = 20;
    let zombie = g.add_card_to_hand(0, catalog::soul_shackled_zombie());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Supply the exile pick (auto-decider declines an "up to N" choice).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![victim])]));
    g.perform_action(GameAction::CastSpell {
        card_id: zombie,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Soul-Shackled Zombie");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "creature card exiled");
    assert_eq!(g.players[1].life, 18, "opponent lost 2");
    assert_eq!(g.players[0].life, 22, "you gained 2");
}
