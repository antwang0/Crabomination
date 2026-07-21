//! Guildpact (GPT) fourth gap wave: the Ravnica-block singularity/meek
//! Leylines (one riding the new `AllNonlandPermanentsAreLegendary` static),
//! the Nephilim-adjacent legends, and a spread of simple spells/creatures.
//! Tests in `classic_sets/gpt`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, OpeningHandEffect, PlayerRef, Selector, StaticEffect};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// 1/1 white Pegasus token with flying.
fn pegasus_token() -> TokenDefinition {
    TokenDefinition {
        name: "Pegasus".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pegasus], ..Default::default() },
        ..Default::default()
    }
}

/// 1/1 green Saproling token.
fn saproling_token() -> TokenDefinition {
    TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    }
}

/// Storm Herd — {8}{W}{W} Sorcery. Create X 1/1 white Pegasus tokens with
/// flying, where X is your life total.
pub fn storm_herd() -> CardDefinition {
    CardDefinition {
        name: "Storm Herd",
        cost: cost(&[generic(8), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::LifeOf(PlayerRef::You),
            definition: pegasus_token(),
        },
        ..Default::default()
    }
}

/// Starved Rusalka — {G} 1/1 Spirit. {G}, Sacrifice a creature: You gain 1 life.
pub fn starved_rusalka() -> CardDefinition {
    CardDefinition {
        name: "Starved Rusalka",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stratozeppelid — {4}{U} 4/4 Beast with flying that can block only creatures
/// with flying.
pub fn stratozeppelid() -> CardDefinition {
    CardDefinition {
        name: "Stratozeppelid",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        ..Default::default()
    }
}

/// Schismotivate — {1}{U}{R} Instant. Target creature gets +4/+0 until end of
/// turn. Another target creature gets -4/-0 until end of turn.
pub fn schismotivate() -> CardDefinition {
    CardDefinition {
        name: "Schismotivate",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(4),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                power: Value::Const(-4),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// To Arms! — {1}{W} Instant. Untap all creatures you control. Draw a card.
pub fn to_arms() -> CardDefinition {
    CardDefinition {
        name: "To Arms!",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                up_to: None,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// 3/3 blue Weird token with defender and flying.
fn weird_token() -> TokenDefinition {
    TokenDefinition {
        name: "Weird".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        keywords: vec![Keyword::Defender, Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Weird], ..Default::default() },
        ..Default::default()
    }
}

/// Thunderheads — {2}{U} Instant with replicate {2}{U}. Create a 3/3 blue Weird
/// with defender and flying, exiled at the beginning of the next end step.
pub fn thunderheads() -> CardDefinition {
    CardDefinition {
        name: "Thunderheads",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Replicate(cost(&[generic(2), u()]))],
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: weird_token() },
            Effect::ExileLastCreatedTokensAtNextEndStep,
        ]),
        ..Default::default()
    }
}

/// Sky Swallower — {3}{U}{U} 8/8 Leviathan with flying. When it enters, target
/// opponent gains control of all other permanents you control.
pub fn sky_swallower() -> CardDefinition {
    CardDefinition {
        name: "Sky Swallower",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Leviathan], ..Default::default() },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GainControl {
            what: Selector::EachPermanent(
                R::Nonland.and(R::ControlledByYou).and(R::OtherThanSource),
            ),
            to: Some(PlayerRef::Target(0)),
            duration: Duration::Permanent,
        })],
        ..Default::default()
    }
}

/// Infiltrator's Magemark — {2}{U} Aura. Enchant creature. Your enchanted
/// creatures get +1/+1 and can't be blocked except by creatures with defender.
pub fn infiltrators_magemark() -> CardDefinition {
    CardDefinition {
        name: "Infiltrator's Magemark",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Creatures you control that are enchanted get +1/+1 and can't be blocked except by creatures with defender.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::IsEnchanted),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(R::HasKeyword(
                    Keyword::Defender,
                )))],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Teysa, Orzhov Scion — {1}{W}{B} legendary 2/3 Human Advisor. Sacrifice three
/// white creatures: Exile target creature. Whenever another black creature you
/// control dies, create a 1/1 white Spirit with flying.
pub fn teysa_orzhov_scion() -> CardDefinition {
    CardDefinition {
        name: "Teysa, Orzhov Scion",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature.and(R::HasColor(Color::White)), 3)),
            effect: Effect::Exile { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::Black),
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Spirit".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    keywords: vec![Keyword::Flying],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Spirit],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Tibor and Lumia — {2}{U}{R} legendary 3/3 Human Wizard. Whenever you cast a
/// blue spell, target creature gains flying until end of turn. Whenever you cast
/// a red spell, this deals 1 damage to each creature without flying.
pub fn tibor_and_lumia() -> CardDefinition {
    CardDefinition {
        name: "Tibor and Lumia",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasColor(Color::Blue),
                    },
                ),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasColor(Color::Red),
                    },
                ),
                effect: Effect::DealDamage {
                    to: Selector::EachPermanent(
                        R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                    ),
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Earth Surge — {3}{G} Enchantment. Each land gets +2/+2 as long as it's a
/// creature.
pub fn earth_surge() -> CardDefinition {
    let anthem = |opponents| StaticAbility {
        description: "Each land gets +2/+2 as long as it's a creature.",
        effect: StaticEffect::AnthemForFilter {
            filter: R::Land.and(R::Creature),
            power: 2,
            toughness: 2,
            keywords: vec![],
            opponents,
            only_your_turn: false,
            scale_by_counters_on_self: None,
        },
    };
    CardDefinition {
        name: "Earth Surge",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![anthem(false), anthem(true)],
        ..Default::default()
    }
}

/// Leyline of the Meek — {2}{W}{W} Enchantment. If in your opening hand, you may
/// begin with it in play. Creature tokens get +1/+1.
pub fn leyline_of_the_meek() -> CardDefinition {
    let anthem = |opponents| StaticAbility {
        description: "Creature tokens get +1/+1.",
        effect: StaticEffect::AnthemForFilter {
            filter: R::Creature.and(R::IsToken),
            power: 1,
            toughness: 1,
            keywords: vec![],
            opponents,
            only_your_turn: false,
            scale_by_counters_on_self: None,
        },
    };
    CardDefinition {
        name: "Leyline of the Meek",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![anthem(false), anthem(true)],
        opening_hand: Some(OpeningHandEffect::StartInPlay { tapped: false, extra: Effect::Noop }),
        ..Default::default()
    }
}

/// Leyline of Singularity — {2}{U}{U} Enchantment. If in your opening hand, you
/// may begin with it in play. All nonland permanents are legendary.
pub fn leyline_of_singularity() -> CardDefinition {
    CardDefinition {
        name: "Leyline of Singularity",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "All nonland permanents are legendary.",
            effect: StaticEffect::AllNonlandPermanentsAreLegendary,
        }],
        opening_hand: Some(OpeningHandEffect::StartInPlay { tapped: false, extra: Effect::Noop }),
        ..Default::default()
    }
}

/// Ulasht, the Hate Seed — {2}{R}{G} legendary 0/0 Hellion Hydra. Enters with a
/// +1/+1 counter for each other red creature you control and each other green
/// creature you control. {1}, Remove a +1/+1 counter: deal 1 damage to target
/// creature, or create a 1/1 green Saproling.
pub fn ulasht_the_hate_seed() -> CardDefinition {
    let red = Value::count(Selector::EachPermanent(
        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource).and(R::HasColor(Color::Red)),
    ));
    let green = Value::count(Selector::EachPermanent(
        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource).and(R::HasColor(Color::Green)),
    ));
    CardDefinition {
        name: "Ulasht, the Hate Seed",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hellion, CreatureType::Hydra],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Sum(vec![red, green]))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::ChooseMode(vec![
                Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: saproling_token(),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
