//! Functionality tests for `catalog::sets::decks::recent63` — green midrange.

use crate::card::{CardType, CreatureType, Keyword, Subtypes};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Target, TurnStep};
use crate::game::*;

fn bear(name: &'static str) -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

fn cast_instant_at(g: &mut GameState, controller: usize, id: CardId, target: CardId) {
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = controller;
    g.priority.player_with_priority = controller;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("cast");
    drain_stack(g);
}

#[test]
fn scion_of_the_wild_scales_with_creatures() {
    let mut g = two_player_game();
    let scion = g.add_card_to_battlefield(0, catalog::scion_of_the_wild());
    g.add_card_to_battlefield(0, bear("A"));
    g.add_card_to_battlefield(0, bear("B"));
    let c = g.compute_battlefield();
    let s = c.iter().find(|c| c.id == scion).unwrap();
    // Scion + 2 bears = 3 creatures you control.
    assert_eq!((s.power, s.toughness), (3, 3));
}

#[test]
fn grazing_gladehart_landfall_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grazing_gladehart());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let land = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "landfall gained 2");
}

#[test]
fn snapping_sailback_enrages() {
    let mut g = two_player_game();
    let sb = g.add_card_to_battlefield(0, catalog::snapping_sailback());
    g.dispatch_triggers_for_events(&[GameEvent::DamageDealt {
        amount: 2, to_card: Some(sb), to_player: None,
    }]);
    drain_stack(&mut g);
    let c = g.compute_battlefield();
    let s = c.iter().find(|c| c.id == sb).unwrap();
    assert_eq!((s.power, s.toughness), (5, 5), "one +1/+1 counter from enrage");
}

#[test]
fn baloth_woodcrasher_landfall_pumps() {
    let mut g = two_player_game();
    let baloth = g.add_card_to_battlefield(0, catalog::baloth_woodcrasher());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    let c = g.compute_battlefield();
    let b = c.iter().find(|c| c.id == baloth).unwrap();
    assert_eq!((b.power, b.toughness), (8, 8), "4/4 → 8/8 on landfall");
    assert!(b.keywords.contains(&Keyword::Trample));
}

#[test]
fn kavu_climber_draws_on_enter() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    let id = g.add_card_to_battlefield(0, catalog::kavu_climber());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

#[test]
fn might_of_oaks_pumps_seven() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, bear("Grunt"));
    let spell = g.add_card_to_hand(0, catalog::might_of_oaks());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast_instant_at(&mut g, 0, spell, target);
    let c = g.compute_battlefield();
    let t = c.iter().find(|c| c.id == target).unwrap();
    assert_eq!((t.power, t.toughness), (9, 9), "2/2 + 7/7");
}

#[test]
fn wildsize_pumps_tramples_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let target = g.add_card_to_battlefield(0, bear("Grunt"));
    let spell = g.add_card_to_hand(0, catalog::wildsize());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    cast_instant_at(&mut g, 0, spell, target);
    let c = g.compute_battlefield();
    let t = c.iter().find(|c| c.id == target).unwrap();
    assert_eq!((t.power, t.toughness), (4, 4), "2/2 + 2/2");
    assert!(t.keywords.contains(&Keyword::Trample));
    // -1 cast + 1 draw = net unchanged vs pre-cast hand.
    assert_eq!(g.players[0].hand.len(), hand);
}

#[test]
fn broken_bond_destroys_and_ramps() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let land = g.add_card_to_hand(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::broken_bond());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Destroy target passed in the cast; the land pick is the only prompt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![land])]));
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(art)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    assert!(g.battlefield_find(land).is_some(), "land put onto the battlefield");
}

#[test]
fn woodfall_primus_destroys_noncreature_and_has_persist() {
    let def = catalog::woodfall_primus();
    assert!(def.keywords.contains(&Keyword::Persist) && def.keywords.contains(&Keyword::Trample));
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(art))]));
    let wp = g.add_card_to_battlefield(0, catalog::woodfall_primus());
    g.fire_self_etb_triggers(wp, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "ETB destroyed the noncreature");
}

