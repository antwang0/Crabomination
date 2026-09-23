//! Commander: the cards the **Vampiric Bloodlust** precon (C17, Edgar Markov)
//! needed beyond what the catalog had. Tests in `tests/recent_b/cmdr_fdc.rs`
//! (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Bloodlord of Vaasgoth** — the granted bloodthirst is checked as the
//!   cast trigger resolves, not as the creature enters.
//! - **Mathas, Fiend Seeker** — a bounty counter grants its dies trigger only
//!   while Mathas is on the battlefield.
//! - **Kheru Mind-Eater** — the exiled card is exiled face up.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EnterMode, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, generic, r, w, x};
use crate::sets::{enters_tapped, tap_add};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
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

fn tapland(name: &'static str, a: Color, bb: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add(a), tap_add(bb)],
        static_abilities: vec![enters_tapped()],
        ..Default::default()
    }
}

fn curse(name: &'static str, mana: crate::mana::ManaCost, each: impl Fn(PlayerRef) -> Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        // CR 506.2 — "is attacked": the active player attacked the cursed
        // player with at least one creature; the attacker profits too when
        // they are your opponent.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
                Predicate::AttackedDefenderWithCountAtLeast {
                    who: PlayerRef::ActivePlayer,
                    defender: PlayerRef::EnchantedPlayer,
                    at_least: 1,
                    include_planeswalkers: false,
                },
            ),
            effect: Effect::Seq(vec![
                each(PlayerRef::You),
                Effect::If {
                    cond: Predicate::PlayerIsOpponent { who: PlayerRef::ActivePlayer },
                    then: Box::new(each(PlayerRef::ActivePlayer)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Bloodlord of Vaasgoth — bloodthirst 3, flying; your Vampire creature
/// spells gain bloodthirst 3. ⚠ The granted bloodthirst is checked as the
/// cast trigger resolves, not as the creature enters.
pub fn bloodlord_of_vaasgoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Bloodthirst(3), Keyword::Flying],
        triggered_abilities: vec![
            crate::effect::shortcut::bloodthirst(3),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::HasCreatureType(CreatureType::Vampire)),
                    },
                ),
                effect: Effect::If {
                    cond: Predicate::PlayerDamagedThisTurn { who: PlayerRef::EachOpponent },
                    then: Box::new(Effect::SpellEntersWithCounters {
                        what: Selector::TriggerSource,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(3),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..creature(
            "Bloodlord of Vaasgoth",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Consuming Vapors — target player sacrifices a creature; you gain life
/// equal to its toughness. Rebound.
pub fn consuming_vapors() -> CardDefinition {
    CardDefinition {
        name: "Consuming Vapors",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Rebound],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: target_filtered(R::Player),
                count: Value::ONE,
                filter: R::Creature,
            },
            Effect::GainLife { who: Selector::You, amount: Value::SacrificedToughness },
        ]),
        ..Default::default()
    }
}

/// Crimson Honor Guard — each end step, 4 damage to that turn's player unless
/// they control a commander.
pub fn crimson_honor_guard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::PlayerControlsACommander { who: PlayerRef::ActivePlayer },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::Const(4),
                }),
            },
        }],
        ..creature(
            "Crimson Honor Guard",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Vampire, CreatureType::Knight],
            4,
            5,
        )
    }
}

fn zombie_token() -> TokenDefinition {
    TokenDefinition {
        name: "Zombie".to_string(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    }
}

/// Curse of Disturbance — the cursed player being attacked makes you (and an
/// attacking opponent) a 2/2 Zombie.
pub fn curse_of_disturbance() -> CardDefinition {
    curse("Curse of Disturbance", cost(&[generic(2), b()]), |who| Effect::CreateToken {
        who,
        count: Value::ONE,
        definition: Arc::new(zombie_token()),
    })
}

/// Curse of Vitality — the cursed player being attacked gains you (and an
/// attacking opponent) 2 life.
pub fn curse_of_vitality() -> CardDefinition {
    curse("Curse of Vitality", cost(&[generic(2), w()]), |who| Effect::GainLife {
        who: Selector::Player(who),
        amount: Value::Const(2),
    })
}

/// Dark Impostor — exile a creature and grow; it has the activated abilities
/// of the creature cards exiled with it.
pub fn dark_impostor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b(), b()]),
            effect: Effect::Seq(vec![
                Effect::ExileTaggedWithSource { what: target_filtered(R::Creature) },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "This creature has all activated abilities of all creature cards \
                          exiled with it.",
            effect: StaticEffect::HasActivatedAbilitiesOfExiledWithSelf,
        }],
        ..creature(
            "Dark Impostor",
            cost(&[generic(2), b()]),
            vec![CreatureType::Vampire, CreatureType::Assassin],
            2,
            2,
        )
    }
}

/// Drana, Kalastria Bloodchief — {X}{B}{B}: target creature gets -0/-X and
/// Drana gets +X/+0.
pub fn drana_kalastria_bloodchief() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), b(), b()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ZERO,
                    toughness: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(-1))),
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::XFromCost,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Drana, Kalastria Bloodchief",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Shaman],
            4,
            4,
        )
    }
}

/// Fell the Mighty — destroy every creature with power greater than the
/// target creature's.
pub fn fell_the_mighty() -> CardDefinition {
    CardDefinition {
        name: "Fell the Mighty",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: Selector::PowerAbove {
                inner: Box::new(Selector::EachPermanent(R::Creature)),
                than: Box::new(Value::PowerOf(Box::new(target_filtered(R::Creature)))),
            },
        },
        ..Default::default()
    }
}

/// Forsaken Sanctuary — W/B tapland.
pub fn forsaken_sanctuary() -> CardDefinition {
    tapland("Forsaken Sanctuary", Color::White, Color::Black)
}

/// Stone Quarry — R/W tapland.
pub fn stone_quarry() -> CardDefinition {
    tapland("Stone Quarry", Color::Red, Color::White)
}

/// Kheru Mind-Eater — the player it hits exiles a card from hand, which you
/// may play. ⚠ The card is exiled face up.
pub fn kheru_mind_eater() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileFromHand {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: MayPlayDuration::WhileExiled,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ]),
        }],
        ..creature("Kheru Mind-Eater", cost(&[generic(2), b()]), vec![CreatureType::Vampire], 1, 3)
    }
}

/// Kindred Charge — choose a type; a hasty token copy of each creature of
/// yours of that type, exiled at the next end step.
pub fn kindred_charge() -> CardDefinition {
    CardDefinition {
        name: "Kindred Charge",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::ForEach {
                selector: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::IsSourceChosenCreatureType),
                ),
                body: Box::new(Effect::CreateTokenCopiesHasteSac {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    exile: true,
                }),
            }),
        },
        ..Default::default()
    }
}

/// Licia, Sanguine Tribune — {1} cheaper per life gained this turn; pay 5
/// life on your turn, once a turn, for three +1/+1 counters.
pub fn licia_sanguine_tribune() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each 1 life you gained this turn.",
            effect: StaticEffect::SelfCostReducedByValue {
                amount: Value::LifeGainedThisTurn(PlayerRef::You),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 5,
            once_per_turn: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..creature(
            "Licia, Sanguine Tribune",
            cost(&[generic(5), r(), w(), b()]),
            vec![CreatureType::Vampire, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Mathas, Fiend Seeker — a bounty counter each end step; a creature with one
/// has "when this dies, each opponent draws a card and gains 2 life". ⚠ The
/// grant lasts only while Mathas is on the battlefield.
pub fn mathas_fiend_seeker() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                kind: CounterType::Bounty,
                amount: Value::ONE,
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "A creature with a bounty counter has \"When this creature dies, each \
                          opponent draws a card and gains 2 life.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::WithCounter(CounterType::Bounty)),
                ability: Box::new(on_dies(Effect::Seq(vec![
                    Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                    Effect::GainLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(2),
                    },
                ]))),
            },
        }],
        ..creature(
            "Mathas, Fiend Seeker",
            cost(&[r(), w(), b()]),
            vec![CreatureType::Vampire],
            3,
            3,
        )
    }
}

/// Outpost Siege — Khans: impulse-draw each upkeep; Dragons: a creature of
/// yours leaving pings any target.
pub fn outpost_siege() -> CardDefinition {
    CardDefinition {
        name: "Outpost Siege",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        enter_modes: Some(vec![
            EnterMode {
                label: "Khans",
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::StepBegins(TurnStep::Upkeep),
                        EventScope::YourControl,
                    ),
                    effect: Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        duration: MayPlayDuration::EndOfThisTurn,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: true,
                        uncast_penalty: None,
                    },
                }],
                ..Default::default()
            },
            EnterMode {
                label: "Dragons",
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl)
                        .with_filter(Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::Creature,
                        }),
                    effect: Effect::DealDamage {
                        to: target_filtered(R::Any),
                        amount: Value::ONE,
                    },
                }],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Vein Drinker — {R}, {T}: fight target creature; it grows when a creature
/// it damaged this turn dies.
pub fn vein_drinker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            effect: Effect::Fight { attacker: Selector::This, defender: target_filtered(R::Creature) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::DamagedBySourceThisTurn,
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Vein Drinker", cost(&[generic(4), b(), b()]), vec![CreatureType::Vampire], 4, 4)
    }
}
