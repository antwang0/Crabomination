//! Commander: one-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), eighteenth file: the commanders that were their own decks' one-card gap.
//! Tests in `tests/recent_b/cmdr_rank3k_r.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{AttackingTokenCleanup, Duration, Effect, PlayerRef, Selector};
use crate::mana::{Color, ManaCost, cost, g, generic, r, u, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
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

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

// ── Palladia-Mors, the Ruiner ───────────────────────────────────────────────

/// Palladia-Mors, the Ruiner — {3}{R}{G}{W} legendary 6/6 Elder Dragon;
/// flying, vigilance, trample; hexproof as long as it hasn't dealt damage yet.
pub fn palladia_mors_the_ruiner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Palladia-Mors has hexproof as long as it hasn't dealt damage yet.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::Not(Box::new(Predicate::SourceHasDealtDamage)),
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::This,
                    keyword: Keyword::Hexproof,
                }),
            },
        }],
        ..legend(
            "Palladia-Mors, the Ruiner",
            cost(&[generic(3), r(), g(), w()]),
            vec![CreatureType::Elder, CreatureType::Dragon],
            6,
            6,
        )
    }
}

// ── Jiang Yanggu, Alone ──────────────────────────────────────────────────────

/// Jiang Yanggu, Alone — {4}{R} legendary 4/4 menace Human Berserker. Whenever
/// a creature you control attacks a player alone, discard a card, then draw a
/// card; then a +1/+1 counter on it for each card you've discarded this turn.
/// (Residual: an attack alone on a planeswalker or battle triggers too.)
pub fn jiang_yanggu_alone() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsAttackingAlone,
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CardsDiscardedThisTurn(PlayerRef::You),
                },
            ]),
        }],
        ..legend(
            "Jiang Yanggu, Alone",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            4,
            4,
        )
    }
}

// ── Super-Adaptoid ───────────────────────────────────────────────────────────

/// Super-Adaptoid — {2} legendary */2 Robot Villain artifact creature; power
/// equal to the legendary creatures you control. Enters or attacks: choose
/// another target creature; for each of haste, flying, first strike, double
/// strike, deathtouch, indestructible, lifelink, menace, reach, trample and
/// vigilance it has and Super-Adaptoid lacks, a counter of that keyword on
/// Super-Adaptoid.
pub fn super_adaptoid() -> CardDefinition {
    let keywords = [
        Keyword::Haste,
        Keyword::Flying,
        Keyword::FirstStrike,
        Keyword::DoubleStrike,
        Keyword::Deathtouch,
        Keyword::Indestructible,
        Keyword::Lifelink,
        Keyword::Menace,
        Keyword::Reach,
        Keyword::Trample,
        Keyword::Vigilance,
    ];
    let adapt = || {
        // The +0/+0 pump only declares the target slot the checks read.
        let mut steps = vec![Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::OtherThanSource)),
            power: Value::ZERO,
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        }];
        steps.extend(keywords.iter().map(|k| Effect::If {
            cond: Predicate::All(vec![
                Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasKeyword(k.clone()),
                },
                Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::HasKeyword(k.clone()),
                })),
            ]),
            then: Box::new(Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: k.clone(),
                amount: Value::ONE,
            }),
            else_: Box::new(Effect::Noop),
        }));
        Effect::Seq(steps)
    };
    let legends = || {
        Selector::EachPermanent(
            R::Creature
                .and(R::ControlledByYou)
                .and(R::HasSupertype(Supertype::Legendary)),
        )
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Super-Adaptoid's power is equal to the number of legendary creatures you control.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: Value::CountOf(Box::new(legends())),
                toughness: Value::Const(2),
            },
        }],
        triggered_abilities: vec![etb(adapt()), on_attack(adapt())],
        ..legend(
            "Super-Adaptoid",
            cost(&[generic(2)]),
            vec![CreatureType::Robot, CreatureType::Villain],
            0,
            2,
        )
    }
}

// ── Impossible Man ───────────────────────────────────────────────────────────

/// Impossible Man — {2}{U} legendary 1/4 flying Alien Shapeshifter. {2}{U}: he
/// becomes a copy of another target permanent until end of turn, except his
/// name is Impossible Man. (Residual: the copy takes the copied name.)
pub fn impossible_man() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::BecomeCopyOfFor {
                what: Selector::This,
                source: target_filtered(R::Permanent.and(R::OtherThanSource)),
                duration: Duration::EndOfTurn,
                non_legendary: false,
            },
            ..Default::default()
        }],
        ..legend(
            "Impossible Man",
            cost(&[generic(2), u()]),
            vec![CreatureType::Alien, CreatureType::Shapeshifter],
            1,
            4,
        )
    }
}

// ── Stangg, Echo Warrior ─────────────────────────────────────────────────────

/// Stangg, Echo Warrior — {2}{R}{G} legendary 3/4 Human Warrior. Whenever he
/// attacks, create Stangg Twin, a legendary 3/4 red and green Human Warrior
/// token, tapped and attacking; sacrifice it at the beginning of the next end
/// step. (Residual: copies of the Auras and Equipment attached to Stangg
/// aren't made for the Twin.)
pub fn stangg_echo_warrior() -> CardDefinition {
    let twin = TokenDefinition {
        name: "Stangg Twin".into(),
        power: 3,
        toughness: 4,
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        colors: vec![Color::Red, Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::CreateTokenAttacking {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(twin),
            cleanup: AttackingTokenCleanup::SacrificeAtNextEndStep,
            defender: None,
        })],
        ..legend(
            "Stangg, Echo Warrior",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            4,
        )
    }
}
