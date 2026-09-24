//! Commander: the cards the **Stalwart Unity** precon (C16, Kynaios and Tiro
//! of Meletis) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_kynaios.rs`.
//!
//! Residuals (each also on its card):
//! - **Kynaios and Tiro of Meletis** — an opponent holding a land who
//!   declines to put it onto the battlefield doesn't draw.
//! - **Humble Defector** — the opponent who gains control is a random one.
//! - **Sidar Kondo of Jamuraa** — the evasion covers creatures you control
//!   with power 2 or less, not other players' small attackers.
//! - **Orzhov Advokist** — a taker's counters go on their greatest-power
//!   creature, and the attack restriction covers the creatures they control
//!   as it resolves.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate};
use crate::game::TurnStep;
use crate::mana::{Color, cost, g, generic, r, u, w};
use crate::sets::{enters_tapped, tap_add_colorless};
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

fn your_turn() -> Option<Predicate> {
    Some(Predicate::IsTurnOf(PlayerRef::You))
}

/// Benefactor's Draught — untap all creatures; until end of turn, whenever a
/// creature an opponent controls blocks, draw a card; draw a card.
pub fn benefactors_draught() -> CardDefinition {
    CardDefinition {
        name: "Benefactor's Draught",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap { what: Selector::EachPermanent(R::Creature), up_to: None },
            Effect::OnMatchingBlocksThisTurn {
                filter: R::Creature.and(R::ControlledByOpponent),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Edric, Spymaster of Trest — whenever a creature deals combat damage to one
/// of your opponents, its controller may draw a card.
pub fn edric_spymaster_of_trest() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer)
                .dealt_by(R::Creature)
                .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer }),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::Triggerer,
                body: Box::new(Effect::MayDo {
                    description: "Draw a card?".into(),
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                }),
            },
        }],
        ..creature(
            "Edric, Spymaster of Trest",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Entrapment Maneuver — target player sacrifices an attacking creature of
/// their choice; you create X 1/1 Soldiers, X its toughness.
pub fn entrapment_maneuver() -> CardDefinition {
    CardDefinition {
        name: "Entrapment Maneuver",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember { who: PlayerRef::Target(0), filter: R::Creature.and(R::IsAttacking) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::SacrificedToughness,
                definition: Arc::new(TokenDefinition {
                    name: "Soldier".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
                    ..Default::default()
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Evolutionary Escalation — at the beginning of your upkeep, three +1/+1
/// counters on target creature you control and three on target creature an
/// opponent controls.
pub fn evolutionary_escalation() -> CardDefinition {
    CardDefinition {
        name: "Evolutionary Escalation",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(3),
                },
                Effect::AddCounter {
                    what: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(3),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Gwafa Hazid, Profiteer — {W}{U}, {T}: a bribery counter on target creature
/// you don't control, and its controller draws a card; creatures with bribery
/// counters can't attack or block.
pub fn gwafa_hazid_profiteer() -> CardDefinition {
    let bribed = || Selector::EachPermanent(R::Creature.and(R::WithCounter(CounterType::Bribery)));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
                    kind: CounterType::Bribery,
                    amount: Value::Const(1),
                },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures with bribery counters on them can't attack.",
                effect: StaticEffect::GrantKeyword { applies_to: bribed(), keyword: Keyword::CantAttack },
            },
            StaticAbility {
                description: "Creatures with bribery counters on them can't block.",
                effect: StaticEffect::GrantKeyword { applies_to: bribed(), keyword: Keyword::CantBlock },
            },
        ],
        ..creature(
            "Gwafa Hazid, Profiteer",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Homeward Path — {T}: {C}; {T}: each player gains control of all creatures
/// they own.
pub fn homeward_path() -> CardDefinition {
    CardDefinition {
        name: "Homeward Path",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::EachPlayerDoes {
                    who: PlayerRef::EachPlayer,
                    body: Box::new(Effect::GainControl {
                        what: Selector::OwnedBy {
                            who: PlayerRef::You,
                            filter: R::Creature.and(R::Not(Box::new(R::ControlledByYou))),
                        },
                        to: None,
                        duration: Duration::Permanent,
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hoofprints of the Stag — whenever you draw a card, you may put a
/// hoofprint counter on it; {2}{W}, remove four: a 4/4 flying white
/// Elemental, only during your turn.
pub fn hoofprints_of_the_stag() -> CardDefinition {
    CardDefinition {
        name: "Hoofprints of the Stag",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Hoofprint, amount: Value::Const(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            remove_counter_cost: Some((CounterType::Hoofprint, 4)),
            condition: your_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Arc::new(TokenDefinition {
                    name: "Elemental".into(),
                    power: 4,
                    toughness: 4,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Humble Defector — {T}: draw two cards, then an opponent gains control of
/// it; only during your turn.
pub fn humble_defector() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: your_turn(),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::RandomOpponent),
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Humble Defector", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 1)
    }
}

/// Keening Stone — {5}, {T}: target player mills X, X the cards in their
/// graveyard.
pub fn keening_stone() -> CardDefinition {
    CardDefinition {
        name: "Keening Stone",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            effect: Effect::Mill {
                who: target_filtered(R::Player),
                amount: Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::Target(0),
                    zone: Zone::Graveyard,
                    filter: R::Any,
                })),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kraum, Ludevic's Opus — flying, haste, partner; whenever an opponent
/// casts their second spell each turn, draw a card.
pub fn kraum_ludevics_opus() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::SpellsCastThisTurnEquals { who: PlayerRef::Triggerer, count: Value::Const(2) },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..creature(
            "Kraum, Ludevic's Opus",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::Zombie, CreatureType::Horror],
            4,
            4,
        )
    }
}

/// Kynaios and Tiro of Meletis — at the beginning of your end step, draw a
/// card; each player may put a land from hand onto the battlefield, then
/// each opponent who didn't draws a card.
pub fn kynaios_and_tiro_of_meletis() -> CardDefinition {
    let put_land = || Effect::PutFromHandOntoBattlefield {
        who: PlayerRef::You,
        filter: R::Land,
        count: Value::Const(1),
        tapped: false,
        haste: false,
        sacrifice_eot: false,
        return_eot: false,
        then: None,
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                put_land(),
                Effect::EachPlayerDoes {
                    who: PlayerRef::EachOpponent,
                    body: Box::new(Effect::If {
                        cond: Predicate::SelectorExists(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Hand,
                            filter: R::Land,
                        }),
                        then: Box::new(put_land()),
                        else_: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                    }),
                },
            ]),
        }],
        ..creature(
            "Kynaios and Tiro of Meletis",
            cost(&[r(), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            8,
        )
    }
}

/// Ludevic, Necro-Alchemist — partner; at the beginning of each player's end
/// step, that player may draw a card if a player other than you lost life
/// this turn.
pub fn ludevic_necro_alchemist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
                then: Box::new(Effect::EachPlayerDoes {
                    who: PlayerRef::ActivePlayer,
                    body: Box::new(Effect::MayDo {
                        description: "Draw a card?".into(),
                        body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature(
            "Ludevic, Necro-Alchemist",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            4,
        )
    }
}

/// Orzhov Advokist — at the beginning of your upkeep, each player may put two
/// +1/+1 counters on a creature they control; a player who does can't
/// attack you with their creatures until your next turn.
pub fn orzhov_advokist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::EachPlayerMayCounterForPeace { counters: 2 },
        }],
        ..creature("Orzhov Advokist", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Advisor], 1, 4)
    }
}

/// Prismatic Geoscope — enters tapped; domain — {T}: X mana in any
/// combination of colors, X the basic land types among your lands.
pub fn prismatic_geoscope() -> CardDefinition {
    CardDefinition {
        name: "Prismatic Geoscope",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColors(Value::DomainCount(PlayerRef::You)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Selfless Squire — flash; on entry, prevent all damage that would be dealt
/// to you this turn; whenever damage to you is prevented, that many +1/+1
/// counters on it.
pub fn selfless_squire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::PreventAllDamageToPlayerThisTurn { who: PlayerRef::You },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DamageToPlayerPrevented, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::TriggerEventAmount,
                },
            },
        ],
        ..creature("Selfless Squire", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Sidar Kondo of Jamuraa — flanking, partner; creatures your opponents
/// control without flying or reach can't block creatures with power 2 or
/// less.
pub fn sidar_kondo_of_jamuraa() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flanking, Keyword::Partner],
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control without flying or reach can't block creatures with power 2 or less.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::CantBeBlockedExceptByWhilePowerAtMost(
                    2,
                    Box::new(R::HasKeyword(Keyword::Flying).or(R::HasKeyword(Keyword::Reach))),
                ),
            },
        }],
        ..creature(
            "Sidar Kondo of Jamuraa",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            5,
        )
    }
}

