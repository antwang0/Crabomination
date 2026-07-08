//! A twenty-fifth wave — Duskmourn (DSK) staples on existing primitives: the
//! "Fear of …" Nightmare enchantment-creature cycle, Eerie/Survival payoffs,
//! and assorted attack/dies triggers. Tests in
//! `crabomination/src/tests/recent25.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement, Selector, Subtypes,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{eerie, etb, on_attack, on_dies, target_any, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Card types for the "Fear of …" Nightmare enchantment-creature cycle.
fn nightmare(power: i32, toughness: i32) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power,
        toughness,
        ..Default::default()
    }
}

/// "another creature you control" target (not the source).
fn another_creature_you_control() -> SelectionRequirement {
    SelectionRequirement::Creature
        .and(SelectionRequirement::ControlledByYou)
        .and(SelectionRequirement::OtherThanSource)
}

// ── "Fear of …" Nightmare cycle ──────────────────────────────────────────────

/// Fear of Failed Tests — {4}{U} 2/7 Nightmare. Whenever it deals combat damage
/// to a player, draw that many cards.
pub fn fear_of_failed_tests() -> CardDefinition {
    CardDefinition {
        name: "Fear of Failed Tests",
        cost: cost(&[generic(4), u()]),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..nightmare(2, 7)
    }
}

/// Fear of Surveillance — {1}{W} 2/2 Nightmare. Vigilance; on attack, surveil 1.
pub fn fear_of_surveillance() -> CardDefinition {
    CardDefinition {
        name: "Fear of Surveillance",
        cost: cost(&[generic(1), w()]),
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_attack(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..nightmare(2, 2)
    }
}

/// Fear of Being Hunted — {1}{R}{R} 4/2 Nightmare. Haste; must be blocked if able.
pub fn fear_of_being_hunted() -> CardDefinition {
    CardDefinition {
        name: "Fear of Being Hunted",
        cost: cost(&[generic(1), r(), r()]),
        keywords: vec![Keyword::Haste, Keyword::MustBeBlocked],
        ..nightmare(4, 2)
    }
}


/// Fear of Immobility — {4}{W} 4/4 Nightmare. ETB: tap target creature; if an
/// opponent controls it, stun it. (The "up to one" optionality is approximated
/// to a required target.)
pub fn fear_of_immobility() -> CardDefinition {
    CardDefinition {
        name: "Fear of Immobility",
        cost: cost(&[generic(4), w()]),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::ControlledByOpponent,
                },
                then: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..nightmare(4, 4)
    }
}

// ── Other DSK creatures ──────────────────────────────────────────────────────

/// Flesh Burrower — {1}{G} 2/2 Insect. Deathtouch; on attack, another target
/// creature you control gains deathtouch until end of turn.
pub fn flesh_burrower() -> CardDefinition {
    CardDefinition {
        name: "Flesh Burrower",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(another_creature_you_control()),
            keyword: Keyword::Deathtouch,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Hardened Escort — {2}{W} 2/4 Human Soldier. On attack, another target
/// creature you control gets +1/+0 and gains indestructible until end of turn.
pub fn hardened_escort() -> CardDefinition {
    CardDefinition {
        name: "Hardened Escort",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(another_creature_you_control()),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Infernal Phantom — {3}{R} 2/3 Spirit. Eerie: gets +2/+0 until end of turn.
/// When it dies, it deals damage equal to its power to any target.
pub fn infernal_phantom() -> CardDefinition {
    CardDefinition {
        name: "Infernal Phantom",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: {
            let mut t = eerie(Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            });
            t.push(on_dies(Effect::DealDamage {
                to: target_any(),
                amount: Value::PowerOf(Box::new(Selector::This)),
            }));
            t
        },
        ..Default::default()
    }
}

/// Lionheart Glimmer — {3}{W}{W} 2/5 Cat Glimmer. Ward {2}; whenever you attack,
/// creatures you control get +1/+1 until end of turn.
pub fn lionheart_glimmer() -> CardDefinition {
    CardDefinition {
        name: "Lionheart Glimmer",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Glimmer],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}


/// Irreverent Gremlin — {1}{R} 2/2 Gremlin. Menace; once each turn when another
/// creature you control with power 2 or less enters, you may discard then draw.
pub fn irreverent_gremlin() -> CardDefinition {
    CardDefinition {
        name: "Irreverent Gremlin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gremlin], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .once_per_turn()
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::PowerAtMost(2))
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::MayDo {
                description: "Discard a card, then draw a card".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Anthropede — {3}{G} 3/4 Insect. Reach; ETB you may pay {2}, destroy target
/// Room. (The "discard a card" alternative cost is dropped.)
pub fn anthropede() -> CardDefinition {
    CardDefinition {
        name: "Anthropede",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {2} to destroy a Room".into(),
            mana_cost: cost(&[generic(2)]),
            body: Box::new(Effect::Destroy {
                what: target_filtered(SelectionRequirement::HasEnchantmentSubtype(
                    EnchantmentSubtype::Room,
                )),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

// ── Second wave ──────────────────────────────────────────────────────────────

/// Living Phone — {2}{W} 2/1 Artifact Toy. When it dies, look at the top five;
/// you may put a creature card with power 2 or less into your hand, rest on bottom.
pub fn living_phone() -> CardDefinition {
    CardDefinition {
        name: "Living Phone",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Toy], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: false,
            pick_filter: Some(SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2))),
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
        })],
        ..Default::default()
    }
}

/// Demonic Counsel — {1}{B} Sorcery. Search your library for a Demon card to
/// hand; Delirium — instead search for any card.
pub fn demonic_counsel() -> CardDefinition {
    CardDefinition {
        name: "Demonic Counsel",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::DeliriumActive { who: PlayerRef::You },
            then: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            }),
            else_: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasCreatureType(CreatureType::Demon),
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}
