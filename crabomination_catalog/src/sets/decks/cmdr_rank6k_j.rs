//! Commander: three-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), tenth file. Tests in `tests/recent_b/cmdr_rank6k_j.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, Selector};
use crate::mana::{b, cost, generic};

/// M.O.D.O.K., Evil Intellect — {3}{B}{B} 3/5 legendary Villain artifact
/// creature, flying. Whenever you draw your second card each turn, target
/// opponent sacrifices a nontoken creature of their choice.
pub fn m_o_d_o_k_evil_intellect() -> CardDefinition {
    CardDefinition {
        name: "M.O.D.O.K., Evil Intellect",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Villain], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: Effect::Sacrifice {
                who: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: R::Creature.and(R::NotToken),
            },
        }],
        ..Default::default()
    }
}

/// Moment of Defiance — {2}{B} instant. Target creature gets +2/+1 and gains
/// lifelink until end of turn. Draw a card.
pub fn moment_of_defiance() -> CardDefinition {
    CardDefinition {
        name: "Moment of Defiance",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Multiply by Zero — {1}{B} instant. Target creature has base power and
/// toughness 0/0 until end of turn.
pub fn multiply_by_zero() -> CardDefinition {
    CardDefinition {
        name: "Multiply by Zero",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::SetBasePT {
            what: target_filtered(R::Creature),
            power: Value::ZERO,
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}
