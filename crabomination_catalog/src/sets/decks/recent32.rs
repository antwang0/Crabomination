//! Aristocrats / sacrifice-matters supplement: sac-fodder payoffs and drain
//! engines. Sacrifice-as-a-cost activated abilities fold the sacrifice as the
//! effect's first step (the cost-as-first-step convention used across this
//! catalog). Tracked in `DECK_FEATURES.md`; tests in `tests/recent32.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{b, cost, generic, r, w};

/// Sacrifice one creature you control as the first step of an activated cost.
/// `another` excludes the source (CR "another creature").
fn sac_creature(another: bool) -> Effect {
    let mut filter = SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou);
    if another {
        filter = filter.and(SelectionRequirement::OtherThanSource);
    }
    Effect::Sacrifice {
        who: Selector::You,
        count: Value::Const(1),
        filter,
    }
}

fn creature(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
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

/// Cartel Aristocrat — {W}{B} 2/2 Human Advisor. Sacrifice another creature:
/// this gains protection from the color of your choice until end of turn.
pub fn cartel_aristocrat() -> CardDefinition {
    let mut def = creature(
        "Cartel Aristocrat",
        cost(&[w(), b()]),
        vec![CreatureType::Human, CreatureType::Advisor],
        2,
        2,
    );
    def.activated_abilities = vec![ActivatedAbility {
        effect: Effect::Seq(vec![
            sac_creature(true),
            Effect::GrantProtectionFromChosenColor {
                what: Selector::This,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }];
    def
}

/// Bloodflow Connoisseur — {2}{B} 1/1 Vampire. Sacrifice a creature: put a
/// +1/+1 counter on this creature.
pub fn bloodflow_connoisseur() -> CardDefinition {
    let mut def = creature(
        "Bloodflow Connoisseur",
        cost(&[generic(2), b()]),
        vec![CreatureType::Vampire],
        1,
        1,
    );
    def.activated_abilities = vec![ActivatedAbility {
        effect: Effect::Seq(vec![
            sac_creature(false),
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }];
    def
}

/// Vampire Aristocrat — {2}{B} 2/2 Vampire Rogue Noble. Sacrifice a creature:
/// this gets +2/+2 until end of turn.
pub fn vampire_aristocrat() -> CardDefinition {
    let mut def = creature(
        "Vampire Aristocrat",
        cost(&[generic(2), b()]),
        vec![
            CreatureType::Vampire,
            CreatureType::Rogue,
            CreatureType::Noble,
        ],
        2,
        2,
    );
    def.activated_abilities = vec![ActivatedAbility {
        effect: Effect::Seq(vec![
            sac_creature(false),
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }];
    def
}

/// Yahenni, Undying Partisan — {2}{B} 2/2 legendary Aetherborn Vampire. Haste.
/// Whenever a creature an opponent controls dies, put a +1/+1 counter on Yahenni.
/// Sacrifice another creature: Yahenni gains indestructible until end of turn.
pub fn yahenni_undying_partisan() -> CardDefinition {
    let mut def = creature(
        "Yahenni, Undying Partisan",
        cost(&[generic(2), b()]),
        vec![CreatureType::Aetherborn, CreatureType::Vampire],
        2,
        2,
    );
    def.supertypes = vec![Supertype::Legendary];
    def.keywords = vec![Keyword::Haste];
    def.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    }];
    def.activated_abilities = vec![ActivatedAbility {
        effect: Effect::Seq(vec![
            sac_creature(true),
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }];
    def
}

/// Bontu the Glorified — {2}{B} 4/6 legendary God. Menace, indestructible.
/// Can't attack or block unless a creature died under your control this turn.
/// {1}{B}, Sacrifice another creature: Scry 1. Each opponent loses 1 life and
/// you gain 1 life.
pub fn bontu_the_glorified() -> CardDefinition {
    let mut def = creature(
        "Bontu the Glorified",
        cost(&[generic(2), b()]),
        vec![CreatureType::God],
        4,
        6,
    );
    def.supertypes = vec![Supertype::Legendary];
    def.keywords = vec![
        Keyword::Menace,
        Keyword::Indestructible,
        Keyword::CantAttackOrBlockUnlessCreatureDiedThisTurn,
    ];
    def.activated_abilities = vec![ActivatedAbility {
        mana_cost: cost(&[generic(1), b()]),
        effect: Effect::Seq(vec![
            sac_creature(true),
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }];
    def
}

/// Smothering Abomination — {2}{B}{B} 4/3 Eldrazi. Devoid, flying. At the
/// beginning of your upkeep, sacrifice a creature. Whenever you sacrifice a
/// creature, draw a card.
pub fn smothering_abomination() -> CardDefinition {
    let mut def = creature(
        "Smothering Abomination",
        cost(&[generic(2), b(), b()]),
        vec![CreatureType::Eldrazi],
        4,
        3,
    );
    def.keywords = vec![Keyword::Devoid, Keyword::Flying];
    def.triggered_abilities = vec![
        TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: sac_creature(false),
        },
        TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        },
    ];
    def
}

/// Butcher Ghoul — {1}{B} 1/1 Zombie. Undying.
pub fn butcher_ghoul() -> CardDefinition {
    let mut def = creature(
        "Butcher Ghoul",
        cost(&[generic(1), b()]),
        vec![CreatureType::Zombie],
        1,
        1,
    );
    def.keywords = vec![Keyword::Undying];
    def
}

/// Elas il-Kor, Sadistic Pilgrim — {W}{B} 2/2 legendary Phyrexian Kor Cleric.
/// Deathtouch. Whenever another creature you control enters, you gain 1 life.
/// Whenever another creature you control dies, each opponent loses 1 life.
pub fn elas_il_kor_sadistic_pilgrim() -> CardDefinition {
    let mut def = creature(
        "Elas il-Kor, Sadistic Pilgrim",
        cost(&[w(), b()]),
        vec![
            CreatureType::Phyrexian,
            CreatureType::Kor,
            CreatureType::Cleric,
        ],
        2,
        2,
    );
    def.supertypes = vec![Supertype::Legendary];
    def.keywords = vec![Keyword::Deathtouch];
    def.triggered_abilities = vec![
        TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        },
        TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        },
    ];
    def
}

/// Mahadi, Emporium Master — {1}{B}{R} 3/3 legendary Devil. At the beginning of
/// your end step, create a Treasure token for each creature that died this turn.
pub fn mahadi_emporium_master() -> CardDefinition {
    let mut def = creature(
        "Mahadi, Emporium Master",
        cost(&[generic(1), b(), r()]),
        vec![CreatureType::Devil],
        3,
        3,
    );
    def.supertypes = vec![Supertype::Legendary];
    def.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(crate::game::types::TurnStep::End),
            EventScope::ActivePlayer,
        ),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CreaturesDiedThisTurnTotal,
            definition: Box::new(crabomination_base::tokens::treasure_token()),
        },
    }];
    def
}

/// Heartless Summoning — {1}{B} Enchantment. Creature spells you cast cost {2}
/// less. Creatures you control get -1/-1.
pub fn heartless_summoning() -> CardDefinition {
    CardDefinition {
        name: "Heartless Summoning",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Creature spells you cast cost {2} less to cast.",
                effect: StaticEffect::CostReduction {
                    filter: SelectionRequirement::Creature,
                    amount: 2,
                },
            },
            StaticAbility {
                description: "Creatures you control get -1/-1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: -1,
                    toughness: -1,
                },
            },
        ],
        ..Default::default()
    }
}
