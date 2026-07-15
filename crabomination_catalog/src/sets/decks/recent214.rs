//! Foundations (FDN) gap batch 13 — a multi-target counter payoff (Biogenic
//! Upgrade, riding the new `Selector::AllTargets`), evergreen-keyword staples,
//! a control Aura, two conditional-keyword creatures, a graveyard-recur Zombie,
//! and a Formidable haste-granter. Tests in `tests/recent214.rs`.

use crate::card::{
    ActivatedAbility, CardType, CardDefinition, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, Keyword, LandType, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack_gain_life, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, Value,
    ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// Biogenic Upgrade — {4}{G}{G} Sorcery. Distribute three +1/+1 counters among
/// one, two, or three target creatures, then double the number of +1/+1
/// counters on each of those creatures.
pub fn biogenic_upgrade() -> CardDefinition {
    CardDefinition {
        name: "Biogenic Upgrade",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DistributeCounters {
                total: Value::Const(3),
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature,
                max_targets: 3,
            },
            Effect::DoubleCountersOnEach {
                what: Selector::AllTargets,
                kind: CounterType::PlusOnePlusOne,
            },
        ]),
        ..Default::default()
    }
}

/// Herald of Faith — {3}{W}{W} 4/3 Angel. Flying; attacks → gain 2 life.
pub fn herald_of_faith() -> CardDefinition {
    CardDefinition {
        name: "Herald of Faith",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack_gain_life(2)],
        ..Default::default()
    }
}

/// Arcanis the Omnipotent — {3}{U}{U}{U} Legendary 3/4 Wizard. {T}: Draw three
/// cards. {2}{U}{U}: Return Arcanis to its owner's hand.
pub fn arcanis_the_omnipotent() -> CardDefinition {
    CardDefinition {
        name: "Arcanis the Omnipotent",
        cost: cost(&[generic(3), u(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        power: 3,
        toughness: 4,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u(), u()]),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Confiscate — {4}{U}{U} Aura. Enchant permanent. You control enchanted
/// permanent.
pub fn confiscate() -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Confiscate",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Permanent) },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains { what: enchanted() })],
        ..Default::default()
    }
}

/// Unflinching Courage — {1}{G}{W} Aura. Enchant creature; +2/+2, trample,
/// lifelink.
pub fn unflinching_courage() -> CardDefinition {
    CardDefinition {
        name: "Unflinching Courage",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Trample, Keyword::Lifelink],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Suspicious Shambler — {3}{B} 4/2 Zombie. {4}{B}{B}, Exile this card from your
/// graveyard: Create two 2/2 black Zombie tokens. Sorcery speed.
pub fn suspicious_shambler() -> CardDefinition {
    CardDefinition {
        name: "Suspicious Shambler",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 4,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b(), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: black_zombie_2_2(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kalastria Highborn — {B}{B} 2/2 Vampire Shaman. Whenever this or another
/// Vampire you control dies, you may pay {B}. If you do, each opponent loses 2
/// life and you gain 2 life. (Aristocrat drain modeled as each-opponent, per
/// Blood Artist / Zulaport; "target player" collapses to that in 1v1.)
pub fn kalastria_highborn() -> CardDefinition {
    CardDefinition {
        name: "Kalastria Highborn",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            // YourControl fires for the source's own death too (SBA funnel).
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Vampire),
                }),
            effect: Effect::MayPay {
                description: "Pay {B} to drain 2?".into(),
                mana_cost: cost(&[b()]),
                body: Box::new(Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(2),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Kargan Dragonrider — {1}{R} 2/2 Human Warrior. As long as you control a
/// Dragon, this creature has flying.
pub fn kargan_dragonrider() -> CardDefinition {
    CardDefinition {
        name: "Kargan Dragonrider",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Has flying as long as you control a Dragon.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Flying,
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Dragon).and(R::ControlledByYou),
                )),
            },
        }],
        ..Default::default()
    }
}

/// Kitesail Corsair — {1}{U} 2/1 Human Pirate. Has flying as long as it's
/// attacking.
pub fn kitesail_corsair() -> CardDefinition {
    CardDefinition {
        name: "Kitesail Corsair",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Has flying as long as it's attacking.",
            effect: StaticEffect::SelfHasKeywordWhile { keyword: Keyword::Flying, condition: R::IsAttacking },
        }],
        ..Default::default()
    }
}

/// Sphinx of the Final Word — {5}{U}{U} 5/5 Sphinx. Can't be countered; flying,
/// hexproof; instant and sorcery spells you control can't be countered.
pub fn sphinx_of_the_final_word() -> CardDefinition {
    CardDefinition {
        name: "Sphinx of the Final Word",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sphinx], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::CantBeCountered, Keyword::Flying, Keyword::Hexproof],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you control can't be countered.",
            effect: StaticEffect::SpellsUncounterable {
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            },
        }],
        ..Default::default()
    }
}

/// Drogskol Reaver — {5}{W}{U} 3/5 Spirit. Flying, double strike, lifelink;
/// whenever you gain life, draw a card.
pub fn drogskol_reaver() -> CardDefinition {
    CardDefinition {
        name: "Drogskol Reaver",
        cost: cost(&[generic(5), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Primeval Bounty — {5}{G} Enchantment. Cast a creature → 3/3 Beast token; cast
/// a noncreature → three +1/+1 counters on target creature you control;
/// landfall → gain 3 life.
pub fn primeval_bounty() -> CardDefinition {
    use crate::effect::shortcut::cast_is_noncreature;
    let cast_is_creature = Predicate::EntityMatches {
        what: Selector::TriggerSource,
        filter: R::HasCardType(CardType::Creature),
    };
    CardDefinition {
        name: "Primeval Bounty",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(cast_is_creature),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: green_beast_3_3(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(cast_is_noncreature()),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(3),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            },
        ],
        ..Default::default()
    }
}

/// Deadly Plot — {3}{B} Instant. Choose one — destroy target creature or
/// planeswalker; or return target Zombie creature card from your graveyard to
/// the battlefield tapped.
pub fn deadly_plot() -> CardDefinition {
    CardDefinition {
        name: "Deadly Plot",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Creature.or(R::Planeswalker)) },
            Effect::Move {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Zombie).and(R::Creature).and(R::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        ]),
        ..Default::default()
    }
}

/// Surrak, the Hunt Caller — {2}{G}{G} Legendary 5/4 Human Warrior. Formidable —
/// at the beginning of combat on your turn, if creatures you control have total
/// power 8 or greater, target creature you control gains haste until end of turn.
pub fn surrak_the_hunt_caller() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Surrak, the Hunt Caller",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::If {
                cond: Predicate::FormidableActive { who: PlayerRef::You },
                then: Box::new(Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Gateway Sneak — {2}{U} 1/3 Vedalken Rogue. Whenever a Gate you control
/// enters, this can't be blocked this turn. Whenever it deals combat damage to
/// a player, draw a card.
pub fn gateway_sneak() -> CardDefinition {
    CardDefinition {
        name: "Gateway Sneak",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasLandType(LandType::Gate),
                    }),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

/// Shipwreck Dowser — {3}{U}{U} 3/3 Merfolk Wizard. Prowess; ETB return target
/// instant or sorcery card from your graveyard to your hand.
pub fn shipwreck_dowser() -> CardDefinition {
    CardDefinition {
        name: "Shipwreck Dowser",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::InYourGraveyard),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Gratuitous Violence — {2}{R}{R}{R} Enchantment. If a creature you control
/// would deal damage to a permanent or player, it deals double that damage.
pub fn gratuitous_violence() -> CardDefinition {
    CardDefinition {
        name: "Gratuitous Violence",
        cost: cost(&[generic(2), r(), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control deal double damage.",
            effect: StaticEffect::DoubleDamageFromControlledCreatures,
        }],
        ..Default::default()
    }
}

// ── shared token bodies ───────────────────────────────────────────────────────

fn black_zombie_2_2() -> TokenDefinition {
    TokenDefinition {
        name: "Zombie".to_string(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    }
}

fn green_beast_3_3() -> TokenDefinition {
    TokenDefinition {
        name: "Beast".to_string(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        ..Default::default()
    }
}
