//! Commander: the cards the **Sliver Swarm** precon (CMM, Sliver Gravemother)
//! needed beyond what the catalog had. Tests in `tests/recent_b/cmdr_fdc.rs`
//! (the precon-batch module).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{afflict, encore, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, RevealMissDest, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};
use std::sync::Arc;

fn sliver(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sliver], ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// "Sliver creatures you control."
fn your_sliver_creatures() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Sliver)).and(R::ControlledByYou)
}

/// "Sliver creatures you control have [ability]."
fn grant_trigger(description: &'static str, ability: TriggeredAbility) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::GrantTriggeredAbility {
            filter: your_sliver_creatures(),
            ability: Box::new(ability),
        },
    }
}

fn sliver_token() -> TokenDefinition {
    TokenDefinition {
        name: "Sliver".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sliver], ..Default::default() },
        ..Default::default()
    }
}

/// Sliver Gravemother — the legend rule spares your Slivers, and every Sliver
/// creature card in your graveyard has encore {X} (X = its mana value).
pub fn sliver_gravemother() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "The \"legend rule\" doesn't apply to Slivers you control.",
                effect: StaticEffect::LegendRuleDoesntApplyToYourMatching(R::HasCreatureType(
                    CreatureType::Sliver,
                )),
            },
            StaticAbility {
                description: "Each Sliver creature card in your graveyard has encore {X}, where \
                              X is its mana value.",
                effect: StaticEffect::GraveyardCardsHaveEncore {
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Sliver)),
                },
            },
        ],
        activated_abilities: vec![encore(cost(&[generic(5)]))],
        ..sliver("Sliver Gravemother", cost(&[w(), u(), b(), r(), g()]), 6, 6)
    }
}

/// Rukarumel, Biologist — your Slivers and nontoken creatures (and your
/// creature cards everywhere else) are the chosen type too; {3}, {T}: a
/// 1/1 colorless Sliver.
pub fn rukarumel_biologist() -> CardDefinition {
    CardDefinition {
        name: "Rukarumel, Biologist",
        cost: cost(&[w(), u(), b(), r(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        static_abilities: vec![
            StaticAbility {
                description: "Slivers you control and nontoken creatures you control are the \
                              chosen type in addition to their other creature types.",
                effect: StaticEffect::MatchingAreChosenTypeToo {
                    filter: R::HasCreatureType(CreatureType::Sliver)
                        .or(R::Creature.and(R::IsToken.negate()))
                        .and(R::ControlledByYou),
                },
            },
            StaticAbility {
                description: "The same is true for creature spells you control and creature \
                              cards you own that aren't on the battlefield.",
                effect: StaticEffect::OwnedCardsOffBattlefieldAreChosenTypeToo { filter: R::Creature },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(sliver_token()),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Capricious Sliver — your Slivers impulse-draw a card when they connect.
pub fn capricious_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![grant_trigger(
            "Sliver creatures you control have \"Whenever this creature deals combat damage to \
             a player, exile the top card of your library. You may play that card this turn.\"",
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            },
        )],
        ..sliver("Capricious Sliver", cost(&[generic(3), r()]), 3, 3)
    }
}

/// Descendants' Fury — when your creatures connect, you may sacrifice one of
/// them to reveal until a creature card sharing a creature type with it, and
/// put that card onto the battlefield.
pub fn descendants_fury() -> CardDefinition {
    CardDefinition {
        name: "Descendants' Fury",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        // CR 603.2c — once per damaged player. "One of them" is an attacking
        // creature of yours that has damaged a player this turn (so a double
        // striker's first-step hit also qualifies in the regular step).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .once_per_batch(),
            effect: Effect::MayDo {
                description: "Sacrifice one of them?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::ONE,
                        filter: R::Creature.and(R::IsAttacking).and(R::DamagedAPlayerThisTurn),
                    },
                    Effect::If {
                        cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                        then: Box::new(Effect::RevealUntilFind {
                            who: PlayerRef::You,
                            find: R::Creature.and(R::SharesCreatureTypeWithSacrificed),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                            cap: Value::Const(500),
                            life_per_revealed: 0,
                            miss_dest: RevealMissDest::BottomRandom,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Firewake Sliver — all Sliver creatures have haste; all Slivers have
/// "{1}, Sacrifice this permanent: Target Sliver creature gets +2/+2."
pub fn firewake_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "All Sliver creatures have haste.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        R::Creature.and(R::HasCreatureType(CreatureType::Sliver)),
                    ),
                    keyword: Keyword::Haste,
                },
            },
            StaticAbility {
                description: "All Slivers have \"{1}, Sacrifice this permanent: Target Sliver \
                              creature gets +2/+2 until end of turn.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Sliver)),
                    ability: ActivatedAbility {
                        mana_cost: cost(&[generic(1)]),
                        sac_cost: true,
                        effect: Effect::PumpPT {
                            what: target_filtered(
                                R::Creature.and(R::HasCreatureType(CreatureType::Sliver)),
                            ),
                            power: Value::Const(2),
                            toughness: Value::Const(2),
                            duration: Duration::EndOfTurn,
                        },
                        ..Default::default()
                    },
                    condition: None,
                },
            },
        ],
        ..sliver("Firewake Sliver", cost(&[generic(1), r(), g()]), 1, 1)
    }
}

/// For the Ancestors — choose a creature type; of the top six, every card of
/// that type goes to hand, the rest to the bottom. Flashback {3}{G}.
pub fn for_the_ancestors() -> CardDefinition {
    CardDefinition {
        name: "For the Ancestors",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), g()]))],
        effect: Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::RevealTopTakeMatchingToHand {
                who: PlayerRef::You,
                count: Value::Const(6),
                filter: R::IsSourceChosenCreatureType,
                distinct_powers: false,
            }),
        },
        ..Default::default()
    }
}

/// Hatchery Sliver — replicate {1}{G}; each Sliver spell you cast has
/// replicate equal to its mana cost.
pub fn hatchery_sliver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Replicate(cost(&[generic(1), g()]))],
        static_abilities: vec![StaticAbility {
            description: "Each Sliver spell you cast has replicate. The replicate cost is equal \
                          to its mana cost.",
            effect: StaticEffect::YourSpellsHaveReplicate {
                filter: R::HasCreatureType(CreatureType::Sliver),
            },
        }],
        ..sliver("Hatchery Sliver", cost(&[generic(1), g()]), 2, 2)
    }
}

/// Hollowhead Sliver — your Slivers rummage: "{T}, Discard a card: Draw a card."
pub fn hollowhead_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Sliver creatures you control have \"{T}, Discard a card: Draw a card.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(your_sliver_creatures()),
                ability: ActivatedAbility {
                    tap_cost: true,
                    discard_cost: Some((R::Any, 1)),
                    effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..sliver("Hollowhead Sliver", cost(&[generic(2), r()]), 2, 2)
    }
}

/// Lazotep Sliver — your Slivers have afflict 2; a nontoken Sliver of yours
/// dying amasses Slivers 2.
pub fn lazotep_sliver() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Sliver],
            ..Default::default()
        },
        static_abilities: vec![grant_trigger("Sliver creatures you control have afflict 2.", afflict(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Sliver).and(R::IsToken.negate()),
                },
            ),
            effect: Effect::Amass {
                who: PlayerRef::You,
                count: Value::Const(2),
                extra_type: Some(CreatureType::Sliver),
            },
        }],
        ..sliver("Lazotep Sliver", cost(&[generic(3), b()]), 4, 4)
    }
}

/// Pillar of Origins — as it enters, choose a creature type; {T}: one mana
/// of any color, spent only on a creature spell of that type.
pub fn pillar_of_origins() -> CardDefinition {
    CardDefinition {
        name: "Pillar of Origins",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::RestrictedToChosenTypePlain(Box::new(ManaPayload::AnyOneColor(
                    Value::ONE,
                ))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Regal Sliver — each of your Slivers entering pumps your Slivers if you're
/// the monarch, and otherwise makes you the monarch.
pub fn regal_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![grant_trigger(
            "Sliver creatures you control have \"When this creature enters, Slivers you control \
             get +1/+1 until end of turn if you're the monarch. Otherwise, you become the \
             monarch.\"",
            etb(Effect::If {
                cond: Predicate::IsMonarch { who: PlayerRef::You },
                then: Box::new(Effect::PumpPT {
                    what: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Sliver).and(R::ControlledByYou),
                    ),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::BecomeMonarch { who: PlayerRef::You }),
            }),
        )],
        ..sliver("Regal Sliver", cost(&[generic(3), w()]), 3, 3)
    }
}

/// Taunting Sliver — each of your Slivers entering goads a creature an
/// opponent controls.
pub fn taunting_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![grant_trigger(
            "Sliver creatures you control have \"When this creature enters, goad target creature \
             an opponent controls.\"",
            etb(Effect::Goad { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) }),
        )],
        ..sliver("Taunting Sliver", cost(&[generic(3), u()]), 3, 3)
    }
}

/// Titan of Littjara — it is the chosen type too; entering or attacking, you
/// may draw a card per other creature of yours sharing a type with it, then
/// discard one.
pub fn titan_of_littjara() -> CardDefinition {
    let loot = || {
        let n = Value::CountOf(Box::new(Selector::EachPermanent(
            R::Creature
                .and(R::ControlledByYou)
                .and(R::OtherThanSource)
                .and(R::SharesCreatureTypeWithSource),
        )));
        Effect::If {
            cond: Predicate::ValueAtLeast(n.clone(), Value::ONE),
            then: Box::new(Effect::MayDo {
                description: "Draw a card for each other creature sharing a type, then discard?"
                    .into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: n },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ])),
            }),
            else_: Box::new(Effect::Noop),
        }
    };
    CardDefinition {
        name: "Titan of Littjara",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Illusion], ..Default::default() },
        power: 6,
        toughness: 6,
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        static_abilities: vec![StaticAbility {
            description: "This creature is the chosen type in addition to its other types.",
            effect: StaticEffect::MatchingAreChosenTypeToo { filter: R::IsSource },
        }],
        triggered_abilities: vec![etb(loot()), on_attack(loot())],
        ..Default::default()
    }
}
