//! OTJ gap batch on existing primitives: Pillage the Bog (land-scaled dig +
//! Plot) and Hell to Pay (X-burn with excess → Treasures). Tests in
//! `crabomination/src/tests/recent192.rs`.

use crate::card::{CardDefinition, CardType, SelectionRequirement as R, Selector, Value};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef};
use crate::mana::{b, cost, g, generic, r, x};

/// Pillage the Bog — {B}{G} Sorcery. Look at the top X cards of your library,
/// where X is twice the number of lands you control. Put one into your hand and
/// the rest on the bottom in a random order. Plot {1}{B}{G}.
pub fn pillage_the_bog() -> CardDefinition {
    CardDefinition {
        name: "Pillage the Bog",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Sorcery],
        plot_cost: Some(cost(&[generic(1), b(), g()])),
        effect: Effect::LookPickToHand {
            then_if_picked: None,
            who: PlayerRef::You,
            count: Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Land.and(R::ControlledByYou),
                )))),
            ),
            rest_to_graveyard: false,
            pick_filter: None,
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: false,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
            rest_to_exile: false,
        },
        ..Default::default()
    }
}

/// Hell to Pay — {X}{R} Sorcery. Deals X damage to target creature. Create a
/// number of tapped Treasure tokens equal to the excess damage dealt this way.
pub fn hell_to_pay() -> CardDefinition {
    let mut treasure = crabomination_base::tokens::treasure_token();
    treasure.tapped = true;
    CardDefinition {
        name: "Hell to Pay",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::XFromCost,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ExcessDamageDealtThisResolution,
                definition: treasure,
            },
        ]),
        ..Default::default()
    }
}
