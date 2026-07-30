//! A cross-set wave (OTJ / DSK / MKM): two Equipment (Treasure-maker and a
//! manifest-dread menace-less +2/+1), a token-payoff Vampire, and two "haven't
//! cast a spell this turn" end-step payoffs. Tests in
//! `crabomination/src/tests/recent153.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus, Keyword,
    Predicate, SelectionRequirement as R, Selector, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef};
use crate::game::effects::treasure_token;
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, generic, u, w};

/// Gold Pan — {2} Equipment. ETB makes a Treasure; equipped creature gets +1/+1.
/// Equip {1}.
pub fn gold_pan() -> CardDefinition {
    CardDefinition {
        name: "Gold Pan",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: treasure_token(),
        })],
        ..Default::default()
    }
}

/// Conductive Machete — {4} Equipment. ETB manifests dread and attaches to that
/// creature; equipped creature gets +2/+1. Equip {4}.
pub fn conductive_machete() -> CardDefinition {
    CardDefinition {
        name: "Conductive Machete",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 1,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ManifestDread {
                who: PlayerRef::You,
            },
            Effect::Attach {
                what: Selector::This,
                to: Selector::take(
                    Selector::EachPermanent(R::FaceDown.and(R::ControlledByYou)),
                    Value::ONE,
                ),
            },
        ]))],
        ..Default::default()
    }
}

/// Baron Bertram Graywater — {2}{W}{B} 3/4 Vampire Noble. Once each turn, when
/// tokens you control enter, make a 1/1 lifelink Vampire Rogue. {1}{B}, sacrifice
/// another creature or artifact: draw a card.
pub fn baron_bertram_graywater() -> CardDefinition {
    let vampire = TokenDefinition {
        name: "Vampire".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    };
    CardDefinition {
        name: "Baron Bertram Graywater",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_turn: true,
                filter: Some(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsToken,
                }),
                ..EventSpec::new(EventKind::TokenCreated, EventScope::YourControl)
            },
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: vampire,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Creature.or(R::Artifact), 1)),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Jem Lightfoote, Sky Explorer — {2}{W}{U} 3/3 with flying and vigilance. At the
/// beginning of your end step, if you haven't cast a spell this turn, draw a card.
pub fn jem_lightfoote_sky_explorer() -> CardDefinition {
    CardDefinition {
        name: "Jem Lightfoote, Sky Explorer",
        cost: cost(&[generic(2), w(), u()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::You,
                    count: Value::Const(0),
                },
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Canyon Crab — {1}{U} 0/5. {1}{U}: +2/-2 until end of turn. At your end step, if
/// you haven't cast a spell this turn, draw a card, then discard a card.
pub fn canyon_crab() -> CardDefinition {
    CardDefinition {
        name: "Canyon Crab",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Crab],
            ..Default::default()
        },
        power: 0,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::You,
                    count: Value::Const(0),
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}
