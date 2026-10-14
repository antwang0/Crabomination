//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), fourteenth file. Tests in `tests/recent_b/cmdr_rank3k_n.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector};
use crate::mana::{Color, ManaCost, b, cost, generic, r};

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

fn land(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        ..Default::default()
    }
}

fn tap_for(pool: ManaPayload) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool,
        },
        ..Default::default()
    }
}

// ── Seifer, Balamb Rival ─────────────────────────────────────────────────────

/// Daily Bugle Building — land. {T}: add {C}. {1}, {T}: add one mana of any
/// color. Smear Campaign — {1}, {T}: target legendary creature gains menace
/// until end of turn; sorcery speed.
pub fn daily_bugle_building() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_for(ManaPayload::Colorless(Value::ONE)),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                ..tap_for(ManaPayload::AnyOneColor(Value::ONE))
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..land("Daily Bugle Building")
    }
}

/// Labyrinth Raptor — {B}{R} 2/2 menace Nightmare Dinosaur. Whenever a creature
/// you control with menace becomes blocked, defending player sacrifices a
/// creature of their choice that's blocking it. {B}{R}: creatures you control
/// with menace get +1/+0 until end of turn.
pub fn labyrinth_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasKeyword(Keyword::Menace),
                },
            ),
            effect: Effect::ChooseOneAmong {
                what: Selector::CreaturesBlockingTriggerSource,
                chooser: PlayerRef::DefendingPlayer,
                chosen: Box::new(Effect::SacrificeSelected {
                    what: Selector::SeparatedPile { chosen: true },
                }),
                other: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), r()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasKeyword(Keyword::Menace)),
                ),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Labyrinth Raptor",
            cost(&[b(), r()]),
            vec![CreatureType::Nightmare, CreatureType::Dinosaur],
            2,
            2,
        )
    }
}

// ── Spider-Man, New Champion ─────────────────────────────────────────────────

/// Fogwell's Gym — land. {T}: add {R}; it deals 1 damage to you. {2}{R}, {T},
/// discard a card: draw a card.
pub fn fogwells_gym() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Colors(vec![Color::Red]),
                    },
                    Effect::DealDamage {
                        to: Selector::You,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..land("Fogwell's Gym")
    }
}

/// Lady Spider, Maybelle Reilly — {1}{R} legendary 1/3 reach Spider Human
/// Hero. Whenever you discard a card, a +1/+1 counter on target Spider you
/// control.
pub fn lady_spider_maybelle_reilly() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasCreatureType(CreatureType::Spider)),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..legend(
            "Lady Spider, Maybelle Reilly",
            cost(&[generic(1), r()]),
            vec![
                CreatureType::Spider,
                CreatureType::Human,
                CreatureType::Hero,
            ],
            1,
            3,
        )
    }
}

// ── Halfdane ─────────────────────────────────────────────────────────────────

/// Cemetery Puca — {1}{U/B}{U/B} 1/2 Shapeshifter. Whenever a creature dies,
/// you may pay {1}; if you do, it becomes a copy of that creature, except it
/// has this ability.
pub fn cemetery_puca() -> CardDefinition {
    let ub = || crate::mana::hybrid(Color::Blue, Color::Black);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::MayPay {
                description: "Pay {1} to have Cemetery Puca become a copy of the creature?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::BecomeCopyOf {
                    what: Selector::This,
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![],
                    keep_own_triggered: true,
                    keep_own_activated: false,
                    keep_name: false,
                }),
                else_: None,
            },
        }],
        ..creature(
            "Cemetery Puca",
            cost(&[generic(1), ub(), ub()]),
            vec![CreatureType::Shapeshifter],
            1,
            2,
        )
    }
}
