//! Functionality tests for `catalog::sets::decks::recent60` — deferred
//! follow-ups cleared with fresh primitives.

use crabomination::card::{CardType, CreatureType, Subtypes};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::mana::Color;

fn vanilla_creature(name: &'static str) -> crabomination::card::CardDefinition {
    crabomination::card::CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

#[test]
fn jolrael_makes_cat_on_your_second_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::jolrael_mwonvuli_recluse());
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].cards_drawn_this_turn = 0;
    // First draw — no token.
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Cat").count(), 0);
    // Second draw — one Cat.
    let mut ev2 = vec![];
    g.draw_one(0, &mut ev2);
    g.dispatch_triggers_for_events(&ev2);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Cat").count(), 1);
}

#[test]
fn jolrael_sets_team_base_pt_to_hand_size() {
    let mut g = two_player_game();
    let jol = g.add_card_to_battlefield(0, catalog::jolrael_mwonvuli_recluse());
    let bear = g.add_card_to_battlefield(0, vanilla_creature("Bear"));
    for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); }
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: jol,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    // 3 cards in hand → each creature you control is base 3/3.
    for id in [jol, bear] {
        let c = cp.iter().find(|c| c.id == id).unwrap();
        assert_eq!((c.power, c.toughness), (3, 3), "creature became 3/3");
    }
}

#[test]
fn loyal_warhound_fetches_only_when_behind_on_lands() {
    // Behind on lands → fetch a basic Plains onto the battlefield tapped.
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_battlefield(1, catalog::forest()); }
    let plains = g.add_card_to_library(0, catalog::plains());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(plains))]));
    let wh = g.add_card_to_battlefield(0, catalog::loyal_warhound());
    g.fire_self_etb_triggers(wh, 0);
    drain_stack(&mut g);
    let fetched = g.battlefield_find(plains).expect("Plains fetched to battlefield");
    assert!(fetched.tapped, "fetched Plains enters tapped");
    assert_eq!(fetched.controller, 0);
}

#[test]
fn loyal_warhound_no_fetch_when_not_behind() {
    let mut g = two_player_game();
    // Even land counts (0 each) → not "an opponent controls more".
    let plains = g.add_card_to_library(0, catalog::plains());
    let wh = g.add_card_to_battlefield(0, catalog::loyal_warhound());
    g.fire_self_etb_triggers(wh, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(plains).is_none(), "no fetch when not behind on lands");
}

#[test]
fn well_of_lost_dreams_pays_to_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::well_of_lost_dreams());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    // Float 3 mana, pay 2 of it to draw 2 (leaving 1 unspent).
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    let hand = g.players[0].hand.len();
    g.adjust_life(0, 3);
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 3 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "drew X=2 cards");
    assert_eq!(g.players[0].mana_pool.total(), 1, "spent 2 of the 3 floated mana");
}

#[test]
fn custodi_soulbinders_counters_and_spirit_activation() {
    let mut g = two_player_game();
    // Two other creatures on the battlefield (one each side) → enters 2/2.
    g.add_card_to_battlefield(0, vanilla_creature("Ally"));
    g.add_card_to_battlefield(1, vanilla_creature("Foe"));
    // Enter through the real ETB funnel so enters-with-counters fires.
    let cs = g.move_card_to_battlefield_for_test(0, catalog::custodi_soulbinders());
    let c = g.compute_battlefield();
    let cc = c.iter().find(|c| c.id == cs).unwrap();
    assert_eq!((cc.power, cc.toughness), (2, 2), "enters with 2 +1/+1 counters");
    // Remove a counter to mint a Spirit.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cs,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count(), 1);
    let c2 = g.compute_battlefield();
    let cc2 = c2.iter().find(|c| c.id == cs).unwrap();
    assert_eq!((cc2.power, cc2.toughness), (1, 1), "one counter removed → 1/1");
}
