//! Commander: the 2026-10-20 (`01DDeUtc`) one-gap pair closers, second
//! file (`cmdr_01dd_k` is the first). Tests in `tests/recent_b/cmdr_01dd_b.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    StaticAbility, StaticEffect,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{cost, generic};

/// Dollhouse of Horrors — {5} artifact. {1}, {T}, exile a creature card from
/// your graveyard: create a token that's a copy of it, except it's a 0/0
/// Construct artifact in addition to its other types and has "This token
/// gets +1/+1 for each Construct you control"; it gains haste until end of
/// turn. Activate only as a sorcery.
pub fn dollhouse_of_horrors() -> CardDefinition {
    CardDefinition {
        name: "Dollhouse of Horrors",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            mana_cost: cost(&[generic(1)]),
            exile_other_filter: Some((R::Creature.and(R::InYourGraveyard), 1)),
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::CostExiledCards,
                    extra_creature_types: vec![CreatureType::Construct],
                    extra_card_types: vec![CardType::Artifact],
                    override_pt: Some((0, 0)),
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                },
                Effect::AddCopiableStaticAbilities {
                    what: Selector::LastCreatedTokens,
                    statics: vec![StaticAbility {
                        description: "This token gets +1/+1 for each Construct you control.",
                        effect: StaticEffect::PumpSelfByControlledPermanents {
                            filter: R::HasCreatureType(CreatureType::Construct).and(R::ControlledByYou),
                            per_power: 1,
                            per_toughness: 1,
                        },
                    }],
                },
                Effect::GrantKeyword {
                    what: Selector::LastCreatedTokens,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
