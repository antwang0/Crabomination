//! Commander: the cards the **Calling All Angels** precon (FDC, Giada, Font
//! of Hope) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_angels.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{enrage, etb, etb_gain_life, exalted, investigate, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, generic, w};
use std::sync::Arc;

fn angel(name: &'static str, mana: crate::mana::ManaCost, extra: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    let mut types = vec![CreatureType::Angel];
    types.extend(extra);
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

fn flier_token(name: &str, creature_type: CreatureType, p: i32, t: i32) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![creature_type], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    })
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// Angel of the Ruins — exile up to two artifacts and/or enchantments on
/// entry; plainscycling {2}.
pub fn angel_of_the_ruins() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![
            Keyword::Flying,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Plains),
        ],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Artifact.or(R::Enchantment),
            effect: Box::new(Effect::Exile { what: Selector::Target(0) }),
        })],
        ..angel("Angel of the Ruins", cost(&[generic(5), w(), w()]), vec![], 5, 7)
    }
}

/// Angelic Field Marshal — lieutenant: +2/+2 and your creatures have
/// vigilance while you control your commander (CR 207.2c).
pub fn angelic_field_marshal() -> CardDefinition {
    let lieutenant = || Predicate::ControlsOwnCommander { who: PlayerRef::You };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Lieutenant — this creature gets +2/+2.",
                effect: StaticEffect::PumpSelfIf {
                    condition: lieutenant(),
                    power: 2,
                    toughness: 2,
                    keywords: vec![],
                },
            },
            StaticAbility {
                description: "Lieutenant — creatures you control have vigilance.",
                effect: StaticEffect::WhileCondition {
                    condition: lieutenant(),
                    inner: Box::new(StaticEffect::GrantKeyword {
                        applies_to: yours(R::Creature),
                        keyword: Keyword::Vigilance,
                    }),
                },
            },
        ],
        ..angel("Angelic Field Marshal", cost(&[generic(2), w(), w()]), vec![], 3, 3)
    }
}

/// Angelic Sleuth — investigate when another permanent of yours leaves with
/// counters on it (read from its last known information).
pub fn angelic_sleuth() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            // CR 603.10a — a leaves-the-battlefield trigger looks back: the
            // counters are the ones it had as it left.
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::ValueAtLeast(
                    Value::TotalCountersOn { what: Box::new(Selector::TriggerSource) },
                    Value::ONE,
                )),
            effect: investigate(1),
        }],
        ..angel("Angelic Sleuth", cost(&[generic(2), w()]), vec![CreatureType::Advisor], 2, 3)
    }
}

/// Court of Grace — you become the monarch; each upkeep a 1/1 Spirit, or a
/// 4/4 Angel while you're the monarch.
pub fn court_of_grace() -> CardDefinition {
    let make = |def: Arc<TokenDefinition>| Box::new(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: def,
    });
    CardDefinition {
        name: "Court of Grace",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::If {
                    cond: Predicate::IsMonarch { who: PlayerRef::You },
                    then: make(flier_token("Angel", CreatureType::Angel, 4, 4)),
                    else_: make(flier_token("Spirit", CreatureType::Spirit, 1, 1)),
                },
            },
        ],
        ..Default::default()
    }
}

/// Defy Death — reanimate a creature card; an Angel gets two +1/+1 counters.
pub fn defy_death() -> CardDefinition {
    CardDefinition {
        name: "Defy Death",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::LastMoved,
                    filter: R::HasCreatureType(CreatureType::Angel),
                },
                then: Box::new(Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Destroy Evil — a creature with toughness 4 or greater, or an enchantment.
pub fn destroy_evil() -> CardDefinition {
    CardDefinition {
        name: "Destroy Evil",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::ToughnessAtLeast(4))) },
            Effect::Destroy { what: target_filtered(R::Enchantment) },
        ]),
        ..Default::default()
    }
}

/// Invoke the Divine — destroy an artifact or enchantment, gain 4.
pub fn invoke_the_divine() -> CardDefinition {
    CardDefinition {
        name: "Invoke the Divine",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        ]),
        ..Default::default()
    }
}

/// Endless Atlas — {2}, {T}: draw, with three lands of one name.
pub fn endless_atlas() -> CardDefinition {
    CardDefinition {
        name: "Endless Atlas",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            condition: Some(Predicate::ControlsLandsWithSameNameAtLeast {
                who: PlayerRef::You,
                at_least: 3,
            }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Firemane Commando — attacking with two or more draws you a card; another
/// player who does so draws one if none of them attacked you.
pub fn firemane_commando() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(
                    Predicate::AttackedWithCountAtLeast { who: PlayerRef::You, at_least: 2 },
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                // An observer of another player's declaration (AnyPlayer is
                // the scope the attack dispatch consults for non-active
                // listeners).
                event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
                    Predicate::All(vec![
                        Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
                        Predicate::AttackedWithCountAtLeast { who: PlayerRef::ActivePlayer, at_least: 2 },
                    ]),
                ),
                effect: Effect::If {
                    cond: Predicate::AttackedDefenderWithCountAtLeast {
                        who: PlayerRef::ActivePlayer,
                        defender: PlayerRef::You,
                        at_least: 1,
                        include_planeswalkers: false,
                    },
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::Draw {
                        who: Selector::Player(PlayerRef::ActivePlayer),
                        amount: Value::ONE,
                    }),
                },
            },
        ],
        ..angel("Firemane Commando", cost(&[generic(3), w()]), vec![CreatureType::Soldier], 4, 3)
    }
}

/// Herald of War — grows on each attack; Angel and Human spells cost {1}
/// less per +1/+1 counter on it.
pub fn herald_of_war() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Angel spells and Human spells you cast cost {1} less for each +1/+1 counter on this creature.",
            effect: StaticEffect::CostReductionPerCounterOnSource {
                filter: R::HasCreatureType(CreatureType::Angel).or(R::HasCreatureType(CreatureType::Human)),
                kind: CounterType::PlusOnePlusOne,
            },
        }],
        ..angel("Herald of War", cost(&[generic(3), w(), w()]), vec![], 3, 3)
    }
}

/// Merchant of Truth — a nontoken creature of yours dying investigates; your
/// Clues have exalted (CR 702.83).
pub fn merchant_of_truth() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
            ),
            effect: investigate(1),
        }],
        static_abilities: vec![StaticAbility {
            description: "Clues you control have exalted.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::HasArtifactSubtype(ArtifactSubtype::Clue).and(R::ControlledByYou),
                ability: Box::new(exalted()),
            },
        }],
        ..angel("Merchant of Truth", cost(&[generic(2), w(), w()]), vec![CreatureType::Detective], 2, 5)
    }
}

/// Metropolis Reformer — you have hexproof; damage dealt to it is life
/// gained.
pub fn metropolis_reformer() -> CardDefinition {
    let mut card = angel("Metropolis Reformer", cost(&[generic(2), w()]), vec![CreatureType::Cleric], 2, 3);
    card.keywords.push(Keyword::Vigilance);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You have hexproof.",
            effect: StaticEffect::ControllerHasHexproof,
        }],
        triggered_abilities: vec![enrage(Effect::GainLife {
            who: Selector::You,
            amount: Value::TriggerEventAmount,
        })],
        ..card
    }
}

/// Sephara, Sky's Blade — or {W} and tap four untapped fliers; lifelink;
/// your other fliers are indestructible.
pub fn sephara_skys_blade() -> CardDefinition {
    let mut card = angel("Sephara, Sky's Blade", cost(&[generic(4), w(), w(), w()]), vec![], 7, 7);
    card.keywords.push(Keyword::Lifelink);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[w()]),
            tap_creatures: Some((R::Creature.and(R::HasKeyword(Keyword::Flying)), 4)),
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control with flying have indestructible.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(
                    R::Creature.and(R::HasKeyword(Keyword::Flying)).and(R::OtherThanSource),
                ),
                keyword: Keyword::Indestructible,
            },
        }],
        ..card
    }
}

/// Seraph Sanctuary — {C}; 1 life on entry and per Angel of yours entering.
pub fn seraph_sanctuary() -> CardDefinition {
    CardDefinition {
        name: "Seraph Sanctuary",
        card_types: vec![CardType::Land],
        activated_abilities: vec![super::super::tap_add_colorless()],
        triggered_abilities: vec![
            etb_gain_life(1),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Angel),
                    }),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Seraph of the Sword — combat damage to it is prevented (CR 615).
pub fn seraph_of_the_sword() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to this creature.",
            effect: StaticEffect::PreventAllCombatDamageToThis,
        }],
        ..angel("Seraph of the Sword", cost(&[generic(3), w()]), vec![], 3, 3)
    }
}

/// Starnheim Aspirant — Angel spells cost {2} less.
pub fn starnheim_aspirant() -> CardDefinition {
    CardDefinition {
        name: "Starnheim Aspirant",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Angel spells you cast cost {2} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::HasCreatureType(CreatureType::Angel), amount: 2 },
        }],
        ..Default::default()
    }
}

/// Tome of Legends — a page counter on entry and whenever your commander
/// enters or attacks; {1}, {T}, remove one: draw.
pub fn tome_of_legends() -> CardDefinition {
    let your_commander = || Predicate::EntityMatches {
        what: Selector::TriggerSource,
        filter: R::IsCommander.and(R::OwnedByYou),
    };
    let page = || Effect::AddCounter { what: Selector::This, kind: CounterType::Page, amount: Value::ONE };
    CardDefinition {
        name: "Tome of Legends",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Book], ..Default::default() },
        enters_with_counters: Some((CounterType::Page, Value::ONE)),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(your_commander()),
                effect: page(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(your_commander()),
                effect: page(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            remove_counter_cost: Some((CounterType::Page, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}
