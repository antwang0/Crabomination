//! Commander: gaps of EDHREC's commanders ranked 1401-2000 (the 2026-10-13
//! rescan), eighth file — single-card gaps. Tests in
//! `tests/recent_b/cmdr_rank2k_h.rs`.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{counter_target_spell, etb, on_attack, on_cast, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::mana::{b, cost, generic, r, u, w, Color, ManaCost};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

/// "Recruit" (Bard's Company): draw a card, then discard a card; a nonland
/// discard makes a 1/1 white Human Soldier.
fn recruit() -> Effect {
    Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::ONE },
        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        Effect::If {
            cond: Predicate::DiscardedNonlandThisEffect { who: PlayerRef::You },
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Human Soldier".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::White],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Human, CreatureType::Soldier],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            }),
            else_: Box::new(Effect::Noop),
        },
    ])
}

/// Bard's Company — {2}{W}{U} 2/3 Human Citizen; flash while you control a
/// Human; other creatures you control get +1/+1; entering or attacking
/// recruits.
pub fn bards_company() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "You may cast this spell as though it had flash if you control a Human.",
                StaticEffect::SelfFlashIf {
                    condition: Predicate::SelectorCountAtLeast {
                        sel: Selector::ControlledBy { who: PlayerRef::You, filter: R::HasCreatureType(CreatureType::Human) },
                        n: Value::ONE,
                    },
                },
            ),
            static_ab(
                "Other creatures you control get +1/+1.",
                StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                    power: 1,
                    toughness: 1,
                },
            ),
        ],
        triggered_abilities: vec![etb(recruit()), on_attack(recruit())],
        ..creature(
            "Bard's Company",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Citizen],
            2,
            3,
        )
    }
}

/// Blightbeetle — {1}{B} 1/1 Insect; protection from green; creatures your
/// opponents control can't have +1/+1 counters put on them.
pub fn blightbeetle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Green)],
        static_abilities: vec![static_ab(
            "Creatures your opponents control can't have +1/+1 counters put on them.",
            StaticEffect::OpponentsCreaturesCantHavePlusOneCounters,
        )],
        ..creature("Blightbeetle", cost(&[generic(1), b()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Courageous Resolve — {2}{W} instant. Up to one creature you control gains
/// protection from each of your opponents this turn; draw a card. Fateful
/// hour: at 5 or less life, you can't lose life or the game this turn and
/// your opponents can't win it.
pub fn courageous_resolve() -> CardDefinition {
    spell(
        "Courageous Resolve",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::GainProtectionFromPlayer {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    from: PlayerRef::EachOpponent,
                    duration: Duration::EndOfTurn,
                }),
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::If {
                cond: Predicate::PlayerLifeAtMost { who: PlayerRef::You, life: 5 },
                then: Box::new(Effect::Seq(vec![
                    Effect::CantLoseLifeThisTurn { who: Selector::You },
                    Effect::CantLoseThisTurn { damage_floor: false },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Disruption Protocol — {U}{U} instant; as an additional cost, tap an
/// untapped artifact you control or pay {1}. Counter target spell.
pub fn disruption_protocol() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::TapOrPay { filter: R::Artifact, pay: 1 }],
        ..spell("Disruption Protocol", cost(&[u(), u()]), CardType::Instant, counter_target_spell())
    }
}

/// Extraction Specialist — {2}{W} 3/2 lifelink Human Rogue; entering returns a
/// creature card with mana value 2 or less from your graveyard to the
/// battlefield, and it can't attack or block while you have the Specialist.
pub fn extraction_specialist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(2))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CantAttack,
                duration: Duration::WhileSourceOnBattlefield,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CantBlock,
                duration: Duration::WhileSourceOnBattlefield,
            },
        ]))],
        ..creature(
            "Extraction Specialist",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            2,
        )
    }
}

/// Induced Amnesia — {2}{U} enchantment. Entering, target player exiles their
/// hand face down and draws that many; when this goes to a graveyard from the
/// battlefield, the exiled cards return to their owner's hand.
pub fn induced_amnesia() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::ExileHandLinked { who: PlayerRef::Target(0) },
                Effect::Draw { who: target_filtered(R::Player), amount: Value::CardsExiledWithSourceCount },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
                effect: Effect::ReturnLinkedExilesToHand {
                    who: PlayerRef::OwnerOf(Box::new(Selector::CardExiledWithSource)),
                },
            },
        ],
        ..spell("Induced Amnesia", cost(&[generic(2), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Samut, Tyrant of Naktamun — {1}{U} legendary 2/1 Human Wizard; instant and
/// sorcery spells you control have split second.
pub fn samut_tyrant_of_naktamun() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![static_ab(
            "Instant and sorcery spells you control have split second.",
            StaticEffect::YourISSpellsHaveSplitSecond,
        )],
        ..creature(
            "Samut, Tyrant of Naktamun",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            1,
        )
    }
}

/// Temporal Extortion — {B}{B}{B}{B} sorcery: take an extra turn. When you
/// cast it, any player may pay half their life, rounded up, to counter it.
pub fn temporal_extortion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_cast(Effect::AnyPlayerMayAccept {
            who: PlayerRef::EachPlayer,
            prompt: "Pay half your life, rounded up, to counter Temporal Extortion?".into(),
            accepted: Box::new(Effect::Seq(vec![
                Effect::LoseHalfLife { who: Selector::Player(PlayerRef::AcceptingPlayer), rounded_up: true },
                Effect::CounterSpell { what: Selector::This },
            ])),
            otherwise: Box::new(Effect::Noop),
        })],
        ..spell(
            "Temporal Extortion",
            cost(&[b(), b(), b(), b()]),
            CardType::Sorcery,
            Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
        )
    }
}

/// Tyrant of Discord — {4}{R}{R}{R} 7/7 Elemental. Entering, target opponent
/// sacrifices a random permanent of theirs, again and again until a land
/// goes.
pub fn tyrant_of_discord() -> CardDefinition {
    let theirs = || Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Permanent };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            // Names the target opponent; loses nothing.
            Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::ZERO },
            // At most one pass per permanent they control; each pass stops
            // once a land has been sacrificed.
            Effect::Repeat {
                count: Value::count(theirs()),
                body: Box::new(Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::ValueAtLeast(
                        Value::SacrificedThisResolutionBy { who: PlayerRef::Target(0), filter: R::Land },
                        Value::ONE,
                    ))),
                    then: Box::new(Effect::SacrificePermanent { what: Selector::RandomOf(Box::new(theirs())) }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        ]))],
        ..creature("Tyrant of Discord", cost(&[generic(4), r(), r(), r()]), vec![CreatureType::Elemental], 7, 7)
    }
}
