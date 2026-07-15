//! Functionality tests for `catalog::sets::decks::recent75`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::two_player_game;
use crabomination::game::*;

#[test]
fn fungusaur_grows_when_dealt_damage() {
    let mut g = two_player_game();
    let saur = g.add_card_to_battlefield(0, catalog::fungusaur());
    let mut events = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(saur), 1, None, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(saur).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "enrage placed a +1/+1 counter");
    let p = g.computed_permanent(saur).unwrap();
    assert_eq!((p.power, p.toughness), (3, 3), "2/2 → 3/3");
}

#[test]
fn serpent_warrior_costs_three_life() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let sw = g.add_card_to_hand(0, catalog::serpent_warrior());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: sw, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 3, "lost 3 life on ETB");
}

#[test]
fn nettletooth_djinn_pings_you_at_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nettletooth_djinn());
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "1 damage to you at upkeep");
}

#[test]
fn hulking_cyclops_cant_block() {
    let mut g = two_player_game();
    let cyc = g.add_card_to_battlefield(0, catalog::hulking_cyclops());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    assert!(!g.blocker_can_block_attacker(cyc, attacker), "can't block");
    assert!(catalog::pygmy_pyrosaur().keywords.contains(&Keyword::CantBlock));
}

#[test]
fn owl_familiar_loots_on_etb() {
    let mut g = two_player_game();
    let hand_before = g.players[0].hand.len();
    g.add_card_to_hand(0, catalog::grizzly_bears()); // something to discard
    let owl = g.add_card_to_hand(0, catalog::owl_familiar());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: owl, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    // Net hand size: -1 owl cast, +1 draw, -1 discard = hand_before (the pre-seeded bear).
    assert_eq!(g.players[0].hand.len(), hand_before, "draw then discard nets zero");
}

#[test]
fn recent75_static_stats() {
    assert!(catalog::ekundu_griffin().keywords.contains(&Keyword::Flying));
    assert!(catalog::ekundu_griffin().keywords.contains(&Keyword::FirstStrike));
    assert!(catalog::fire_drake().keywords.contains(&Keyword::Flying));
    assert_eq!((catalog::muck_rats().power, catalog::muck_rats().toughness), (1, 1));
}
