//! Kamigawa: Neon Dynasty batch 7 — Ninja value, Channel utility, and a couple
//! of gy-recursion spells. Rides existing primitives. Tests in `tests/recent101.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, mint_treasures, target_filtered};
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, u, w};

/// Coiling Stalker — {1}{G} 2/1 Snake Ninja. Ninjutsu {1}{G}. Combat damage: put
/// a +1/+1 counter on target creature you control that has no +1/+1 counter.
pub fn coiling_stalker() -> CardDefinition {
    CardDefinition {
        name: "Coiling Stalker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Ninja],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::WithCounter(CounterType::PlusOnePlusOne).negate()),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Sunblade Samurai — {4}{W} 4/4 enchantment creature Human Samurai, vigilance.
/// Channel — {2}, Discard this card: search for a basic Plains to hand; gain 2.
pub fn sunblade_samurai() -> CardDefinition {
    CardDefinition {
        name: "Sunblade Samurai",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Samurai],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand.and(R::HasLandType(LandType::Plains)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Moonsnare Specialist — {3}{U} 2/2 Human Ninja. Ninjutsu {2}{U}. ETB: return up
/// to one target creature to its owner's hand.
pub fn moonsnare_specialist() -> CardDefinition {
    CardDefinition {
        name: "Moonsnare Specialist",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ninja],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(2), u()]))],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..Default::default()
    }
}

/// Undercity Scrounger — {2}{B} 1/4 Artifact Human Rogue. {T}: create a Treasure,
/// but only if a creature died this turn.
pub fn undercity_scrounger() -> CardDefinition {
    CardDefinition {
        name: "Undercity Scrounger",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::CreaturesDiedThisTurnTotalAtLeast {
                at_least: Value::Const(1),
            }),
            effect: mint_treasures(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Season of Renewal — {2}{G} Instant. Choose one or both — return a creature
/// card / an enchantment card from your graveyard to your hand.
pub fn season_of_renewal() -> CardDefinition {
    CardDefinition {
        name: "Season of Renewal",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Move {
                    what: target_filtered(R::Enchantment.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ],
        },
        ..Default::default()
    }
}

/// Assassin's Ink — {2}{B}{B} Instant. Costs {1} less if you control an artifact
/// and {1} less if you control an enchantment. Destroy target creature or
/// planeswalker.
pub fn assassins_ink() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Assassin's Ink",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![
            StaticAbility {
                description: "Costs {1} less if you control an artifact.",
                effect: StaticEffect::SelfCostReducedIfControlEach {
                    filters: vec![R::Artifact.and(R::ControlledByYou)],
                    amount: 1,
                },
            },
            StaticAbility {
                description: "Costs {1} less if you control an enchantment.",
                effect: StaticEffect::SelfCostReducedIfControlEach {
                    filters: vec![R::Enchantment.and(R::ControlledByYou)],
                    amount: 1,
                },
            },
        ],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.or(R::Planeswalker)),
        },
        ..Default::default()
    }
}

/// Mnemonic Sphere — {1}{U} Artifact. {1}{U}, Sacrifice this: draw two. Channel —
/// {U}, Discard this card: draw a card.
pub fn mnemonic_sphere() -> CardDefinition {
    CardDefinition {
        name: "Mnemonic Sphere",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                sac_cost: true,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                from_hand: true,
                discard_self_cost: true,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Suit Up — {2}{U} Instant. Until end of turn, target creature or Vehicle
/// becomes a 4/5 artifact creature; draw a card.
pub fn suit_up() -> CardDefinition {
    CardDefinition {
        name: "Suit Up",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::BecomeCreature {
                what: target_filtered(
                    R::Creature.or(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Vehicle)),
                ),
                power: Value::Const(4),
                toughness: Value::Const(5),
                creature_types: vec![],
                keywords: vec![],
                duration: crate::effect::Duration::EndOfTurn,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Careful Consideration — {2}{U}{U} Instant. You draw four cards, then discard
/// three (two if cast in your main phase). (Printed "target player" is modeled as
/// you — the standard self-cast; main-phase discount honored via the step check.)
pub fn careful_consideration() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Careful Consideration",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(4),
            },
            Effect::If {
                cond: Predicate::Any(vec![
                    Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                    Predicate::CurrentStepIs(TurnStep::PostCombatMain),
                ]),
                then: Box::new(Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(2),
                    random: false,
                }),
                else_: Box::new(Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(3),
                    random: false,
                }),
            },
        ]),
        ..Default::default()
    }
}
