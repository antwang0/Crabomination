//! Assorted staples wave — a metalcraft artifact, sacrifice-outlet damage
//! dealers, an upkeep counter-grower, flashback pump, and tempo bodies. All
//! ride existing engine primitives (plus `Predicate::MetalcraftActive`). Tests
//! in `tests/recent68.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    StaticAbility, StaticEffect, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{dash, etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, Value};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, Color};

/// Chrome Steed — {4} 2/2 Artifact Creature — Horse. Metalcraft — gets +2/+2
/// while you control three or more artifacts.
pub fn chrome_steed() -> CardDefinition {
    CardDefinition {
        name: "Chrome Steed",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horse], ..Default::default() },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Metalcraft — +2/+2 while you control three or more artifacts.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::MetalcraftActive { who: PlayerRef::You },
                power: 2,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Vulshok Replica — {3} 3/1 Artifact Creature — Berserker. {1}{R}, Sacrifice
/// this: it deals 3 damage to any target.
pub fn vulshok_replica() -> CardDefinition {
    CardDefinition {
        name: "Vulshok Replica",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Berserker], ..Default::default() },
        power: 3,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Colossodon Yearling — {2}{G} 2/4 Beast (vanilla).
pub fn colossodon_yearling() -> CardDefinition {
    CardDefinition {
        name: "Colossodon Yearling",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 2,
        toughness: 4,
        ..Default::default()
    }
}

/// Bloodhall Ooze — {R} 1/1 Ooze. At each of your upkeeps, if you control a
/// black permanent you may add a +1/+1 counter; likewise for a green permanent.
pub fn bloodhall_ooze() -> CardDefinition {
    CardDefinition {
        name: "Bloodhall Ooze",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ooze], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            upkeep_may_grow("black", Color::Black),
            upkeep_may_grow("green", Color::Green),
        ],
        ..Default::default()
    }
}

fn upkeep_may_grow(color_name: &'static str, color: Color) -> TriggeredAbility {
    let cond = Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::HasColor(color).and(R::ControlledByYou)),
        n: Value::Const(1),
    };
    let description = match color_name {
        "green" => "Put a +1/+1 counter on Bloodhall Ooze (control a green permanent)?".to_string(),
        _ => "Put a +1/+1 counter on Bloodhall Ooze (control a black permanent)?".to_string(),
    };
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer)
            .with_filter(cond),
        effect: Effect::MayDo {
            description,
            body: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        },
    }
}

/// Sylvan Might — {1}{G} Instant. Target creature gets +2/+2 and gains trample
/// until end of turn. Flashback {2}{G}{G}.
pub fn sylvan_might() -> CardDefinition {
    CardDefinition {
        name: "Sylvan Might",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), g(), g()]))],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Nimble Innovator — {3}{U} 2/2 Vedalken Artificer. When it enters, draw a
/// card.
pub fn nimble_innovator() -> CardDefinition {
    CardDefinition {
        name: "Nimble Innovator",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Barrage Ogre — {3}{R}{R} 3/3 Ogre Warrior. {T}, Sacrifice an artifact: it
/// deals 2 damage to any target.
pub fn barrage_ogre() -> CardDefinition {
    CardDefinition {
        name: "Barrage Ogre",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Craw Giant — {3}{G}{G}{G}{G} 6/4 Giant. Trample, rampage 2.
pub fn craw_giant() -> CardDefinition {
    CardDefinition {
        name: "Craw Giant",
        cost: cost(&[generic(3), g(), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 6,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Rampage(2)],
        ..Default::default()
    }
}

/// Reckless Imp — {2}{B} 2/2 Imp. Flying; can't block. Dash {1}{B}.
pub fn reckless_imp() -> CardDefinition {
    CardDefinition {
        name: "Reckless Imp",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Imp], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::CantBlock],
        alternative_cost: Some(dash(cost(&[generic(1), b()]))),
        ..Default::default()
    }
}
