//! MKM (Murders at Karlov Manor) gap batch — Gruul X burn + land ramp.
//! Tests in `tests/recent_b/recent262.rs`.

use crate::card::CardDefinition;
use crate::card::CardType;
use crate::effect::shortcut::target;
use crate::effect::{Effect, Value};
use crate::mana::{cost, g, r, x};

/// Worldsoul's Rage — {X}{R}{G} Sorcery. Deals X damage to any target, then
/// puts up to X land cards from your hand and/or graveyard onto the
/// battlefield tapped.
pub fn worldsouls_rage() -> CardDefinition {
    CardDefinition {
        name: "Worldsoul's Rage",
        cost: cost(&[x(), r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target(), amount: Value::XFromCost },
            Effect::DeployLandsFromHandAndGraveyard { count: Value::XFromCost },
        ]),
        ..Default::default()
    }
}
