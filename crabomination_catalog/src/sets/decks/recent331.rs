//! The most-built **commanders** the catalog was missing —
//! `COMMANDER_BACKLOG.md`'s section 1 rather than its staples list.
//!
//! The split from `sets::cmdr` is the one that file's header draws: `cmdr.rs`
//! is for cards whose printed text is *about* the format (it reads the command
//! zone, a commander, or a colour identity). A most-built commander is an
//! ordinary legendary creature; what makes it a commander is the deck it
//! leads, not its text.
//!
//! ⚠ The theme here is narrower than "commanders": these are the ones whose
//! text is about the **whole table**, so they do something different at two
//! seats than at eight, and they are worth having for the pod runner even
//! before a deck is built around them.
//!
//! ⚠⚠ **Check the catalog before writing one.** Ayara, First of Locthwain was
//! drafted for this module and dropped: the concurrent session had already
//! shipped it in `recent330` while this file was being written, and the
//! generated `COMMANDER_BACKLOG.md` a batch is sized from is a snapshot, not
//! a lock. `grep -rn "pub fn <slug>" crabomination_catalog/src` costs one
//! second; the ambiguous-glob error that catches it otherwise costs a build.
//!
//! Tests in `tests/core_rules/commander_cards.rs`, which already holds this
//! branch's multi-seat card tests.

use crate::card::{
    CardDefinition, CardType, CreatureType, Selector, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::{Effect, EventKind, EventScope, EventSpec, PlayerRef};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, u};

/// Nekusar, the Mindrazer — {2}{U}{B}{R} Legendary Creature — Zombie Wizard
/// 2/4. "At the beginning of each player's draw step, that player draws an
/// additional card. Whenever an opponent draws a card, Nekusar deals 1 damage
/// to that player." (EDHREC's 18th most-built commander.)
///
/// Both halves are shapes the catalog already ships: the first is Howling
/// Mine's exactly (`StepBegins(Draw)` scoped to `AnyPlayer`, drawing for the
/// active player — the player whose draw step it is), and the second is
/// Underworld Dreams' exactly (`CardDrawn` scoped to `OpponentControl`,
/// damaging `PlayerRef::Triggerer`).
///
/// 📐 It is the clearest card in the catalog for what a seat count does to a
/// board: at two seats it draws one opponent an extra card a turn and pings
/// them once, and at eight it does that to seven players — the symmetric
/// half scales with the table and the damage half only hits opponents.
pub fn nekusar_the_mindrazer() -> CardDefinition {
    CardDefinition {
        name: "Nekusar, the Mindrazer",
        cost: cost(&[generic(2), u(), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Draw),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}
