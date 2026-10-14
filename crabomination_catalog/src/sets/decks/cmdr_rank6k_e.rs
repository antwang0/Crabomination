//! Commander: two-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), fifth file. Tests in `tests/recent_b/cmdr_rank6k_e.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, Value,
};
use crate::effect::shortcut::unearth;
use crate::effect::{Effect, PlayerRef, Selector};
use crate::mana::{cost, g, generic, w, x};
use std::sync::Arc;

/// The Prismatic Piper — {5} 3/3 colorless legendary Shapeshifter, partner.
/// Residual: "if it is your commander, choose a color before the game begins;
/// it is the chosen color" is not modelled — the Piper stays colorless (and so
/// adds nothing to its pair's color identity). See INCOMPLETE_CARDS.md.
pub fn the_prismatic_piper() -> CardDefinition {
    CardDefinition {
        name: "The Prismatic Piper",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Partner],
        ..Default::default()
    }
}

/// Mask of the Jadecrafter — {2} artifact. {X}, {T}, sacrifice it: an X/X
/// colorless Golem artifact creature token, sorcery speed. Unearth {2}{G}.
pub fn mask_of_the_jadecrafter() -> CardDefinition {
    let golem = Arc::new(TokenDefinition {
        name: "Golem".into(),
        power: 0,
        toughness: 0,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        dynamic_pt: Some((Value::XFromCost, Value::XFromCost)),
        ..Default::default()
    });
    CardDefinition {
        name: "Mask of the Jadecrafter",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[x()]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: golem },
                ..Default::default()
            },
            unearth(cost(&[generic(2), g()])),
        ],
        ..Default::default()
    }
}

/// Rebbec, Architect of Ascension — {3}{W} 3/4 legendary Human Artificer,
/// partner. Artifacts you control have protection from each mana value among
/// artifacts you control (CR 702.16b, via `R::ManaValueEqualsOneAmong`).
pub fn rebbec_architect_of_ascension() -> CardDefinition {
    let yours = R::Artifact.and(R::ControlledByYou);
    CardDefinition {
        name: "Rebbec, Architect of Ascension",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Partner],
        static_abilities: vec![StaticAbility {
            description: "Artifacts you control have protection from each mana value among artifacts you control.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours.clone()),
                keyword: Keyword::ProtectionFromMatching(Box::new(R::ManaValueEqualsOneAmong(Box::new(yours)))),
            },
        }],
        ..Default::default()
    }
}
