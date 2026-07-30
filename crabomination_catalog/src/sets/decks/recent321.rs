//! Mirrodin (MRD) gap batch 6 — the last of the blue/artifact rares. Tests in
//! `recent_b/mrd`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, LandType, Predicate, Selector, SelectionRequirement as R, StaticAbility,
    Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect};
use crate::game::TurnStep;
use crate::mana::{cost, generic, u, ManaCost};

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

/// Extraplanar Lens — imprints a land; every land sharing its name pays double.
pub fn extraplanar_lens() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Exile a land you control with this?".into(),
                body: Box::new(Effect::ExileWithSource {
                    what: target_filtered(R::Land.and(R::ControlledByYou)),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::TappedForMana, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Land.and(R::SameNameAsExiledWithSource),
                    }),
                effect: Effect::AddMana {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    pool: ManaPayload::AnyTypeTriggerSourceProduces,
                },
            },
        ],
        ..artifact("Extraplanar Lens", cost(&[generic(3)]))
    }
}

/// Quicksilver Fountain — floods the board one land at a time.
pub fn quicksilver_fountain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::AddCounter {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Land
                            .and(R::Not(Box::new(R::HasLandType(LandType::Island)))),
                    },
                    kind: CounterType::Flood,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                    .with_filter(Predicate::Not(Box::new(Predicate::SelectorExists(
                        Selector::EachPermanent(
                            R::Land.and(R::Not(Box::new(R::HasLandType(LandType::Island)))),
                        ),
                    )))),
                effect: Effect::RemoveAllCounters {
                    what: Selector::EachPermanent(R::Land),
                },
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Each land with a flood counter on it is an Island.",
            effect: StaticEffect::LandTypeChanger {
                applies_to: Selector::EachPermanent(
                    R::Land.and(R::WithCounter(CounterType::Flood)),
                ),
                land_type: LandType::Island,
                replace: false,
            },
        }],
        ..artifact("Quicksilver Fountain", cost(&[generic(3)]))
    }
}

/// Timesifter — every upkeep, the top of the decks bids for an extra turn.
pub fn timesifter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::ExileTopGreatestManaValueTakesExtraTurn,
        }],
        ..artifact("Timesifter", cost(&[generic(5)]))
    }
}

/// Proteus Staff — bottoms a creature and digs its controller a new one.
pub fn proteus_staff() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::BottomThenRevealUntilCreature {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..artifact("Proteus Staff", cost(&[generic(3)]))
    }
}

/// Quicksilver Elemental — borrows every activated ability on the table. (The
/// "spend blue as any colour for its own abilities" rider is dropped: the
/// engine's colour-relaxation static is table-wide.)
pub fn quicksilver_elemental() -> CardDefinition {
    CardDefinition {
        name: "Quicksilver Elemental",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::GainAllActivatedAbilitiesOf {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mindslaver — {4}, {T}, sacrifice: take the wheel on a player's next turn.
pub fn mindslaver() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::ControlPlayerNextTurn { who: PlayerRef::Target(0) },
            ..Default::default()
        }],
        ..artifact("Mindslaver", cost(&[generic(6)]))
    }
}
