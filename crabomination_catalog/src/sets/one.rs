//! Phyrexia: All Will Be One — Incubate (CR 701.53). "Incubate N" creates an
//! Incubator double-faced token with N +1/+1 counters; `{2}: Transform` flips
//! it to a 0/0 Phyrexian artifact creature (so it becomes N/N).

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement, StaticAbility,
    StaticEffect, Subtypes,
};
use crate::effect::shortcut::{etb, gain_life, on_dies, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, u, w};

/// Anthem: "Phyrexians you control have `keyword`."
fn phyrexians_have(keyword: Keyword) -> StaticEffect {
    StaticEffect::GrantKeyword {
        applies_to: Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(CreatureType::Phyrexian)
                .and(SelectionRequirement::ControlledByYou),
        ),
        keyword,
    }
}

fn incubate(amount: u32) -> Effect {
    Effect::Incubate { who: PlayerRef::You, amount: Value::Const(amount as i32) }
}

/// Eyes of Gitaxias — {2}{U} Sorcery. Incubate 3. Draw a card.
pub fn eyes_of_gitaxias() -> CardDefinition {
    CardDefinition {
        name: "Eyes of Gitaxias",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            incubate(3),
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Merciless Repurposing — {4}{B}{B} Instant. Exile target creature. Incubate 3.
pub fn merciless_repurposing() -> CardDefinition {
    CardDefinition {
        name: "Merciless Repurposing",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Exile,
            },
            incubate(3),
        ]),
        ..Default::default()
    }
}

/// Phyrexian Awakening — {2}{W} Enchantment. ETB: incubate 4. Phyrexians you
/// control have vigilance.
pub fn phyrexian_awakening() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Awakening",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(incubate(4))],
        static_abilities: vec![StaticAbility {
            description: "Phyrexians you control have vigilance.",
            effect: phyrexians_have(Keyword::Vigilance),
        }],
        ..Default::default()
    }
}

/// Tangled Skyline — {4}{G} Enchantment. ETB: gain 5 life and incubate 5.
/// Phyrexians you control have reach.
pub fn tangled_skyline() -> CardDefinition {
    CardDefinition {
        name: "Tangled Skyline",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Seq(vec![gain_life(5), incubate(5)]))],
        static_abilities: vec![StaticAbility {
            description: "Phyrexians you control have reach.",
            effect: phyrexians_have(Keyword::Reach),
        }],
        ..Default::default()
    }
}

/// Injector Crocodile — {4}{B}{B} Creature — Phyrexian Crocodile 5/5. When it
/// dies, incubate 3. Swampcycling {2}.
pub fn injector_crocodile() -> CardDefinition {
    CardDefinition {
        name: "Injector Crocodile",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Crocodile],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), crate::card::LandType::Swamp)],
        triggered_abilities: vec![on_dies(incubate(3))],
        ..Default::default()
    }
}
