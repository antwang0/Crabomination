//! Functionality tests for `catalog::sets::decks::recent204`.

use crabomination::card::{CardType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Saw grants +2/+0 to the creature it equips.
#[test]
fn saw_grants_power_bonus() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let saw = g.add_card_to_battlefield(0, catalog::saw());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: saw, target: bear }).expect("equip Saw");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 2), "+2/+0 on the 2/2");
}

/// Unable to Scream turns the enchanted creature into a 0/2 Toy artifact creature
/// with no abilities.
#[test]
fn unable_to_scream_makes_a_toy() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer
    let aura = g.add_card_to_hand(0, catalog::unable_to_scream());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Unable to Scream");
    drain_stack(&mut g);
    let c = g.computed_permanent(victim).unwrap();
    assert_eq!((c.power, c.toughness), (0, 2), "base 0/2");
    assert!(!c.keywords.contains(&Keyword::Flying), "lost flying");
    assert!(c.card_types.contains(&CardType::Artifact), "now an artifact");
    assert!(c.subtypes.creature_types.contains(&CreatureType::Toy), "a Toy");
}

/// Sporogenic Infection edicts on enter and destroys the host when it's damaged.
#[test]
fn sporogenic_infection_edicts_then_destroys_on_damage() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // The opponent has the host plus a spare to lose to the edict.
    let host = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::sporogenic_infection());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(host)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Sporogenic Infection");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_creature()).count(),
        1,
        "opponent sacrificed one creature to the edict"
    );
    // Damage the host → destroyed.
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(host), 1, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let sbas = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&sbas);
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    assert!(g.battlefield_find(host).is_none(), "host destroyed after taking damage");
}

/// Under the Skin manifests dread and returns a permanent from the graveyard.
#[test]
fn under_the_skin_manifests_and_recurs() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::under_the_skin());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Under the Skin");
    drain_stack(&mut g);
    // A face-down 2/2 manifested onto the battlefield.
    let facedown = g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).count();
    assert_eq!(facedown, 1, "one manifested face-down creature");
    // The graveyard permanent came back to hand (net: -1 spell +1 return = hand0).
    assert_eq!(g.players[0].hand.len(), hand0, "returned a permanent from the graveyard");
}
