use crate::card::{CardDefinition, CardType, SelectionRequirement, Subtypes};
use crate::effect::shortcut::{
    add_mana, counter_target_spell, deal, draw, exile_target, pump_target, target, target_filtered,
};
use crate::effect::Effect;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Swords to Plowshares — {W}: exile target creature
pub fn swords_to_plowshares() -> CardDefinition {
    CardDefinition {
        name: "Swords to Plowshares",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: exile_target(),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Counterspell — {U}{U}: counter target spell
pub fn counterspell() -> CardDefinition {
    CardDefinition {
        name: "Counterspell",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: counter_target_spell(),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Ancestral Recall — {U}: draw 3 cards
pub fn ancestral_recall() -> CardDefinition {
    CardDefinition {
        name: "Ancestral Recall",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: draw(3),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Dark Ritual — {B}: add {B}{B}{B} to your mana pool
pub fn dark_ritual() -> CardDefinition {
    CardDefinition {
        name: "Dark Ritual",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: add_mana(vec![Color::Black, Color::Black, Color::Black]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Terror — {1}{B}: destroy target non-black, non-artifact creature
pub fn terror() -> CardDefinition {
    let filter = SelectionRequirement::Creature
        .and(SelectionRequirement::HasColor(Color::Black).negate())
        .and(SelectionRequirement::HasCardType(CardType::Artifact).negate());
    CardDefinition {
        name: "Terror",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy { what: target_filtered(filter) },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Lightning Bolt — {R}: deal 3 damage to any target
pub fn lightning_bolt() -> CardDefinition {
    CardDefinition {
        name: "Lightning Bolt",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: deal(3, target()),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Giant Growth — {G}: target creature gets +3/+3 until end of turn
pub fn giant_growth() -> CardDefinition {
    CardDefinition {
        name: "Giant Growth",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: pump_target(3, 3),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Fog — {G} Instant. "Prevent all combat damage that would be dealt this
/// turn." (CR 615.1)
pub fn fog() -> CardDefinition {
    CardDefinition {
        name: "Fog",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PreventAllCombatDamageThisTurn,
        ..Default::default()
    }
}

/// Holy Day — {W} Instant. "Prevent all combat damage that would be dealt
/// this turn." (White Fog.)
pub fn holy_day() -> CardDefinition {
    CardDefinition {
        name: "Holy Day",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PreventAllCombatDamageThisTurn,
        ..Default::default()
    }
}

