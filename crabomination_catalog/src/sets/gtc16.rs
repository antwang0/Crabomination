//! Gatecrash (GTC) wave 16: the remaining primitive-gated guild cards. Tests in
//! `classic_sets/gtc`.

use crate::card::{CardDefinition, CardType, SelectionRequirement as R, Value};
use crate::effect::{Effect, Selector};
use crate::mana::{cost, r, w, x};

/// Aurelia's Fury — {X}{R}{W} Instant. Deals X damage divided among any number
/// of targets; each creature dealt damage this way is tapped, and each player
/// dealt damage this way can't cast noncreature spells this turn.
pub fn aurelias_fury() -> CardDefinition {
    CardDefinition {
        name: "Aurelia's Fury",
        cost: cost(&[x(), r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamageDivided {
                total: Value::XFromCost,
                filter: R::Creature.or(R::Player).or(R::Planeswalker),
                max_targets: 20,
            },
            // `DamagedThisResolution` yields only creatures + players; Tap
            // ignores the players and the noncreature lock ignores the
            // creatures.
            Effect::Tap { what: Selector::DamagedThisResolution { filter: R::Creature } },
            Effect::CantCastNoncreatureThisTurn {
                who: Selector::DamagedThisResolution { filter: R::Creature },
            },
        ]),
        ..Default::default()
    }
}
