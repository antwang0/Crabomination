//! Blue Wizard payoffs (batch 4). All ride existing primitives (tap-other
//! costs, `LookPickToHand`, kicker-conditional ETB, the I/S-graveyard CDA).
//! Tests in `tests/recent93.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, DynamicPt, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{draw, etb, target_filtered};
use crate::effect::{PlayerRef, ZoneDest};
use crate::mana::{cost, generic, r, u, x, Color};

/// Galecaster Colossus — {5}{U}{U} 5/6 Giant Wizard. Tap an untapped Wizard you
/// control: return target nonland permanent you don't control to its owner's
/// hand.
pub fn galecaster_colossus() -> CardDefinition {
    CardDefinition {
        name: "Galecaster Colossus",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::HasCreatureType(CreatureType::Wizard)),
            effect: Effect::Move {
                what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gadwick, the Wizened — {X}{U}{U}{U} 3/3 Human Wizard. Enters → draw X. Cast a
/// blue spell → tap target nonland permanent an opponent controls.
pub fn gadwick_the_wizened() -> CardDefinition {
    CardDefinition {
        name: "Gadwick, the Wizened",
        cost: cost(&[x(), u(), u(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::Draw { who: Selector::You, amount: Value::XFromCost }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasColor(Color::Blue),
                    },
                ),
                effect: Effect::Tap {
                    what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
                },
            },
        ],
        ..Default::default()
    }
}

/// Sphinx of Lost Truths — {3}{U}{U} 3/5 Sphinx, flying, Kicker {1}{U}. Enters →
/// draw three, then if it wasn't kicked, discard three.
pub fn sphinx_of_lost_truths() -> CardDefinition {
    CardDefinition {
        name: "Sphinx of Lost Truths",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sphinx], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[generic(1), u()]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            draw(3),
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(3),
                    random: false,
                }),
            },
        ]))],
        ..Default::default()
    }
}

/// Rielle, the Everwise — {1}{U}{R} 0/3 Human Wizard. Gets +1/+0 for each
/// instant/sorcery card in your graveyard. (The first-discard-each-turn "draw
/// that many" trigger is dropped.)
pub fn rielle_the_everwise() -> CardDefinition {
    CardDefinition {
        name: "Rielle, the Everwise",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        dynamic_pt: Some(DynamicPt::InstantsSorceriesInControllerGraveyard { base_t: 3 }),
        ..Default::default()
    }
}
