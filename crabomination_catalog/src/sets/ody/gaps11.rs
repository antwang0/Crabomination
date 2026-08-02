//! Odyssey (ODY) gap-closing wave 11: the replacement-effect rares and the
//! counter traps that close the set. Tests in `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipScale, EquipBonus, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    TriggeredAbility, WardCost,
};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, StaticEffect, Value,
    shortcut::target_filtered,
};
use crate::mana::{ManaCost, b, cost, generic, r, w};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..enchantment(name, c)
    }
}

fn upkeep(scope: EventScope, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), scope),
        effect,
    }
}

// ── Damage replacements ─────────────────────────────────────────────────────

/// Delaying Shield — {3}{W}. Bank the damage now, pay for it at upkeep.
pub fn delaying_shield() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to you, put that many delay counters on this \
                          enchantment instead.",
            effect: StaticEffect::ReplaceDamageToYouWithCountersOnSource {
                kind: CounterType::Delay,
            },
        }],
        triggered_abilities: vec![upkeep(
            EventScope::YourControl,
            Effect::Seq(vec![
                Effect::Repeat {
                    count: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Delay,
                    },
                    body: Box::new(Effect::UnlessPlayerPays {
                        who: PlayerRef::You,
                        cost: WardCost::Mana(cost(&[generic(1), w()])),
                        then: Box::new(Effect::LoseLife {
                            who: Selector::You,
                            amount: Value::ONE,
                        }),
                        if_paid: None,
                    }),
                },
                Effect::RemoveAllCounters { what: Selector::This },
            ]),
        )],
        ..enchantment("Delaying Shield", cost(&[generic(3), w()]))
    }
}

/// Nefarious Lich — {B}{B}{B}{B}. Your graveyard is your life total.
pub fn nefarious_lich() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "If damage would be dealt to you, exile that many cards from your \
                              graveyard instead. If you can't, you lose the game.",
                effect: StaticEffect::ReplaceDamageToYouWithGraveyardExile,
            },
            StaticAbility {
                description: "If you would gain life, draw that many cards instead.",
                effect: StaticEffect::LifeGainBecomesDraw,
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::LoseGame { who: PlayerRef::You },
        }],
        ..enchantment("Nefarious Lich", cost(&[b(), b(), b(), b()]))
    }
}

// ── Counter traps ───────────────────────────────────────────────────────────

/// Mine Layer — {3}{R} 1/1 Dwarf that mines the lands it points at.
pub fn mine_layer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            tap_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Land),
                kind: CounterType::Mine,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Land.and(R::WithCounter(CounterType::Mine)),
                    },
                ),
                effect: Effect::Destroy { what: Selector::TriggerSource },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::RemoveCounter {
                    what: Selector::EachPermanent(R::Land.and(R::WithCounter(CounterType::Mine))),
                    kind: CounterType::Mine,
                    amount: Value::Const(100),
                },
            },
        ],
        ..creature("Mine Layer", cost(&[generic(3), r()]), vec![CreatureType::Dwarf], 1, 1)
    }
}

/// Traveling Plague — {3}{B}{B} Aura that keeps growing and reattaches
/// itself when its host dies.
pub fn traveling_plague() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            upkeep(
                EventScope::AnyPlayer,
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Plague,
                    amount: Value::ONE,
                },
            ),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
                effect: Effect::ReturnSelfAttachedToChoiceOf {
                    chooser: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                },
            },
        ],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: -1,
                per_toughness: -1,
                count_self_counters: Some(CounterType::Plague),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..aura("Traveling Plague", cost(&[generic(3), b(), b()]), R::Creature)
    }
}

/// Steam Vines — {1}{R}{R} Aura that blows up the land it sits on and hops
/// to the next one.
pub fn steam_vines() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::EnchantedBySource),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(
                        Selector::attached_to(Selector::This),
                    ))),
                    amount: Value::ONE,
                },
                Effect::Destroy { what: Selector::attached_to(Selector::This) },
                Effect::ReturnSelfAttachedToChoiceOf {
                    chooser: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                },
            ]),
        }],
        ..aura("Steam Vines", cost(&[generic(1), r(), r()]), R::Land)
    }
}
