//! Commander: 2026-10-19 (`01DJHkWk`) — two-gap closers: T'Pol, Vulcan
//! Representative + Sand Silos; U.S.S. Enterprise-D + Cybernetic Specialist;
//! Arzakon + Tam, the Possibility.
//! Tests in `tests/recent_b/cmdr_01dj_b.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, MayPlayDuration,
    PlaneswalkerSubtype, SelectionRequirement as R, StationBand, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, station, target_filtered};
use crate::effect::{
    Effect, ManaPayload, PlayerRef, Predicate, Selector, StaticAbility, StaticEffect, Value,
    ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
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

// ── T'Pol, Vulcan Representative ────────────────────────────────────────────

/// Sand Silos — land; enters tapped; may stay tapped (CR 502.3). Upkeep, if
/// tapped: a storage counter. {T}, remove any number of storage counters:
/// {U} for each. (Hollow Trees' blue twin.)
pub fn sand_silos() -> CardDefinition {
    CardDefinition {
        name: "Sand Silos",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::MayChooseNotToUntap],
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::Tapped,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Storage,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_x: Some(CounterType::Storage),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Blue], Value::XFromCost),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// T'Pol, Vulcan Representative — {4}{U} legendary 4/3 Vulcan Scientist.
/// Enters: tap target creature an opponent controls and put two stun counters
/// on it.
pub fn tpol_vulcan_representative() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(2),
            },
        ]))],
        ..creature(
            "T'Pol, Vulcan Representative",
            cost(&[generic(4), u()]),
            vec![CreatureType::Vulcan, CreatureType::Scientist],
            4,
            3,
        )
    }
}

// ── U.S.S. Enterprise-D, Galaxy-Class ───────────────────────────────────────

/// Cybernetic Specialist — {2} 2/1 artifact creature Human Scientist. Enters:
/// surveil 1. {1}, {T}: put target card from your graveyard on the bottom of
/// your library.
pub fn cybernetic_specialist() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::InYourGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: crate::effect::LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..creature(
            "Cybernetic Specialist",
            cost(&[generic(2)]),
            vec![CreatureType::Human, CreatureType::Scientist],
            2,
            1,
        )
    }
}

/// U.S.S. Enterprise-D, Galaxy-Class — {3} legendary Spacecraft, 4/5 at 7+
/// with flying and vigilance. Station. Whenever one or more charge counters
/// are put on it for the first time each turn, exile the top card of your
/// library; you may play it this turn.
pub fn uss_enterprise_d_galaxy_class() -> CardDefinition {
    CardDefinition {
        name: "U.S.S. Enterprise-D, Galaxy-Class",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Spacecraft],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        activated_abilities: vec![station()],
        station: vec![StationBand {
            min: 7,
            keywords: vec![Keyword::Flying, Keyword::Vigilance],
            pt: Some((4, 5)),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::Charge), EventScope::SelfSource)
                .once_per_turn(),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..Default::default()
    }
}

// ── Arzakon ─────────────────────────────────────────────────────────────────

/// Arzakon — {W}{U}{B}{R}{G} legendary planeswalker, loyalty 4; can be your
/// commander. +2: 3 damage to any other target. −3: a Black Lotus token. −7:
/// each player shuffles hand and graveyard into their library, then draws
/// seven.
pub fn arzakon() -> CardDefinition {
    let lotus = TokenDefinition {
        name: "Black Lotus".into(),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(3)),
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Arzakon",
        cost: cost(&[w(), u(), b(), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Arzakon],
            ..Default::default()
        },
        base_loyalty: 4,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::DealDamage {
                    to: target_filtered(R::any_target().and(R::OtherThanSource)),
                    amount: Value::Const(3),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(lotus),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::Seq(vec![
                    Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
                    Effect::Draw {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::Const(7),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Tam, the Possibility — {1}{G}{U} legendary 2/4 Gorgon Wizard. Planeswalker
/// spells you cast cost {1} less. {W}{U}{B}{R}{G}, {T}: proliferate X times,
/// X = the planeswalker types among planeswalkers you control.
pub fn tam_the_possibility() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Planeswalker spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::Planeswalker,
                amount: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            tap_cost: true,
            effect: Effect::Repeat {
                count: Value::DistinctPlaneswalkerTypesAmong(Box::new(Selector::EachPermanent(
                    R::Planeswalker.and(R::ControlledByYou),
                ))),
                body: Box::new(Effect::Proliferate),
            },
            ..Default::default()
        }],
        ..creature(
            "Tam, the Possibility",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Gorgon, CreatureType::Wizard],
            2,
            4,
        )
    }
}
