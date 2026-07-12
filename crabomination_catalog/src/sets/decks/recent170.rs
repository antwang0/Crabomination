//! More Aetherdrift (DFT) gap cards on existing primitives: the "Roads" dual
//! land cycle (enters-tapped-unless-Mount/Vehicle + a sac-for-Pilot ability),
//! exhaust-payoff Vehicles, a max-speed cost reducer, and an artifact-anthem
//! Vehicle. Tests in `crabomination/src/tests/recent170.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::{colored, cost, generic, Color};

/// The 1/1 colorless Pilot the Roads lands mint — it saddles/crews as though
/// its power were 2 greater (`StaticEffect::CrewSaddlePowerBonus`).
fn roads_pilot_token() -> TokenDefinition {
    TokenDefinition {
        name: "Pilot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pilot], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This token saddles Mounts and crews Vehicles as though its power were 2 greater.",
            effect: StaticEffect::CrewSaddlePowerBonus { applies_to: Selector::This, amount: 2 },
        }],
        ..Default::default()
    }
}

/// The shared Roads land body: enters tapped unless you control a Mount or
/// Vehicle; `{T}: Add {color}`; `{1}{color}, {T}, Sacrifice: create a Pilot
/// (sorcery speed)`.
fn roads_land(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Mount)
                            .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                            .and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Tap { what: Selector::This }),
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(color, Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), colored(color)]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: roads_pilot_token(),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Foul Roads — Land. {T}: Add {B}. Enters-tapped-unless-Mount/Vehicle + Pilot sac.
pub fn foul_roads() -> CardDefinition {
    roads_land("Foul Roads", Color::Black)
}

/// Rocky Roads — Land. {T}: Add {R}.
pub fn rocky_roads() -> CardDefinition {
    roads_land("Rocky Roads", Color::Red)
}

/// Reef Roads — Land. {T}: Add {U}.
pub fn reef_roads() -> CardDefinition {
    roads_land("Reef Roads", Color::Blue)
}

/// Rangers' Aetherhive — {1}{G}{U} Artifact — Vehicle 3/5. Vigilance. Whenever
/// you activate an exhaust ability, create a 1/1 Thopter with flying. Crew 1.
pub fn rangers_aetherhive() -> CardDefinition {
    CardDefinition {
        name: "Rangers' Aetherhive",
        cost: cost(&[generic(1), crate::mana::g(), crate::mana::u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Vigilance, Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ExhaustAbilityActivated, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: thopter_token(),
            },
        }],
        ..Default::default()
    }
}

fn thopter_token() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        ..Default::default()
    }
}

/// Racers' Scoreboard — {4} Artifact. Start your engines! ETB: draw two, then
/// discard a card. Max speed — spells you cast cost {1} less.
pub fn racers_scoreboard() -> CardDefinition {
    CardDefinition {
        name: "Racers' Scoreboard",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Max speed — spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReductionWhile {
                filter: R::Any,
                amount: 1,
                condition: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
            },
        }],
        ..Default::default()
    }
}

/// Salvation Engine — {4}{W} Artifact — Vehicle 6/10. Other artifact creatures
/// you control get +2/+2. Whenever it attacks, return a target artifact card
/// from your graveyard to the battlefield. Crew 6.
pub fn salvation_engine() -> CardDefinition {
    CardDefinition {
        name: "Salvation Engine",
        cost: cost(&[generic(4), crate::mana::w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 6,
        toughness: 10,
        keywords: vec![Keyword::Crew(6)],
        static_abilities: vec![StaticAbility {
            description: "Other artifact creatures you control get +2/+2.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Artifact.and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                power: 2,
                toughness: 2,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..Default::default()
    }
}
