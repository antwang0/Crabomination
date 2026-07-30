//! Modern-archetype gaps batch: Eggs/Stations, Melira combo, Iona, Reshape.
//! Tests in `tests/recent106.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement, Selector, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{bolster, target_filtered};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, StaticEffect, ZoneDest};
use crate::mana::{Color, cost, g, generic, hybrid, u, w, x};

/// Grinding Station — {2} Artifact. {T}, sac an artifact: target player
/// mills three. Artifact ETB: may untap this.
pub fn grinding_station() -> CardDefinition {
    CardDefinition {
        name: "Grinding Station",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((SelectionRequirement::Artifact, 1)),
            effect: Effect::Mill {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                },
            ),
            effect: Effect::MayDo {
                description: "Untap Grinding Station?".into(),
                body: Box::new(Effect::Untap {
                    what: Selector::This,
                    up_to: None,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Anafenza, Kin-Tree Spirit — {W}{W} 2/2. Another nontoken creature you
/// control enters: bolster 1.
pub fn anafenza_kin_tree_spirit() -> CardDefinition {
    CardDefinition {
        name: "Anafenza, Kin-Tree Spirit",
        cost: cost(&[w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::NotToken),
                }),
            effect: bolster(1),
        }],
        ..Default::default()
    }
}

/// Slitherhead — {B/G} 1/1. Scavenge {0}.
pub fn slitherhead() -> CardDefinition {
    CardDefinition {
        name: "Slitherhead",
        cost: cost(&[hybrid(Color::Black, Color::Green)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Zombie],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        // Scavenge {0} (CR 702.96): exile from graveyard, sorcery speed,
        // its power in +1/+1 counters on a target creature.
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Iona, Shield of Emeria — {6}{W}{W}{W} 7/7 Angel. Flying; enters choosing
/// a color; opponents can't cast spells of that color.
pub fn iona_shield_of_emeria() -> CardDefinition {
    CardDefinition {
        name: "Iona, Shield of Emeria",
        cost: cost(&[generic(6), w(), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseColorForSelf,
        }],
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't cast spells of the chosen color.",
            effect: StaticEffect::OpponentsCantCastChosenColor,
        }],
        ..Default::default()
    }
}

/// Thopter Assembly — {6} 5/5 flier. Your upkeep with no other Thopters:
/// bounce it for five 1/1 Thopters.
pub fn thopter_assembly() -> CardDefinition {
    CardDefinition {
        name: "Thopter Assembly",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            )
            .with_filter(Predicate::Not(Box::new(Predicate::SelectorExists(
                Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Thopter)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
            )))),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(5),
                    definition: thopter_token(),
                },
            ]),
        }],
        ..Default::default()
    }
}

fn thopter_token() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Reshape — {X}{U}{U} Sorcery. Sac an artifact; fetch an artifact with
/// mana value X or less onto the battlefield.
pub fn reshape() -> CardDefinition {
    CardDefinition {
        name: "Reshape",
        cost: cost(&[x(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Additional cost folded into resolution (Thud pattern).
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: SelectionRequirement::Artifact,
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Artifact
                    .and(SelectionRequirement::ManaValueAtMostXFromCost),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        ]),
        ..Default::default()
    }
}

/// Wild Cantor — {R/G} 1/1. Sacrifice: add one mana of any color.
pub fn wild_cantor() -> CardDefinition {
    CardDefinition {
        name: "Wild Cantor",
        cost: cost(&[hybrid(Color::Red, Color::Green)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Melira, Sylvok Outcast — {1}{G} 2/2. No poison for you; no -1/-1 counters
/// on your creatures; opponents' creatures lose infect.
pub fn melira_sylvok_outcast() -> CardDefinition {
    CardDefinition {
        name: "Melira, Sylvok Outcast",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![
            StaticAbility {
                description: "You can't get poison counters.",
                effect: StaticEffect::PlayerCannotGetPoison,
            },
            StaticAbility {
                description: "Creatures you control can't have -1/-1 counters put on them.",
                effect: StaticEffect::NoMinusCountersOnYourCreatures,
            },
            StaticAbility {
                description: "Creatures your opponents control lose infect.",
                effect: StaticEffect::LoseKeyword {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    keyword: Keyword::Infect,
                },
            },
        ],
        ..Default::default()
    }
}
