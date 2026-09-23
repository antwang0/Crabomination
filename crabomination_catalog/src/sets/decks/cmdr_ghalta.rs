//! Commander: the cards the **Tramplesaurus Rex** precon (FDC, Ghalta, Primal
//! Hunger) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_nghathrod.rs`, after the Mind Flayarrrs batch.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::catalog::sets::tap_add;
use crate::effect::shortcut::etb;
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, StaticAbility, StaticEffect,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, g, generic};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// "You control a creature with power 4 or greater" — ferocious.
fn ferocious() -> Predicate {
    Predicate::SelectorExists(Selector::EachPermanent(
        R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
    ))
}

/// Arachnogenesis — a reach Spider per creature attacking you (CR 506.3: you,
/// not another player), and only Spiders deal combat damage this turn.
pub fn arachnogenesis() -> CardDefinition {
    let spider = Arc::new(TokenDefinition {
        name: "Spider".into(),
        power: 1,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        keywords: vec![Keyword::Reach],
        ..Default::default()
    });
    CardDefinition {
        name: "Arachnogenesis",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Creature.and(R::IsAttackingYou),
                ))),
                definition: spider,
            },
            Effect::PreventCombatDamageExceptDealtBy {
                except: R::HasCreatureType(CreatureType::Spider),
            },
        ]),
        ..Default::default()
    }
}

/// Colossal Majesty — draws each upkeep while you control a power-4 creature
/// (intervening "if", CR 603.4).
pub fn colossal_majesty() -> CardDefinition {
    CardDefinition {
        name: "Colossal Majesty",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(ferocious()),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Curious Altisaur — a card whenever a Dinosaur of yours connects.
pub fn curious_altisaur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dinosaur),
                }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Curious Altisaur", cost(&[generic(3), g()]), vec![CreatureType::Dinosaur], 2, 5)
    }
}

/// Dungrove Elder — hexproof, as big as your Forest count.
pub fn dungrove_elder() -> CardDefinition {
    let forests = || {
        Value::CountOf(Box::new(Selector::EachPermanent(
            R::HasLandType(LandType::Forest).and(R::ControlledByYou),
        )))
    };
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        static_abilities: vec![StaticAbility {
            description: "Dungrove Elder's power and toughness are each equal to the number of Forests you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: forests(), toughness: forests() },
        }],
        ..creature("Dungrove Elder", cost(&[generic(2), g()]), vec![CreatureType::Treefolk], 0, 0)
    }
}

/// Ezuri's Predation — a 4/4 Beast for each opposing creature, each fighting
/// its own (CR 701.14).
pub fn ezuris_predation() -> CardDefinition {
    let beast = Arc::new(TokenDefinition {
        name: "Phyrexian Beast".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Beast],
            ..Default::default()
        },
        ..Default::default()
    });
    CardDefinition {
        name: "Ezuri's Predation",
        cost: cost(&[generic(5), g(), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateTokensToFightEach {
            filter: R::Creature.and(R::ControlledByOpponent),
            definition: beast,
        },
        ..Default::default()
    }
}

/// Monstrous Onslaught — your biggest creature's power, divided among any
/// number of target creatures. ⚠ X is read at resolution, not "as you cast
/// this spell": a pump or removal in response changes it.
pub fn monstrous_onslaught() -> CardDefinition {
    CardDefinition {
        name: "Monstrous Onslaught",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamageDivided {
            total: Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
            filter: R::Creature,
            max_targets: 8,
            retaliate_to_source: false,
        },
        ..Default::default()
    }
}

/// Surrak and Goreclaw — trample for the team; each other nontoken creature
/// enters with a +1/+1 counter and haste.
pub fn surrak_and_goreclaw() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                keyword: Keyword::Trample,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..creature(
            "Surrak and Goreclaw",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Human, CreatureType::Bear],
            6,
            5,
        )
    }
}

/// Tangleweave Armor — living weapon; the equipped creature grows by your
/// biggest commander's mana value (CR 903.3).
pub fn tangleweave_armor() -> CardDefinition {
    let germ = Arc::new(TokenDefinition {
        name: "Phyrexian Germ".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian],
            ..Default::default()
        },
        ..Default::default()
    });
    let x = || Value::GreatestCommanderManaValue(PlayerRef::You);
    CardDefinition {
        name: "Tangleweave Armor",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        static_abilities: vec![StaticAbility {
            description: "Equipped creature gets +X/+X, where X is the greatest mana value among your commanders.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: x(),
                toughness: x(),
            },
        }],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: germ },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        ..Default::default()
    }
}

/// Thickest in the Thicket — doubles a creature's power in counters, then
/// draws two each end step you hold the (tied) biggest creature.
pub fn thickest_in_the_thicket() -> CardDefinition {
    CardDefinition {
        name: "Thickest in the Thicket",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: crate::effect::shortcut::target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::If {
                    cond: Predicate::ControlsGreatestPowerCreature { who: PlayerRef::You },
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Whiptongue Hydra — clears the skies and grows by what it killed.
pub fn whiptongue_hydra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::PermanentsDestroyedThisResolution,
            },
        ]))],
        ..creature(
            "Whiptongue Hydra",
            cost(&[generic(5), g()]),
            vec![CreatureType::Lizard, CreatureType::Hydra],
            4,
            4,
        )
    }
}

/// Whisperer of the Wilds — {G}, or {G}{G} while ferocious.
pub fn whisperer_of_the_wilds() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_add(Color::Green),
            ActivatedAbility {
                tap_cost: true,
                condition: Some(ferocious()),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Green, Color::Green]),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Whisperer of the Wilds",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            0,
            2,
        )
    }
}

/// Yeva, Nature's Herald — flash, and flash for your green creatures.
pub fn yeva_natures_herald() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "You may cast green creature spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash {
                filter: R::Creature.and(R::HasColor(Color::Green)),
            },
        }],
        ..creature(
            "Yeva, Nature's Herald",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Shaman],
            4,
            4,
        )
    }
}
