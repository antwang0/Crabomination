//! Commander: 2026-10-18 (`019Y6J4R`) closers, nineteenth batch. Tests in
//! `tests/recent_b/cmdr_019y_s.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Subtypes, Supertype,
    TriggeredAbility,
};
use crate::effect::Effect;
use crate::mana::{Color, cost, generic, hybrid};

/// Don & Raph, Hard Science — {1}{U/R}{U/R} legendary 2/4 menace Mutant Ninja
/// Turtle. Whenever they attack, the next noncreature spell you cast this turn
/// has affinity for artifacts (CR 702.41a).
pub fn don_raph_hard_science() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    CardDefinition {
        name: "Don & Raph, Hard Science",
        cost: cost(&[generic(1), ur(), ur()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::NextNoncreatureSpellHasAffinityForArtifacts,
        }],
        ..Default::default()
    }
}
