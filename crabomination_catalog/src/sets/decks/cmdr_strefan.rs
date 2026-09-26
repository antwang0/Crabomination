//! Commander: the cards the **Vampiric Bloodline** precon (VOC, Strefan,
//! Maurer Progenitor) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_strefan.rs`.
//!
//! Residuals (each also on its card):
//! - **Avacyn's Judgment** — no madness: its `{X}{R}` madness needs an X
//!   the madness cast can't choose, so it is a plain 2-damage divider.
//! - **Shadowgrange Archfiend** — no madness: "{2}{B}, Pay 8 life" has a life
//!   half a madness cost can't carry.
//! - **Imposing Grandeur** — counts a commander in any zone, not only the
//!   battlefield or command zone.
//! - **Predators' Hour** — the stolen card is exiled face up.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate};
use crate::game::TurnStep;
use crate::mana::{b, cost, generic, r};
use crabomination_base::tokens::blood_token;
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

fn vampire() -> R {
    R::HasCreatureType(CreatureType::Vampire)
}

fn blood(n: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(blood_token()) }
}

fn blood_tokens() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Blood).and(R::IsToken)
}

/// Impulse: exile your top card; you may play it this turn.
fn impulse() -> Effect {
    Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::Const(1),
        duration: MayPlayDuration::EndOfThisTurn,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost: true,
        uncast_penalty: None,
    }
}

/// Arterial Alchemy — a Blood token per opponent; your Blood tokens are
/// Equipment with "+2/+0" and equip {2}.
pub fn arterial_alchemy() -> CardDefinition {
    CardDefinition {
        name: "Arterial Alchemy",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(blood(Value::CountOf(Box::new(Selector::Player(PlayerRef::EachOpponent)))))],
        static_abilities: vec![StaticAbility {
            description: "Blood tokens you control are Equipment with \"Equipped creature gets +2/+0\" and equip {2}.",
            effect: StaticEffect::MatchingArtifactsAreEquipment {
                filter: blood_tokens().and(R::ControlledByYou),
                equip: cost(&[generic(2)]),
                power: 2,
                filtered_equip: None,
            },
        }],
        ..Default::default()
    }
}

/// Avacyn's Judgment — 2 damage divided among any number of targets.
pub fn avacyns_judgment() -> CardDefinition {
    CardDefinition {
        name: "Avacyn's Judgment",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamageDivided {
            total: Value::Const(2),
            filter: R::Creature.or(R::Player).or(R::Planeswalker),
            max_targets: 2,
            retaliate_to_source: false,
        },
        ..Default::default()
    }
}

/// Falkenrath Gorger — each Vampire creature card you own off the
/// battlefield has madness equal to its mana cost.
pub fn falkenrath_gorger() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each Vampire creature card you own that isn't on the battlefield has madness.",
            effect: StaticEffect::OwnedCardsHaveMadness { filter: R::Creature.and(vampire()) },
        }],
        ..creature(
            "Falkenrath Gorger",
            cost(&[r()]),
            vec![CreatureType::Vampire, CreatureType::Berserker],
            2,
            1,
        )
    }
}

/// Imposing Grandeur — each player may discard their hand and draw as many
/// as their commander's greatest mana value.
pub fn imposing_grandeur() -> CardDefinition {
    CardDefinition {
        name: "Imposing Grandeur",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachPlayer),
            body: Box::new(Effect::AsPlayer {
                who: PlayerRef::Triggerer,
                body: Box::new(Effect::MayDo {
                    description: "Discard your hand and draw for your commander's mana value?".into(),
                    body: Box::new(Effect::Seq(vec![
                        // The whole hand: no choice to make, so the unchosen
                        // (in-order) discard path.
                        Effect::Discard {
                            who: Selector::You,
                            amount: Value::HandSizeOf(PlayerRef::You),
                            random: true,
                        },
                        Effect::Draw {
                            who: Selector::You,
                            amount: Value::GreatestCommanderManaValueInPlayOrCommandZone(PlayerRef::You),
                        },
                    ])),
                }),
            }),
        },
        ..Default::default()
    }
}

/// Kamber, the Plunderer — partner with Laurine; lifelink; an opponent's
/// creature dying gains you 1 and makes a Blood token.
pub fn kamber_the_plunderer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::PartnerWith("Laurine, the Diversion".into()), Keyword::Lifelink],
        triggered_abilities: vec![
            crate::effect::shortcut::partner_with_search("Laurine, the Diversion"),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                    blood(Value::Const(1)),
                ]),
            },
        ],
        ..creature(
            "Kamber, the Plunderer",
            cost(&[generic(3), b()]),
            vec![CreatureType::Vampire, CreatureType::Rogue],
            3,
            4,
        )
    }
}

/// Laurine, the Diversion — partner with Kamber; first strike; {2},
/// sacrifice an artifact or creature: goad target creature.
pub fn laurine_the_diversion() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::PartnerWith("Kamber, the Plunderer".into()), Keyword::FirstStrike],
        triggered_abilities: vec![crate::effect::shortcut::partner_with_search("Kamber, the Plunderer")],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Artifact.or(R::Creature), 1)),
            sac_other_may_be_source: true,
            effect: Effect::Goad { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature(
            "Laurine, the Diversion",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Markov Enforcer — as it or another Vampire you control enters, it may fight
/// a creature an opponent controls; a creature it damaged this turn dying
/// makes a Blood token.
pub fn markov_enforcer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: vampire().or(R::IsSource) },
                ),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Fight {
                        attacker: Selector::This,
                        defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::DamagedBySourceThisTurn },
                ),
                effect: blood(Value::Const(1)),
            },
        ],
        ..creature(
            "Markov Enforcer",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Vampire, CreatureType::Soldier],
            6,
            6,
        )
    }
}

/// Midnight Arsonist — destroy up to X target artifacts without mana
/// abilities, X the number of Vampires you control.
pub fn midnight_arsonist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CapTargetsAt {
            amount: Value::CountOf(Box::new(Selector::EachPermanent(vampire().and(R::ControlledByYou)))),
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 6,
                min_targets: 0,
                filter: R::Artifact.and(R::HasManaAbility.negate()),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            }),
        })],
        ..creature("Midnight Arsonist", cost(&[generic(3), r()]), vec![CreatureType::Vampire], 3, 2)
    }
}

/// Mob Rule — steal every creature with power 4 or greater, or every one with
/// power 3 or less, untapped and hasty, until end of turn.
pub fn mob_rule() -> CardDefinition {
    let steal = |filter: R| {
        let all = || Selector::EachPermanent(R::Creature.and(filter.clone()));
        Effect::Seq(vec![
            Effect::GainControl { what: all(), to: None, duration: Duration::EndOfTurn },
            Effect::Untap { what: all(), up_to: None },
            Effect::GrantKeyword { what: all(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
        ])
    };
    CardDefinition {
        name: "Mob Rule",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![steal(R::PowerAtLeast(4)), steal(R::PowerAtMost(3))]),
        ..Default::default()
    }
}

/// Predators' Hour — your creatures gain menace and a connect trigger that
/// exiles the damaged player's top card for you to play, any mana.
pub fn predators_hour() -> CardDefinition {
    let yours = || R::Creature.and(R::ControlledByYou);
    CardDefinition {
        name: "Predators' Hour",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::EachPermanent(yours()),
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantTriggeredAbilityThisTurnToMatching {
                filter: yours(),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::TriggerEventPlayer,
                        count: Value::Const(1),
                        duration: MayPlayDuration::WhileExiled,
                        pay_any_color: true,
                        max_mana_value: None,
                        pay_own_cost: false,
                        uncast_penalty: None,
                    },
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Scion of Opulence — it or another nontoken Vampire of yours dying makes a
/// Treasure; {R}, sacrifice two artifacts: impulse.
pub fn scion_of_opulence() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: vampire().and(R::NotToken) },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Arc::new(crate::game::effects::treasure_token()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((R::Artifact, 2)),
            effect: impulse(),
            ..Default::default()
        }],
        ..creature(
            "Scion of Opulence",
            cost(&[generic(2), r()]),
            vec![CreatureType::Vampire, CreatureType::Noble],
            3,
            1,
        )
    }
}

/// Shadowgrange Archfiend — each opponent sacrifices their greatest-power
/// creature; you gain the greatest power sacrificed.
pub fn shadowgrange_archfiend() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: R::Creature.and(R::HasGreatestPowerAmongControlled(Box::new(R::Creature))),
            },
            Effect::GainLife { who: Selector::You, amount: Value::GreatestSacrificedPowerThisResolution },
        ]))],
        ..creature("Shadowgrange Archfiend", cost(&[generic(6), b()]), vec![CreatureType::Demon], 8, 4)
    }
}

/// Sinister Waltz — three target creature cards in your graveyard: two at
/// random return, the third goes to the bottom of your library.
pub fn sinister_waltz() -> CardDefinition {
    CardDefinition {
        name: "Sinister Waltz",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 3,
                min_targets: 3,
                filter: R::Creature.from_your_graveyard(),
                // Declares the three graveyard slots; the draw is below.
                effect: Box::new(Effect::Noop),
            },
            Effect::ReturnTargetCardsAtRandom { count: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Strefan, Maurer Progenitor — flying; a Blood token per player who lost life
/// this turn at your end step; attacking, trade two Blood tokens for a Vampire
/// from hand, tapped, attacking and indestructible.
pub fn strefan_maurer_progenitor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: blood(Value::Sum(vec![
                    Value::OpponentsWhoLostLifeThisTurn,
                    Value::IfPred {
                        pred: Box::new(Predicate::PlayerLostLifeThisTurn { who: PlayerRef::You }),
                        then: Box::new(Value::Const(1)),
                        else_: Box::new(Value::Const(0)),
                    },
                ])),
            },
            on_attack(Effect::MaySacrifice {
                description: "Sacrifice two Blood tokens for a Vampire from your hand?".into(),
                filter: blood_tokens(),
                count: Value::Const(2),
                then: Box::new(Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Creature.and(vampire()),
                    count: Value::Const(1),
                    tapped: true,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: Some(Box::new(Effect::Seq(vec![
                        Effect::JoinCombatAttacking { what: Selector::LastMoved },
                        Effect::GrantKeyword {
                            what: Selector::LastMoved,
                            keyword: Keyword::Indestructible,
                            duration: Duration::EndOfTurn,
                        },
                    ]))),
                }),
                else_: None,
            }),
        ],
        ..creature(
            "Strefan, Maurer Progenitor",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Vampire, CreatureType::Noble],
            3,
            2,
        )
    }
}

/// Stromkirk Condemned — discard a card: Vampires you control get +1/+1
/// until end of turn, once each turn.
pub fn stromkirk_condemned() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(vampire()).and(R::ControlledByYou)),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Stromkirk Condemned",
            cost(&[b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Horror],
            2,
            2,
        )
    }
}

/// Stromkirk Occultist — trample; connecting, impulse; madness {1}{R}.
pub fn stromkirk_occultist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Madness(cost(&[generic(1), r()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: impulse(),
        }],
        ..creature(
            "Stromkirk Occultist",
            cost(&[generic(2), r()]),
            vec![CreatureType::Vampire, CreatureType::Horror],
            3,
            2,
        )
    }
}
