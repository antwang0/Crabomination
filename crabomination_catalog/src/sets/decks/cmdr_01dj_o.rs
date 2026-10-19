//! Commander: 2026-10-19 (`01DJHkWk`) — She-Hulk Attorney-at-Law's prepared
//! clerics and Elesh Norn's incubators.
//! Tests in `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate, Selector, Value};
use crate::mana::{Color, ManaCost, cost, generic, hybrid, w, x};
use std::sync::Arc;

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

fn incubate(n: i32) -> Effect {
    Effect::Incubate {
        who: PlayerRef::You,
        amount: Value::Const(n),
    }
}

/// Seed Suture — {G/W} sorcery, the prepared spell of the two Seed Suture
/// creatures: a +1/+1 counter on target creature; gain 1 life.
fn seed_suture() -> Arc<CardDefinition> {
    Arc::new(CardDefinition {
        name: "Seed Suture",
        cost: cost(&[hybrid(Color::Green, Color::White)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    })
}

// ── She-Hulk, Attorney-at-Law ───────────────────────────────────────────────

/// She-Hulk, Attorney-at-Law — {2}{G/W} legendary 3/3 vigilance Gamma Advisor
/// Hero. Power-up — {6}{G/W}: a +1/+1 counter on her, then double the +1/+1
/// counters on each creature you control.
pub fn she_hulk_attorney_at_law() -> CardDefinition {
    let gw = || hybrid(Color::Green, Color::White);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), gw()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::DoubleCountersOnEach {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "She-Hulk, Attorney-at-Law",
            cost(&[generic(2), gw()]),
            vec![
                CreatureType::Gamma,
                CreatureType::Advisor,
                CreatureType::Hero,
            ],
            3,
            3,
        )
    }
}

/// Blossom-Blessed Angel // Seed Suture — {3}{W} 2/4 flying, vigilance Angel
/// Cleric that enters prepared.
pub fn blossom_blessed_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        prepare_spell: Some(seed_suture()),
        ..creature(
            "Blossom-Blessed Angel",
            cost(&[generic(3), w()]),
            vec![CreatureType::Angel, CreatureType::Cleric],
            2,
            4,
        )
    }
}

/// Emergency Phytomedic // Seed Suture — {G/W} 1/1 Dryad Cleric that enters
/// prepared.
pub fn emergency_phytomedic() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        prepare_spell: Some(seed_suture()),
        ..creature(
            "Emergency Phytomedic",
            cost(&[hybrid(Color::Green, Color::White)]),
            vec![CreatureType::Dryad, CreatureType::Cleric],
            1,
            1,
        )
    }
}

// ── Elesh Norn ──────────────────────────────────────────────────────────────

/// Norn's Inquisitor — {1}{W} 1/1 Phyrexian Knight. Enters: incubate 2.
/// Whenever a permanent you control transforms into a Phyrexian, a +1/+1
/// counter on it.
pub fn norns_inquisitor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(incubate(2)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Transformed, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Phyrexian),
                    },
                ),
                effect: Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature(
            "Norn's Inquisitor",
            cost(&[generic(1), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Knight],
            1,
            1,
        )
    }
}

/// Progenitor Exarch — {X}{X}{W} 1/2 Phyrexian Cat Cleric. Enters: incubate 3
/// X times. {T}: transform target Incubator token you control.
pub fn progenitor_exarch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Repeat {
            count: Value::XFromCost,
            body: Box::new(incubate(3)),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Transform {
                what: target_filtered(
                    R::IsToken
                        .and(R::ControlledByYou)
                        .and(R::HasName("Incubator".to_string())),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Progenitor Exarch",
            cost(&[x(), x(), w()]),
            vec![
                CreatureType::Phyrexian,
                CreatureType::Cat,
                CreatureType::Cleric,
            ],
            1,
            2,
        )
    }
}

/// Tiller of Flesh — {3}{W} 2/4 Phyrexian Knight. Whenever you cast a spell
/// that targets one or more permanents, incubate 2.
pub fn tiller_of_flesh() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellTargetsMatch(R::Permanent)),
            effect: incubate(2),
        }],
        ..creature(
            "Tiller of Flesh",
            cost(&[generic(3), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Knight],
            2,
            4,
        )
    }
}
