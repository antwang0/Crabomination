//! Commander: the 2026-10-13 rescan's two-card gaps, second half — Brion,
//! Aerith, Kraven, Katilda and Lier, Endrek Sahr, Inferno of the Star Mounts,
//! Avatar Roku, Olivia Crimson Bride, Amarant Coral (`cmdr_edhrec24` is the
//! first half). Tests in `tests/recent_b/cmdr_edhrec25.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};

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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn on(kind: EventKind, scope: EventScope, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(kind, scope), effect }
}

fn source_is(kind: EventKind, scope: EventScope, filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(kind, scope)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect,
    }
}

fn plus_one(what: Selector, n: i32) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::Const(n) }
}

fn until_eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn also_type(what: Selector, kind: CreatureType) -> Effect {
    Effect::AddCreatureTypes { what, creature_types: vec![kind], duration: Duration::Permanent }
}

// ── Brion Stoutarm ───────────────────────────────────────────────────────────

/// Calamity Bearer — {2}{R}{R} 3/4 Giant Berserker. Giant sources you
/// control deal double damage.
pub fn calamity_bearer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a Giant source you control would deal damage to a permanent or player, it deals double that damage instead.",
            effect: StaticEffect::DoubleDamageFromControlledMatching { filter: R::HasCreatureType(CreatureType::Giant) },
        }],
        ..creature("Calamity Bearer", cost(&[generic(2), r(), r()]), vec![CreatureType::Giant, CreatureType::Berserker], 3, 4)
    }
}

// ── Aerith Gainsborough ──────────────────────────────────────────────────────

/// Staff of the Sun Magus — {3} artifact. Casting a white spell, or a Plains
/// entering under your control, gains you 1 life.
pub fn staff_of_the_sun_magus() -> CardDefinition {
    let gain = || Effect::GainLife { who: Selector::You, amount: Value::ONE };
    CardDefinition {
        name: "Staff of the Sun Magus",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            source_is(EventKind::SpellCast, EventScope::YourControl, R::HasColor(Color::White), gain()),
            source_is(EventKind::EntersBattlefield, EventScope::YourControl, R::HasLandType(LandType::Plains), gain()),
        ],
        ..Default::default()
    }
}

/// Urdnan, Dromoka Warrior — {1}{W} 1/1 legendary Human Warrior. ETB: a
/// +1/+1 counter on target creature. Whenever you attack, target attacking
/// creature with a +1/+1 counter gains first strike — double strike with two
/// or more.
pub fn urdnan_dromoka_warrior() -> CardDefinition {
    let counters = || Value::CountersOn { what: Box::new(Selector::Target(0)), kind: CounterType::PlusOnePlusOne };
    CardDefinition {
        triggered_abilities: vec![
            etb(plus_one(target_filtered(R::Creature), 1)),
            on(
                EventKind::YouAttack,
                EventScope::YourControl,
                Effect::If {
                    cond: Predicate::ValueAtLeast(counters(), Value::Const(2)),
                    then: Box::new(until_eot(Selector::Target(0), Keyword::DoubleStrike)),
                    else_: Box::new(until_eot(
                        target_filtered(R::IsAttacking.and(R::WithCounter(CounterType::PlusOnePlusOne))),
                        Keyword::FirstStrike,
                    )),
                },
            ),
        ],
        ..legend("Urdnan, Dromoka Warrior", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Warrior], 1, 1)
    }
}

// ── Kraven the Hunter ────────────────────────────────────────────────────────

/// Lich's Relic — {B} Equipment. ETB: you may pay {2}; when you do, for each
/// opponent destroy up to one target creature or planeswalker they control.
/// Equipped creature gets +2/+1. Equip {2}.
pub fn lichs_relic() -> CardDefinition {
    CardDefinition {
        name: "Lich's Relic",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {2} to destroy a creature or planeswalker per opponent?".into(),
            mana_cost: cost(&[generic(2)]),
            body: Box::new(Effect::ReflexiveTrigger {
                body: Box::new(Effect::ForEachOpponentTarget {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 8,
                        min_targets: 0,
                        filter: R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent),
                        effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                    }),
                }),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Gix's Command — {3}{B}{B} sorcery. Choose two: two +1/+1 counters and
/// lifelink on up to one creature; destroy each creature with power 2 or
/// less; return up to two creature cards from your graveyard to your hand;
/// each opponent sacrifices their greatest-power creature.
pub fn gixs_command() -> CardDefinition {
    spell(
        "Gix's Command",
        cost(&[generic(3), b(), b()]),
        CardType::Sorcery,
        Effect::ChooseN { up_to: None,
            picks: vec![0, 3],
            modes: vec![
                // "Up to one creature" is chosen as the mode resolves (no target).
                Effect::ChooseSomeAmong {
                    what: Selector::EachPermanent(R::Creature),
                    chooser: PlayerRef::You,
                    count: Value::ONE,
                    up_to: true,
                    chosen: Box::new(Effect::Seq(vec![
                        plus_one(Selector::SeparatedPile { chosen: true }, 2),
                        until_eot(Selector::SeparatedPile { chosen: true }, Keyword::Lifelink),
                    ])),
                    other: Box::new(Effect::Noop),
                },
                Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::PowerAtMost(2))) },
                Effect::MoveChosen {
                    from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature },
                    filter: None,
                    count: Value::Const(2),
                    up_to: true,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: R::Creature.and(R::HasGreatestPowerAmongControlled(Box::new(R::Creature))),
                },
            ],
        },
    )
}

// ── Katilda and Lier ─────────────────────────────────────────────────────────

/// Katilda and Lier — {G}{W}{U} 3/3 legendary Human. Whenever you cast a
/// Human spell, target instant or sorcery card in your graveyard gains
/// flashback until end of turn (its mana cost).
pub fn katilda_and_lier() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasCreatureType(CreatureType::Human))),
            effect: Effect::GrantFlashbackThisTurn {
                what: target_filtered(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).from_your_graveyard(),
                ),
            },
        }],
        ..legend("Katilda and Lier", cost(&[g(), w(), u()]), vec![CreatureType::Human], 3, 3)
    }
}

// ── Endrek Sahr, Master Breeder ──────────────────────────────────────────────

/// Whisper, Blood Liturgist — {3}{B} 2/2 legendary Human Cleric. {T},
/// sacrifice two creatures: return target creature card from your graveyard
/// to the battlefield.
pub fn whisper_blood_liturgist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 2)),
            sac_other_may_be_source: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.from_your_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..legend("Whisper, Blood Liturgist", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Cleric], 2, 2)
    }
}

/// Soul Exchange — {B}{B} sorcery. Additional cost: exile a creature you
/// control. Return target creature card from your graveyard to the
/// battlefield; put a +2/+2 counter on it if the exiled creature was a Thrull.
pub fn soul_exchange() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::ExilePermanent { filter: R::Creature, count: 1 }],
        ..spell(
            "Soul Exchange",
            cost(&[b(), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.from_your_graveyard()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::If {
                    cond: Predicate::EntityMatchesAny {
                        what: Selector::CostExiledCards,
                        filter: R::HasCreatureType(CreatureType::Thrull),
                    },
                    then: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusTwoPlusTwo,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

// ── Inferno of the Star Mounts / Avatar Roku ─────────────────────────────────

/// Soulbright Flamekin — {1}{R} 2/1 Elemental Shaman. {2}: target creature
/// gains trample until end of turn; the third resolution this turn may add
/// {R}{R}{R}{R}{R}{R}{R}{R}.
pub fn soulbright_flamekin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                until_eot(target_filtered(R::Creature), Keyword::Trample),
                Effect::NthResolutionThisTurn {
                    branches: vec![
                        Effect::Noop,
                        Effect::Noop,
                        Effect::MayDo {
                            description: "Add {R}{R}{R}{R}{R}{R}{R}{R}?".into(),
                            body: Box::new(Effect::AddMana {
                                who: PlayerRef::You,
                                pool: ManaPayload::OfColor(Color::Red, Value::Const(8)),
                            }),
                        },
                    ],
                },
            ]),
            ..Default::default()
        }],
        ..creature("Soulbright Flamekin", cost(&[generic(1), r()]), vec![CreatureType::Elemental, CreatureType::Shaman], 2, 1)
    }
}

/// Fang, Roku's Companion — {3}{R}{R} 4/4 legendary Dragon, flying. Whenever
/// he attacks, another target legendary creature you control gets +X/+0
/// (X his power). When he dies, if he wasn't a Spirit, return him as a
/// Spirit too.
pub fn fang_rokus_companion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_attack(Effect::PumpPT {
                what: target_filtered(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(Predicate::Not(
                    Box::new(Predicate::EntityMatches {
                        what: Selector::This,
                        filter: R::HasCreatureType(CreatureType::Spirit),
                    }),
                )),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    also_type(Selector::LastMoved, CreatureType::Spirit),
                ]),
            },
        ],
        ..legend("Fang, Roku's Companion", cost(&[generic(3), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

// ── Olivia, Crimson Bride ────────────────────────────────────────────────────

/// Drana, the Last Bloodchief — {3}{B}{B} 4/4 legendary Vampire Cleric,
/// flying. Whenever she attacks, the defending player chooses a nonlegendary
/// creature card in your graveyard; it returns with a +1/+1 counter and is a
/// Vampire too.
pub fn drana_the_last_bloodchief() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::ChooseOneAmong {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::Creature.and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
            },
            chooser: PlayerRef::DefendingPlayer,
            chosen: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::SeparatedPile { chosen: true },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                plus_one(Selector::LastMoved, 1),
                also_type(Selector::LastMoved, CreatureType::Vampire),
            ])),
            other: Box::new(Effect::Noop),
        })],
        ..legend(
            "Drana, the Last Bloodchief",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Cleric],
            4,
            4,
        )
    }
}

/// Olivia, Mobilized for War — {1}{B}{R} 3/3 legendary Vampire Knight,
/// flying. Whenever another creature you control enters, you may discard a
/// card; if you do, it gets a +1/+1 counter, haste until end of turn, and
/// becomes a Vampire.
pub fn olivia_mobilized_for_war() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![source_is(
            EventKind::EntersBattlefield,
            EventScope::AnotherOfYours,
            R::Creature,
            Effect::MayDiscard {
                description: "Discard a card to empower the creature?".into(),
                count: Value::ONE,
                then: Box::new(Effect::Seq(vec![
                    plus_one(Selector::TriggerSource, 1),
                    until_eot(Selector::TriggerSource, Keyword::Haste),
                    also_type(Selector::TriggerSource, CreatureType::Vampire),
                ])),
                else_: None,
            },
        )],
        ..legend(
            "Olivia, Mobilized for War",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Vampire, CreatureType::Knight],
            3,
            3,
        )
    }
}

// ── Amarant Coral ────────────────────────────────────────────────────────────

/// Amarant Coral — {2}{R}{G} 5/4 legendary Human Monk, trample; attacks each
/// combat if able. Its combat damage to an opponent is dealt again to each
/// other opponent.
pub fn amarant_coral() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::MustAttack],
        triggered_abilities: vec![on(
            EventKind::DealsCombatDamageToPlayer,
            EventScope::SelfSource,
            Effect::DealDamageFrom {
                source: Selector::This,
                to: Selector::Player(PlayerRef::EachOpponentExceptTriggerer),
                amount: Value::TriggerEventAmount,
            },
        )],
        ..legend("Amarant Coral", cost(&[generic(2), r(), g()]), vec![CreatureType::Human, CreatureType::Monk], 5, 4)
    }
}

/// Strength of Will — {1}{G} instant. Until end of turn, target creature you
/// control gains indestructible and "Whenever this creature is dealt damage,
/// put that many +1/+1 counters on it."
pub fn strength_of_will() -> CardDefinition {
    let mine = || target_filtered(R::Creature.and(R::ControlledByYou));
    spell(
        "Strength of Will",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            until_eot(mine(), Keyword::Indestructible),
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(on(
                    EventKind::DealtDamage,
                    EventScope::SelfSource,
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::TriggerEventAmount,
                    },
                )),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Evra, Halcyon Witness — {4}{W}{W} 4/4 legendary Avatar, lifelink. {4}:
/// exchange your life total with Evra's power.
pub fn evra_halcyon_witness() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            effect: Effect::ExchangeLifeWithSourcePower,
            ..Default::default()
        }],
        ..legend("Evra, Halcyon Witness", cost(&[generic(4), w(), w()]), vec![CreatureType::Avatar], 4, 4)
    }
}
