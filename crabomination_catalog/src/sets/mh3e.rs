//! Modern Horizons 3 (MH3), batch 5. The ten-card "Landscape" common land
//! cycle (colorless tap + sac-fetch three basics tapped + tricolor cycling),
//! plus a handful of energy/replicate/aura payoffs. Tests in `tests/mh3e.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, generic, u, w, ManaSymbol, g, r};

/// One member of the MH3 "Landscape" cycle: `{T}: Add {C}`; `{T}, Sacrifice:
/// fetch a basic of one of three types onto the battlefield tapped`; Cycling
/// for its three colors.
fn landscape(name: &'static str, types: [LandType; 3], cycle: &[ManaSymbol]) -> CardDefinition {
    let filter = R::IsBasicLand.and(
        R::HasLandType(types[0])
            .or(R::HasLandType(types[1]))
            .or(R::HasLandType(types[2])),
    );
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::Cycling(cost(cycle))],
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

use LandType::{Forest as F, Island as I, Mountain as M, Plains as P, Swamp as S};

pub fn bountiful_landscape() -> CardDefinition {
    landscape("Bountiful Landscape", [F, I, M], &[g(), u(), r()])
}
pub fn contaminated_landscape() -> CardDefinition {
    landscape("Contaminated Landscape", [P, I, S], &[w(), u(), b()])
}
pub fn deceptive_landscape() -> CardDefinition {
    landscape("Deceptive Landscape", [P, S, F], &[w(), b(), g()])
}
pub fn foreboding_landscape() -> CardDefinition {
    landscape("Foreboding Landscape", [S, F, I], &[b(), g(), u()])
}
pub fn perilous_landscape() -> CardDefinition {
    landscape("Perilous Landscape", [I, M, P], &[u(), r(), w()])
}
pub fn seething_landscape() -> CardDefinition {
    landscape("Seething Landscape", [I, S, M], &[u(), b(), r()])
}
pub fn shattered_landscape() -> CardDefinition {
    landscape("Shattered Landscape", [M, P, S], &[r(), w(), b()])
}
pub fn sheltering_landscape() -> CardDefinition {
    landscape("Sheltering Landscape", [M, F, P], &[r(), g(), w()])
}
pub fn tranquil_landscape() -> CardDefinition {
    landscape("Tranquil Landscape", [F, P, I], &[g(), w(), u()])
}
pub fn twisted_landscape() -> CardDefinition {
    landscape("Twisted Landscape", [S, M, F], &[b(), r(), g()])
}

/// Vega, the Watcher — {1}{W}{U} 2/2 Bird Spirit with flying. Whenever you cast
/// a spell from anywhere other than your hand, draw a card.
pub fn vega_the_watcher() -> CardDefinition {
    CardDefinition {
        name: "Vega, the Watcher",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::SpellNotCastFromHand,
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}
