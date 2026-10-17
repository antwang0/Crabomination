//! Commander: 2026-10-17 (`019Y6J4R`) closers, seventh batch. Tests in
//! `tests/recent_b/cmdr_019y_g.rs`.

use crate::card::{ActivatedAbility, CardDefinition, CardType, CreatureType, Subtypes, Supertype, Value};
use crate::effect::{Effect, ManaPayload, PlayerRef, Selector};
use crate::mana::{cost, g, generic, u};

/// Alaundo the Seer — {2}{G}{U} legendary 3/5 Human Shaman. {T}: draw a card,
/// then exile a card from your hand with time counters equal to its mana
/// value (it's cast free when the last comes off, a creature with haste), then
/// remove a time counter from each other card you own in exile.
pub fn alaundo_the_seer() -> CardDefinition {
    CardDefinition {
        name: "Alaundo the Seer",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Shaman], ..Default::default() },
        power: 3,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::ExileFromHandWithTimeCountersThenTick { who: PlayerRef::You },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Corrupted Grafstone — {2} artifact, enters tapped. {T}: one mana of a color
/// of a card in your graveyard.
pub fn corrupted_grafstone() -> CardDefinition {
    CardDefinition {
        name: "Corrupted Grafstone",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColorAmongCardsInYourGraveyard },
            ..Default::default()
        }],
        ..Default::default()
    }
}
