//! The EDHREC Commander-staple gap — cards `COMMANDER_BACKLOG.md`'s top-1000
//! slice named and which needed **no new primitive**, only the card.
//!
//! The rule for what lands here rather than in `sets::cmdr`: `cmdr.rs` is for
//! cards whose printed text is *about* the format (it reads the command zone,
//! a commander, or a colour identity). These are ordinary cards that happen to
//! be Commander staples.
//!
//! Tests in `tests/modern/lands_equipment_vehicles.rs` (the module that
//! already holds this branch's card-shape tables).

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, StaticAbility,
    Subtypes, Supertype,
};
use crate::effect::StaticEffect;
use crate::mana::{cost, g, generic, w};

/// Parallel Lives — {3}{G} Enchantment. "If an effect would create one or
/// more tokens under your control, it creates twice that many of those tokens
/// instead."
///
/// Doubling Season's token half on its own, and the same primitive:
/// `StaticEffect::DoubleTokens` is already controller-scoped, which is the
/// printed "under your control".
pub fn parallel_lives() -> CardDefinition {
    CardDefinition {
        name: "Parallel Lives",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If an effect would create one or more tokens under your control, \
                          it creates twice that many of those tokens instead.",
            effect: StaticEffect::DoubleTokens,
        }],
        ..Default::default()
    }
}

/// Avacyn, Angel of Hope — {5}{W}{W}{W} Legendary Creature — Angel 8/8.
/// "Flying, vigilance, indestructible. Other permanents you control have
/// indestructible."
///
/// ⚠ **Permanents, not creatures.** `AnthemForFilter`'s filter is matched
/// against the controller's permanents rather than their creatures, so the
/// printed noun is carried by leaving `R::Creature` out — Avacyn's lands and
/// artifacts are indestructible too, which is most of what the card does in a
/// Commander pod. `R::OtherThanSource` is the printed "Other"; Avacyn's own
/// indestructible is printed separately and is a keyword on the card.
pub fn avacyn_angel_of_hope() -> CardDefinition {
    CardDefinition {
        name: "Avacyn, Angel of Hope",
        cost: cost(&[generic(5), w(), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Indestructible],
        static_abilities: vec![StaticAbility {
            description: "Other permanents you control have indestructible.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::OtherThanSource,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Indestructible],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}
