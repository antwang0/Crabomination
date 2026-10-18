//! Commander: 2026-10-18 (`019Y6J4R`) closers that build on the WOE Role
//! tokens (`super::woe_roles`), so this file joins the `decks2` part. Tests in
//! `tests/recent_b/cmdr_019y_n.rs`.

use crate::card::{CardDefinition, CardType, EnchantmentSubtype, SelectionRequirement as R, Subtypes, Value};
use crate::effect::{Effect, PlayerRef, Selector};
use crate::mana::{cost, generic, u};
use std::sync::Arc;

/// Gadwick's First Duel — {1}{U} Saga. I — a Cursed Role token attached to up
/// to one target creature. II — scry 2. III — when you next cast an instant or
/// sorcery spell with mana value 3 or less this turn, copy it; you may choose
/// new targets for the copy.
pub fn gadwicks_first_duel() -> CardDefinition {
    CardDefinition {
        name: "Gadwick's First Duel",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::CreateTokenAttachedTo {
                        target: Selector::Target(0),
                        definition: Arc::new(super::woe_roles::cursed_role()),
                    }),
                },
            ),
            (2, Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
            (
                3,
                Effect::OnYourNextSpellMatchingThisTurn {
                    filter: R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .and(R::ManaValueAtMost(3)),
                    body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
                },
            ),
        ],
        ..Default::default()
    }
}
