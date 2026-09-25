//! Commander: the cards the **Deadly Disguise** precon (MKC, Kaust, Eyes of
//! the Glade) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_kaust.rs`.
//!
//! Residuals (each also on its card):
//! - **Boltbender** — turning it up re-aims one target spell, not any number
//!   of spells and abilities.
//! - **Tesak, Judith's Hellhound** — other Dogs don't gain unleash.
//! - **Unexplained Absence** — it never takes one of your own permanents
//!   (the printed "for each player" includes you).
//! - **Veiled Ascension** — face-down creatures get their flying counter
//!   from a trigger after they enter, not as they enter.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{afflict, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, hybrid, r, w, x, Color, ManaCost};

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

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// "When this is turned face up, …"
fn turned_up(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource), effect }
}

/// "Whenever a permanent you control is turned face up, …" (this one
/// included).
fn yours_turned_up(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::YourControl), effect }
}

fn yours() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn base_two_two() -> R {
    R::BasePowerToughnessIs(2, 2)
}

/// Kaust, Eyes of the Glade — creatures turned face up this turn draw on
/// combat damage to a player; tap to turn a face-down attacker of yours up.
pub fn kaust_eyes_of_the_glade() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::TurnedFaceUpThisTurn },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::TurnFaceUpFree {
                what: target_filtered(R::FaceDown.and(R::IsAttacking).and(R::ControlledByYou)),
                if_cant: None,
            },
            ..Default::default()
        }],
        ..legend(
            "Kaust, Eyes of the Glade",
            cost(&[hybrid(Color::Red, Color::White), g()]),
            vec![CreatureType::Dryad, CreatureType::Detective],
            2,
            2,
        )
    }
}

/// Ashcloud Phoenix — flying; dying returns it face down; turning it up
/// deals 2 to each player.
pub fn ashcloud_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(4), r(), r()]))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::PutFaceDownOntoBattlefield { what: Selector::This },
            },
            turned_up(Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) }),
        ],
        ..creature("Ashcloud Phoenix", cost(&[generic(2), r(), r()]), vec![CreatureType::Phoenix], 4, 1)
    }
}

/// Boltbender — disguise; turning it up lets you re-aim a spell.
///
/// Residual: one target spell, not any number of spells and abilities.
pub fn boltbender() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Disguise(cost(&[generic(1), r()]))],
        triggered_abilities: vec![turned_up(Effect::ChooseNewTargetsForSpell {
            what: target_filtered(R::IsSpellOnStack),
        })],
        ..creature(
            "Boltbender",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Wizard],
            4,
            2,
        )
    }
}

/// Duskana, the Rage Mother — draws per base-2/2 creature on entering; a
/// base-2/2 attacker of yours gets +3/+3.
pub fn duskana_the_rage_mother() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Draw {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(yours().and(base_two_two()))),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: base_two_two() },
                ),
                effect: Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..legend(
            "Duskana, the Rage Mother",
            cost(&[generic(2), r(), g(), w()]),
            vec![CreatureType::Bear],
            5,
            5,
        )
    }
}

/// Hidden Dragonslayer — lifelink, megamorph; turning it up destroys an
/// opponent's creature with power 4 or greater.
pub fn hidden_dragonslayer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink, Keyword::Megamorph(cost(&[generic(2), w()]))],
        triggered_abilities: vec![turned_up(Effect::Destroy {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::PowerAtLeast(4))),
        })],
        ..creature(
            "Hidden Dragonslayer",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            1,
        )
    }
}

/// Hooded Hydra — enters with X counters; dying makes a Snake per counter;
/// morph, turned up with five counters.
pub fn hooded_hydra() -> CardDefinition {
    let snake = Arc::new(TokenDefinition {
        name: "Snake".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), g(), g()]))],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        turned_face_up_counters: Some((CounterType::PlusOnePlusOne, 5)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                definition: snake,
            },
        }],
        ..creature("Hooded Hydra", cost(&[x(), g(), g()]), vec![CreatureType::Snake, CreatureType::Hydra], 0, 0)
    }
}

/// Master of Pearls — morph; turning it up gives your creatures +2/+2.
pub fn master_of_pearls() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), w(), w()]))],
        triggered_abilities: vec![turned_up(Effect::PumpPT {
            what: Selector::EachPermanent(yours()),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Master of Pearls", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Monk], 2, 2)
    }
}

/// Mastery of the Unseen — a permanent of yours turning up gains life per
/// creature; {3}{W}: manifest.
pub fn mastery_of_the_unseen() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![yours_turned_up(Effect::GainLife {
            who: Selector::You,
            amount: Value::count(Selector::EachPermanent(yours())),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: Effect::Manifest { who: PlayerRef::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..enchantment("Mastery of the Unseen", cost(&[generic(1), w()]))
    }
}

/// Neheb, the Eternal — afflict 3; your second main phase adds {R} per life
/// your opponents lost this turn.
pub fn neheb_the_eternal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            afflict(3),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::ActivePlayer),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::Red, Value::TotalLifeLostThisTurn(PlayerRef::EachOpponent)),
                },
            },
        ],
        ..legend(
            "Neheb, the Eternal",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Zombie, CreatureType::Minotaur, CreatureType::Warrior],
            4,
            6,
        )
    }
}

/// Obscuring Aether — face-down creature spells cost {1} less; {1}{G}: turn
/// it face down.
pub fn obscuring_aether() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Face-down creature spells you cast cost {1} less to cast.",
            effect: StaticEffect::FaceDownSpellsCostLess { amount: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::TurnFaceDown { what: Selector::This },
            ..Default::default()
        }],
        ..enchantment("Obscuring Aether", cost(&[g()]))
    }
}

/// Panoptic Projektor — tap: the next face-down creature spell this turn
/// costs {3} less; turned-face-up triggers of yours fire twice.
pub fn panoptic_projektor() -> CardDefinition {
    CardDefinition {
        name: "Panoptic Projektor",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::NextFaceDownSpellCostsLessThisTurn { amount: 3 },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "If turning a face-down permanent face up causes a triggered ability of a permanent you \
                          control to trigger, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerTurnedFaceUpTriggers,
        }],
        ..Default::default()
    }
}

/// Printlifter Ooze — deathtouch, disguise; any creature of yours turning up
/// makes a trampling Ooze with a counter per other creature you control.
pub fn printlifter_ooze() -> CardDefinition {
    let ooze = Arc::new(TokenDefinition {
        name: "Ooze".into(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ooze], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Disguise(cost(&[generic(3), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            // X counts the other creatures before the token exists.
            effect: Effect::WithX {
                x: Value::count(Selector::EachPermanent(yours())),
                body: Box::new(Effect::Seq(vec![
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: ooze },
                    Effect::AddCounter {
                        what: Selector::LastCreatedToken,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::XFromCost,
                    },
                ])),
            },
        }],
        ..creature("Printlifter Ooze", cost(&[generic(1), g()]), vec![CreatureType::Ooze], 2, 2)
    }
}

/// Salt Road Ambushers — another permanent of yours turning up as a
/// creature gets two +1/+1 counters; megamorph.
pub fn salt_road_ambushers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Megamorph(cost(&[generic(3), g(), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..creature(
            "Salt Road Ambushers",
            cost(&[generic(3), g()]),
            vec![CreatureType::Dog, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Showstopping Surprise — turn a creature of yours face up, then it deals
/// its power to each other creature.
pub fn showstopping_surprise() -> CardDefinition {
    CardDefinition {
        name: "Showstopping Surprise",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::TurnFaceUpFree { what: target_filtered(yours()), if_cant: None },
            Effect::DealDamageEqualToPowerToEach {
                source: Selector::Target(0),
                targets: Selector::EachPermanent(R::Creature),
                each_opponent: false,
            },
        ]),
        ..Default::default()
    }
}

/// Tesak, Judith's Hellhound — unleash; your creatures with counters have
/// haste; attacking adds {R} per attacker.
///
/// Residual: other Dogs don't gain unleash.
pub fn tesak_judiths_hellhound() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unleash],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with counters on them have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours().and(R::WithAnyCounter)),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![on_attack(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColor(
                Color::Red,
                Value::count(Selector::EachPermanent(yours().and(R::IsAttacking))),
            ),
        })],
        ..legend(
            "Tesak, Judith's Hellhound",
            cost(&[generic(3), r()]),
            vec![CreatureType::Elemental, CreatureType::Dog],
            3,
            3,
        )
    }
}

/// True Identity — once a turn, it or another permanent of yours turning
/// up scries 1 and draws; disguise.
pub fn true_identity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Disguise(cost(&[w()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::YourControl).once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..enchantment("True Identity", cost(&[generic(1), w()]))
    }
}

/// Unexplained Absence — exile nonland permanents; each one's controller
/// cloaks the top card of their library.
///
/// Residual: it never takes one of your own permanents.
pub fn unexplained_absence() -> CardDefinition {
    CardDefinition {
        name: "Unexplained Absence",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        // One target per opponent (`ForEachOpponentTarget` caps and spreads
        // the slots).
        effect: Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Permanent.and(R::Not(Box::new(R::Land))).and(R::ControlledByOpponent),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Cloak {
                        who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                        amount: Value::ONE,
                        from_hand: false,
                    },
                    Effect::Exile { what: Selector::Target(0) },
                ])),
            }),
        },
        ..Default::default()
    }
}

/// Veiled Ascension — your face-down creatures get flying counters; your
/// upkeep may cloak.
///
/// Residual: an entering face-down creature gets its counter from a
/// trigger rather than entering with it.
pub fn veiled_ascension() -> CardDefinition {
    let flying_counter = |what: Selector| Effect::AddKeywordCounter { what, keyword: Keyword::Flying, amount: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![
            etb(flying_counter(Selector::EachPermanent(yours().and(R::FaceDown)))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::FaceDown) },
                ),
                effect: flying_counter(Selector::TriggerSource),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
                effect: Effect::MayDo {
                    description: "Cloak the top card of your library?".into(),
                    body: Box::new(Effect::Cloak { who: PlayerRef::You, amount: Value::ONE, from_hand: false }),
                },
            },
        ],
        ..enchantment("Veiled Ascension", cost(&[generic(3), w()]))
    }
}
