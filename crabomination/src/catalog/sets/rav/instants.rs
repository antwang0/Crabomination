use crate::card::{CardDefinition, CardType, SelectionRequirement, Subtypes};
use crate::effect::shortcut::{deal, gain_life, target, target_filtered};
use crate::effect::Effect;
use crate::mana::{b, cost, g, generic, r, w};

/// Lightning Helix — {R}{W}: deal 3 damage to any target, you gain 3 life
pub fn lightning_helix() -> CardDefinition {
    CardDefinition {
        name: "Lightning Helix",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![deal(3, target()), gain_life(3)]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Putrefy — {1}{B}{G}: destroy target artifact or creature; it can't be regenerated.
///
/// The "can't be regenerated" clause (CR 701.15g) is wired via
/// `Effect::DestroyNoRegen`, so a regeneration shield won't save the
/// target.
pub fn putrefy() -> CardDefinition {
    CardDefinition {
        name: "Putrefy",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DestroyNoRegen {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}
