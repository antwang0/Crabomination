//! Foundations (FDN) gap batch 14 — a once-per-game transformer, a page-counter
//! Book, an upkeep clone engine, and four legends (an Elf token payoff, a
//! sacrifice-value Cleric, a noncreature-spell flicker/token, a Raid
//! reanimator, and an aristocrat draw/burn). Tests in `tests/recent215.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{cast_is_noncreature, on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, Selector, Value,
    ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Mild-Mannered Librarian — {G} 1/1 Human. {3}{G}: becomes a Werewolf, gets two
/// +1/+1 counters, and you draw a card. Activate only once.
pub fn mild_mannered_librarian() -> CardDefinition {
    CardDefinition {
        name: "Mild-Mannered Librarian",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            activate_once: true,
            effect: Effect::Seq(vec![
                Effect::BecomeCreatureType {
                    what: Selector::This,
                    creature_types: vec![CreatureType::Werewolf],
                    duration: Duration::Permanent,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mazemind Tome — {2} Artifact Book. {T}, add a page counter: Scry 1. {2},{T},
/// add a page counter: Draw. At 4+ page counters, exile it and gain 4 life.
/// (The state trigger is modeled inline: each activation re-checks the count.)
pub fn mazemind_tome() -> CardDefinition {
    let cash_out = || Effect::If {
        cond: Predicate::SourceHasCountersAtLeast {
            counter: CounterType::Page,
            n: 4,
        },
        then: Box::new(Effect::Seq(vec![
            Effect::Exile {
                what: Selector::This,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
            },
        ])),
        else_: Box::new(Effect::Noop),
    };
    let add_page = || Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Page,
        amount: Value::ONE,
    };
    CardDefinition {
        name: "Mazemind Tome",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Book],
            ..Default::default()
        },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    add_page(),
                    Effect::Scry {
                        who: PlayerRef::You,
                        amount: Value::ONE,
                    },
                    cash_out(),
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    add_page(),
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    cash_out(),
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Extravagant Replication — {4}{U}{U} Enchantment. At the beginning of your
/// upkeep, create a token that's a copy of another target nonland permanent you
/// control.
pub fn extravagant_replication() -> CardDefinition {
    CardDefinition {
        name: "Extravagant Replication",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::Nonland.and(R::Permanent).and(R::ControlledByYou)),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Lathril, Blade of the Elves — {2}{B}{G} Legendary 2/3 Elf Noble. Menace.
/// Combat damage to a player → that many 1/1 Elf Warrior tokens. {T}, Tap ten
/// untapped Elves you control: each opponent loses 10 life, you gain 10.
pub fn lathril_blade_of_the_elves() -> CardDefinition {
    CardDefinition {
        name: "Lathril, Blade of the Elves",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Noble],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                definition: Box::new(elf_warrior_1_1()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            tap_n_filter: Some((
                R::HasCreatureType(CreatureType::Elf).and(R::ControlledByYou),
                10,
            )),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(10),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ayli, Eternal Pilgrim — {W}{B} Legendary 2/3 Kor Cleric. Deathtouch. {1}, Sac
/// another creature: gain life equal to its toughness. {1}{W}{B}, Sac another
/// creature: exile target nonland permanent (only if 10+ life above starting).
pub fn ayli_eternal_pilgrim() -> CardDefinition {
    CardDefinition {
        name: "Ayli, Eternal Pilgrim",
        cost: cost(&[w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::SacrificedToughness,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w(), b()]),
                sac_other_filter: Some((R::Creature, 1)),
                condition: Some(Predicate::PlayerLifeAtLeastAboveStarting {
                    who: PlayerRef::You,
                    delta: 10,
                }),
                effect: Effect::Exile {
                    what: target_filtered(R::Nonland.and(R::Permanent)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Kykar, Zephyr Awakener — {2}{W}{U} Legendary 3/4 Bird Wizard. Flying. When you
/// cast a noncreature spell, choose one — flicker another target creature you
/// control (returns next end step); or make a 1/1 white flying Spirit.
pub fn kykar_zephyr_awakener() -> CardDefinition {
    CardDefinition {
        name: "Kykar, Zephyr Awakener",
        cost: cost(&[generic(2), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_noncreature()),
            effect: Effect::ChooseMode(vec![
                Effect::ExileReturnNextEndStep {
                    what: target_filtered(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(white_spirit_flyer_1_1()),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Alesha, Who Laughs at Fate — {1}{B}{R} Legendary 2/2 Human Warrior. First
/// strike. Attacks → a +1/+1 counter on it. Raid — at your end step, if you
/// attacked, return a creature card with mana value ≤ its power from your
/// graveyard to the battlefield.
pub fn alesha_who_laughs_at_fate() -> CardDefinition {
    CardDefinition {
        name: "Alesha, Who Laughs at Fate",
        cost: cost(&[generic(1), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![
            on_attack(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::PlayerAttackedThisTurn {
                    who: PlayerRef::You,
                }),
                effect: Effect::Move {
                    what: target_filtered(
                        R::Creature
                            .and(R::InYourGraveyard)
                            .and(R::ManaValueAtMostSourcePower),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            },
        ],
        ..Default::default()
    }
}

/// Garna, Bloodfist of Keld — {1}{B}{R}{R} Legendary 4/3 Human Berserker.
/// Whenever another creature you control dies, draw a card if it had attacked
/// this turn; otherwise Garna deals 1 damage to each opponent.
pub fn garna_bloodfist_of_keld() -> CardDefinition {
    CardDefinition {
        name: "Garna, Bloodfist of Keld",
        cost: cost(&[generic(1), b(), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::AttackedThisTurn,
                },
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── shared token bodies ───────────────────────────────────────────────────────

fn elf_warrior_1_1() -> TokenDefinition {
    TokenDefinition {
        name: "Elf Warrior".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn white_spirit_flyer_1_1() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    }
}
