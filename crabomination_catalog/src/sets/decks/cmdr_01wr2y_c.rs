//! Commander: closers for Worf, Honor-Bound Warrior's and Rutherford,
//! Upgraded Ensign's EDHREC average decks (scan of 2026-11-27). Tests in
//! `tests/recent_b/cmdr_01wr2y_a.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, Predicate,
    SelectionRequirement as R, Selector, StationBand, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, investigate, station, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Value, ZoneDest};
use crate::mana::{Color, cost, generic, hybrid, r, w};

/// "Whenever you attack while creatures you control have total power 8 or
/// greater" — the Star Trek attack-threshold trigger.
fn attack_with_eight_power(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
            .with_filter(Predicate::ValueAtLeast(Value::TotalPowerControlled, Value::Const(8))),
        effect,
    }
}

/// Worf, Honor-Bound Warrior — {R/W}{R/W} legendary 1/1 Klingon Warrior
/// Officer. Attacking with total power 8+: double Worf's power until end of
/// turn. Tap two untapped artifacts and/or creatures you control (Worf may be
/// one): a +1/+1 counter on Worf, sorcery speed.
pub fn worf_honor_bound_warrior() -> CardDefinition {
    CardDefinition {
        name: "Worf, Honor-Bound Warrior",
        cost: cost(&[hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Officer],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![attack_with_eight_power(Effect::DoublePower {
            what: Selector::This,
            times: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            tap_others_cost: Some((R::Artifact.or(R::Creature), 2)),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kirk, Commanding Officer — {1}{R}{W} legendary 3/3 Human Officer.
/// Attacking with total power 8+: exile the top card of your library, you may
/// play it this turn. {2}, sacrifice Kirk: destroy target artifact or
/// enchantment.
pub fn kirk_commanding_officer() -> CardDefinition {
    CardDefinition {
        name: "Kirk, Commanding Officer",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Officer], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![attack_with_eight_power(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: MayPlayDuration::EndOfThisTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Shields Up! — {1}{W} instant. Target artifact or creature you control
/// gains hexproof and indestructible until end of turn; if it's a creature, a
/// +1/+1 counter on it.
pub fn shields_up() -> CardDefinition {
    let grant = |keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword, duration: Duration::EndOfTurn };
    CardDefinition {
        name: "Shields Up!",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Artifact.or(R::Creature).and(R::ControlledByYou)),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            grant(Keyword::Indestructible),
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Creature },
                then: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Test of Identity — {1}{R} instant. Choose one — target Klingon gets +1/+1
/// and double strike until end of turn; or discard a card, and if you do,
/// draw two cards.
pub fn test_of_identity() -> CardDefinition {
    CardDefinition {
        name: "Test of Identity",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Klingon))),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::DoubleStrike, duration: Duration::EndOfTurn },
            ]),
            Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                crate::effect::shortcut::if_discarded(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
            ]),
        ]),
        ..Default::default()
    }
}

/// U.S.S. Cerritos, Following Up — {5} legendary Spacecraft. ETB: two 1/1 red
/// Officer creature tokens. Station; 6+: a 4/4 flying artifact creature.
pub fn u_s_s_cerritos_following_up() -> CardDefinition {
    let officer = Arc::new(TokenDefinition {
        name: "Officer".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Officer], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        name: "U.S.S. Cerritos, Following Up",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        power: 4,
        toughness: 4,
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 6, keywords: vec![Keyword::Flying], pt: Some((4, 4)), ..Default::default() }],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: officer,
        })],
        ..Default::default()
    }
}

/// The Measure of a Man — {2}{W} Saga. I — exile up to one target nontoken
/// artifact or creature. II — investigate. III — return the exiled card to the
/// battlefield under its owner's control; you gain 2 life.
pub fn the_measure_of_a_man() -> CardDefinition {
    CardDefinition {
        name: "The Measure of a Man",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::ExileLinked {
                        what: target_filtered(R::Artifact.or(R::Creature).and(R::NotToken)),
                    }),
                },
            ),
            (2, investigate(1)),
            (
                3,
                Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::CardExiledWithSource,
                        to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            ),
        ],
        ..Default::default()
    }
}
