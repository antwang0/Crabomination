use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, SelectionRequirement, Subtypes,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::mana::{cost, g, generic, r, u, w};

/// Lyev Skyknight — {1}{W}{U} 3/1 Human Knight with flying. ETB: detain target
/// nonland permanent an opponent controls (CR 701.35).
pub fn lyev_skyknight() -> CardDefinition {
    CardDefinition {
        name: "Lyev Skyknight",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Detain {
            what: target_filtered(
                SelectionRequirement::Permanent
                    .and(SelectionRequirement::Nonland)
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        ..Default::default()
    }
}

/// Ghor-Clan Rampager — {2}{R}{G} 4/4 Trample
pub fn ghor_clan_rampager() -> CardDefinition {
    CardDefinition {
        name: "Ghor-Clan Rampager",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}
