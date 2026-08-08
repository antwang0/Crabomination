//! A Wilds of Eldraine (WOE) wave: Celebration payoffs, Food/aristocrat value,
//! tap-matters, and a Bargain dragon. All ride existing primitives. Tests in
//! `crabomination/src/tests/recent141.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{deal, draw, etb, target_filtered};
use crate::effect::{Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef};
use crate::game::effects::food_token;
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, x};

// ── White / White-Blue ─────────────────────────────────────────────────────────

/// Lady of Laughter — {3}{W}{W} 4/5 Faerie Noble with flying. Celebration —
/// your end step, if 2+ nonland permanents entered under your control, draw.
pub fn lady_of_laughter() -> CardDefinition {
    CardDefinition {
        name: "Lady of Laughter",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Noble],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::CelebrationActive {
                    who: PlayerRef::You,
                },
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Sharae of Numbing Depths — {2}{W}{U} 2/3 legendary Merfolk Wizard. ETB tap +
/// stun an opponent's creature; tapping enemy creatures draws once each turn.
pub fn sharae_of_numbing_depths() -> CardDefinition {
    CardDefinition {
        name: "Sharae of Numbing Depths",
        cost: cost(&[generic(2), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::ONE,
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::YouTapped)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    })
                    .once_per_turn(),
                effect: draw(1),
            },
        ],
        ..Default::default()
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────────

/// Ingenious Prodigy — {X}{U} 0/1 Human Wizard with skulk, entering with X
/// +1/+1 counters. Upkeep: you may remove a +1/+1 counter to draw a card.
pub fn ingenious_prodigy() -> CardDefinition {
    CardDefinition {
        name: "Ingenious Prodigy",
        cost: cost(&[x(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Skulk],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::SourceHasCountersAtLeast {
                    counter: CounterType::PlusOnePlusOne,
                    n: 1,
                },
                then: Box::new(Effect::MayDo {
                    description: "remove a +1/+1 counter to draw a card".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                        draw(1),
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Talion's Messenger — {2}{U} 1/3 Faerie Noble with flying. Attacking with 1+
/// Faeries loots, then puts a +1/+1 counter on a Faerie you control.
pub fn talions_messenger() -> CardDefinition {
    CardDefinition {
        name: "Talion's Messenger",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Noble],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                Predicate::AttackedWithCreatureMatching {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Faerie),
                },
            ),
            effect: Effect::Seq(vec![
                draw(1),
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                Effect::AddCounter {
                    what: target_filtered(
                        R::HasCreatureType(CreatureType::Faerie).and(R::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Black ────────────────────────────────────────────────────────────────────────

/// Malevolent Witchkite — {4}{B}{B} 5/4 Dragon Warlock with flying. ETB
/// sacrifice any number of artifacts, enchantments, and/or tokens; draw that many.
pub fn malevolent_witchkite() -> CardDefinition {
    CardDefinition {
        name: "Malevolent Witchkite",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Warlock],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: R::Artifact.or(R::Enchantment).or(R::IsToken),
            per_each: Box::new(draw(1)),
        })],
        ..Default::default()
    }
}

/// Old Flitterfang — {4}{B} 3/4 legendary Rat Faerie with flying. Each end step,
/// if a creature died this turn, make a Food; sac another creature/artifact to pump.
pub fn old_flitterfang() -> CardDefinition {
    CardDefinition {
        name: "Old Flitterfang",
        cost: cost(&[generic(4), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Faerie],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::ValueAtLeast(
                    Value::CreaturesDiedThisTurnTotal,
                    Value::Const(1),
                )),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(food_token()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_other_filter: Some((R::Creature.or(R::Artifact).and(R::OtherThanSource), 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Obyra, Dreaming Duelist — {U}{B} 2/2 legendary Faerie Warrior with flash and
/// flying. Whenever another Faerie you control enters, each opponent loses 1 life.
pub fn obyra_dreaming_duelist() -> CardDefinition {
    CardDefinition {
        name: "Obyra, Dreaming Duelist",
        cost: cost(&[u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Faerie),
                }),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────────

/// Unruly Catapult — {2}{R} 0/4 Construct with defender. {T}: 1 damage to each
/// opponent. Untaps whenever you cast an instant or sorcery spell.
pub fn unruly_catapult() -> CardDefinition {
    CardDefinition {
        name: "Unruly Catapult",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: deal(1, Selector::Player(PlayerRef::EachOpponent)),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_instant_or_sorcery()),
            effect: Effect::Untap {
                what: Selector::This,
                up_to: None,
            },
        }],
        ..Default::default()
    }
}

/// Realm-Scorcher Hellkite — {4}{R}{R} 4/6 Dragon with flying, haste, and
/// Bargain. ETB if bargained, add four mana in any combination of colors.
/// {1}{R}: 1 damage to any target.
pub fn realm_scorcher_hellkite() -> CardDefinition {
    CardDefinition {
        name: "Realm-Scorcher Hellkite",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Bargain],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasBargained),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColors(Value::Const(4)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: deal(1, crate::effect::shortcut::target_any()),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Raging Battle Mouse — {1}{R} 2/1 Mouse. Your second spell each turn costs
/// {1} less. Celebration — combat on your turn, if 2+ nonland permanents
/// entered under your control, a creature you control gets +1/+1.
pub fn raging_battle_mouse() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Raging Battle Mouse",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mouse],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "The second spell you cast each turn costs {1} less to cast.",
            effect: StaticEffect::CostReductionNthSpell {
                filter: R::Any,
                nth: 2,
                amount: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::CelebrationActive {
                    who: PlayerRef::You,
                },
                then: Box::new(Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

// ── Green ────────────────────────────────────────────────────────────────────────

/// Tough Cookie — {1}{G} 2/2 Food Golem artifact creature. ETB create a Food.
/// {2}{G}: a noncreature artifact you control becomes a 4/4 until end of turn.
/// {2}, {T}, Sacrifice this: gain 3 life.
pub fn tough_cookie() -> CardDefinition {
    CardDefinition {
        name: "Tough Cookie",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Food],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(food_token()),
        })],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                effect: Effect::BecomeCreature {
                    what: target_filtered(R::Artifact.and(R::Noncreature).and(R::ControlledByYou)),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    creature_types: vec![],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
