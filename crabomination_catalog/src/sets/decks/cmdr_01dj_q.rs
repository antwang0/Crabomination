//! Commander: 2026-10-19 (`01DJHkWk`) — Sharae of Numbing Depths' Lorwyn
//! Merfolk: Champions of the Shoal, Eclipsed Merrow, Meanders Guide,
//! Wanderbrine Trapper; Turtles in Time (Donatello, Turtle Techie).
//! Tests in `tests/recent_b/cmdr_01dj_b.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, LandType, SelectionRequirement as R, Subtypes,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, LookPick, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, cost, generic, hybrid, u, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn merfolk() -> R {
    R::HasCreatureType(CreatureType::Merfolk)
}

/// Champions of the Shoal — {3}{U} 4/6 Merfolk Soldier. As an additional
/// cost, behold a Merfolk and exile it. Whenever it enters or becomes tapped,
/// tap up to one target creature and put a stun counter on it. When it
/// leaves, return the exiled card to its owner's hand.
pub fn champions_of_the_shoal() -> CardDefinition {
    let stun = || Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(R::Creature),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::ONE,
            },
        ])),
    };
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::BeholdAndExile { filter: merfolk() }],
        triggered_abilities: vec![
            etb(stun()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                effect: stun(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::CardExiledWithSource))),
                },
            },
        ],
        ..creature(
            "Champions of the Shoal",
            cost(&[generic(3), u()]),
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            4,
            6,
        )
    }
}

/// Eclipsed Merrow — {W/U}{W/U}{W/U} 2/3 Merfolk Scout. Enters: look at the
/// top four; you may put a Merfolk, Plains or Island card from among them
/// into your hand; the rest go to the bottom in a random order.
pub fn eclipsed_merrow() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(
                merfolk()
                    .or(R::HasLandType(LandType::Plains))
                    .or(R::HasLandType(LandType::Island)),
            ),
            take: Some(Value::ONE),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })))],
        ..creature(
            "Eclipsed Merrow",
            cost(&[wu(), wu(), wu()]),
            vec![CreatureType::Merfolk, CreatureType::Scout],
            2,
            3,
        )
    }
}

/// Meanders Guide — {2}{W} 3/2 Merfolk Scout. Whenever it attacks, you may
/// tap another untapped Merfolk you control. When you do, return target
/// creature card with mana value 3 or less from your graveyard to the
/// battlefield.
pub fn meanders_guide() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayTap {
                description: "Tap another untapped Merfolk to return a creature card with mana value 3 or less?".into(),
                filter: merfolk().and(R::ControlledByYou).and(R::OtherThanSource),
                count: Value::ONE,
                // CR 603.12 — "When you do": the target is chosen after the tap.
                then: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(Effect::Move {
                        what: target_filtered(
                            R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(3)),
                        ),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    }),
                }),
                else_: None,
            },
        }],
        ..creature(
            "Meanders Guide",
            cost(&[generic(2), w()]),
            vec![CreatureType::Merfolk, CreatureType::Scout],
            3,
            2,
        )
    }
}

/// Wanderbrine Trapper — {W} 2/1 Merfolk Scout. {1}, {T}, tap another
/// untapped creature you control: tap target creature an opponent controls.
pub fn wanderbrine_trapper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            tap_other_filter: Some(R::Creature.and(R::ControlledByYou).and(R::Untapped)),
            effect: Effect::Tap {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
            ..Default::default()
        }],
        ..creature(
            "Wanderbrine Trapper",
            cost(&[w()]),
            vec![CreatureType::Merfolk, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// Turtles in Time — {5}{U}{U} sorcery. Return all creatures to their owners'
/// hands. Each player may shuffle their hand and graveyard into their
/// library, then each player who does draws seven cards. Exile Turtles in
/// Time.
pub fn turtles_in_time() -> CardDefinition {
    CardDefinition {
        name: "Turtles in Time",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::EachPermanent(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::EachPlayerMayShuffleHandAndGraveyardThenDraw { draw: 7 },
        ]),
        exile_on_resolve: true,
        ..Default::default()
    }
}
