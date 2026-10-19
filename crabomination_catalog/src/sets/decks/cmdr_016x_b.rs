//! Commander: 2026-10-19 (`016Xh16H`) — two- and three-gap closers: Old Hob,
//! Kang, Ultimo, Mila, Quickbeam, Elesh Norn, Elminster, Djeru, Tomik,
//! Tobias Andrion. Tests in
//! `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility,
    PlaneswalkerSubtype, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{draw, etb, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w, x};
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

// ── Quickbeam, Upstart Ent ──────────────────────────────────────────────────

/// Lumberknot — {2}{G}{G} 1/1 hexproof Treefolk; a +1/+1 counter whenever a
/// creature dies.
pub fn lumberknot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Lumberknot", cost(&[generic(2), g(), g()]), vec![CreatureType::Treefolk], 1, 1)
    }
}

// ── Elesh Norn ──────────────────────────────────────────────────────────────

fn incubate(n: i32) -> Effect {
    Effect::Incubate { who: PlayerRef::You, amount: Value::Const(n) }
}

/// Norn's Inquisitor — {1}{W} 1/1 Phyrexian Knight. ETB incubate 2; a
/// permanent of yours transforming into a Phyrexian gets a +1/+1 counter.
pub fn norns_inquisitor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(incubate(2)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Transformed, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Phyrexian),
                    },
                ),
                effect: Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature("Norn's Inquisitor", cost(&[generic(1), w()]), vec![CreatureType::Phyrexian, CreatureType::Knight], 1, 1)
    }
}

/// Progenitor Exarch — {X}{X}{W} 1/2 Phyrexian Cat Cleric. ETB incubate 3,
/// X times; {T}: transform target Incubator token you control.
pub fn progenitor_exarch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Repeat { count: Value::XFromCost, body: Box::new(incubate(3)) })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Transform {
                what: target_filtered(R::IsToken.and(R::HasName("Incubator".into())).and(R::ControlledByYou)),
            },
            ..Default::default()
        }],
        ..creature(
            "Progenitor Exarch",
            cost(&[x(), x(), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Cat, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Tiller of Flesh — {3}{W} 2/4 Phyrexian Knight. Casting a spell that
/// targets a permanent incubates 2.
pub fn tiller_of_flesh() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellTargetsMatch(R::Permanent)),
            effect: incubate(2),
        }],
        ..creature("Tiller of Flesh", cost(&[generic(3), w()]), vec![CreatureType::Phyrexian, CreatureType::Knight], 2, 4)
    }
}

// ── Elminster / Djeru ───────────────────────────────────────────────────────

/// Deification — {1}{W} enchantment. As it enters, choose a planeswalker
/// type; your planeswalkers of that type have hexproof, and while you
/// control a creature, damage can't take their last loyalty counter.
pub fn deification() -> CardDefinition {
    let chosen = || R::Planeswalker.and(R::HasChosenPlaneswalkerTypeOfSource);
    CardDefinition {
        name: "Deification",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::NamePlaneswalkerType { what: Selector::This }),
        static_abilities: vec![
            StaticAbility {
                description: "Planeswalkers you control of the chosen type have hexproof.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(chosen().and(R::ControlledByYou)),
                    keyword: Keyword::Hexproof,
                },
            },
            StaticAbility {
                description: "As long as you control a creature, if damage dealt to a planeswalker you control of the chosen type would result in all loyalty counters on it being removed, instead all but one of those counters are removed.",
                effect: StaticEffect::DamageLeavesYourPlaneswalkersAtOneLoyalty { filter: chosen(), requires_creature: true },
            },
        ],
        ..Default::default()
    }
}

// ── Tomik, Orzhov Lawmage / Tobias Andrion ──────────────────────────────────

fn empower(n: i32) -> Effect {
    Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(n) }
}

/// Way of the Healer — {3}{W} legendary enchantment. ETB empower Jace 5;
/// planeswalkers you control have "[−2]: a 2/2 colorless Wizard Soldier named
/// Cadet, then surveil 1".
pub fn way_of_the_healer() -> CardDefinition {
    let cadet = TokenDefinition {
        name: "Cadet".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Way of the Healer",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(empower(5))],
        static_abilities: vec![StaticAbility {
            description: "Planeswalkers you control have \"[−2]: Create a 2/2 colorless Wizard Soldier creature token named Cadet. Surveil 1.\"",
            effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities {
                abilities: vec![LoyaltyAbility {
                    loyalty_cost: -2,
                    effect: Effect::Seq(vec![
                        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(cadet) },
                        Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                    ]),
                    ..Default::default()
                }],
            },
        }],
        ..Default::default()
    }
}

/// Campus Crier — {1}{W} 3/1 Human Advisor. {1}, exile it from your
/// graveyard: empower Jace 2.
pub fn campus_crier() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: empower(2),
            ..Default::default()
        }],
        ..creature("Campus Crier", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Advisor], 3, 1)
    }
}

/// Protégé's Awakening — {3}{U} sorcery. Empower Jace 6, then draw a card.
pub fn proteges_awakening() -> CardDefinition {
    spell("Protege's Awakening", cost(&[generic(3), u()]), Effect::Seq(vec![empower(6), draw(1)]))
}

/// Awesome Presence — {U} Aura. Enchanted creature can't be blocked unless
/// the defending player pays {3} for each creature blocking it (CR 509.1d).
pub fn awesome_presence() -> CardDefinition {
    CardDefinition {
        name: "Awesome Presence",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature can't be blocked unless defending player pays {3} for each creature they control that's blocking it.",
            effect: StaticEffect::BlockTaxForEnchantedAttacker { amount: 3 },
        }],
        ..Default::default()
    }
}
