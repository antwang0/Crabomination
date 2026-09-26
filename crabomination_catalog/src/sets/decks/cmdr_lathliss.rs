//! Commander: the cards the **Reign of Dragons** precon (FDC, Lathliss,
//! Dragon Queen) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_lathliss.rs`.
//!
//! Residuals (each also on its card):
//! - **Carnelian Orb of Dragonkind** — its mana gives any creature spell
//!   haste, not only a Dragon.
//! - **Goddric, Cloaked Reveler** — while celebrating it keeps its Human
//!   Noble types beside Dragon.
//! - **Leyline Tyrant** — the dying payment is any mana, not only {R}.
//! - **Thundermane Dragon** — a creature cast from the top doesn't gain
//!   haste; the top card isn't shown to you.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{myriad, on_dies, target_any, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Predicate};
use crate::mana::{Color, ManaCost, SpendRestriction, cost, generic, r, x};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn dragon() -> R {
    R::HasCreatureType(CreatureType::Dragon)
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn treasure() -> Arc<TokenDefinition> {
    Arc::new(crabomination_base::tokens::treasure_token())
}

/// Breath Weapon — 2 damage to each non-Dragon creature.
pub fn breath_weapon() -> CardDefinition {
    CardDefinition {
        name: "Breath Weapon",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(dragon())))),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Carnelian Orb of Dragonkind — {T}: add {R}; a Dragon creature spell it
/// pays for gains haste.
///
/// ⚠ Residual: any creature spell it pays for gains haste.
pub fn carnelian_orb_of_dragonkind() -> CardDefinition {
    CardDefinition {
        name: "Carnelian Orb of Dragonkind",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::OfColor(Color::Red, Value::ONE)),
                    SpendRestriction::CreatureHaste,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Goddric, Cloaked Reveler — haste; celebrating, it's a 4/4 flying Dragon
/// with "{R}: Dragons you control get +1/+0 until end of turn".
///
/// ⚠ Residual: it keeps its Human Noble types.
pub fn goddric_cloaked_reveler() -> CardDefinition {
    let celebrating = |inner: StaticEffect, description: &'static str| StaticAbility {
        description,
        effect: StaticEffect::WhileCondition {
            condition: Predicate::CelebrationActive { who: PlayerRef::You },
            inner: Box::new(inner),
        },
    };
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![
            celebrating(
                StaticEffect::SetBasePtForFilter { applies_to: Selector::This, power: 4, toughness: 4 },
                "Celebration — base power and toughness 4/4.",
            ),
            celebrating(
                StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Flying },
                "Celebration — flying.",
            ),
            // "Is a Dragon": he loses his other creature types (the reminder
            // text), then gains Dragon — same source, applied in this order.
            celebrating(
                StaticEffect::MatchingLoseAllCreatureTypes { applies_to: Selector::This },
                "Celebration — loses all other creature types.",
            ),
            celebrating(
                StaticEffect::AddCreatureTypeToMatching { applies_to: Selector::This, creature_type: CreatureType::Dragon },
                "Celebration — a Dragon.",
            ),
            celebrating(
                StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::This,
                    ability: ActivatedAbility {
                        mana_cost: cost(&[r()]),
                        effect: Effect::PumpPT {
                            what: yours(R::Creature.and(dragon())),
                            power: Value::ONE,
                            toughness: Value::ZERO,
                            duration: Duration::EndOfTurn,
                        },
                        ..Default::default()
                    },
                    condition: None,
                },
                "Celebration — {R}: Dragons you control get +1/+0 until end of turn.",
            ),
        ],
        ..legendary(creature(
            "Goddric, Cloaked Reveler",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Noble],
            3,
            3,
        ))
    }
}

/// Goldlust Triad — flying, myriad; combat damage to a player makes a
/// Treasure.
pub fn goldlust_triad() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            myriad(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: treasure() },
            },
        ],
        ..creature("Goldlust Triad", cost(&[generic(4), r()]), vec![CreatureType::Dragon], 4, 3)
    }
}

/// Hit the Mother Lode — discover 10; a tapped Treasure per point the
/// discovered card's mana value falls short of 10.
pub fn hit_the_mother_lode() -> CardDefinition {
    CardDefinition {
        name: "Hit the Mother Lode",
        cost: cost(&[generic(4), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discover { n: Value::Const(10), filter: None },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Diff(Box::new(Value::Const(10)), Box::new(Value::DiscoveredManaValue)),
                definition: treasure(),
            },
            Effect::Tap { what: Selector::LastCreatedTokens },
        ]),
        ..Default::default()
    }
}

/// Leyline Tyrant — flying; your red mana doesn't empty between steps;
/// dying, you may pay any amount to deal that much damage.
///
/// ⚠ Residual: the payment may be any mana, not only {R}.
pub fn leyline_tyrant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You don't lose unspent red mana as steps and phases end.",
            effect: StaticEffect::UnspentColorManaPersists(Color::Red),
        }],
        triggered_abilities: vec![on_dies(Effect::MayPayX {
            description: "Pay any amount of {R} to deal that much damage?".into(),
            body: Box::new(Effect::DealDamage { to: target_any(), amount: Value::XFromCost }),
        })],
        ..creature("Leyline Tyrant", cost(&[generic(2), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Minion of the Mighty — menace; pack tactics: attacking with 6+ total
/// power, you may put a Dragon from hand onto the battlefield tapped and
/// attacking.
pub fn minion_of_the_mighty() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::AttackedWithTotalPowerAtLeast { who: PlayerRef::You, at_least: 6 }),
            // "You may put" — the hand pick is already optional (min 0).
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.and(dragon()),
                count: Value::ONE,
                tapped: true,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: Some(Box::new(Effect::JoinCombatAttacking { what: Selector::LastMoved })),
            },
        }],
        ..creature("Minion of the Mighty", cost(&[r()]), vec![CreatureType::Kobold], 0, 1)
    }
}

/// Nogi, Draco-Zealot — Dragon spells cost {1} less; attacking with three
/// Dragons out, it becomes a 5/5 flying Dragon until end of turn.
pub fn nogi_draco_zealot() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Dragon spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: dragon(), amount: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::SelectorCountAtLeast {
                sel: yours(dragon()),
                n: Value::Const(3),
            }),
            effect: Effect::Seq(vec![
                Effect::AddCreatureTypes {
                    what: Selector::This,
                    creature_types: vec![CreatureType::Dragon],
                    duration: Duration::EndOfTurn,
                },
                Effect::SetBasePT {
                    what: Selector::This,
                    power: Value::Const(5),
                    toughness: Value::Const(5),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ]),
        }],
        ..legendary(creature(
            "Nogi, Draco-Zealot",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Kobold, CreatureType::Shaman],
            3,
            3,
        ))
    }
}

/// Orb of Dragonkind — {1}, {T}: two mana in any colors for Dragons;
/// {R}, {T}, sacrifice it: a Dragon from the top seven.
pub fn orb_of_dragonkind() -> CardDefinition {
    CardDefinition {
        name: "Orb of Dragonkind",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyColors(Value::Const(2))),
                        SpendRestriction::CreatureOfTypeOrItsAbility(CreatureType::Dragon),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::Const(7),
                    pick_filter: Some(dragon()),
                    take: Some(Value::ONE),
                    optional: true,
                    rest_bottom_random: true,
                    ..Default::default()
                })),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Parapet Thrasher — flying; your Dragons connecting with an opponent pick
/// a mode not chosen this turn: destroy an artifact of theirs, 4 damage to
/// each other opponent, or impulse the top card.
pub fn parapet_thrasher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_batch: true,
                ..EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        Predicate::EntityMatches { what: Selector::TriggerSource, filter: dragon() },
                        Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer },
                    ]),
                )
            },
            effect: Effect::ChooseUnchosenModeThisTurn {
                modes: vec![
                    Effect::Destroy { what: target_filtered(R::Artifact.and(R::ControlledByTriggerPlayer)) },
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponentExceptTriggerer),
                        amount: Value::Const(4),
                    },
                    Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        duration: crate::card::MayPlayDuration::EndOfThisTurn,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: true,
                        uncast_penalty: None,
                    },
                ],
            },
        }],
        ..creature("Parapet Thrasher", cost(&[generic(2), r(), r()]), vec![CreatureType::Dragon], 4, 3)
    }
}

/// Shivan Devastator — flying, haste; enters with X +1/+1 counters.
pub fn shivan_devastator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        ..creature(
            "Shivan Devastator",
            cost(&[x(), r()]),
            vec![CreatureType::Dragon, CreatureType::Hydra],
            0,
            0,
        )
    }
}

/// The Elder Dragon War — Saga, read ahead. I: 2 damage to each creature and
/// each opponent. II: discard any number, draw that many. III: a 4/4 flying
/// Dragon.
pub fn the_elder_dragon_war() -> CardDefinition {
    CardDefinition {
        name: "The Elder Dragon War",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        read_ahead: true,
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(2) },
                    Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
                ]),
            ),
            (
                2,
                Effect::Seq(vec![
                    Effect::DiscardAnyNumber { who: Selector::You, filter: R::Any, max: None },
                    Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisEffect },
                ]),
            ),
            (
                3,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(TokenDefinition {
                        name: "Dragon".into(),
                        power: 4,
                        toughness: 4,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Red],
                        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
                        keywords: vec![Keyword::Flying],
                        ..Default::default()
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Thundermane Dragon — flying; you may cast creature spells with power 4 or
/// greater from the top of your library.
///
/// ⚠ Residual: no haste for a creature cast that way; the top card isn't
/// shown to you.
pub fn thundermane_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You may cast creature spells with power 4 or greater from the top of your library.",
            effect: StaticEffect::PlayFromLibraryTop { filter: R::Creature.and(R::PowerAtLeast(4)) },
        }],
        ..creature("Thundermane Dragon", cost(&[generic(3), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}
