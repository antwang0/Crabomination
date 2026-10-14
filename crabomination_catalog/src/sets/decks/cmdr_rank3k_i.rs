//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), ninth file. Tests in `tests/recent_b/cmdr_rank3k_i.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, u, w};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: n,
        definition: Arc::new(def),
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

// ── MJ, Rising Star ──────────────────────────────────────────────────────────

/// Spider-Man, Peter Parker — {3}{W}{W} legendary 4/4 flyer. Whenever you gain
/// life, a +1/+1 counter on target creature you control; it gains
/// indestructible until end of turn.
pub fn spider_man_peter_parker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..legend(
            "Spider-Man, Peter Parker",
            cost(&[generic(3), w(), w()]),
            vec![
                CreatureType::Spider,
                CreatureType::Human,
                CreatureType::Hero,
            ],
            4,
            4,
        )
    }
}

/// Amazing Alliance — {1}{W}{W} enchantment. Creatures you control get +1/+1.
/// Whenever you attack with one or more legendary creatures, gain that much
/// life.
pub fn amazing_alliance() -> CardDefinition {
    let legends = || {
        R::Creature
            .and(R::IsAttacking)
            .and(R::HasSupertype(Supertype::Legendary))
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: yours(R::Creature),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                .with_filter(Predicate::SelectorExists(yours(legends()))),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::PermanentCountControlledByMatching(PlayerRef::You, legends()),
            },
        }],
        ..spell(
            "Amazing Alliance",
            cost(&[generic(1), w(), w()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Silver Surfer, Cosmic Voyager ────────────────────────────────────────────

/// Silver Surfer, Cosmic Voyager — {4}{U}{U} legendary 5/5 flash flyer. ETB:
/// exile any number of other target permanents you control; they return under
/// their owners' control at the beginning of the next end step (a land
/// tapped).
pub fn silver_surfer_cosmic_voyager() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource),
            effect: Box::new(Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::Land,
                },
                then: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                    what: Selector::Target(0),
                    tapped: true,
                }),
                else_: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                    what: Selector::Target(0),
                    tapped: false,
                }),
            }),
        })],
        ..legend(
            "Silver Surfer, Cosmic Voyager",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Alien, CreatureType::Hero],
            5,
            5,
        )
    }
}

/// Spaceshift — {1}{U} instant. Exile target artifact or creature, then return
/// it under its owner's control with a +1/+1 counter on it.
pub fn spaceshift() -> CardDefinition {
    spell(
        "Spaceshift",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(R::Artifact.or(R::Creature)),
            },
            Effect::ClearLastMoved,
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    tapped: false,
                },
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
    )
}

// ── Jason Bright, Glowing Prophet ────────────────────────────────────────────

/// Patchwork Crawler — {1}{U} 1/2 Zombie Horror. {2}{U}: exile target creature
/// card from your graveyard and a +1/+1 counter on it. It has the activated
/// abilities of every creature card exiled with it.
pub fn patchwork_crawler() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has all activated abilities of all creature cards exiled with it.",
            effect: StaticEffect::HasActivatedAbilitiesOfExiledWithSelf,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.from_your_graveyard()),
                    to: ZoneDest::ExileWithSourceStamp,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Patchwork Crawler",
            cost(&[generic(1), u()]),
            vec![CreatureType::Zombie, CreatureType::Horror],
            1,
            2,
        )
    }
}

/// Compelling Deterrence — {1}{U} instant. Return target nonland permanent to
/// its owner's hand; then that player discards a card if you control a Zombie.
pub fn compelling_deterrence() -> CardDefinition {
    spell(
        "Compelling Deterrence",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::Land.negate())),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::If {
                cond: Predicate::SelectorExists(yours(R::HasCreatureType(CreatureType::Zombie))),
                then: Box::new(Effect::Discard {
                    who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    amount: Value::ONE,
                    random: false,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

// ── Kodama of the Center Tree / Kodama of the South Tree ─────────────────────

/// Permeating Mass — {G} 1/3 Spirit. Whenever it deals combat damage to a
/// creature, that creature becomes a copy of it.
pub fn permeating_mass() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToCreature,
                EventScope::SelfSource,
            ),
            effect: Effect::BecomeCopyOf {
                what: Selector::Target(0),
                source: Selector::This,
                extra_creature_types: vec![],
                keep_own_triggered: false,
                keep_own_activated: false,
                keep_name: false,
            },
        }],
        ..creature(
            "Permeating Mass",
            cost(&[g()]),
            vec![CreatureType::Spirit],
            1,
            3,
        )
    }
}

/// Invoke the Ancients — {1}{G}{G}{G}{G} sorcery. Two 4/5 green Spirits, each
/// with your choice of a reach, vigilance or trample counter. (Residual: the
/// pick is "up to one", so a seat may leave a Spirit without a counter.)
pub fn invoke_the_ancients() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 4,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    };
    let counter = |keyword| Effect::AddKeywordCounter {
        what: Selector::TriggerSource,
        keyword,
        amount: Value::ONE,
    };
    spell(
        "Invoke the Ancients",
        cost(&[generic(1), g(), g(), g(), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            make(Value::Const(2), spirit),
            Effect::ForEach {
                selector: Selector::LastCreatedTokens,
                body: Box::new(Effect::ChooseUpToN {
                    max: Box::new(Value::ONE),
                    modes: vec![
                        counter(Keyword::Reach),
                        counter(Keyword::Vigilance),
                        counter(Keyword::Trample),
                    ],
                }),
            },
        ]),
    )
}

// ── Infernal Kirin / Kyoki, Sanity's Eclipse ─────────────────────────────────

/// Myojin of Grim Betrayal — {5}{B}{B}{B} legendary 5/2 Spirit. Cast from hand,
/// it enters with an indestructible counter. Remove it: put onto the
/// battlefield under your control every creature card put into any graveyard
/// from anywhere this turn.
pub fn myojin_of_grim_betrayal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::CastFromHand),
            effect: Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_keyword_counter_cost: Some(Keyword::Indestructible),
            effect: Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: Zone::Graveyard,
                    filter: R::Creature.and(R::PutIntoGraveyardThisTurn),
                },
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            ..Default::default()
        }],
        ..legend(
            "Myojin of Grim Betrayal",
            cost(&[generic(5), b(), b(), b()]),
            vec![CreatureType::Spirit],
            5,
            2,
        )
    }
}

/// Urborg Scavengers — {2}{B} 2/2 Spirit. Enters or attacks: exile target card
/// from a graveyard, then a +1/+1 counter on it. It has each of flying, first
/// strike, double strike, deathtouch, haste, hexproof, indestructible,
/// lifelink, menace, reach, trample and vigilance while a card exiled with it
/// has that keyword.
pub fn urborg_scavengers() -> CardDefinition {
    let scavenge = || {
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Any.and(R::InGraveyard)),
                to: ZoneDest::ExileWithSourceStamp,
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ])
    };
    let shared = [
        Keyword::Flying,
        Keyword::FirstStrike,
        Keyword::DoubleStrike,
        Keyword::Deathtouch,
        Keyword::Haste,
        Keyword::Hexproof,
        Keyword::Indestructible,
        Keyword::Lifelink,
        Keyword::Menace,
        Keyword::Reach,
        Keyword::Trample,
        Keyword::Vigilance,
    ];
    CardDefinition {
        static_abilities: shared
            .into_iter()
            .map(|keyword| StaticAbility {
                description: "This creature has each listed keyword a card exiled with it has.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::ValueAtLeast(
                        Value::CardsExiledWithSourceMatching(R::HasKeyword(keyword.clone())),
                        Value::ONE,
                    ),
                    inner: Box::new(StaticEffect::GrantKeyword {
                        applies_to: Selector::This,
                        keyword,
                    }),
                },
            })
            .collect(),
        triggered_abilities: vec![etb(scavenge()), on_attack(scavenge())],
        ..creature(
            "Urborg Scavengers",
            cost(&[generic(2), b()]),
            vec![CreatureType::Spirit],
            2,
            2,
        )
    }
}
