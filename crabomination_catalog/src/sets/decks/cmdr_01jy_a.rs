//! Commander: the 2026-10-21 (`01Jywzky`) pair-deck closers. Tests in
//! `tests/recent_b/cmdr_01jy_a.rs`.

use crate::card::{CardDefinition, CardType, SelectionRequirement as R};
use crate::effect::shortcut::{draw, target_filtered};
use crate::effect::{DelayedTriggerKind, Effect, PlayerRef, Predicate, Selector};
use crate::mana::{Color, cost, generic, r, u};

/// Burnout — {1}{R} instant. Counter target instant spell if it's blue. Draw a
/// card at the beginning of the next turn's upkeep.
pub fn burnout() -> CardDefinition {
    CardDefinition {
        name: "Burnout",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::HasColor(Color::Blue) },
                then: Box::new(Effect::CounterSpell {
                    what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Instant))),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::DelayUntil { kind: DelayedTriggerKind::NextUpkeep, body: Box::new(draw(1)) },
        ]),
        ..Default::default()
    }
}

/// Mana Short — {2}{U} instant. Tap all lands target player controls and that
/// player loses all unspent mana.
pub fn mana_short() -> CardDefinition {
    CardDefinition {
        name: "Mana Short",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Tap { what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Land } },
            Effect::LoseUnspentMana { who: PlayerRef::Target(0) },
        ]),
        ..Default::default()
    }
}
