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

/// Mosswood Dreadknight // Dread Whispers — {1}{G} 3/2 trample Human Knight.
/// When it dies, you may cast it from your graveyard as an Adventure until the
/// end of your next turn. Dread Whispers ({1}{B} sorcery — Adventure): you
/// draw a card and you lose 1 life.
pub fn mosswood_dreadknight() -> CardDefinition {
    CardDefinition {
        name: "Mosswood Dreadknight",
        cost: cost(&[generic(1), crate::mana::g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Knight], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::GrantAdventureFromGraveyard)],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Dread Whispers",
            cost: cost(&[generic(1), crate::mana::b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::Draw { who: crate::effect::Selector::You, amount: crate::card::Value::ONE },
                Effect::LoseLife { who: crate::effect::Selector::You, amount: crate::card::Value::ONE },
            ]),
        })),
        ..Default::default()
    }
}
