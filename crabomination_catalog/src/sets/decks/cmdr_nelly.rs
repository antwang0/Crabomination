//! Commander: the cards the **Blame Game** precon (MKC, Nelly Borca) needed
//! beyond what the catalog had. Tests in `tests/recent_b/cmdr_nelly.rs`.
//!
//! Residuals (each also on its card):
//! - **Agitator Ant** — each taker's counters go on their greatest-power
//!   creature rather than one they choose.
//! - **Feather, Radiant Arbiter** — a headless caster only copies onto its
//!   own creatures (a person is offered every legal creature).

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, GoadLasts, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, r, w};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn goaded_static() -> StaticAbility {
    StaticAbility { description: "Enchanted creature is goaded.", effect: StaticEffect::AttachedIsGoaded }
}

fn draw(who: PlayerRef) -> Effect {
    Effect::Draw { who: Selector::Player(who), amount: Value::ONE }
}

fn soldier() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    }
}

/// Nelly Borca, Impulsive Accuser — vigilance; attacking, suspect target
/// creature, then goad every suspected creature; when an opponent's creatures
/// connect with another of your opponents, you and their controller draw.
pub fn nelly_borca_impulsive_accuser() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            on_attack(Effect::Seq(vec![
                Effect::Suspect { what: target_filtered(R::Creature) },
                Effect::Goad { what: Selector::EachPermanent(R::Creature.and(R::IsSuspected)) },
            ])),
            // CR 603.2c — once per combat-damage step however many opponents
            // were hit; the filter reads each hit (dealer and damaged seat both
            // your opponents) before the batch folds.
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer)
                    .with_filter(Predicate::All(vec![
                        Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer },
                        Predicate::PlayerIsOpponent {
                            who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                        },
                    ]))
                    .once_per_batch_across_players(),
                effect: Effect::Seq(vec![
                    draw(PlayerRef::You),
                    draw(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                ]),
            },
        ],
        ..creature(
            "Nelly Borca, Impulsive Accuser",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Human, CreatureType::Detective],
            2,
            4,
        )
    }
}

/// Agitator Ant — at your end step each player may put two +1/+1 counters on
/// a creature they control; each such creature is goaded. ⚠ The counters go
/// on the taker's greatest-power creature (`EachPlayerMayCounterThenGoad`).
pub fn agitator_ant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::EachPlayerMayCounterThenGoad { counters: 2 },
        }],
        ..creature("Agitator Ant", cost(&[generic(2), r()]), vec![CreatureType::Insect], 2, 2)
    }
}

/// Bloodthirsty Blade — equipped creature gets +2/+0 and is goaded; {1}:
/// attach it to target creature an opponent controls, as a sorcery. No equip.
pub fn bloodthirsty_blade() -> CardDefinition {
    CardDefinition {
        name: "Bloodthirsty Blade",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        equipped_bonus: Some(EquipBonus { power: 2, ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Equipped creature is goaded.",
            effect: StaticEffect::AttachedIsGoaded,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            effect: Effect::AttachSourceTo { host: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Darien, King of Kjeldor — whenever you're dealt damage, you may create
/// that many 1/1 white Soldiers.
pub fn darien_king_of_kjeldor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Create that many 1/1 Soldiers?".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::TriggerEventAmount,
                    definition: Arc::new(soldier()),
                }),
            },
        }],
        ..creature(
            "Darien, King of Kjeldor",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Feather, Radiant Arbiter — flying, lifelink; a noncreature spell you cast
/// aimed only at Feather may be copied onto other creatures at {2} apiece.
/// ⚠ A headless caster copies onto its own creatures only.
pub fn feather_radiant_arbiter() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Not(Box::new(R::Creature)).and(R::SpellTargetsOnlySource),
                },
            ),
            effect: Effect::MayPayToCopyOntoOtherCreatures { per_copy: cost(&[generic(2)]) },
        }],
        ..creature("Feather, Radiant Arbiter", cost(&[r(), w(), w()]), vec![CreatureType::Angel], 4, 3)
    }
}

/// Fiendish Duo — first strike; damage to an opponent is doubled.
pub fn fiendish_duo() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "If a source would deal damage to an opponent, it deals double that damage instead.",
            effect: StaticEffect::DoubleDamageToOpponentPlayers,
        }],
        ..creature("Fiendish Duo", cost(&[generic(4), r(), r()]), vec![CreatureType::Devil], 5, 5)
    }
}

/// Havoc Eater — flying; on entry, goad up to one creature per opponent and
/// grow by their total power.
pub fn havoc_eater() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ForEachOpponentTarget {
            // The wrapper caps the slots at one per opponent; this is the total.
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 15,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Goad { what: Selector::Target(0) },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::PowerOf(Box::new(Selector::Target(0))),
                    },
                ])),
            }),
        })],
        ..creature("Havoc Eater", cost(&[generic(5), r(), r()]), vec![CreatureType::Elemental], 3, 3)
    }
}

/// Hot Pursuit — on entry, suspect an opponent's creature, goaded while this
/// stays; with two or more players out, your combats steal every goaded or
/// suspected creature for the turn.
pub fn hot_pursuit() -> CardDefinition {
    CardDefinition {
        name: "Hot Pursuit",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Suspect { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
                Effect::GoadWhile { what: Selector::Target(0), hold: GoadLasts::WhileSourceOnBattlefield },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl)
                    .with_filter(Predicate::PlayersLostAtLeast(2)),
                effect: Effect::ForEach {
                    selector: Selector::EachPermanent(R::Creature.and(R::IsGoaded.or(R::IsSuspected))),
                    body: Box::new(Effect::Seq(vec![
                        Effect::GainControl { what: Selector::TriggerSource, to: None, duration: Duration::EndOfTurn },
                        Effect::Untap { what: Selector::TriggerSource, up_to: None },
                        Effect::GrantKeyword {
                            what: Selector::TriggerSource,
                            keyword: Keyword::Haste,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// Immortal Obligation — return an opponent's creature card to the battlefield
/// under their control with a duty counter: while it has one it is goaded
/// and can't attack you or yours, nor block your creatures.
pub fn immortal_obligation() -> CardDefinition {
    spell(
        "Immortal Obligation",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InOpponentGraveyard)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    tapped: false,
                },
            },
            Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Duty, amount: Value::ONE },
            Effect::GoadWhile { what: Selector::Target(0), hold: GoadLasts::Obligation(CounterType::Duty) },
        ]),
    )
}

/// Mob Verdict — secret council: each player votes for another player; each
/// vote an opponent got is 2 damage to them and each of their creatures, each
/// vote you got a card.
pub fn mob_verdict() -> CardDefinition {
    spell(
        "Mob Verdict",
        cost(&[generic(2), r(), r()]),
        CardType::Sorcery,
        Effect::EachPlayerVotesForAPlayer {
            on_opponent: Box::new(Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(2) },
                Effect::DealDamage {
                    to: Selector::ControlledBy { who: PlayerRef::Triggerer, filter: R::Creature },
                    amount: Value::Const(2),
                },
            ])),
            on_you: Box::new(draw(PlayerRef::You)),
        },
    )
}

/// Otherworldly Escort — flash; dying as a non-Spirit, it returns with four
/// charge counters as a Spirit Detective; {1}{W}, {T}, remove a charge
/// counter: destroy a creature that dealt damage to you this turn.
pub fn otherworldly_escort() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::ReturnSelfRetypedWithCounters {
                unless: CreatureType::Spirit,
                types: vec![CreatureType::Spirit, CreatureType::Detective],
                kind: CounterType::Charge,
                amount: 4,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Charge, 1)),
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::DealtDamageToControllerThisTurn)),
            },
            ..Default::default()
        }],
        ..creature(
            "Otherworldly Escort",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Detective],
            4,
            3,
        )
    }
}

/// Prisoner's Dilemma — each opponent secretly chooses silence or snitch:
/// 4 to each if all stay silent, 8 to each if all snitch, else 12 to each
/// silent one. Flashback {5}{R}{R}.
pub fn prisoners_dilemma() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), r(), r()]))],
        ..spell(
            "Prisoner's Dilemma",
            cost(&[generic(3), r(), r()]),
            CardType::Sorcery,
            Effect::OpponentsChooseSilenceOrSnitch { all_silence: 4, all_snitch: 8, mixed: 12 },
        )
    }
}

/// Ransom Note — a Clue that surveils 1 on entry; {2}, sacrifice: cloak your
/// top card, goad target creature, or draw.
pub fn ransom_note() -> CardDefinition {
    CardDefinition {
        name: "Ransom Note",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Clue], ..Default::default() },
        triggered_abilities: vec![etb(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::ChooseMode(vec![
                Effect::Cloak { who: PlayerRef::You, amount: Value::ONE, from_hand: false },
                Effect::Goad { what: target_filtered(R::Creature) },
                draw(PlayerRef::You),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Redemption Arc — enchanted creature has indestructible and is goaded;
/// {1}{W}: exile enchanted creature.
pub fn redemption_arc() -> CardDefinition {
    CardDefinition {
        name: "Redemption Arc",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Indestructible], ..Default::default() }),
        static_abilities: vec![goaded_static()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Exile { what: enchanted() },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Smuggler's Share — each end step, a card per opponent who drew two or more
/// this turn, then a Treasure per opponent who had two or more lands enter.
pub fn smugglers_share() -> CardDefinition {
    let per_opponent = |cond: Predicate, body: Effect| Effect::ForEachOpponent {
        body: Box::new(Effect::If { cond, then: Box::new(body), else_: Box::new(Effect::Noop) }),
    };
    CardDefinition {
        name: "Smuggler's Share",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                per_opponent(
                    Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 2 },
                    draw(PlayerRef::You),
                ),
                per_opponent(
                    Predicate::LandsEnteredThisTurnAtLeast { who: PlayerRef::Triggerer, at_least: 2 },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(crabomination_base::tokens::treasure_token()),
                    },
                ),
            ]),
        }],
        ..Default::default()
    }
}

/// Spectacular Showdown — a double strike counter on target creature, which is
/// then goaded; overload {4}{R}{R}{R} does it to each creature.
pub fn spectacular_showdown() -> CardDefinition {
    let showdown = |what: Selector, goaded: Selector| {
        Effect::Seq(vec![
            Effect::AddKeywordCounter { what, keyword: Keyword::DoubleStrike, amount: Value::ONE },
            Effect::Goad { what: goaded },
        ])
    };
    let each = || Selector::EachPermanent(R::Creature);
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(4), r(), r(), r()]),
            effect_override: Some(showdown(each(), each())),
            ..Default::default()
        }),
        ..spell(
            "Spectacular Showdown",
            cost(&[generic(1), r()]),
            CardType::Sorcery,
            showdown(target_filtered(R::Creature), Selector::Target(0)),
        )
    }
}

/// Take the Bait — only during combat on an opponent's turn: prevent all
/// combat damage to you and your planeswalkers this turn, untap and goad the
/// attackers, and add a combat phase after this one.
pub fn take_the_bait() -> CardDefinition {
    CardDefinition {
        cast_only_during_combat: true,
        cast_condition: Some(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
        ..spell(
            "Take the Bait",
            cost(&[generic(2), r(), w()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::PreventAllCombatDamageToPlayerAndWalkersThisTurn { who: PlayerRef::You },
                Effect::Untap { what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)), up_to: None },
                Effect::Goad { what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)) },
                Effect::AdditionalCombatPhase { count: Value::ONE },
            ]),
        )
    }
}

/// Vengeful Ancestor — flying; entering or attacking, goad target creature;
/// whenever a goaded creature attacks, it deals 1 damage to its controller.
pub fn vengeful_ancestor() -> CardDefinition {
    let goad = || Effect::Goad { what: target_filtered(R::Creature) };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(goad()),
            on_attack(goad()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsGoaded },
                ),
                effect: Effect::DealDamageFrom {
                    source: Selector::TriggerSource,
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                    amount: Value::ONE,
                },
            },
        ],
        ..creature(
            "Vengeful Ancestor",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Spirit, CreatureType::Dragon],
            3,
            4,
        )
    }
}
