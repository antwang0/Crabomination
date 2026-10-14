//! Commander: one-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), seventh file. Tests in `tests/recent_b/cmdr_rank6k_g.rs`.

use crate::card::{
    CardDefinition, CardType, Keyword, LandType, SelectionRequirement as R, StaticAbility,
    StaticEffect, Value,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{cost, generic, r, w};
use std::sync::Arc;

/// Island Sanctuary — {1}{W} enchantment. If you would draw a card during
/// your draw step, you may skip that draw instead; if you do, until your
/// next turn you can't be attacked except by creatures with flying and/or
/// islandwalk (CR 614.1a, `game::draw_skip_shield`).
pub fn island_sanctuary() -> CardDefinition {
    CardDefinition {
        name: "Island Sanctuary",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card during your draw step, instead you may skip that draw. If you do, until your next turn, you can't be attacked except by creatures with flying and/or islandwalk.",
            effect: StaticEffect::ControllerMaySkipDrawStepDrawForAttackShield {
                attackers: R::HasKeyword(Keyword::Flying).or(R::HasKeyword(Keyword::Landwalk(LandType::Island))),
            },
        }],
        ..Default::default()
    }
}

/// You've Been Caught Stealing — {1}{R} sorcery. Choose one: each creature
/// blocks this turn if able; or a Treasure for each opponent who was dealt
/// damage this turn.
pub fn youve_been_caught_stealing() -> CardDefinition {
    CardDefinition {
        name: "You've Been Caught Stealing",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::MustBlock,
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::OpponentsDealtDamageThisTurn,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
        ]),
        ..Default::default()
    }
}
