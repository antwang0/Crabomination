//! More Aetherdrift (DFT) gap cards on existing primitives: a max-speed value
//! artifact, attack/saddle payoffs, an ETB removal Vehicle, and an artifact
//! reanimator. Tests in `crabomination/src/tests/recent172.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{attacks_while_saddled, etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, u, w};

/// Starting Column — {3} Artifact. Start your engines! {T}: Add one mana of any
/// color. Max speed — {T}, Sacrifice this: draw two, then discard a card.
pub fn starting_column() -> CardDefinition {
    CardDefinition {
        name: "Starting Column",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::StartYourEngines],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                condition: Some(Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 }),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Haunted Hellride — {1}{U}{B} Artifact — Vehicle 3/3. Whenever you attack,
/// target creature you control gets +1/+0 and gains deathtouch until end of
/// turn; untap it. Crew 1.
pub fn haunted_hellride() -> CardDefinition {
    CardDefinition {
        name: "Haunted Hellride",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
                Effect::Untap { what: Selector::Target(0), up_to: None },
            ]),
        }],
        ..Default::default()
    }
}

/// Unswerving Sloth — {3}{W}{W} 5/5 Sloth Mount. Whenever it attacks while
/// saddled, it gains indestructible until end of turn; untap all creatures you
/// control. Saddle 4.
pub fn unswerving_sloth() -> CardDefinition {
    CardDefinition {
        name: "Unswerving Sloth",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sloth, CreatureType::Mount],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Saddle(4)],
        triggered_abilities: vec![attacks_while_saddled(Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                up_to: None,
            },
        ]))],
        ..Default::default()
    }
}

/// Thundering Broodwagon — {2}{B}{B}{G}{G} Artifact — Vehicle 6/5. Menace,
/// reach. ETB: destroy target nonland permanent an opponent controls with mana
/// value 4 or less. Crew 3. Cycling {2}.
pub fn thundering_broodwagon() -> CardDefinition {
    CardDefinition {
        name: "Thundering Broodwagon",
        cost: cost(&[generic(2), b(), b(), g(), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Menace, Keyword::Reach, Keyword::Crew(3), Keyword::Cycling(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(
                R::Nonland.and(R::ControlledByOpponent).and(R::ManaValueAtMost(4)),
            ),
        })],
        ..Default::default()
    }
}

/// Tune Up — {3}{W} Sorcery. Return target artifact card from your graveyard to
/// the battlefield. (If it's a Vehicle it becomes an artifact creature — that
/// rider is dropped.)
pub fn tune_up() -> CardDefinition {
    CardDefinition {
        name: "Tune Up",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}
