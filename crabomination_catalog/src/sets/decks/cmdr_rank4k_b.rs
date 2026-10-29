//! Commander: one-card gaps of EDHREC's commanders ranked 3501-4500 (the
//! 2026-10-14 scan), second file. Tests in `tests/recent_b/cmdr_rank4k_b.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LevelBand, LoyaltyAbility, Predicate,
    SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
use std::sync::Arc;

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

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: n,
        definition: Arc::new(def),
    }
}

fn plus_one(what: Selector) -> Effect {
    Effect::AddCounter {
        what,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    }
}

// ── Arni, Humble Scribe ──────────────────────────────────────────────────────

/// Banishing Knack — {U} instant. Until end of turn, target creature gains
/// "{T}: Return target nonland permanent to its owner's hand."
pub fn banishing_knack() -> CardDefinition {
    spell(
        "Banishing Knack",
        cost(&[u()]),
        CardType::Instant,
        Effect::GainActivatedAbility {
            what: target_filtered(R::Creature),
            ability: Box::new(ActivatedAbility {
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::Permanent.and(R::Not(Box::new(R::Land)))),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                ..Default::default()
            }),
            duration: Duration::EndOfTurn,
        },
    )
}

// ── Vraska, the Cutting Glare ────────────────────────────────────────────────

/// Ferocity of the Hunt — {1}{B/G} flash Aura. Enchanted creature gets +1/+0
/// and has deathtouch. When it dies, return that card to the battlefield
/// tapped under its owner's control.
pub fn ferocity_of_the_hunt() -> CardDefinition {
    CardDefinition {
        name: "Ferocity of the Hunt",
        cost: ManaCost {
            symbols: vec![generic(1), hybrid(Color::Black, Color::Green)],
        },
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            keywords: vec![Keyword::Deathtouch],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                    tapped: true,
                },
            },
        }],
        ..Default::default()
    }
}

// ── Hazduhr the Abbot ────────────────────────────────────────────────────────

/// Near-Death Experience — {2}{W}{W}{W} enchantment. At the beginning of your
/// upkeep, if you have exactly 1 life, you win the game (CR 104.2b).
pub fn near_death_experience() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::PlayerLifeExactly {
                    who: PlayerRef::You,
                    life: 1,
                }),
            effect: Effect::WinGame { who: PlayerRef::You },
        }],
        ..spell(
            "Near-Death Experience",
            cost(&[generic(2), w(), w(), w()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Kang the Conqueror ───────────────────────────────────────────────────────

/// Lighthouse Chronologist — {1}{U} 1/3 Human Wizard, level up {U}. LEVEL 4-6:
/// 2/4. LEVEL 7+: 3/5; at the beginning of each end step, if it's not your
/// turn, take an extra turn after this one.
pub fn lighthouse_chronologist() -> CardDefinition {
    CardDefinition {
        level_bands: vec![
            LevelBand {
                min: 4,
                max: Some(6),
                power: 2,
                toughness: 4,
                keywords: vec![],
            },
            LevelBand {
                min: 7,
                max: None,
                power: 3,
                toughness: 5,
                keywords: vec![],
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Level,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::All(vec![
                    Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
                    Predicate::SourceHasCountersAtLeast {
                        counter: CounterType::Level,
                        n: 7,
                    },
                ])),
            effect: Effect::TakeExtraTurn {
                who: PlayerRef::You,
                count: Value::ONE,
            },
        }],
        ..creature(
            "Lighthouse Chronologist",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            3,
        )
    }
}

// ── Marwyn, the Preserver ────────────────────────────────────────────────────

/// Carnivorous Cultivator // Enroot — {1}{G} 2/3 deathtouch Elf Warlock that
/// enters prepared. Combat damage to a player: return target land card from
/// your graveyard to your hand. Enroot ({G} sorcery): search your library for
/// a land card, put it into your graveyard, then shuffle.
pub fn carnivorous_cultivator() -> CardDefinition {
    let enroot = spell(
        "Enroot",
        cost(&[g()]),
        CardType::Sorcery,
        Effect::Search {
            who: PlayerRef::You,
            filter: R::Land,
            to: ZoneDest::Graveyard,
        },
    );
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        prepare_spell: Some(Arc::new(enroot)),
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(R::Land.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..creature(
            "Carnivorous Cultivator",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Warlock],
            2,
            3,
        )
    }
}

// ── Reed Richards, Smartest Man ──────────────────────────────────────────────

/// Fantastic Bounce — {3}{U} sorcery; {2} less if it targets a tapped
/// creature. Return target nonland permanent to its owner's hand. Draw a card.
pub fn fantastic_bounce() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if_target: Some((R::Creature.and(R::Tapped), 2)),
        ..spell(
            "Fantastic Bounce",
            cost(&[generic(3), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Permanent.and(R::Not(Box::new(R::Land)))),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
        )
    }
}

// ── Samwise the Stouthearted ─────────────────────────────────────────────────

/// Moment of Glory — {W} sorcery, flashback {4}{W}. A +1/+1 counter on target
/// creature you control; cast from a graveyard, also one on each other
/// creature you control.
pub fn moment_of_glory() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(4), w()]))],
        ..spell(
            "Moment of Glory",
            cost(&[w()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                plus_one(target_filtered(R::Creature.and(R::ControlledByYou))),
                Effect::If {
                    cond: Predicate::CastFromGraveyard,
                    then: Box::new(plus_one(Selector::EachPermanentExceptTargets(
                        R::Creature.and(R::ControlledByYou),
                    ))),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

// ── Beregond of the Guard ────────────────────────────────────────────────────

/// Repurposed Enforcer — {1}{W} 3/2 Human Soldier. Whenever it attacks,
/// empower Jace X, where X is the number of creatures you control.
pub fn repurposed_enforcer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::EmpowerJace {
            who: PlayerRef::You,
            count: Value::PermanentCountControlledByMatching(PlayerRef::You, R::Creature),
        })],
        ..creature(
            "Repurposed Enforcer",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            2,
        )
    }
}

// ── Ishi-Ishi, Akki Crackshot ────────────────────────────────────────────────

/// Goblin Diplomats — {1}{R} 2/1 Goblin. {T}: each creature attacks this turn
/// if able.
pub fn goblin_diplomats() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeywordToMatchingThisTurn {
                filter: R::Creature,
                keyword: Keyword::MustAttack,
            },
            ..Default::default()
        }],
        ..creature(
            "Goblin Diplomats",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin],
            2,
            1,
        )
    }
}

// ── Myojin of Infinite Rage ──────────────────────────────────────────────────

/// Salvager of Ruin — {3} 2/1 Construct artifact creature. Sacrifice it: return
/// target permanent card in your graveyard put there from the battlefield this
/// turn to your hand.
pub fn salvager_of_ruin() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(
                    R::Permanent
                        .and(R::PutIntoGraveyardFromBattlefieldThisTurn)
                        .from_your_graveyard(),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Salvager of Ruin",
            cost(&[generic(3)]),
            vec![CreatureType::Construct],
            2,
            1,
        )
    }
}

// ── Esior, Wardwing Familiar // Radiant, Serra Archangel ─────────────────────

/// Radiant, Serra Archangel — {6}{W} legendary 6/4 flying Angel, partner. Tap
/// another untapped creature you control with flying: it gains protection from
/// the color of your choice until end of turn.
pub fn radiant_serra_archangel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Partner],
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            effect: Effect::GrantProtectionFromChosenColor {
                what: Selector::This,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Radiant, Serra Archangel",
            cost(&[generic(6), w()]),
            vec![CreatureType::Angel],
            6,
            4,
        )
    }
}

// ── Joel, Resolute Survivor ──────────────────────────────────────────────────

/// Joel, Resolute Survivor — {3}{B}{G} legendary 4/4 menace Human Survivor,
/// Partner—Survivors. Whenever a creature token dies, a +1/+1 counter on Joel
/// and draw a card; once each turn.
pub fn joel_resolute_survivor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::PartnerLabel("Survivors".into())],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsToken,
                })
                .once_per_turn(),
            effect: Effect::Seq(vec![
                plus_one(Selector::This),
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..legend(
            "Joel, Resolute Survivor",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Human, CreatureType::Survivor],
            4,
            4,
        )
    }
}

// ── Captain James T. Kirk ────────────────────────────────────────────────────

/// Captain James T. Kirk — {2}{R} legendary 2/2 Human Officer. Enters or
/// attacks: choose one (one or more with an empty hand) — loot; a 1/1 red
/// Officer; creatures you control get +1/+0 until end of turn. The empty-hand
/// "one or more" is `ChooseUpToN` (3): an empty answer runs all three.
pub fn captain_james_t_kirk() -> CardDefinition {
    let officer = TokenDefinition {
        name: "Officer".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Officer],
            ..Default::default()
        },
        ..Default::default()
    };
    let modes = move || {
        vec![
            Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
            make(Value::ONE, officer.clone()),
            Effect::PumpPT {
                what: yours(R::Creature),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        ]
    };
    let pick = move || Effect::If {
        cond: Predicate::ValueEquals(Value::HandSizeOf(PlayerRef::You), Value::ZERO),
        then: Box::new(Effect::ChooseUpToN {
            max: Box::new(Value::Const(3)),
            modes: modes(),
        }),
        else_: Box::new(Effect::ChooseMode(modes())),
    };
    CardDefinition {
        triggered_abilities: vec![etb(pick()), on_attack(pick())],
        ..legend(
            "Captain James T. Kirk",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Officer],
            2,
            2,
        )
    }
}

// ── Greensleeves ─────────────────────────────────────────────────────────────

/// Greensleeves — {4}{G}{G} legendary planeswalker, loyalty 5; can be your
/// commander. +2: a Giant Badger token ({1}{G}{G} 2/2 that gets +2/+2 when it
/// blocks). −3: mill three, put the permanent cards among them into your hand.
/// −8: creatures you control have base 8/8 and trample until end of turn.
pub fn greensleeves() -> CardDefinition {
    let badger = TokenDefinition {
        name: "Giant Badger".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Badger],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Greensleeves",
        cost: cost(&[generic(4), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        base_loyalty: 5,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: make(Value::ONE, badger),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::Mill {
                        who: Selector::You,
                        amount: Value::Const(3),
                    },
                    Effect::ForEach {
                        selector: Selector::LastMoved,
                        body: Box::new(Effect::If {
                            cond: Predicate::EntityMatches {
                                what: Selector::TriggerSource,
                                filter: R::Permanent,
                            },
                            then: Box::new(Effect::Move {
                                what: Selector::TriggerSource,
                                to: ZoneDest::Hand(PlayerRef::You),
                            }),
                            else_: Box::new(Effect::Noop),
                        }),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::Seq(vec![
                    Effect::SetBasePT {
                        what: yours(R::Creature),
                        power: Value::Const(8),
                        toughness: Value::Const(8),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: yours(R::Creature),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── White Widow, Yelena Belova ───────────────────────────────────────────────

/// Ashnod, Flesh Mechanist — {B} legendary 1/1 deathtouch Human Artificer.
/// Whenever it attacks, you may sacrifice another creature; if you do, a
/// tapped Powerstone. {5}, exile a creature card from your graveyard: a tapped
/// 3/3 colorless Zombie artifact creature token.
pub fn ashnod_flesh_mechanist() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        tapped: true,
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description: "Sacrifice another creature for a tapped Powerstone?".into(),
            filter: R::Creature.and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(make(
                Value::ONE,
                TokenDefinition {
                    tapped: true,
                    ..crabomination_base::tokens::powerstone_token()
                },
            )),
            else_: None,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            exile_other_filter: Some((R::Creature.and(R::InYourGraveyard), 1)),
            effect: make(Value::ONE, zombie),
            ..Default::default()
        }],
        ..legend(
            "Ashnod, Flesh Mechanist",
            cost(&[b()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            1,
        )
    }
}
