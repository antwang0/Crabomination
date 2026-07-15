//! Gap batch — DSK artifact/Nightmare value on existing primitives.
//! Tests in `tests/recent232.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate, Selector,
    Value, ZoneDest,
};
use crate::mana::{cost, generic, Color};

/// Haunted Screen — {3} Artifact. {T}: Add {W} or {B}. {T}, Pay 1 life: Add
/// {G}, {U}, or {R}. {7}: Put seven +1/+1 counters on this artifact. It becomes
/// a 0/0 Spirit creature in addition to its other types. Activate only once.
pub fn haunted_screen() -> CardDefinition {
    CardDefinition {
        name: "Haunted Screen",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::White, Color::Black], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(
                        vec![Color::Green, Color::Blue, Color::Red],
                        Value::ONE,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(7)]),
                activate_once: true,
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: crate::card::CounterType::PlusOnePlusOne,
                        amount: Value::Const(7),
                    },
                    Effect::AnimateAsCreature { what: Selector::This, duration: Duration::Permanent },
                    Effect::AddCreatureTypes {
                        what: Selector::This,
                        creature_types: vec![CreatureType::Spirit],
                        duration: Duration::Permanent,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Fear of Infinity — {1}{U}{B} 2/2 Enchantment Creature — Nightmare. Flying,
/// lifelink. This creature can't block. Eerie — Whenever an enchantment you
/// control enters and whenever you fully unlock a Room, you may return this card
/// from your graveyard to your hand.
pub fn fear_of_infinity() -> CardDefinition {
    let recur = || Effect::MayDo {
        description: "Return Fear of Infinity from your graveyard to your hand?".into(),
        body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
    };
    CardDefinition {
        name: "Fear of Infinity",
        cost: cost(&[generic(1), crate::mana::u(), crate::mana::b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink, Keyword::CantBlock],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Enchantment.and(R::ControlledByYou).and(R::OtherThanSource),
                    }),
                effect: recur(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::RoomFullyUnlocked, EventScope::FromYourGraveyard),
                effect: recur(),
            },
        ],
        ..Default::default()
    }
}
