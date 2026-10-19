//! Commander: 2026-10-19 (`016Xh16H`) — Starling, Aerial Ally's white tricks
//! and Falcon, Winged Wonder's blue Heroes. Tests in
//! `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value};
use crate::mana::{Color, ManaCost, cost, generic, u, w};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

// ── Starling, Aerial Ally ───────────────────────────────────────────────────

/// Doomskar Oracle — {2}{W} 3/2 Human Cleric. Your second spell each turn
/// gains you 2 life; foretell {W}.
pub fn doomskar_oracle() -> CardDefinition {
    CardDefinition {
        foretell_cost: Some(cost(&[w()])),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::Const(2) }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..creature("Doomskar Oracle", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Cleric], 3, 2)
    }
}

/// Sanctum Spirit — {3}{W} 3/2 lifelink Spirit. Discard a historic card:
/// indestructible until end of turn.
pub fn sanctum_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::historic(), 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Sanctum Spirit", cost(&[generic(3), w()]), vec![CreatureType::Spirit], 3, 2)
    }
}

/// Tazeem Raptor — {2}{W} 2/2 flying Bird. ETB you may return a land you
/// control to its owner's hand.
pub fn tazeem_raptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return a land you control to its owner's hand?".into(),
            body: Box::new(Effect::ReturnOneYouControl { filter: R::Land, keep_best: false }),
        })],
        ..creature("Tazeem Raptor", cost(&[generic(2), w()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Guiding Bolt — {2}{W} instant. Destroy target creature with power 4 or
/// greater; scry 2.
pub fn guiding_bolt() -> CardDefinition {
    spell(
        "Guiding Bolt",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::PowerAtLeast(4))) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
    )
}

/// Impeccable Timing — {1}{W} instant. 3 damage to target attacking or
/// blocking creature.
pub fn impeccable_timing() -> CardDefinition {
    spell(
        "Impeccable Timing",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            amount: Value::Const(3),
        },
    )
}

/// Leap of Faith — {2}{W} instant. Target creature gains flying and is
/// shielded from all damage this turn.
pub fn leap_of_faith() -> CardDefinition {
    spell(
        "Leap of Faith",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            Effect::PreventAllDamageThisTurn { target: Selector::Target(0), redirect_to: None },
        ]),
    )
}

/// Angelic Blessing — {2}{W} sorcery. Target creature gets +3/+3 and flying
/// until end of turn.
pub fn angelic_blessing() -> CardDefinition {
    spell(
        "Angelic Blessing",
        cost(&[generic(2), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
        ]),
    )
}

// ── Falcon, Winged Wonder ───────────────────────────────────────────────────

/// Falcon, Winged Wonder — {4}{U} 3/4 legendary flying Human Hero. ETB:
/// Redwing, a legendary 1/1 blue flying Bird Scout token that surveils 1
/// when it attacks.
pub fn falcon_winged_wonder() -> CardDefinition {
    let redwing = TokenDefinition {
        name: "Redwing".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird, CreatureType::Scout], ..Default::default() },
        triggered_abilities: vec![on_attack(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(redwing),
        })],
        ..creature("Falcon, Winged Wonder", cost(&[generic(4), u()]), vec![CreatureType::Human, CreatureType::Hero], 3, 4)
    }
}

/// Bold Biochemist — {1}{U} 1/3 Human Scientist. Power-up — {5}{U}: a +1/+1
/// counter and draw two (CR 702.193).
pub fn bold_biochemist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..creature("Bold Biochemist", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Scientist], 1, 3)
    }
}

/// S.H.I.E.L.D. Deployment Drone — {2}{U} 2/2 flying artifact Robot; ETB a
/// 1/1 white Soldier.
pub fn s_h_i_e_l_d_deployment_drone() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(soldier),
        })],
        ..creature("S.H.I.E.L.D. Deployment Drone", cost(&[generic(2), u()]), vec![CreatureType::Robot], 2, 2)
    }
}

/// Stitched Mangler — {2}{U} 2/3 Zombie Horror, enters tapped. ETB tap
/// target creature an opponent controls; it skips its next untap.
pub fn stitched_mangler() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::SkipNextUntap { what: Selector::Target(0) },
        ]))],
        ..creature("Stitched Mangler", cost(&[generic(2), u()]), vec![CreatureType::Zombie, CreatureType::Horror], 2, 3)
    }
}

/// TVA Bureaucrat — {1}{U} 1/3 Human Advisor. A noncreature spell you cast
/// gives it +1/+0 and makes it unblockable this turn.
pub fn tva_bureaucrat() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Creature).negate(),
            }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("TVA Bureaucrat", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Advisor], 1, 3)
    }
}

/// Dismissive Denial — {2}{U}{U} instant. Counter target spell. Basic
/// landcycling {2}.
pub fn dismissive_denial() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand)))],
        ..spell(
            "Dismissive Denial",
            cost(&[generic(2), u(), u()]),
            CardType::Instant,
            Effect::CounterSpell { what: Selector::Target(0) },
        )
    }
}
