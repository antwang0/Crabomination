//! Commander: the cards the **Draconic Rage** precon (AFC, Vrondiss, Rage of
//! Ancients) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Berserker's Frenzy** — the 1–14 result's "any number of creatures" is
//!   every creature your opponents control.
//! - **Component Pouch** — "two mana of different colors" may be one color
//!   twice.
//! - **Dragonborn Champion** — damage to its controller doesn't draw.
//! - **Druid of Purification** — the choosing starts with the next player, and
//!   every player chooses (no "may").
//! - **Klauth** — "spend this mana only to cast spells" isn't enforced.
//! - **Sword of Hours** — "the damage dealt" is the damage to each recipient,
//!   one roll per recipient.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, Keyword, SelectionRequirement as R, Selector, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{cast_is_noncreature, draw, etb, on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate,
    StaticAbility, StaticEffect, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, x};

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
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// CR 706 — one die, a results table, nothing else.
fn roll(sides: u8, results: Vec<(u8, u8, Effect)>) -> Effect {
    Effect::RollDie {
        sides,
        count: Value::ONE,
        modifier: Value::Const(0),
        reroll_at_most: 0,
        ignore_lowest: 0,
        results,
        on_doubles: None,
    }
}

fn plus_one(what: Selector, amount: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount }
}

fn self_keyword(mana: ManaCost, keyword: Keyword) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect: Effect::GrantKeyword { what: Selector::This, keyword, duration: Duration::EndOfTurn },
        ..Default::default()
    }
}

/// Bag of Tricks — {4}{G}, {T}: roll a d8; reveal until a creature card of
/// that mana value, put it onto the battlefield, the rest on the bottom.
pub fn bag_of_tricks() -> CardDefinition {
    let arms = (1..=8u8)
        .map(|n| {
            (
                n,
                n,
                Effect::RevealUntilOneToBattlefieldRestBottom {
                    filter: R::Creature.and(R::ManaValueExactly(n as u32)),
                    damage_controller: false,
                },
            )
        })
        .collect();
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            tap_cost: true,
            effect: roll(8, arms),
            ..Default::default()
        }],
        ..spell("Bag of Tricks", cost(&[generic(1), g()]), CardType::Artifact, Effect::Noop)
    }
}

/// Berserker's Frenzy — before blockers: roll two d20, keep the higher. 1–14:
/// creatures block this turn if able; 15–20: you choose this turn's blocks.
/// Residual: the 1–14 "any number of creatures" is every creature your
/// opponents control.
pub fn berserkers_frenzy() -> CardDefinition {
    CardDefinition {
        cast_only_before_blockers_step: true,
        ..spell(
            "Berserker's Frenzy",
            cost(&[generic(2), r()]),
            CardType::Instant,
            Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::Const(0),
                reroll_at_most: 0,
                ignore_lowest: 1,
                results: vec![
                    (
                        1,
                        14,
                        Effect::GrantKeyword {
                            what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                            keyword: Keyword::MustBlock,
                            duration: Duration::EndOfTurn,
                        },
                    ),
                    (15, 20, Effect::ChooseBlocksThisTurn),
                ],
                on_doubles: None,
            },
        )
    }
}

/// Chaos Dragon — flying, haste, attacks each combat; at the beginning of
/// combat on your turn each player rolls a d20, and it can't attack the
/// opponents who rolled highest this combat.
pub fn chaos_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::MustAttack],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::EachPlayerRollsSourceCantAttackHighest { sides: 20 },
        }],
        ..creature("Chaos Dragon", cost(&[generic(1), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Component Pouch — {T}, remove a component counter: two mana of different
/// colors; {T}: roll a d20, 1–9 one component counter, 10–20 two.
/// Residual: the two colors may match.
pub fn component_pouch() -> CardDefinition {
    let counters = |n: i32| Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Component,
        amount: Value::Const(n),
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Component, 1)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyColors(Value::Const(2)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: roll(20, vec![(1, 9, counters(1)), (10, 20, counters(2))]),
                ..Default::default()
            },
        ],
        ..spell("Component Pouch", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Dragonborn Champion — trample; whenever a source you control deals 5 or
/// more damage to a player, draw a card. Residual: damage to you doesn't draw.
pub fn dragonborn_champion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::YourSourceDamagedOpponent)
                .with_filter(Predicate::ValueAtLeast(Value::TriggerEventAmount, Value::Const(5))),
            effect: draw(1),
        }],
        ..creature(
            "Dragonborn Champion",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Dragon, CreatureType::Warrior],
            5,
            3,
        )
    }
}

/// Druid of Purification — enters: each player chooses an artifact or
/// enchantment you don't control, starting with you; destroy each chosen.
/// Residual: nobody may decline.
pub fn druid_of_purification() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::EachPlayerChoosesToDestroy {
            filter: R::Artifact.or(R::Enchantment).and(R::ControlledByOpponent),
            starting_with_you: true,
        })],
        ..creature(
            "Druid of Purification",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            2,
            3,
        )
    }
}

/// Earth-Cult Elemental — enters: roll a d20. 1–9 each player sacrifices a
/// permanent, 10–19 each opponent does, 20 each opponent sacrifices two.
pub fn earth_cult_elemental() -> CardDefinition {
    let sac = |who: PlayerRef, n: i32| Effect::Sacrifice {
        who: Selector::Player(who),
        count: Value::Const(n),
        filter: R::Permanent,
    };
    CardDefinition {
        triggered_abilities: vec![etb(roll(
            20,
            vec![
                (1, 9, sac(PlayerRef::EachPlayer, 1)),
                (10, 19, sac(PlayerRef::EachOpponent, 1)),
                (20, 20, sac(PlayerRef::EachOpponent, 2)),
            ],
        ))],
        ..creature(
            "Earth-Cult Elemental",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Elemental],
            6,
            6,
        )
    }
}

/// Klauth's Will — choose one (both with a commander): X damage to each
/// creature without flying; destroy up to X artifacts and/or enchantments.
pub fn klauths_will() -> CardDefinition {
    let modes = || {
        vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
                amount: Value::XFromCost,
            },
            Effect::CapTargetsAt {
                amount: Value::XFromCost,
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 20,
                    min_targets: 0,
                    filter: R::Artifact.or(R::Enchantment),
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                }),
            },
        ]
    };
    spell(
        "Klauth's Will",
        cost(&[x(), r(), r(), g()]),
        CardType::Instant,
        Effect::If {
            cond: Predicate::YouControlACommander,
            then: Box::new(Effect::ChooseN { picks: vec![0, 1], modes: modes() }),
            else_: Box::new(Effect::ChooseMode(modes())),
        },
    )
}

/// Klauth, Unrivaled Ancient — flying, haste; attacking, add mana in any
/// colors equal to the attackers' total power, kept until end of turn.
/// Residual: the "only to cast spells" restriction isn't enforced.
pub fn klauth_unrivaled_ancient() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::AddManaKeptThisTurnAnyColors {
            who: PlayerRef::You,
            amount: Value::PowerOf(Box::new(Selector::EachPermanent(R::IsAttacking))),
        })],
        ..creature(
            "Klauth, Unrivaled Ancient",
            cost(&[generic(5), r(), g()]),
            vec![CreatureType::Dragon],
            4,
            4,
        )
    }
}

/// Maddening Hex — enchant player; whenever they cast a noncreature spell,
/// roll a d6, deal that much damage to them, then move to another opponent
/// at random.
pub fn maddening_hex() -> CardDefinition {
    CardDefinition {
        name: "Maddening Hex",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::All(vec![
                    Predicate::SamePlayer(PlayerRef::Triggerer, PlayerRef::EnchantedPlayer),
                    cast_is_noncreature(),
                ]),
            ),
            effect: Effect::Seq(vec![
                roll(
                    6,
                    vec![(
                        1,
                        6,
                        Effect::DealDamage {
                            to: Selector::Player(PlayerRef::EnchantedPlayer),
                            amount: Value::LastDieRoll,
                        },
                    )],
                ),
                Effect::Attach {
                    what: Selector::This,
                    to: Selector::Player(PlayerRef::RandomOtherOpponentThanEnchanted),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Neverwinter Hydra — as it enters, roll X d6 for its +1/+1 counters;
/// trample, ward {4}.
pub fn neverwinter_hydra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Ward(WardCost::generic(4))],
        as_enters_effect: Some(Effect::RollDie {
            sides: 6,
            count: Value::XFromCost,
            modifier: Value::Const(0),
            reroll_at_most: 0,
            ignore_lowest: 0,
            results: vec![(1, 6, plus_one(Selector::This, Value::LastDieRoll))],
            on_doubles: None,
        }),
        ..creature(
            "Neverwinter Hydra",
            cost(&[x(), x(), g(), g()]),
            vec![CreatureType::Hydra],
            0,
            0,
        )
    }
}

/// Rile — 1 damage to your creature, it gains trample; draw a card.
pub fn rile() -> CardDefinition {
    spell(
        "Rile",
        cost(&[r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByYou)),
                amount: Value::ONE,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Skyship Stalker — flying; {R}: +1/+0, first strike, or haste until end of
/// turn.
pub fn skyship_stalker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            self_keyword(cost(&[r()]), Keyword::FirstStrike),
            self_keyword(cost(&[r()]), Keyword::Haste),
        ],
        ..creature(
            "Skyship Stalker",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Cat, CreatureType::Dragon],
            3,
            3,
        )
    }
}

/// Sword of Hours — equipped creature attacking gets a +1/+1 counter; dealing
/// combat damage, roll a d12 and double its +1/+1 counters on a 12 or a
/// result above the damage. Equip {2}. Residual: one roll per recipient.
pub fn sword_of_hours() -> CardDefinition {
    let double = || Effect::DoubleCountersOnEach { what: Selector::This, kind: CounterType::PlusOnePlusOne };
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![
                TriggeredAbility {
                    event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                    effect: plus_one(Selector::This, Value::ONE),
                },
                TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamage, EventScope::SelfSource),
                    effect: roll(
                        12,
                        vec![
                            (12, 12, double()),
                            (
                                1,
                                11,
                                Effect::If {
                                    cond: Predicate::ValueAtLeast(
                                        Value::LastDieRoll,
                                        Value::Sum(vec![Value::TriggerEventAmount, Value::ONE]),
                                    ),
                                    then: Box::new(double()),
                                    else_: Box::new(Effect::Noop),
                                },
                            ),
                        ],
                    ),
                },
            ],
            ..Default::default()
        }),
        ..spell("Sword of Hours", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Underdark Rift — {T}: {C}. {5}, {T}, exile it: roll a d10; put target
/// artifact, creature or planeswalker beneath the top that many cards of its
/// owner's library. Sorcery speed.
pub fn underdark_rift() -> CardDefinition {
    CardDefinition {
        name: "Underdark Rift",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                exile_self_cost: true,
                sorcery_speed: true,
                effect: roll(
                    10,
                    vec![(
                        1,
                        10,
                        Effect::PutIntoLibraryBeneathTop {
                            what: target_filtered(R::Artifact.or(R::Creature).or(R::Planeswalker)),
                            count: Value::LastDieRoll,
                        },
                    )],
                ),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn dragon_spirit() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Dragon Spirit".into(),
        power: 5,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Spirit],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
            effect: Effect::SacrificeSource,
        }],
        ..Default::default()
    })
}

/// Vrondiss, Rage of Ancients — enrage: you may create a 5/4 Dragon Spirit
/// that sacrifices itself when it deals damage; whenever you roll dice, you
/// may have Vrondiss deal 1 damage to itself.
pub fn vrondiss_rage_of_ancients() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Create a 5/4 Dragon Spirit?".into(),
                    body: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: dragon_spirit(),
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::RolledDice, EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Have Vrondiss deal 1 damage to itself?".into(),
                    body: Box::new(Effect::DealDamage { to: Selector::This, amount: Value::ONE }),
                },
            },
        ],
        ..creature(
            "Vrondiss, Rage of Ancients",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Dragon, CreatureType::Barbarian],
            5,
            4,
        )
    }
}

/// Wild Endeavor — roll two d4 and choose: that many 3/3 Beasts, and search
/// for basic lands equal to the other result, onto the battlefield tapped.
pub fn wild_endeavor() -> CardDefinition {
    let beast = Arc::new(TokenDefinition {
        name: "Beast".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        ..Default::default()
    });
    spell(
        "Wild Endeavor",
        cost(&[generic(4), g(), g()]),
        CardType::Sorcery,
        Effect::RollTwoDiceAssign {
            sides: 4,
            first: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::LastDieRoll,
                definition: beast,
            }),
            second: Box::new(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::LastDieRoll,
            }),
        },
    )
}

/// Wulfgar of Icewind Dale — melee; your attack triggers from attacking
/// creatures trigger an additional time.
pub fn wulfgar_of_icewind_dale() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Melee],
        static_abilities: vec![StaticAbility {
            description: "If a creature you control attacking causes a triggered ability of a \
                          permanent you control to trigger, it triggers an additional time.",
            effect: StaticEffect::DoubleControllerAttackTriggers,
        }],
        ..creature(
            "Wulfgar of Icewind Dale",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Human, CreatureType::Barbarian],
            4,
            4,
        )
    }
}
