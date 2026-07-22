//! Return to Ravnica (RTR) gap wave 11: an upkeep control-swap enchantment, a
//! graveyard-recursion Golgari legend, a coin-flip artifact bomb, and an
//! X-charge-counter mana enchantment. Tests in `classic_sets/rtr`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::card::DynamicPt;
use crate::effect::{Duration, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, ManaCost};

/// Conjured Currency — {5}{U} Enchantment. At the beginning of your upkeep, you
/// may exchange control of this enchantment and target permanent you neither
/// own nor control.
pub fn conjured_currency() -> CardDefinition {
    CardDefinition {
        name: "Conjured Currency",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Exchange control of Conjured Currency and target permanent?".into(),
                body: Box::new(Effect::ExchangeControl {
                    a: Selector::This,
                    b: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Permanent
                            .and(R::ControlledByYou.negate())
                            .and(R::OwnedByYou.negate()),
                    },
                }),
            },
        }],
        ..Default::default()
    }
}

/// Jarad, Golgari Lich Lord — {B}{B}{G}{G} 2/2 Zombie Elf. +1/+1 for each
/// creature card in your graveyard; {1}{B}{G}, Sacrifice another creature: each
/// opponent loses life equal to its power; Sacrifice a Swamp and a Forest:
/// return Jarad from your graveyard to your hand.
pub fn jarad_golgari_lich_lord() -> CardDefinition {
    CardDefinition {
        name: "Jarad, Golgari Lich Lord",
        cost: cost(&[b(), b(), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Elf],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        dynamic_pt: Some(DynamicPt::BasePlusCreaturesInControllerGraveyard { base: 2 }),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b(), g()]),
                sac_other_filter: Some((R::Creature.and(R::OtherThanSource), 1)),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::SacrificedPower,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: ManaCost::default(),
                from_graveyard: true,
                condition: Some(Predicate::All(vec![
                    Predicate::ValueAtLeast(land_count(LandType::Swamp), Value::ONE),
                    Predicate::ValueAtLeast(land_count(LandType::Forest), Value::ONE),
                ])),
                effect: Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::ONE,
                        filter: R::HasLandType(LandType::Swamp),
                    },
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::ONE,
                        filter: R::HasLandType(LandType::Forest),
                    },
                    Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn land_count(ty: LandType) -> Value {
    Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::ControlledByYou)),
        filter: R::HasLandType(ty),
    }
}

/// Volatile Rig — {4} 4/4 Construct with trample that attacks each combat if
/// able. When it's dealt damage, flip a coin; lose the flip → sacrifice it.
/// When it dies, flip a coin; lose the flip → 4 damage to each creature and
/// each player.
pub fn volatile_rig() -> CardDefinition {
    CardDefinition {
        name: "Volatile Rig",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::MustAttack],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: Effect::FlipCoin {
                    count: Value::ONE,
                    on_heads: Box::new(Effect::Noop),
                    on_tails: Box::new(Effect::SacrificePermanent { what: Selector::This }),
                },
            },
            on_dies(Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Noop),
                on_tails: Box::new(Effect::Seq(vec![
                    Effect::ForEach {
                        selector: Selector::EachPermanent(R::Creature),
                        body: Box::new(Effect::DealDamage {
                            to: Selector::TriggerSource,
                            amount: Value::Const(4),
                        }),
                    },
                    Effect::ForEach {
                        selector: Selector::Player(PlayerRef::EachPlayer),
                        body: Box::new(Effect::DealDamage {
                            to: Selector::TriggerSource,
                            amount: Value::Const(4),
                        }),
                    },
                ])),
            }),
        ],
        ..Default::default()
    }
}

/// Izzet Staticaster — {1}{U}{R} 0/3 Human Wizard with flash and haste.
/// {T}: This creature deals 1 damage to target creature and each other creature
/// with the same name as that creature.
pub fn izzet_staticaster() -> CardDefinition {
    CardDefinition {
        name: "Izzet Staticaster",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::SameNameDamage {
                subject: target_filtered(R::Creature),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Urban Burgeoning — {G} Aura. Enchant land. Enchanted land has "Untap this
/// land during each other player's untap step."
pub fn urban_burgeoning() -> CardDefinition {
    CardDefinition {
        name: "Urban Burgeoning",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land untaps during each other player's untap step.",
            effect: StaticEffect::UntapAttachedEachUntapStep,
        }],
        ..Default::default()
    }
}

/// Street Sweeper — {6} 4/6 Construct artifact creature. Whenever it attacks,
/// destroy all Auras attached to target land.
pub fn street_sweeper() -> CardDefinition {
    CardDefinition {
        name: "Street Sweeper",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 4,
        toughness: 6,
        triggered_abilities: vec![on_attack(Effect::Destroy {
            what: Selector::AttachedToMe(Box::new(target_filtered(R::Land))),
        })],
        ..Default::default()
    }
}

/// Jarad's Orders — {2}{B}{G} Sorcery. Search your library for up to two
/// creature cards, put one into your hand and the other into your graveyard,
/// then shuffle. (Two sequential single searches — the player routes each pick.)
pub fn jarads_orders() -> CardDefinition {
    CardDefinition {
        name: "Jarad's Orders",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::ONE,
            },
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::Creature,
                to: ZoneDest::Graveyard,
                count: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Racecourse Fury — {R} Aura. Enchant land. Enchanted land has "{T}: Target
/// creature gains haste until end of turn."
pub fn racecourse_fury() -> CardDefinition {
    CardDefinition {
        name: "Racecourse Fury",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Target creature gains haste until end of turn.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::GrantKeyword {
                        what: target_filtered(R::Creature),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Security Blockade — {2}{W} Aura. Enchant land. When it enters, create a 2/2
/// white Knight creature token with vigilance. Enchanted land has "{T}: Prevent
/// the next 1 damage that would be dealt to you this turn."
pub fn security_blockade() -> CardDefinition {
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    };
    CardDefinition {
        name: "Security Blockade",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: knight,
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Prevent the next 1 damage that would be dealt to you this turn.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::PreventNextDamage {
                        target: Selector::Player(PlayerRef::You),
                        amount: Value::ONE,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Mana Bloom — {X}{G} Enchantment. Enters with X charge counters. Remove a
/// charge counter: add one mana of any color (once each turn). At the beginning
/// of your upkeep, if it has no charge counters, return it to its owner's hand.
pub fn mana_bloom() -> CardDefinition {
    CardDefinition {
        name: "Mana Bloom",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::Charge, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            once_per_turn: true,
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::ValueAtMost(
                    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
                    Value::Const(0),
                )),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
        }],
        ..Default::default()
    }
}
