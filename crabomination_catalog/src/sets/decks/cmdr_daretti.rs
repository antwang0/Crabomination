//! Commander: the cards the **Built From Scratch** precon (C14, Daretti,
//! Scrap Savant) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_daretti.rs`.
//!
//! Residuals (each also on its card):
//! - **Bitter Feud** — the two chosen players are its controller and the
//!   opponent it picks (the engine's pick, as for Sawhorn Nemesis).
//! - **Impact Resonance** — X is read as the spell resolves, from each
//!   permanent's per-source tally and each player's largest single hit.
//! - **Volcanic Offering** — the opponent who chooses is the caster's most
//!   hostile one, for both halves.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{evoke, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, cost, generic, r};
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

fn not_yours(filter: R) -> R {
    filter.and(R::ControlledByYou.negate())
}

/// Bitter Feud — as it enters, choose a second player; damage between you
/// and them, either way, is doubled (CR 614.5).
pub fn bitter_feud() -> CardDefinition {
    CardDefinition {
        name: "Bitter Feud",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::RememberPlayerOnSource { who: PlayerRef::HostileOpponent }),
        static_abilities: vec![StaticAbility {
            description: "Damage a source controlled by one of the chosen players would deal to the other \
                          chosen player or their permanents is doubled.",
            effect: StaticEffect::DoubleDamageBetweenYouAndChosenPlayer,
        }],
        ..Default::default()
    }
}

/// Epochrasite — enters with three +1/+1 counters unless cast from hand;
/// dies into exile with three time counters and suspend.
pub fn epochrasite() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::Not(Box::new(Predicate::CastFromHand))),
                then: Box::new(Value::Const(3)),
                else_: Box::new(Value::Const(0)),
            },
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::GrantSuspend { what: Selector::This, time_counters: 3 },
        }],
        ..creature("Epochrasite", cost(&[generic(2)]), vec![CreatureType::Construct], 1, 1)
    }
}

/// Feldon of the Third Path — {2}{R}, {T}: a hasty artifact token copy of a
/// creature card in your graveyard, sacrificed at the next end step.
pub fn feldon_of_the_third_path() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: target_filtered(R::Creature.from_your_graveyard()),
                    extra_creature_types: vec![],
                    extra_card_types: vec![CardType::Artifact],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![Keyword::Haste],
                },
                Effect::SacrificeLastCreatedTokensAtNextEndStep,
            ]),
            ..Default::default()
        }],
        ..creature(
            "Feldon of the Third Path",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            3,
        )
    }
}

/// Hoard-Smelter Dragon — flying; {3}{R}: destroy target artifact and get
/// +X/+0, X its mana value.
pub fn hoard_smelter_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            effect: Effect::Seq(vec![
                // X is fixed before the artifact leaves (it is "that
                // artifact's mana value" either way).
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ManaValueOf(Box::new(target_filtered(R::Artifact))),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::Destroy { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..creature("Hoard-Smelter Dragon", cost(&[generic(4), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Impact Resonance — X damage divided among any number of creatures, X the
/// greatest amount one source dealt one permanent or player this turn.
pub fn impact_resonance() -> CardDefinition {
    CardDefinition {
        name: "Impact Resonance",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamageDivided {
            total: Value::GreatestDamageFromOneSourceThisTurn,
            filter: R::Creature,
            max_targets: 4,
            retaliate_to_source: false,
        },
        ..Default::default()
    }
}

/// Incite Rebellion — each player, and each creature they control, takes
/// damage equal to the number of creatures that player controls.
pub fn incite_rebellion() -> CardDefinition {
    let theirs = || Selector::ControlledBy { who: PlayerRef::Triggerer, filter: R::Creature };
    CardDefinition {
        name: "Incite Rebellion",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachPlayer),
            body: Box::new(Effect::WithX {
                x: Value::CountOf(Box::new(theirs())),
                body: Box::new(Effect::Seq(vec![
                    Effect::DealDamage { to: Selector::Player(PlayerRef::Triggerer), amount: Value::XFromCost },
                    Effect::DealDamage { to: theirs(), amount: Value::XFromCost },
                ])),
            }),
        },
        ..Default::default()
    }
}

/// Liquimetal Coating — {T}: target permanent becomes an artifact in addition
/// to its other types until end of turn.
pub fn liquimetal_coating() -> CardDefinition {
    CardDefinition {
        name: "Liquimetal Coating",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddCardTypeIndefinitely {
                what: target_filtered(R::Permanent),
                card_type: CardType::Artifact,
                until_eot: true,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Panic Spellbomb — {T}, sacrifice: target creature can't block this turn;
/// to the graveyard from the battlefield, may pay {R} to draw.
pub fn panic_spellbomb() -> CardDefinition {
    CardDefinition {
        name: "Panic Spellbomb",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {R} to draw a card?".into(),
                mana_cost: cost(&[r()]),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Scrap Mastery — every player exiles their graveyard's artifact cards,
/// sacrifices their artifacts, then returns what they exiled.
pub fn scrap_mastery() -> CardDefinition {
    CardDefinition {
        name: "Scrap Mastery",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::EachPlayerRecyclesArtifacts,
        ..Default::default()
    }
}

/// Spitebellows — 6/1; leaving the battlefield, 6 damage to target creature;
/// evoke {1}{R}{R}.
pub fn spitebellows() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(evoke(cost(&[generic(1), r(), r()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(6) },
        }],
        ..creature("Spitebellows", cost(&[generic(5), r()]), vec![CreatureType::Elemental], 6, 1)
    }
}

/// Trading Post — four {1}, {T} trades: a card for 4 life, 1 life for a
/// Goat, a creature for an artifact card back, an artifact for a card.
pub fn trading_post() -> CardDefinition {
    let one_tap = |effect: Effect| ActivatedAbility {
        mana_cost: cost(&[generic(1)]),
        tap_cost: true,
        effect,
        ..Default::default()
    };
    let goat = Arc::new(TokenDefinition {
        name: "Goat".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goat], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        name: "Trading Post",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                discard_cost: Some((R::Any, 1)),
                ..one_tap(Effect::GainLife { who: Selector::You, amount: Value::Const(4) })
            },
            ActivatedAbility {
                life_cost: 1,
                ..one_tap(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: goat })
            },
            ActivatedAbility {
                sac_other_filter: Some((R::Creature, 1)),
                ..one_tap(Effect::Move {
                    what: target_filtered(R::Artifact.from_your_graveyard()),
                    to: ZoneDest::Hand(PlayerRef::You),
                })
            },
            ActivatedAbility {
                sac_other_filter: Some((R::Artifact, 1)),
                sac_other_may_be_source: true,
                ..one_tap(Effect::Draw { who: Selector::You, amount: Value::Const(1) })
            },
        ],
        ..Default::default()
    }
}

/// Tyrant's Familiar — flying, haste; lieutenant: +2/+2 and "whenever this
/// attacks, 7 damage to target creature defending player controls".
pub fn tyrants_familiar() -> CardDefinition {
    let lieutenant = || Predicate::ControlsOwnCommander { who: PlayerRef::You };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Lieutenant — this creature gets +2/+2.",
            effect: StaticEffect::PumpSelfIf { condition: lieutenant(), power: 2, toughness: 2, keywords: vec![] },
        }],
        triggered_abilities: vec![{
            let mut t = on_attack(Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                amount: Value::Const(7),
            });
            t.event = t.event.with_filter(lieutenant());
            t
        }],
        ..creature("Tyrant's Familiar", cost(&[generic(5), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Volcanic Offering — destroy a nonbasic land you don't control and one of
/// an opponent's choice; 7 damage to a creature you don't control and to one
/// of an opponent's choice.
pub fn volcanic_offering() -> CardDefinition {
    let land = || not_yours(R::IsNonbasicLand);
    let creature = || not_yours(R::Creature);
    CardDefinition {
        name: "Volcanic Offering",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: Selector::TargetFiltered { slot: 0, filter: land() } },
            Effect::OpponentChoosesPermanentThen {
                filter: land(),
                body: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: creature() },
                amount: Value::Const(7),
            },
            Effect::OpponentChoosesPermanentThen {
                filter: creature(),
                body: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(7) }),
            },
        ]),
        ..Default::default()
    }
}

/// Warmonger Hellkite — flying; all creatures attack each combat if able;
/// {1}{R}: attacking creatures get +1/+0.
pub fn warmonger_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "All creatures attack each combat if able.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::MustAttack,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::IsAttacking),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Warmonger Hellkite", cost(&[generic(4), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Word of Seizing — split second; untap target permanent, gain control of it
/// until end of turn, and it gains haste.
pub fn word_of_seizing() -> CardDefinition {
    CardDefinition {
        name: "Word of Seizing",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::SplitSecond],
        effect: Effect::Seq(vec![
            Effect::GainControl { what: target_filtered(R::Permanent), to: None, duration: Duration::EndOfTurn },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}
