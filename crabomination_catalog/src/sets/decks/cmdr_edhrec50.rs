//! Commander: the 2026-10-13 rescan's gaps for Borborygmos and Fblthp. Tests
//! in `tests/recent_b/cmdr_edhrec50.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, LibraryPosition, PlayerRef, ZoneDest};
use crate::mana::{cost, g, generic, r, u, x, ManaCost};

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// "Draw a card, then you may discard any number of land cards. When you
/// discard one or more this way, twice that much damage to target creature."
fn borborygmos_loot() -> Effect {
    Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::ONE },
        Effect::DiscardAnyNumber { who: Selector::You, filter: R::Land, max: None },
        Effect::If {
            cond: Predicate::ValueAtLeast(Value::CardsDiscardedThisEffect, Value::ONE),
            then: Box::new(Effect::WithX {
                x: Value::CardsDiscardedThisEffect,
                body: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(Effect::DealDamage {
                        to: target_filtered(R::Creature),
                        amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost)),
                    }),
                }),
            }),
            else_: Box::new(Effect::Noop),
        },
    ])
}

/// Borborygmos and Fblthp — {2}{G}{U}{R} legendary 6/5 Cyclops Homunculus.
/// Entering or attacking: draw, then discard any number of lands for twice
/// that much damage to a creature. {1}{U}: put it into its owner's library
/// third from the top.
pub fn borborygmos_and_fblthp() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(borborygmos_loot()),
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: borborygmos_loot() },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::This)),
                    pos: LibraryPosition::FromTop(2),
                },
            },
            ..Default::default()
        }],
        ..legend(
            "Borborygmos and Fblthp",
            cost(&[generic(2), g(), u(), r()]),
            vec![CreatureType::Cyclops, CreatureType::Homunculus],
            6,
            5,
        )
    }
}

/// Fblthp, Impossibly Lost — {1}{U} legendary 1/1 Homunculus. When your
/// opponents are dealt combat damage during your turn: draw two; with an empty
/// library you win (CR 104.2b); then Fblthp shuffles into his owner's library.
pub fn fblthp_impossibly_lost() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .once_per_batch()
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::ValueAtLeast(
                        Value::LibrarySizeOf(PlayerRef::You),
                        Value::ONE,
                    ))),
                    then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::This)),
                        pos: LibraryPosition::Shuffled,
                    },
                },
            ]),
        }],
        ..legend("Fblthp, Impossibly Lost", cost(&[generic(1), u()]), vec![CreatureType::Homunculus], 1, 1)
    }
}

/// Fblthp, Knows the Way — {X}{G}{G} legendary */2 Homunculus Scout; domain
/// power. Enters: up to X basic land cards to hand (the "different names"
/// rider isn't enforced — `SearchUpToN`).
pub fn fblthp_knows_the_way() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Domain — Fblthp's power is equal to the number of basic land types among lands you control.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: Value::DomainCount(PlayerRef::You),
                toughness: Value::Const(2),
            },
        }],
        // "with different names": each pick differs from the ones before it,
        // which are this resolution's moved cards once the list is cleared.
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand.and(R::NameDiffersFromLastMoved),
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::XFromCost,
            },
        ]))],
        ..legend(
            "Fblthp, Knows the Way",
            cost(&[x(), g(), g()]),
            vec![CreatureType::Homunculus, CreatureType::Scout],
            0,
            2,
        )
    }
}
