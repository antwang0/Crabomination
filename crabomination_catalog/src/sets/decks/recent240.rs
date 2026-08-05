//! DSK (Duskmourn) gap batch 2 — Nightmare removal, graveyard recursion, and
//! Survival payoffs. Tests in `tests/recent_b/recent240.rs`.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, CreatureType, ExileReturnZone, Keyword,
    MayPlayDuration, SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::etb;
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, StaticEffect, Value,
};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, w};

/// "Survival — At the beginning of your second main phase, if this creature is
/// tapped, [effect]."
fn survival(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::PostCombatMain),
            EventScope::ActivePlayer,
        ),
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::This,
                filter: R::Tapped,
            },
            then: Box::new(effect),
            else_: Box::new(Effect::Noop),
        },
    }
}

/// Fear of Abduction — {4}{W}{W} Enchantment Creature — Nightmare 5/5. Flying.
/// Additional cost: exile a creature you control. ETB: exile target creature an
/// opponent controls until this leaves; on leave, exiled cards go to hand.
pub fn fear_of_abduction() -> CardDefinition {
    CardDefinition {
        name: "Fear of Abduction",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        additional_cast_cost: vec![AdditionalCastCost::ExilePermanent {
            filter: R::Creature,
            count: 1,
        }],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
            },
            return_to: ExileReturnZone::Hand,
        })],
        ..Default::default()
    }
}

/// Say Its Name — {1}{G} Sorcery. Mill three, then you may return a creature or
/// land card from your graveyard to your hand. (The three-copy graveyard combo
/// that tutors Altanak is dropped — a niche recursion cost.)
pub fn say_its_name() -> CardDefinition {
    CardDefinition {
        name: "Say Its Name",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::ReturnGraveyardCardsToHand {
                filter: R::Creature.or(R::Land),
                max: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Coordinated Clobbering — {G} Sorcery. Tap one or two target untapped
/// creatures you control; each deals damage equal to its power to target
/// creature an opponent controls.
pub fn coordinated_clobbering() -> CardDefinition {
    let mine = R::Creature.and(R::ControlledByYou).and(R::Untapped);
    let theirs = R::Creature.and(R::ControlledByOpponent);
    CardDefinition {
        name: "Coordinated Clobbering",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        // Slots: 0 = your creature (req), 1 = opponent's creature (req),
        // 2 = your second creature (optional — "one or two").
        effect: Effect::OptionalTargets {
            min: 2,
            body: Box::new(Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: mine.clone(),
                    },
                },
                Effect::DealDamageEqualToPower {
                    source: Selector::Target(0),
                    target: Selector::TargetFiltered {
                        slot: 1,
                        filter: theirs,
                    },
                },
                Effect::Tap {
                    what: Selector::TargetFiltered {
                        slot: 2,
                        filter: mine,
                    },
                },
                Effect::DealDamageEqualToPower {
                    source: Selector::Target(2),
                    target: Selector::Target(1),
                },
            ])),
        },
        ..Default::default()
    }
}

/// Waltz of Rage — {3}{R}{R} Sorcery. Target creature you control deals damage
/// equal to its power to each other creature. Until end of turn, whenever a
/// creature you control dies, exile the top card of your library — you may play
/// it until the end of your next turn.
pub fn waltz_of_rage() -> CardDefinition {
    CardDefinition {
        name: "Waltz of Rage",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamageEqualToPowerToEach {
                source: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                targets: Selector::EachPermanent(R::Creature),
                each_opponent: false,
            },
            Effect::CreaturesYouControlDyingThisTurn {
                body: Box::new(Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Veteran Survivor — {W} Human Survivor 2/1. Survival — exile up to one target
/// card from a graveyard. While 3+ cards are exiled with it, +3/+3 and hexproof.
pub fn veteran_survivor() -> CardDefinition {
    CardDefinition {
        name: "Veteran Survivor",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Survivor],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![survival(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::InGraveyard,
            effect: Box::new(Effect::ExileWithSource {
                what: Selector::Target(0),
            }),
        })],
        static_abilities: vec![StaticAbility {
            description: "While 3+ cards exiled with this, it gets +3/+3 and has hexproof.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::ValueAtLeast(
                    Value::CardsExiledWithSourceCount,
                    Value::Const(3),
                ),
                applies_to: Selector::This,
                power: 3,
                toughness: 3,
                keywords: vec![Keyword::Hexproof],
            },
        }],
        ..Default::default()
    }
}
