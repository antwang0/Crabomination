//! Remaining real STX (Strixhaven 2021) printed cards — final sweep. These
//! ride existing primitives plus the new `SelectionRequirement::EnteredThisTurn`
//! and `Duration::Permanent` land-animation. Each ships with a test in
//! `crate::tests::stx`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword, Predicate, Selector,
    SelectionRequirement, StaticAbility, StaticEffect, Subtypes, Supertype, Value, WardCost, Zone,
};
use crate::effect::shortcut::{dies_gain_life, draw, etb};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u};

/// Emergent Sequence — {1}{G} Sorcery. Search your library for a basic land,
/// put it onto the battlefield tapped, then shuffle. That land becomes a 0/0
/// green-and-blue Fractal creature that's still a land. Put a +1/+1 counter on
/// it for each land you had enter under your control this turn.
pub fn emergent_sequence() -> CardDefinition {
    CardDefinition {
        name: "Emergent Sequence",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::BecomeCreature {
                what: Selector::LastMoved,
                power: Value::Const(0),
                toughness: Value::Const(0),
                creature_types: vec![CreatureType::Fractal],
                keywords: vec![],
                duration: Duration::Permanent,
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Land
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::EnteredThisTurn),
                )),
            },
        ]),
        ..Default::default()
    }
}

/// Augmenter Pugilist — {1}{G}{G} 3/3 Troll Druid with trample. While you
/// control eight or more lands it gets +5/+5. (Front face of the Augmenter
/// Pugilist // Echoing Equation MDFC; the back-face copy effect is a
/// continuous layer-1 rewrite not yet modeled, so only the creature ships.)
pub fn augmenter_pugilist() -> CardDefinition {
    CardDefinition {
        name: "Augmenter Pugilist",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "As long as you control eight or more lands, this gets +5/+5.",
            effect: StaticEffect::PumpSelfIf {
                condition: crate::card::Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(8),
                },
                power: 5,
                toughness: 5,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Flamethrower Sonata — {1}{R} Sorcery, back face of Torrent Sculptor.
/// Discard a card, then draw a card. When you discard an instant or sorcery
/// card this way, deal damage equal to its mana value to target creature or
/// planeswalker you don't control. (A target is chosen on cast; the damage is
/// skipped when the discard isn't an instant/sorcery.)
fn flamethrower_sonata() -> CardDefinition {
    let is = SelectionRequirement::HasCardType(CardType::Instant)
        .or(SelectionRequirement::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Flamethrower Sonata",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            draw(1),
            Effect::If {
                cond: Predicate::SelectorExists(Selector::DiscardedThisResolution {
                    filter: is.clone(),
                }),
                then: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::ManaValueOf(Box::new(Selector::DiscardedThisResolution {
                        filter: is,
                    })),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Torrent Sculptor // Flamethrower Sonata — {2}{U}{U} 2/2 Merfolk Wizard with
/// Ward {2}. ETB: exile an instant or sorcery card from your graveyard and put
/// +1/+1 counters on this equal to half that card's mana value, rounded up.
pub fn torrent_sculptor() -> CardDefinition {
    let is = SelectionRequirement::HasCardType(CardType::Instant)
        .or(SelectionRequirement::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Torrent Sculptor",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: is,
                    }),
                    count: Box::new(Value::Const(1)),
                },
                to: ZoneDest::Exile,
            },
            // ceil(mv/2) = floor((mv+1)/2).
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::HalfDown(Box::new(Value::Sum(vec![
                    Value::ManaValueOf(Box::new(Selector::LastMoved)),
                    Value::Const(1),
                ]))),
            },
        ]))],
        back_face: Some(Box::new(flamethrower_sonata())),
        ..Default::default()
    }
}

/// Search for Blex — {2}{B}{B} Sorcery, back face of Blex. Look at the top five
/// cards of your library; put any number into your hand and the rest into your
/// graveyard. Lose 3 life for each card put into your hand this way.
fn search_for_blex() -> CardDefinition {
    CardDefinition {
        name: "Search for Blex",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DigToHandLoseLife {
            count: Value::Const(5),
            life_per_card: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Blex, Vexing Pest // Search for Blex — {2}{G} 3/2 Legendary Pest. Anthem for
/// your Pests/Bats/Insects/Snakes/Spiders; dies → gain 4 life.
pub fn blex_vexing_pest() -> CardDefinition {
    let kin = SelectionRequirement::HasCreatureType(CreatureType::Pest)
        .or(SelectionRequirement::HasCreatureType(CreatureType::Bat))
        .or(SelectionRequirement::HasCreatureType(CreatureType::Insect))
        .or(SelectionRequirement::HasCreatureType(CreatureType::Snake))
        .or(SelectionRequirement::HasCreatureType(CreatureType::Spider));
    CardDefinition {
        name: "Blex, Vexing Pest",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Other Pests, Bats, Insects, Snakes, and Spiders you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    kin.and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![dies_gain_life(4)],
        back_face: Some(Box::new(search_for_blex())),
        ..Default::default()
    }
}
