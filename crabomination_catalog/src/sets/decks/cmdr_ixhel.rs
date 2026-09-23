//! Commander: the cards the **Corrupting Influence** precon (ONC, Ixhel,
//! Scion of Atraxa) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).

use crate::card::{
    ActivatedAbility, CardDefinition, Zone, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::catalog::sets::enters_tapped;
use crate::effect::shortcut::etb;
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, w};
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

/// "An opponent has three or more poison counters" (CR 702.166's ability
/// word, Corrupted).
fn corrupted() -> Predicate {
    Predicate::CorruptedActive { who: PlayerRef::You }
}

/// Carrion Call — two 1/1 infect Insects at instant speed.
pub fn carrion_call() -> CardDefinition {
    CardDefinition {
        name: "Carrion Call",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: Arc::new(TokenDefinition {
                name: "Phyrexian Insect".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Phyrexian, CreatureType::Insect],
                    ..Default::default()
                },
                keywords: vec![Keyword::Infect],
                ..Default::default()
            }),
        },
        ..Default::default()
    }
}

/// Glistening Sphere — a tapped mana rock that proliferates, and makes three
/// of one color while an opponent is corrupted. ⚠ The corrupted ability is a
/// conditional mana ability, which auto-payment does not tap; it is
/// activated directly.
pub fn glistening_sphere() -> CardDefinition {
    CardDefinition {
        name: "Glistening Sphere",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![enters_tapped()],
        triggered_abilities: vec![etb(Effect::Proliferate)],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(corrupted()),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(3)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ichor Rats — infect, and a poison counter for every player.
pub fn ichor_rats() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        triggered_abilities: vec![etb(Effect::AddPoison {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::ONE,
        })],
        ..creature(
            "Ichor Rats",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Rat],
            2,
            1,
        )
    }
}

/// Norn's Choirmaster — proliferate whenever a commander of yours enters or
/// attacks.
pub fn norns_choirmaster() -> CardDefinition {
    let commander = || Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsCommander };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(commander()),
                effect: Effect::Proliferate,
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(commander()),
                effect: Effect::Proliferate,
            },
        ],
        ..creature(
            "Norn's Choirmaster",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Angel],
            5,
            4,
        )
    }
}

/// Phyresis Outbreak — a poison counter for each opponent, then each of
/// their creatures shrinks by its own controller's poison count.
pub fn phyresis_outbreak() -> CardDefinition {
    let poison = || {
        Value::Negate(Box::new(Value::PoisonCountersOf(PlayerRef::ControllerOf(Box::new(
            Selector::TriggerSource,
        )))))
    };
    CardDefinition {
        name: "Phyresis Outbreak",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddPoison { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            Effect::ForEach {
                selector: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: poison(),
                    toughness: poison(),
                    duration: crate::effect::Duration::EndOfTurn,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Vishgraz, the Doomhive — three toxic Mites, and +1/+1 for every poison
/// counter across its controller's opponents.
pub fn vishgraz_the_doomhive() -> CardDefinition {
    let mite = Arc::new(TokenDefinition {
        name: "Phyrexian Mite".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Mite],
            ..Default::default()
        },
        keywords: vec![Keyword::Toxic(1), Keyword::CantBlock],
        ..Default::default()
    });
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Menace, Keyword::Toxic(1)],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: mite,
        })],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Vishgraz gets +1/+1 for each poison counter your opponents have.",
            effect: crate::card::StaticEffect::PumpSelfByValue {
                amount: Value::PoisonCountersAmong(PlayerRef::EachOpponent),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..creature(
            "Vishgraz, the Doomhive",
            cost(&[generic(2), w(), b(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Insect],
            3,
            3,
        )
    }
}

fn corrupted_opponents() -> Value {
    Value::PlayersWithPoisonAtLeast { who: PlayerRef::EachOpponent, at_least: 3 }
}

/// Contaminant Grafter — proliferate once per combat it connects, and a
/// corrupted end-step card plus an optional land drop.
pub fn contaminant_grafter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Toxic(1)],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .once_per_batch_across_players(),
                effect: Effect::Proliferate,
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::End),
                    EventScope::YourControl,
                )
                .with_filter(corrupted()),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: R::Land,
                        count: Value::ONE,
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    },
                ]),
            },
        ],
        ..creature(
            "Contaminant Grafter",
            cost(&[generic(4), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Druid],
            5,
            5,
        )
    }
}

/// Geth's Summons — a creature back from your graveyard, and one from each
/// corrupted opponent's. ⚠ Both picks are made at resolution rather than
/// targeted, and "three or more poison as you cast" is read then too.
pub fn geths_summons() -> CardDefinition {
    let to_you = || ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false };
    let one_from = |who: PlayerRef| Effect::MoveChosen {
        from: Selector::CardsInZone { who, zone: Zone::Graveyard, filter: R::Creature },
        filter: None,
        count: Value::ONE,
        up_to: true,
        to: to_you(),
    };
    CardDefinition {
        name: "Geth's Summons",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            one_from(PlayerRef::You),
            Effect::ForEachOpponent {
                body: Box::new(Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::PoisonCountersOf(PlayerRef::Triggerer),
                        Value::Const(3),
                    ),
                    then: Box::new(one_from(PlayerRef::Triggerer)),
                    else_: Box::new(Effect::Noop),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Glissa's Retriever — haste, toxic 3, evasion, and a corrupted death that
/// exiles it to rebuy a card per corrupted opponent. ⚠ The rebuy is picked
/// at resolution of the one trigger, not targeted by a reflexive one.
pub fn glissas_retriever() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Toxic(3), Keyword::CantBeBlockedByPowerAtMost(2)],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::If {
            cond: corrupted(),
            then: Box::new(Effect::Seq(vec![
                Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                Effect::MoveChosen {
                    from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Any },
                    filter: None,
                    count: corrupted_opponents(),
                    up_to: true,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Glissa's Retriever",
            cost(&[generic(5), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Beast],
            6,
            6,
        )
    }
}

/// Wurmquake — an X/X toxic Wurm for the mana spent (flashback counts), and
/// one more per corrupted opponent.
pub fn wurmquake() -> CardDefinition {
    let wurm = Arc::new(TokenDefinition {
        name: "Phyrexian Wurm".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wurm],
            ..Default::default()
        },
        keywords: vec![Keyword::Trample, Keyword::Toxic(1)],
        ..Default::default()
    });
    let x = || Value::CastSpellManaSpent;
    CardDefinition {
        name: "Wurmquake",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(8), g(), g()]))],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Sum(vec![Value::ONE, corrupted_opponents()]),
                definition: wurm,
            },
            Effect::PumpPT {
                what: Selector::LastCreatedTokens,
                power: x(),
                toughness: x(),
                duration: crate::effect::Duration::Permanent,
            },
        ]),
        ..Default::default()
    }
}

/// Ixhel, Scion of Atraxa — flying, vigilance, toxic 2; at your end step each
/// corrupted opponent exiles their top card for you to play, spending mana as
/// though it were any color. ⚠ The card is exiled face up (hidden
/// information only — who may play it is unchanged).
pub fn ixhel_scion_of_atraxa() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Toxic(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(corrupted()),
            effect: Effect::ForEachOpponent {
                body: Box::new(Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::PoisonCountersOf(PlayerRef::Triggerer),
                        Value::Const(3),
                    ),
                    then: Box::new(Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::Triggerer,
                        count: Value::ONE,
                        duration: crate::card::MayPlayDuration::WhileExiled,
                        pay_any_color: true,
                        max_mana_value: None,
                        pay_own_cost: false,
                        uncast_penalty: None,
                    }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        }],
        ..creature(
            "Ixhel, Scion of Atraxa",
            cost(&[generic(1), w(), b(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Angel],
            2,
            5,
        )
    }
}

/// Norn's Decree — an opponent whose creatures connect with you gets a
/// poison counter, and a player who attacks a poisoned player draws.
pub fn norns_decree() -> CardDefinition {
    CardDefinition {
        name: "Norn's Decree",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            // CR 603.2c — one per batch; the damaged player is you and the
            // dealer's controller is bound as the Triggerer.
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer)
                    .dealt_by(R::ControlledByOpponent)
                    .with_filter(Predicate::SamePlayer(PlayerRef::TriggerEventPlayer, PlayerRef::You))
                    .once_per_batch(),
                effect: Effect::AddPoison {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer)
                    .with_filter(Predicate::AnAttackedPlayerHasPoisonAtLeast { at_least: 1 }),
                effect: Effect::Draw { who: Selector::Player(PlayerRef::ActivePlayer), amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}
