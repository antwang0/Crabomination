//! Commander: 2026-10-19 (`01DJHkWk`) — the Marvel Villains cluster (The
//! Masters of Evil, Crossbones, Citizen V, Crimson Cowl, …) that ~20 EDHREC
//! Villain decks share. Tests in `tests/recent_b/cmdr_01dj_a.rs`.

use super::cmdr_015x_a::{plan, plan_counter, plan_payoff};
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Zone,
};
use crate::effect::shortcut::{connive, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, u};
use std::sync::Arc;

fn villain() -> R {
    R::HasCreatureType(CreatureType::Villain)
}

fn hero() -> R {
    R::HasCreatureType(CreatureType::Hero)
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn my_villains() -> R {
    mine().and(villain())
}

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(def: CardDefinition) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..def
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

/// A 2/1 black Villain creature token with menace.
fn villain_token() -> TokenDefinition {
    TokenDefinition {
        name: "Villain".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Villain],
            ..Default::default()
        },
        keywords: vec![Keyword::Menace],
        ..Default::default()
    }
}

fn make_villains(count: Value, tapped: bool) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count,
        definition: Arc::new(TokenDefinition {
            tapped,
            ..villain_token()
        }),
    }
}

/// "Whenever a Villain you control dies, …"
fn on_villain_dies(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: villain(),
            },
        ),
        effect,
    }
}

fn each_opponent() -> Selector {
    Selector::Player(PlayerRef::EachOpponent)
}

/// The Masters of Evil — {5}{B} legendary 5/6 Human Villain. Other Villains
/// you control get +2/+1. {1}{B}, discard it: search your library for a Plan
/// card, reveal it, put it into your hand.
pub fn the_masters_of_evil() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Villains you control get +2/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(my_villains().and(R::OtherThanSource)),
                power: 2,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Plan),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..legend(creature(
            "The Masters of Evil",
            cost(&[generic(5), b()]),
            vec![CreatureType::Human, CreatureType::Villain],
            5,
            6,
        ))
    }
}

/// Madame Masque — {4}{B} legendary 3/2 Human Villain. She connives as she
/// enters; your second draw each turn makes a 2/1 menace Villain.
pub fn madame_masque() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(connive(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
                effect: make_villains(Value::ONE, false),
            },
        ],
        ..legend(creature(
            "Madame Masque",
            cost(&[generic(4), b()]),
            vec![CreatureType::Human, CreatureType::Villain],
            3,
            2,
        ))
    }
}

/// Crossbones, Malicious Mercenary — {3}{B} legendary 3/3 deathtouch. Another
/// Villain of yours entering puts a +1/+1 counter on him and he deals 2
/// damage to each opponent, once each turn.
pub fn crossbones_malicious_mercenary() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(villain()),
                })
                .once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::DealDamage {
                    to: each_opponent(),
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..legend(creature(
            "Crossbones, Malicious Mercenary",
            cost(&[generic(3), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Mercenary,
                CreatureType::Villain,
            ],
            3,
            3,
        ))
    }
}

/// Citizen V, Helmut Zemo — {2}{B} legendary 2/2 lifelink. Whenever you gain
/// life, a +1/+1 counter on each Villain you control.
pub fn citizen_v_helmut_zemo() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(my_villains()),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..legend(creature(
            "Citizen V, Helmut Zemo",
            cost(&[generic(2), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Noble,
                CreatureType::Villain,
            ],
            2,
            2,
        ))
    }
}

/// Crimson Cowl, Master of Evil — {2}{B} legendary 1/3. Whenever one or more
/// nontoken Villains you control attack a player, a 2/1 menace Villain.
pub fn crimson_cowl_master_of_evil() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: villain().and(R::NotToken).and(R::IsAttackingOpponentPlayer),
                })
                .once_per_batch_per_defender(),
            effect: make_villains(Value::ONE, false),
        }],
        ..legend(creature(
            "Crimson Cowl, Master of Evil",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Villain],
            1,
            3,
        ))
    }
}

/// Thunderbolts Conspiracy — {3}{B} flash enchantment. A Villain of yours
/// dying returns under its owner's control with a finality counter, a Hero in
/// addition to its other types.
pub fn thunderbolts_conspiracy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![on_villain_dies(Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::SpellEntersWithCounters { what: Selector::TriggerSource, kind: CounterType::Finality, amount: Value::ONE },
            Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                    tapped: false,
                },
            },
            Effect::AddCreatureTypes {
                what: Selector::LastMoved,
                creature_types: vec![CreatureType::Hero],
                duration: Duration::Permanent,
            },
        ]))],
        ..spell(
            "Thunderbolts Conspiracy",
            cost(&[generic(3), b()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

/// Scorpion, Seething Striker — {3}{B} legendary 3/3 deathtouch. At the
/// beginning of your end step, if a creature died this turn, target creature
/// you control connives.
pub fn scorpion_seething_striker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast {
                at_least: Value::ONE,
            }),
            effect: Effect::Connive {
                what: target_filtered(mine()),
                amount: Value::ONE,
            },
        }],
        ..legend(creature(
            "Scorpion, Seething Striker",
            cost(&[generic(3), b()]),
            vec![
                CreatureType::Scorpion,
                CreatureType::Human,
                CreatureType::Villain,
            ],
            3,
            3,
        ))
    }
}

/// HYDRA Troopers — {2}{B} 3/2 Human Soldier Villain. Enters: a tapped 2/1
/// menace Villain if two or more creature cards are in your graveyard;
/// otherwise mill two.
pub fn hydra_troopers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::Creature,
                },
                n: Value::Const(2),
            },
            then: Box::new(make_villains(Value::ONE, true)),
            else_: Box::new(Effect::Mill {
                who: Selector::You,
                amount: Value::Const(2),
            }),
        })],
        ..creature(
            "HYDRA Troopers",
            cost(&[generic(2), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Villain,
            ],
            3,
            2,
        )
    }
}

/// Venom, Eddie Brock — {4}{B}{B} legendary 6/4 menace Symbiote Villain.
/// Another creature dying puts a +1/+1 counter on him; a Villain also draws
/// you a card.
pub fn venom_eddie_brock() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::OtherThanSource,
                },
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: villain(),
                    },
                    then: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legend(creature(
            "Venom, Eddie Brock",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Symbiote, CreatureType::Villain],
            6,
            4,
        ))
    }
}

/// Red Room Recruit — {1}{B} 1/2 Human Spy Villain; connives as it enters.
pub fn red_room_recruit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(connive(1))],
        ..creature(
            "Red Room Recruit",
            cost(&[generic(1), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Spy,
                CreatureType::Villain,
            ],
            1,
            2,
        )
    }
}

/// Criminal Enterprise — {2}{B} enchantment. Enters: a 2/1 menace Villain. A
/// Villain of yours dying deals 1 damage to each opponent and gains you 1.
pub fn criminal_enterprise() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(make_villains(Value::ONE, false)),
            on_villain_dies(Effect::Seq(vec![
                Effect::DealDamage {
                    to: each_opponent(),
                    amount: Value::ONE,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ])),
        ],
        ..spell(
            "Criminal Enterprise",
            cost(&[generic(2), b()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

/// Mechanical Mobster — {3} 2/1 Human Robot Villain artifact creature. Enters:
/// exile up to one target card from a graveyard; target creature you control
/// connives.
pub fn mechanical_mobster() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::InGraveyard,
                    },
                    to: ZoneDest::Exile,
                },
                Effect::Connive {
                    what: target_filtered(mine()),
                    amount: Value::ONE,
                },
            ])),
        })],
        ..creature(
            "Mechanical Mobster",
            cost(&[generic(3)]),
            vec![
                CreatureType::Human,
                CreatureType::Robot,
                CreatureType::Villain,
            ],
            2,
            1,
        )
    }
}

/// Valentina Allegra de Fontaine — {2}{B} legendary 2/3 Human Spy Villain.
/// Other Villains you control are Heroes too. {T}: target Hero you control
/// connives; sorcery speed.
pub fn valentina_allegra_de_fontaine() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Villains you control are Heroes in addition to their other types.",
            effect: StaticEffect::AddCreatureTypeToMatching {
                applies_to: Selector::EachPermanent(my_villains().and(R::OtherThanSource)),
                creature_type: CreatureType::Hero,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Connive {
                what: target_filtered(mine().and(hero())),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend(creature(
            "Valentina Allegra de Fontaine",
            cost(&[generic(2), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Spy,
                CreatureType::Villain,
            ],
            2,
            3,
        ))
    }
}

/// MACH-1, Swooping Scoundrel — {1}{B} legendary 1/3 flying artifact
/// creature. As it enters and whenever you gain life, surveil 1 — once each
/// turn (the lifegain half sits out the turn it entered, which spent it).
pub fn mach_1_swooping_scoundrel() -> CardDefinition {
    let surveil = || Effect::Surveil {
        who: PlayerRef::You,
        amount: Value::ONE,
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(surveil()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl)
                    .with_filter(Predicate::Not(Box::new(Predicate::EntityMatches {
                        what: Selector::This,
                        filter: R::EnteredThisTurn,
                    })))
                    .once_per_turn(),
                effect: surveil(),
            },
        ],
        ..legend(creature(
            "MACH-1, Swooping Scoundrel",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Villain],
            1,
            3,
        ))
    }
}

/// Villainous Syndication — {1}{B} enchantment — Plan. Tap an untapped
/// Villain you control: mill a card and add a plan counter; sorcery speed. At
/// the fourth, sacrifice it: return target creature card from your graveyard
/// to the battlefield.
pub fn villainous_syndication() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(my_villains()),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                plan_counter(),
            ]),
            ..Default::default()
        }],
        ..plan(
            "Villainous Syndication",
            cost(&[generic(1), b()]),
            vec![plan_payoff(
                4,
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            )],
        )
    }
}

/// Decoy Ploy — {1}{B} instant. Choose one or both: return target Villain
/// card from your graveyard to your hand; return target Hero card likewise.
pub fn decoy_ploy() -> CardDefinition {
    let back = |filter: R| Effect::Move {
        what: target_filtered(filter.and(R::InYourGraveyard)),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    spell(
        "Decoy Ploy",
        cost(&[generic(1), b()]),
        CardType::Instant,
        Effect::ChooseModesCast {
            modes: vec![back(villain()), back(hero())],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}

/// Lethal Protection — {3}{B} sorcery. Destroy target creature; return up to
/// one target creature card from your graveyard to your hand.
pub fn lethal_protection() -> CardDefinition {
    spell(
        "Lethal Protection",
        cost(&[generic(3), b()]),
        CardType::Sorcery,
        Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::Destroy {
                    what: target_filtered(R::Creature),
                },
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::InYourGraveyard),
                    },
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ])),
        },
    )
}

/// HYDRA Infiltration — {3}{B} enchantment. Enters: target opponent discards
/// two. A creature of yours attacking alone drains target opponent for 1.
pub fn hydra_infiltration() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Discard {
                who: target_filtered(R::OpponentPlayer),
                amount: Value::Const(2),
                random: false,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(Predicate::AttackingAlone),
                effect: Effect::Drain {
                    from: target_filtered(R::OpponentPlayer),
                    to: Selector::You,
                    amount: Value::ONE,
                },
            },
        ],
        ..spell(
            "HYDRA Infiltration",
            cost(&[generic(3), b()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

/// Avengers: Under Siege — {2}{B}{R} Saga. I: two 2/1 menace Villains. II: 2
/// damage to each non-Villain creature and each opponent. III: a Treasure for
/// each Villain you control.
pub fn avengers_under_siege() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, make_villains(Value::Const(2), false)),
            (
                2,
                Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::EachPermanent(R::Creature.and(villain().negate())),
                        amount: Value::Const(2),
                    },
                    Effect::DealDamage {
                        to: each_opponent(),
                        amount: Value::Const(2),
                    },
                ]),
            ),
            (
                3,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::count(Selector::EachPermanent(my_villains())),
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                },
            ),
        ],
        ..spell(
            "Avengers: Under Siege",
            cost(&[generic(2), b(), r()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

/// Doc Ock's Henchmen — {2}{U} flash 2/1 Human Villain; connives whenever it
/// attacks.
pub fn doc_ocks_henchmen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![on_attack(connive(1))],
        ..creature(
            "Doc Ock's Henchmen",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Villain],
            2,
            1,
        )
    }
}

/// Mysterio, Master of Illusion — {3}{U} legendary 3/3 Human Villain. Enters:
/// a 3/3 blue Illusion Villain for each nontoken Villain you control; those
/// tokens are exiled when Mysterio leaves.
pub fn mysterio_master_of_illusion() -> CardDefinition {
    let illusion = TokenDefinition {
        name: "Illusion Villain".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Illusion, CreatureType::Villain],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::count(Selector::EachPermanent(my_villains().and(R::NotToken))),
                definition: Arc::new(illusion),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::Move {
                    what: Selector::TokensCreatedBySource,
                    to: ZoneDest::Exile,
                },
            },
        ],
        ..legend(creature(
            "Mysterio, Master of Illusion",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Villain],
            3,
            3,
        ))
    }
}
