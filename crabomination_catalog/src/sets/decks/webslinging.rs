//! **Web-slinging** cards (CR 702.188). A static ability on the stack: "You may
//! cast this spell by paying [cost] and returning a tapped creature you control
//! to its owner's hand rather than paying its mana cost." Modeled on the shared
//! alternative-cost primitive (`AlternativeCost.mana_cost` + `return_to_hand` of
//! one tapped creature) — `GameAction::CastSpellAlternative` already pays the
//! web-slinging cost and bounces the tapped creature. Tracked in `DECK_FEATURES.md`.

use crate::card::{ActivatedAbility, TokenDefinition};
use crate::card::{
    AlternativeCost, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement, Selector, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::Duration;
use crate::effect::shortcut::target_filtered;
use crate::mana::{Color, ManaCost, cost, g, generic, w};

/// CR 702.188 — the web-slinging alternative cost: pay `mana` and return one
/// tapped creature you control to its owner's hand.
fn web_slinging(mana: ManaCost) -> AlternativeCost {
    AlternativeCost {
        mana_cost: mana,
        return_to_hand: Some((
            SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
            1,
        )),
        ..Default::default()
    }
}

/// Spider-Man, Web-Slinger — {2}{W} 3/3 Legendary Spider Human Hero.
/// Web-slinging {W}.
pub fn spider_man_web_slinger() -> CardDefinition {
    CardDefinition {
        name: "Spider-Man, Web-Slinger",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spider,
                CreatureType::Human,
                CreatureType::Hero,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        alternative_cost: Some(web_slinging(cost(&[w()]))),
        ..Default::default()
    }
}

/// Amazing Spider-Girl — {3}{W}{W} 5/4 Legendary Spider Human Hero with Flying
/// and Vigilance. Web-slinging {2}{W}.
pub fn amazing_spider_girl() -> CardDefinition {
    CardDefinition {
        name: "Amazing Spider-Girl",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spider,
                CreatureType::Human,
                CreatureType::Hero,
            ],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        alternative_cost: Some(web_slinging(cost(&[generic(2), w()]))),
        ..Default::default()
    }
}

/// Silk, Web Weaver — {2}{G}{W} 3/5 Legendary Spider Human Hero. Web-slinging
/// {1}{G}{W}. Whenever you cast a creature spell, create a 1/1 green and white
/// Human Citizen token. `{3}{G}{W}: Creatures you control get +2/+2 and gain
/// vigilance until end of turn.`
pub fn silk_web_weaver() -> CardDefinition {
    CardDefinition {
        name: "Silk, Web Weaver",
        cost: cost(&[generic(2), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spider,
                CreatureType::Human,
                CreatureType::Hero,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        alternative_cost: Some(web_slinging(cost(&[generic(1), g(), w()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                },
            ),
            effect: Effect::CreateToken {
                who: crate::effect::PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(TokenDefinition {
                    name: "Human Citizen".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green, Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Human, CreatureType::Citizen],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g(), w()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spider-Man India — {3}{G}{W} 4/4 Legendary Spider Human Hero. Web-slinging
/// {1}{G}{W}. Whenever you cast a creature spell, put a +1/+1 counter on target
/// creature you control; it gains flying until end of turn.
pub fn spider_man_india() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Spider-Man India",
        cost: cost(&[generic(3), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spider,
                CreatureType::Human,
                CreatureType::Hero,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        alternative_cost: Some(web_slinging(cost(&[generic(1), g(), w()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                },
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}
