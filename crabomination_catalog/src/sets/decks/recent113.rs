//! Modern Horizons staples + the even-mana-value lock (Void Winnower). New
//! engine work: `StaticEffect::OpponentsCant{CastEvenMv,BlockWithEvenMv}`
//! (CR 601.3e / 509.1) and `CostReductionFirstCreatureSpell` (Conduit of
//! Ruin). Tests in `tests/recent113.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement, StaticAbility,
    Subtypes, TokenDefinition, TriggeredAbility, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
};
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color};

// ── Void Winnower / Price of Progress / Conduit of Ruin (new primitives) ──────

/// Void Winnower — {9} 11/9 Eldrazi. Opponents can't cast spells with even
/// mana values, nor block with creatures with even mana values (zero is even).
pub fn void_winnower() -> CardDefinition {
    CardDefinition {
        name: "Void Winnower",
        cost: cost(&[generic(9)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 11,
        toughness: 9,
        static_abilities: vec![
            StaticAbility {
                description: "Your opponents can't cast spells with even mana values.",
                effect: StaticEffect::OpponentsCantCastEvenMv,
            },
            StaticAbility {
                description: "Your opponents can't block with creatures with even mana values.",
                effect: StaticEffect::OpponentsCantBlockWithEvenMv,
            },
        ],
        ..Default::default()
    }
}

/// Price of Progress — {1}{R} Instant. Deals damage to each player equal to
/// twice the number of nonbasic lands that player controls.
pub fn price_of_progress() -> CardDefinition {
    CardDefinition {
        name: "Price of Progress",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachPlayer),
            body: Box::new(Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(Value::NonbasicLandCountControlledBy(PlayerRef::Triggerer)),
                ),
            }),
        },
        ..Default::default()
    }
}

/// Conduit of Ruin — {6} 5/5 Eldrazi. Cast: you may tutor a colorless creature
/// with mana value 7+ to the top; your first creature spell each turn costs {2}
/// less.
pub fn conduit_of_ruin() -> CardDefinition {
    CardDefinition {
        name: "Conduit of Ruin",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Search for a colorless creature with mana value 7 or greater?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::Colorless)
                        .and(SelectionRequirement::ManaValueAtLeast(7)),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                }),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "The first creature spell you cast each turn costs {2} less to cast.",
            effect: StaticEffect::CostReductionFirstCreatureSpell { amount: 2 },
        }],
        ..Default::default()
    }
}

// ── Modern Horizons (MH1) staples ─────────────────────────────────────────────

/// Changeling Outcast — {B} 1/1 changeling. Can't block and can't be blocked.
pub fn changeling_outcast() -> CardDefinition {
    CardDefinition {
        name: "Changeling Outcast",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Changeling, Keyword::CantBlock, Keyword::Unblockable],
        ..Default::default()
    }
}

/// Impostor of the Sixth Pride — {1}{W} 3/1 changeling.
pub fn impostor_of_the_sixth_pride() -> CardDefinition {
    CardDefinition {
        name: "Impostor of the Sixth Pride",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Changeling],
        ..Default::default()
    }
}

/// Segovian Angel — {W} 1/1 flying, vigilance.
pub fn segovian_angel() -> CardDefinition {
    CardDefinition {
        name: "Segovian Angel",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..Default::default()
    }
}

/// Putrid Goblin — {1}{B} 2/2 Zombie Goblin. Persist.
pub fn putrid_goblin() -> CardDefinition {
    CardDefinition {
        name: "Putrid Goblin",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Goblin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Persist],
        ..Default::default()
    }
}

/// Undead Augur — {B}{B} 2/2 Zombie Wizard. When this or another Zombie you
/// control dies, draw a card and lose 1 life.
pub fn undead_augur() -> CardDefinition {
    CardDefinition {
        name: "Undead Augur",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Zombie),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// King of the Pride — {2}{W} 2/1 Cat. Other Cats you control get +2/+1.
pub fn king_of_the_pride() -> CardDefinition {
    CardDefinition {
        name: "King of the Pride",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Other Cats you control get +2/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Cat)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 2,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Ravenous Giant — {2}{R}{R} 5/5 Giant. At your upkeep, deals 1 damage to you.
pub fn ravenous_giant() -> CardDefinition {
    CardDefinition {
        name: "Ravenous Giant",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::DealDamage { to: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Irregular Cohort — {2}{W}{W} 2/2 changeling. ETB: create a 2/2 colorless
/// Shapeshifter token with changeling.
pub fn irregular_cohort() -> CardDefinition {
    CardDefinition {
        name: "Irregular Cohort",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Changeling],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Shapeshifter".into(),
                power: 2,
                toughness: 2,
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Shapeshifter],
                    ..Default::default()
                },
                keywords: vec![Keyword::Changeling],
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Vesperlark — {2}{W} 2/1 Elemental. Flying; evoke {1}{W}. When it leaves the
/// battlefield, return a creature card with power 1 or less from your graveyard.
pub fn vesperlark() -> CardDefinition {
    CardDefinition {
        name: "Vesperlark",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(1), w()]),
            evoke_sacrifice: true,
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(1)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..Default::default()
    }
}

/// Martyr's Soul — {2}{W} 3/2 Spirit Soldier. Convoke; ETB, if you control no
/// tapped lands, enters with two +1/+1 counters.
pub fn martyrs_soul() -> CardDefinition {
    CardDefinition {
        name: "Martyr's Soul",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(
                Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::Land
                        .and(SelectionRequirement::Tapped)
                        .and(SelectionRequirement::ControlledByYou),
                )))),
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Scour All Possibilities — {1}{U} Sorcery. Scry 2, then draw. Flashback {4}{U}.
pub fn scour_all_possibilities() -> CardDefinition {
    CardDefinition {
        name: "Scour All Possibilities",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(4), u()]))],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Rain of Revelation — {3}{U} Instant. Draw three cards, then discard a card.
pub fn rain_of_revelation() -> CardDefinition {
    CardDefinition {
        name: "Rain of Revelation",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        ]),
        ..Default::default()
    }
}

/// Pyrophobia — {1}{R} Sorcery. Deals 3 damage to target creature. (The
/// "Cowards can't block" rider is a benign creature-type flavor line.)
pub fn pyrophobia() -> CardDefinition {
    CardDefinition {
        name: "Pyrophobia",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Mob — {4}{B} Instant. Convoke; destroy target creature.
pub fn mob() -> CardDefinition {
    CardDefinition {
        name: "Mob",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Convoke],
        effect: Effect::Destroy { what: target_filtered(SelectionRequirement::Creature) },
        ..Default::default()
    }
}

/// Nature's Chant — {1}{G/W} Instant. Destroy target artifact or enchantment.
pub fn natures_chant() -> CardDefinition {
    CardDefinition {
        name: "Nature's Chant",
        cost: cost(&[generic(1), hybrid(Color::Green, Color::White)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Or(
                Box::new(SelectionRequirement::Artifact),
                Box::new(SelectionRequirement::Enchantment),
            )),
        },
        ..Default::default()
    }
}

/// Igneous Elemental — {4}{R}{R} 4/3 Elemental. Costs {2} less if a land card is
/// in your graveyard. ETB: may deal 2 damage to target creature.
pub fn igneous_elemental() -> CardDefinition {
    CardDefinition {
        name: "Igneous Elemental",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 4,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "This spell costs {2} less to cast if there is a land card in your graveyard.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::SelectorExists(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Land,
                }),
                amount: 2,
            },
        }],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Deal 2 damage to target creature?".into(),
            body: Box::new(Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            }),
        })],
        ..Default::default()
    }
}

/// Mother Bear — {1}{G} 2/2 Bear. {3}{G}{G}, exile this from your graveyard:
/// create two 2/2 green Bears. Sorcery speed.
pub fn mother_bear() -> CardDefinition {
    let bear = TokenDefinition {
        name: "Bear".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        colors: vec![Color::Green],
        ..Default::default()
    };
    CardDefinition {
        name: "Mother Bear",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g(), g()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: bear,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Goatnap — {2}{R} Sorcery. Gain control of target creature until end of turn,
/// untap it, and it gains haste; a Goat also gets +3/+0.
pub fn goatnap() -> CardDefinition {
    CardDefinition {
        name: "Goatnap",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(SelectionRequirement::Creature),
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Goat),
                },
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(3),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Savage Swipe — {G} Sorcery. Target creature you control gets +2/+2 if its
/// power is 2, then it fights target creature you don't control.
pub fn savage_swipe() -> CardDefinition {
    CardDefinition {
        name: "Savage Swipe",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::PowerAtMost(2)
                        .and(SelectionRequirement::PowerAtLeast(2)),
                },
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Fight {
                attacker: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Fists of Flame — {1}{R} Instant. Draw a card; until end of turn, target
/// creature gains trample and gets +1/+0 for each card you've drawn this turn.
pub fn fists_of_flame() -> CardDefinition {
    CardDefinition {
        name: "Fists of Flame",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::CardsDrawnThisTurn(PlayerRef::You),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Modern Horizons (MH1) — batch 2 ───────────────────────────────────────────

/// Orcish Hellraiser — {1}{R} 3/2 Orc Warrior. Echo {R}; dies: deals 2 damage
/// to any target player or planeswalker.
pub fn orcish_hellraiser() -> CardDefinition {
    CardDefinition {
        name: "Orcish Hellraiser",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Echo(cost(&[r()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Or(
                    Box::new(SelectionRequirement::Player),
                    Box::new(SelectionRequirement::Planeswalker),
                )),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Vengeful Devil — {1}{R} 1/1 Devil. Haste; morbid {T}: deal 1 damage to any
/// target (only if a creature died this turn).
pub fn vengeful_devil() -> CardDefinition {
    CardDefinition {
        name: "Vengeful Devil",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Devil], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) }),
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Any),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pondering Mage — {3}{U}{U} 3/4 Human Wizard. ETB: reorder the top three,
/// then draw a card.
pub fn pondering_mage() -> CardDefinition {
    CardDefinition {
        name: "Pondering Mage",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::RearrangeTop { who: PlayerRef::You, amount: Value::Const(3) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Treetop Ambusher — {1}{G} 2/1 Elf Berserker. Dash {1}{G}; attacks: target
/// creature you control gets +1/+1 until end of turn.
pub fn treetop_ambusher() -> CardDefinition {
    CardDefinition {
        name: "Treetop Ambusher",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(1), g()]),
            dash: true,
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Graveshifter — {3}{B} 2/2 changeling. ETB: you may return target creature
/// card from your graveyard to your hand.
pub fn graveshifter() -> CardDefinition {
    CardDefinition {
        name: "Graveshifter",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Changeling],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return target creature card from your graveyard to your hand?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..Default::default()
    }
}

/// Recruit the Worthy — {W} Instant. Buyback {3}; create a 1/1 white Soldier.
pub fn recruit_the_worthy() -> CardDefinition {
    CardDefinition {
        name: "Recruit the Worthy",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Buyback(cost(&[generic(3)]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Soldier".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Soldier],
                    ..Default::default()
                },
                colors: vec![Color::White],
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Headless Specter — {1}{B}{B} 2/2 Specter. Flying; hellbent — combat damage to
/// a player makes them discard at random if you have no cards in hand.
pub fn headless_specter() -> CardDefinition {
    CardDefinition {
        name: "Headless Specter",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Specter], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::HellbentActive { who: PlayerRef::You }),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
                random: true,
            },
        }],
        ..Default::default()
    }
}

/// Excavating Anurid — {4}{G} 4/4 Frog Beast. ETB: may sacrifice a land to draw.
/// Threshold — while 7+ cards are in your graveyard, +1/+1 and vigilance.
pub fn excavating_anurid() -> CardDefinition {
    CardDefinition {
        name: "Excavating Anurid",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice a land to draw a card?".into(),
            filter: SelectionRequirement::Land,
            count: Value::Const(1),
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            else_: None,
        })],
        static_abilities: vec![StaticAbility {
            description: "Threshold — this creature gets +1/+1 and has vigilance.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ThresholdActive { who: PlayerRef::You },
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Vigilance],
            },
        }],
        ..Default::default()
    }
}

/// Goblin War Party — {3}{R} Sorcery. Choose one — three 1/1 red Goblins; or
/// creatures you control get +1/+1 and gain haste. Entwine {2}{R}.
pub fn goblin_war_party() -> CardDefinition {
    CardDefinition {
        name: "Goblin War Party",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Entwine(cost(&[generic(2), r()]))],
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: TokenDefinition {
                    name: "Goblin".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Goblin],
                        ..Default::default()
                    },
                    colors: vec![Color::Red],
                    ..Default::default()
                },
            },
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        ]),
        ..Default::default()
    }
}

/// Viashino Sandsprinter — {1}{R}{R} 4/1 Lizard Warrior. Trample, haste; returns
/// to its owner's hand at the beginning of the end step. Cycling {R}.
pub fn viashino_sandsprinter() -> CardDefinition {
    CardDefinition {
        name: "Viashino Sandsprinter",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::Cycling(cost(&[r()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..Default::default()
    }
}

/// Trustworthy Scout — {1}{W} 2/2 Human Scout. {1}{W}, exile this from your
/// graveyard: search for a card named Trustworthy Scout and put it into hand.
pub fn trustworthy_scout() -> CardDefinition {
    CardDefinition {
        name: "Trustworthy Scout",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasName("Trustworthy Scout".into()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Windcaller Aven — {4}{U}{U} 4/3 Bird Wizard. Flying; cycling {U}. (The "when
/// you cycle this, target creature gains flying" rider is approximated away.)
pub fn windcaller_aven() -> CardDefinition {
    CardDefinition {
        name: "Windcaller Aven",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Cycling(cost(&[u()]))],
        ..Default::default()
    }
}

/// Zhalfirin Decoy — {1}{W} 1/3 Human Soldier. {T}: tap target creature.
/// Activate only if a creature entered under your control this turn (CR 603 —
/// its own arrival counts).
pub fn zhalfirin_decoy() -> CardDefinition {
    CardDefinition {
        name: "Zhalfirin Decoy",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::CreatureEnteredThisTurn { who: PlayerRef::You }),
            effect: Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}
