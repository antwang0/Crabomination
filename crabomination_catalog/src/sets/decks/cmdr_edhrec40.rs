//! Commander: the 2026-10-13 rescan's gaps for Braids, Conjurer Adept;
//! Taigam; Carth the Lion (and, in later batches, Gogo, Edea, Uugguu and
//! Loot). Tests in `tests/recent_b/cmdr_edhrec40.rs`.

use crate::card::{
    ActivatedAbility, BattleSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EntersAsCopy, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype,
    Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{craft, etb, target_any, target_filtered};
use crate::effect::{ActivationTrait, Duration, Effect, ManaPayload, PlayerRef, VoteOption, ZoneDest};
use crate::mana::SpendRestriction;
use crate::mana::{b, cost, g, generic, r, u, Color, ManaCost};
use crabomination_base::tokens::treasure_token;
use crabomination_base::turn_step::TurnStep;
use std::sync::Arc;

// ── Braids, Conjurer Adept / Taigam / Gogo ───────────────────────────────────

/// Shadow of the Second Sun — {4}{U}{U} Aura, enchant player. At the beginning
/// of each of the enchanted player's postcombat main phases, there is an
/// additional beginning phase after this phase (CR 500.8).
pub fn shadow_of_the_second_sun() -> CardDefinition {
    CardDefinition {
        name: "Shadow of the Second Sun",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::AnyPlayer)
                .with_filter(Predicate::IsTurnOf(PlayerRef::EnchantedPlayer)),
            effect: Effect::AdditionalBeginningPhase { count: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Jhoira's Timebug — {2} 1/2 artifact Insect. {T}: a permanent you control or
/// a suspended card you own; if it has a time counter it may lose one or gain
/// another (any permanent you control is a legal target).
pub fn jhoiras_timebug() -> CardDefinition {
    let target = || {
        target_filtered(
            R::OnBattlefield
                .and(R::ControlledByYou)
                .or(R::InExile.and(R::IsSuspended).and(R::OwnedByYou)),
        )
    };
    CardDefinition {
        name: "Jhoira's Timebug",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PlayerChoosesOne {
                who: PlayerRef::You,
                options: vec![
                    VoteOption::new(
                        "Remove a time counter",
                        Effect::RemoveCounter { what: target(), kind: CounterType::Time, amount: Value::ONE },
                    ),
                    VoteOption::new(
                        "Put another time counter",
                        Effect::If {
                            cond: Predicate::EntityMatches {
                                what: Selector::Target(0),
                                filter: R::WithCounter(CounterType::Time),
                            },
                            then: Box::new(Effect::AddCounter {
                                what: Selector::Target(0),
                                kind: CounterType::Time,
                                amount: Value::ONE,
                            }),
                            else_: Box::new(Effect::Noop),
                        },
                    ),
                    VoteOption::new("Leave it", Effect::Noop),
                ],
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Carth the Lion ───────────────────────────────────────────────────────────

fn legendary_enchantment(name: &'static str, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

/// Way of the Necromancer — {1}{B} legendary enchantment. Enters: empower
/// Jace 2. Whenever a creature you control dies, each planeswalker you control
/// gets a loyalty counter.
pub fn way_of_the_necromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(2) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(R::Planeswalker.and(R::ControlledByYou)),
                    kind: CounterType::Loyalty,
                    amount: Value::ONE,
                },
            },
        ],
        ..legendary_enchantment("Way of the Necromancer", cost(&[generic(1), b()]))
    }
}

/// Way of the Paradox — {2}{G} legendary enchantment. Enters: empower Jace 5.
/// Whenever you activate a loyalty ability (CR 606.1), you gain 1 life and may
/// play an additional land this turn.
pub fn way_of_the_paradox() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(5) }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::AbilityActivatedWith(ActivationTrait::Loyalty),
                    EventScope::YourControl,
                ),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                    Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::ONE },
                ]),
            },
        ],
        ..legendary_enchantment("Way of the Paradox", cost(&[generic(2), g()]))
    }
}

/// Vraska, Relic Seeker — {4}{B}{G} planeswalker, loyalty 6. +2: a 2/2 black
/// Pirate with menace; −3: destroy an artifact, creature or enchantment and
/// make a Treasure; −10: target player's life total becomes 1.
pub fn vraska_relic_seeker() -> CardDefinition {
    let pirate = TokenDefinition {
        name: "Pirate".into(),
        power: 2,
        toughness: 2,
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pirate], ..Default::default() },
        keywords: vec![Keyword::Menace],
        ..Default::default()
    };
    CardDefinition {
        name: "Vraska, Relic Seeker",
        cost: cost(&[generic(4), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Vraska], ..Default::default() },
        base_loyalty: 6,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(pirate) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::Destroy {
                        what: target_filtered(R::Artifact.or(R::Creature).or(R::HasCardType(CardType::Enchantment))),
                    },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(treasure_token()),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::SetLifeTotal { who: target_filtered(R::Player), amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn your_step(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

// ── Loot, the Key to Everything ──────────────────────────────────────────────

/// Loot, the Key to Everything — {G}{U}{R} legendary 1/2 Beast Noble. Ward
/// {1}. Upkeep: exile the top X (card types among your other nonland
/// permanents); you may play them this turn.
pub fn loot_the_key_to_everything() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![your_step(
            TurnStep::Upkeep,
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::CardTypesAmong(Box::new(Selector::EachPermanent(
                    R::Nonland.and(R::ControlledByYou).and(R::OtherThanSource),
                ))),
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        )],
        ..legend(
            "Loot, the Key to Everything",
            cost(&[g(), u(), r()]),
            vec![CreatureType::Beast, CreatureType::Noble],
            1,
            2,
        )
    }
}

/// Invasion of Kaldheim // Pyre of the World Tree — {3}{R} Siege, defense 4.
/// Enters: exile your hand, draw that many; play the exiled cards until the
/// end of your next turn. Pyre: discard a land card: 2 damage to any target;
/// whenever you discard a land card, exile your top card, playable this turn.
pub fn invasion_of_kaldheim() -> CardDefinition {
    let exiled = || Selector::ExiledThisResolution { filter: R::Any };
    let pyre = CardDefinition {
        name: "Pyre of the World Tree",
        card_types: vec![CardType::Enchantment],
        color_indicator: vec![Color::Red],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Land, 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Kaldheim",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExileHand { who: PlayerRef::You },
            Effect::Draw { who: Selector::You, amount: Value::count(exiled()) },
            Effect::GrantMayPlay {
                what: exiled(),
                duration: MayPlayDuration::EndOfControllersNextTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        ]))],
        back_face: Some(Box::new(pyre)),
        ..Default::default()
    }
}

/// Omenpath Journey — {3}{G} enchantment. Enters: exile up to five land cards
/// with different names from your library. Your end step: one of them at
/// random onto the battlefield tapped.
pub fn omenpath_journey() -> CardDefinition {
    let fetch = || Effect::Search {
        who: PlayerRef::You,
        filter: R::Land.and(R::Not(Box::new(R::SameNameAsExiledWithSource))),
        to: ZoneDest::ExileWithSourceStamp,
    };
    CardDefinition {
        name: "Omenpath Journey",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq((0..5).map(|_| fetch()).collect())),
            your_step(
                TurnStep::End,
                Effect::ChooseOneAtRandomAmong {
                    what: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Exile, filter: R::ExiledWithSource },
                    chosen: Box::new(Effect::Move {
                        what: Selector::SeparatedPile { chosen: true },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    }),
                    other: Box::new(Effect::Noop),
                },
            ),
        ],
        ..Default::default()
    }
}

// ── Uugguu, the Omniplasm ────────────────────────────────────────────────────

/// Uugguu, the Omniplasm — {B}{G}{U} legendary 2/2 Ooze. Whenever another
/// nontoken Ooze creature you control dies, two token copies of it, 2/2 and
/// not legendary.
pub fn uugguu_the_omniplasm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken.and(R::HasCreatureType(CreatureType::Ooze)),
                },
            ),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::Const(2),
                source: Selector::TriggerSource,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: Some((2, 2)),
                override_colors: None,
                enters_tapped: false,
                non_legendary: true,
                legendary: false,
                extra_keywords: vec![],
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            },
        }],
        ..legend("Uugguu, the Omniplasm", cost(&[b(), g(), u()]), vec![CreatureType::Ooze], 2, 2)
    }
}

/// Omni-Changeling — {3}{U}{U} 0/0 Shapeshifter. Changeling, convoke. It may
/// enter as a copy of any creature, except it has changeling.
pub fn omni_changeling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling, Keyword::Convoke],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_keywords: vec![Keyword::Changeling],
            ..Default::default()
        }),
        ..creature("Omni-Changeling", cost(&[generic(3), u(), u()]), vec![CreatureType::Shapeshifter], 0, 0)
    }
}

/// Gutter Grime — {4}{G} enchantment. Whenever a nontoken creature you
/// control dies, a slime counter here, then a green Ooze token whose P/T are
/// the slime counters on Gutter Grime.
pub fn gutter_grime() -> CardDefinition {
    let slime = || Value::CountersOn { what: Box::new(Selector::CreatorOfSource), kind: CounterType::Slime };
    let ooze = TokenDefinition {
        name: "Ooze".into(),
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ooze], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This token's power and toughness are each equal to the number of slime counters on Gutter Grime.",
            effect: StaticEffect::SelfBasePtFromValue { power: slime(), toughness: slime() },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Gutter Grime",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Slime, amount: Value::ONE },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(ooze) },
            ]),
        }],
        ..Default::default()
    }
}

// ── Edea, Possessed Sorceress ────────────────────────────────────────────────

fn not_owned() -> R {
    R::Not(Box::new(R::OwnedByYou))
}

/// Edea, Possessed Sorceress — {2}{U}{B}{R} legendary 2/5 Human Warlock. Ward
/// {2}. Your beginning of combat: steal an opponent's creature until end of
/// turn, untapped and hasty. A creature you control but don't own dying
/// returns to its owner's battlefield, and you draw.
pub fn edea_possessed_sorceress() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![
            your_step(
                TurnStep::BeginCombat,
                Effect::Seq(vec![
                    Effect::GainControl {
                        what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                        to: None,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Untap { what: Selector::Target(0), up_to: None },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
            ),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: not_owned() }),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                            tapped: false,
                        },
                    },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
            },
        ],
        ..legend(
            "Edea, Possessed Sorceress",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            2,
            5,
        )
    }
}

/// Chaos Shrine's Black Crystal — {3}{B} legendary artifact. A nontoken
/// creature you control dying is exiled with it; your upkeep: you may put one
/// of those onto the battlefield under your control with a finality counter.
pub fn chaos_shrines_black_crystal() -> CardDefinition {
    let picked = || Selector::SeparatedPile { chosen: true };
    CardDefinition {
        name: "Chaos Shrine's Black Crystal",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken }),
                effect: Effect::Move { what: Selector::TriggerSource, to: ZoneDest::ExileWithSourceStamp },
            },
            your_step(
                TurnStep::Upkeep,
                Effect::MayDo {
                    description: "Put a creature card exiled with Chaos Shrine's Black Crystal onto the battlefield?"
                        .into(),
                    body: Box::new(Effect::ChooseOneAmong {
                        what: Selector::CardsInZone {
                            who: PlayerRef::EachPlayer,
                            zone: Zone::Exile,
                            filter: R::Creature.and(R::ExiledWithSource),
                        },
                        chooser: PlayerRef::You,
                        chosen: Box::new(Effect::Seq(vec![
                            Effect::Move {
                                what: picked(),
                                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                            },
                            Effect::AddCounter { what: picked(), kind: CounterType::Finality, amount: Value::ONE },
                        ])),
                        other: Box::new(Effect::Noop),
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Garland, Royal Kidnapper — {2}{U}{B} legendary 3/4 Human Knight. Enters:
/// target opponent becomes the monarch. Whenever an opponent becomes the
/// monarch, steal a creature they control for as long as they're the monarch.
/// Your creatures you don't own get +2/+2 and can't be sacrificed.
pub fn garland_royal_kidnapper() -> CardDefinition {
    let stolen = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(not_owned()));
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch {
                who: PlayerRef::ControllerOf(Box::new(Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer })),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameMonarch, EventScope::OpponentControl),
                effect: Effect::GainControlWhileMonarch {
                    what: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)),
                    monarch: PlayerRef::Triggerer,
                },
            },
        ],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control but don't own get +2/+2.",
                effect: StaticEffect::PumpPT { applies_to: stolen(), power: 2, toughness: 2 },
            },
            StaticAbility {
                description: "Creatures you control but don't own can't be sacrificed.",
                effect: StaticEffect::GrantKeyword { applies_to: stolen(), keyword: Keyword::CantBeSacrificed },
            },
        ],
        ..legend(
            "Garland, Royal Kidnapper",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Human, CreatureType::Knight],
            3,
            4,
        )
    }
}

// ── Gogo, Master of Mimicry ──────────────────────────────────────────────────

/// The Enigma Jewel // Locus of Enlightenment — {U} legendary artifact. Enters
/// tapped; {T}: {C}{C} for abilities only. Craft with four nonlands with
/// activated abilities {8}{U} (the printed "four or more" crafts with exactly
/// four). Locus: the crafted cards' activated abilities, each once a turn;
/// whenever you activate a non-mana ability, copy it.
pub fn the_enigma_jewel() -> CardDefinition {
    let locus = CardDefinition {
        name: "Locus of Enlightenment",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        color_indicator: vec![Color::Blue],
        static_abilities: vec![StaticAbility {
            description: "Locus of Enlightenment has each activated ability of the exiled cards used to craft it. \
                          You may activate each of those abilities only once each turn.",
            effect: StaticEffect::HasActivatedAbilitiesOfExiledWithSelfOnceEachTurn,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivated, EventScope::YourControl),
            effect: Effect::CopyActivatedAbilityMayChooseTargets,
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "The Enigma Jewel",
        cost: cost(&[u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "The Enigma Jewel enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colorless(Value::Const(2))),
                        SpendRestriction::AbilitiesOnly,
                    ),
                },
                ..Default::default()
            },
            craft(cost(&[generic(8), u()]), R::Nonland.and(R::HasActivatedAbility), 4),
        ],
        back_face: Some(Box::new(locus)),
        ..Default::default()
    }
}
