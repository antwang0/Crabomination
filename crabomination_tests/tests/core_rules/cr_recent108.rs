//! Resolution-time picks that stand in for a player's choice take the most
//! valuable object: `Selector::TakeGreatestManaValue` keeps the priciest
//! cards (ties in resolution order) — "choose a nonland card exiled this
//! way; you may cast it" wants the card worth casting, not the first one.

use crabomination::card::Zone;
use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crabomination::game::effects::EffectContext;
use crabomination::game::*;

#[test]
fn take_greatest_mana_value_keeps_the_priciest_cards() {
    let mut g = two_player_game();
    let cheap = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let big = g.add_card_to_graveyard(0, catalog::shivan_dragon());
    let mid = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let take = |count: i32| Effect::Move {
        what: Selector::TakeGreatestManaValue {
            inner: Box::new(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: crabomination::card::SelectionRequirement::Creature,
            }),
            count: Box::new(Value::Const(count)),
        },
        to: ZoneDest::Hand(PlayerRef::You),
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&take(1), &ctx).expect("resolve");
    assert!(g.players[0].hand.iter().any(|c| c.id == big), "Shivan Dragon (6) first");
    g.resolve_effect(&take(1), &ctx).expect("resolve");
    assert!(g.players[0].hand.iter().any(|c| c.id == mid), "then Craw Wurm (5)");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cheap), "the Bears stay");
}
