//! Functionality tests for `catalog::sets::decks::recent250` (control + protection
//! Auras).

use crabomination::card::{CreatureType, Keyword};
use crabomination::catalog;
use crabomination::game::{drain_stack, two_player_game};

/// Coerced to Kill steals the enchanted creature and makes it a 1/1 deathtouch
/// Assassin; control reverts when the Aura leaves.
#[test]
fn coerced_to_kill_steals_and_reshapes() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer
    let aura = g.add_card_to_battlefield(0, catalog::coerced_to_kill());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(victim);
    g.fire_self_etb_triggers(aura, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0, "stole control");
    let cp = g.computed_permanent(victim).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "base P/T 1/1");
    assert!(cp.keywords.contains(&Keyword::Deathtouch), "gains deathtouch");
    assert!(cp.subtypes.creature_types.contains(&CreatureType::Assassin), "is an Assassin");
    // Removing the Aura reverts control.
    g.remove_to_graveyard_with_triggers(aura);
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 1, "control reverts");
}

/// Airtight Alibi buffs +2/+2, untaps, grants hexproof, and clears suspicion.
#[test]
fn airtight_alibi_buffs_and_cleanses() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.battlefield_find_mut(bear).unwrap().suspected = true;
    let aura = g.add_card_to_battlefield(0, catalog::airtight_alibi());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    g.fire_self_etb_triggers(aura, 0);
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(!c.tapped, "untapped on ETB");
    assert!(!c.suspected, "no longer suspected");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Hexproof), "gains hexproof");
}
