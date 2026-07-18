//! DSK (Duskmourn) gap batch 2 — Nightmare removal, graveyard recursion, and
//! Survival payoffs. Tests in `tests/recent_b/recent240.rs`.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, CreatureType, ExileReturnZone, Keyword,
    SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, Selector, Value};
use crate::mana::{cost, g, generic, w};

/// Fear of Abduction — {4}{W}{W} Enchantment Creature — Nightmare 5/5. Flying.
/// Additional cost: exile a creature you control. ETB: exile target creature an
/// opponent controls until this leaves; on leave, exiled cards go to hand.
pub fn fear_of_abduction() -> CardDefinition {
    CardDefinition {
        name: "Fear of Abduction",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        additional_cast_cost: vec![AdditionalCastCost::ExilePermanent { filter: R::Creature, count: 1 }],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByOpponent) },
            return_to: ExileReturnZone::Hand,
        })],
        ..Default::default()
    }
}

/// Say Its Name — {1}{G} Sorcery. Mill three, then you may return a creature or
/// land card from your graveyard to your hand. (The three-copy graveyard combo
/// that tutors Altanak is dropped — a niche recursion cost.)
pub fn say_its_name() -> CardDefinition {
    CardDefinition {
        name: "Say Its Name",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::ReturnGraveyardCardsToHand { filter: R::Creature.or(R::Land), max: Value::Const(1) },
        ]),
        ..Default::default()
    }
}
