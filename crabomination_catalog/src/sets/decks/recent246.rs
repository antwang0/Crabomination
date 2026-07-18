//! MKM (Murders at Karlov Manor) gap batch — suspect payoffs, graveyard-matters
//! enchantments, and Aura value. Tests in `tests/recent_b/recent246.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Subtypes,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, investigate, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// 2/2 white and blue Detective creature token.
fn detective_token() -> TokenDefinition {
    TokenDefinition {
        name: "Detective".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Detective], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// Rune-Brand Juggler — {B}{R} Creature — Human Shaman 2/2. ETB suspect up to one
/// target creature you control. {3}{B}{R}, Sacrifice a suspected creature: target
/// creature gets -5/-5 until end of turn.
pub fn rune_brand_juggler() -> CardDefinition {
    CardDefinition {
        name: "Rune-Brand Juggler",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Suspect {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b(), r()]),
            sac_other_filter: Some((R::Creature.and(R::IsSuspected), 1)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-5),
                toughness: Value::Const(-5),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Chalk Outline — {3}{G} Enchantment. Whenever one or more creature cards leave
/// your graveyard, create a 2/2 white and blue Detective token, then investigate.
pub fn chalk_outline() -> CardDefinition {
    CardDefinition {
        name: "Chalk Outline",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                .with_filter(crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: detective_token(),
                },
                investigate(1),
            ]),
        }],
        ..Default::default()
    }
}

/// Soul Enervation — {3}{B} Enchantment, flash. ETB target creature gets -4/-4
/// until end of turn. Whenever one or more creature cards leave your graveyard,
/// each opponent loses 1 life and you gain 1 life.
pub fn soul_enervation() -> CardDefinition {
    CardDefinition {
        name: "Soul Enervation",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-4),
                toughness: Value::Const(-4),
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                    .with_filter(crate::effect::Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    }),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Convenient Target — {R} Aura. Enchant creature. ETB suspect enchanted
/// creature. Enchanted creature gets +1/+1. {2}{R}: Return this from your
/// graveyard to your hand.
pub fn convenient_target() -> CardDefinition {
    CardDefinition {
        name: "Convenient Target",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Suspect {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            from_graveyard: true,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Curious Inquiry — {U} Aura. Enchant creature. Enchanted creature gets +1/+1
/// and has "Whenever this creature deals combat damage to a player, investigate."
pub fn curious_inquiry() -> CardDefinition {
    CardDefinition {
        name: "Curious Inquiry",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: investigate(1),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Due Diligence — {2}{W} Aura. Enchant creature. ETB target creature you control
/// gets +2/+2 and vigilance until end of turn. Enchanted creature gets +2/+2 and
/// has vigilance. (The ETB "other than enchanted creature" exclusion is
/// approximated by any creature you control.)
pub fn due_diligence() -> CardDefinition {
    CardDefinition {
        name: "Due Diligence",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Vigilance],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}
