//! Commander: 2026-10-19 (`016Xh16H`) — two- and three-gap closers: Old Hob,
//! Kang, Ultimo, Mila. Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility,
    PlaneswalkerSubtype, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{draw, etb, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, r, u, w};
use crate::sets::{dual_land_untyped, etb_gain_one, tapland_untyped};
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

fn spell(name: &'static str, mana: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

// ── Old Hob, Alleycat Blues ─────────────────────────────────────────────────

/// Old Hob, Alleycat Blues — {4}{R} 4/4 legendary Cat Mutant Rebel. Each
/// combat on your turn, a 2/2 red Mutant with haste that's destroyed at the
/// next end step; {1}{W}: target attacking creature token gains indestructible.
pub fn old_hob_alleycat_blues() -> CardDefinition {
    let mutant = TokenDefinition {
        name: "Mutant".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mutant], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(mutant) },
                Effect::GrantKeyword { what: Selector::LastCreatedToken, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                // The delayed destroy is bound to this Mutant now: a deferred
                // `LastCreatedToken` re-reads at the end step and would hit
                // whatever token was made last.
                Effect::DestroyAtNextEndStep { what: Selector::LastCreatedToken },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::IsToken).and(R::IsAttacking)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Old Hob, Alleycat Blues",
            cost(&[generic(4), r()]),
            vec![CreatureType::Cat, CreatureType::Mutant, CreatureType::Rebel],
            4,
            4,
        )
    }
}

/// Mechanized Ninja Cavalry — {1}{R/W} 1/1 artifact Robot Ninja; ETB a 1/1
/// colorless Robot artifact creature token.
pub fn mechanized_ninja_cavalry() -> CardDefinition {
    let robot = TokenDefinition {
        name: "Robot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(robot),
        })],
        ..creature(
            "Mechanized Ninja Cavalry",
            cost(&[generic(1), hybrid(Color::Red, Color::White)]),
            vec![CreatureType::Robot, CreatureType::Ninja],
            1,
            1,
        )
    }
}

// ── Kang, Temporal Tyrant ───────────────────────────────────────────────────

/// A.I.M. Labs — the Dismal Backwater shape: tapped, gain 1, {U} or {B}.
pub fn a_i_m_labs() -> CardDefinition {
    tapland_untyped("A.I.M. Labs", Color::Blue, Color::Black, etb_gain_one())
}

/// Rewrite History — {2}{U} Enchantment — Plan. One or more of your creatures
/// becoming tapped (CR 603.2c) loots and adds a plan counter; the fourth
/// sacrifices it, and when you do (CR 603.7) up to two target instant and/or
/// sorcery cards return from your graveyard to your hand.
pub fn rewrite_history() -> CardDefinition {
    CardDefinition {
        name: "Rewrite History",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Plan], ..Default::default() },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::ControlledByYou),
                    })
                    .once_per_batch(),
                effect: Effect::Seq(vec![
                    draw(1),
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Plan, amount: Value::ONE },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::Plan), EventScope::SelfSource)
                    .with_filter(Predicate::SourceHasCountersAtLeast { counter: CounterType::Plan, n: 4 }),
                effect: Effect::Seq(vec![
                    Effect::SacrificeSource,
                    Effect::If {
                        cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                        then: Box::new(Effect::ReflexiveTrigger {
                            body: Box::new(Effect::ApplyToTargets {
                                max_targets: 2,
                                min_targets: 0,
                                filter: R::InYourGraveyard
                                    .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                                effect: Box::new(Effect::Move {
                                    what: Selector::Target(0),
                                    to: ZoneDest::Hand(PlayerRef::You),
                                }),
                            }),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

// ── Ultimo, Civilization's End ──────────────────────────────────────────────

/// Gix's Caress — {2}{B} sorcery. Target opponent reveals their hand; you
/// choose a nonland card to discard. Create a tapped Powerstone.
pub fn gixs_caress() -> CardDefinition {
    spell(
        "Gix's Caress",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::DiscardChosen {
                from: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: R::Nonland,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    tapped: true,
                    ..crabomination_base::tokens::powerstone_token()
                }),
            },
        ]),
    )
}

/// Okiba Salvage — {4}{B} sorcery. Reanimate a creature or Vehicle card of
/// yours; two +1/+1 counters on it if you control an artifact and an
/// enchantment.
pub fn okiba_salvage() -> CardDefinition {
    let controls = |f: R| Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(f.and(R::ControlledByYou)),
        n: Value::ONE,
    };
    spell(
        "Okiba Salvage",
        cost(&[generic(4), b()]),
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)).from_your_graveyard(),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::If {
                cond: Predicate::All(vec![controls(R::Artifact), controls(R::Enchantment)]),
                then: Box::new(Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Norritt — {3}{B} 1/1 Imp. {T}: untap target blue creature. {T}, before
/// attackers: a non-Wall creature of the active player's that didn't enter
/// this turn must attack, and is destroyed at the next end step unless it did.
pub fn norritt() -> CardDefinition {
    let before_attackers = Predicate::Any(vec![
        Predicate::CurrentStepIs(TurnStep::Untap),
        Predicate::CurrentStepIs(TurnStep::Upkeep),
        Predicate::CurrentStepIs(TurnStep::Draw),
        Predicate::CurrentStepIs(TurnStep::PreCombatMain),
        Predicate::CurrentStepIs(TurnStep::BeginCombat),
    ]);
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Untap {
                    what: target_filtered(R::Creature.and(R::HasColor(Color::Blue))),
                    up_to: None,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(before_attackers),
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(
                            R::Creature
                                .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Wall))))
                                .and(R::ControlledByActivePlayer)
                                .and(R::Not(Box::new(R::EnteredThisTurn))),
                        ),
                        keyword: Keyword::MustAttack,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::DelayUntilWithCapture {
                        kind: DelayedTriggerKind::NextEndStep,
                        capture: Selector::Target(0),
                        body: Box::new(Effect::If {
                            cond: Predicate::EntityMatches {
                                what: Selector::Target(0),
                                filter: R::Not(Box::new(R::AttackedThisTurn)),
                            },
                            then: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                            else_: Box::new(Effect::Noop),
                        }),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..creature("Norritt", cost(&[generic(3), b()]), vec![CreatureType::Imp], 1, 1)
    }
}

// ── Mila, Crafty Companion / Djeru / Tomik, Orzhov Lawmage ──────────────────

/// Gideon of the Trials — {1}{W}{W}, loyalty 3. +1: until your next turn,
/// prevent all damage target permanent would deal. 0: a 4/4 indestructible
/// Human Soldier until end of turn, damage to him prevented. 0: an emblem —
/// while you control a Gideon planeswalker, you can't lose and your opponents
/// can't win (CR 104.3d, CR 114.4).
pub fn gideon_of_the_trials() -> CardDefinition {
    let gideon_out = Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(
            R::Planeswalker.and(R::ControlledByYou).and(R::HasPlaneswalkerType(PlaneswalkerSubtype::Gideon)),
        ),
        n: Value::ONE,
    };
    CardDefinition {
        name: "Gideon of the Trials",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Gideon], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Permanent),
                    keyword: Keyword::DealsNoDamage,
                    duration: Duration::UntilNextTurn,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(4),
                        toughness: Value::Const(4),
                        creature_types: vec![CreatureType::Human, CreatureType::Soldier],
                        keywords: vec![Keyword::Indestructible],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::PreventAllDamageThisTurn { target: Selector::This, redirect_to: None },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Gideon of the Trials".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "As long as you control a Gideon planeswalker, you can't lose the game and your opponents can't win the game.",
                        effect: StaticEffect::WhileCondition {
                            condition: gideon_out,
                            inner: Box::new(StaticEffect::ControllerCantLoseGame),
                        },
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dedicated Commons — land; enters tapped unless you control a
/// planeswalker; {T}: {R} or {W}.
pub fn dedicated_commons() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control a planeswalker.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Planeswalker.and(R::ControlledByYou)),
                    n: Value::ONE,
                },
            },
        }],
        ..dual_land_untyped("Dedicated Commons", Color::Red, Color::White, vec![])
    }
}
