//! MKM (Murders at Karlov Manor) gap batch — Detectives, graveyard-activated
//! payoffs, Disguise, and Clue-flavored value. Tests in
//! `tests/recent_b/recent243.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate,
    Selector, Value,
};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, Color};

/// The Chase Is On — {2}{R} Instant. Target creature gets +3/+0 and gains first
/// strike until end of turn. Investigate.
pub fn the_chase_is_on() -> CardDefinition {
    CardDefinition {
        name: "The Chase Is On",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            crate::effect::shortcut::investigate(1),
        ]),
        ..Default::default()
    }
}

/// Galvanize — {1}{R} Instant. Deals 3 damage to target creature, or 5 if you've
/// drawn two or more cards this turn.
pub fn galvanize() -> CardDefinition {
    CardDefinition {
        name: "Galvanize",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::CardsDrawnThisTurn(PlayerRef::You),
                Value::Const(2),
            ),
            then: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(5),
            }),
            else_: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(3),
            }),
        },
        ..Default::default()
    }
}

/// Red Herring — {1}{R} Artifact Creature — Clue Fish 2/2. Haste; attacks each
/// combat if able. {2}, Sacrifice this creature: Draw a card.
pub fn red_herring() -> CardDefinition {
    CardDefinition {
        name: "Red Herring",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish],
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Clue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste, Keyword::MustAttack],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vengeful Creeper — {4}{G} Creature — Plant Elemental 5/5. Disguise {5}{G}.
/// When turned face up, destroy target artifact or enchantment an opponent
/// controls.
pub fn vengeful_creeper() -> CardDefinition {
    CardDefinition {
        name: "Vengeful Creeper",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Elemental],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Disguise(cost(&[generic(5), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(
                    R::Artifact.or(R::Enchantment).and(R::ControlledByOpponent),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Rubblebelt Maverick — {G} Creature — Human Detective 1/1. ETB surveil 2.
/// {G}, Exile this card from your graveyard: put a +1/+1 counter on target
/// creature. Activate only as a sorcery.
pub fn rubblebelt_maverick() -> CardDefinition {
    CardDefinition {
        name: "Rubblebelt Maverick",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            mana_cost: cost(&[g()]),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Leering Onlooker — {1}{B} Creature — Vampire 1/3, flying. {2}{B}{B}, Exile
/// this card from your graveyard: create two tapped 1/1 black Bat tokens with
/// flying.
pub fn leering_onlooker() -> CardDefinition {
    CardDefinition {
        name: "Leering Onlooker",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            exile_self_cost: true,
            mana_cost: cost(&[generic(2), b(), b()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: TokenDefinition {
                    name: "Bat".into(),
                    power: 1,
                    toughness: 1,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Bat],
                        ..Default::default()
                    },
                    tapped: true,
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tunnel Tipster — {1}{G} Creature — Mole Scout 1/1. At your end step, if a
/// face-down creature entered under your control this turn, put a +1/+1 counter
/// on it. {T}: Add {G}.
pub fn tunnel_tipster() -> CardDefinition {
    CardDefinition {
        name: "Tunnel Tipster",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mole, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::FaceDownActivityThisTurn { who: PlayerRef::You }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Green, Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gravestone Strider — {2} Artifact Creature — Golem 1/3. {1}: Add one mana of
/// any color; activate only once each turn. {2}, Exile this card from your
/// graveyard: exile target card from a graveyard.
pub fn gravestone_strider() -> CardDefinition {
    CardDefinition {
        name: "Gravestone Strider",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 1,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                once_per_turn: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                from_graveyard: true,
                exile_self_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::ExileTaggedWithSource { what: target_filtered(R::InGraveyard) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
