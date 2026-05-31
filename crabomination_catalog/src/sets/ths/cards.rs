//! Theros (THS) — assorted commons/uncommons used as devotion-shell
//! filler. Simple bodies / ETBs / one instant.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, Keyword,
    SelectionRequirement, Selector, Subtypes, Value,
};
use crate::effect::{PlayerRef, ZoneDest, shortcut::etb, shortcut::target_filtered};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Sedge Scorpion — {G} Creature — Scorpion 1/1. Deathtouch.
pub fn sedge_scorpion() -> CardDefinition {
    CardDefinition {
        name: "Sedge Scorpion",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Scorpion], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Pharika's Chosen — {B} Creature — Snake 1/1. Deathtouch.
pub fn pharikas_chosen() -> CardDefinition {
    CardDefinition {
        name: "Pharika's Chosen",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Yoked Ox — {W} Creature — Ox 0/4.
pub fn yoked_ox() -> CardDefinition {
    CardDefinition {
        name: "Yoked Ox",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ox], ..Default::default() },
        power: 0,
        toughness: 4,
        ..Default::default()
    }
}

/// Two-Headed Cerberus — {2}{R} Creature — Dog 2/2. Double strike.
pub fn two_headed_cerberus() -> CardDefinition {
    CardDefinition {
        name: "Two-Headed Cerberus",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::DoubleStrike],
        ..Default::default()
    }
}

/// Voyaging Satyr — {1}{G} Creature — Satyr Druid 1/2. {T}: Untap target land.
pub fn voyaging_satyr() -> CardDefinition {
    CardDefinition {
        name: "Voyaging Satyr",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(SelectionRequirement::Land), up_to: None },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Leonin Snarecaster — {1}{W} Creature — Cat Soldier 2/1. When it enters,
/// you may tap target creature. (The "may" is taken — collapsed to a tap.)
pub fn leonin_snarecaster() -> CardDefinition {
    CardDefinition {
        name: "Leonin Snarecaster",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Tap {
            what: target_filtered(SelectionRequirement::Creature),
        })],
        ..Default::default()
    }
}

/// Voyage's End — {1}{U} Instant. Return target creature to its owner's
/// hand. Scry 1.
pub fn voyages_end() -> CardDefinition {
    CardDefinition {
        name: "Voyage's End",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Nessian Courser — {2}{G} Creature — Centaur Warrior 3/3.
pub fn nessian_courser() -> CardDefinition {
    CardDefinition {
        name: "Nessian Courser",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        ..Default::default()
    }
}

/// Vulpine Goliath — {5}{G} Creature — Fox 4/4. Trample.
pub fn vulpine_goliath() -> CardDefinition {
    CardDefinition {
        name: "Vulpine Goliath",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fox], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        ..Default::default()
    }
}

/// Felhide Minotaur — {2}{R} Creature — Minotaur 3/2.
pub fn felhide_minotaur() -> CardDefinition {
    CardDefinition {
        name: "Felhide Minotaur",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Minotaur], ..Default::default() },
        power: 3,
        toughness: 2,
        ..Default::default()
    }
}

/// Griptide — {2}{U} Instant. Put target creature on top of its owner's
/// library.
pub fn griptide() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Griptide",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::Top,
            },
        },
        ..Default::default()
    }
}

/// Lash of the Whip — {4}{B} Instant. Target creature gets -4/-4 until end
/// of turn.
pub fn lash_of_the_whip() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Lash of the Whip",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-4),
            toughness: Value::Const(-4),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Pharika's Cure — {1}{B} Instant. Deal 2 damage to target creature and
/// you gain 2 life.
pub fn pharikas_cure() -> CardDefinition {
    CardDefinition {
        name: "Pharika's Cure",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Fade into Antiquity — {2}{G} Sorcery. Exile target enchantment.
pub fn fade_into_antiquity() -> CardDefinition {
    CardDefinition {
        name: "Fade into Antiquity",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Exile { what: target_filtered(SelectionRequirement::Enchantment) },
        ..Default::default()
    }
}
