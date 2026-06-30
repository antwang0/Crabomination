//! Functionality tests for `catalog::sets::decks::recent36` — ramp, tokens,
//! graveyard-fill commons, and the city's-blessing combat gate.

use crate::card::Keyword;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::mana::Color;
use crate::game::two_player_game;
use crate::game::*;

fn drain_dies(g: &mut GameState, id: CardId) {
    g.battlefield_find_mut(id).unwrap().damage = 999;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

#[test]
fn hour_of_promise_fetches_two_lands_tapped() {
    let mut g = two_player_game();
    let hop = g.add_card_to_hand(0, catalog::hour_of_promise());
    let l1 = g.add_card_to_library(0, catalog::forest());
    let l2 = g.add_card_to_library(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(l1)), DecisionAnswer::Search(Some(l2)),
    ]));
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: hop, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hour of Promise");
    drain_stack(&mut g);
    assert!(g.battlefield_find(l1).is_some_and(|c| c.tapped));
    assert!(g.battlefield_find(l2).is_some_and(|c| c.tapped));
}

#[test]
fn pirs_whim_makes_opponent_sacrifice() {
    let mut g = two_player_game();
    let whim = g.add_card_to_hand(0, catalog::pirs_whim());
    let land = g.add_card_to_library(0, catalog::forest());
    let art = g.add_card_to_battlefield(1, catalog::ornithopter());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(land))]));
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: whim, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Pir's Whim");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some(), "we fetched a land");
    assert!(g.battlefield_find(art).is_none(), "opponent sacrificed their artifact");
}

#[test]
fn wayward_swordtooth_gated_until_city_blessing() {
    let mut g = two_player_game();
    let dino = g.add_card_to_battlefield(0, catalog::wayward_swordtooth());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.battlefield_find_mut(dino).unwrap().summoning_sick = false;
    assert!(!g.legal_attackers(0).contains(&dino), "gated without the city's blessing");
    g.players[0].city_blessing = true;
    assert!(g.legal_attackers(0).contains(&dino), "can attack with the city's blessing");
}

#[test]
fn wayward_swordtooth_grants_extra_land() {
    let g = two_player_game();
    assert!(catalog::wayward_swordtooth().static_abilities.iter()
        .any(|s| matches!(s.effect, crate::card::StaticEffect::ExtraLandPerTurn)));
    let _ = g;
}

#[test]
fn gather_the_pack_takes_a_creature() {
    let mut g = two_player_game();
    let gtp = g.add_card_to_hand(0, catalog::gather_the_pack());
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: gtp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Gather the Pack");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature to hand");
}

#[test]
fn trackers_instincts_has_flashback() {
    assert!(catalog::trackers_instincts().keywords.iter()
        .any(|k| matches!(k, Keyword::Flashback(_))));
}

#[test]
fn dictate_of_kruphix_draws_extra_on_draw_step() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dictate_of_kruphix());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    g.active_player_idx = 0;
    let hand_before = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "active player drew the extra card");
}

#[test]
fn mogg_flunkies_cant_act_alone() {
    assert!(catalog::mogg_flunkies().keywords.contains(&Keyword::CantAttackOrBlockAlone));
}

#[test]
fn wily_goblin_makes_a_treasure() {
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::wily_goblin());
    g.fire_self_etb_triggers(gob, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "Treasure minted");
}

#[test]
fn hunted_witness_leaves_a_lifelink_soldier() {
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::hunted_witness());
    drain_dies(&mut g, w);
    let sol = g.battlefield.iter().find(|c| c.definition.name == "Soldier").expect("Soldier token");
    assert!(sol.definition.keywords.contains(&Keyword::Lifelink));
}

#[test]
fn brindle_shoat_leaves_a_three_three_boar() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::brindle_shoat());
    drain_dies(&mut g, s);
    let boar = g.battlefield.iter().find(|c| c.definition.name == "Boar").expect("Boar token");
    assert_eq!((boar.power(), boar.toughness()), (3, 3));
}

#[test]
fn goblin_assault_mints_a_hasty_goblin_each_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goblin_assault());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let gob = g.battlefield.iter().find(|c| c.definition.name == "Goblin").expect("Goblin token");
    assert!(gob.definition.keywords.contains(&Keyword::Haste));
}

#[test]
fn goblin_rally_makes_four() {
    let mut g = two_player_game();
    let rally = g.add_card_to_hand(0, catalog::goblin_rally());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: rally, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Goblin Rally");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count(), 4);
}

#[test]
fn bottomless_pit_discards_at_each_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bottomless_pit());
    for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); }
    g.active_player_idx = 0;
    let hand_before = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "active player discarded one");
}
