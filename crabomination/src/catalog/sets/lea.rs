//! Limited Edition Alpha (LEA) — 1993

use super::{no_abilities, tap_add};
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, Keyword,
    SelectionRequirement, SpellEffect, TriggerCondition, TriggeredAbility,
};
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};

// ── Basic Lands ───────────────────────────────────────────────────────────────

pub fn plains() -> CardDefinition {
    CardDefinition {
        name: "Plains",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::White)],
        triggered_abilities: vec![],
    }
}

pub fn island() -> CardDefinition {
    CardDefinition {
        name: "Island",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Blue)],
        triggered_abilities: vec![],
    }
}

pub fn swamp() -> CardDefinition {
    CardDefinition {
        name: "Swamp",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Black)],
        triggered_abilities: vec![],
    }
}

pub fn mountain() -> CardDefinition {
    CardDefinition {
        name: "Mountain",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Red)],
        triggered_abilities: vec![],
    }
}

pub fn forest() -> CardDefinition {
    CardDefinition {
        name: "Forest",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Green)],
        triggered_abilities: vec![],
    }
}

// ── Moxen (Power Nine artifacts) ─────────────────────────────────────────────

/// Mox Pearl — {0} Artifact, {T}: Add {W}
pub fn mox_pearl() -> CardDefinition {
    CardDefinition {
        name: "Mox Pearl",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::White)],
        triggered_abilities: vec![],
    }
}

/// Mox Sapphire — {0} Artifact, {T}: Add {U}
pub fn mox_sapphire() -> CardDefinition {
    CardDefinition {
        name: "Mox Sapphire",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Blue)],
        triggered_abilities: vec![],
    }
}

/// Mox Jet — {0} Artifact, {T}: Add {B}
pub fn mox_jet() -> CardDefinition {
    CardDefinition {
        name: "Mox Jet",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Black)],
        triggered_abilities: vec![],
    }
}

/// Mox Ruby — {0} Artifact, {T}: Add {R}
pub fn mox_ruby() -> CardDefinition {
    CardDefinition {
        name: "Mox Ruby",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Red)],
        triggered_abilities: vec![],
    }
}

/// Mox Emerald — {0} Artifact, {T}: Add {G}
pub fn mox_emerald() -> CardDefinition {
    CardDefinition {
        name: "Mox Emerald",
        cost: ManaCost::default(),
        card_types: vec![CardType::Artifact],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Green)],
        triggered_abilities: vec![],
    }
}

// ── White Creatures ───────────────────────────────────────────────────────────

/// Savannah Lions — {W} 2/1
pub fn savannah_lions() -> CardDefinition {
    CardDefinition {
        name: "Savannah Lions",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        power: 2, toughness: 1,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

/// White Knight — {W}{W} 2/2 First Strike
pub fn white_knight() -> CardDefinition {
    CardDefinition {
        name: "White Knight",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Creature],
        power: 2, toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

/// Serra Angel — {3}{W}{W} 4/4 Flying Vigilance
pub fn serra_angel() -> CardDefinition {
    CardDefinition {
        name: "Serra Angel",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        power: 4, toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

// ── White Spells ──────────────────────────────────────────────────────────────

/// Wrath of God — {2}{W}{W} Sorcery: destroy all creatures
pub fn wrath_of_god() -> CardDefinition {
    CardDefinition {
        name: "Wrath of God",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::DestroyAllCreatures],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

// ── Blue Creatures ────────────────────────────────────────────────────────────

/// Mahamoti Djinn — {3}{U}{U} 5/6 Flying
pub fn mahamoti_djinn() -> CardDefinition {
    CardDefinition {
        name: "Mahamoti Djinn",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        power: 5, toughness: 6,
        keywords: vec![Keyword::Flying],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

/// Prodigal Sorcerer — {2}{U} 1/1, {T}: Deal 1 damage to any target
pub fn prodigal_sorcerer() -> CardDefinition {
    CardDefinition {
        name: "Prodigal Sorcerer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        power: 1, toughness: 1,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effects: vec![SpellEffect::DealDamage { amount: 1, target: SelectionRequirement::Any }],
        }],
        triggered_abilities: vec![],
    }
}

// ── Blue Spells ───────────────────────────────────────────────────────────────

/// Ancestral Recall — {U}: draw 3 cards
pub fn ancestral_recall() -> CardDefinition {
    CardDefinition {
        name: "Ancestral Recall",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::DrawCards { amount: 3 }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

// ── Black Creatures ───────────────────────────────────────────────────────────

/// Black Knight — {B}{B} 2/2 First Strike
pub fn black_knight() -> CardDefinition {
    CardDefinition {
        name: "Black Knight",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        power: 2, toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

/// Hypnotic Specter — {1}{B}{B} 2/2 Flying
/// Whenever Hypnotic Specter attacks, defending player discards a card at random.
pub fn hypnotic_specter() -> CardDefinition {
    CardDefinition {
        name: "Hypnotic Specter",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
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
    }
}

/// Sengir Vampire — {3}{B}{B} 4/4 Flying
pub fn sengir_vampire() -> CardDefinition {
    CardDefinition {
        name: "Sengir Vampire",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        power: 4, toughness: 4,
        keywords: vec![Keyword::Flying],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

// ── Black Spells ──────────────────────────────────────────────────────────────

/// Dark Ritual — {B}: add {B}{B}{B} to your mana pool
pub fn dark_ritual() -> CardDefinition {
    CardDefinition {
        name: "Dark Ritual",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::AddMana {
            colors: vec![Color::Black, Color::Black, Color::Black],
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

/// Terror — {1}{B}: destroy target non-black, non-artifact creature
pub fn terror() -> CardDefinition {
    CardDefinition {
        name: "Terror",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::DestroyCreature {
            target: SelectionRequirement::Creature
                .and(SelectionRequirement::HasColor(Color::Black).not())
                .and(SelectionRequirement::Artifact.not()),
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

// ── Red Creatures ─────────────────────────────────────────────────────────────

/// Shivan Dragon — {4}{R}{R} 5/5 Flying
pub fn shivan_dragon() -> CardDefinition {
    CardDefinition {
        name: "Shivan Dragon",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        power: 5, toughness: 5,
        keywords: vec![Keyword::Flying],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

// ── Red Spells ────────────────────────────────────────────────────────────────

/// Lightning Bolt — {R}: deal 3 damage to any target
pub fn lightning_bolt() -> CardDefinition {
    CardDefinition {
        name: "Lightning Bolt",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::DealDamage {
            amount: 3,
            target: SelectionRequirement::Any,
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

// ── Green Creatures ───────────────────────────────────────────────────────────

/// Grizzly Bears — {1}{G} 2/2
pub fn grizzly_bears() -> CardDefinition {
    CardDefinition {
        name: "Grizzly Bears",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        power: 2, toughness: 2,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

/// Llanowar Elves — {G} 1/1, {T}: Add {G}
pub fn llanowar_elves() -> CardDefinition {
    CardDefinition {
        name: "Llanowar Elves",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        power: 1, toughness: 1,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: vec![tap_add(Color::Green)],
        triggered_abilities: vec![],
    }
}

/// Elvish Archer — {1}{G} 1/2 First Strike
pub fn elvish_archer() -> CardDefinition {
    CardDefinition {
        name: "Elvish Archer",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        power: 1, toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

/// Craw Wurm — {4}{G}{G} 6/4
pub fn craw_wurm() -> CardDefinition {
    CardDefinition {
        name: "Craw Wurm",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        power: 6, toughness: 4,
        keywords: vec![],
        spell_effects: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}

// ── Green Spells ──────────────────────────────────────────────────────────────

/// Giant Growth — {G}: target creature gets +3/+3 until end of turn
pub fn giant_growth() -> CardDefinition {
    CardDefinition {
        name: "Giant Growth",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        power: 0, toughness: 0,
        keywords: vec![],
        spell_effects: vec![SpellEffect::PumpCreature {
            power_bonus: 3,
            toughness_bonus: 3,
        }],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
    }
}
