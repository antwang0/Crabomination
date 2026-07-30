//! Ravnica batch 5: Rakdos Hellbent, Boros/Gruul combat, and a Dimir land
//! swap. Reuses existing primitives — `Predicate::HellbentActive`, Bloodthirst,
//! `Value::{PowerOf,TriggerEventAmount,CardsDrawnThisTurn}`, `PreventNextDamage`,
//! and `ExchangeControl`. Tests in `recent_b/recent_295`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{bloodthirst, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

// ── Gruul / Boros combat ────────────────────────────────────────────────────

/// Bloodscale Prowler — {2}{R} 3/1 Lizard Warrior with Bloodthirst 1.
pub fn bloodscale_prowler() -> CardDefinition {
    CardDefinition {
        name: "Bloodscale Prowler",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Bloodthirst(1)],
        triggered_abilities: vec![bloodthirst(1)],
        ..Default::default()
    }
}

/// Ordruun Commando — {3}{R} 4/1 Minotaur Soldier. {W}: Prevent the next 1
/// damage that would be dealt to this creature this turn.
pub fn ordruun_commando() -> CardDefinition {
    CardDefinition {
        name: "Ordruun Commando",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::PreventNextDamage {
                target: Selector::This,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Feral Animist — {1}{R}{G} 2/1 Goblin Shaman. {3}: This creature gets +X/+0
/// until end of turn, where X is its power.
pub fn feral_animist() -> CardDefinition {
    CardDefinition {
        name: "Feral Animist",
        cost: cost(&[generic(1), r(), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Coalhauler Swine — {4}{R}{R} 4/4 Boar Beast. Whenever it's dealt damage, it
/// deals that much damage to each player.
pub fn coalhauler_swine() -> CardDefinition {
    CardDefinition {
        name: "Coalhauler Swine",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    }
}

// ── Simic Graft ─────────────────────────────────────────────────────────────

/// Vigean Hydropon — {1}{G}{U} 0/0 Plant Mutant, Graft 5. Can't attack or block.
pub fn vigean_hydropon() -> CardDefinition {
    CardDefinition {
        name: "Vigean Hydropon",
        cost: cost(&[generic(1), crate::mana::g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Mutant],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(5))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        ..Default::default()
    }
}

// ── Rakdos Hellbent ─────────────────────────────────────────────────────────

/// Twinstrike — {3}{B}{R} Instant. Deal 2 damage to each of two target
/// creatures; Hellbent — destroy those creatures instead if you have no cards
/// in hand.
pub fn twinstrike() -> CardDefinition {
    let t0 = || Selector::TargetFiltered {
        slot: 0,
        filter: R::Creature,
    };
    let t1 = || Selector::TargetFiltered {
        slot: 1,
        filter: R::Creature,
    };
    CardDefinition {
        name: "Twinstrike",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::HellbentActive {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::Seq(vec![
                Effect::Destroy { what: t0() },
                Effect::Destroy { what: t1() },
            ])),
            else_: Box::new(Effect::Seq(vec![
                Effect::DealDamage {
                    to: t0(),
                    amount: Value::Const(2),
                },
                Effect::DealDamage {
                    to: t1(),
                    amount: Value::Const(2),
                },
            ])),
        },
        ..Default::default()
    }
}

// ── Izzet / Dimir ───────────────────────────────────────────────────────────

/// Poisonbelly Ogre — {4}{B} 3/3 Ogre Warrior. Whenever another creature
/// enters, its controller loses 1 life.
pub fn poisonbelly_ogre() -> CardDefinition {
    CardDefinition {
        name: "Poisonbelly Ogre",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                },
            ),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Devouring Light — {1}{W}{W} Instant with Convoke. Exile target attacking or
/// blocking creature.
pub fn devouring_light() -> CardDefinition {
    CardDefinition {
        name: "Devouring Light",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Convoke],
        effect: Effect::Move {
            what: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Fangren Pathcutter — {4}{G}{G} 4/6 Beast. Whenever it attacks, attacking
/// creatures gain trample until end of turn.
pub fn fangren_pathcutter() -> CardDefinition {
    CardDefinition {
        name: "Fangren Pathcutter",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: Selector::EachPermanent(R::IsAttacking),
            keyword: Keyword::Trample,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Root-Kin Ally — {4}{G}{G} 3/3 Elemental Warrior with Convoke. Tap two
/// untapped creatures you control: This creature gets +2/+2 until end of turn.
pub fn root_kin_ally() -> CardDefinition {
    CardDefinition {
        name: "Root-Kin Ally",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Convoke],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature.and(R::ControlledByYou), 2)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Boros Radiance ──────────────────────────────────────────────────────────

/// Cleansing Beam — {4}{R} Instant. Radiance — deal 2 damage to target creature
/// and each other creature that shares a color with it.
pub fn cleansing_beam() -> CardDefinition {
    CardDefinition {
        name: "Cleansing Beam",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::RadianceDamage {
            subject: target_filtered(R::Creature),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Wojek Embermage — {3}{R} 1/2 Human Wizard. Radiance — {T}: deal 1 damage to
/// target creature and each other creature that shares a color with it.
pub fn wojek_embermage() -> CardDefinition {
    CardDefinition {
        name: "Wojek Embermage",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::RadianceDamage {
                subject: target_filtered(R::Creature),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
