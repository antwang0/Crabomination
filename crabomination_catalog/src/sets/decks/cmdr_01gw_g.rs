//! Commander: 2026-10-19 (`01GwrDKP`) closers, seventh batch (5-7-gap tier) —
//! War Machine, James Rhodes' tappers. Tests in `tests/recent_b/cmdr_01gw_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, Subtypes,
    Supertype,
};
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{Duration, Effect, Selector, Value};
use crate::mana::{cost, generic, hybrid, u, w, Color, ManaCost};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CardType>, subtypes: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: types,
        subtypes: Subtypes { creature_types: subtypes, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// War Machine, James Rhodes — {3}{W/U} 3/2 flying legendary artifact
/// creature Human Hero. Attacking, tap up to one target creature.
pub fn war_machine_james_rhodes() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Tap { what: target_filtered(R::Creature) }),
        })],
        ..creature(
            "War Machine, James Rhodes",
            cost(&[generic(3), hybrid(Color::White, Color::Blue)]),
            vec![CardType::Artifact, CardType::Creature],
            vec![CreatureType::Human, CreatureType::Hero],
            3,
            2,
        )
    }
}

/// Alaborn Cavalier — {2}{W}{W} 2/2 Human Knight. Attacking, you may tap
/// target creature.
pub fn alaborn_cavalier() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Tap target creature?".into(),
            body: Box::new(Effect::Tap { what: target_filtered(R::Creature) }),
        })],
        ..creature("Alaborn Cavalier", cost(&[generic(2), w(), w()]), vec![CardType::Creature], vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

/// Master of Diversion — {2}{W} 2/2 Human Scout. Attacking, tap target
/// creature defending player controls.
pub fn master_of_diversion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Tap {
            what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
        })],
        ..creature("Master of Diversion", cost(&[generic(2), w()]), vec![CardType::Creature], vec![CreatureType::Human, CreatureType::Scout], 2, 2)
    }
}

/// Undulating Witness — {4}{U} 3/5 flying Serpent. {2}: +1/-1 until end of
/// turn. Basic landcycling {2}.
pub fn undulating_witness() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand))),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Undulating Witness", cost(&[generic(4), u()]), vec![CardType::Creature], vec![CreatureType::Serpent], 3, 5)
    }
}

/// Righteous Fury — {4}{W}{W} sorcery. Destroy all tapped creatures; gain 2
/// life for each creature destroyed this way.
pub fn righteous_fury() -> CardDefinition {
    CardDefinition {
        name: "Righteous Fury",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::Tapped)) },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(Box::new(Value::PermanentsDestroyedThisResolution), Box::new(Value::Const(2))),
            },
        ]),
        ..Default::default()
    }
}
