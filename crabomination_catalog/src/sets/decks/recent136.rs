//! A Wilds of Eldraine (WOE) wave: modal ETBs, Food/Rat aristocrats, Spectacle,
//! and Adventure/Role value. All ride existing primitives. Tests in
//! `crabomination/src/tests/recent136.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, ZoneRef,
};
use crate::game::effects::food_token;
use crate::mana::{Color, b, cost, g, generic, u};

use super::woe_roles::{royal_role, sorcerer_role};

// ── Green ─────────────────────────────────────────────────────────────────────

/// Royal Treatment — {G} Instant. Target creature you control gains hexproof
/// until end of turn; create a Royal Role token attached to it.
pub fn royal_treatment() -> CardDefinition {
    CardDefinition {
        name: "Royal Treatment",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::CreateTokenAttachedTo {
                target: Selector::Target(0),
                definition: royal_role(),
            },
        ]),
        ..Default::default()
    }
}

/// 1/1 black Rat token with "This token can't block."
fn rat_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        keywords: vec![Keyword::CantBlock],
        ..Default::default()
    }
}

/// Every creature you control (layer-agnostic team selector).
fn your_creatures() -> Selector {
    Selector::EachMatching {
        zone: ZoneRef::Battlefield,
        filter: R::Creature.and(R::ControlledByYou),
    }
}

// ── Blue ──────────────────────────────────────────────────────────────────────

/// Merfolk Coralsmith — {2}{U} 2/3 Merfolk. {1}: gets +1/-1 until end of turn.
/// When it dies, scry 2.
pub fn merfolk_coralsmith() -> CardDefinition {
    CardDefinition {
        name: "Merfolk Coralsmith",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Living Lectern — {1}{U} 0/4 Construct. {1}, Sacrifice this creature: Draw a
/// card. Create a Sorcerer Role token attached to another target creature you
/// control. Activate only as a sorcery.
pub fn living_lectern() -> CardDefinition {
    CardDefinition {
        name: "Living Lectern",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::CreateTokenAttachedTo {
                    target: target_filtered(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    definition: sorcerer_role(),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Black ──────────────────────────────────────────────────────────────────────

/// Stingblade Assassin — {3}{B} 3/1 Faerie Assassin with flash and flying. ETB,
/// destroy target creature an opponent controls that was dealt damage this turn.
pub fn stingblade_assassin() -> CardDefinition {
    CardDefinition {
        name: "Stingblade Assassin",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(
                R::Creature
                    .and(R::ControlledByOpponent)
                    .and(R::DealtDamageThisTurn),
            ),
        })],
        ..Default::default()
    }
}

/// Lord Skitter's Butcher — {2}{B} 2/3 Rat Peasant. ETB choose one: make a Rat;
/// or may-sacrifice another creature to scry 2 then draw; or your creatures gain
/// menace until end of turn.
pub fn lord_skitters_butcher() -> CardDefinition {
    CardDefinition {
        name: "Lord Skitter's Butcher",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Peasant],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: rat_token(),
            },
            Effect::MaySacrifice {
                description: "Sacrifice another creature to scry 2 and draw?".into(),
                filter: R::Creature.and(R::OtherThanSource),
                count: Value::ONE,
                then: Box::new(Effect::Seq(vec![
                    Effect::Scry {
                        who: PlayerRef::You,
                        amount: Value::Const(2),
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                ])),
                else_: None,
            },
            Effect::GrantKeyword {
                what: your_creatures(),
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

// ── Green / Artifact ────────────────────────────────────────────────────────────

/// Provisions Merchant — {2}{G}{G} 3/3 Beast Peasant. ETB create a Food. When it
/// attacks, you may sacrifice a Food; if you do, attacking creatures get +1/+1
/// and gain trample until end of turn.
pub fn provisions_merchant() -> CardDefinition {
    CardDefinition {
        name: "Provisions Merchant",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Peasant],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: food_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MaySacrifice {
                    description: "Sacrifice a Food to pump your attackers?".into(),
                    filter: R::HasArtifactSubtype(crate::card::ArtifactSubtype::Food),
                    count: Value::ONE,
                    then: Box::new(Effect::Seq(vec![
                        Effect::PumpPT {
                            what: Selector::EachMatching {
                                zone: ZoneRef::Battlefield,
                                filter: R::Creature.and(R::ControlledByYou).and(R::IsAttacking),
                            },
                            power: Value::ONE,
                            toughness: Value::ONE,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: Selector::EachMatching {
                                zone: ZoneRef::Battlefield,
                                filter: R::Creature.and(R::ControlledByYou).and(R::IsAttacking),
                            },
                            keyword: Keyword::Trample,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Scarecrow Guide — {2} 2/1 Scarecrow with reach. {1}: Add one mana of any
/// color. Activate only once each turn.
pub fn scarecrow_guide() -> CardDefinition {
    CardDefinition {
        name: "Scarecrow Guide",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Scarecrow],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            once_per_turn: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
