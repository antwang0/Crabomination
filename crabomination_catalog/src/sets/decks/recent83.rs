//! Walls + defender-ramp batch: vanilla/keyword walls, defender mana dorks
//! (Vine Trellis, Overgrown Battlement), a tapper (Dazzling Ramparts), an ETB
//! land tutor (Gatecreeper Vine), and a fog-with-lifegain (Blunt the Assault).
//! Tests in `tests/recent83.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, ManaPayload, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{cost, generic, g, u, w, Color};

/// A vanilla/keyword wall: 0/`tough` with the given types + keywords.
fn wall(name: &'static str, mana: &[crate::mana::ManaSymbol], tough: i32,
        types: Vec<CardType>, ctypes: Vec<CreatureType>, kw: Vec<Keyword>) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(mana),
        card_types: types,
        subtypes: Subtypes { creature_types: ctypes, ..Default::default() },
        power: 0,
        toughness: tough,
        keywords: kw,
        ..Default::default()
    }
}

/// Kraken Hatchling — {U} 0/4 Kraken. Vanilla.
pub fn kraken_hatchling() -> CardDefinition {
    wall("Kraken Hatchling", &[u()], 4, vec![CardType::Creature], vec![CreatureType::Kraken], vec![])
}

/// Angelic Wall — {1}{W} 0/4 Wall. Defender, flying.
pub fn angelic_wall() -> CardDefinition {
    wall("Angelic Wall", &[generic(1), w()], 4, vec![CardType::Creature],
        vec![CreatureType::Wall], vec![Keyword::Defender, Keyword::Flying])
}

/// Steel Wall — {1} 0/4 Artifact Wall. Defender.
pub fn steel_wall() -> CardDefinition {
    wall("Steel Wall", &[generic(1)], 4, vec![CardType::Artifact, CardType::Creature],
        vec![CreatureType::Wall], vec![Keyword::Defender])
}

/// Fortified Rampart — {1}{W} 0/6 Wall. Defender.
pub fn fortified_rampart() -> CardDefinition {
    wall("Fortified Rampart", &[generic(1), w()], 6, vec![CardType::Creature],
        vec![CreatureType::Wall], vec![Keyword::Defender])
}

/// Dazzling Ramparts — {4}{W} 0/7 Wall. Defender. {1}{W}, {T}: Tap target
/// creature.
pub fn dazzling_ramparts() -> CardDefinition {
    let mut c = wall("Dazzling Ramparts", &[generic(4), w()], 7, vec![CardType::Creature],
        vec![CreatureType::Wall], vec![Keyword::Defender]);
    c.activated_abilities = vec![ActivatedAbility {
        mana_cost: cost(&[generic(1), w()]),
        tap_cost: true,
        effect: Effect::Tap { what: target_filtered(R::Creature) },
        ..Default::default()
    }];
    c
}

/// Vine Trellis — {1}{G} 0/4 Plant Wall. Defender. {T}: Add {G}.
pub fn vine_trellis() -> CardDefinition {
    let mut c = wall("Vine Trellis", &[generic(1), g()], 4, vec![CardType::Creature],
        vec![CreatureType::Plant, CreatureType::Wall], vec![Keyword::Defender]);
    c.activated_abilities = vec![ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Green, Value::Const(1)) },
        ..Default::default()
    }];
    c
}

/// Overgrown Battlement — {1}{G} 0/4 Wall. Defender. {T}: Add {G} for each
/// creature you control with defender.
pub fn overgrown_battlement() -> CardDefinition {
    let mut c = wall("Overgrown Battlement", &[generic(1), g()], 4, vec![CardType::Creature],
        vec![CreatureType::Wall], vec![Keyword::Defender]);
    c.activated_abilities = vec![ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColor(Color::Green, Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
                filter: R::HasKeyword(Keyword::Defender),
            }),
        },
        ..Default::default()
    }];
    c
}

/// Gatecreeper Vine — {1}{G} 0/2 Plant. Defender. ETB: you may search your
/// library for a basic land card, reveal it, put it into your hand, then
/// shuffle. (The "or a Gate card" clause is dropped.)
pub fn gatecreeper_vine() -> CardDefinition {
    let mut c = wall("Gatecreeper Vine", &[generic(1), g()], 2, vec![CardType::Creature],
        vec![CreatureType::Plant], vec![Keyword::Defender]);
    c.triggered_abilities = vec![etb(Effect::Search {
        who: PlayerRef::You,
        filter: R::IsBasicLand,
        to: ZoneDest::Hand(PlayerRef::You),
    })];
    c
}

/// Blunt the Assault — {3}{G} Instant. You gain 1 life for each creature on the
/// battlefield. Prevent all combat damage that would be dealt this turn.
pub fn blunt_the_assault() -> CardDefinition {
    CardDefinition {
        name: "Blunt the Assault",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(R::Creature)),
                    filter: R::Any,
                },
            },
            Effect::PreventAllCombatDamageThisTurn,
        ]),
        ..Default::default()
    }
}
