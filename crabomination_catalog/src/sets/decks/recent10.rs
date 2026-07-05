//! A tenth staples wave — simple Avatar/Lorwyn commons riding existing
//! primitives (ETB value, prowess, sacrifice-matters). Tests in
//! `crabomination/src/tests/recent10.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate};
use crabomination_base::tokens::clue_token;

/// A 1/1 white Ally token.
fn white_ally() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Ally".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ally], ..Default::default() },
        ..Default::default()
    }
}

/// Glider Kids — {2}{W} 2/3, Flying. When it enters, scry 1.
pub fn glider_kids() -> CardDefinition {
    use crate::mana::{cost, generic, w};
    CardDefinition {
        name: "Glider Kids",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot, CreatureType::Ally],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::ONE })],
        ..Default::default()
    }
}

/// Messenger Hawk — {2}{U/B} 1/2, Flying. When it enters, create a Clue.
pub fn messenger_hawk() -> CardDefinition {
    use crate::mana::{cost, generic, hybrid, Color};
    CardDefinition {
        name: "Messenger Hawk",
        cost: cost(&[generic(2), hybrid(Color::Blue, Color::Black)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: clue_token(),
        })],
        ..Default::default()
    }
}

/// Ostrich-Horse — {2}{G} 3/1. When it enters, mill three cards; you may put a
/// land card from among them into your hand.
pub fn ostrich_horse() -> CardDefinition {
    use crate::mana::{cost, g, generic};
    CardDefinition {
        name: "Ostrich-Horse",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Horse],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::MillThenToHand {
            amount: Value::Const(3),
            filter: SelectionRequirement::Land,
            otherwise: None,
        })],
        ..Default::default()
    }
}

/// Rowdy Snowballers — {2}{U} 2/2. When it enters, tap target creature an
/// opponent controls.
pub fn rowdy_snowballers() -> CardDefinition {
    use crate::mana::{cost, generic, u};
    CardDefinition {
        name: "Rowdy Snowballers",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant, CreatureType::Ally],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Tap {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        ..Default::default()
    }
}

/// Treetop Freedom Fighters — {2}{R} 2/1, Haste. When it enters, create a 1/1
/// white Ally.
pub fn treetop_freedom_fighters() -> CardDefinition {
    use crate::mana::{cost, generic, r};
    CardDefinition {
        name: "Treetop Freedom Fighters",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Ally],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: white_ally(),
        })],
        ..Default::default()
    }
}

/// Pirate Peddlers — {2}{B} 2/2, Deathtouch. Whenever you sacrifice another
/// permanent, put a +1/+1 counter on it.
pub fn pirate_peddlers() -> CardDefinition {
    use crate::mana::{b, cost, generic};
    CardDefinition {
        name: "Pirate Peddlers",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::OtherThanSource,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Iguana Parrot — {2}{U} 2/2, Flying, Vigilance, Prowess.
pub fn iguana_parrot() -> CardDefinition {
    use crate::mana::{cost, generic, u};
    CardDefinition {
        name: "Iguana Parrot",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Bird, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Prowess],
        ..Default::default()
    }
}

/// Boar-q-pine — {2}{R} 2/2. Whenever you cast a noncreature spell, put a
/// +1/+1 counter on it.
pub fn boar_q_pine() -> CardDefinition {
    use crate::mana::{cost, generic, r};
    CardDefinition {
        name: "Boar-q-pine",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Boar], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}
