//! Ravnica batch 11: a death-tuck Wizard, a Hellbent Rat, a hand-emptying
//! counter, a group basic-type animator, and a delayed-token burn. Each ships
//! a new engine primitive. Tests in `recent_b/recent_301`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Selector, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Predicate};
use crate::mana::{b, cost, generic, r, u};

/// Sadistic Augermage — {2}{B} 3/1 Human Wizard. When it dies, each player puts
/// a card from their hand on top of their library.
pub fn sadistic_augermage() -> CardDefinition {
    CardDefinition {
        name: "Sadistic Augermage",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::EachPlayerPutsHandCardOnTop {
                who: Selector::Player(PlayerRef::EachPlayer),
            },
        }],
        ..Default::default()
    }
}

/// Gobhobbler Rats — {B}{R} 2/2 Rat. Hellbent — while you have no cards in hand
/// it gets +1/+0 and has "{B}: Regenerate this creature."
pub fn gobhobbler_rats() -> CardDefinition {
    CardDefinition {
        name: "Gobhobbler Rats",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        power: 2,
        toughness: 2,
        static_abilities: vec![
            StaticAbility {
                description: "Hellbent — gets +1/+0 while you have no cards in hand.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::HellbentActive { who: PlayerRef::You },
                    power: 1,
                    toughness: 0,
                    keywords: vec![],
                },
            },
            StaticAbility {
                description: "Hellbent — has \"{B}: Regenerate this creature\" while you have no cards in hand.",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::This,
                    ability: ActivatedAbility {
                        mana_cost: cost(&[b()]),
                        effect: Effect::Regenerate { what: Selector::This },
                        ..Default::default()
                    },
                    condition: Some(Predicate::HellbentActive { who: PlayerRef::You }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Perplex — {1}{U}{B} Instant. Counter target spell unless its controller
/// discards their hand. Transmute {1}{U}{B}.
pub fn perplex() -> CardDefinition {
    CardDefinition {
        name: "Perplex",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnless {
            what: target_filtered(R::IsSpellOnStack),
            cost: crate::card::WardCost::DiscardHand,
        },
        activated_abilities: vec![crate::effect::shortcut::transmute(cost(&[generic(1), u(), b()]), 3)],
        ..Default::default()
    }
}

/// Terraformer — {2}{U} 2/2 Human Wizard. {1}: Choose a basic land type. Each
/// land you control becomes that type until end of turn.
pub fn terraformer() -> CardDefinition {
    CardDefinition {
        name: "Terraformer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::LandsBecomeChosenBasicType {
                what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skeletonize — {4}{R} Instant. Deals 3 damage to target creature. When a
/// creature dealt damage this way dies this turn, create a 1/1 black Skeleton
/// with "{B}: Regenerate this token."
pub fn skeletonize() -> CardDefinition {
    CardDefinition {
        name: "Skeletonize",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        // Register the death-watch before the damage so it's live when the
        // 3 damage kills the creature within this same resolution.
        effect: Effect::Seq(vec![
            Effect::WhenTargetDiesThisTurn {
                slot: 0,
                filter: None,
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: skeleton_token(),
                }),
            },
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// 1/1 black Skeleton with "{B}: Regenerate this token."
fn skeleton_token() -> TokenDefinition {
    TokenDefinition {
        name: "Skeleton".into(),
        colors: vec![crate::mana::Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Skeleton], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}
