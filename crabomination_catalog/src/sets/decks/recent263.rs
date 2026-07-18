//! Mixed gap batch. Glacial Dragonhunt (TDM) rides a filtered reflexive
//! discard on existing primitives. Tests in `tests/recent_b/recent263.rs`.

use crate::card::{CardDefinition, CardType, Keyword, SelectionRequirement as R};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef, Predicate, Selector, Value};
use crate::mana::{cost, generic, r, u};

/// Glacial Dragonhunt — {U}{R} Sorcery. Draw a card, then you may discard a
/// card. When you discard a nonland card this way, deal 3 damage to target
/// creature. Harmonize {4}{U}{R}.
pub fn glacial_dragonhunt() -> CardDefinition {
    CardDefinition {
        name: "Glacial Dragonhunt",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Harmonize(cost(&[generic(4), u(), r()]))],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::MayDiscard {
                description: "Discard a card?".into(),
                count: Value::Const(1),
                // Reflexive damage only when the discarded card was a nonland.
                then: Box::new(Effect::If {
                    cond: Predicate::DiscardedNonlandThisEffect { who: PlayerRef::You },
                    then: Box::new(Effect::Reflexive {
                        body: Box::new(Effect::DealDamage {
                            to: target_filtered(R::Creature),
                            amount: Value::Const(3),
                        }),
                    }),
                    else_: Box::new(Effect::Noop),
                }),
                else_: None,
            },
        ]),
        ..Default::default()
    }
}
