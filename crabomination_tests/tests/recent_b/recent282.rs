//! Functionality tests for `catalog::sets::decks::recent282`.

use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::{two_player_game, Target};

/// Elven Farsight draws when the top card (post-scry) is a creature.
#[test]
fn elven_farsight_reveals_and_draws_creature() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears()); // top after scry
    let hand = g.players[0].hand.len();
    g.resolve_effect(&catalog::elven_farsight().effect.clone(), &EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "revealed a creature and drew");
}

/// Eagle of Deliverance grants indestructibility and cantrips on a small target.
#[test]
fn eagle_of_deliverance_shields_and_draws() {
    let mut g = two_player_game();
    let eagle = g.add_card_to_battlefield(0, catalog::eagle_of_deliverance());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → power ≤ 2
    g.add_card_to_library(0, catalog::forest()); // something to draw
    let hand = g.players[0].hand.len();
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..EffectContext::for_ability(eagle, 0, None) };
    g.resolve_effect(&catalog::eagle_of_deliverance().triggered_abilities[0].effect.clone(), &ctx).unwrap();
    assert_eq!(
        g.battlefield_find(bear).unwrap().counters.get(&crabomination::card::CounterType::Indestructible).copied().unwrap_or(0),
        1,
        "indestructible counter placed",
    );
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew for a power-2 target");
}

/// Horses of the Bruinen bounces two creatures and tempts the Ring.
#[test]
fn horses_of_the_bruinen_bounces_and_tempts() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ctx = EffectContext { targets: vec![Target::Permanent(a), Target::Permanent(b)], ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&catalog::horses_of_the_bruinen().effect.clone(), &ctx).unwrap();
    assert!(!g.battlefield.iter().any(|c| c.id == a || c.id == b), "both creatures bounced");
    assert_eq!(g.players[1].hand.len(), 2, "returned to owner's hand");
    assert!(g.players[0].ring_temptations > 0, "the Ring tempted");
}
