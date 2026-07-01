//! Spellslinger / tempo: instant-sorcery payoffs, token makers, and a raid
//! finisher. Tests in `tests/recent59.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{cast_is_instant_or_sorcery, etb, target_filtered};
use crate::effect::PlayerRef;
use crate::mana::{cost, generic, r, u, w, Color};

/// Sky Terror — {R}{W} 2/2 Dinosaur with flying and menace.
pub fn sky_terror() -> CardDefinition {
    CardDefinition {
        name: "Sky Terror",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Menace],
        ..Default::default()
    }
}

/// Talrand's Invocation — {2}{U}{U} Sorcery. Create two 2/2 blue flying Drakes.
pub fn talrands_invocation() -> CardDefinition {
    let drake = TokenDefinition {
        name: "Drake".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Drake], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Talrand's Invocation",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: drake },
        ..Default::default()
    }
}

/// Ondu Cleric — {1}{W} 1/1 Kor Cleric Ally. When this or another Ally you
/// control enters, you may gain life equal to the number of Allies you control.
pub fn ondu_cleric() -> CardDefinition {
    CardDefinition {
        name: "Ondu Cleric",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Cleric, CreatureType::Ally],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            // "this or another Ally you control" → any Ally you control entering.
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource, filter: R::HasCreatureType(CreatureType::Ally),
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
                ))),
            },
        }],
        ..Default::default()
    }
}

/// Aven Eternal — {2}{U} 2/2 Zombie Bird Warrior with flying. ETB: amass
/// Zombies 1.
pub fn aven_eternal() -> CardDefinition {
    CardDefinition {
        name: "Aven Eternal",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Bird, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Amass {
            who: PlayerRef::You, count: Value::Const(1), extra_type: Some(CreatureType::Zombie),
        })],
        ..Default::default()
    }
}

/// Storm Fleet Arsonist — {4}{R} 4/4 Orc Pirate. Raid — ETB, if you attacked
/// this turn, target opponent sacrifices a permanent of their choice.
pub fn storm_fleet_arsonist() -> CardDefinition {
    CardDefinition {
        name: "Storm Fleet Arsonist",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Pirate], ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
            then: Box::new(Effect::Sacrifice {
                who: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: R::Permanent,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Metallurgic Summonings — {3}{U}{U} Enchantment. Whenever you cast an
/// instant/sorcery, create an X/X colorless Construct (X = that spell's mana
/// value). {3}{U}{U}, exile this (modeled as sacrifice): return all I/S from
/// your graveyard, if you control 6+ artifacts.
pub fn metallurgic_summonings() -> CardDefinition {
    let construct = TokenDefinition {
        name: "Construct".into(),
        power: 0,
        toughness: 0,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Metallurgic Summonings",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            // An X/X token = a 0/0 body plus X +1/+1 counters (X = the cast
            // spell's mana value, read off the trigger source on the stack).
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: construct },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u(), u()]),
            sac_cost: true,
            condition: Some(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                n: Value::Const(6),
            }),
            effect: Effect::ReturnGraveyardCardsToHand {
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                max: Value::Const(99),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
