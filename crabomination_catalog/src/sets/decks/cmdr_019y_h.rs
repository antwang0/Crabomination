//! Commander: 2026-10-17 (`019Y6J4R`) closers, eighth batch. Tests in
//! `tests/recent_b/cmdr_019y_h.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, SelectionRequirement as R, StaticAbility, Subtypes, Supertype,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, u};

/// Timebender — {U} 1/1 Human Wizard, morph {U}. Turned face up, choose one:
/// remove two time counters from target permanent or suspended card; or put
/// two on target permanent with a time counter or suspended card.
pub fn timebender() -> CardDefinition {
    let suspended = || R::InExile.and(R::IsSuspended);
    CardDefinition {
        name: "Timebender",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Wizard], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Morph(cost(&[u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::RemoveCounter {
                    what: target_filtered(R::OnBattlefield.or(suspended())),
                    kind: CounterType::Time,
                    amount: Value::Const(2),
                },
                Effect::AddCounter {
                    what: target_filtered(R::OnBattlefield.and(R::WithCounter(CounterType::Time)).or(suspended())),
                    kind: CounterType::Time,
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Soldevi Excavations — land. If it would enter, sacrifice an untapped Island
/// instead (else it goes to the graveyard). {T}: {C}{U}. {1}, {T}: scry 1.
pub fn soldevi_excavations() -> CardDefinition {
    CardDefinition {
        name: "Soldevi Excavations",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "If this land would enter, sacrifice an untapped Island instead. If you do, put this land onto the battlefield. If you don't, put it into its owner's graveyard.",
            effect: StaticEffect::EntersOnlyIfPaid {
                cost: WardCost::SacrificeMatchingN(
                    Box::new(R::Land.and(R::HasLandType(LandType::Island)).and(R::Untapped).and(R::OtherThanSource)),
                    1,
                ),
                optional: false,
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Blue]) },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Glissa, Herald of Predation — {3}{B}{G} legendary 3/5 Phyrexian Zombie Elf.
/// At the beginning of combat on your turn, choose one: incubate 2 twice;
/// transform all Incubator tokens you control; or Phyrexians you control gain
/// first strike and deathtouch until end of turn.
pub fn glissa_herald_of_predation() -> CardDefinition {
    let phyrexians = || Selector::EachPermanent(
        R::Creature.and(R::HasCreatureType(CreatureType::Phyrexian)).and(R::ControlledByYou),
    );
    let incubate = || Effect::Incubate { who: PlayerRef::You, amount: Value::Const(2) };
    CardDefinition {
        name: "Glissa, Herald of Predation",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Elf],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::ChooseMode(vec![
                Effect::Seq(vec![incubate(), incubate()]),
                Effect::Transform {
                    what: Selector::EachPermanent(
                        R::IsToken.and(R::HasName("Incubator".into())).and(R::ControlledByYou),
                    ),
                },
                Effect::Seq(vec![
                    Effect::GrantKeyword { what: phyrexians(), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: phyrexians(), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
                ]),
            ]),
        }],
        ..Default::default()
    }
}
