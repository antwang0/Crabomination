//! **Freerunning** cards (CR 702.179, Assassin's Creed). A static ability on
//! the stack: "You may cast this spell for its freerunning cost if you dealt
//! combat damage to a player this turn with an Assassin or commander."
//! Modeled on the shared alternative-cost primitive — `AlternativeCost.mana_cost`
//! gated on `Predicate::DealtCombatDamageToPlayerThisTurn`. The "with an Assassin
//! or commander" sub-clause is approximated as "with any creature" (the engine
//! tracks the dealing creature's controller, not its type).

use crate::card::{
    AlternativeCost, CardDefinition, CardType, CreatureType, Effect, Keyword, Predicate,
    SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{PlayerRef, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

/// CR 702.179 — the freerunning alternative cost: pay `mana`, legal only if a
/// creature you control dealt combat damage to a player this turn.
fn freerunning(mana: ManaCost) -> AlternativeCost {
    AlternativeCost {
        mana_cost: mana,
        condition: Some(Predicate::DealtCombatDamageToPlayerThisTurn {
            who: PlayerRef::You,
        }),
        ..Default::default()
    }
}

/// Brotherhood Ambushers — {4}{B} 6/3 Human Assassin. Freerunning {3}{B}.
pub fn brotherhood_ambushers() -> CardDefinition {
    CardDefinition {
        name: "Brotherhood Ambushers",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 6,
        toughness: 3,
        alternative_cost: Some(freerunning(cost(&[generic(3), b()]))),
        ..Default::default()
    }
}

/// Merciless Harlequin — {2}{B} 2/1 Human Assassin. Freerunning {1}{B}. ETB:
/// draw a card and lose 1 life.
pub fn merciless_harlequin() -> CardDefinition {
    CardDefinition {
        name: "Merciless Harlequin",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        alternative_cost: Some(freerunning(cost(&[generic(1), b()]))),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Achilles Davenport — {2}{U}{B} 3/3 Legendary Human Assassin. Menace.
/// Freerunning {U}{B}. Other Assassins you control get +1/+1.
pub fn achilles_davenport() -> CardDefinition {
    CardDefinition {
        name: "Achilles Davenport",
        cost: cost(&[generic(2), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        alternative_cost: Some(freerunning(cost(&[u(), b()]))),
        static_abilities: vec![StaticAbility {
            description: "Other Assassins you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Assassin)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Eagle Vision — {4}{U} Sorcery. Freerunning {1}{U}. Draw three cards.
pub fn eagle_vision() -> CardDefinition {
    CardDefinition {
        name: "Eagle Vision",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Const(3),
        },
        alternative_cost: Some(freerunning(cost(&[generic(1), u()]))),
        ..Default::default()
    }
}

/// Distract the Guards — {1}{W}{W} Sorcery. Freerunning {1}{W}. Create three
/// 1/1 white Human Rogue creature tokens.
pub fn distract_the_guards() -> CardDefinition {
    CardDefinition {
        name: "Distract the Guards",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: Box::new(TokenDefinition {
                name: "Human Rogue".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Human, CreatureType::Rogue],
                    ..Default::default()
                },
                ..Default::default()
            }),
        },
        alternative_cost: Some(freerunning(cost(&[generic(1), w()]))),
        ..Default::default()
    }
}

/// Chain Assassination — {2}{B}{B} Instant. Freerunning {1}{B}. Destroy target
/// creature. If another creature died this turn, draw a card. ("Another" is
/// read before this spell's own kill registers — the draw fires when a death
/// already happened earlier this turn.)
pub fn chain_assassination() -> CardDefinition {
    CardDefinition {
        name: "Chain Assassination",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::CreaturesDiedThisTurnTotalAtLeast {
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
        ]),
        alternative_cost: Some(freerunning(cost(&[generic(1), b()]))),
        ..Default::default()
    }
}

/// Restart Sequence — {3}{B} Sorcery. Freerunning {1}{B}. Return target creature
/// card from your graveyard to the battlefield.
pub fn restart_sequence() -> CardDefinition {
    CardDefinition {
        name: "Restart Sequence",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
            },
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        alternative_cost: Some(freerunning(cost(&[generic(1), b()]))),
        ..Default::default()
    }
}

/// Escape Detection — {1}{U}{U} Instant. Return target creature to its owner's
/// hand, then draw a card. Its freerunning cost is "return a blue creature you
/// control" (no mana), still gated on the combat-damage condition.
pub fn escape_detection() -> CardDefinition {
    CardDefinition {
        name: "Escape Detection",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        alternative_cost: Some(AlternativeCost {
            awaken: false,
            return_to_hand: Some((
                SelectionRequirement::Creature.and(SelectionRequirement::HasColor(Color::Blue)),
                1,
            )),
            condition: Some(Predicate::DealtCombatDamageToPlayerThisTurn {
                who: PlayerRef::You,
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Overpowering Attack — {3}{R}{R} Sorcery. Freerunning {2}{R}. Untap all
/// creatures you control that attacked this turn; then take an additional
/// combat phase followed by an additional main phase.
pub fn overpowering_attack() -> CardDefinition {
    CardDefinition {
        name: "Overpowering Attack",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::AttackedThisTurn),
                ),
                up_to: None,
            },
            Effect::AdditionalCombatPhaseAfterMain {
                count: Value::Const(1),
            },
        ]),
        alternative_cost: Some(freerunning(cost(&[generic(2), r()]))),
        ..Default::default()
    }
}

/// Viewpoint Synchronization — {4}{G} Sorcery. Freerunning {2}{G}. Search your
/// library for up to three basic land cards; put two onto the battlefield
/// tapped and one into your hand, then shuffle. (Approximated as a single
/// `SearchUpToN` of three basics into your hand.)
pub fn viewpoint_synchronization() -> CardDefinition {
    CardDefinition {
        name: "Viewpoint Synchronization",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            count: Value::Const(3),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        alternative_cost: Some(freerunning(cost(&[generic(2), g()]))),
        ..Default::default()
    }
}
