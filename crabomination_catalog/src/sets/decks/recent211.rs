//! Foundations (FDN) gap batch 10 — a deathtouch-poison lord (CR 702.72 +
//! poison), a mass "target player" bounce, a Punisher enchantment, a Dragon
//! payoff, and a redirect. Tests in `tests/recent211.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, Keyword,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, EventKind, EventScope, EventSpec, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r};

/// Fynn, the Fangbearer — {1}{G} 1/3 Legendary Human Warrior. Deathtouch;
/// whenever a creature you control with deathtouch deals combat damage to a
/// player, that player gets two poison counters. (CR 702.72 + poison.)
pub fn fynn_the_fangbearer() -> CardDefinition {
    CardDefinition {
        name: "Fynn, the Fangbearer",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasKeyword(Keyword::Deathtouch),
            }),
            effect: Effect::AddPoison {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// River's Rebuke — {4}{U}{U} Sorcery. Return all nonland permanents target
/// player controls to their owner's hand.
pub fn rivers_rebuke() -> CardDefinition {
    CardDefinition {
        name: "River's Rebuke",
        cost: cost(&[generic(4), crate::mana::u(), crate::mana::u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::ControlledBy {
                who: PlayerRef::Target(0),
                filter: R::Nonland,
            },
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}

/// Painful Quandary — {3}{B}{B} Enchantment. Whenever an opponent casts a
/// spell, that player loses 5 life unless they discard a card.
pub fn painful_quandary() -> CardDefinition {
    CardDefinition {
        name: "Painful Quandary",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::Punisher {
                chooser: Selector::Player(PlayerRef::Triggerer),
                options: vec![Effect::Discard {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::ONE,
                    random: false,
                }],
                otherwise: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(5),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Lathliss, Dragon Queen — {4}{R}{R} 6/6 Legendary Dragon. Flying; whenever
/// another nontoken Dragon you control enters, create a 5/5 red Dragon token
/// with flying. {1}{R}: Dragons you control get +1/+0 until end of turn.
pub fn lathliss_dragon_queen() -> CardDefinition {
    let dragon = TokenDefinition {
        name: "Dragon".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Lathliss, Dragon Queen",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dragon).and(R::NotToken),
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: dragon,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::EachMatching {
                    zone: crate::effect::ZoneRef::Battlefield,
                    filter: R::HasCreatureType(CreatureType::Dragon).and(R::ControlledByYou),
                },
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bolt Bend — {3}{R} Instant. This spell costs {3} less to cast if you control
/// a creature with power 4 or greater. Change the target of target spell or
/// ability with a single target.
pub fn bolt_bend() -> CardDefinition {
    CardDefinition {
        name: "Bolt Bend",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Costs {3} less if you control a creature with power 4 or greater.",
            effect: crate::card::StaticEffect::SelfCostReducedIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Battlefield,
                        filter: R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                    },
                    n: Value::ONE,
                },
                amount: 3,
            },
        }],
        effect: Effect::ChooseNewTargetsForSpell {
            what: target_filtered(R::IsSpellOnStack),
        },
        ..Default::default()
    }
}
