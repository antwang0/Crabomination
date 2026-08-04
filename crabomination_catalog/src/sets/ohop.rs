//! Planechase (OHOP / PC2) — plane and phenomenon cards for the CR 901
//! variant. They live in the planar deck and function from the command zone;
//! the planar die's chaos face fires their `ChaosEnsues` triggers and its
//! Planeswalker face turns the next one face up. Tests in `classic_sets/ohop`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, r};

/// A plane: statics and triggers that function from the command zone.
fn plane(
    name: &'static str,
    statics: Vec<StaticAbility>,
    triggered_abilities: Vec<TriggeredAbility>,
) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Plane],
        static_abilities: statics,
        triggered_abilities,
        ..Default::default()
    }
}

/// A phenomenon: one "when you encounter this" trigger, after which its
/// controller planeswalks away (CR 704.6f).
fn phenomenon(name: &'static str, on_encounter: Effect) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Phenomenon],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Encountered, EventScope::SelfSource),
            effect: on_encounter,
        }],
        ..Default::default()
    }
}

/// "Whenever chaos ensues, …"
fn chaos(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::ChaosEnsues, EventScope::SelfSource),
        effect,
    }
}

fn token(
    name: &'static str,
    p: i32,
    t: i32,
    colors: Vec<Color>,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

/// Academy at Tolaria West — an empty hand refills to seven; chaos empties it.
pub fn academy_at_tolaria_west() -> CardDefinition {
    plane(
        "Academy at Tolaria West",
        vec![],
        vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::ValueAtMost(
                    Value::HandSizeOf(PlayerRef::You),
                    Value::ZERO,
                )),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(7) },
            },
            chaos(Effect::Discard {
                who: Selector::You,
                amount: Value::HandSizeOf(PlayerRef::You),
                random: false,
            }),
        ],
    )
}

/// Krosa — everything is two sizes bigger; chaos pays out all five colours.
pub fn krosa() -> CardDefinition {
    plane(
        "Krosa",
        vec![StaticAbility {
            description: "All creatures get +2/+2.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power: 2,
                toughness: 2,
                keywords: vec![],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        vec![chaos(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![
                Color::White,
                Color::Blue,
                Color::Black,
                Color::Red,
                Color::Green,
            ]),
        })],
    )
}

/// Lethe Lake — ten off your library each upkeep, ten off anyone's on chaos.
pub fn lethe_lake() -> CardDefinition {
    plane(
        "Lethe Lake",
        vec![],
        vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::Mill { who: Selector::You, amount: Value::Const(10) },
            },
            chaos(Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(10),
            }),
        ],
    )
}

/// Panopticon — a card for arriving, one every draw step, one on chaos.
pub fn panopticon() -> CardDefinition {
    plane(
        "Panopticon",
        vec![],
        vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Encountered, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Draw),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            chaos(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        ],
    )
}

/// Sanctum of Serra — leaving it wipes the board; chaos resets you to 20.
pub fn sanctum_of_serra() -> CardDefinition {
    plane(
        "Sanctum of Serra",
        vec![],
        vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlaneswalkedAwayFrom, EventScope::SelfSource),
                effect: Effect::Destroy {
                    what: Selector::EachPermanent(R::Land.negate()),
                },
            },
            chaos(Effect::MayDo {
                description: "Have your life total become 20".into(),
                body: Box::new(Effect::SetLifeTotal {
                    who: Selector::You,
                    amount: Value::Const(20),
                }),
            }),
        ],
    )
}

/// The Eon Fog — nothing untaps; chaos untaps your board anyway.
pub fn the_eon_fog() -> CardDefinition {
    plane(
        "The Eon Fog",
        vec![StaticAbility {
            description: "Players skip their untap steps.",
            effect: StaticEffect::PermanentsDontUntap,
        }],
        vec![chaos(Effect::Untap {
            what: Selector::EachPermanent(R::ControlledByYou),
            up_to: None,
        })],
    )
}

/// The Fourth Sphere — a nonblack creature a turn; chaos pays you a Zombie.
pub fn the_fourth_sphere() -> CardDefinition {
    plane(
        "The Fourth Sphere",
        vec![],
        vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature.and(R::HasColor(Color::Black).negate()),
                },
            },
            chaos(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: token(
                    "Zombie",
                    2,
                    2,
                    vec![Color::Black],
                    vec![CreatureType::Zombie],
                    vec![],
                ),
            }),
        ],
    )
}

/// The Hippodrome — everything shrinks by five power; chaos finishes the job.
pub fn the_hippodrome() -> CardDefinition {
    plane(
        "The Hippodrome",
        vec![StaticAbility {
            description: "All creatures get -5/-0.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power: -5,
                toughness: 0,
                keywords: vec![],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        vec![chaos(Effect::MayDo {
            description: "Destroy target creature with power 0 or less".into(),
            body: Box::new(Effect::Destroy {
                what: crate::effect::shortcut::target_filtered(R::Creature.and(R::PowerAtMost(0))),
            }),
        })],
    )
}

/// Goldmeadow — every land brings three Goats; chaos brings one more.
pub fn goldmeadow() -> CardDefinition {
    plane(
        "Goldmeadow",
        vec![],
        vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Land,
                    }),
                effect: Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    count: Value::Const(3),
                    definition: token(
                        "Goat",
                        0,
                        1,
                        vec![Color::White],
                        vec![CreatureType::Goat],
                        vec![],
                    ),
                },
            },
            chaos(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: token(
                    "Goat",
                    0,
                    1,
                    vec![Color::White],
                    vec![CreatureType::Goat],
                    vec![],
                ),
            }),
        ],
    )
}

/// Shiv — every creature can firebreathe; chaos mints a Dragon.
pub fn shiv() -> CardDefinition {
    plane(
        "Shiv",
        vec![StaticAbility {
            description: "All creatures have \"{R}: This creature gets +1/+0 until end of turn.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Creature),
                ability: ActivatedAbility {
                    mana_cost: cost(&[r()]),
                    effect: Effect::PumpPT {
                        what: Selector::This,
                        power: Value::ONE,
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        vec![chaos(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: token(
                "Dragon",
                5,
                5,
                vec![Color::Red],
                vec![CreatureType::Dragon],
                vec![Keyword::Flying],
            ),
        })],
    )
}

/// Fields of Summer — two life per spell; chaos offers ten.
pub fn fields_of_summer() -> CardDefinition {
    plane(
        "Fields of Summer",
        vec![],
        vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
                effect: Effect::MayDoBy {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    description: "Gain 2 life".into(),
                    body: Box::new(Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(2),
                    }),
                },
            },
            chaos(Effect::MayDo {
                description: "Gain 10 life".into(),
                body: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(10),
                }),
            }),
        ],
    )
}

/// Naar Isle — the flames build every upkeep; chaos throws three at a player.
pub fn naar_isle() -> CardDefinition {
    plane(
        "Naar Isle",
        vec![],
        vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: crate::card::CounterType::Fire,
                        amount: Value::ONE,
                    },
                    Effect::DealDamage {
                        to: Selector::You,
                        amount: Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: crate::card::CounterType::Fire,
                        },
                    },
                ]),
            },
            chaos(Effect::DealDamage {
                to: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(3),
            }),
        ],
    )
}

/// Undercity Reaches — a card for every connecting hit.
pub fn undercity_reaches() -> CardDefinition {
    plane(
        "Undercity Reaches",
        vec![],
        vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer),
                effect: Effect::MayDoBy {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    description: "Draw a card".into(),
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            },
            chaos(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
        ],
    )
}

/// Planewide Disaster — the board is swept as you arrive.
pub fn planewide_disaster() -> CardDefinition {
    phenomenon(
        "Planewide Disaster",
        Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
    )
}

/// Mutual Epiphany — everyone draws four on the way through.
pub fn mutual_epiphany() -> CardDefinition {
    phenomenon(
        "Mutual Epiphany",
        Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(4) },
    )
}
