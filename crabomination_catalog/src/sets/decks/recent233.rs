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
/// creature becomes a white Rabbit with base P/T 0/1; +{3} target player draws
/// two cards.
pub fn metamorphic_blast() -> CardDefinition {
    CardDefinition {
        name: "Metamorphic Blast",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: spree(vec![
            mode(
                cost(&[generic(1)]),
                Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: target_filtered(R::Creature),
                        power: Value::Const(0),
                        toughness: Value::Const(1),
                        creature_types: vec![CreatureType::Rabbit],
                        keywords: vec![],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::BecomeColor {
                        what: Selector::Target(0),
                        colors: vec![crate::mana::Color::White],
                        duration: Duration::EndOfTurn,
                        additive: false,
                    },
                ]),
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

/// Return the Favor — {R}{R} Instant. Spree: +{1} copy target instant spell,
/// sorcery spell, activated ability, or triggered ability (you may choose new
/// targets); +{1} change the target of target spell or ability with a single
/// target.
pub fn return_the_favor() -> CardDefinition {
    let spell_or_ability = || {
        target_filtered(
            R::IsSpellOnStack
                .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
                .or(R::HasAbilityOnStack),
        )
    };
    CardDefinition {
        name: "Return the Favor",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        effect: spree(vec![
            mode(
                cost(&[generic(1)]),
                Effect::If {
                    cond: crate::effect::Predicate::EntityMatches { what: Selector::Target(0), filter: R::IsSpellOnStack },
                    then: Box::new(Effect::CopySpellMayChooseTargets { what: spell_or_ability(), count: Value::Const(1) }),
                    else_: Box::new(Effect::CopyAbility { what: Selector::Target(0), times: Value::Const(1) }),
                },
            ),
            mode(
                cost(&[generic(1)]),
                Effect::ChooseNewTargetsForSpell {
                    what: target_filtered(R::IsSpellOnStack.or(R::HasAbilityOnStack).and(R::SpellWithSingleTarget)),
                },
            ),
        ]),
        ..Default::default()
    }
}
