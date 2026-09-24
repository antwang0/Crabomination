//! Commander: the cards the **Devour for Power** precon (CMD, The Mimeoplasm)
//! needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_mimeoplasm.rs`.
//!
//! Residuals (each also on its card):
//! - **The Mimeoplasm** — the engine picks the two cards: it copies the
//!   greatest-power creature card in any graveyard and takes counters from the
//!   next greatest, rather than letting the player pick either role.
//! - **Desecrator Hag** — a tie for greatest power is broken by graveyard
//!   order, not by the player.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    Zone,
};
use crate::effect::shortcut::{etb, target_any};
use crate::effect::{Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, u, Color, ManaCost};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

/// The greatest-power creature card across every graveyard; asked again after
/// the first exile, it is the runner-up.
fn greatest_power_in_graveyards() -> Selector {
    Selector::TakeGreatestPower {
        inner: Box::new(Selector::CardsInZone { who: PlayerRef::EachPlayer, zone: Zone::Graveyard, filter: R::Creature }),
        count: Box::new(Value::ONE),
    }
}

/// The Mimeoplasm — {2}{B}{G}{U} 0/0 legendary Ooze. As it enters you may
/// exile two creature cards from graveyards; it enters as a copy of one with
/// +1/+1 counters equal to the other's power. (Residual: the engine picks —
/// copy the greatest power, count the runner-up.)
pub fn the_mimeoplasm() -> CardDefinition {
    let in_graveyards = Selector::CardsInZone { who: PlayerRef::EachPlayer, zone: Zone::Graveyard, filter: R::Creature };
    CardDefinition {
        // CR 614.12 — a replacement on entering, so the copy's own "when this
        // enters" abilities trigger (the 2011-09-22 ruling).
        as_enters_effect: Some(Effect::If {
            // "You can't choose to exile just one creature card."
            cond: Predicate::ValueAtLeast(Value::CountOf(Box::new(in_graveyards)), Value::Const(2)),
            then: Box::new(Effect::MayDo {
                description: "Exile two creature cards from graveyards and enter as a copy of one?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: greatest_power_in_graveyards(), to: ZoneDest::Exile },
                    Effect::BecomeCopyOf {
                        what: Selector::This,
                        source: Selector::LastMoved,
                        extra_creature_types: vec![],
                        keep_own_triggered: false,
                        keep_own_activated: false,
                    },
                    // Counted before it moves: `LastMoved` would sum both exiles.
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::PowerOf(Box::new(greatest_power_in_graveyards())),
                    },
                    Effect::Move { what: greatest_power_in_graveyards(), to: ZoneDest::Exile },
                ])),
            }),
            else_: Box::new(Effect::Noop),
        }),
        ..legend("The Mimeoplasm", cost(&[generic(2), b(), g(), u()]), vec![CreatureType::Ooze], 0, 0)
    }
}

/// Damia, Sage of Stone — {4}{B}{G}{U} 4/4 legendary Gorgon Wizard. Deathtouch;
/// skip your draw step; at the beginning of your upkeep, if you have fewer
/// than seven cards in hand, draw up to seven.
pub fn damia_sage_of_stone() -> CardDefinition {
    let hand = || Value::HandSizeOf(PlayerRef::You);
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        static_abilities: vec![StaticAbility {
            description: "Skip your draw step.",
            effect: StaticEffect::ControllerSkipsDrawStep,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::ValueAtMost(hand(), Value::Const(6))),
            // CR 603.4 — re-read on resolution: the difference, never below 0.
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Max(Box::new(Value::Const(0)), Box::new(Value::Diff(Box::new(Value::Const(7)), Box::new(hand())))),
            },
        }],
        ..legend(
            "Damia, Sage of Stone",
            cost(&[generic(4), b(), g(), u()]),
            vec![CreatureType::Gorgon, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Desecrator Hag — {2}{B/G}{B/G} 2/2 Hag. ETB: return the greatest-power
/// creature card in your graveyard to your hand (not targeted).
pub fn desecrator_hag() -> CardDefinition {
    let bg = || hybrid(Color::Black, Color::Green);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::TakeGreatestPower {
                inner: Box::new(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature }),
                count: Box::new(Value::ONE),
            },
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature("Desecrator Hag", cost(&[generic(2), bg(), bg()]), vec![CreatureType::Hag], 2, 2)
    }
}

/// Dreadship Reef — {T}: {C}; {1},{T}: a storage counter; {1}, remove X
/// storage counters: X mana in any combination of {U} and {B}.
pub fn dreadship_reef() -> CardDefinition {
    CardDefinition {
        name: "Dreadship Reef",
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
                    pool: ManaPayload::OfColors(vec![Color::Blue, Color::Black], Value::XFromCost),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Riddlekeeper — {2}{U} 1/4 Homunculus. Whenever a creature attacks you or a
/// planeswalker you control, that creature's controller mills two cards.
pub fn riddlekeeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            // The dispatcher binds the attacker's controller to slot 0.
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2) },
        }],
        ..creature("Riddlekeeper", cost(&[generic(2), u()]), vec![CreatureType::Homunculus], 1, 4)
    }
}

/// Shared Trauma — {B} Sorcery. Join forces: each player may pay any amount of
/// mana; each player mills the total.
pub fn shared_trauma() -> CardDefinition {
    CardDefinition {
        name: "Shared Trauma",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::JoinForces {
            description: "Join forces — pay any amount of mana; each player mills that many".into(),
            body: Box::new(Effect::Mill {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::TriggerEventAmount,
            }),
        },
        ..Default::default()
    }
}

/// Skullbriar, the Walking Grave — {B}{G} 1/1 legendary Zombie Elemental.
/// Haste; grows on combat damage to a player; its counters stay on it through
/// every zone but a hand or a library.
pub fn skullbriar_the_walking_grave() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        keeps_counters_off_battlefield: true,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legend(
            "Skullbriar, the Walking Grave",
            cost(&[b(), g()]),
            vec![CreatureType::Zombie, CreatureType::Elemental],
            1,
            1,
        )
    }
}

/// Triskelavus — {7} 1/1 flying artifact Construct, enters with three +1/+1
/// counters. {1}, remove a +1/+1 counter: a 1/1 flying Triskelavite token that
/// can sacrifice itself to deal 1 damage to any target.
pub fn triskelavus() -> CardDefinition {
    let triskelavite = TokenDefinition {
        name: "Triskelavite".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Triskelavite], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(triskelavite) },
            ..Default::default()
        }],
        ..creature("Triskelavus", cost(&[generic(7)]), vec![CreatureType::Construct], 1, 1)
    }
}

/// Vorosh, the Hunter — {3}{B}{G}{U} 6/6 legendary flying Dragon. On combat
/// damage to a player you may pay {2}{G} for six +1/+1 counters.
pub fn vorosh_the_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {2}{G} to put six +1/+1 counters on Vorosh?".into(),
                mana_cost: cost(&[generic(2), g()]),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(6),
                }),
                else_: None,
            },
        }],
        ..legend("Vorosh, the Hunter", cost(&[generic(3), b(), g(), u()]), vec![CreatureType::Dragon], 6, 6)
    }
}
