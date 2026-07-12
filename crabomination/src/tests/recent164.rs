//! Functionality tests for `catalog::sets::decks::recent164` (Foundations).

use crate::catalog;
use crate::card::Keyword;
use crate::game::types::{Attack, AttackTarget, Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

/// Fleeting Flight prevents combat damage to the buffed creature — it survives a
/// bigger attacker in combat while still dealing its own.
#[test]
fn fleeting_flight_prevents_incoming_combat_damage() {
    let mut g = two_player_game();
    // Defender's 2/3 (after the counter) blocks an attacking 4/4.
    let attacker = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 flying
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(1, catalog::fleeting_flight());
    g.players[1].mana_pool.add(Color::White, 1);
    // Player 1 casts Fleeting Flight on their blocker (at instant speed).
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(blocker)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Fleeting Flight");
    drain_stack(&mut g);
    assert!(g.computed_permanent(blocker).unwrap().keywords.contains(&Keyword::Flying), "gained flying");
    // Now resolve combat: attacker (4/4) is blocked by the 3/3 blocker.
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }]))
    .unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).unwrap();
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    // The blocker took no combat damage (prevented), so it survives; it still
    // dealt its 3 to the 4/4.
    assert!(g.battlefield_find(blocker).is_some(), "blocker survives — all combat damage to it was prevented");
    assert_eq!(g.battlefield_find(blocker).map(|c| c.damage), Some(0), "no damage marked");
    assert_eq!(g.battlefield_find(attacker).map(|c| c.damage), Some(3), "attacker still took the blocker's 3");
}

/// Goblin Negotiation makes a Goblin for each point of excess damage.
#[test]
fn goblin_negotiation_makes_goblins_from_excess() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::goblin_negotiation());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    // X = 5 → 5 damage to a 2/2 → 3 excess → 3 Goblins.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![], mode: None, x_value: Some(5),
    })
    .expect("cast Goblin Negotiation for X=5");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "the 2/2 died");
    let goblins = g.battlefield.iter().filter(|c| c.definition.name == "Goblin" && c.controller == 0).count();
    assert_eq!(goblins, 3, "3 excess damage → 3 Goblins");
}

/// Homunculus Horde copies itself on the second draw each turn.
#[test]
fn homunculus_horde_copies_on_second_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::homunculus_horde());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    // First draw of the turn — no trigger.
    let mut evs = vec![];
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Homunculus Horde").count(), 1, "no copy on the first draw");
    // Second draw — trigger mints a copy.
    let mut evs = vec![];
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Homunculus Horde").count(), 2, "second draw minted a copy");
}
