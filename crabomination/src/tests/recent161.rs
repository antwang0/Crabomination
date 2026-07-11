//! Functionality tests for `catalog::sets::decks::recent161` (Foundations).

use crate::catalog;
use crate::game::types::{Attack, AttackTarget, Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn fill_mana(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 8);
    }
    g.players[0].mana_pool.add_colorless(8);
}

/// Incinerating Blast deals 6 to a creature.
#[test]
fn incinerating_blast_burns() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::incinerating_blast());
    fill_mana(&mut g);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Incinerating Blast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "6 damage killed the 4/4");
}

/// Needletooth Pack's Morbid grows a creature after a death.
#[test]
fn needletooth_pack_morbid_grows() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::needletooth_pack());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // A creature died this turn.
    let chump = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(chump);
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.priority.player_with_priority = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    // Two +1/+1 counters landed on a creature you control.
    let bumped = g.computed_permanent(ally).unwrap().power == 4
        || g.battlefield.iter().any(|c| c.definition.name == "Needletooth Pack" && c.controller == 0 && g.computed_permanent(c.id).unwrap().power == 6);
    assert!(bumped, "Morbid added two +1/+1 counters");
}

/// Grappling Kraken taps and stuns on landfall.
#[test]
fn grappling_kraken_landfall_stuns() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grappling_kraken());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    let c = g.battlefield_find(foe).unwrap();
    assert!(c.tapped, "landfall tapped the opponent's creature");
    assert!(c.counters.get(&crate::card::CounterType::Stun).copied().unwrap_or(0) >= 1, "stun counter placed");
}

/// Joust Through hits an attacking creature. (Player 0 swings, then burns their
/// own attacker during combat — exercises the attacking/blocking target filter.)
#[test]
fn joust_through_hits_attacker() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::serra_angel());
    g.clear_sickness(attacker);
    let id = g.add_card_to_hand(0, catalog::joust_through());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.players[0].mana_pool.add(Color::White, 1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(attacker)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Joust Through");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(attacker).map(|c| c.damage), Some(3), "3 damage to the attacker");
    assert_eq!(g.players[0].life, life + 1, "gained 1 life");
}

/// Quakestrider Ceratops is a 12/8 vanilla.
#[test]
fn quakestrider_ceratops_is_a_giant() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::quakestrider_ceratops());
    let p = g.computed_permanent(c).unwrap();
    assert_eq!((p.power, p.toughness), (12, 8));
}
