//! Commander: 2026-10-19 (`016Xh16H`) — Ayesha Tanaka's banding soldiers
//! (CR 702.22). Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
};
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, Selector, Value};
use crate::mana::{ManaCost, cost, generic, w};

fn soldier(name: &'static str, mana: ManaCost, keywords: Vec<Keyword>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: p,
        toughness: t,
        keywords,
        ..Default::default()
    }
}

fn gains_banding(what: Selector) -> Effect {
    Effect::GrantKeyword { what, keyword: Keyword::Banding, duration: Duration::EndOfTurn }
}

fn banded_with_this() -> Selector {
    Selector::BandedWith(Box::new(Selector::This))
}

/// Icatian Phalanx — {4}{W} 2/4 banding Human Soldier.
pub fn icatian_phalanx() -> CardDefinition {
    soldier("Icatian Phalanx", cost(&[generic(4), w()]), vec![Keyword::Banding], 2, 4)
}

/// Kjeldoran Escort — {2}{W}{W} 2/3 banding Human Soldier.
pub fn kjeldoran_escort() -> CardDefinition {
    soldier("Kjeldoran Escort", cost(&[generic(2), w(), w()]), vec![Keyword::Banding], 2, 3)
}

/// Kjeldoran Phalanx — {5}{W} 2/5 first strike, banding.
pub fn kjeldoran_phalanx() -> CardDefinition {
    soldier("Kjeldoran Phalanx", cost(&[generic(5), w()]), vec![Keyword::FirstStrike, Keyword::Banding], 2, 5)
}

/// Kjeldoran Skycaptain — {4}{W} 2/2 flying, first strike, banding.
pub fn kjeldoran_skycaptain() -> CardDefinition {
    soldier(
        "Kjeldoran Skycaptain",
        cost(&[generic(4), w()]),
        vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Banding],
        2,
        2,
    )
}

/// Icatian Skirmishers — {3}{W} 1/1 first strike, banding. Attacking, all
/// creatures banded with it gain first strike until end of turn.
pub fn icatian_skirmishers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: banded_with_this(),
            keyword: Keyword::FirstStrike,
            duration: Duration::EndOfTurn,
        })],
        ..soldier(
            "Icatian Skirmishers",
            cost(&[generic(3), w()]),
            vec![Keyword::FirstStrike, Keyword::Banding],
            1,
            1,
        )
    }
}

/// Wall of Shields — {3} 0/4 artifact creature Wall; defender, banding.
pub fn wall_of_shields() -> CardDefinition {
    CardDefinition {
        name: "Wall of Shields",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender, Keyword::Banding],
        ..Default::default()
    }
}

/// Urza's Engine — {5} 1/5 trample artifact creature Juggernaut. {3}: it
/// gains banding until end of turn. {3}: attacking creatures banded with it
/// gain trample until end of turn.
pub fn urzas_engine() -> CardDefinition {
    CardDefinition {
        name: "Urza's Engine",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Juggernaut], ..Default::default() },
        power: 1,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: gains_banding(Selector::This),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::GrantKeyword {
                    what: banded_with_this(),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Baton of Morale — {2} artifact. {2}: target creature gains banding until
/// end of turn.
pub fn baton_of_morale() -> CardDefinition {
    CardDefinition {
        name: "Baton of Morale",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: gains_banding(target_filtered(R::Creature)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cooperation — {2}{W} Aura; enchanted creature has banding.
pub fn cooperation() -> CardDefinition {
    CardDefinition {
        name: "Cooperation",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has banding.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                keyword: Keyword::Banding,
            },
        }],
        ..Default::default()
    }
}

/// Formation — {1}{W} instant. Target creature gains banding until end of
/// turn; draw a card at the beginning of the next turn's upkeep.
pub fn formation() -> CardDefinition {
    CardDefinition {
        name: "Formation",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            gains_banding(target_filtered(R::Creature)),
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextUpkeep,
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        ]),
        ..Default::default()
    }
}

/// General Jarkeld — {3}{W} 1/2 legendary Human Soldier. {T}: switch the
/// blockers of two target blocked attacking creatures, if each could be
/// blocked by all of the other's blockers; declare blockers step only.
pub fn general_jarkeld() -> CardDefinition {
    let blocked_attacker = || R::Creature.and(R::IsAttacking).and(R::IsBlocked);
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(crate::effect::Predicate::CurrentStepIs(crate::game::types::TurnStep::DeclareBlockers)),
            effect: Effect::SwitchBlockers {
                first: Selector::TargetFiltered { slot: 0, filter: blocked_attacker() },
                second: Selector::TargetFiltered { slot: 1, filter: blocked_attacker() },
            },
            ..Default::default()
        }],
        ..soldier("General Jarkeld", cost(&[generic(3), w()]), vec![], 1, 2)
    }
}
