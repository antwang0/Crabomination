//! Functionality tests for `catalog::sets::decks::recent150` (OTJ/DSK/WOE wave).

use crate::card::Keyword;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::Target;
use crate::game::*;
use crate::game::two_player_game;

fn fill_mana(g: &mut GameState) {
    for c in [
        crate::mana::Color::White,
        crate::mana::Color::Blue,
        crate::mana::Color::Black,
        crate::mana::Color::Red,
        crate::mana::Color::Green,
    ] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Consuming Ashes exiles a target creature (and surveils when MV ≤ 3).
#[test]
fn consuming_ashes_exiles_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let id = g.add_card_to_hand(0, catalog::consuming_ashes());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Consuming Ashes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature exiled");
}

/// Failed Fording bounces a nonland permanent to its owner's hand.
#[test]
fn failed_fording_bounces_permanent() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::failed_fording());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Failed Fording");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "bounced off the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "returned to owner's hand");
}

/// Harrier Strix taps a permanent on entry.
#[test]
fn harrier_strix_taps_on_etb() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::harrier_strix());
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).unwrap().tapped, "ETB tapped the target");
}

/// Irascible Wolverine exiles the top card and lets you play it this turn.
#[test]
fn irascible_wolverine_impulse_top() {
    let mut g = two_player_game();
    let top = g.next_id();
    g.players[0].add_to_library_top(top, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::irascible_wolverine());
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == top), "top card exiled for impulse play");
}

/// Killer's Mask manifests a dread creature and equips it (granting menace).
#[test]
fn killers_mask_manifest_and_equip() {
    let mut g = two_player_game();
    let id1 = g.next_id();
    g.players[0].add_to_library_top(id1, catalog::grizzly_bears());
    let id2 = g.next_id();
    g.players[0].add_to_library_top(id2, catalog::forest());
    let mask = g.move_card_to_battlefield_for_test(0, catalog::killers_mask());
    drain_stack(&mut g);
    // A 2/2 face-down manifest exists and the Equipment is attached to it.
    let manifest = g.battlefield.iter().find(|c| c.face_down && c.controller == 0).map(|c| c.id);
    assert!(manifest.is_some(), "manifested a face-down creature");
    let attached_to = g.battlefield_find(mask).unwrap().attached_to;
    assert_eq!(attached_to, manifest, "Equipment attached to the manifest");
    assert!(g.computed_permanent(manifest.unwrap()).unwrap().keywords.contains(&Keyword::Menace),
        "equipped creature has menace");
}

/// Jump Scare gives +2/+2 and flying until end of turn.
#[test]
fn jump_scare_pumps_and_flies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::jump_scare());
    fill_mana(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Jump Scare");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "+2/+2");
    assert!(c.keywords.contains(&Keyword::Flying), "gained flying");
}

/// Expel the Interlopers destroys creatures with power ≥ the chosen number.
#[test]
fn expel_the_interlopers_destroys_by_chosen_power() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    let small = g.add_card_to_battlefield(0, catalog::savannah_lions());  // 2/1
    let id = g.add_card_to_hand(0, catalog::expel_the_interlopers());
    fill_mana(&mut g);
    // Choose 4 → only power-4+ creatures die.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(4)]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Expel the Interlopers");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "power-6 creature destroyed");
    assert!(g.battlefield_find(small).is_some(), "power-2 creature survives the chosen 4");
}
