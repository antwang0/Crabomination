//! Guildpact gap batch: Bloodthirst bodies, evasion, land removal, and a
//! discard payoff on existing primitives. Tests in `recent_b/recent_305`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Selector, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{bloodthirst, target_filtered};
use crate::effect::{Duration, Effect};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, u, w};

/// Battering Wurm — {6}{G} 4/3 Wurm. Bloodthirst 1; creatures with power less
/// than this creature's can't block it.
pub fn battering_wurm() -> CardDefinition {
    CardDefinition {
        name: "Battering Wurm",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::CantBeBlockedByPowerLess],
        triggered_abilities: vec![bloodthirst(1)],
        ..Default::default()
    }
}

/// Caustic Rain — {2}{B}{B} Sorcery. Exile target land.
pub fn caustic_rain() -> CardDefinition {
    CardDefinition {
        name: "Caustic Rain",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Exile { what: target_filtered(R::Land) },
        ..Default::default()
    }
}

/// Daggerclaw Imp — {2}{B} 3/1 Imp with flying that can't block.
pub fn daggerclaw_imp() -> CardDefinition {
    CardDefinition {
        name: "Daggerclaw Imp",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Imp], ..Default::default() },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::CantBlock],
        ..Default::default()
    }
}

/// Dryad Sophisticate — {1}{G} 2/1 Dryad with nonbasic landwalk.
pub fn dryad_sophisticate() -> CardDefinition {
    CardDefinition {
        name: "Dryad Sophisticate",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dryad], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::LandwalkFiltered(Box::new(R::IsNonbasicLand))],
        ..Default::default()
    }
}

/// Harrier Griffin — {5}{W} 3/3 Griffin with flying. At the beginning of your
/// upkeep, tap target creature.
pub fn harrier_griffin() -> CardDefinition {
    CardDefinition {
        name: "Harrier Griffin",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Griffin], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
        }],
        ..Default::default()
    }
}

/// Gristleback — {2}{G} 2/2 Boar Beast. Bloodthirst 1; Sacrifice: gain life
/// equal to this creature's power.
pub fn gristleback() -> CardDefinition {
    CardDefinition {
        name: "Gristleback",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![bloodthirst(1)],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Frazzle — {3}{U} Instant. Counter target nonblue spell.
pub fn frazzle() -> CardDefinition {
    CardDefinition {
        name: "Frazzle",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: target_filtered(
                R::IsSpellOnStack.and(R::Not(Box::new(R::HasColor(crate::mana::Color::Blue)))),
            ),
        },
        ..Default::default()
    }
}

/// Abyssal Nocturnus — {1}{B}{B} 2/2 Horror. Whenever an opponent discards a
/// card, this creature gets +2/+2 and gains fear until end of turn.
pub fn abyssal_nocturnus() -> CardDefinition {
    CardDefinition {
        name: "Abyssal Nocturnus",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horror], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Fear,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}
