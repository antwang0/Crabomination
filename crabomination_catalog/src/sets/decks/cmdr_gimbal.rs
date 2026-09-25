//! Commander: the cards the **Tinker Time** precon (MOC, Gimbal, Gremlin
//! Prodigy) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_gimbal.rs`.
//!
//! Residuals (each also on its card):
//! - **Dance with Calamity** — the controller exiles until the total mana
//!   value reaches nine (the engine's stop point), not as many times as they
//!   choose.
//! - **Path of the Animist** — Will of the Planeswalkers is a vote with no
//!   effect: outside Planechase, planeswalking and chaos do nothing (CR 901).
//! - **Pain Distributor** — "that player" is the artifact's owner.
//! - **Gimbal** — its trample grant, like every static type filter, reads
//!   printed types (an animated artifact misses it; ENGINE_BACKLOG).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{bolster, etb, investigate, mint_treasures, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, u, Color, ManaCost};
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

fn your_step(step: TurnStep) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl)
}

/// "the number of differently named artifact tokens you control".
fn distinct_artifact_tokens() -> Value {
    Value::DistinctNamesControlledMatching(R::Artifact.and(R::IsToken))
}

fn any_color_rock() -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
        ..Default::default()
    }
}

/// Gimbal, Gremlin Prodigy — your artifact creatures have trample; at your end
/// step a 0/0 Gremlin with a +1/+1 counter per differently named artifact
/// token you control.
pub fn gimbal_gremlin_prodigy() -> CardDefinition {
    let gremlin = TokenDefinition {
        name: "Gremlin".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gremlin], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Artifact.and(R::Creature).and(R::ControlledByYou)),
                keyword: Keyword::Trample,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::End),
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(gremlin) },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: distinct_artifact_tokens(),
                },
            ]),
        }],
        ..creature(
            "Gimbal, Gremlin Prodigy",
            cost(&[generic(2), g(), u(), r()]),
            vec![CreatureType::Gremlin, CreatureType::Artificer],
            4,
            4,
        )
    }
}

/// Aid from the Cowl — revolt: at your end step reveal the top card; a
/// permanent card may go onto the battlefield, anything else may go to the
/// bottom.
pub fn aid_from_the_cowl() -> CardDefinition {
    CardDefinition {
        name: "Aid from the Cowl",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::End).with_filter(Predicate::RevoltActive { who: PlayerRef::You }),
            effect: Effect::RevealTopThenIf {
                who: PlayerRef::You,
                filter: R::Permanent,
                then: Box::new(Effect::RevealTopMayPutOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Permanent,
                    counter: None,
                    extra_types: vec![],
                }),
                else_: Some(Box::new(Effect::MayDo {
                    description: "Put the revealed card on the bottom of your library?".into(),
                    body: Box::new(Effect::Move {
                        what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                        to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Bottom },
                    }),
                })),
            },
        }],
        ..Default::default()
    }
}

/// Cutthroat Negotiator — parley on attack: a tapped Treasure per nonland card
/// revealed, then everyone draws.
pub fn cutthroat_negotiator() -> CardDefinition {
    let mut treasure = crate::game::effects::treasure_token();
    treasure.tapped = true;
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Parley {
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::CardsRevealedThisEffect,
                    definition: Arc::new(treasure),
                }),
            },
        }],
        ..creature(
            "Cutthroat Negotiator",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Orc, CreatureType::Pirate],
            4,
            3,
        )
    }
}

/// Dance with Calamity — shuffle, exile from the top, and if the total mana
/// value is 13 or less cast the spells free. Residual: the exiling stops at a
/// total of nine rather than by choice.
pub fn dance_with_calamity() -> CardDefinition {
    CardDefinition {
        name: "Dance with Calamity",
        cost: cost(&[generic(7), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ShuffleLibrary { who: PlayerRef::You },
            Effect::ExileTopPushingLuck {
                stop_at: 9,
                limit: 13,
                then: Box::new(Effect::CastAnyOrderWithoutPaying {
                    what: Selector::ExiledThisResolution { filter: R::Any },
                    source_zone: crate::card::Zone::Exile,
                    filter: None,
                    cap: None,
                    total_mana_value: None,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Ghirapur Aether Grid — tap two untapped artifacts: 1 damage to any target.
pub fn ghirapur_aether_grid() -> CardDefinition {
    CardDefinition {
        name: "Ghirapur Aether Grid",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::Artifact, 2)),
            effect: Effect::DealDamage { to: target_filtered(R::Any), amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hedron Detonator — each artifact of yours entering pings an opponent;
/// {T}, sacrifice two artifacts: impulse-draw one.
pub fn hedron_detonator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact },
            ),
            effect: Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 2)),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..creature(
            "Hedron Detonator",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Artificer],
            2,
            3,
        )
    }
}

/// Inspiring Statuary — nonartifact spells you cast have improvise.
pub fn inspiring_statuary() -> CardDefinition {
    CardDefinition {
        name: "Inspiring Statuary",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Nonartifact spells you cast have improvise.",
            effect: StaticEffect::GrantImproviseToSpells { filter: R::Not(Box::new(R::Artifact)) },
        }],
        ..Default::default()
    }
}

/// Masterful Replication — two 3/3 Golems, or every other artifact of yours
/// becomes a copy of one until end of turn.
pub fn masterful_replication() -> CardDefinition {
    let golem = TokenDefinition {
        name: "Golem".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Masterful Replication",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(golem) },
            Effect::BecomeCopyOfFor {
                what: Selector::EachPermanent(
                    R::Artifact.and(R::ControlledByYou).and(R::OtherThanTargetSlot(0)),
                ),
                source: target_filtered(R::Artifact.and(R::ControlledByYou)),
                duration: Duration::EndOfTurn,
                non_legendary: false,
            },
        ]),
        ..Default::default()
    }
}

/// Pain Distributor — menace; each player's first spell each turn makes them a
/// Treasure; an opponent's artifact dying costs its owner 1 life.
pub fn pain_distributor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                    Predicate::ValueAtMost(
                        Value::SpellsCastThisTurn(PlayerRef::TriggerEventPlayer),
                        Value::ONE,
                    ),
                ),
                effect: Effect::CreateToken {
                    who: PlayerRef::TriggerEventPlayer,
                    count: Value::ONE,
                    definition: Arc::new(crate::game::effects::treasure_token()),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::OpponentControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact },
                ),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                    amount: Value::ONE,
                },
            },
        ],
        ..creature(
            "Pain Distributor",
            cost(&[generic(2), r()]),
            vec![CreatureType::Devil, CreatureType::Citizen],
            2,
            3,
        )
    }
}

/// Path of the Animist — two basic lands tapped; the planeswalkers' vote does
/// nothing outside Planechase.
pub fn path_of_the_animist() -> CardDefinition {
    CardDefinition {
        name: "Path of the Animist",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(
            (0..2)
                .map(|_| Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                })
                .collect(),
        ),
        ..Default::default()
    }
}

/// Rashmi and Ragavan — your first spell on your turn: exile an opponent's top
/// card and make a Treasure; cast it free if its mana value is less than your
/// artifact count, else you may cast it this turn.
pub fn rashmi_and_ragavan() -> CardDefinition {
    let artifacts = Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::SamePlayer(PlayerRef::ActivePlayer, PlayerRef::You),
                Predicate::ValueAtMost(Value::SpellsCastThisTurn(PlayerRef::You), Value::ONE),
            ])),
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: target_filtered(R::OpponentPlayer),
                    amount: Value::ONE,
                    link_to_source: false,
                    face_down: false,
                },
                mint_treasures(1),
                Effect::WithX {
                    x: Value::Diff(Box::new(artifacts), Box::new(Value::ONE)),
                    body: Box::new(Effect::CastWithoutPayingImmediate {
                        what: Selector::ExiledThisResolution {
                            filter: R::Nonland.and(R::ManaValueAtMostXFromCost),
                        },
                        source_zone: crate::card::Zone::Exile,
                        exile_after: false,
                        copy: false,
                        reduce_generic: 0,
                        pay_own_cost: false,
                    }),
                },
                Effect::GrantMayPlay {
                    what: Selector::ExiledThisResolution { filter: R::InExile },
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ]),
        }],
        ..creature(
            "Rashmi and Ragavan",
            cost(&[generic(1), g(), u(), r()]),
            vec![CreatureType::Elf, CreatureType::Monkey],
            2,
            4,
        )
    }
}

/// Sandsteppe War Riders — trample; bolster X each combat on your turn, X the
/// differently named artifact tokens you control.
pub fn sandsteppe_war_riders() -> CardDefinition {
    let Effect::AddCounter { what, kind, .. } = bolster(1) else { unreachable!() };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::BeginCombat),
            effect: Effect::AddCounter { what, kind, amount: distinct_artifact_tokens() },
        }],
        ..creature(
            "Sandsteppe War Riders",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Schema Thief — flying; connecting copies one of that player's artifacts.
pub fn schema_thief() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::Artifact.and(R::ControlledByTriggerPlayer)),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
        }],
        ..creature(
            "Schema Thief",
            cost(&[generic(3), u()]),
            vec![CreatureType::Vedalken, CreatureType::Rogue, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Skyclave Relic — kicker {3}, indestructible mana rock; kicked, two tapped
/// token copies.
pub fn skyclave_relic() -> CardDefinition {
    CardDefinition {
        name: "Skyclave Relic",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Kicker(cost(&[generic(3)])), Keyword::Indestructible],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::Const(2),
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: true,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![any_color_rock()],
        ..Default::default()
    }
}

/// Spell Swindle — counter target spell; a Treasure per point of its mana
/// value.
pub fn spell_swindle() -> CardDefinition {
    CardDefinition {
        name: "Spell Swindle",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CounteredSpellManaValue,
                definition: Arc::new(crate::game::effects::treasure_token()),
            },
        ]),
        ..Default::default()
    }
}

/// Weirding Wood — enchant land; investigate; the land taps for two mana of
/// any one color.
pub fn weirding_wood() -> CardDefinition {
    CardDefinition {
        name: "Weirding Wood",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        triggered_abilities: vec![etb(investigate(1))],
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Add two mana of any one color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyOneColor(Value::Const(2)),
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}
