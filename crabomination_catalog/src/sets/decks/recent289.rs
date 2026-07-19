//! OTJ gap batch — Vehicles (Luxurious Locomotive rides the new `crewed_by`
//! tracker + `Value::SourceCrewerCount`; Mobile Homestead rides the new
//! `Effect::LookTopMayDeployLand`) + Wylie Duke (becomes-tapped value).
//! Tests in `recent_b/recent289`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CreatureType, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, StaticEffect,
};
use crate::game::effects::treasure_token;
use crate::mana::{cost, g, generic, w};

fn vehicle() -> Subtypes {
    Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() }
}

/// Luxurious Locomotive — {5} Artifact — Vehicle 6/5. Crew 1. When it attacks,
/// create a Treasure for each creature that crewed it this turn.
pub fn luxurious_locomotive() -> CardDefinition {
    CardDefinition {
        name: "Luxurious Locomotive",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        subtypes: vehicle(),
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::SourceCrewerCount,
                definition: treasure_token(),
            },
        }],
        ..Default::default()
    }
}

/// Mobile Homestead — {2} Artifact — Vehicle 3/3. Crew 2. Haste while you
/// control a Mount. When it attacks, look at the top card; if it's a land you
/// may put it onto the battlefield tapped.
pub fn mobile_homestead() -> CardDefinition {
    let controls_mount = Predicate::ValueAtLeast(
        Value::CountOf(Box::new(Selector::EachPermanent(
            R::HasCreatureType(CreatureType::Mount).and(R::ControlledByYou),
        ))),
        Value::ONE,
    );
    CardDefinition {
        name: "Mobile Homestead",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: vehicle(),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Crew(2)],
        static_abilities: vec![StaticAbility {
            description: "This Vehicle has haste as long as you control a Mount.",
            effect: StaticEffect::SelfHasKeywordIf { keyword: Keyword::Haste, condition: controls_mount },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::LookTopMayDeployLand { tapped: true },
        }],
        ..Default::default()
    }
}

/// Wylie Duke, Atiin Hero — {1}{G}{W} Legendary Creature — Human Ranger 4/2.
/// Vigilance. Whenever Wylie Duke becomes tapped, you gain 1 life and draw a
/// card.
pub fn wylie_duke_atiin_hero() -> CardDefinition {
    CardDefinition {
        name: "Wylie Duke, Atiin Hero",
        cost: cost(&[generic(1), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ranger],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}
