//! Ravnica batch 8: Golgari value (death payoffs, dredge, sacrifice) + a guild
//! spell. Tests in `recent_b/recent_298`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Predicate};
use crate::mana::{Color, b, cost, g, generic, w};

fn saproling_token() -> TokenDefinition {
    TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Saproling],
            ..Default::default()
        },
        ..Default::default()
    }
}

// ── Golgari ─────────────────────────────────────────────────────────────────

/// Golgari Germination — {1}{B}{G} Enchantment. Whenever a nontoken creature
/// you control dies, create a 1/1 green Saproling.
pub fn golgari_germination() -> CardDefinition {
    CardDefinition {
        name: "Golgari Germination",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::IsToken.negate()),
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: saproling_token(),
            },
        }],
        ..Default::default()
    }
}

/// Corpse Blockade — {2}{B} 1/4 Zombie with Defender. Sacrifice another
/// creature: This creature gains deathtouch until end of turn.
pub fn corpse_blockade() -> CardDefinition {
    CardDefinition {
        name: "Corpse Blockade",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vulturous Zombie — {4}{B}{G} 5/5 Zombie with Flying. Whenever another
/// creature dies, put a +1/+1 counter on this creature.
pub fn vulturous_zombie() -> CardDefinition {
    CardDefinition {
        name: "Vulturous Zombie",
        cost: cost(&[generic(4), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Grave-Shell Scarab — {2}{B}{G}{G} 4/4 Insect with Dredge 1. {1}, Sacrifice
/// this creature: Draw a card.
pub fn grave_shell_scarab() -> CardDefinition {
    CardDefinition {
        name: "Grave-Shell Scarab",
        cost: cost(&[generic(2), b(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Dredge(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vindictive Mob — {4}{B}{B} 5/5 Human Berserker. When it enters, sacrifice a
/// creature. Can't be blocked by Saprolings.
pub fn vindictive_mob() -> CardDefinition {
    CardDefinition {
        name: "Vindictive Mob",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::HasCreatureType(
            CreatureType::Saproling,
        )))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::Creature,
            },
        }],
        ..Default::default()
    }
}

// ── Simic ───────────────────────────────────────────────────────────────────

// ── Guild spells ────────────────────────────────────────────────────────────

/// Seed Spark — {3}{W} Instant. Destroy target artifact or enchantment. If {G}
/// was spent to cast this spell, create two 1/1 green Saprolings.
pub fn seed_spark() -> CardDefinition {
    CardDefinition {
        name: "Seed Spark",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Enchantment)),
            },
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast {
                    color: Color::Green,
                    at_least: 1,
                },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: saproling_token(),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}
