//! A Wilds of Eldraine (WOE) wave completing cards deferred from waves 14-18:
//! an untap-lock Aura (Bitter Chill), a planeswalker-hate Food Knight (Syr
//! Ginger), an Aura-anthem Archon, and a conditional-enters-tapped Whale
//! adventure (the new `StaticEffect::EntersTappedUnless`). Tests in
//! `crabomination/src/tests/recent146.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CreatureType,
    EnchantmentSubtype, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TriggeredAbility, Value, WardCost,
};
use crate::card::Zone;
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, LibraryPosition, PlayerRef, ZoneDest,
};
use crate::mana::{b, cost, g, generic, u, w};

/// Bitter Chill — {1}{U} Aura. ETB taps and locks the enchanted creature; when
/// it leaves for the graveyard you may pay {1} to scry 1 and draw.
pub fn bitter_chill() -> CardDefinition {
    CardDefinition {
        name: "Bitter Chill",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        triggered_abilities: vec![
            etb(Effect::Tap { what: Selector::AttachedTo(Box::new(Selector::This)) }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::MayPay {
                    description: "Pay {1}: scry 1, then draw a card.".into(),
                    mana_cost: cost(&[generic(1)]),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ])),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Syr Ginger, the Meal Ender — {2} legendary Food Knight 3/1. Trample,
/// hexproof, and haste while an opponent controls a planeswalker; grows and
/// scries when your artifacts die; sac for life equal to its power.
pub fn syr_ginger_the_meal_ender() -> CardDefinition {
    let opponent_has_pw =
        Predicate::SelectorExists(Selector::EachPermanent(
            R::Planeswalker.and(R::ControlledByOpponent),
        ));
    let while_pw = |keyword: Keyword| StaticAbility {
        description: "Has trample, hexproof, and haste while an opponent controls a planeswalker.",
        effect: StaticEffect::SelfHasKeywordWhilePredicate {
            keyword,
            condition: opponent_has_pw.clone(),
        },
    };
    CardDefinition {
        name: "Syr Ginger, the Meal Ender",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Knight],
            artifact_subtypes: vec![ArtifactSubtype::Food],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        static_abilities: vec![
            while_pw(Keyword::Trample),
            while_pw(Keyword::Hexproof),
            while_pw(Keyword::Haste),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                }),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Archon of the Wild Rose — {2}{W}{W} 4/4 Archon with flying. Your other
/// Aura-enchanted creatures have base power and toughness 4/4 and flying.
pub fn archon_of_the_wild_rose() -> CardDefinition {
    let your_enchanted = || {
        Selector::EachPermanent(
            R::Creature
                .and(R::ControlledByYou)
                .and(R::IsEnchanted)
                .and(R::OtherThanSource),
        )
    };
    CardDefinition {
        name: "Archon of the Wild Rose",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Archon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Other creatures you control enchanted by Auras have base P/T 4/4.",
                effect: StaticEffect::SetBasePtForFilter {
                    applies_to: your_enchanted(),
                    power: 4,
                    toughness: 4,
                },
            },
            StaticAbility {
                description: "… and have flying.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: your_enchanted(),
                    keyword: Keyword::Flying,
                },
            },
        ],
        ..Default::default()
    }
}

/// Back for Seconds — {2}{B} Sorcery with Bargain. Return up to two creature
/// cards from your graveyard to your hand; if bargained, one of them with mana
/// value 4 or less may hit the battlefield instead.
pub fn back_for_seconds() -> CardDefinition {
    let gy_creatures = |n: i32| {
        Selector::take(
            Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature },
            Value::Const(n),
        )
    };
    CardDefinition {
        name: "Back for Seconds",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Bargain],
        // Approximation: the two graveyard targets are auto-picked. When
        // bargained you may reanimate one MV≤4 creature (in lieu of the second
        // return); declining reanimation still returns one.
        effect: Effect::If {
            cond: Predicate::SpellWasBargained,
            then: Box::new(Effect::Seq(vec![
                Effect::MayDo {
                    description: "Put a creature card with mana value 4 or less onto the battlefield."
                        .into(),
                    body: Box::new(Effect::Move {
                        what: Selector::take(
                            Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: Zone::Graveyard,
                                filter: R::Creature.and(R::ManaValueAtMost(4)),
                            },
                            Value::ONE,
                        ),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                },
                Effect::Move { what: gy_creatures(1), to: ZoneDest::Hand(PlayerRef::You) },
            ])),
            else_: Box::new(Effect::Move {
                what: gy_creatures(2),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}

/// Faunsbane Troll — {2}{B}{G} 4/4 Troll. ETB hangs a Monster Role on itself.
/// {1}, Sacrifice an Aura attached to it: fight a creature you don't control,
/// exiling it if it would die. Sorcery-speed.
pub fn faunsbane_troll() -> CardDefinition {
    CardDefinition {
        name: "Faunsbane Troll",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Troll], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::CreateTokenAttachedTo {
            target: Selector::This,
            definition: super::woe_roles::monster_role(),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            sac_other_filter: Some((
                R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Aura)
                    .and(R::AttachedToSource),
                1,
            )),
            effect: Effect::Seq(vec![
                // Install the exile-if-would-die replacement before the fight
                // deals (potentially lethal) damage.
                Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
                Effect::Fight {
                    attacker: Selector::This,
                    defender: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Horned Loch-Whale // Lagoon Breach — {4}{U}{U} 6/6 Whale with flash and ward
/// {2} that enters tapped unless it's your turn. Adventure {1}{U} Instant:
/// bounce an attacking creature you don't control to the top or bottom of its
/// owner's library.
pub fn horned_loch_whale() -> CardDefinition {
    CardDefinition {
        name: "Horned Loch-Whale",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Whale], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flash, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        static_abilities: vec![StaticAbility {
            description: "This creature enters tapped unless it's your turn.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Lagoon Breach",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Move {
                what: target_filtered(R::IsAttacking.and(R::ControlledByOpponent)),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::OwnerChoice,
                },
            },
        })),
        ..Default::default()
    }
}
