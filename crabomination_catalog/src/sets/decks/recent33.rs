//! More aristocrats / sacrifice-outlet staples. Sacrifice-as-cost activated
//! abilities fold the sacrifice as the effect's first step (the catalog
//! convention). Tracked in `DECK_FEATURES.md`; tests in `tests/recent33.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{PlayerRef, ZoneDest};
use crate::mana::{Color, b, cost, g, generic};

/// Sacrifice a creature you control (folded as an activated-cost first step).
fn sac_your_creature() -> Effect {
    Effect::Sacrifice {
        who: Selector::You,
        count: Value::Const(1),
        filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    }
}

/// Endless Cockroaches — {1}{B}{B} 1/1 Insect. When it dies, return it to its
/// owner's hand.
pub fn endless_cockroaches() -> CardDefinition {
    CardDefinition {
        name: "Endless Cockroaches",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..Default::default()
    }
}

/// Poison-Tip Archer — {2}{B}{G} 2/3 Elf Archer. Reach, deathtouch. Whenever
/// another creature dies, each opponent loses 1 life.
pub fn poison_tip_archer() -> CardDefinition {
    CardDefinition {
        name: "Poison-Tip Archer",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Archer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach, Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::OtherThanSource,
                },
            ),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Altar of Dementia — {2} Artifact. Sacrifice a creature: target player mills
/// cards equal to the sacrificed creature's power.
pub fn altar_of_dementia() -> CardDefinition {
    CardDefinition {
        name: "Altar of Dementia",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::Mill {
                    who: target_filtered(SelectionRequirement::Player),
                    amount: Value::SacrificedPower,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sadistic Hypnotist — {3}{B}{B} 2/2 Human Minion. Sacrifice a creature:
/// target player discards two cards. Activate only as a sorcery.
pub fn sadistic_hypnotist() -> CardDefinition {
    CardDefinition {
        name: "Sadistic Hypnotist",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Minion],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                sac_your_creature(),
                Effect::Discard {
                    who: target_filtered(SelectionRequirement::Player),
                    amount: Value::Const(2),
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sprout Swarm — {1}{G} Instant. Convoke. Buyback {3}. Create a 1/1 green
/// Saproling creature token.
pub fn sprout_swarm() -> CardDefinition {
    let saproling = TokenDefinition {
        name: "Saproling".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Saproling],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Sprout Swarm",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Convoke, Keyword::Buyback(cost(&[generic(3)]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: Box::new(saproling),
        },
        ..Default::default()
    }
}
