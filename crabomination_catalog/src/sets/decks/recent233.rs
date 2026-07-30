//! Gap batch — OTJ Spree instants on existing primitives. Tests in
//! `tests/recent233.rs`.

use crate::card::{CardDefinition, CardType, CreatureType, SelectionRequirement as R};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Selector, SpreeMode, Value};
use crate::mana::{ManaCost, cost, generic, r, u};

fn spree(modes: Vec<SpreeMode>) -> Effect {
    Effect::Spree { modes }
}
fn mode(c: ManaCost, effect: Effect) -> SpreeMode {
    SpreeMode { cost: c, effect }
}

/// Metamorphic Blast — {U} Instant. Spree: +{1} until end of turn, target
/// creature becomes a 0/1 Rabbit; +{3} target player draws two cards. (The
/// Rabbit's white color is approximated — only its base P/T and type change.)
pub fn metamorphic_blast() -> CardDefinition {
    CardDefinition {
        name: "Metamorphic Blast",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: spree(vec![
            mode(
                cost(&[generic(1)]),
                Effect::BecomeCreature {
                    what: target_filtered(R::Creature),
                    power: Value::Const(0),
                    toughness: Value::Const(1),
                    creature_types: vec![CreatureType::Rabbit],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
            ),
            mode(
                cost(&[generic(3)]),
                Effect::Draw {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                },
            ),
        ]),
        ..Default::default()
    }
}

/// Return the Favor — {R}{R} Instant. Spree: +{1} copy target instant or sorcery
/// spell (you may choose new targets); +{1} change the target of target spell or
/// ability with a single target. (The "activated/triggered ability" copy target
/// is approximated to instant/sorcery spells.)
pub fn return_the_favor() -> CardDefinition {
    let a_spell = || {
        target_filtered(
            R::IsSpellOnStack
                .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
        )
    };
    CardDefinition {
        name: "Return the Favor",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        effect: spree(vec![
            mode(
                cost(&[generic(1)]),
                Effect::CopySpellMayChooseTargets {
                    what: a_spell(),
                    count: Value::Const(1),
                },
            ),
            mode(
                cost(&[generic(1)]),
                Effect::ChooseNewTargetsForSpell {
                    what: target_filtered(R::IsSpellOnStack),
                },
            ),
        ]),
        ..Default::default()
    }
}
