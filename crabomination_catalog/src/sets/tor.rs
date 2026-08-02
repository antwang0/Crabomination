//! Torment (TOR) — 2002. The black-heavy half of the Odyssey block: Madness,
//! Threshold payoffs and the Cephalid self-mill shell. Tests in
//! `classic_sets/tor`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes, Supertype,
    TriggeredAbility, TokenDefinition,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, StaticEffect, Value,
    ZoneDest,
    shortcut::{draw, etb, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, c, types, p, t)
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn threshold() -> Predicate {
    Predicate::ThresholdActive { who: PlayerRef::You }
}

/// "Threshold — as long as there are seven or more cards in your graveyard,
/// this creature has [ability]."
fn threshold_grant(description: &'static str, ability: TriggeredAbility) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::WhileCondition {
            condition: threshold(),
            inner: Box::new(StaticEffect::GrantTriggeredAbility {
                filter: R::IsSource,
                ability: Box::new(ability),
            }),
        },
    }
}

fn upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(crate::game::TurnStep::Upkeep),
            EventScope::YourControl,
        ),
        effect,
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Accelerate — {1}{R}. Haste and a card.
pub fn accelerate() -> CardDefinition {
    instant(
        "Accelerate",
        cost(&[generic(1), r()]),
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Acorn Harvest — {3}{G}. Two Squirrels, twice.
pub fn acorn_harvest() -> CardDefinition {
    CardDefinition {
        // The printed flashback is "{1}{G}, Pay 3 life"; the life half of a
        // flashback cost isn't modeled.
        keywords: vec![Keyword::Flashback(cost(&[generic(1), g()]))],
        ..sorcery(
            "Acorn Harvest",
            cost(&[generic(3), g()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: TokenDefinition {
                    name: "Squirrel".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Squirrel],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )
    }
}

/// Churning Eddy — {3}{U}. Bounce a creature and a land.
pub fn churning_eddy() -> CardDefinition {
    sorcery(
        "Churning Eddy",
        cost(&[generic(3), u()]),
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::Move {
                what: Selector::TargetFiltered { slot: 1, filter: R::Land },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        ]),
    )
}

/// Circular Logic — {2}{U}. Counter unless they pay for your whole graveyard.
pub fn circular_logic() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Madness(cost(&[u()]))],
        ..instant(
            "Circular Logic",
            cost(&[generic(2), u()]),
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: ManaCost::default(),
                exile: false,
                extra_generic: Some(Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: R::Any,
                }),
            },
        )
    }
}

/// Breakthrough — {X}{U}. Draw four, keep X.
pub fn breakthrough() -> CardDefinition {
    sorcery(
        "Breakthrough",
        cost(&[x(), u()]),
        Effect::Seq(vec![
            draw(4),
            Effect::Discard {
                who: Selector::You,
                amount: Value::Max(
                    Box::new(Value::ZERO),
                    Box::new(Value::Diff(
                        Box::new(Value::CountOf(Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: crate::card::Zone::Hand,
                            filter: R::Any,
                        }))),
                        Box::new(Value::XFromCost),
                    )),
                ),
                random: false,
            },
        ]),
    )
}

/// Cleansing Meditation — {1}{W}{W}. Wrath for enchantments.
pub fn cleansing_meditation() -> CardDefinition {
    // The Threshold half ("return all enchantments destroyed this way to the
    // battlefield") is not modeled.
    sorcery(
        "Cleansing Meditation",
        cost(&[generic(1), w(), w()]),
        Effect::Destroy { what: Selector::EachPermanent(R::Enchantment) },
    )
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Angel of Retribution — {6}{W} 5/5 flying first striker.
pub fn angel_of_retribution() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..creature("Angel of Retribution", cost(&[generic(6), w()]), vec![CreatureType::Angel], 5, 5)
    }
}

/// Aquamoeba — {1}{U} 1/3 that flips its stats for a card.
pub fn aquamoeba() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::SwitchPT { what: Selector::This, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..creature(
            "Aquamoeba",
            cost(&[generic(1), u()]),
            vec![CreatureType::Elemental, CreatureType::Beast],
            1,
            3,
        )
    }
}

/// Aven Trooper — {3}{W} 1/1 flier that trades cards for stats.
pub fn aven_trooper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Aven Trooper",
            cost(&[generic(3), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Balshan Collaborator — {3}{U} 2/2 flier that pumps off black mana.
pub fn balshan_collaborator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Balshan Collaborator",
            cost(&[generic(3), u()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Balthor the Stout — {1}{R}{R} 2/2 Barbarian lord.
pub fn balthor_the_stout() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Barbarian creatures get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature
                    .and(R::HasCreatureType(CreatureType::Barbarian))
                    .and(R::OtherThanSource),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: target_filtered(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Barbarian))
                        .and(R::OtherThanSource),
                ),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Balthor the Stout",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Dwarf, CreatureType::Barbarian],
            2,
            2,
        )
    }
}

/// Boneshard Slasher — {1}{B} 1/1 flier that gets big and brittle past
/// Threshold.
pub fn boneshard_slasher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Threshold — +2/+2.",
                effect: StaticEffect::PumpSelfIf {
                    condition: threshold(),
                    power: 2,
                    toughness: 2,
                    keywords: vec![],
                },
            },
            threshold_grant(
                "Threshold — when this becomes the target of a spell or ability, sacrifice it.",
                TriggeredAbility {
                    event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
                    effect: Effect::SacrificeSource,
                },
            ),
        ],
        ..creature("Boneshard Slasher", cost(&[generic(1), b()]), vec![CreatureType::Horror], 1, 1)
    }
}

/// Cabal Surgeon — {2}{B}{B} 2/1 that buys creatures back out of the
/// graveyard.
pub fn cabal_surgeon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), b()]),
            tap_cost: true,
            exile_other_filter: Some((R::Any, 2)),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature(
            "Cabal Surgeon",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Human, CreatureType::Minion],
            2,
            1,
        )
    }
}

/// Cabal Torturer — {1}{B}{B} 1/1 pinger that doubles up past Threshold.
pub fn cabal_torturer() -> CardDefinition {
    let shrink = |c: ManaCost, n: i32, condition| ActivatedAbility {
        mana_cost: c,
        tap_cost: true,
        condition,
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-n),
            toughness: Value::Const(-n),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            shrink(cost(&[b()]), 1, None),
            shrink(cost(&[generic(3), b(), b()]), 2, Some(threshold())),
        ],
        ..creature(
            "Cabal Torturer",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Human, CreatureType::Minion],
            1,
            1,
        )
    }
}

/// Centaur Chieftain — {3}{G} 3/3 haste that rallies the team past Threshold.
pub fn centaur_chieftain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![threshold_grant(
            "Threshold — when this enters, your creatures get +1/+1 and trample.",
            etb(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ])),
        )],
        ..creature("Centaur Chieftain", cost(&[generic(3), g()]), vec![CreatureType::Centaur], 3, 3)
    }
}

/// Centaur Veteran — {5}{G} 3/3 trampler that regenerates for a card.
pub fn centaur_veteran() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Centaur Veteran", cost(&[generic(5), g()]), vec![CreatureType::Centaur], 3, 3)
    }
}

/// The Cephalid self-millers: pointing anything at them fills the graveyard.
fn cephalid_miller(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    mill: i32,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::Mill { who: Selector::You, amount: Value::Const(mill) },
        }],
        ..creature(name, c, types, p, t)
    }
}

/// Cephalid Aristocrat — {4}{U} 3/3 that mills two off any pointer.
pub fn cephalid_aristocrat() -> CardDefinition {
    cephalid_miller(
        "Cephalid Aristocrat",
        cost(&[generic(4), u()]),
        vec![CreatureType::Octopus, CreatureType::Noble],
        3,
        3,
        2,
    )
}

/// Cephalid Illusionist — {1}{U} 1/1 that mills three and fogs one creature.
pub fn cephalid_illusionist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::PreventCombatDamageToTargetThisTurn {
                    target: target_filtered(R::Creature.and(R::ControlledByYou)),
                },
                Effect::PreventCombatDamageByTargetThisTurn { target: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..cephalid_miller(
            "Cephalid Illusionist",
            cost(&[generic(1), u()]),
            vec![CreatureType::Octopus, CreatureType::Wizard],
            1,
            1,
            3,
        )
    }
}

/// Cephalid Sage — {3}{U} 2/3 that draws three past Threshold.
pub fn cephalid_sage() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![threshold_grant(
            "Threshold — when this enters, draw three cards, then discard two.",
            etb(Effect::Seq(vec![
                draw(3),
                Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
            ])),
        )],
        ..creature("Cephalid Sage", cost(&[generic(3), u()]), vec![CreatureType::Octopus], 2, 3)
    }
}

/// Cephalid Snitch — {1}{U} 1/1 that strips protection from black.
pub fn cephalid_snitch() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::LoseKeywordThisTurn {
                what: target_filtered(R::Creature),
                keyword: Keyword::Protection(Color::Black),
            },
            ..Default::default()
        }],
        ..creature(
            "Cephalid Snitch",
            cost(&[generic(1), u()]),
            vec![CreatureType::Octopus, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Cephalid Vandal — {1}{U} 1/1 that mills faster every turn.
pub fn cephalid_vandal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![upkeep(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Shred,
                amount: Value::ONE,
            },
            Effect::Mill {
                who: Selector::You,
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Shred,
                },
            },
        ]))],
        ..creature(
            "Cephalid Vandal",
            cost(&[generic(1), u()]),
            vec![CreatureType::Octopus, CreatureType::Rogue],
            1,
            1,
        )
    }
}

/// Ambassador Laquatus — {1}{U}{U} 1/3 that mills for {3}.
pub fn ambassador_laquatus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..legend(
            "Ambassador Laquatus",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Chainer, Dementia Master — {3}{B}{B} 3/3 that reanimates into Nightmares.
pub fn chainer_dementia_master() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All Nightmares get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::HasCreatureType(CreatureType::Nightmare)),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b(), b()]),
            life_cost: 3,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::AddCreatureTypes {
                    what: Selector::LastMoved,
                    creature_types: vec![CreatureType::Nightmare],
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Exile {
                what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Nightmare)),
            },
        }],
        ..legend(
            "Chainer, Dementia Master",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Human, CreatureType::Minion],
            3,
            3,
        )
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Compulsion — {1}{U}. Loot for {1}{U}, or cash itself in.
pub fn compulsion() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                discard_cost: Some((R::Any, 1)),
                effect: draw(1),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                sac_cost: true,
                effect: draw(1),
                ..Default::default()
            },
        ],
        ..enchantment("Compulsion", cost(&[generic(1), u()]))
    }
}

/// Coral Net — {U} Aura that taxes a green or white creature a card a turn.
pub fn coral_net() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                R::Creature.and(R::HasColor(Color::Green).or(R::HasColor(Color::White))),
            ),
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::SacrificeSourceUnlessCost {
                    cost: crate::card::WardCost::Discard(1),
                },
            }],
            ..Default::default()
        }),
        ..enchantment("Coral Net", cost(&[u()]))
    }
}

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// Alter Reality — {1}{U}. Swap a colour word on a spell or permanent.
pub fn alter_reality() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), u()]))],
        ..instant(
            "Alter Reality",
            cost(&[generic(1), u()]),
            Effect::ReplaceColorWord {
                what: target_filtered(R::Permanent.or(R::IsSpellOnStack)),
                duration: Duration::Permanent,
            },
        )
    }
}

/// Anurid Scavenger — {2}{G} 3/3 with protection from black that eats its own
/// graveyard each upkeep or dies.
pub fn anurid_scavenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        triggered_abilities: vec![upkeep(Effect::SacrificeSourceUnlessCost {
            cost: crate::card::WardCost::BottomFromGraveyard(1),
        })],
        ..creature(
            "Anurid Scavenger",
            cost(&[generic(2), g()]),
            vec![CreatureType::Frog, CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Crackling Club — {R} Aura. +1/+0, or cash it in for a ping.
pub fn crackling_club() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus { power: 1, ..Default::default() }),
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..enchantment("Crackling Club", cost(&[r()]))
    }
}

/// Crazed Firecat — {5}{R}{R} 4/4. Flip until you lose; grow by the wins.
pub fn crazed_firecat() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::FlipUntilLoss {
            per_win: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..creature(
            "Crazed Firecat",
            cost(&[generic(5), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Cat],
            4,
            4,
        )
    }
}

/// Crippling Fatigue — {1}{B}{B}. −2/−2, twice if you pay the life.
pub fn crippling_fatigue() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), b()]))],
        flashback_additional_cost: vec![crate::card::AdditionalCastCost::PayLife { amount: 3 }],
        ..sorcery(
            "Crippling Fatigue",
            cost(&[generic(1), b(), b()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Dwell on the Past — {G}. Shuffle up to four graveyard cards back in.
pub fn dwell_on_the_past() -> CardDefinition {
    sorcery(
        "Dwell on the Past",
        cost(&[g()]),
        Effect::ApplyToTargets {
            max_targets: 4,
            min_targets: 0,
            filter: R::InGraveyard,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: crate::effect::LibraryPosition::Shuffled,
                },
            }),
        },
    )
}

/// Enslaved Dwarf — {R} 1/1. Sacrifice to pump a black creature.
pub fn enslaved_dwarf() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::HasColor(Color::Black))),
                    power: Value::Const(1),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Enslaved Dwarf", cost(&[r()]), vec![CreatureType::Dwarf], 1, 1)
    }
}

/// Faceless Butcher — {2}{B}{B} 2/3. Jails another creature while it lives.
pub fn faceless_butcher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Creature.and(R::OtherThanSource)),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        ..creature(
            "Faceless Butcher",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Nightmare, CreatureType::Horror],
            2,
            3,
        )
    }
}

/// Far Wanderings — {2}{G}. A basic land, or three past Threshold.
pub fn far_wanderings() -> CardDefinition {
    let fetch = |n: i32| Effect::SearchUpToN {
        who: PlayerRef::You,
        filter: R::Land.and(R::IsBasicLand),
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        count: Value::Const(n),
    };
    sorcery(
        "Far Wanderings",
        cost(&[generic(2), g()]),
        Effect::If {
            cond: threshold(),
            then: Box::new(fetch(3)),
            else_: Box::new(fetch(1)),
        },
    )
}

/// Flash of Defiance — {1}{R}. Green and white creatures can't block.
pub fn flash_of_defiance() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), r()]))],
        flashback_additional_cost: vec![crate::card::AdditionalCastCost::PayLife { amount: 3 }],
        ..sorcery(
            "Flash of Defiance",
            cost(&[generic(1), r()]),
            Effect::GrantKeywordToMatchingThisTurn {
                filter: R::Creature.and(R::HasColor(Color::Green).or(R::HasColor(Color::White))),
                keyword: Keyword::CantBlock,
            },
        )
    }
}

/// Frantic Purification — {2}{W}. Naturalize for enchantments, with Madness.
pub fn frantic_purification() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Madness(cost(&[w()]))],
        ..instant(
            "Frantic Purification",
            cost(&[generic(2), w()]),
            Effect::Destroy { what: target_filtered(R::Enchantment) },
        )
    }
}

/// Ghostly Wings — {1}{U} Aura. +1/+1 and flying; discard to bounce the host.
pub fn ghostly_wings() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Move {
                what: Selector::attached_to(Selector::This),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..enchantment("Ghostly Wings", cost(&[generic(1), u()]))
    }
}

/// Gravegouger — {2}{B} 2/2. Holds two graveyard cards hostage.
pub fn gravegouger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::InGraveyard,
            effect: Box::new(Effect::ExileUntilSourceLeaves {
                what: Selector::Target(0),
                return_to: crate::card::ExileReturnZone::Graveyard,
            }),
        })],
        ..creature(
            "Gravegouger",
            cost(&[generic(2), b()]),
            vec![CreatureType::Nightmare, CreatureType::Horror],
            2,
            2,
        )
    }
}

/// Grotesque Hybrid — {4}{B} 3/3. Its combat damage kills; discard to evade.
pub fn grotesque_hybrid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::Destroy { what: Selector::TriggerSource },
        }],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Protection(Color::Green),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Protection(Color::White),
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Grotesque Hybrid", cost(&[generic(4), b()]), vec![CreatureType::Zombie], 3, 3)
    }
}

/// Hell-Bent Raider — {1}{R}{R} 2/2 first strike, haste. Random discard buys
/// protection from white.
pub fn hell_bent_raider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            discard_cost_random: true,
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Protection(Color::White),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Hell-Bent Raider",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian],
            2,
            2,
        )
    }
}

/// The Hydromorph pair — {U}, sacrifice: counter a spell aimed at your side.
fn hydromorph(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        keywords,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::CounterSpell {
                what: target_filtered(
                    R::IsSpellOnStack.and(R::SpellTargetsControllerOrControlled),
                ),
            },
            ..Default::default()
        }],
        ..creature(name, c, types, p, t)
    }
}

/// Hydromorph Guardian — {2}{U} 2/2.
pub fn hydromorph_guardian() -> CardDefinition {
    hydromorph(
        "Hydromorph Guardian",
        cost(&[generic(2), u()]),
        vec![CreatureType::Elemental],
        2,
        2,
        vec![],
    )
}

/// Hydromorph Gull — {3}{U}{U} 3/3 flier.
pub fn hydromorph_gull() -> CardDefinition {
    hydromorph(
        "Hydromorph Gull",
        cost(&[generic(3), u(), u()]),
        vec![CreatureType::Elemental, CreatureType::Bird],
        3,
        3,
        vec![Keyword::Flying],
    )
}
