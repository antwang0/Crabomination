//! Theros (THS) — batch 5: the last sixteen gap cards. Tests in `classic_sets/ths`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, LandType, LoyaltyAbility,
    MayPlayDuration, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, heroic, monstrosity, on_becomes_monstrous, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, ZoneDest,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    mana: ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: ct,
            ..Default::default()
        },
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
    }
}

/// "Return that card to the battlefield under your control" — the shared body
/// of Rescue from the Underworld's two delayed returns.
fn reanimate_slot0() -> Effect {
    Effect::Move {
        what: Selector::Target(0),
        to: ZoneDest::Battlefield {
            controller: PlayerRef::You,
            tapped: false,
        },
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, p, t, ct, kw)
    }
}

/// Artisan of Forms — {1}{U} 1/1 Human Wizard. Heroic: you may have it become
/// a copy of target creature, except it keeps this ability.
pub fn artisan_of_forms() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::MayDo {
            description: "Become a copy of target creature".into(),
            body: Box::new(Effect::BecomeCopyOf {
                what: Selector::This,
                source: target_filtered(R::Creature),
                extra_creature_types: vec![],
                keep_own_triggered: true,
                keep_own_activated: false,
            }),
        })],
        ..creature(
            "Artisan of Forms",
            cost(&[generic(1), u()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Ashen Rider — {4}{W}{W}{B}{B} 5/5 Archon with flying. Enters or dies: exile
/// target permanent.
pub fn ashen_rider() -> CardDefinition {
    let exile = || Effect::Exile {
        what: target_filtered(R::Permanent),
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(exile()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: exile(),
            },
        ],
        ..creature(
            "Ashen Rider",
            cost(&[generic(4), w(), w(), b(), b()]),
            5,
            5,
            vec![CreatureType::Archon],
            vec![Keyword::Flying],
        )
    }
}

/// Chained to the Rocks — {W} Aura enchanting a Mountain you control. ETB:
/// exile target creature an opponent controls until this Aura leaves.
pub fn chained_to_the_rocks() -> CardDefinition {
    CardDefinition {
        name: "Chained to the Rocks",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::HasLandType(LandType::Mountain).and(R::ControlledByYou)),
        },
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Curse of the Swine — {X}{U}{U} Sorcery. Exile X target creatures; each
/// victim's controller creates a 2/2 green Boar.
pub fn curse_of_the_swine() -> CardDefinition {
    CardDefinition {
        name: "Curse of the Swine",
        cost: cost(&[x(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CapTargetsAtX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                        count: Value::ONE,
                        definition: TokenDefinition {
                            name: "Boar".into(),
                            power: 2,
                            toughness: 2,
                            card_types: vec![CardType::Creature],
                            colors: vec![Color::Green],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Boar],
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    },
                    Effect::Exile {
                        what: Selector::Target(0),
                    },
                ])),
            }),
        },
        ..Default::default()
    }
}

/// Daxos of Meletis — {1}{W}{U} 2/2 legendary Human Soldier. Unblockable by
/// power 3+; combat damage to a player exiles their top card, gains you life
/// equal to its mana value, and lets you cast it this turn with any color mana.
pub fn daxos_of_meletis() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::DefendingPlayer,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: true,
                    max_mana_value: None,
                    pay_own_cost: false,
                    uncast_penalty: None,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ManaValueOf(Box::new(Selector::LastMoved)),
                },
            ]),
        }],
        ..legend(
            "Daxos of Meletis",
            cost(&[generic(1), w(), u()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![Keyword::CantBeBlockedByPowerAtLeast(3)],
        )
    }
}

/// Gift of Immortality — {2}{W} Aura. When enchanted creature dies, return that
/// card to the battlefield; at the beginning of the next end step, return this
/// Aura attached to it.
pub fn gift_of_immortality() -> CardDefinition {
    CardDefinition {
        name: "Gift of Immortality",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                        tapped: false,
                    },
                },
                Effect::DelayUntilWithCapture {
                    kind: DelayedTriggerKind::NextEndStep,
                    capture: Selector::TriggerSource,
                    body: Box::new(Effect::ReturnSelfAttachedToTarget),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Polis Crusher — {2}{R}{G} 4/4 Cyclops with trample and protection from
/// enchantments. Monstrosity 3; while monstrous, combat damage to a player
/// destroys target enchantment that player controls.
pub fn polis_crusher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(4), r(), g()]), 3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                filter: Some(Predicate::SourceIsMonstrous),
                ..EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
            },
            effect: Effect::Destroy {
                what: target_filtered(R::Enchantment.and(R::ControlledByOpponent)),
            },
        }],
        ..creature(
            "Polis Crusher",
            cost(&[generic(2), r(), g()]),
            4,
            4,
            vec![CreatureType::Cyclops],
            vec![
                Keyword::Trample,
                Keyword::ProtectionFromCardType(CardType::Enchantment),
            ],
        )
    }
}

/// Polukranos, World Eater — {2}{G}{G} 5/5 legendary Hydra. {X}{X}{G}:
/// Monstrosity X; when it becomes monstrous it deals X damage divided among
/// target creatures your opponents control, each of which hits back for its
/// power.
pub fn polukranos_world_eater() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), x(), g()]),
            effect: Effect::Monstrosity {
                n: Value::XFromCost,
            },
            sorcery_speed: true,
            ..Default::default()
        }],
        triggered_abilities: vec![on_becomes_monstrous(Effect::DealDamageDivided {
            total: Value::TriggerEventAmount,
            filter: R::Creature.and(R::ControlledByOpponent),
            max_targets: 8,
            retaliate_to_source: true,
        })],
        ..legend(
            "Polukranos, World Eater",
            cost(&[generic(2), g(), g()]),
            5,
            5,
            vec![CreatureType::Hydra],
            vec![],
        )
    }
}

/// Prophet of Kruphix — {3}{G}{U} 2/3 Human Wizard. Your creatures and lands
/// untap during each other player's untap step; you may cast creature spells
/// as though they had flash.
pub fn prophet_of_kruphix() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Untap all creatures and lands you control during each other \
                              player's untap step.",
                effect: StaticEffect::UntapYoursEachUntapStepFiltered(
                    R::Creature.or(R::Land).and(R::ControlledByYou),
                ),
            },
            StaticAbility {
                description: "You may cast creature spells as though they had flash.",
                effect: StaticEffect::ControllerSpellsHaveFlash {
                    filter: R::Creature,
                },
            },
        ],
        ..creature(
            "Prophet of Kruphix",
            cost(&[generic(3), g(), u()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Psychic Intrusion — {3}{U}{B} Sorcery. Target opponent reveals their hand;
/// exile a nonland card from their hand or graveyard and cast it while it
/// remains exiled, spending mana as though it were any color.
pub fn psychic_intrusion() -> CardDefinition {
    CardDefinition {
        name: "Psychic Intrusion",
        cost: cost(&[generic(3), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileChosenFromHandOrGraveyard {
                who: PlayerRef::Target(0),
                filter: R::Nonland,
            },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: true,
            },
        ]),
        ..Default::default()
    }
}

/// Rescue from the Underworld — {4}{B} Instant. Sacrifice a creature as an
/// additional cost; at the beginning of your next upkeep return both it and
/// target creature card in your graveyard, then exile this card.
pub fn rescue_from_the_underworld() -> CardDefinition {
    CardDefinition {
        name: "Rescue from the Underworld",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        effect: Effect::Seq(vec![
            Effect::DelayUntilWithCapture {
                kind: DelayedTriggerKind::YourNextUpkeep,
                capture: Selector::Target(0),
                body: Box::new(reanimate_slot0()),
            },
            Effect::DelayUntilWithCapture {
                kind: DelayedTriggerKind::YourNextUpkeep,
                capture: Selector::SacrificedCard,
                body: Box::new(reanimate_slot0()),
            },
            Effect::ExileSource,
        ]),
        ..Default::default()
    }
}

/// Shipbreaker Kraken — {4}{U}{U} 6/6 Kraken. {6}{U}{U}: Monstrosity 4; when it
/// becomes monstrous, tap up to four target creatures which don't untap for as
/// long as you control this.
pub fn shipbreaker_kraken() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(6), u(), u()]), 4)],
        triggered_abilities: vec![on_becomes_monstrous(Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::TapAndLockWhileSourcePresent {
                what: Selector::Target(0),
            }),
        })],
        ..creature(
            "Shipbreaker Kraken",
            cost(&[generic(4), u(), u()]),
            6,
            6,
            vec![CreatureType::Kraken],
            vec![],
        )
    }
}

/// Triad of Fates — {2}{W}{B} 3/3 legendary Human Wizard. Marks creatures with
/// fate counters, then either blinks one or exiles it to draw two.
pub fn triad_of_fates() -> CardDefinition {
    let fated = || R::Creature.and(R::WithCounter(CounterType::Fate));
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    kind: CounterType::Fate,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Exile {
                        what: target_filtered(fated()),
                    },
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                            tapped: false,
                        },
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Exile {
                        what: target_filtered(fated()),
                    },
                    Effect::Draw {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(
                            0,
                        )))),
                        amount: Value::Const(2),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..legend(
            "Triad of Fates",
            cost(&[generic(2), w(), b()]),
            3,
            3,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Triton Tactics — {U} Instant. Up to two target creatures get +0/+3 and
/// untap; at end of combat, everything they blocked this turn taps and skips
/// its next untap.
pub fn triton_tactics() -> CardDefinition {
    CardDefinition {
        name: "Triton Tactics",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::Const(0),
                        toughness: Value::Const(3),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Untap {
                        what: Selector::Target(0),
                        up_to: None,
                    },
                ])),
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::EndOfCombat,
                body: Box::new(Effect::TapBlockedByAndSkipUntap {
                    what: Selector::AllTargets,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Underworld Cerberus — {3}{B}{R} 6/6 Dog. Blockable only by three or more
/// creatures; graveyard cards can't be targeted; on death it exiles itself and
/// every player returns their graveyard creatures to hand.
pub fn underworld_cerberus() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Cards in graveyards can't be the targets of spells or abilities.",
            effect: StaticEffect::GraveyardCardsUntargetable,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileSource,
                Effect::Move {
                    what: Selector::CardsInZone {
                        who: PlayerRef::EachPlayer,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::Creature,
                    },
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ]),
        }],
        ..creature(
            "Underworld Cerberus",
            cost(&[generic(3), b(), r()]),
            6,
            6,
            vec![CreatureType::Dog],
            vec![Keyword::CantBeBlockedExceptByN(3)],
        )
    }
}

/// Xenagos, the Reveler — {2}{R}{G} legendary planeswalker, loyalty 3. +1 adds
/// {R}/{G} per creature you control; 0 mints a hasty Satyr; −6 exiles seven and
/// deploys the creatures and lands among them.
pub fn xenagos_the_reveler() -> CardDefinition {
    CardDefinition {
        name: "Xenagos, the Reveler",
        cost: cost(&[generic(2), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Xenagos],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(
                        vec![Color::Red, Color::Green],
                        Value::CountOf(Box::new(Selector::EachPermanent(
                            R::Creature.and(R::ControlledByYou),
                        ))),
                    ),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Satyr".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Red, Color::Green],
                        keywords: vec![Keyword::Haste],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Satyr],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::LookTopPutMatchingOntoBattlefield {
                    count: Value::Const(7),
                    filter: R::Creature.or(R::Land),
                    then: None,
                    max: None,
                    tapped: false,
                    exile_rest: true,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
