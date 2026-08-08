//! Modern Horizons 2 sweep, batch 6 — coin flips, devour-artifact, renown
//! payoffs, kicker land-hosing, reveal-until burn. Tests in `tests/mh2e.rs`.

use crate::card::MayPlayDuration;
use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement, Selector, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, investigate, renown, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, StaticEffect, ZoneDest};
use crate::mana::{Color, b, cost, generic, r, u, w};

use SelectionRequirement as R;

fn thopter_token() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Barbed Spike — {1}{W} Equipment. ETB mint a 1/1 Thopter and attach;
/// +1/+0; equip {2}.
pub fn barbed_spike() -> CardDefinition {
    CardDefinition {
        name: "Barbed Spike",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(thopter_token()),
            },
            Effect::Attach {
                what: Selector::This,
                to: Selector::LastCreatedToken,
            },
        ]))],
        ..Default::default()
    }
}

/// Break Ties — {2}{W} instant. Choose one: destroy artifact / destroy
/// enchantment / exile a graveyard card. Reinforce 1—{W}.
pub fn break_ties() -> CardDefinition {
    CardDefinition {
        name: "Break Ties",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Reinforce(1, cost(&[w()]))],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact),
            },
            Effect::Destroy {
                what: target_filtered(R::Enchantment),
            },
            Effect::Move {
                what: target_filtered(R::Any),
                to: ZoneDest::Exile,
            },
        ]),
        ..Default::default()
    }
}

/// Breya's Apprentice — {2}{R} 2/3. ETB a Thopter; {T}, sac an artifact:
/// impulse the top card or +2/+0 a creature.
pub fn breyas_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Breya's Apprentice",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(thopter_token()),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::ChooseMode(vec![
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: false,
                    uncast_penalty: None,
                },
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Calibrated Blast — {2}{R} instant. Reveal until a nonland; damage equal
/// to its mana value to any target. Flashback {3}{R}{R}.
pub fn calibrated_blast() -> CardDefinition {
    CardDefinition {
        name: "Calibrated Blast",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), r(), r()]))],
        effect: Effect::RevealUntilNonlandDamage { to: target_any() },
        ..Default::default()
    }
}

/// Caprichrome — {3}{W} 2/2 flash, vigilance. Devour artifact 1.
pub fn caprichrome() -> CardDefinition {
    CardDefinition {
        name: "Caprichrome",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goat],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: R::Artifact.and(R::OtherThanSource),
            per_each: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        })],
        ..Default::default()
    }
}

/// Constable of the Realm — {4}{W} 3/3, renown 2. Whenever +1/+1 counters
/// land on it, exile up to one other nonland permanent until it leaves.
pub fn constable_of_the_realm() -> CardDefinition {
    CardDefinition {
        name: "Constable of the Realm",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            renown(2),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                    EventScope::SelfSource,
                ),
                effect: Effect::ExileUntilSourceLeaves {
                    what: target_filtered(
                        R::Permanent.and(R::Land.negate()).and(R::OtherThanSource),
                    ),
                    return_to: crate::card::ExileReturnZone::Battlefield,
                },
            },
        ],
        ..Default::default()
    }
}

/// Goblin Traprunner — {3}{R} 4/2. Attacks: flip three coins; each win
/// mints a 1/1 Goblin tapped and attacking.
pub fn goblin_traprunner() -> CardDefinition {
    let goblin = TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Goblin Traprunner",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::FlipCoin {
            count: Value::Const(3),
            on_heads: Box::new(Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(goblin),
                cleanup: crate::effect::AttackingTokenCleanup::None,
            }),
            on_tails: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Liquimetal Torque — {2}. {T}: add {C}; {T}: target nonland permanent
/// becomes an artifact in addition this turn.
pub fn liquimetal_torque() -> CardDefinition {
    CardDefinition {
        name: "Liquimetal Torque",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCardTypeIndefinitely {
                    what: target_filtered(R::Permanent.and(R::Land.negate())),
                    card_type: CardType::Artifact,
                    until_eot: true,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Search the Premises — {3}{W}. Whenever a creature attacks you or a
/// planeswalker you control, investigate.
pub fn search_the_premises() -> CardDefinition {
    CardDefinition {
        name: "Search the Premises",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: investigate(1),
        }],
        ..Default::default()
    }
}

/// So Shiny — {2}{U} Aura. ETB with a token: tap enchanted + scry 2; the
/// enchanted creature doesn't untap.
pub fn so_shiny() -> CardDefinition {
    CardDefinition {
        name: "So Shiny",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::EachPermanent(R::IsToken.and(R::ControlledByYou)),
                filter: R::Any,
            },
            then: Box::new(Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                },
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}

/// Tide Shaper — {U} 1/1, kicker {1}. Kicked ETB: target land becomes an
/// Island (modeled indefinitely). +1/+1 while an opponent controls an Island.
pub fn tide_shaper() -> CardDefinition {
    CardDefinition {
        name: "Tide Shaper",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Kicker(cost(&[generic(1)]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::BecomeBasicLand {
                what: target_filtered(R::Land),
                land_type: LandType::Island,
                duration: Duration::Permanent,
            }),
            else_: Box::new(Effect::Noop),
        })],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 as long as an opponent controls an Island.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::EntityMatches {
                    what: Selector::EachPermanent(
                        R::HasLandType(LandType::Island).and(R::ControlledByOpponent),
                    ),
                    filter: R::Any,
                },
                power: 1,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Tizerus Charger — {2}{B} 3/2, escape {4}{B} + exile five. Escapes with a
/// +1/+1 counter or a flying counter (your choice).
pub fn tizerus_charger() -> CardDefinition {
    CardDefinition {
        name: "Tizerus Charger",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pegasus],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Escape(cost(&[generic(4), b()]), 5)],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::CastFromGraveyard,
            then: Box::new(Effect::ChooseMode(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::AddKeywordCounter {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    amount: Value::ONE,
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Graceful Restoration — {3}{W}{B} sorcery. Choose one: reanimate a
/// creature with an extra +1/+1 counter; or reanimate up to two with
/// power ≤ 2.
pub fn graceful_restoration() -> CardDefinition {
    CardDefinition {
        name: "Graceful Restoration",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature.and(R::PowerAtMost(2)),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
            },
        ]),
        ..Default::default()
    }
}
