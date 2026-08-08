//! Utility / nonbasic lands staple cluster. Several exercise existing
//! primitives (pain-style mana+self-damage, conditional ETB-tapped, manland
//! animation, charge/`+1/+1` counters, `DistinctNamesControlledMatching`) plus
//! the new `StaticEffect::PreventAllDamageToController` (Glacial Chasm, CR 615).
//! Tests in `tests/recent40.rs`.

use crate::card::CumulativeUpkeepCost;
use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{ActivatedAbility, Duration, ManaPayload, PlayerRef, Predicate};
use crate::mana::{Color, b, cost, generic, snow_mana, w};

/// `{T}: Add {color}.`
fn tap_for(color: Color) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![color]),
        },
        ..Default::default()
    }
}

/// `{T}: Add {C}.`
fn tap_for_colorless(n: u32) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colorless(Value::Const(n as i32)),
        },
        ..Default::default()
    }
}

/// SelfSource ETB trigger: enter tapped unless you control a land of `gate`.
fn enters_tapped_unless_land(gate: LandType) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(
                R::HasLandType(gate).and(R::ControlledByYou),
            )),
            then: Box::new(Effect::Noop),
            else_: Box::new(Effect::Tap {
                what: Selector::This,
            }),
        },
    }
}

/// Ancient Tomb — `{T}: Add {C}{C}. This land deals 2 damage to you.`
pub fn ancient_tomb() -> CardDefinition {
    CardDefinition {
        name: "Ancient Tomb",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(2)),
                },
                Effect::DealDamage {
                    to: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// City of Traitors — `{T}: Add {C}{C}.` Sacrifices itself when you play
/// another land.
pub fn city_of_traitors() -> CardDefinition {
    CardDefinition {
        name: "City of Traitors",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_for_colorless(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Land),
                }),
            effect: Effect::Sacrifice {
                who: Selector::This,
                count: Value::Const(1),
                filter: R::HasCardType(CardType::Land),
            },
        }],
        ..Default::default()
    }
}

/// Tarnished Citadel — `{T}: Add {C}.` and `{T}: Add one mana of any color.
/// This land deals 3 damage to you.`
pub fn tarnished_citadel() -> CardDefinition {
    CardDefinition {
        name: "Tarnished Citadel",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_for_colorless(1),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyOneColor(Value::Const(1)),
                    },
                    Effect::DealDamage {
                        to: Selector::You,
                        amount: Value::Const(3),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cascading Cataracts — Indestructible. `{T}: Add {C}.` and
/// `{5}, {T}: Add five mana in any combination of colors.`
pub fn cascading_cataracts() -> CardDefinition {
    CardDefinition {
        name: "Cascading Cataracts",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::Indestructible],
        activated_abilities: vec![
            tap_for_colorless(1),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(5)]),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyColors(Value::Const(5)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Castle Locthwain — enters tapped unless you control a Swamp. `{T}: Add {B}.`
/// `{1}{B}{B}, {T}: Draw a card, then lose life equal to cards in your hand.`
pub fn castle_locthwain() -> CardDefinition {
    CardDefinition {
        name: "Castle Locthwain",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![enters_tapped_unless_land(LandType::Swamp)],
        activated_abilities: vec![
            tap_for(Color::Black),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1), b(), b()]),
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::HandSizeOf(PlayerRef::You),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Castle Ardenvale — enters tapped unless you control a Plains. `{T}: Add {W}.`
/// `{2}{W}{W}, {T}: Create a 1/1 white Human creature token.`
pub fn castle_ardenvale() -> CardDefinition {
    let human = TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Castle Ardenvale",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![enters_tapped_unless_land(LandType::Plains)],
        activated_abilities: vec![
            tap_for(Color::White),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), w(), w()]),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: Box::new(human),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Faceless Haven — `{T}: Add {C}.` `{S}{S}{S}: This land becomes a 4/3
/// creature with vigilance and all creature types until end of turn (still a
/// land).`
pub fn faceless_haven() -> CardDefinition {
    CardDefinition {
        name: "Faceless Haven",
        supertypes: vec![crate::card::Supertype::Snow],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_for_colorless(1),
            ActivatedAbility {
                mana_cost: cost(&[snow_mana(), snow_mana(), snow_mana()]),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(3),
                    creature_types: vec![],
                    keywords: vec![Keyword::Vigilance, Keyword::Changeling],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Crawling Barrens — `{T}: Add {C}.` `{4}: Put two +1/+1 counters on this
/// land, then it becomes a 0/0 Elemental until end of turn (still a land).`
pub fn crawling_barrens() -> CardDefinition {
    CardDefinition {
        name: "Crawling Barrens",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_for_colorless(1),
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(0),
                        toughness: Value::Const(0),
                        creature_types: vec![CreatureType::Elemental],
                        keywords: vec![],
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Field of the Dead — enters tapped. `{T}: Add {C}.` Whenever this or another
/// land you control enters, if you control seven or more lands with different
/// names, create a 2/2 black Zombie.
pub fn field_of_the_dead() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Field of the Dead",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_for_colorless(1)],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Tap {
                    what: Selector::This,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::All(vec![
                        Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::HasCardType(CardType::Land),
                        },
                        Predicate::ValueAtLeast(
                            Value::DistinctNamesControlledMatching(crate::card::SelectionRequirement::Land),
                            Value::Const(7),
                        ),
                    ])),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: Box::new(zombie),
                },
            },
        ],
        ..Default::default()
    }
}

/// Glacial Chasm — Cumulative upkeep—Pay 2 life. When it enters, sacrifice a
/// land. Creatures you control can't attack. Prevent all damage that would be
/// dealt to you (the new `StaticEffect::PreventAllDamageToController`).
pub fn glacial_chasm() -> CardDefinition {
    CardDefinition {
        name: "Glacial Chasm",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Life(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: R::HasCardType(CardType::Land),
            },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control can't attack.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::CantAttack,
                },
            },
            StaticAbility {
                description: "Prevent all damage that would be dealt to you.",
                effect: StaticEffect::PreventAllDamageToController,
            },
        ],
        ..Default::default()
    }
}
