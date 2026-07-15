//! Functionality tests for `catalog::sets::decks::recent178`.

use crabomination::card::{ArtifactSubtype, Keyword};
use crabomination::catalog;
use crabomination::game::*;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Marching Duodrone makes a Treasure for each player when it attacks.
#[test]
fn marching_duodrone_treasures_each_player_on_attack() {
    let mut g = two_player_game();
    let drone = g.add_card_to_battlefield(0, catalog::marching_duodrone());
    g.clear_sickness(drone);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: drone,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let treasures = g
        .battlefield
        .iter()
        .filter(|c| c.definition.subtypes.artifact_subtypes.contains(&ArtifactSubtype::Treasure))
        .count();
    assert_eq!(treasures, 2, "one Treasure per player");
}

/// Fiendish Panda gains a +1/+1 counter on lifegain and reanimates a small
/// non-Bear creature when it dies.
#[test]
fn fiendish_panda_counter_and_reanimate() {
    let mut g = two_player_game();
    let panda = g.add_card_to_battlefield(0, catalog::fiendish_panda());
    g.adjust_life(0, 1);
    g.dispatch_triggers_for_events(&[GameEvent::LifeGained { player: 0, amount: 1 }]);
    drain_stack(&mut g);
    let counters = *g.battlefield_find(panda).unwrap().counters.get(&crabomination::card::CounterType::PlusOnePlusOne).unwrap_or(&0);
    assert_eq!(counters, 1, "lifegain adds a counter");
    // A 2-MV bear-free creature in the graveyard is returnable (Panda power 4).
    let mut target = catalog::grizzly_bears(); // {1}{G} = MV 2, but it's a Bear...
    target.name = "Small Elf";
    target.subtypes.creature_types = vec![crabomination::card::CreatureType::Elf];
    let dead = g.add_card_to_graveyard(0, target);
    // Kill the Panda → death trigger reanimates the Elf.
    let snap = g.battlefield_find(panda).unwrap().clone();
    g.remove_to_graveyard_with_triggers(panda);
    g.died_card_snapshots.insert(panda, snap);
    g.dispatch_triggers_for_events(&[GameEvent::CreatureDied { card_id: panda }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "reanimated the non-Bear creature");
}

/// Quick-Draw Katana grants +2/+0 always and first strike during your turn.
#[test]
fn quick_draw_katana_during_turn_bonus() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let katana = g.add_card_to_battlefield(0, catalog::quick_draw_katana());
    g.battlefield_find_mut(katana).unwrap().attached_to = Some(bear);
    g.active_player_idx = 0;
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "+2/+0 from the katana");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "first strike on your turn");
}

/// Salvation Swan blinks a nonflying creature you control when a Bird enters.
#[test]
fn salvation_swan_blinks_nonflyer() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::salvation_swan());
    let ground = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // no flying
    // Another Bird entering triggers the Swan's blink on the ground creature.
    let mut bird = catalog::grizzly_bears();
    bird.name = "Test Bird";
    bird.subtypes.creature_types = vec![crabomination::card::CreatureType::Bird];
    let entered = g.add_card_to_battlefield(0, bird);
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: entered }]);
    drain_stack(&mut g);
    // The ground creature was exiled (to return at end step) — gone for now.
    assert!(g.battlefield_find(ground).is_none(), "nonflyer exiled by the blink");
    assert!(g.exile.iter().any(|c| c.id == ground), "waiting in exile to return");
}
