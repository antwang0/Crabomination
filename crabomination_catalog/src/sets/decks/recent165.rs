//! A Foundations wave — Raid tempo, a tap-down Aura, and turn/threshold
//! conditional keywords. Tests in `crabomination/src/tests/recent165.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Value,
};
use crate::effect::shortcut::{etb, etb_loot, on_attack_loot, raid_etb, target_filtered};
use crate::effect::{Effect, PlayerRef};
use crate::mana::{cost, generic, u, w};

/// Skyship Buccaneer — {3}{U}{U} 4/3 Human Pirate. Flying. Raid — when it enters,
/// if you attacked this turn, draw a card.
pub fn skyship_buccaneer() -> CardDefinition {
    CardDefinition {
        name: "Skyship Buccaneer",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![raid_etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        ..Default::default()
    }
}

/// Starlight Snare — {2}{U} Aura. Enchant creature. When it enters, tap enchanted
/// creature. Enchanted creature doesn't untap during its controller's untap step.
pub fn starlight_snare() -> CardDefinition {
    CardDefinition {
        name: "Starlight Snare",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::Tap {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::AttachedTo(Box::new(Selector::This)) },
        }],
        ..Default::default()
    }
}

/// Inspiring Paladin — {2}{W} 3/3 Human Knight. During your turn, it has first
/// strike. (The team-wide "your +1/+1-countered creatures have first strike"
/// rider is dropped.)
pub fn inspiring_paladin() -> CardDefinition {
    CardDefinition {
        name: "Inspiring Paladin",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::FirstStrike,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Dreadwing Scavenger — {1}{U}{B} 2/2 Nightmare Bird. Flying. When it enters or
/// attacks, draw a card, then discard a card. Threshold — it has deathtouch as
/// long as seven or more cards are in your graveyard. (The Threshold +1/+1 is
/// dropped.)
pub fn dreadwing_scavenger() -> CardDefinition {
    use crate::mana::b;
    CardDefinition {
        name: "Dreadwing Scavenger",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Threshold — has deathtouch while seven or more cards are in your graveyard.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Deathtouch,
                condition: Predicate::ThresholdActive { who: PlayerRef::You },
            },
        }],
        triggered_abilities: vec![etb_loot(), on_attack_loot()],
        ..Default::default()
    }
}
