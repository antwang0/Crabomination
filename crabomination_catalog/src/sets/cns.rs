//! Conspiracy (CNS / CN2) — CR 315 conspiracy cards. They start in the
//! command zone and never leave; hidden-agenda ones start face down with a
//! secretly chosen card name (`GameState::seat_conspiracy`). Tests in
//! `classic_sets/cns`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::effect::shortcut::target_filtered;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn conspiracy(name: &'static str) -> CardDefinition {
    CardDefinition { name, card_types: vec![CardType::Conspiracy], ..Default::default() }
}

/// "At the beginning of the first upkeep of the game, …"
fn first_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
            .with_filter(Predicate::ValueAtMost(Value::TurnNumber, Value::ONE)),
        effect,
    }
}

fn token(name: &str, p: i32, t: i32, ct: CreatureType, colors: Vec<Color>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        ..Default::default()
    }
}

// ── Face-up conspiracies ───────────────────────────────────────────────────

/// Power Play — you are the starting player.
pub fn power_play() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You are the starting player.",
            effect: StaticEffect::ControllerIsStartingPlayer,
        }],
        ..conspiracy("Power Play")
    }
}

/// Hymn of the Wilds — a creature discount bought with your instants and
/// sorceries.
pub fn hymn_of_the_wilds() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "The first creature spell you cast each turn costs {1} less.",
                effect: StaticEffect::CostReductionFirstCreatureSpell { amount: 1 },
            },
            StaticAbility {
                description: "You can't cast instant or sorcery spells.",
                effect: StaticEffect::ControllerCantCastInstantsOrSorceries,
            },
        ],
        ..conspiracy("Hymn of the Wilds")
    }
}

/// Weight Advantage — your creatures hit as hard as they are tough.
pub fn weight_advantage() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your creatures assign combat damage equal to their toughness.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::AssignsCombatDamageByToughness],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..conspiracy("Weight Advantage")
    }
}

/// Sentinel Dispatch — a free wall on the game's first upkeep.
pub fn sentinel_dispatch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![first_upkeep(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                card_types: vec![CardType::Artifact, CardType::Creature],
                keywords: vec![Keyword::Defender],
                ..token("Construct", 1, 1, CreatureType::Construct, vec![])
            },
        })],
        ..conspiracy("Sentinel Dispatch")
    }
}

/// Hold the Perimeter — you get a blocker, everyone else gets a Goblin that
/// can't block.
pub fn hold_the_perimeter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            first_upkeep(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    keywords: vec![Keyword::Defender],
                    ..token("Soldier", 1, 2, CreatureType::Soldier, vec![Color::White])
                },
            }),
            first_upkeep(Effect::CreateToken {
                who: PlayerRef::EachOpponent,
                count: Value::ONE,
                definition: TokenDefinition {
                    keywords: vec![Keyword::CantBlock],
                    ..token("Goblin", 1, 1, CreatureType::Goblin, vec![Color::Red])
                },
            }),
        ],
        ..conspiracy("Hold the Perimeter")
    }
}

// ── Hidden agenda ──────────────────────────────────────────────────────────

/// Brago's Favor — spells with the chosen name cost {1} less.
pub fn bragos_favor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells with the chosen name you cast cost {1} less.",
            effect: StaticEffect::NamedSpellCostReduction { amount: 1 },
        }],
        ..conspiracy("Brago's Favor")
    }
}

/// Immediate Action — creatures with the chosen name have haste.
pub fn immediate_action() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with the chosen name have haste.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::NamedBySource),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Haste],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..conspiracy("Immediate Action")
    }
}

/// Iterative Analysis — casting an instant or sorcery with the chosen name
/// draws you a card.
pub fn iterative_analysis() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: crate::effect::Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .and(R::NamedBySource),
                },
            ),
            effect: Effect::Draw { who: crate::effect::Selector::You, amount: Value::ONE },
        }],
        ..conspiracy("Iterative Analysis")
    }
}

/// Muzzio's Preparations — creatures with the chosen name enter bigger.
pub fn muzzios_preparations() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with the chosen name enter with an extra \
                          +1/+1 counter.",
            effect: StaticEffect::MatchingEntersWithExtraCounters {
                filter: R::Creature.and(R::NamedBySource),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: 1,
            },
        }],
        ..conspiracy("Muzzio's Preparations")
    }
}

/// The chosen-name creatures you control.
fn chosen_creatures() -> R {
    R::Creature.and(R::NamedBySource).and(R::ControlledByYou)
}

/// Incendiary Dissent — chosen-name creatures can pump themselves.
pub fn incendiary_dissent() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with the chosen name have \"{R}: +1/+0\".",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(chosen_creatures()),
                ability: ActivatedAbility {
                    mana_cost: cost(&[r()]),
                    effect: Effect::PumpPT {
                        what: Selector::This,
                        power: Value::ONE,
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..conspiracy("Incendiary Dissent")
    }
}

/// Secrets of Paradise — chosen-name creatures tap for any colour.
pub fn secrets_of_paradise() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with the chosen name have \
                          \"{T}: Add one mana of any color\".",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(chosen_creatures()),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyColors(Value::ONE),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..conspiracy("Secrets of Paradise")
    }
}

/// Adriana's Valor — a chosen-name attacker can buy indestructible.
pub fn adrianas_valor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Chosen-name attackers may pay {W} for indestructible.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: chosen_creatures(),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                    effect: Effect::MayPay {
                        description: "Pay {W} for indestructible?".into(),
                        mana_cost: cost(&[w()]),
                        body: Box::new(Effect::GrantKeyword {
                            what: Selector::This,
                            keyword: Keyword::Indestructible,
                            duration: Duration::EndOfTurn,
                        }),
                        else_: None,
                    },
                }),
            },
        }],
        ..conspiracy("Adriana's Valor")
    }
}

/// Hired Heist — chosen-name creatures cash their combat damage for a card.
pub fn hired_heist() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Chosen-name creatures may pay {U} on combat damage to draw.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: chosen_creatures(),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::DealsCombatDamageToPlayer,
                        EventScope::SelfSource,
                    ),
                    effect: Effect::MayPay {
                        description: "Pay {U} to draw a card?".into(),
                        mana_cost: cost(&[u()]),
                        body: Box::new(Effect::Draw {
                            who: Selector::You,
                            amount: Value::ONE,
                        }),
                        else_: None,
                    },
                }),
            },
        }],
        ..conspiracy("Hired Heist")
    }
}

/// Assemble the Rank and Vile — chosen-name creatures leave a Zombie behind.
pub fn assemble_the_rank_and_vile() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Chosen-name creatures may pay {B} on death for a 2/2 Zombie.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: chosen_creatures(),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                    effect: Effect::MayPay {
                        description: "Pay {B} for a 2/2 Zombie?".into(),
                        mana_cost: cost(&[b()]),
                        body: Box::new(Effect::Seq(vec![
                            Effect::CreateToken {
                                who: PlayerRef::You,
                                count: Value::ONE,
                                definition: token(
                                    "Zombie",
                                    2,
                                    2,
                                    CreatureType::Zombie,
                                    vec![Color::Black],
                                ),
                            },
                            Effect::Tap { what: Selector::LastCreatedToken },
                        ])),
                        else_: None,
                    },
                }),
            },
        }],
        ..conspiracy("Assemble the Rank and Vile")
    }
}

/// Natural Unity — chosen-name creatures grow each combat for {G}.
pub fn natural_unity() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Chosen-name creatures may pay {G} each combat for a +1/+1 counter.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: chosen_creatures(),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::StepBegins(TurnStep::BeginCombat),
                        EventScope::YourControl,
                    ),
                    effect: Effect::MayPay {
                        description: "Pay {G} for a +1/+1 counter?".into(),
                        mana_cost: cost(&[g()]),
                        body: Box::new(Effect::AddCounter {
                            what: Selector::This,
                            kind: crate::card::CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        }),
                        else_: None,
                    },
                }),
            },
        }],
        ..conspiracy("Natural Unity")
    }
}

/// Double Stroke — the chosen instant or sorcery gets cast twice.
pub fn double_stroke() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .and(R::NamedBySource),
                },
            ),
            effect: Effect::CopySpellMayChooseTargets {
                what: Selector::TriggerSource,
                count: Value::ONE,
            },
        }],
        ..conspiracy("Double Stroke")
    }
}

/// Secret Summoning — the first chosen-name creature fetches the rest.
pub fn secret_summoning() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "A chosen-name creature entering searches out its twins.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: chosen_creatures(),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                    effect: Effect::SearchSameNameAs {
                        who: PlayerRef::You,
                        subject: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::You),
                        count: Some(Value::LibrarySizeOf(PlayerRef::You)),
                    },
                }),
            },
        }],
        ..conspiracy("Secret Summoning")
    }
}

/// Worldknit — your lands tap for any colour. (The printed card-pool gate is
/// unconditional here: the engine has no card-pool concept.)
pub fn worldknit() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Lands you control have \"{T}: Add one mana of any color\".",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyColors(Value::ONE),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..conspiracy("Worldknit")
    }
}

// ── Conspiracy's regular cards ─────────────────────────────────────────────

/// Deathreap Ritual — morbid: a card at each end step something died.
pub fn deathreap_ritual() -> CardDefinition {
    CardDefinition {
        name: "Deathreap Ritual",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast {
                    at_least: Value::ONE,
                }),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..Default::default()
    }
}

/// Brago, King Eternal — his combat damage blinks your board.
pub fn brago_king_eternal() -> CardDefinition {
    CardDefinition {
        name: "Brago, King Eternal",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Noble],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ExileAndReturnToOwner {
                what: Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Not(Box::new(R::Land)).and(R::OtherThanSource),
                },
            },
        }],
        ..Default::default()
    }
}

/// Canal Dredger — recycles your graveyard to the bottom of your library.
/// (Its draft-time clause has no in-game effect.)
pub fn canal_dredger() -> CardDefinition {
    CardDefinition {
        name: "Canal Dredger",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::InYourGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: crate::effect::LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cogwork Spy — a 2/1 flier. (Its draft-time peek has no in-game effect.)
pub fn cogwork_spy() -> CardDefinition {
    CardDefinition {
        name: "Cogwork Spy",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Construct],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}
