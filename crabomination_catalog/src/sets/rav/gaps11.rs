//! Ravnica (RAV) gap wave 11: a Convoke overrun, a control-swap ETB, the
//! Firemane recursion loop, and a cast-triggered animation. Reuses existing
//! primitives. Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{cost, generic, r, u, w};

/// Overwhelm — {5}{G}{G} Sorcery with Convoke. Creatures you control get +3/+3
/// until end of turn.
pub fn overwhelm() -> CardDefinition {
    CardDefinition {
        name: "Overwhelm",
        cost: cost(&[generic(5), crate::mana::g(), crate::mana::g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Convoke],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            body: Box::new(Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Spawnbroker — {2}{U} 1/1 Human Wizard. When it enters, you may exchange
/// control of target creature you control and target creature an opponent
/// controls (the printed power ≤ restriction on the opponent's creature is
/// approximated).
pub fn spawnbroker() -> CardDefinition {
    CardDefinition {
        name: "Spawnbroker",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exchange control of two creatures?".into(),
            body: Box::new(Effect::ExchangeControl {
                a: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                b: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            }),
        })],
        ..Default::default()
    }
}

/// Firemane Angel — {3}{R}{W}{W} 4/3 Angel with flying and first strike. At
/// the beginning of your upkeep, you may gain 1 life. {6}{R}{R}{W}{W}: Return
/// this card from your graveyard to the battlefield; activate only during your
/// upkeep. (The upkeep lifegain is modeled on the battlefield; its graveyard
/// side is a minor omission.)
pub fn firemane_angel() -> CardDefinition {
    CardDefinition {
        name: "Firemane Angel",
        cost: cost(&[generic(3), r(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Gain 1 life?".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), r(), r(), w(), w()]),
            from_graveyard: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::CurrentStepIs(TurnStep::Upkeep),
            ])),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Halcyon Glaze — {1}{U}{U} Enchantment. Whenever you cast a creature spell,
/// this enchantment becomes a 4/4 Illusion creature with flying in addition to
/// its other types until end of turn.
pub fn halcyon_glaze() -> CardDefinition {
    CardDefinition {
        name: "Halcyon Glaze",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(4),
                toughness: Value::Const(4),
                creature_types: vec![CreatureType::Illusion],
                keywords: vec![Keyword::Flying],
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Light of Sanction — {1}{W}{W} Enchantment. Prevent all damage that would be
/// dealt to creatures you control by sources you control.
pub fn light_of_sanction() -> CardDefinition {
    CardDefinition {
        name: "Light of Sanction",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage to your creatures from your sources.",
            effect: StaticEffect::PreventDamageToYourCreaturesFromYourSources,
        }],
        ..Default::default()
    }
}
