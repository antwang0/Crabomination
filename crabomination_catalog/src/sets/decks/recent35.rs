//! Blink / tempo / tutor staples plus the Spike counter-engine and a pair of
//! Auras. Most ride existing primitives; Stonehorn Dignitary exercises the new
//! `Effect::SkipNextCombatPhase` (CR 506). Tests in `tests/recent35.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement, Selector,
    Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Predicate, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// Spike Weaver — {2}{G}{G} 0/0 Spike. Enters with three +1/+1 counters.
/// {2}, Remove a +1/+1 counter: put a +1/+1 counter on target creature.
/// {1}, Remove a +1/+1 counter: prevent all combat damage this turn.
pub fn spike_weaver() -> CardDefinition {
    CardDefinition {
        name: "Spike Weaver",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spike], ..Default::default() },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                effect: Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                effect: Effect::PreventAllCombatDamageThisTurn,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Glimmerpoint Stag — {2}{W}{W} 3/3 Elk with Vigilance. ETB: exile another
/// target permanent; return it at the next end step.
pub fn glimmerpoint_stag() -> CardDefinition {
    CardDefinition {
        name: "Glimmerpoint Stag",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elk], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileReturnNextEndStep {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::OtherThanSource),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Weathered Wayfarer — {W} 1/1 Human Nomad Cleric. {W}, {T}: search your
/// library for a land card and put it into your hand. Activate only if an
/// opponent controls more lands than you.
pub fn weathered_wayfarer() -> CardDefinition {
    CardDefinition {
        name: "Weathered Wayfarer",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w()]),
            condition: Some(Predicate::OpponentControlsMoreLandsThanYou),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Plea for Guidance — {5}{W} Sorcery. Search your library for up to two
/// enchantment cards and put them into your hand.
pub fn plea_for_guidance() -> CardDefinition {
    CardDefinition {
        name: "Plea for Guidance",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: SelectionRequirement::Enchantment,
            to: ZoneDest::Hand(PlayerRef::You),
            count: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Three Dreams — {4}{W} Sorcery. Search your library for up to three Aura
/// cards and put them into your hand. (The "with different names" rider is
/// dropped.)
pub fn three_dreams() -> CardDefinition {
    CardDefinition {
        name: "Three Dreams",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Aura),
            to: ZoneDest::Hand(PlayerRef::You),
            count: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Fleetfoot Dancer — {1}{R}{G}{W} 4/4 Elf Druid with trample, lifelink, haste.
pub fn fleetfoot_dancer() -> CardDefinition {
    CardDefinition {
        name: "Fleetfoot Dancer",
        cost: cost(&[generic(1), r(), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Lifelink, Keyword::Haste],
        ..Default::default()
    }
}

/// Stormscape Apprentice — {U} 1/1 Human Wizard. {W}, {T}: tap target creature.
/// {B}, {T}: target player loses 1 life. (1v1-faithful: each opponent.)
pub fn stormscape_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Stormscape Apprentice",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[w()]),
                effect: Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[b()]),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cavern Harpy — {U}{B} 2/1 Harpy Beast with flying. ETB: return a blue or
/// black creature you control to its owner's hand. Pay 1 life: return this to
/// its owner's hand.
pub fn cavern_harpy() -> CardDefinition {
    CardDefinition {
        name: "Cavern Harpy",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Harpy, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasColor(Color::Blue)
                            .or(SelectionRequirement::HasColor(Color::Black))),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stonecloaker — {2}{W} 3/2 Gargoyle with flash and flying. ETB: return a
/// creature you control to its owner's hand. ETB: exile target card from a
/// graveyard.
pub fn stonecloaker() -> CardDefinition {
    CardDefinition {
        name: "Stonecloaker",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gargoyle], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Exile {
                    what: target_filtered(SelectionRequirement::InGraveyard),
                },
            },
        ],
        ..Default::default()
    }
}

/// Stonehorn Dignitary — {3}{W} 1/4 Rhino Soldier. ETB: an opponent skips their
/// next combat phase (1v1-faithful: each opponent).
pub fn stonehorn_dignitary() -> CardDefinition {
    CardDefinition {
        name: "Stonehorn Dignitary",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rhino, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::SkipNextCombatPhase { who: PlayerRef::EachOpponent },
        }],
        ..Default::default()
    }
}

/// Narcolepsy — {1}{U} Aura. Enchant creature. At the beginning of each upkeep,
/// tap the enchanted creature (a no-op if already tapped).
pub fn narcolepsy() -> CardDefinition {
    CardDefinition {
        name: "Narcolepsy",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Tap { what: Selector::AttachedTo(Box::new(Selector::This)) },
        }],
        ..Default::default()
    }
}

/// Bile Blight — {B}{B} Instant. Target creature and all other creatures with
/// the same name get -3/-3 until end of turn.
pub fn bile_blight() -> CardDefinition {
    CardDefinition {
        name: "Bile Blight",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: Selector::SharingNameWith(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature,
            })),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}
