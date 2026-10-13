//! Commander: the 2026-10-13 rescan's gaps for Zuko, Firebending Master and
//! Keleth, Sunmane Familiar + Yoshimaru. Tests in
//! `tests/recent_b/cmdr_edhrec49.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LookPick, PlayerRef, VoteOption, ZoneDest};
use crate::mana::{cost, generic, r, w, Color, ManaCost};
use std::sync::Arc;

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn aura(name: &'static str, mana: ManaCost, host: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(host) },
        ..Default::default()
    }
}

fn token(name: &str, color: Color, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

// ── Zuko, Firebending Master ─────────────────────────────────────────────────

/// Zuko, Avatar Hunter — {3}{R}{R} legendary 4/5 Human Noble, reach. Each red
/// spell you cast makes a 2/2 red Soldier.
pub fn zuko_avatar_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasColor(Color::Red))),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(token("Soldier", Color::Red, vec![CreatureType::Soldier], 2, 2, vec![])),
            },
        }],
        ..legend("Zuko, Avatar Hunter", cost(&[generic(3), r(), r()]), vec![CreatureType::Human, CreatureType::Noble], 4, 5)
    }
}

/// Ghitu Firebreathing — {1}{R} Aura, flash. {R}: enchanted creature gets
/// +1/+0 this turn. {R}: return this Aura to its owner's hand.
pub fn ghitu_firebreathing() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::PumpPT {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                ..Default::default()
            },
        ],
        ..aura("Ghitu Firebreathing", cost(&[generic(1), r()]), R::Creature)
    }
}

/// Lost in Memories — {1}{R} Aura, flash, enchant creature you control. +1/+1
/// and "dealing combat damage to a player, an instant or sorcery card in your
/// graveyard gains flashback (its mana cost) this turn".
pub fn lost_in_memories() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::GrantFlashbackThisTurn {
                    what: target_filtered(
                        R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery))
                            .and(R::InYourGraveyard),
                    ),
                },
            }],
            ..Default::default()
        }),
        ..aura("Lost in Memories", cost(&[generic(1), r()]), R::Creature.and(R::ControlledByYou))
    }
}

// ── Keleth, Sunmane Familiar + Yoshimaru ─────────────────────────────────────

/// Keleth, Sunmane Familiar — {1}{W} legendary 1/1 Horse. A commander you
/// control attacking gets a +1/+1 counter (CR 903.3). Partner.
pub fn keleth_sunmane_familiar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsCommander }),
            effect: Effect::AddCounter { what: Selector::TriggerSource, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legend("Keleth, Sunmane Familiar", cost(&[generic(1), w()]), vec![CreatureType::Horse], 1, 1)
    }
}

/// Yoshimaru, Beloved Companion — {2}{W} legendary 2/2 Dog. +1/+1 counters on
/// your creatures come one extra. {6}: a +1/+1 counter on target legendary
/// creature.
pub fn yoshimaru_beloved_companion() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If one or more +1/+1 counters would be put on a creature you control, that many plus one \
                          +1/+1 counters are put on it instead.",
            effect: StaticEffect::ExtraPlusOneCounters,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend("Yoshimaru, Beloved Companion", cost(&[generic(2), w()]), vec![CreatureType::Dog], 2, 2)
    }
}

/// Elspeth Resplendent — {3}{W}{W} planeswalker, loyalty 5. +1: a +1/+1
/// counter and a flying, first strike, lifelink or vigilance counter on a
/// creature (the printed "up to one" takes one when there is one). −3: look at
/// the top seven, put a permanent card with mana value 3 or less onto the
/// battlefield with a shield counter, the rest to the bottom at random. −7:
/// five 3/3 white flying Angels.
pub fn elspeth_resplendent() -> CardDefinition {
    let kw = |label: &str, keyword: Keyword| {
        VoteOption::new(label, Effect::AddKeywordCounter { what: Selector::Target(0), keyword, amount: Value::ONE })
    };
    CardDefinition {
        name: "Elspeth Resplendent",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Elspeth], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(R::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::PlayerChoosesOne {
                        who: PlayerRef::You,
                        options: vec![
                            kw("Flying", Keyword::Flying),
                            kw("First strike", Keyword::FirstStrike),
                            kw("Lifelink", Keyword::Lifelink),
                            kw("Vigilance", Keyword::Vigilance),
                        ],
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::Const(7),
                    pick_filter: Some(R::Permanent.and(R::ManaValueAtMost(3))),
                    to_battlefield: true,
                    optional: true,
                    rest_bottom_random: true,
                    then_if_picked: Some(Box::new(Effect::AddCounter {
                        what: Selector::LastMoved,
                        kind: CounterType::Shield,
                        amount: Value::ONE,
                    })),
                    ..Default::default()
                })),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(5),
                    definition: Arc::new(token("Angel", Color::White, vec![CreatureType::Angel], 3, 3, vec![Keyword::Flying])),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
