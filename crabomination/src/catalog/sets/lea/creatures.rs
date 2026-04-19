use super::{no_abilities, tap_add};
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword,
    SelectionRequirement, SpellEffect, Subtypes, TriggerCondition, TriggeredAbility,
};
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};

/// Savannah Lions — {W} 2/1
pub fn savannah_lions() -> CardDefinition {
    CardDefinition {
        name: "Savannah Lions",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat, CreatureType::Lion], ..Default::default() },
        power: 2, toughness: 1,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// White Knight — {W}{W} 2/2 First Strike
pub fn white_knight() -> CardDefinition {
    CardDefinition {
        name: "White Knight",
        cost: cost(&[w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Knight], ..Default::default() },
        power: 2, toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Serra Angel — {3}{W}{W} 4/4 Flying Vigilance
pub fn serra_angel() -> CardDefinition {
    CardDefinition {
        name: "Serra Angel",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 4, toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Mahamoti Djinn — {3}{U}{U} 5/6 Flying
pub fn mahamoti_djinn() -> CardDefinition {
    CardDefinition {
        name: "Mahamoti Djinn",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Djinn], ..Default::default() },
        power: 5, toughness: 6,
        keywords: vec![Keyword::Flying],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Prodigal Sorcerer — {2}{U} 1/1, {T}: Deal 1 damage to any target
pub fn prodigal_sorcerer() -> CardDefinition {
    CardDefinition {
        name: "Prodigal Sorcerer",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Wizard], ..Default::default() },
        power: 1, toughness: 1,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effects: vec![SpellEffect::DealDamage { amount: 1, target: SelectionRequirement::Any }],
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Black Knight — {B}{B} 2/2 First Strike
pub fn black_knight() -> CardDefinition {
    CardDefinition {
        name: "Black Knight",
        cost: cost(&[b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Knight], ..Default::default() },
        power: 2, toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Hypnotic Specter — {1}{B}{B} 2/2 Flying
/// Whenever Hypnotic Specter attacks, defending player discards a card at random.
pub fn hypnotic_specter() -> CardDefinition {
    CardDefinition {
        name: "Hypnotic Specter",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Specter], ..Default::default() },
        power: 2, toughness: 2,
        keywords: vec![Keyword::Flying],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                condition: TriggerCondition::Attacks,
                effects: vec![SpellEffect::OpponentDiscardRandom],
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Sengir Vampire — {3}{B}{B} 4/4 Flying
pub fn sengir_vampire() -> CardDefinition {
    CardDefinition {
        name: "Sengir Vampire",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 4, toughness: 4,
        keywords: vec![Keyword::Flying],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Shivan Dragon — {4}{R}{R} 5/5 Flying
pub fn shivan_dragon() -> CardDefinition {
    CardDefinition {
        name: "Shivan Dragon",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 5, toughness: 5,
        keywords: vec![Keyword::Flying],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Grizzly Bears — {1}{G} 2/2
pub fn grizzly_bears() -> CardDefinition {
    CardDefinition {
        name: "Grizzly Bears",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        power: 2, toughness: 2,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Birds of Paradise — {G} 0/1 Flying, {T}: Add one mana of any color
pub fn birds_of_paradise() -> CardDefinition {
    CardDefinition {
        name: "Birds of Paradise",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 0, toughness: 1,
        keywords: vec![Keyword::Flying],
        spell_effects: vec![],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effects: vec![SpellEffect::AddManaAnyColor { amount: 1 }],
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Llanowar Elves — {G} 1/1, {T}: Add {G}
pub fn llanowar_elves() -> CardDefinition {
    CardDefinition {
        name: "Llanowar Elves",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Druid], ..Default::default() },
        power: 1, toughness: 1,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Green)],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Elvish Archer — {1}{G} 1/2 First Strike
pub fn elvish_archer() -> CardDefinition {
    CardDefinition {
        name: "Elvish Archer",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Archer], ..Default::default() },
        power: 1, toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}

/// Craw Wurm — {4}{G}{G} 6/4
pub fn craw_wurm() -> CardDefinition {
    CardDefinition {
        name: "Craw Wurm",
        cost: cost(&[generic(4), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        power: 6, toughness: 4,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
    }
}
