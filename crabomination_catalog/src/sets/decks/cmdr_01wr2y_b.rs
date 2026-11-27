//! Commander: closers for Martok, Hero of the Empire; Dr. Pulaski, Cynical
//! Clinician; and Sela, Tal Shiar Paradox's EDHREC average decks (scan of
//! 2026-11-27). Tests in `tests/recent_b/cmdr_01wr2y_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Value, ZoneDest};
use crate::mana::{b, cost, generic, w};

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    card_types: Vec<CardType>,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn basic_landcycling() -> Keyword {
    Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand)))
}

/// Type-2 Phaser — {2} Equipment, equip {3}. ETB: tap target creature an
/// opponent controls and put a stun counter on it. Equipped creature gets +3/+0.
pub fn type_2_phaser() -> CardDefinition {
    CardDefinition {
        name: "Type-2 Phaser",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 3, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
        ]))],
        ..Default::default()
    }
}

/// Hologram Janeway — {2}{W} legendary 3/3 artifact creature Illusion
/// Officer. Whenever you attack while creatures you control have total power 8
/// or greater, put a +1/+1 counter on target creature.
pub fn hologram_janeway() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(Value::TotalPowerControlled, Value::Const(8))),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Hologram Janeway",
            cost(&[generic(2), w()]),
            vec![CardType::Artifact, CardType::Creature],
            vec![CreatureType::Illusion, CreatureType::Officer],
            3,
            3,
        )
    }
}

/// 70,000 Light-Years from Home — {1}{W} sorcery. Exile target creature or
/// Spacecraft; its controller may search their library for a basic land card
/// and put it into their hand.
pub fn seventy_thousand_light_years_from_home() -> CardDefinition {
    CardDefinition {
        name: "70,000 Light-Years from Home",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Spacecraft))),
            },
            Effect::Search {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Disruptor Pistol — {2} artifact. {4}, {T}, sacrifice it: 5 damage to
/// target creature.
pub fn disruptor_pistol() -> CardDefinition {
    CardDefinition {
        name: "Disruptor Pistol",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(5) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Free Borg Revolutionaries — {3}{W}{W} 4/5 artifact creature Borg Rebel.
/// ETB: creatures you control get +1/+1 until end of turn. Basic landcycling {2}.
pub fn free_borg_revolutionaries() -> CardDefinition {
    CardDefinition {
        keywords: vec![basic_landcycling()],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Free Borg Revolutionaries",
            cost(&[generic(3), w(), w()]),
            vec![CardType::Artifact, CardType::Creature],
            vec![CreatureType::Borg, CreatureType::Rebel],
            4,
            5,
        )
    }
}

/// Shuttle Crew — {2}{W} 1/1 Human Officer, flying. ETB: 4 damage to target
/// tapped creature an opponent controls.
pub fn shuttle_crew() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::Tapped).and(R::ControlledByOpponent)),
            amount: Value::Const(4),
        })],
        ..creature(
            "Shuttle Crew",
            cost(&[generic(2), w()]),
            vec![CardType::Creature],
            vec![CreatureType::Human, CreatureType::Officer],
            1,
            1,
        )
    }
}

/// Rogue Artificial Intelligence — {4}{B}{B} 6/5 artifact creature Construct.
/// ETB: choose one — gain 3 life; or surveil 2. Basic landcycling {2}.
pub fn rogue_artificial_intelligence() -> CardDefinition {
    CardDefinition {
        keywords: vec![basic_landcycling()],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
        ]))],
        ..creature(
            "Rogue Artificial Intelligence",
            cost(&[generic(4), b(), b()]),
            vec![CardType::Artifact, CardType::Creature],
            vec![CreatureType::Construct],
            6,
            5,
        )
    }
}

/// Salt Vampire — {1}{B} 2/2 Vampire Shapeshifter, lifelink. ETB: look at
/// target opponent's hand; you may choose a nonland card from it — if you do,
/// that player exiles it, then draws a card.
pub fn salt_vampire() -> CardDefinition {
    let exiled = || Value::CountOf(Box::new(Selector::ExiledThisResolution { filter: R::Any }));
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::LookAtHand { who: target_filtered(R::OpponentPlayer) },
            Effect::MayDo {
                description: "Exile a nonland card from their hand (they draw a card)?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::ExileChosenFromHand {
                        from: Selector::Target(0),
                        count: Value::ONE,
                        filter: R::Nonland,
                        link_to_source: false,
                        face_down: false,
                    },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(exiled(), Value::ONE),
                        then: Box::new(Effect::Draw {
                            who: Selector::Player(PlayerRef::Target(0)),
                            amount: Value::ONE,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
        ]))],
        ..creature(
            "Salt Vampire",
            cost(&[generic(1), b()]),
            vec![CardType::Creature],
            vec![CreatureType::Vampire, CreatureType::Shapeshifter],
            2,
            2,
        )
    }
}

/// A Good Day to Die — {B} instant. Target creature gets +2/+0 until end of
/// turn; when that creature dies this turn, you draw a card.
pub fn a_good_day_to_die() -> CardDefinition {
    CardDefinition {
        name: "A Good Day to Die",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                slot: 0,
                filter: None,
            },
        ]),
        ..Default::default()
    }
}
