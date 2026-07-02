//! Primitive-driven batch: living weapon scaled off all graveyards (Bonehoard),
//! an {X}-exile-from-graveyard debuff (Necropolis Fiend), a different-controllers
//! bounce (Run Away Together), and land-type-gated combat (Sea Serpent). Tests
//! in `tests/recent80.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus,
    EquipScale, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, u, x, Color};

/// Bonehoard — {4} Artifact — Equipment. Living weapon. Equipped creature gets
/// +X/+X, where X is the number of creature cards in all graveyards. Equip {2}.
pub fn bonehoard() -> CardDefinition {
    let germ = TokenDefinition {
        name: "Phyrexian Germ".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Bonehoard",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Creature,
                per_power: 1,
                per_toughness: 1,
                count_self_counters: None,
                count_graveyard: None,
                count_all_graveyards: Some(R::Creature),
            }),
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: germ },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        ..Default::default()
    }
}

/// Necropolis Fiend — {7}{B}{B} 4/5 Demon. Delve, Flying. {X},{T}, Exile X
/// cards from your graveyard: Target creature gets -X/-X until end of turn.
pub fn necropolis_fiend() -> CardDefinition {
    CardDefinition {
        name: "Necropolis Fiend",
        cost: cost(&[generic(7), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Delve, Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            exile_other_filter: Some((R::Any, 0)),
            exile_other_x: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                toughness: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::XFromCost)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ghost Ship — {2}{U}{U} 2/4 Spirit. Flying. {U}{U}{U}: Regenerate this
/// creature.
pub fn ghost_ship() -> CardDefinition {
    CardDefinition {
        name: "Ghost Ship",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u(), u()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Serpent Assassin — {3}{B}{B} 2/2 Snake Assassin. When it enters, you may
/// destroy target nonblack creature.
pub fn serpent_assassin() -> CardDefinition {
    CardDefinition {
        name: "Serpent Assassin",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "destroy target nonblack creature".into(),
            body: Box::new(Effect::Destroy {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
            }),
        })],
        ..Default::default()
    }
}

/// Sea Monster — {4}{U}{U} 6/6 Serpent. Can't attack unless defending player
/// controls an Island.
pub fn sea_monster() -> CardDefinition {
    CardDefinition {
        name: "Sea Monster",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Serpent], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::CanAttackOnlyIfDefenderControls(Box::new(R::HasLandType(
            LandType::Island,
        )))],
        ..Default::default()
    }
}

/// Caustic Bronco — {1}{B} 2/2 Snake Horse Mount. Whenever it attacks, reveal
/// the top card and put it into your hand; you lose life equal to its mana
/// value if it isn't saddled, otherwise each opponent loses that much. Saddle 3.
pub fn caustic_bronco() -> CardDefinition {
    CardDefinition {
        name: "Caustic Bronco",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Horse, CreatureType::Mount],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Saddle(3)],
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::SourceSaddled,
            then: Box::new(Effect::RevealTopToHandLoseMv { who: PlayerRef::EachOpponent }),
            else_: Box::new(Effect::RevealTopToHandLoseMv { who: PlayerRef::You }),
        })],
        ..Default::default()
    }
}

/// Goblin Recruiter — {1}{R} 1/1 Goblin. When this creature enters, search your
/// library for any number of Goblin cards, reveal them, then shuffle and put
/// those cards on top in any order. ("Any number" is modeled as up to 10.)
pub fn goblin_recruiter() -> CardDefinition {
    CardDefinition {
        name: "Goblin Recruiter",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::HasCreatureType(CreatureType::Goblin),
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            count: Value::Const(10),
        })],
        ..Default::default()
    }
}

/// Sea Serpent — {5}{U} 5/5 Serpent. Can't attack unless defending player
/// controls an Island. When you control no Islands, sacrifice this creature.
/// (The "control no Islands" sacrifice is modeled as an upkeep check, matching
/// Dandân.)
pub fn sea_serpent() -> CardDefinition {
    CardDefinition {
        name: "Sea Serpent",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Serpent], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::CanAttackOnlyIfDefenderControls(Box::new(R::HasLandType(
            LandType::Island,
        )))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasLandType(LandType::Island).and(R::ControlledByYou),
                )))),
                then: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Graveyard }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}
