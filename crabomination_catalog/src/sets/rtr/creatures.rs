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

/// Ghor-Clan Rampager — {2}{R}{G} 4/4 Trample. Bloodrush (CR 702.78) —
/// {R}{G}, Discard this card: Target attacking creature gets +4/+4 and gains
/// trample until end of turn.
pub fn ghor_clan_rampager() -> CardDefinition {
    use crate::card::{ActivatedAbility, Value};
    use crate::effect::Duration;
    CardDefinition {
        name: "Ghor-Clan Rampager",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::IsAttacking)),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: crate::card::Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}
