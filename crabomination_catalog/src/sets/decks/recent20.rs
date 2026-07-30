//! A twentieth staples wave — Outlaws of Thunder Junction (OTJ): the **commit
//! a crime** payoffs (`EventKind::CommittedCrime`, CR 700.13), **pack tactics**
//! (`Predicate::AttackedWithTotalPowerAtLeast`), and the **outlaw** type group
//! (`SelectionRequirement::IsOutlaw`). Tests in `crabomination/src/tests/recent20.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{deal, etb, on_attack, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::effects::treasure_token;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// A 1/1 red Mercenary token with a sorcery-speed tap pump (Lassoed by the
/// Law, Rakish Crew).
fn mercenary_token() -> TokenDefinition {
    TokenDefinition {
        name: "Mercenary".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mercenary],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// 2/2 blue-and-black Zombie Rogue (Gisa).
fn zombie_rogue_token(tapped: bool) -> TokenDefinition {
    TokenDefinition {
        name: "Zombie Rogue".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Rogue],
            ..Default::default()
        },
        tapped,
        ..Default::default()
    }
}

// ── Pack tactics ─────────────────────────────────────────────────────────────

/// Battle Cry Goblin — {1}{R} 2/2 Goblin. {1}{R}: Goblins you control get
/// +1/+0 and gain haste. Pack tactics — when it attacks, if you attacked with
/// total power 6+, make a 1/1 red Goblin tapped and attacking.
pub fn battle_cry_goblin() -> CardDefinition {
    let goblins = || {
        Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(CreatureType::Goblin)
                .and(SelectionRequirement::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Battle Cry Goblin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: goblins(),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: goblins(),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::AttackedWithTotalPowerAtLeast {
                    who: PlayerRef::You,
                    at_least: 6,
                },
            ),
            effect: Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Goblin".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Goblin],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                cleanup: Default::default(),
            },
        }],
        ..Default::default()
    }
}

// ── Commit a crime ───────────────────────────────────────────────────────────

/// Gisa, the Hellraiser — {3}{B}{B} 4/4 Legendary Human Warlock. Ward—{2},
/// Pay 2 life. Skeletons and Zombies you control get +1/+1 and have menace.
/// Whenever you commit a crime, create two tapped 2/2 Zombie Rogue tokens
/// (once each turn).
pub fn gisa_the_hellraiser() -> CardDefinition {
    let undead = || {
        Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(CreatureType::Skeleton)
                .or(SelectionRequirement::HasCreatureType(CreatureType::Zombie))
                .and(SelectionRequirement::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Gisa, the Hellraiser",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Ward(WardCost::ManaAndLife(cost(&[generic(2)]), 2))],
        static_abilities: vec![
            StaticAbility {
                description: "Skeletons and Zombies you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: undead(),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Skeletons and Zombies you control have menace.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: undead(),
                    keyword: Keyword::Menace,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: zombie_rogue_token(true),
            },
        }],
        ..Default::default()
    }
}

/// Magda, the Hoardmaster — {1}{R} 2/2 Legendary Dwarf Berserker. Whenever you
/// commit a crime, create a tapped Treasure (once each turn). Sacrifice three
/// Treasures (sorcery-speed): create a 4/4 red Scorpion Dragon with flying and
/// haste.
pub fn magda_the_hoardmaster() -> CardDefinition {
    CardDefinition {
        name: "Magda, the Hoardmaster",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    tapped: true,
                    ..treasure_token()
                },
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            sac_other_filter: Some((
                SelectionRequirement::HasArtifactSubtype(crate::card::ArtifactSubtype::Treasure),
                3,
            )),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Scorpion Dragon".into(),
                    power: 4,
                    toughness: 4,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    keywords: vec![Keyword::Flying, Keyword::Haste],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Scorpion, CreatureType::Dragon],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Marchesa, Dealer of Death — {U}{B}{R} 3/4 Legendary Human Rogue. Whenever
/// you commit a crime, you may pay {1}: look at the top two, one to hand and
/// the other to your graveyard.
pub fn marchesa_dealer_of_death() -> CardDefinition {
    CardDefinition {
        name: "Marchesa, Dealer of Death",
        cost: cost(&[u(), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1} to dig two (one to hand, one to graveyard)?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::LookPickToHand {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    rest_to_graveyard: true,
                    pick_filter: None,
                    take: Some(Value::Const(1)),
                    to_battlefield: false,
                    gain_life_if_pick: None,
                    gain_life_greatest_power_rest: false,
                    optional: false,
                    picked_lands_to_battlefield: false,
                    rest_bottom_random: false,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Forsaken Miner — {B} 2/2 Skeleton Rogue. Can't block. Whenever you commit a
/// crime, you may pay {B}: return this from your graveyard to the battlefield.
pub fn forsaken_miner() -> CardDefinition {
    CardDefinition {
        name: "Forsaken Miner",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::FromYourGraveyard),
            effect: Effect::MayPay {
                description: "Pay {B} to return Forsaken Miner from your graveyard?".into(),
                mana_cost: cost(&[b()]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Nimble Brigand — {2}{U} 1/3 Human Rogue. Can't be blocked if you've
/// committed a crime this turn. Whenever it deals combat damage to a player,
/// draw a card.
pub fn nimble_brigand() -> CardDefinition {
    CardDefinition {
        name: "Nimble Brigand",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Can't be blocked if you've committed a crime this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::CommittedCrimeThisTurn {
                    who: PlayerRef::You,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Unblockable],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Outlaw matters ───────────────────────────────────────────────────────────

/// Vial Smasher, Gleeful Grenadier — {B}{R} 3/2 Legendary Goblin Mercenary.
/// Whenever another outlaw you control enters, deal 1 damage to target opponent.
pub fn vial_smasher_gleeful_grenadier() -> CardDefinition {
    CardDefinition {
        name: "Vial Smasher, Gleeful Grenadier",
        cost: cost(&[b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::IsOutlaw,
                }),
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::OpponentPlayer),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Rakish Crew — {2}{B} Enchantment. ETB make a 1/1 red Mercenary. Whenever an
/// outlaw you control dies, each opponent loses 1 life and you gain 1.
pub fn rakish_crew() -> CardDefinition {
    CardDefinition {
        name: "Rakish Crew",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: mercenary_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::IsOutlaw,
                    }),
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(1),
                    },
                    Effect::GainLife {
                        who: Selector::Player(PlayerRef::You),
                        amount: Value::Const(1),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Hellspur Brute — {4}{R} 5/4 Minotaur Mercenary with trample and Affinity for
/// outlaws ({1} less per outlaw you control).
pub fn hellspur_brute() -> CardDefinition {
    CardDefinition {
        name: "Hellspur Brute",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        affinity_filter: Some(
            SelectionRequirement::IsOutlaw.and(SelectionRequirement::ControlledByYou),
        ),
        ..Default::default()
    }
}

// ── Plot / value ─────────────────────────────────────────────────────────────

/// Rictus Robber — {3}{B} 4/3 Zombie Rogue. ETB, if a creature died this turn,
/// make a 2/2 Zombie Rogue. Plot {2}{B}.
pub fn rictus_robber() -> CardDefinition {
    CardDefinition {
        name: "Rictus Robber",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        plot_cost: Some(cost(&[generic(2), b()])),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast {
                at_least: Value::Const(1),
            },
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: zombie_rogue_token(false),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Highway Robbery — {1}{R} Sorcery. Discard a card or sacrifice a land; if you
/// do, draw two. Plot {1}{R}.
pub fn highway_robbery() -> CardDefinition {
    CardDefinition {
        name: "Highway Robbery",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        plot_cost: Some(cost(&[generic(1), r()])),
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
                random: false,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Djinn of Fool's Fall — {4}{U} 4/3 Djinn with flying. Plot {3}{U}.
pub fn djinn_of_fools_fall() -> CardDefinition {
    CardDefinition {
        name: "Djinn of Fool's Fall",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        plot_cost: Some(cost(&[generic(3), u()])),
        ..Default::default()
    }
}

// ── Simple staples ───────────────────────────────────────────────────────────

/// Holy Cow — {2}{W} 2/2 Ox Angel with flash and flying. ETB gain 2 life,
/// scry 1.
pub fn holy_cow() -> CardDefinition {
    CardDefinition {
        name: "Holy Cow",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ox, CreatureType::Angel],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::You),
                amount: Value::Const(2),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Sterling Keykeeper — {1}{W} 2/2 Human Mercenary. {2}, {T}: tap target
/// non-Mount creature.
pub fn sterling_keykeeper() -> CardDefinition {
    CardDefinition {
        name: "Sterling Keykeeper",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature.and(
                    SelectionRequirement::Not(Box::new(SelectionRequirement::HasCreatureType(
                        CreatureType::Mount,
                    ))),
                )),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Treasure Dredger — {1}{B} 2/2 Human Rogue. {1}, {T}, Pay 1 life: create a
/// Treasure.
pub fn treasure_dredger() -> CardDefinition {
    CardDefinition {
        name: "Treasure Dredger",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            life_cost: 1,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Razzle-Dazzler — {1}{U} 1/2 Human Wizard. Whenever you cast your second
/// spell each turn, put a +1/+1 counter on it; it can't be blocked this turn.
pub fn razzle_dazzler() -> CardDefinition {
    CardDefinition {
        name: "Razzle-Dazzler",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Slick Sequence — {U}{R} Instant. Deal 2 to any target; if you've cast
/// another spell this turn, draw a card.
pub fn slick_sequence() -> CardDefinition {
    CardDefinition {
        name: "Slick Sequence",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            deal(2, target_any()),
            Effect::If {
                cond: Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Quilled Charger — {3}{R} 4/3 Porcupine Mount. Saddle 2. Whenever it attacks
/// while saddled, it gets +1/+2 and gains menace.
pub fn quilled_charger() -> CardDefinition {
    CardDefinition {
        name: "Quilled Charger",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Porcupine, CreatureType::Mount],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Saddle(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::SourceSaddled),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Lassoed by the Law — {3}{W} Enchantment. ETB exile a nonland permanent an
/// opponent controls until this leaves; ETB also make a 1/1 red Mercenary.
pub fn lassoed_by_the_law() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Lassoed by the Law",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::ExileUntilSourceLeaves {
                what: target_filtered(
                    SelectionRequirement::Nonland.and(SelectionRequirement::ControlledByOpponent),
                ),
                return_to: ExileReturnZone::Battlefield,
            }),
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: mercenary_token(),
            }),
        ],
        ..Default::default()
    }
}

/// Roxanne, Starfall Savant — {3}{R}{G} 4/3 Legendary Cat Druid. When she
/// enters or attacks, create a tapped Meteorite (ETB deal 2 to any target,
/// {T}: add one mana of any color).
pub fn roxanne_starfall_savant() -> CardDefinition {
    use crate::effect::ManaPayload;
    let meteorite = || TokenDefinition {
        name: "Meteorite".into(),
        card_types: vec![CardType::Artifact],
        tapped: true,
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_any(),
            amount: Value::Const(2),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    let make = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: meteorite(),
    };
    CardDefinition {
        name: "Roxanne, Starfall Savant",
        cost: cost(&[generic(3), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(make()), on_attack(make())],
        ..Default::default()
    }
}

/// Honest Rutstein — {1}{B}{G} 3/2 Legendary Human Warlock. ETB return a
/// creature card from your graveyard to hand. Creature spells you cast cost
/// {1} less.
pub fn honest_rutstein() -> CardDefinition {
    CardDefinition {
        name: "Honest Rutstein",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Creature spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::Creature,
                amount: 1,
            },
        }],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Stoic Sphinx — {2}{U}{U} 5/3 Sphinx with flash and flying. Has hexproof as
/// long as you haven't cast a spell this turn.
pub fn stoic_sphinx() -> CardDefinition {
    CardDefinition {
        name: "Stoic Sphinx",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Has hexproof as long as you haven't cast a spell this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::You,
                    count: Value::Const(0),
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Hexproof],
            },
        }],
        ..Default::default()
    }
}

/// Bovine Intervention — {1}{W} Instant. Destroy target artifact or creature;
/// its controller creates a 2/2 white Ox.
pub fn bovine_intervention() -> CardDefinition {
    CardDefinition {
        name: "Bovine Intervention",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Ox".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Ox],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                ),
            },
        ]),
        ..Default::default()
    }
}
