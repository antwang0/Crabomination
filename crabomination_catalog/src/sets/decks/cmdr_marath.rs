//! Commander: the cards the **Nature of the Beast** precon (C13, Marath,
//! Will of the Wild) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_marath.rs`.
//!
//! Residuals (each also on its card):
//! - **Fiery Justice** — the 5 life goes to the most hostile opponent (the
//!   engine's pick of "target opponent").
//! - **Magus of the Arena** — you pick the opponent's creature; the
//!   opponent should.
//! - **Naya Soulbeast** — the top cards are read as it enters, not revealed
//!   as it is cast.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_attack, target_any, target_filtered};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, PlayerStaticTarget, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, w, x};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn big() -> R {
    R::Creature.and(R::PowerAtLeast(5))
}

fn green_token(name: &str, ct: CreatureType, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        keywords,
        ..Default::default()
    }
}

/// Marath, Will of the Wild — enters with a +1/+1 counter per mana spent to
/// cast it; {X}, remove X +1/+1 counters: X counters on a creature, X damage
/// to any target, or an X/X Elemental.
pub fn marath_will_of_the_wild() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::CastSpellManaSpent)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            remove_counter_x: Some(CounterType::PlusOnePlusOne),
            effect: Effect::ChooseMode(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::XFromCost,
                },
                Effect::DealDamage { to: target_any(), amount: Value::XFromCost },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(
                        green_token("Elemental", CreatureType::Elemental, 0, 0, vec![])
                            .entering_with(CounterType::PlusOnePlusOne, Value::XFromCost),
                    ),
                },
            ]),
            ..Default::default()
        }],
        ..legendary(creature(
            "Marath, Will of the Wild",
            cost(&[r(), g(), w()]),
            vec![CreatureType::Elemental, CreatureType::Beast],
            0,
            0,
        ))
    }
}

/// Curse of Chaos — enchant player; whenever a player attacks them, that
/// attacker may discard a card to draw one.
pub fn curse_of_chaos() -> CardDefinition {
    CardDefinition {
        name: "Curse of Chaos",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
                Predicate::AttackedDefenderWithCountAtLeast {
                    who: PlayerRef::ActivePlayer,
                    defender: PlayerRef::EnchantedPlayer,
                    at_least: 1,
                    include_planeswalkers: false,
                },
            ),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::ActivePlayer), Value::ONE),
                then: Box::new(Effect::MayDoBy {
                    who: PlayerRef::ActivePlayer,
                    description: "Discard a card to draw a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard {
                            who: Selector::Player(PlayerRef::ActivePlayer),
                            amount: Value::ONE,
                            random: false,
                        },
                        Effect::Draw { who: Selector::Player(PlayerRef::ActivePlayer), amount: Value::ONE },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Fiery Justice — 5 damage divided among any number of targets; target
/// opponent gains 5 life.
///
/// ⚠ Residual: the life goes to the engine's most hostile opponent.
pub fn fiery_justice() -> CardDefinition {
    CardDefinition {
        name: "Fiery Justice",
        cost: cost(&[r(), g(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamageDivided {
                total: Value::Const(5),
                filter: R::Creature.or(R::Planeswalker).or(R::Player),
                max_targets: 5,
                retaliate_to_source: false,
            },
            Effect::GainLife { who: Selector::Player(PlayerRef::HostileOpponent), amount: Value::Const(5) },
        ]),
        ..Default::default()
    }
}

/// From the Ashes — destroy all nonbasic lands; each player may search for a
/// basic land per land of theirs destroyed.
pub fn from_the_ashes() -> CardDefinition {
    CardDefinition {
        name: "From the Ashes",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::Land.and(R::Not(Box::new(R::IsBasicLand)))) },
            Effect::EachPlayerDoes {
                who: PlayerRef::EachPlayer,
                body: Box::new(Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    count: Value::PermanentsDestroyedThisResolutionControlledBy(PlayerRef::You),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Gahiji, Honored One — a creature attacking one of your opponents gets
/// +2/+0 until end of turn.
pub fn gahiji_honored_one() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::IsAttackingAnOpponent,
            }),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..legendary(creature(
            "Gahiji, Honored One",
            cost(&[generic(2), r(), g(), w()]),
            vec![CreatureType::Beast],
            4,
            4,
        ))
    }
}

/// Magus of the Arena — {3}, {T}: tap a creature you control and an
/// opponent's creature; they fight.
///
/// ⚠ Residual: you pick the opponent's creature.
pub fn magus_of_the_arena() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Tap { what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) } },
                Effect::Tap {
                    what: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                },
                Effect::Fight { attacker: Selector::Target(0), defender: Selector::Target(1) },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Magus of the Arena",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            5,
            5,
        )
    }
}

/// Mayael the Anima — {3}{R}{G}{W}, {T}: look at the top five; a creature
/// with power 5 or greater may go onto the battlefield.
pub fn mayael_the_anima() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r(), g(), w()]),
            tap_cost: true,
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(5),
                pick_filter: Some(big()),
                take: Some(Value::ONE),
                to_battlefield: true,
                optional: true,
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..legendary(creature(
            "Mayael the Anima",
            cost(&[r(), g(), w()]),
            vec![CreatureType::Elf, CreatureType::Shaman],
            2,
            3,
        ))
    }
}

/// Mystic Barrier — on entry and at your upkeep, choose left or right; each
/// player may attack only the nearest opponent that way.
pub fn mystic_barrier() -> CardDefinition {
    CardDefinition {
        name: "Mystic Barrier",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::ChooseAttackDirection),
        static_abilities: vec![StaticAbility {
            description: "Each player may attack only the nearest opponent in the last chosen direction.",
            effect: StaticEffect::AttackOnlyNearestOpponentInChosenDirection,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::ChooseAttackDirection,
        }],
        ..Default::default()
    }
}

/// Naya Soulbeast — trample; enters with a +1/+1 counter per mana value of
/// each player's top card.
///
/// ⚠ Residual: the top cards are read as it enters, not revealed on cast.
pub fn naya_soulbeast() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::TotalManaValueOf(Box::new(Selector::TopOfLibrary { who: PlayerRef::EachPlayer, count: Value::ONE })),
        )),
        ..creature("Naya Soulbeast", cost(&[generic(6), g(), g()]), vec![CreatureType::Beast], 0, 0)
    }
}

/// Rain of Thorns — choose one or more: destroy target artifact, target
/// enchantment, target land.
pub fn rain_of_thorns() -> CardDefinition {
    let destroy = |filter: R| Effect::Destroy { what: target_filtered(filter) };
    CardDefinition {
        name: "Rain of Thorns",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![destroy(R::Artifact), destroy(R::Enchantment), destroy(R::Land)],
            min: 1,
            max: 3,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Rakeclaw Gargantuan — {1}: a creature with power 5 or greater gains first
/// strike until end of turn.
pub fn rakeclaw_gargantuan() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(big()),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Rakeclaw Gargantuan",
            cost(&[generic(2), r(), g(), w()]),
            vec![CreatureType::Beast],
            5,
            3,
        )
    }
}

/// Spawning Grounds — enchant land; it has "{T}: create a 5/5 green Beast
/// with trample".
pub fn spawning_grounds() -> CardDefinition {
    CardDefinition {
        name: "Spawning Grounds",
        cost: cost(&[generic(6), g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(green_token("Beast", CreatureType::Beast, 5, 5, vec![Keyword::Trample])),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Spellbreaker Behemoth — can't be countered; your creature spells with
/// power 5 or greater can't be countered.
pub fn spellbreaker_behemoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        static_abilities: vec![StaticAbility {
            description: "Creature spells you control with power 5 or greater can't be countered.",
            effect: StaticEffect::SpellsUncounterable { filter: big().and(R::ControlledByYou) },
        }],
        ..creature(
            "Spellbreaker Behemoth",
            cost(&[generic(1), r(), g(), g()]),
            vec![CreatureType::Beast],
            5,
            5,
        )
    }
}

/// Terra Ravager — attacking, +X/+0 where X is the defending player's land
/// count.
pub fn terra_ravager() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByDefendingPlayer))),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Terra Ravager",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Beast],
            0,
            4,
        )
    }
}

/// Where Ancients Tread — a creature of yours with power 5 or greater
/// entering may deal 5 damage to any target.
pub fn where_ancients_tread() -> CardDefinition {
    CardDefinition {
        name: "Where Ancients Tread",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: big() }),
            effect: Effect::MayDo {
                description: "Deal 5 damage to any target?".into(),
                body: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(5) }),
            },
        }],
        ..Default::default()
    }
}

/// Witch Hunt — players can't gain life; 4 damage to you at your upkeep; at
/// your end step a random opponent gains control of it.
pub fn witch_hunt() -> CardDefinition {
    CardDefinition {
        name: "Witch Hunt",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Players can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife { target: PlayerStaticTarget::EachPlayer },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::DealDamage { to: Selector::You, amount: Value::Const(4) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::RandomOpponent),
                    duration: Duration::Permanent,
                },
            },
        ],
        ..Default::default()
    }
}
