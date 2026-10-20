//! Commander: the 2026-10-20 (`01DDeUtc`) closers — Selenia, Dark Angel's
//! life-total resets. Tests in `tests/recent_b/cmdr_01dd_a.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, DynamicPt, EventKind, EventScope, EventSpec, Keyword, StaticAbility,
    StaticEffect, Subtypes, TriggeredAbility,
};
use crate::effect::{Effect, Selector, Value};
use crate::mana::{cost, generic, w};

fn angel(name: &'static str, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn you_have_hexproof() -> StaticAbility {
    StaticAbility { description: "You have hexproof.", effect: StaticEffect::ControllerHasHexproof }
}

/// Enduring Angel // Angelic Enforcer — {2}{W}{W}{W} 3/3 Angel, flying, double
/// strike; you have hexproof. If your life total would be reduced to 0 or
/// less, instead transform it and your life total becomes 3 (you lose if it
/// didn't transform). The back is a flying */* sized by your life total that
/// doubles your life total when it attacks.
pub fn enduring_angel() -> CardDefinition {
    let back = CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![you_have_hexproof()],
        dynamic_pt: Some(DynamicPt::ControllerLife),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DoubleLife { who: Selector::You },
        }],
        ..angel("Angelic Enforcer", 0, 0)
    };
    CardDefinition {
        cost: cost(&[generic(2), w(), w(), w()]),
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike],
        static_abilities: vec![
            you_have_hexproof(),
            StaticAbility {
                description: "If your life total would be reduced to 0 or less, instead transform this creature and your life total becomes 3.",
                effect: StaticEffect::LifeToZeroTransformsSelf { life: 3 },
            },
        ],
        back_face: Some(Box::new(back)),
        ..angel("Enduring Angel", 3, 3)
    }
}

/// Exquisite Archangel — {5}{W}{W} 5/5 Angel, flying. If you would lose the
/// game, instead exile it and your life total becomes your starting life total.
pub fn exquisite_archangel() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(5), w(), w()]),
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If you would lose the game, instead exile this creature and your life total becomes equal to your starting life total.",
            effect: StaticEffect::ReplaceControllerLossWithExileSelfAtStartingLife,
        }],
        ..angel("Exquisite Archangel", 5, 5)
    }
}

/// Oketra's Last Mercy — {1}{W}{W} sorcery. Your life total becomes your
/// starting life total. Lands you control don't untap during your next untap
/// step.
pub fn oketras_last_mercy() -> CardDefinition {
    CardDefinition {
        name: "Oketra's Last Mercy",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SetLifeTotal { who: Selector::You, amount: Value::StartingLifeTotal },
            Effect::LandsDontUntapNextUntapStep { who: Selector::You },
        ]),
        ..Default::default()
    }
}
