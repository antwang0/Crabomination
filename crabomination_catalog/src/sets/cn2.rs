//! Conspiracy: Take the Crown (CN2) — the in-game half of the set: the
//! monarch shell, melee, goad, monstrosity and the council's dilemma votes.
//! The draft-matters cards ride the CR 905.2b shell and aren't here.
//! Tests in `classic_sets/cn2`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, VoteOption, VoteTally,
    shortcut::{draw, etb, on_attack, target_filtered},
};
use crate::game::TurnStep;
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

/// "When this creature enters, you become the monarch." (CR 725.3)
fn etb_monarch() -> TriggeredAbility {
    etb(Effect::BecomeMonarch { who: PlayerRef::You })
}

/// `{cost}: Monstrosity n` (CR 701.31), sorcery-speed like every printing.
fn monstrosity(c: ManaCost, n: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        effect: Effect::Monstrosity { n: Value::Const(n) },
        sorcery_speed: true,
        ..Default::default()
    }
}

/// "As long as this creature is monstrous, it has [keyword]."
fn monstrous_keyword(description: &'static str, keyword: Keyword) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::PumpSelfIf {
            condition: Predicate::SourceIsMonstrous,
            power: 0,
            toughness: 0,
            keywords: vec![keyword],
        },
    }
}

/// "Whenever this creature attacks, you may goad target creature defending
/// player controls." (CR 701.39)
fn on_attack_goad() -> TriggeredAbility {
    on_attack(Effect::MayDo {
        description: "Goad target creature defending player controls".to_string(),
        body: Box::new(Effect::Goad {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        }),
    })
}

// ── Monarch ───────────────────────────────────────────────────────────────

/// Ballot Broker — {2}{W} 2/3 Human Advisor that votes twice (CR 701.38).
pub fn ballot_broker() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "While voting, you may vote an additional time.",
            effect: StaticEffect::AdditionalVotes(1),
        }],
        ..creature(
            "Ballot Broker",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            2,
            3,
        )
    }
}

/// Crown-Hunter Hireling — {4}{R} 4/4 Ogre Mercenary that crowns you, then can
/// only attack the monarch.
pub fn crown_hunter_hireling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackUnlessDefenderIsMonarch],
        triggered_abilities: vec![etb_monarch()],
        ..creature(
            "Crown-Hunter Hireling",
            cost(&[generic(4), r()]),
            vec![CreatureType::Ogre, CreatureType::Mercenary],
            4,
            4,
        )
    }
}

/// Garrulous Sycophant — {2}{B} 1/4 Human Advisor that drains 1 at your end
/// step while you wear the crown.
pub fn garrulous_sycophant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::IsMonarch { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..creature(
            "Garrulous Sycophant",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            4,
        )
    }
}

/// Knights of the Black Rose — {3}{W}{B} 4/4 that crowns you, then punishes
/// whoever takes the crown off you.
pub fn knights_of_the_black_rose() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb_monarch(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameMonarch, EventScope::OpponentControl)
                    .with_filter(Predicate::WasMonarchAtTurnStart { who: PlayerRef::You }),
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::Const(2),
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            },
        ],
        ..creature(
            "Knights of the Black Rose",
            cost(&[generic(3), w(), b()]),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Protector of the Crown — {5}{W} 2/5 Giant Soldier that crowns you and eats
/// every point of damage aimed at you (CR 614.9).
pub fn protector_of_the_crown() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_monarch()],
        static_abilities: vec![StaticAbility {
            description: "All damage that would be dealt to you is dealt to this creature instead.",
            effect: StaticEffect::RedirectDamageToSelf,
        }],
        ..creature(
            "Protector of the Crown",
            cost(&[generic(5), w()]),
            vec![CreatureType::Giant, CreatureType::Soldier],
            2,
            5,
        )
    }
}

/// Queen Marchesa — {1}{R}{W}{B} 3/3 deathtouch haste; crowns you on entry and
/// mints an Assassin each upkeep the crown sits elsewhere.
pub fn queen_marchesa() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Deathtouch, Keyword::Haste],
        triggered_abilities: vec![
            etb_monarch(),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::IsMonarch { who: PlayerRef::EachOpponent }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Assassin".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: vec![Color::Black],
                        card_types: vec![CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Assassin],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Deathtouch, Keyword::Haste],
                        ..Default::default()
                    },
                },
            },
        ],
        ..creature(
            "Queen Marchesa",
            cost(&[generic(1), r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            3,
            3,
        )
    }
}

/// Throne of the High City — a colorless land that buys the crown outright.
pub fn throne_of_the_high_city() -> CardDefinition {
    CardDefinition {
        name: "Throne of the High City",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::BecomeMonarch { who: PlayerRef::You },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Melee ─────────────────────────────────────────────────────────────────

/// Wings of the Guard — {1}{W} 1/1 Bird with flying and melee.
pub fn wings_of_the_guard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Melee],
        ..creature("Wings of the Guard", cost(&[generic(1), w()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Deputized Protester — {2}{R} 2/1 Human Warrior with menace and melee.
pub fn deputized_protester() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Melee],
        ..creature(
            "Deputized Protester",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            1,
        )
    }
}

/// Custodi Soulcaller — {1}{W}{W} 1/2 with melee whose attack reanimates a
/// creature costing at most the number of players it attacked.
pub fn custodi_soulcaller() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Melee],
        triggered_abilities: vec![on_attack(Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature
                    .and(R::InYourGraveyard)
                    .and(R::ManaValueAtMostOpponentsAttackedThisCombat),
            },
            to: crate::effect::ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        })],
        ..creature(
            "Custodi Soulcaller",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Fang of the Pack — {5}{G} 5/3 Wolf with melee that lends melee to a friend
/// at each of your combats.
pub fn fang_of_the_pack() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Melee],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                keyword: Keyword::Melee,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Fang of the Pack", cost(&[generic(5), g()]), vec![CreatureType::Wolf], 5, 3)
    }
}

/// Grenzo's Ruffians — {2}{R}{R} 2/2 Goblin with melee that splashes its combat
/// damage onto every other opponent.
pub fn grenzos_ruffians() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Melee],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponentExceptTriggerer),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature("Grenzo's Ruffians", cost(&[generic(2), r(), r()]), vec![CreatureType::Goblin], 2, 2)
    }
}

// ── Goad ──────────────────────────────────────────────────────────────────

/// Coveted Peacock — {3}{U}{U} 3/4 flier that goads a blocker away each attack.
pub fn coveted_peacock() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack_goad()],
        ..creature("Coveted Peacock", cost(&[generic(3), u(), u()]), vec![CreatureType::Bird], 3, 4)
    }
}

/// Goblin Racketeer — {3}{R} 4/2 Goblin Rogue with the same attack goad.
pub fn goblin_racketeer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack_goad()],
        ..creature(
            "Goblin Racketeer",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Rogue],
            4,
            2,
        )
    }
}

/// Jeering Homunculus — {1}{U} 0/4 that goads on the way in.
pub fn jeering_homunculus() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Goad target creature".to_string(),
            body: Box::new(Effect::Goad { what: target_filtered(R::Creature) }),
        })],
        ..creature(
            "Jeering Homunculus",
            cost(&[generic(1), u()]),
            vec![CreatureType::Homunculus],
            0,
            4,
        )
    }
}

/// Besmirch — {1}{R}{R} sorcery: borrow a creature for the turn, then goad it
/// so it can't come straight back at you.
pub fn besmirch() -> CardDefinition {
    CardDefinition {
        name: "Besmirch",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::Goad { what: Selector::Target(0) },
        ]),
        ..Default::default()
    }
}

/// Grenzo, Havoc Raiser — {R}{R} 2/2 whose connections either goad a blocker or
/// steal the top card of the damaged player's library.
pub fn grenzo_havoc_raiser() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::ChooseMode(vec![
                Effect::Goad {
                    what: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)),
                },
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::TriggerEventPlayer,
                    count: Value::ONE,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: true,
                    max_mana_value: None,
                    pay_own_cost: false,
                    uncast_penalty: None,
                },
            ]),
        }],
        ..creature(
            "Grenzo, Havoc Raiser",
            cost(&[r(), r()]),
            vec![CreatureType::Goblin, CreatureType::Rogue],
            2,
            2,
        )
    }
}

// ── Monstrosity ───────────────────────────────────────────────────────────

/// Domesticated Hydra — {2}{G}{G} 3/3 with Monstrosity X and trample once
/// monstrous.
pub fn domesticated_hydra() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), g(), g(), g()]),
            effect: Effect::Monstrosity { n: Value::XFromCost },
            sorcery_speed: true,
            ..Default::default()
        }],
        static_abilities: vec![monstrous_keyword(
            "As long as this creature is monstrous, it has trample.",
            Keyword::Trample,
        )],
        ..creature(
            "Domesticated Hydra",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Hydra],
            3,
            3,
        )
    }
}

/// Sinuous Vermin — {1}{B} 2/2 Rat Horror: Monstrosity 3, menace once monstrous.
pub fn sinuous_vermin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(3), b(), b()]), 3)],
        static_abilities: vec![monstrous_keyword(
            "As long as this creature is monstrous, it has menace.",
            Keyword::Menace,
        )],
        ..creature(
            "Sinuous Vermin",
            cost(&[generic(1), b()]),
            vec![CreatureType::Rat, CreatureType::Horror],
            2,
            2,
        )
    }
}

/// Skittering Crustacean — {2}{U} 2/3 Crab: Monstrosity 4, hexproof once
/// monstrous.
pub fn skittering_crustacean() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(6), u()]), 4)],
        static_abilities: vec![monstrous_keyword(
            "As long as this creature is monstrous, it has hexproof.",
            Keyword::Hexproof,
        )],
        ..creature("Skittering Crustacean", cost(&[generic(2), u()]), vec![CreatureType::Crab], 2, 3)
    }
}

/// Splitting Slime — {3}{G}{G} 3/3 Ooze that clones itself when it goes
/// monstrous.
pub fn splitting_slime() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(4), g(), g()]), 3)],
        triggered_abilities: vec![crate::effect::shortcut::on_becomes_monstrous(
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
        )],
        ..creature("Splitting Slime", cost(&[generic(3), g(), g()]), vec![CreatureType::Ooze], 3, 3)
    }
}

// ── Council's dilemma & the rest ──────────────────────────────────────────

/// Orchard Elemental — {5}{G} 2/2 whose entry vote trades counters for life.
pub fn orchard_elemental() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Vote {
            options: vec![
                VoteOption {
                    label: "Sprout".to_string(),
                    effect: Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                },
                VoteOption {
                    label: "Harvest".to_string(),
                    effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                },
            ],
            tally: VoteTally::PerVote,
        })],
        ..creature("Orchard Elemental", cost(&[generic(5), g()]), vec![CreatureType::Elemental], 2, 2)
    }
}

/// Illusion of Choice — {U} instant: you answer every ballot this turn, then
/// draw (CR 701.38).
pub fn illusion_of_choice() -> CardDefinition {
    CardDefinition {
        name: "Illusion of Choice",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ControlVotesThisTurn { who: PlayerRef::You },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Deadly Designs — {1}{B} enchantment anyone can feed; at five plot counters
/// it blows up for two creatures.
pub fn deadly_designs() -> CardDefinition {
    CardDefinition {
        name: "Deadly Designs",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            any_player: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Plot,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        state_trigger: Some(crate::card::StateTriggeredAbility {
            condition: Predicate::SourceHasCountersAtLeast {
                counter: CounterType::Plot,
                n: 5,
            },
            effect: Effect::Seq(vec![
                Effect::SacrificeSource,
                Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                },
            ]),
        }),
        ..Default::default()
    }
}

/// Spectral Grasp — {1}{W} Aura that keeps the enchanted creature out of your
/// half of combat entirely. (The block half is a flat "can't block", exact at
/// two players.)
pub fn spectral_grasp() -> CardDefinition {
    CardDefinition {
        name: "Spectral Grasp",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::CantBlock], ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature can't attack you or planeswalkers you control.",
            effect: StaticEffect::CreaturesCantAttackController {
                protect_planeswalkers: true,
                filter: Some(R::IsHostOfSource),
            },
        }],
        ..Default::default()
    }
}

/// Selvala, Heart of the Wilds — {1}{G}{G} 2/3 Elf Scout: a draw for whoever
/// lands the biggest creature, and a mana ability the size of your board.
pub fn selvala_heart_of_the_wilds() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature
                        .and(R::OtherThanSource)
                        .and(R::HasGreatestPowerAmongAllCreatures),
                }),
            effect: Effect::MayDoBy {
                who: PlayerRef::Triggerer,
                description: "Draw a card".to_string(),
                body: Box::new(Effect::Draw {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::ONE,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColors(Value::GreatestPowerControlled { who: PlayerRef::You }),
            },
            ..Default::default()
        }],
        ..creature(
            "Selvala, Heart of the Wilds",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Scout],
            2,
            3,
        )
    }
}
