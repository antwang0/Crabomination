//! Commander: the cards the **Family Matters** precon (BLC, Zinnia, Valley's
//! Voice) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_zinnia.rs`.
//!
//! Residuals (each also on its card):
//! - **Zinnia, Valley's Voice** — the granted offspring copy is Zinnia's
//!   trigger, so it is lost if Zinnia leaves before the creature enters; a
//!   creature with its own kicker or offspring gets no second one.
//! - **Echoing Assault** — one copy per combat, not one per player attacked.
//! - **Combat Celebrant** — a second exert in the same turn is allowed but
//!   does nothing (the bonus is once a turn).
//! - **Rose Room Treasurer** — the {X} is paid from floating mana.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, evolve, on_attack, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, r, u, w, x};
use std::sync::Arc;

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

fn token(name: &str, colors: Vec<Color>, ct: CreatureType, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// Alliance — "whenever another creature you control enters".
fn alliance(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
        effect,
    }
}

/// CR 702.175 — the 1/1 token copy an offspring payment buys.
fn offspring_copy(source: Selector) -> Effect {
    Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source,
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: Some((1, 1)),
        override_colors: None,
        enters_tapped: false,
        non_legendary: false,
        legendary: false,
        extra_keywords: vec![],
    }
}

/// A printed offspring: the ETB copy when its cost was paid.
fn offspring_etb() -> TriggeredAbility {
    etb(Effect::If {
        cond: Predicate::SpellWasKicked,
        then: Box::new(offspring_copy(Selector::This)),
        else_: Box::new(Effect::Noop),
    })
}

/// Zinnia, Valley's Voice — flying; +X/+0 for your other creatures with base
/// power 1; creature spells you cast gain offspring {2}.
/// Residual: the granted copy is Zinnia's own trigger, so it is lost if Zinnia
/// leaves before the creature enters; a creature with its own kicker or
/// offspring gets no second one.
pub fn zinnia_valleys_voice() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Zinnia gets +X/+0, where X is the number of other creatures you control with base power 1.",
                effect: StaticEffect::PumpPTByValue {
                    applies_to: Selector::This,
                    power: Value::CountOf(Box::new(yours(R::Creature.and(R::BasePowerIs(1)).and(R::OtherThanSource)))),
                    toughness: Value::Const(0),
                },
            },
            StaticAbility {
                description: "Creature spells you cast gain offspring {2} as you cast them.",
                effect: StaticEffect::CreatureSpellsGainOffspring { cost: cost(&[generic(2)]) },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::PaidGrantedOffspring) },
            ),
            effect: offspring_copy(Selector::TriggerSource),
        }],
        ..creature("Zinnia, Valley's Voice", cost(&[u(), r(), w()]), vec![CreatureType::Bird, CreatureType::Bard], 1, 3)
    }
}

/// Arthur, Marigold Knight — haste; when it and another creature attack, a
/// creature from the top six joins the attack and returns to hand at end of
/// combat.
pub fn arthur_marigold_knight() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::AttackingWithAtLeast(2)),
            effect: Effect::LookTopMayDeployAttacking {
                count: Value::Const(6),
                filter: R::Creature,
                return_at_end_of_combat: true,
            },
        }],
        ..creature(
            "Arthur, Marigold Knight",
            cost(&[generic(2), u(), r(), w()]),
            vec![CreatureType::Mouse, CreatureType::Knight],
            4,
            5,
        )
    }
}

/// Agate Instigator — offspring {1}{R}; alliance: 1 damage to each opponent.
pub fn agate_instigator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Offspring(cost(&[generic(1), r()]))],
        triggered_abilities: vec![
            offspring_etb(),
            alliance(Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE }),
        ],
        ..creature("Agate Instigator", cost(&[generic(1), r()]), vec![CreatureType::Lizard, CreatureType::Rogue], 1, 3)
    }
}

/// Boss's Chauffeur — enters with one plus your other creatures in +1/+1
/// counters; alliance grows it; dying, a Citizen per counter.
pub fn bosss_chauffeur() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Sum(vec![
                Value::ONE,
                Value::CountOf(Box::new(yours(R::Creature.and(R::OtherThanSource)))),
            ]),
        )),
        triggered_abilities: vec![
            alliance(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                    definition: Arc::new(token(
                        "Citizen",
                        vec![Color::Green, Color::White],
                        CreatureType::Citizen,
                        1,
                        1,
                        vec![],
                    )),
                },
            },
        ],
        ..creature("Boss's Chauffeur", cost(&[generic(4), w()]), vec![CreatureType::Elf, CreatureType::Citizen], 0, 0)
    }
}

/// Calamity of Cinders — convoke; 6 damage to each untapped creature.
pub fn calamity_of_cinders() -> CardDefinition {
    CardDefinition {
        name: "Calamity of Cinders",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Convoke],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::Untapped)),
            body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(6) }),
        },
        ..Default::default()
    }
}

/// Combat Celebrant — exert as it attacks: untap your other creatures and
/// take an additional combat phase.
/// Residual: a second exert in the same turn is allowed but does nothing (the
/// bonus is once a turn, which is what "if it hasn't been exerted this turn"
/// buys).
pub fn combat_celebrant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Exert],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Exerted, EventScope::SelfSource),
            effect: Effect::NthResolutionThisTurn {
                branches: vec![Effect::Seq(vec![
                    Effect::Untap { what: yours(R::Creature.and(R::OtherThanSource)), up_to: None },
                    Effect::AdditionalCombatPhase { count: Value::ONE },
                ])],
            },
        }],
        ..creature("Combat Celebrant", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Warrior], 4, 1)
    }
}

/// Devilish Valet — trample, haste; alliance doubles its power.
pub fn devilish_valet() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![alliance(Effect::PumpPT {
            what: Selector::This,
            power: Value::PowerOf(Box::new(Selector::This)),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Devilish Valet", cost(&[generic(2), r()]), vec![CreatureType::Devil, CreatureType::Warrior], 1, 3)
    }
}

/// Echoing Assault — your creature tokens have menace; whenever you attack a
/// player, a 1/1 token copy of a nontoken creature attacking that player
/// joins it, sacrificed at the next end step.
pub fn echoing_assault() -> CardDefinition {
    CardDefinition {
        name: "Echoing Assault",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control have menace.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(R::IsToken)),
                keyword: Keyword::Menace,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            // CR 603.2 — once for each player attacked, "that player" bound.
            event: EventSpec::new(EventKind::Attacks, EventScope::YouAttackedPlayer),
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(
                        R::Creature.and(R::NotToken).and(R::IsAttackingTriggerPlayer).and(R::ControlledByYou),
                    ),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: Some((1, 1)),
                    override_colors: None,
                    enters_tapped: true,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
                Effect::JoinCombatAttacking { what: Selector::LastCreatedToken },
                Effect::SacrificeLastCreatedTokensAtNextEndStep,
            ]),
        }],
        ..Default::default()
    }
}

/// Fortune Teller's Talent — Class. Look at your top card any time; level 2:
/// after you've cast a spell this turn, play cards from the top; level 3:
/// spells from anywhere but your hand cost {2} less.
pub fn fortune_tellers_talent() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| crate::card::ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    CardDefinition {
        name: "Fortune Teller's Talent",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "Level 2 — As long as you've cast a spell this turn, you may play cards from the top of your library.",
                effect: StaticEffect::WhileClassLevelAtLeast {
                    n: 2,
                    inner: Box::new(StaticEffect::WhileCondition {
                        condition: Predicate::SpellsCastThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
                        inner: Box::new(StaticEffect::PlayFromLibraryTop { filter: R::Any }),
                    }),
                },
            },
            StaticAbility {
                description: "Level 3 — Spells you cast from anywhere other than your hand cost {2} less to cast.",
                effect: StaticEffect::WhileClassLevelAtLeast {
                    n: 3,
                    inner: Box::new(StaticEffect::NonHandCastCostReduction { amount: 2 }),
                },
            },
        ],
        activated_abilities: vec![level_up(cost(&[generic(3), u()]), 1), level_up(cost(&[generic(2), u()]), 2)],
        ..Default::default()
    }
}

/// Jacked Rabbit — ravenous (enters with X +1/+1 counters, draws at X ≥ 5);
/// attacking, a 1/1 Rabbit per point of its power.
pub fn jacked_rabbit() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(5)),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            }),
            on_attack(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(Selector::This)),
                definition: Arc::new(token("Rabbit", vec![Color::White], CreatureType::Rabbit, 1, 1, vec![])),
            }),
        ],
        ..creature("Jacked Rabbit", cost(&[x(), generic(1), w()]), vec![CreatureType::Rabbit, CreatureType::Warrior], 1, 2)
    }
}

/// Murmuration — your Birds get +1/+1 and vigilance; at your end step, a
/// Storm Crow per spell you cast this turn.
pub fn murmuration() -> CardDefinition {
    let birds = || yours(R::Creature.and(R::HasCreatureType(CreatureType::Bird)));
    CardDefinition {
        name: "Murmuration",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Birds you control get +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: birds(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Birds you control have vigilance.",
                effect: StaticEffect::GrantKeyword { applies_to: birds(), keyword: Keyword::Vigilance },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::SpellsCastThisTurn(PlayerRef::You),
                definition: Arc::new(token(
                    "Storm Crow",
                    vec![Color::Blue],
                    CreatureType::Bird,
                    1,
                    2,
                    vec![Keyword::Flying],
                )),
            },
        }],
        ..Default::default()
    }
}

/// Pollywog Prodigy — evolve; draws when an opponent casts a noncreature
/// spell with mana value less than its power.
pub fn pollywog_prodigy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            evolve(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Noncreature },
                    Predicate::ValueAtMost(
                        Value::Sum(vec![Value::ManaValueOf(Box::new(Selector::TriggerSource)), Value::ONE]),
                        Value::PowerOf(Box::new(Selector::This)),
                    ),
                ])),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..creature("Pollywog Prodigy", cost(&[generic(1), u()]), vec![CreatureType::Frog, CreatureType::Wizard], 1, 3)
    }
}

/// Rapid Augmenter — haste; your entering base-power-1 creatures gain haste;
/// one that wasn't cast grows it and makes it unblockable this turn.
pub fn rapid_augmenter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::BasePowerIs(1)) },
                ),
                effect: Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                    Predicate::Not(Box::new(Predicate::TriggerSourceEnteredByCast)),
                ])),
                effect: Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
                ]),
            },
        ],
        ..creature(
            "Rapid Augmenter",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Otter, CreatureType::Artificer],
            1,
            3,
        )
    }
}

/// Rose Room Treasurer — alliance: a Treasure the first two times a turn,
/// then "you may pay {X}: X damage to any target".
/// Residual: the {X} is paid from floating mana.
pub fn rose_room_treasurer() -> CardDefinition {
    let treasure = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(crabomination_base::tokens::treasure_token()),
    };
    CardDefinition {
        triggered_abilities: vec![alliance(Effect::EscalatingThisTurn {
            modes: vec![
                treasure(),
                treasure(),
                Effect::MayPayX {
                    description: "Pay {X} to have Rose Room Treasurer deal X damage to any target?".into(),
                    body: Box::new(Effect::DealDamage { to: target_any(), amount: Value::XFromCost }),
                },
            ],
        })],
        ..creature("Rose Room Treasurer", cost(&[generic(3), r()]), vec![CreatureType::Ogre, CreatureType::Warrior], 4, 3)
    }
}

/// Shield Broker — its ETB puts a shield counter on a noncommander creature
/// you don't control and takes it while the counter stays.
pub fn shield_broker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::IsCommander.negate())),
                kind: CounterType::Shield,
                amount: Value::ONE,
            },
            Effect::GainControlWhileCounter { what: Selector::Target(0), kind: CounterType::Shield },
        ]))],
        ..creature(
            "Shield Broker",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Octopus, CreatureType::Advisor],
            3,
            4,
        )
    }
}

/// Stolen by the Fae — bounce a creature with mana value X; X 1/1 flying
/// Faeries.
pub fn stolen_by_the_fae() -> CardDefinition {
    CardDefinition {
        name: "Stolen by the Fae",
        cost: cost(&[x(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::ManaValueExactlyXFromCost)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: Arc::new(token(
                    "Faerie",
                    vec![Color::Blue],
                    CreatureType::Faerie,
                    1,
                    1,
                    vec![Keyword::Flying],
                )),
            },
        ]),
        ..Default::default()
    }
}

/// Storm of Souls — every creature card in your graveyard returns as a 1/1
/// Spirit with flying in addition to its other types; exile it.
pub fn storm_of_souls() -> CardDefinition {
    CardDefinition {
        name: "Storm of Souls",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Sorcery],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::ReturnAllMatchingFromGraveyardToBattlefield {
                who: PlayerRef::You,
                filter: R::Creature,
                sacrifice_eot: false,
            },
            Effect::SetBasePT {
                what: Selector::LastMoved,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::Permanent,
            },
            Effect::AddCreatureTypes {
                what: Selector::LastMoved,
                creature_types: vec![CreatureType::Spirit],
                duration: Duration::Permanent,
            },
            Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Flying, duration: Duration::Permanent },
        ]),
        ..Default::default()
    }
}

/// Tetsuko Umezawa, Fugitive — your creatures with power or toughness 1 or
/// less can't be blocked.
pub fn tetsuko_umezawa_fugitive() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with power or toughness 1 or less can't be blocked.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature),
                keyword: Keyword::UnblockableWhilePowerOrToughnessAtMost(1),
            },
        }],
        ..creature("Tetsuko Umezawa, Fugitive", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Rogue], 1, 3)
    }
}

/// Thriving Bluff — enters tapped; {T}: {R} or a chosen other color.
pub fn thriving_bluff() -> CardDefinition {
    super::cmdr_bumbleflower::thriving("Thriving Bluff", Color::Red)
}
