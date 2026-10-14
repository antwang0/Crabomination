//! Commander: two-card gaps of EDHREC's commanders ranked 3501-4500 (the
//! 2026-10-14 scan), tenth file. Tests in `tests/recent_b/cmdr_rank4k_j.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, ExileReturnZone, Keyword, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, Selector};
use crate::mana::{ManaCost, cost, generic, w};

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

// ── Karn, Argent Defender ────────────────────────────────────────────────────

/// Karn, Argent Defender — {2} legendary 1/3 Golem artifact creature.
/// Artifacts and creatures entering the battlefield don't cause abilities to
/// trigger.
pub fn karn_argent_defender() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Artifacts and creatures entering the battlefield don't cause abilities to trigger.",
            effect: StaticEffect::SuppressCreatureEtbTriggers {
                also_dies: false,
                also_artifacts: true,
            },
        }],
        ..legend(
            "Karn, Argent Defender",
            cost(&[generic(2)]),
            vec![CreatureType::Golem],
            1,
            3,
        )
    }
}

/// Ward of Bones — {6} artifact. Each opponent who controls more creatures than
/// you can't cast creature spells; the same for artifacts and enchantments.
/// Each opponent who controls more lands than you can't play lands.
pub fn ward_of_bones() -> CardDefinition {
    CardDefinition {
        name: "Ward of Bones",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Each opponent who controls more creatures than you can't cast creature spells. The same is true for artifacts and enchantments. Each opponent who controls more lands than you can't play lands.",
            effect: StaticEffect::OpponentsWithMoreCantPlayThatType,
        }],
        ..Default::default()
    }
}

// ── Sigrid, God-Favored ──────────────────────────────────────────────────────

/// Sigrid, God-Favored — {1}{W}{W} legendary 2/2 flash, first strike Human
/// Warrior with protection from God creatures. ETB: exile up to one target
/// attacking or blocking creature until Sigrid leaves the battlefield.
pub fn sigrid_god_favored() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flash,
            Keyword::FirstStrike,
            Keyword::ProtectionFromCreatureType(CreatureType::God),
        ],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature.and(R::IsAttacking.or(R::IsBlocking)),
            effect: Box::new(Effect::ExileUntilSourceLeaves {
                what: Selector::Target(0),
                return_to: ExileReturnZone::Battlefield,
            }),
        })],
        ..legend(
            "Sigrid, God-Favored",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Hallowed Moonlight — {1}{W} instant. Until end of turn, if a creature would
/// enter and it wasn't cast, exile it instead. Draw a card.
pub fn hallowed_moonlight() -> CardDefinition {
    CardDefinition {
        name: "Hallowed Moonlight",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ExileUncastCreaturesThisTurn,
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}
