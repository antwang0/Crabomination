//! The Lost Caverns of Ixalan (LCI) — 2023. Introduces the Discover
//! (CR 701.57) keyword action.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, SelectionRequirement, Selector,
    Subtypes, Value,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::game::effects::{map_token, treasure_token};
use crate::mana::{b, cost, g, generic, r, u, x};

/// Geological Appraiser — {2}{R}{R} 3/2 Human Artificer. "When this enters,
/// if you cast it, discover 3." (The "if you cast it" gate is approximated as
/// firing on any ETB — the engine doesn't tag cast-vs-put entries.)
pub fn geological_appraiser() -> CardDefinition {
    CardDefinition {
        name: "Geological Appraiser",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Discover { n: Value::Const(3) })],
        ..Default::default()
    }
}

/// Trumpeting Carnosaur — {4}{R}{R} 7/6 Dinosaur with trample. "When this
/// enters, discover 5." (The "{2}{R}, Discard this card: 3 damage" from-hand
/// ability is omitted — activated-from-hand abilities aren't modeled.)
pub fn trumpeting_carnosaur() -> CardDefinition {
    CardDefinition {
        name: "Trumpeting Carnosaur",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Discover { n: Value::Const(5) })],
        ..Default::default()
    }
}

/// Spyglass Siren — {U} 1/1 Siren Pirate with flying. "When this enters,
/// create a Map token." (Map tokens ship via `map_token()`.)
pub fn spyglass_siren() -> CardDefinition {
    CardDefinition {
        name: "Spyglass Siren",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Siren, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: map_token(),
        })],
        ..Default::default()
    }
}

/// Defossilize — {4}{B} Sorcery. "Return target creature card from your
/// graveyard to the battlefield. That creature explores, then it explores
/// again."
pub fn defossilize() -> CardDefinition {
    CardDefinition {
        name: "Defossilize",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::Explore { who: Selector::LastMoved },
            Effect::Explore { who: Selector::LastMoved },
        ]),
        ..Default::default()
    }
}

/// Goldvein Hydra — {X}{G} 0/0 Hydra with vigilance, trample, haste. Enters
/// with X +1/+1 counters. When it dies, create Treasures equal to its power
/// (its last-known counter-boosted power, via CR 603.10 leaves-battlefield LKI).
pub fn goldvein_hydra() -> CardDefinition {
    CardDefinition {
        name: "Goldvein Hydra",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hydra], ..Default::default() },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Haste],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::PowerOf(Box::new(Selector::This)),
            definition: treasure_token(),
        })],
        ..Default::default()
    }
}
