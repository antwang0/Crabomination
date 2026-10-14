//! Commander: one-card gaps of EDHREC's commanders ranked 3501-4500 (the
//! 2026-10-14 scan), third file. Tests in `tests/recent_b/cmdr_rank4k_c.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, cost, g, generic, r, u, w};

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

fn draw(n: i32) -> Effect {
    Effect::Draw {
        who: Selector::You,
        amount: Value::Const(n),
    }
}

// ── Jedit Ojanen ─────────────────────────────────────────────────────────────

/// Yargle, Goliath of Otaria — {4}{U} legendary 3/9 Frog Spirit.
pub fn yargle_goliath_of_otaria() -> CardDefinition {
    legend(
        "Yargle, Goliath of Otaria",
        cost(&[generic(4), u()]),
        vec![CreatureType::Frog, CreatureType::Spirit],
        3,
        9,
    )
}

// ── Desdemona, Freedom's Edge / G'raha Tia ───────────────────────────────────

/// Eye of Jace — {1} artifact. At the beginning of your upkeep, surveil 1;
/// then with seven or more cards in your graveyard, sacrifice it, it deals 2
/// damage to each opponent, and you gain 2 life.
pub fn eye_of_jace() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::GraveyardSizeOf(PlayerRef::You),
                        Value::Const(7),
                    ),
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSource,
                        Effect::DealDamage {
                            to: Selector::Player(PlayerRef::EachOpponent),
                            amount: Value::Const(2),
                        },
                        Effect::GainLife {
                            who: Selector::You,
                            amount: Value::Const(2),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..spell("Eye of Jace", cost(&[generic(1)]), CardType::Artifact, Effect::Noop)
    }
}

// ── Tarox Bladewing ──────────────────────────────────────────────────────────

/// Tarox Bladewing — {2}{R}{R}{R} legendary 4/3 flying, haste Dragon.
/// Grandeur — discard another card named Tarox Bladewing: +X/+X until end of
/// turn, where X is its power.
pub fn tarox_bladewing() -> CardDefinition {
    let power = || Value::PowerOf(Box::new(Selector::This));
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::HasName("Tarox Bladewing".to_string()), 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: power(),
                toughness: power(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend(
            "Tarox Bladewing",
            cost(&[generic(2), r(), r(), r()]),
            vec![CreatureType::Dragon],
            4,
            3,
        )
    }
}

// ── Halsin, Emerald Archdruid ────────────────────────────────────────────────

/// Halsin, Emerald Archdruid — {3}{G} legendary 2/4 Elf Druid; choose a
/// Background. {1}: target token you control becomes a green 4/4 Bear creature
/// in addition to its other colors and types until end of turn.
pub fn halsin_emerald_archdruid() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Seq(vec![
                Effect::BecomeCreature {
                    what: target_filtered(R::IsToken.and(R::ControlledByYou)),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![CreatureType::Bear],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                Effect::BecomeColor {
                    what: Selector::Target(0),
                    colors: vec![crate::mana::Color::Green],
                    duration: Duration::EndOfTurn,
                    additive: true,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Halsin, Emerald Archdruid",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            2,
            4,
        )
    }
}

// ── M'Baku, Jabari Chieftain ─────────────────────────────────────────────────

/// Spore Cloud — {1}{G}{G} instant. Tap all blocking creatures. Prevent all
/// combat damage this turn. Each attacking and blocking creature doesn't untap
/// during its controller's next untap step.
pub fn spore_cloud() -> CardDefinition {
    spell(
        "Spore Cloud",
        cost(&[generic(1), g(), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::SkipNextUntap {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            },
            Effect::Tap {
                what: Selector::EachPermanent(R::Creature.and(R::IsBlocking)),
            },
            Effect::PreventAllCombatDamageThisTurn,
        ]),
    )
}

// ── General Leo Cristophe ────────────────────────────────────────────────────

/// Quantum Entanglement — {1}{W} flash enchantment. When it enters and at the
/// beginning of your end step, you may pay {1}{W}; when you do, exile target
/// creature you control, then return it under its owner's control.
pub fn quantum_entanglement() -> CardDefinition {
    let blink = || Effect::MayPay {
        description: "Pay {1}{W} to blink a creature you control?".into(),
        mana_cost: cost(&[generic(1), w()]),
        body: Box::new(Effect::Reflexive {
            body: Box::new(Effect::ExileAndReturnToOwner {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
            }),
        }),
        else_: None,
    };
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(blink()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: blink(),
            },
        ],
        ..spell(
            "Quantum Entanglement",
            cost(&[generic(1), w()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── The Unspeakable ──────────────────────────────────────────────────────────

/// Vision of the Unspeakable — 0/0 flying, trample Spirit enchantment creature
/// that gets +1/+1 for each card in your hand.
fn vision_of_the_unspeakable() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Trample],
        dynamic_pt: Some(crate::card::DynamicPt::ControllerHandSize),
        ..creature(
            "Vision of the Unspeakable",
            ManaCost::default(),
            vec![CreatureType::Spirit],
            0,
            0,
        )
    }
}

/// Behold the Unspeakable // Vision of the Unspeakable — {3}{U}{U} Saga. I:
/// creatures you don't control get -2/-0 until your next turn. II: with one or
/// fewer cards in hand, draw four; otherwise scry 2, then draw two. III: exile
/// it, then return it transformed.
pub fn behold_the_unspeakable() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                    power: Value::Const(-2),
                    toughness: Value::ZERO,
                    duration: Duration::UntilNextTurn,
                },
            ),
            (
                2,
                Effect::If {
                    cond: Predicate::ValueAtMost(Value::HandSizeOf(PlayerRef::You), Value::ONE),
                    then: Box::new(draw(4)),
                    else_: Box::new(Effect::Seq(vec![
                        Effect::Scry {
                            who: PlayerRef::You,
                            amount: Value::Const(2),
                        },
                        draw(2),
                    ])),
                },
            ),
            (3, Effect::ExileSelfReturnTransformed),
        ],
        back_face: Some(Box::new(vision_of_the_unspeakable())),
        ..spell(
            "Behold the Unspeakable",
            cost(&[generic(3), u(), u()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Dragon Man, Reformed Robot ───────────────────────────────────────────────

/// Does Machines — {1}{U} Class. Enters: mill two, draw two, discard two.
/// {1}{U}: level 2 — return up to two target artifact cards from your
/// graveyard to your hand. {4}{U}: level 3 — at the beginning of combat on
/// your turn, three +1/+1 counters on target artifact you control; a
/// noncreature one becomes a 0/0 Robot creature too.
pub fn does_machines() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Class],
            ..Default::default()
        },
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                draw(2),
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(2),
                    random: false,
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(2)),
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Artifact.and(R::InYourGraveyard),
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: crate::effect::ZoneDest::Hand(PlayerRef::You),
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(R::Artifact.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(3),
                    },
                    Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::Target(0),
                            filter: R::Creature,
                        },
                        then: Box::new(Effect::Noop),
                        else_: Box::new(Effect::BecomeCreature {
                            what: Selector::Target(0),
                            power: Value::ZERO,
                            toughness: Value::ZERO,
                            creature_types: vec![CreatureType::Robot],
                            keywords: vec![],
                            duration: Duration::Permanent,
                        }),
                    },
                ]),
            },
        ],
        activated_abilities: vec![
            level_up(cost(&[generic(1), u()]), 1),
            level_up(cost(&[generic(4), u()]), 2),
        ],
        ..spell(
            "Does Machines",
            cost(&[generic(1), u()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}
