//! MID/VOW gap batch — a modified-matters pump and a tuck-and-Zombie tempo
//! instant. All on existing primitives. Tests in `tests/recent_b/recent272.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, Predicate, SelectionRequirement as R,
    Subtypes, TokenDefinition,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{cost, generic, r, u, Color};

/// Ambitious Assault — {2}{R} Instant. Creatures you control get +2/+0 until end
/// of turn. If you control a modified creature, draw a card.
pub fn ambitious_assault() -> CardDefinition {
    CardDefinition {
        name: "Ambitious Assault",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::IsModified),
                    ),
                    n: Value::ONE,
                },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Revenge of the Drowned — {3}{U} Instant. Target creature's owner puts it on
/// their choice of the top or bottom of their library. You create a 2/2 black
/// Zombie creature token with decayed.
pub fn revenge_of_the_drowned() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        keywords: vec![Keyword::Decayed],
        ..Default::default()
    };
    CardDefinition {
        name: "Revenge of the Drowned",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::OwnerChoice,
                },
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: zombie },
        ]),
        ..Default::default()
    }
}
