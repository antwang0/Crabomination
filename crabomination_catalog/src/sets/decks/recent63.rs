//! Green midrange batch: a */* avatar, landfall payoffs, enrage, pump spells,
//! and a Persist bomb. Tests in `tests/recent63.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, DynamicPt, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{cost, g, generic};

/// Scion of the Wild — {1}{G}{G} */* Avatar; power and toughness each equal the
/// number of creatures you control.
pub fn scion_of_the_wild() -> CardDefinition {
    CardDefinition {
        name: "Scion of the Wild",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Avatar], ..Default::default() },
        dynamic_pt: Some(DynamicPt::CreaturesControlled { base: 0 }),
        ..Default::default()
    }
}

/// Grazing Gladehart — {2}{G} 2/2 Antelope. Landfall — whenever a land you
/// control enters, you may gain 2 life.
pub fn grazing_gladehart() -> CardDefinition {
    CardDefinition {
        name: "Grazing Gladehart",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Antelope], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Gain 2 life".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
            },
        }],
        ..Default::default()
    }
}

/// Snapping Sailback — {4}{G} 4/4 Dinosaur with flash. Enrage — whenever it's
/// dealt damage, put a +1/+1 counter on it.
pub fn snapping_sailback() -> CardDefinition {
    CardDefinition {
        name: "Snapping Sailback",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Baloth Woodcrasher — {4}{G}{G} 4/4 Beast. Landfall — whenever a land you
/// control enters, it gets +4/+4 and gains trample until end of turn.
pub fn baloth_woodcrasher() -> CardDefinition {
    CardDefinition {
        name: "Baloth Woodcrasher",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Kavu Climber — {3}{G}{G} 3/3 Kavu. When it enters, draw a card.
pub fn kavu_climber() -> CardDefinition {
    CardDefinition {
        name: "Kavu Climber",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kavu], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        ..Default::default()
    }
}

/// Might of Oaks — {3}{G} Instant. Target creature gets +7/+7 until end of turn.
pub fn might_of_oaks() -> CardDefinition {
    CardDefinition {
        name: "Might of Oaks",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(7),
            toughness: Value::Const(7),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Wildsize — {2}{G} Instant. Target creature gets +2/+2 and gains trample until
/// end of turn. Draw a card.
pub fn wildsize() -> CardDefinition {
    CardDefinition {
        name: "Wildsize",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Broken Bond — {1}{G} Sorcery. Destroy target artifact or enchantment. You may
/// put a land card from your hand onto the battlefield.
pub fn broken_bond() -> CardDefinition {
    CardDefinition {
        name: "Broken Bond",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        // The land-put is inherently optional ("you may") — no MayDo wrapper.
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Land,
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
            },
        ]),
        ..Default::default()
    }
}

/// Woodfall Primus — {5}{G}{G}{G} 6/6 Treefolk Shaman with trample and Persist.
/// ETB: destroy target noncreature permanent.
pub fn woodfall_primus() -> CardDefinition {
    CardDefinition {
        name: "Woodfall Primus",
        cost: cost(&[generic(5), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Shaman],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample, Keyword::Persist],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::Noncreature),
        })],
        ..Default::default()
    }
}
