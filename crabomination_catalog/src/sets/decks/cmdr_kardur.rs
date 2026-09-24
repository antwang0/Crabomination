//! Commander: the cards the **Chaos Incarnate** Secret Lair Commander deck
//! (SCD, Kardur, Doomscourge) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_kardur.rs`.
//!
//! Residuals (each also on its card):
//! - **Kardur, Doomscourge** — creatures that enter after its ETB are goaded
//!   by a delayed trigger rather than a static rule (they are goaded once it
//!   resolves, before any combat).
//! - **Theater of Horrors** — the permission outlives the enchantment, and
//!   lands among the exiled cards can't be played (the engine-wide may-play
//!   land gap).
//! - **Wildfire Devils** — the random player's pick is their first instant or
//!   sorcery in graveyard order.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, MayPlayDuration, SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{encore, etb, landfall, on_attack, target_any, target_filtered};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r};

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn step(s: TurnStep, scope: EventScope) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(s), scope)
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn opposing_creatures() -> R {
    R::Creature.and(R::ControlledByOpponent)
}

/// Archfiend of Depravity — flying; at each opponent's end step they keep up
/// to two creatures and sacrifice the rest.
pub fn archfiend_of_depravity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: step(TurnStep::End, EventScope::OpponentControl),
            effect: Effect::SacrificeAllButN {
                who: Selector::Player(PlayerRef::ActivePlayer),
                keep: Value::Const(2),
                filter: R::Creature,
            },
        }],
        ..creature("Archfiend of Depravity", cost(&[generic(3), b(), b()]), vec![CreatureType::Demon], 5, 4)
    }
}

/// Breath of Malfegor — 5 damage to each opponent.
pub fn breath_of_malfegor() -> CardDefinition {
    spell(
        "Breath of Malfegor",
        cost(&[generic(3), b(), r()]),
        CardType::Instant,
        Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(5) },
    )
}

/// Dredge the Mire — each opponent chooses a creature card in their
/// graveyard; you get them all.
pub fn dredge_the_mire() -> CardDefinition {
    spell(
        "Dredge the Mire",
        cost(&[generic(3), b()]),
        CardType::Sorcery,
        Effect::EachOpponentChoosesFromGraveyard {
            filter: R::Creature,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Explosion of Riches — you draw, each other player may; every card drawn
/// this way is 5 damage to an opponent chosen at random.
pub fn explosion_of_riches() -> CardDefinition {
    let boom = || Effect::DealDamage { to: Selector::Player(PlayerRef::RandomOpponent), amount: Value::Const(5) };
    spell(
        "Explosion of Riches",
        cost(&[generic(5), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            boom(),
            Effect::EachOtherPlayerMayDraw { per_draw: Box::new(boom()) },
        ]),
    )
}

/// Geode Rager — first strike; landfall: goad each creature target player
/// controls.
pub fn geode_rager() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![landfall(Effect::Goad {
            what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
        })],
        ..creature("Geode Rager", cost(&[generic(4), r(), r()]), vec![CreatureType::Elemental], 4, 3)
    }
}

/// Kaervek the Merciless — an opponent's spell deals its mana value to any
/// target.
pub fn kaervek_the_merciless() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..creature(
            "Kaervek the Merciless",
            cost(&[generic(5), b(), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            5,
            4,
        )
    }
}

/// Kardur, Doomscourge — ETB goads every opposing creature until your next
/// turn, those entering later too; an attacking creature dying drains each
/// opponent 1.
pub fn kardur_doomscourge() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Goad { what: Selector::EachPermanent(opposing_creatures()) },
                Effect::WheneverCreatureEntersUntilYourNextTurn {
                    filter: opposing_creatures(),
                    body: Box::new(Effect::Goad { what: Selector::TriggerSource }),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsAttacking }),
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ]),
            },
        ],
        ..creature(
            "Kardur, Doomscourge",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Demon, CreatureType::Berserker],
            4,
            3,
        )
    }
}

/// Magmatic Force — at the beginning of each upkeep, 3 damage to any target.
pub fn magmatic_force() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: step(TurnStep::Upkeep, EventScope::AnyPlayer),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
        }],
        ..creature("Magmatic Force", cost(&[generic(5), r(), r(), r()]), vec![CreatureType::Elemental], 7, 7)
    }
}

/// Molten Slagheap — {T}: {C}; {1},{T}: a storage counter; {1}, remove X
/// storage counters: X mana in any combination of {B} and {R}.
pub fn molten_slagheap() -> CardDefinition {
    CardDefinition {
        name: "Molten Slagheap",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Storage, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                remove_counter_x: Some(CounterType::Storage),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::Black, Color::Red], Value::XFromCost),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rakshasa Debaser — attacking, it reanimates a creature card from the
/// defending player's graveyard under your control; encore {6}{B}{B}.
pub fn rakshasa_debaser() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Move {
            what: target_filtered(R::Creature.from_any_graveyard().and(R::OwnedByDefendingPlayer)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        activated_abilities: vec![encore(cost(&[generic(6), b(), b()]))],
        ..creature("Rakshasa Debaser", cost(&[generic(4), b(), b()]), vec![CreatureType::Demon], 6, 6)
    }
}

/// Scythe Specter — flying; combat damage to a player makes each opponent
/// discard, and whoever discarded the greatest mana value loses that much.
pub fn scythe_specter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Discard { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE, random: false },
                Effect::GreatestDiscardersLoseLife,
            ]),
        }],
        ..creature("Scythe Specter", cost(&[generic(4), b(), b()]), vec![CreatureType::Specter], 4, 4)
    }
}

/// Stensia Bloodhall — {T}: {C}; {3}{B}{R},{T}: 2 damage to target player or
/// planeswalker.
pub fn stensia_bloodhall() -> CardDefinition {
    CardDefinition {
        name: "Stensia Bloodhall",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), b(), r()]),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Theater of Horrors — each upkeep exile your top card; on your turn, once an
/// opponent has lost life, you may play those cards; {3}{R}: 1 damage to
/// target opponent or planeswalker.
pub fn theater_of_horrors() -> CardDefinition {
    CardDefinition {
        name: "Theater of Horrors",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: step(TurnStep::Upkeep, EventScope::YourControl),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::HolderTurnsAfterOpponentLostLife { holder: 0 },
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            effect: Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer.or(R::Planeswalker)),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Titan Hunter — at each player's end step, if no creature died this turn,
/// 4 damage to that player; {1}{B}, sacrifice a creature: gain 4 life.
pub fn titan_hunter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: step(TurnStep::End, EventScope::AnyPlayer).with_filter(Predicate::Not(Box::new(
                Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE },
            ))),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::ActivePlayer), amount: Value::Const(4) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Creature, 1)),
            sac_other_may_be_source: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            ..Default::default()
        }],
        ..creature("Titan Hunter", cost(&[generic(4), b()]), vec![CreatureType::Human, CreatureType::Warrior], 4, 5)
    }
}

/// Unlicensed Disintegration — destroy target creature; with an artifact,
/// 3 damage to its controller.
pub fn unlicensed_disintegration() -> CardDefinition {
    spell(
        "Unlicensed Disintegration",
        cost(&[generic(1), b(), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))),
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(3),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Wildfire Devils — entering and at your upkeep, a random player exiles an
/// instant or sorcery from their graveyard; you may cast a copy free.
pub fn wildfire_devils() -> CardDefinition {
    let body = || {
        Effect::Seq(vec![
            Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::RandomPlayer,
                        zone: crate::card::Zone::Graveyard,
                        filter: instant_or_sorcery(),
                    }),
                    count: Box::new(Value::ONE),
                },
                to: ZoneDest::Exile,
            },
            Effect::CopyCardAndCastFree { what: Selector::ExiledThisResolution { filter: instant_or_sorcery() } },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(body()),
            TriggeredAbility { event: step(TurnStep::Upkeep, EventScope::YourControl), effect: body() },
        ],
        ..creature("Wildfire Devils", cost(&[generic(3), r()]), vec![CreatureType::Devil], 4, 2)
    }
}

