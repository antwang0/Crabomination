//! A twenty-second wave — three keyword mechanics. TLA Firebending
//! (`Keyword::Firebending(n)`, CR 702.189): an attack-triggered mana ability
//! adding N {R} that survives until end of combat. TMNT Sneak
//! (`shortcut::sneak`, CR 702.190): a declare-blockers alt cast that returns an
//! unblocked attacker. Bloodthirst (`shortcut::bloodthirst`, CR 702.54):
//! enters with N +1/+1 counters if an opponent took damage this turn. Tests in
//! `crabomination/src/tests/recent22.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    SelectionRequirement, Selector, SpellSubtype, Subtypes, Supertype, Value,
};
use crate::effect::shortcut::{bloodthirst, sneak};
use crate::effect::{Duration, Effect};
use crate::mana::{b, cost, generic, r, u};

/// Jeong Jeong, the Deserter — {2}{R} 2/3 legendary Human Rebel Ally with
/// firebending 1. Exhaust — {3}: put a +1/+1 counter on it; when you next cast a
/// Lesson spell this turn, copy it (you may choose new targets). (A non-Lesson
/// cast first consumes the one-shot harmlessly.)
pub fn jeong_jeong_the_deserter() -> CardDefinition {
    CardDefinition {
        name: "Jeong Jeong, the Deserter",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Ally],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Firebending(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::OnYourNextSpellCastThisTurn {
                    body: Box::new(Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson),
                        },
                        then: Box::new(Effect::CopySpellMayChooseTargets {
                            what: Selector::TriggerSource,
                            count: Value::ONE,
                        }),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ran and Shaw — {3}{R}{R} 4/4 legendary Dragon with flying and firebending 2.
/// (The cast-ETB "copy if 3+ Dragons/Lessons in your graveyard" rider is
/// dropped.)
pub fn ran_and_shaw() -> CardDefinition {
    CardDefinition {
        name: "Ran and Shaw",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Firebending(2)],
        ..Default::default()
    }
}

/// Sozin's Comet — {3}{R}{R} Sorcery. Each creature you control gains
/// firebending 5 until end of turn. (Foretell is dropped.)
pub fn sozins_comet() -> CardDefinition {
    CardDefinition {
        name: "Sozin's Comet",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GrantKeyword {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Firebending(5),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Donatello's Technique — {2}{U} Sorcery, Sneak {U}. Draw two cards.
pub fn donatellos_technique() -> CardDefinition {
    CardDefinition {
        name: "Donatello's Technique",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Sneak(cost(&[u()]))],
        alternative_cost: Some(sneak(cost(&[u()]))),
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Jennika's Technique — {2}{R} Instant, Sneak {R}. Deals 2 damage to each
/// creature.
pub fn jennikas_technique() -> CardDefinition {
    CardDefinition {
        name: "Jennika's Technique",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Sneak(cost(&[r()]))],
        alternative_cost: Some(sneak(cost(&[r()]))),
        effect: Effect::DealDamage {
            to: Selector::EachPermanent(SelectionRequirement::Creature),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Bloodrage Vampire — {2}{B} 3/1 Vampire with bloodthirst 1.
pub fn bloodrage_vampire() -> CardDefinition {
    CardDefinition {
        name: "Bloodrage Vampire",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Bloodthirst(1)],
        triggered_abilities: vec![bloodthirst(1)],
        ..Default::default()
    }
}

/// Furyborn Hellkite — {4}{R}{R}{R} 6/6 Dragon with flying and bloodthirst 6.
pub fn furyborn_hellkite() -> CardDefinition {
    CardDefinition {
        name: "Furyborn Hellkite",
        cost: cost(&[generic(4), r(), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Bloodthirst(6)],
        triggered_abilities: vec![bloodthirst(6)],
        ..Default::default()
    }
}
