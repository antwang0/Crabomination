//! More Aetherdrift (DFT) gap cards, all on existing primitives: Vehicles with
//! Cycling / exhaust animate, Speed payoffs (max-speed triggers, speed-scaled
//! search), one-sided bite, target-conditional cost reductions, exile-until-
//! leaves. Tests in `crabomination/src/tests/recent169.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Skybox Ferry — {5} Artifact — Vehicle 4/4. Flying. Crew 2. Cycling {2}.
pub fn skybox_ferry() -> CardDefinition {
    CardDefinition {
        name: "Skybox Ferry",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Crew(2), Keyword::Cycling(cost(&[generic(2)]))],
        ..Default::default()
    }
}

/// Ripclaw Wrangler — {3}{B} Artifact — Vehicle 4/3. ETB: each opponent
/// discards a card. Crew 2.
pub fn ripclaw_wrangler() -> CardDefinition {
    CardDefinition {
        name: "Ripclaw Wrangler",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
            random: false,
        })],
        ..Default::default()
    }
}

/// Pothole Mole — {2}{G} 2/3 Mole. ETB: mill three, then you may return a land
/// card from your graveyard to your hand.
pub fn pothole_mole() -> CardDefinition {
    CardDefinition {
        name: "Pothole Mole",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mole], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::ReturnGraveyardCardsToHand { filter: R::Land, max: Value::ONE },
        ]))],
        ..Default::default()
    }
}

/// Roadside Blowout — {2}{U} Sorcery. Costs {2} less if it targets a permanent
/// with mana value 1. Return target creature or Vehicle an opponent controls to
/// its owner's hand. Draw a card.
pub fn roadside_blowout() -> CardDefinition {
    CardDefinition {
        name: "Roadside Blowout",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        self_cost_reduction_if_target: Some((R::ManaValueExactly(1), 2)),
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    R::Creature
                        .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                        .and(R::ControlledByOpponent),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Run Over — {1}{G} Instant. Costs {1} less if it targets a Mount or Vehicle
/// you control. Target creature you control deals damage equal to its power to
/// target creature an opponent controls.
pub fn run_over() -> CardDefinition {
    CardDefinition {
        name: "Run Over",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((
            R::HasCreatureType(CreatureType::Mount)
                .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                .and(R::ControlledByYou),
            1,
        )),
        effect: Effect::DealDamageEqualToPower {
            source: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
            target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
        },
        ..Default::default()
    }
}

/// Pride of the Road — {3}{W} 2/5 Zombie Cat Warrior. Vigilance. Start your
/// engines! Max speed — at the beginning of combat on your turn, target creature
/// or Vehicle you control gains double strike until end of turn.
pub fn pride_of_the_road() -> CardDefinition {
    CardDefinition {
        name: "Pride of the Road",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Vigilance, Keyword::StartYourEngines],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::BeginCombat), EventScope::YourControl)
                .with_filter(Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 }),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    R::Creature
                        .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                        .and(R::ControlledByYou),
                ),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Rangers' Refueler — {1}{U} Artifact — Vehicle 3/3. Whenever you activate an
/// exhaust ability, draw a card. Exhaust — {4}: becomes an artifact creature;
/// put a +1/+1 counter on it. Crew 2.
pub fn rangers_refueler() -> CardDefinition {
    CardDefinition {
        name: "Rangers' Refueler",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ExhaustAbilityActivated, EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCardTypeIndefinitely { what: Selector::This, card_type: CardType::Creature, until_eot: false },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rocketeer Boostbuggy — {R}{G} Artifact — Vehicle 3/2. Whenever it attacks,
/// create a Treasure. Exhaust — {3}: becomes an artifact creature; put a +1/+1
/// counter on it. Crew 1.
pub fn rocketeer_boostbuggy() -> CardDefinition {
    CardDefinition {
        name: "Rocketeer Boostbuggy",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: crate::effect::shortcut::mint_treasures(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCardTypeIndefinitely { what: Selector::This, card_type: CardType::Creature, until_eot: false },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Point the Way — {G} Enchantment. Start your engines! `{3}{G}, Sacrifice
/// this: Search your library for up to X basic land cards, where X is your
/// speed. Put them onto the battlefield tapped, then shuffle.`
pub fn point_the_way() -> CardDefinition {
    CardDefinition {
        name: "Point the Way",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::StartYourEngines],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            sac_cost: true,
            effect: Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::PlayerSpeed(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Perilous Snare — {2}{W} Artifact. Start your engines! ETB: exile target
/// nonland permanent an opponent controls until this leaves. Max speed — {T}:
/// put a +1/+1 counter on target creature or Vehicle you control (sorcery
/// speed).
pub fn perilous_snare() -> CardDefinition {
    CardDefinition {
        name: "Perilous Snare",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Nonland.and(R::ControlledByOpponent)),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            condition: Some(Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 }),
            effect: Effect::AddCounter {
                what: target_filtered(
                    R::Creature
                        .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                        .and(R::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
