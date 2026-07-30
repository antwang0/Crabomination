//! White removal staples (artifact/enchantment hate, attacker removal, a board
//! wipe, and an exile-and-replace). All reuse existing primitives. Tests in
//! `tests/recent45.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, SelectionRequirement as R, Selector,
    Subtypes, TokenDefinition, Value,
};
use crate::effect::PlayerRef;
use crate::effect::shortcut::target_filtered;
use crate::mana::{cost, generic, w};

/// Fragmentize — {W} Sorcery. Destroy target artifact or enchantment with mana
/// value 4 or less.
pub fn fragmentize() -> CardDefinition {
    CardDefinition {
        name: "Fragmentize",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered((R::Artifact.or(R::Enchantment)).and(R::ManaValueAtMost(4))),
        },
        ..Default::default()
    }
}

/// Erase — {W} Instant. Exile target enchantment.
pub fn erase() -> CardDefinition {
    CardDefinition {
        name: "Erase",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: target_filtered(R::Enchantment),
        },
        ..Default::default()
    }
}

/// Rebuke — {2}{W} Instant. Destroy target attacking creature.
pub fn rebuke() -> CardDefinition {
    CardDefinition {
        name: "Rebuke",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.and(R::IsAttacking)),
        },
        ..Default::default()
    }
}

/// Depopulate — {2}{W}{W} Sorcery. Destroy all nontoken creatures.
pub fn depopulate() -> CardDefinition {
    CardDefinition {
        name: "Depopulate",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.and(R::IsToken.negate())),
        },
        ..Default::default()
    }
}

/// Crib Swap — {2}{W} Instant. Exile target creature; its controller creates a
/// 1/1 colorless Shapeshifter creature token with changeling.
pub fn crib_swap() -> CardDefinition {
    let shapeshifter = TokenDefinition {
        name: "Shapeshifter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        keywords: vec![Keyword::Changeling],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Crib Swap",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: shapeshifter,
            },
            Effect::Exile {
                what: target_filtered(R::Creature),
            },
        ]),
        ..Default::default()
    }
}
