//! OTJ gap batch — Doc Aurlock (graveyard/exile/plot cost reductions via the new
//! `StaticEffect::{ExileCastCostReduction, PlotCostReduction}`).
//! Tests in `recent_b/recent288`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, StaticAbility, Subtypes, Supertype,
    TriggeredAbility,
};
use crate::effect::shortcut::etb;
use crate::effect::{
    DelayedTriggerKind, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate,
    StaticEffect, Value,
};
use crate::mana::{cost, g, generic, u, w};

/// Doc Aurlock, Grizzled Genius — {G}{U} Legendary Creature — Bear Druid 2/3.
/// Spells you cast from your graveyard or from exile cost {2} less; plotting
/// cards from your hand costs {2} less.
pub fn doc_aurlock_grizzled_genius() -> CardDefinition {
    let reduce = |effect: StaticEffect, description: &'static str| StaticAbility {
        description,
        effect,
    };
    CardDefinition {
        name: "Doc Aurlock, Grizzled Genius",
        cost: cost(&[g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![
            reduce(
                StaticEffect::GraveyardCastCostReduction { amount: 2 },
                "Spells you cast from your graveyard cost {2} less to cast.",
            ),
            reduce(
                StaticEffect::ExileCastCostReduction { amount: 2 },
                "Spells you cast from exile cost {2} less to cast.",
            ),
            reduce(
                StaticEffect::PlotCostReduction { amount: 2 },
                "Plotting cards from your hand costs {2} less.",
            ),
        ],
        ..Default::default()
    }
}

/// Fortune, Loyal Steed — {2}{W} Legendary Creature — Beast Mount 2/4. Saddle 1.
/// ETB scry 2. Whenever Fortune attacks while saddled, at end of combat exile
/// it and up to one creature that saddled it this turn, then return them to the
/// battlefield under their owners' control.
pub fn fortune_loyal_steed() -> CardDefinition {
    CardDefinition {
        name: "Fortune, Loyal Steed",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Mount],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Saddle(1)],
        triggered_abilities: vec![
            etb(Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                    .with_filter(Predicate::SourceSaddled),
                effect: Effect::DelayUntil {
                    kind: DelayedTriggerKind::EndOfCombat,
                    body: Box::new(Effect::ExileAndReturnSelfWithSaddler),
                },
            },
        ],
        ..Default::default()
    }
}
