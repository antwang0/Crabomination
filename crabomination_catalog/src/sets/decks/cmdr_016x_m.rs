//! Commander: 2026-10-19 (`016Xh16H`) — Meng Huo, Barbarian King's green
//! Portal Three Kingdoms army. Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

/// Meng Huo, Barbarian King — {3}{G}{G} 4/4 legendary Human Barbarian
/// Soldier; other green creatures you control get +1/+1.
pub fn meng_huo_barbarian_king() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Other green creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasColor(Color::Green)).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..creature(
            "Meng Huo, Barbarian King",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Human, CreatureType::Barbarian, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Meng Huo's Horde — {4}{G} 4/5 Human Soldier.
pub fn meng_huos_horde() -> CardDefinition {
    creature(
        "Meng Huo's Horde",
        cost(&[generic(4), g()]),
        vec![CreatureType::Human, CreatureType::Soldier],
        4,
        5,
    )
}

/// Hunting Cheetah — {2}{G} 2/3 Cat. Whenever it deals damage to an
/// opponent, you may search your library for a Forest card to your hand.
pub fn hunting_cheetah() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer }),
            effect: Effect::MayDo {
                description: "Search your library for a Forest card?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasLandType(LandType::Forest),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..creature("Hunting Cheetah", cost(&[generic(2), g()]), vec![CreatureType::Cat], 2, 3)
    }
}

/// Slashing Tiger — {2}{G}{G} 3/3 Cat; becoming blocked, +2/+2 until end of
/// turn.
pub fn slashing_tiger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Slashing Tiger", cost(&[generic(2), g(), g()]), vec![CreatureType::Cat], 3, 3)
    }
}

/// Taoist Hermit — {2}{G} 2/2 hexproof Human Mystic.
pub fn taoist_hermit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        ..creature("Taoist Hermit", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Mystic], 2, 2)
    }
}

/// Heavy Fog — {1}{G} instant; cast only during the declare attackers step
/// after you've been attacked. Prevent all damage attacking creatures would
/// deal you this turn.
pub fn heavy_fog() -> CardDefinition {
    CardDefinition {
        cast_condition: Some(Predicate::All(vec![
            Predicate::CurrentStepIs(TurnStep::DeclareAttackers),
            Predicate::ValueAtLeast(Value::CreaturesAttackingPlayer(PlayerRef::You), Value::ONE),
        ])),
        ..spell(
            "Heavy Fog",
            cost(&[generic(1), g()]),
            CardType::Instant,
            Effect::PreventAllDamageBetweenThisTurn {
                from: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                to: Selector::You,
            },
        )
    }
}

/// False Mourning — {G} sorcery. Put target card from your graveyard on top
/// of your library.
pub fn false_mourning() -> CardDefinition {
    spell(
        "False Mourning",
        cost(&[g()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::Any.from_your_graveyard()),
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        },
    )
}

/// Riding the Dilu Horse — {2}{G} sorcery. Target creature gets +2/+2 and
/// horsemanship, indefinitely.
pub fn riding_the_dilu_horse() -> CardDefinition {
    spell(
        "Riding the Dilu Horse",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::Permanent,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Horsemanship, duration: Duration::Permanent },
        ]),
    )
}

/// Wielding the Green Dragon — {1}{G} sorcery. Target creature gets +4/+4
/// until end of turn.
pub fn wielding_the_green_dragon() -> CardDefinition {
    spell(
        "Wielding the Green Dragon",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(4),
            toughness: Value::Const(4),
            duration: Duration::EndOfTurn,
        },
    )
}
