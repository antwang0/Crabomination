//! Functionality tests for `catalog::sets::decks::recent44` — artifact/enchantment
//! hate, ability-tax hate-bears, and the Hushwing Gryff ETB-suppressor.

use crate::card::StaticEffect;
use crate::catalog;
use crate::game::two_player_game;
use crate::game::*;

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: idx, target, additional_targets: Vec::new(), x_value: None,
    }).expect("ability activates");
    drain_stack(g);
}

#[test]
fn uktabi_orangutan_smashes_an_artifact_on_etb() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::ratchet_bomb());
    let ape = g.add_card_to_battlefield(0, catalog::uktabi_orangutan());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Target(Target::Permanent(art)),
    ]));
    g.fire_self_etb_triggers(ape, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "the artifact is destroyed");
}

#[test]
fn viridian_zealot_sacs_to_destroy_artifact_or_enchantment() {
    let mut g = two_player_game();
    let zealot = g.add_card_to_battlefield(0, catalog::viridian_zealot());
    let ench = g.add_card_to_battlefield(1, catalog::mark_of_asylum());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, zealot, 0, Some(Target::Permanent(ench)));
    assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
    assert!(g.battlefield_find(zealot).is_none(), "Zealot sacrificed itself");
}

#[test]
fn glowrider_taxes_noncreature_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::glowrider());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let bolt = g.players[0].hand.iter().find(|c| c.id == bolt).unwrap().clone();
    let bear = g.players[0].hand.iter().find(|c| c.id == bear).unwrap().clone();
    assert_eq!(crate::game::actions::extra_cost_for_spell(&g, 0, &bolt, None, 0), 1, "noncreature taxed");
    assert_eq!(crate::game::actions::extra_cost_for_spell(&g, 0, &bear, None, 0), 0, "creature untaxed");
}

#[test]
fn ingot_chewer_has_an_evoke_cost() {
    let alt = catalog::ingot_chewer().alternative_cost.unwrap();
    assert!(alt.evoke_sacrifice, "evoke sacrifices the body on ETB");
}

#[test]
fn energy_flux_grants_artifacts_an_upkeep_tax() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::energy_flux());
    let art = g.add_card_to_battlefield(0, catalog::ratchet_bomb());
    let granted = g.statics_granted_triggers_for(g.battlefield_find(art).unwrap());
    assert!(!granted.is_empty(), "the artifact inherits Energy Flux's upkeep tax");
}

#[test]
fn hushwing_gryff_suppresses_creature_etb_triggers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hushwing_gryff());
    let art = g.add_card_to_battlefield(1, catalog::ratchet_bomb());
    // Another Uktabi Orangutan enters: its ETB destroy is suppressed.
    let ape = g.add_card_to_battlefield(0, catalog::uktabi_orangutan());
    g.fire_self_etb_triggers(ape, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_some(), "the ETB trigger never fired");
    // Sanity: the suppressor's static is present.
    assert!(catalog::hushwing_gryff().static_abilities.iter().any(|s|
        matches!(s.effect, StaticEffect::SuppressCreatureEtbTriggers { .. })));
}

#[test]
fn harsh_mentor_punishes_opponent_ability_activations() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::harsh_mentor());
    // An opponent activates a non-mana ability (Ratchet Bomb's charge).
    let bomb = g.add_card_to_battlefield(1, catalog::ratchet_bomb());
    let life = g.players[1].life;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bomb, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("charge ability");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "Harsh Mentor deals 2 to the activating opponent");
}
