//! MKM (Murders at Karlov Manor) gap batch — Gruul X burn + land ramp,
//! Izzet loot-and-burn. Tests in `tests/recent_b/recent262.rs`.

use crate::card::{CardDefinition, CardType, SelectionRequirement as R};
use crate::effect::shortcut::target;
use crate::effect::{Effect, Selector, Value};
use crate::mana::{cost, g, generic, r, u, x};

/// Worldsoul's Rage — {X}{R}{G} Sorcery. Deals X damage to any target, then
/// puts up to X land cards from your hand and/or graveyard onto the
/// battlefield tapped.
pub fn worldsouls_rage() -> CardDefinition {
    CardDefinition {
        name: "Worldsoul's Rage",
        cost: cost(&[x(), r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target(),
                amount: Value::XFromCost,
            },
            Effect::DeployLandsFromHandAndGraveyard {
                count: Value::XFromCost,
            },
        ]),
        ..Default::default()
    }
}

/// Ill-Timed Explosion — {2}{U}{R} Sorcery. Draw two cards, then you may
/// discard two cards; if you do, deal X damage to each creature, where X is
/// the greatest mana value among the cards discarded this way.
pub fn ill_timed_explosion() -> CardDefinition {
    CardDefinition {
        name: "Ill-Timed Explosion",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::MayDiscard {
                description: "Discard two cards?".into(),
                count: Value::Const(2),
                then: Box::new(Effect::ForEach {
                    selector: Selector::EachPermanent(R::Creature),
                    body: Box::new(Effect::DealDamage {
                        to: Selector::TriggerSource,
                        amount: Value::GreatestDiscardedManaValueThisEffect,
                    }),
                }),
                else_: None,
            },
        ]),
        ..Default::default()
    }
}
