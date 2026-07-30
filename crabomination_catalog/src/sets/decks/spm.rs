//! Marvel's Spider-Man (SPM) — Standard-legal staples on existing primitives.
//! Spiders-matter aggro (white/green), Villain value (blue/black), and a
//! handful of red burn/tempo cards. Tests in `crabomination/src/tests/spm.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{each_your_creature, etb, on_attack, on_dies, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Selector, StaticEffect, ZoneDest,
};
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w, x};

/// Aunt May — {W} 0/2 Human Citizen. Whenever another creature you control
/// enters, you gain 1 life; if it's a Spider, put a +1/+1 counter on it.
pub fn aunt_may() -> CardDefinition {
    CardDefinition {
        name: "Aunt May",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Spider),
                    },
                    then: Box::new(Effect::AddCounter {
                        what: Selector::TriggerSource,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// City Pigeon — {W} 1/1 Bird. Flying. When it leaves the battlefield, create
/// a Food token.
pub fn city_pigeon() -> CardDefinition {
    CardDefinition {
        name: "City Pigeon",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentLeavesBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::food_token(),
            },
        }],
        ..Default::default()
    }
}

/// Gallant Citizen — {G/W}{G/W} 1/1. When it enters, draw a card.
pub fn gallant_citizen() -> CardDefinition {
    CardDefinition {
        name: "Gallant Citizen",
        cost: cost(&[
            hybrid(Color::Green, Color::White),
            hybrid(Color::Green, Color::White),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Common Crook — {1}{B} 2/2 Human Rogue Villain. When it dies, create a
/// Treasure token.
pub fn common_crook() -> CardDefinition {
    CardDefinition {
        name: "Common Crook",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Rogue,
                CreatureType::Villain,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crate::game::effects::treasure_token(),
        })],
        ..Default::default()
    }
}

/// Kraven's Cats — {1}{G} 2/2 Cat Villain. {2}{G}: this creature gets +2/+2
/// until end of turn. Activate only once each turn.
pub fn kravens_cats() -> CardDefinition {
    CardDefinition {
        name: "Kraven's Cats",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Villain],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lurking Lizards — {1}{G} 1/3 Lizard Villain. Trample. Whenever you cast a
/// spell with mana value 4 or greater, put a +1/+1 counter on this creature.
pub fn lurking_lizards() -> CardDefinition {
    CardDefinition {
        name: "Lurking Lizards",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Villain],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::ManaValueAtLeast(4))),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Angry Rabble — {1}{R} 2/2 Human Citizen. Trample. Whenever you cast a spell
/// with mana value 4+, deal 1 damage to each opponent. {5}{R} (sorcery): put
/// two +1/+1 counters on this creature.
pub fn angry_rabble() -> CardDefinition {
    CardDefinition {
        name: "Angry Rabble",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::ManaValueAtLeast(4))),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), r()]),
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Merciless Enforcers — {1}{B} 2/1 Human Mercenary Villain. Lifelink. {3}{B}:
/// this creature deals 1 damage to each opponent.
pub fn merciless_enforcers() -> CardDefinition {
    CardDefinition {
        name: "Merciless Enforcers",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Mercenary,
                CreatureType::Villain,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mysterio's Phantasm — {1}{U} 1/3 Illusion Villain. Flying, vigilance.
/// Whenever it attacks, mill a card.
pub fn mysterios_phantasm() -> CardDefinition {
    CardDefinition {
        name: "Mysterio's Phantasm",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Illusion, CreatureType::Villain],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![on_attack(Effect::Mill {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Scorpion's Sting — {1}{B} Instant. Target creature gets -3/-3 until end of
/// turn.
pub fn scorpions_sting() -> CardDefinition {
    CardDefinition {
        name: "Scorpion's Sting",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Thwip! — {W} Instant. Target creature gets +2/+2 and gains flying until end
/// of turn. If it's a Spider, you gain 2 life.
pub fn thwip() -> CardDefinition {
    CardDefinition {
        name: "Thwip!",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasCreatureType(CreatureType::Spider),
                },
                then: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Grow Extra Arms — {1}{G} Instant. Costs {1} less to cast if it targets a
/// Spider. Target creature gets +4/+4 until end of turn.
pub fn grow_extra_arms() -> CardDefinition {
    CardDefinition {
        name: "Grow Extra Arms",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((R::HasCreatureType(CreatureType::Spider), 1)),
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(4),
            toughness: Value::Const(4),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Romantic Rendezvous — {1}{R} Sorcery. Discard a card, then draw two cards.
pub fn romantic_rendezvous() -> CardDefinition {
    CardDefinition {
        name: "Romantic Rendezvous",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
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

/// Pumpkin Bombardment — {B/R} Sorcery. Deals 3 damage to target creature.
/// ("Discard a card or pay {2}" additional cost modeled as discard a card.)
pub fn pumpkin_bombardment() -> CardDefinition {
    CardDefinition {
        name: "Pumpkin Bombardment",
        cost: cost(&[hybrid(Color::Black, Color::Red)]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::Discard {
            count: 1,
            filter: None,
        }],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Spider-Bot — {2} 2/1 Spider Robot Scout artifact. Reach. When it enters,
/// search your library for a basic land card and put it on top.
pub fn spider_bot() -> CardDefinition {
    CardDefinition {
        name: "Spider-Bot",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spider,
                CreatureType::Robot,
                CreatureType::Scout,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: LibraryPosition::Top,
            },
            count: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Morlun, Devourer of Spiders — {X}{B}{B} 2/1 Vampire Villain. Lifelink.
/// Enters with X +1/+1 counters. When it enters, deals X damage to target
/// opponent. (Modeled as X damage to each opponent — one in a duel.)
pub fn morlun_devourer_of_spiders() -> CardDefinition {
    CardDefinition {
        name: "Morlun, Devourer of Spiders",
        cost: cost(&[x(), b(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Villain],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::XFromCost,
        })],
        ..Default::default()
    }
}

/// Selfless Police Captain — {1}{W} 1/1 Human Detective. Enters with a +1/+1
/// counter. When it leaves the battlefield, put a +1/+1 counter on target
/// creature you control. (Printed "its counters" modeled as the one it enters
/// with; a growing count is approximated as a single counter.)
pub fn selfless_police_captain() -> CardDefinition {
    CardDefinition {
        name: "Selfless Police Captain",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentLeavesBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Mob Lookout — {1}{U/B} 0/3 Human Rogue Villain. When it enters, target
/// creature you control connives.
pub fn mob_lookout() -> CardDefinition {
    CardDefinition {
        name: "Mob Lookout",
        cost: cost(&[generic(1), hybrid(Color::Blue, Color::Black)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Rogue,
                CreatureType::Villain,
            ],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            },
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::DiscardedThisResolution { filter: R::Nonland }),
            },
        ]))],
        ..Default::default()
    }
}

/// Radioactive Spider — {G} 1/1 Spider. Reach, deathtouch. {2}, Sacrifice
/// this creature (sorcery): search your library for a Spider Hero card and put
/// it into your hand.
pub fn radioactive_spider() -> CardDefinition {
    CardDefinition {
        name: "Radioactive Spider",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Reach, Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::HasCreatureType(CreatureType::Spider)
                    .and(R::HasCreatureType(CreatureType::Hero)),
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spider-Suit — {1} Equipment. Equipped creature gets +2/+2 and is a Spider
/// Hero in addition to its other types. Equip {3}.
pub fn spider_suit() -> CardDefinition {
    CardDefinition {
        name: "Spider-Suit",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            add_creature_types: vec![CreatureType::Spider, CreatureType::Hero],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Doc Ock's Tentacles — {1} Equipment. Whenever a creature you control with
/// mana value 5 or greater enters, you may attach this to it. Equipped
/// creature gets +4/+4. Equip {5}.
pub fn doc_ocks_tentacles() -> CardDefinition {
    CardDefinition {
        name: "Doc Ock's Tentacles",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(5)]))],
        equipped_bonus: Some(EquipBonus {
            power: 4,
            toughness: 4,
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::ManaValueAtLeast(5)),
                }),
            effect: Effect::MayDo {
                description: "Attach Doc Ock's Tentacles to it".into(),
                body: Box::new(Effect::Attach {
                    what: Selector::This,
                    to: Selector::TriggerSource,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Mary Jane Watson — {1}{G/W} 2/2 Human Performer. Whenever a Spider you
/// control enters, draw a card. Triggers only once each turn.
pub fn mary_jane_watson() -> CardDefinition {
    CardDefinition {
        name: "Mary Jane Watson",
        cost: cost(&[generic(1), hybrid(Color::Green, Color::White)]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Performer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Spider),
                })
                .once_per_turn(),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Spider-Girl, Legacy Hero — {G}{W} 2/2 Spider Human Hero. Has flying during
/// your turn. When it leaves the battlefield, create a 1/1 green-and-white
/// Human Citizen creature token.
pub fn spider_girl_legacy_hero() -> CardDefinition {
    CardDefinition {
        name: "Spider-Girl, Legacy Hero",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spider,
                CreatureType::Human,
                CreatureType::Hero,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "During your turn, Spider-Girl has flying.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Flying,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentLeavesBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Human Citizen".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green, Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Human, CreatureType::Citizen],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Spider-Ham, Peter Porker — {1}{G} 2/2 Spider Boar Hero. When it enters,
/// create a Food token. (The "Animal May-Ham" menagerie anthem is omitted.)
pub fn spider_ham_peter_porker() -> CardDefinition {
    CardDefinition {
        name: "Spider-Ham, Peter Porker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Boar, CreatureType::Hero],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crabomination_base::tokens::food_token(),
        })],
        ..Default::default()
    }
}

/// Vibrant Cityscape — Land. {T}, Sacrifice this land: search your library for
/// a basic land card, put it onto the battlefield tapped, then shuffle.
pub fn vibrant_cityscape() -> CardDefinition {
    CardDefinition {
        name: "Vibrant Cityscape",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
                count: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Flying Octobot — {1}{U} 1/1 Robot Villain artifact. Flying. Whenever another
/// Villain you control enters, put a +1/+1 counter on it (once each turn).
pub fn flying_octobot() -> CardDefinition {
    CardDefinition {
        name: "Flying Octobot",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Villain],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Villain).and(R::OtherThanSource),
                })
                .once_per_turn(),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Hobgoblin, Mantled Marauder — {1}{R} 1/2 Goblin Human Villain. Flying,
/// haste. Whenever you discard a card, Hobgoblin gets +2/+0 until end of turn.
pub fn hobgoblin_mantled_marauder() -> CardDefinition {
    CardDefinition {
        name: "Hobgoblin, Mantled Marauder",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Goblin,
                CreatureType::Human,
                CreatureType::Villain,
            ],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Skyward Spider — {W/U}{W/U} 2/2 Spider Human Hero. Ward {2}. Has flying as
/// long as it's modified.
pub fn skyward_spider() -> CardDefinition {
    CardDefinition {
        name: "Skyward Spider",
        cost: cost(&[
            hybrid(Color::White, Color::Blue),
            hybrid(Color::White, Color::Blue),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spider,
                CreatureType::Human,
                CreatureType::Hero,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        static_abilities: vec![StaticAbility {
            description: "Skyward Spider has flying as long as it's modified.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Flying,
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsModified,
                },
            },
        }],
        ..Default::default()
    }
}

/// Costume Closet — {1}{W} Artifact. Enters with two +1/+1 counters. {T}
/// (sorcery): move a +1/+1 counter from this onto target creature you control.
/// Whenever a modified creature you control leaves, put a +1/+1 counter on this.
pub fn costume_closet() -> CardDefinition {
    CardDefinition {
        name: "Costume Closet",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::MoveCounters {
                from: Selector::This,
                to: target_filtered(R::Creature.and(R::ControlledByYou)),
                counter: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PermanentLeavesBattlefield,
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::IsModified),
            }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Eerie Gravestone — {2} Artifact. ETB: draw a card. {1}{B}, Sacrifice this:
/// mill four, you may put a creature card from among them into your hand.
pub fn eerie_gravestone() -> CardDefinition {
    CardDefinition {
        name: "Eerie Gravestone",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_cost: true,
            effect: Effect::MillThenToHandN {
                amount: Value::Const(4),
                filter: R::Creature,
                take: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spectacular Tactics — {1}{W} Instant. Choose one — put a +1/+1 counter on a
/// creature you control and give it hexproof; or destroy a creature with power
/// 4 or greater.
pub fn spectacular_tactics() -> CardDefinition {
    CardDefinition {
        name: "Spectacular Tactics",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Hexproof,
                    duration: Duration::EndOfTurn,
                },
            ]),
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::PowerAtLeast(4))),
            },
        ]),
        ..Default::default()
    }
}

/// Spectacular Spider-Man — {1}{W} 3/2 Spider Human Hero. Flash. {1}: gains
/// flying until end of turn. {1}, Sacrifice this: creatures you control gain
/// hexproof and indestructible until end of turn.
pub fn spectacular_spider_man() -> CardDefinition {
    CardDefinition {
        name: "Spectacular Spider-Man",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spider,
                CreatureType::Human,
                CreatureType::Hero,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: each_your_creature(),
                        keyword: Keyword::Hexproof,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: each_your_creature(),
                        keyword: Keyword::Indestructible,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Scout the City — {1}{G} Sorcery. Choose one — mill three, you may take a
/// permanent card from among them, and gain 3 life; or destroy a creature with
/// flying.
pub fn scout_the_city() -> CardDefinition {
    CardDefinition {
        name: "Scout the City",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::MillThenToHandN {
                    amount: Value::Const(3),
                    filter: R::Permanent,
                    take: Value::Const(1),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
            ]),
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            },
        ]),
        ..Default::default()
    }
}

/// A 1/1 green-and-white Human Citizen creature token.
fn human_citizen_token() -> TokenDefinition {
    TokenDefinition {
        name: "Human Citizen".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// News Helicopter — {3} 1/1 Construct artifact. Flying. ETB: create a 1/1
/// green-and-white Human Citizen token.
pub fn news_helicopter() -> CardDefinition {
    CardDefinition {
        name: "News Helicopter",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: human_citizen_token(),
        })],
        ..Default::default()
    }
}

/// Spider-Byte, Web Warden — {2}{U} 2/2 Spider Avatar Hero. ETB: return up to
/// one target nonland permanent to its owner's hand.
pub fn spider_byte_web_warden() -> CardDefinition {
    CardDefinition {
        name: "Spider-Byte, Web Warden",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spider,
                CreatureType::Avatar,
                CreatureType::Hero,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Permanent.and(R::Nonland),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(
                    0,
                )))),
            }),
        })],
        ..Default::default()
    }
}

/// Web Up — {2}{W} Enchantment. ETB: exile target nonland permanent an opponent
/// controls until this enchantment leaves the battlefield.
pub fn web_up() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Web Up",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Taxi Driver — {1}{R} 3/1 Human Pilot. {1}, {T}: target creature gains haste
/// until end of turn.
pub fn taxi_driver() -> CardDefinition {
    CardDefinition {
        name: "Taxi Driver",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Web-Warriors — {4}{G/W} 4/3 Spider Hero. ETB: put a +1/+1 counter on each
/// other creature you control.
pub fn web_warriors() -> CardDefinition {
    CardDefinition {
        name: "Web-Warriors",
        cost: cost(&[generic(4), hybrid(Color::Green, Color::White)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Hero],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Starling, Aerial Ally — {4}{W} 3/4 Human Hero. Flying. ETB: another target
/// creature you control gains flying until end of turn.
pub fn starling_aerial_ally() -> CardDefinition {
    CardDefinition {
        name: "Starling, Aerial Ally",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Hero],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Ezekiel Sims, Spider-Totem — {4}{G} 3/5 Spider Human Advisor. Reach. At the
/// beginning of combat on your turn, target Spider you control gets +2/+2 until
/// end of turn.
pub fn ezekiel_sims_spider_totem() -> CardDefinition {
    CardDefinition {
        name: "Ezekiel Sims, Spider-Totem",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spider,
                CreatureType::Human,
                CreatureType::Advisor,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::PumpPT {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Spider).and(R::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Agent Venom — {2}{B} 2/3 Symbiote Soldier Hero. Flash, menace. Whenever
/// another nontoken creature you control dies, draw a card and lose 1 life.
pub fn agent_venom() -> CardDefinition {
    CardDefinition {
        name: "Agent Venom",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Symbiote,
                CreatureType::Soldier,
                CreatureType::Hero,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken,
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                crate::effect::shortcut::lose_life(1, Selector::You),
            ]),
        }],
        ..Default::default()
    }
}
