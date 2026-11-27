//! Commander: one-short closers for the 2026-11-13 Star Trek commanders'
//! EDHREC average decks (scan of 2026-11-27): Picard / Janeway Endgame, Kirk
//! Boldly Going, Miles O'Brien, Enterprise NX-01, Excelsior, Defiant Mirror
//! Migrant, Elim Garak. Tests in `tests/recent_b/cmdr_01wr6s_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, Subtypes,
    Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{draw, etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect, Value};
use crate::mana::{b, cost, g, generic, u, w, x};

/// Tactical Officer — {1}{W} 2/1 Human Officer. Whenever you attack while
/// creatures you control have total power 8 or greater, a +1/+1 counter on
/// it; untap it.
pub fn tactical_officer() -> CardDefinition {
    CardDefinition {
        name: "Tactical Officer",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Officer], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(Value::TotalPowerControlled, Value::Const(8))),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::Untap { what: Selector::This, up_to: None },
            ]),
        }],
        ..Default::default()
    }
}

/// Kirk, Man of Action — {2}{G}{W}{U} legendary 3/3 Human Officer. Other
/// Officers, Artificers, Doctors, Pilots, Scientists and Spacecraft creatures
/// you control get +2/+2.
pub fn kirk_man_of_action() -> CardDefinition {
    let crew = [
        CreatureType::Officer,
        CreatureType::Artificer,
        CreatureType::Doctor,
        CreatureType::Pilot,
        CreatureType::Scientist,
    ]
    .into_iter()
    .fold(R::HasArtifactSubtype(ArtifactSubtype::Spacecraft), |f, t| f.or(R::HasCreatureType(t)));
    CardDefinition {
        name: "Kirk, Man of Action",
        cost: cost(&[generic(2), g(), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Officer], ..Default::default() },
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Other Officers, Artificers, Doctors, Pilots, Scientists, and Spacecraft creatures you control get +2/+2.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(crew).and(R::ControlledByYou).and(R::OtherThanSource),
                power: 2,
                toughness: 2,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Evasive Maneuvers — {W} instant. Target blocking creature you control gets
/// +2/+2 until end of turn. Draw a card.
pub fn evasive_maneuvers() -> CardDefinition {
    CardDefinition {
        name: "Evasive Maneuvers",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsBlocking).and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Collector's Case — {1}{U} artifact. ETB: tap up to one target creature
/// and put two stun counters on it. {3}{U}, {T}: tap target creature.
pub fn collectors_case() -> CardDefinition {
    CardDefinition {
        name: "Collector's Case",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::Const(2) },
            ])),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fulgent Distraction — {2}{W} instant. Choose two target creatures; tap
/// them, then unattach all Equipment from them.
pub fn fulgent_distraction() -> CardDefinition {
    let slot = |n: u8| Selector::TargetFiltered { slot: n, filter: R::Creature };
    let equipment_on = |n: u8| {
        Selector::EachPermanent(R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::AttachedToTargetSlot(n)))
    };
    CardDefinition {
        name: "Fulgent Distraction",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Tap { what: slot(0) },
            Effect::Tap { what: slot(1) },
            Effect::Unattach { what: equipment_on(0) },
            Effect::Unattach { what: equipment_on(1) },
        ]),
        ..Default::default()
    }
}

/// Karn's Sylex — {3} legendary artifact; enters tapped. Players can't pay
/// life to cast spells or to activate abilities that aren't mana abilities.
/// {X}, {T}, exile it: destroy each nonland permanent with mana value X or
/// less; sorcery speed.
pub fn karns_sylex() -> CardDefinition {
    CardDefinition {
        name: "Karn's Sylex",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            crate::sets::enters_tapped(),
            StaticAbility {
                description: "Players can't pay life to cast spells or to activate abilities that aren't mana abilities.",
                effect: StaticEffect::PlayersCantPayLifeForNonManaCosts,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Destroy {
                what: Selector::EachPermanent(R::Not(Box::new(R::Land)).and(R::ManaValueAtMostXFromCost)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// D'Ghor Bloodletter — {2}{B} 2/2 Klingon Assassin. ETB, choose one — all
/// other creatures get -1/-1 until end of turn; or each player sacrifices a
/// creature.
pub fn dghor_bloodletter() -> CardDefinition {
    CardDefinition {
        name: "D'Ghor Bloodletter",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Klingon, CreatureType::Assassin], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::Sacrifice { who: Selector::Player(PlayerRef::EachPlayer), count: Value::ONE, filter: R::Creature },
        ]))],
        ..Default::default()
    }
}

/// Midnight Charm — {B} instant. Choose one — 1 damage to target creature
/// and you gain 1 life; target creature gains first strike until end of
/// turn; or tap target creature.
pub fn midnight_charm() -> CardDefinition {
    CardDefinition {
        name: "Midnight Charm",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::Tap { what: target_filtered(R::Creature) },
        ]),
        ..Default::default()
    }
}
