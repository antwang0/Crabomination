//! An MKM wave: an Investigate combat trick, a sac-for-value Merfolk, a
//! sacrifice-draw Ogre, and a small reanimator. All ride existing primitives.
//! Tests in `crabomination/src/tests/recent155.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R,
    Selector, Subtypes, Value,
};
use crate::effect::shortcut::{etb, investigate, on_you_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::mana::{b, cost, generic, u, w};

/// Auspicious Arrival — {1}{W} Instant. Target creature gets +2/+2 until end of
/// turn, then Investigate.
pub fn auspicious_arrival() -> CardDefinition {
    CardDefinition {
        name: "Auspicious Arrival",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            investigate(1),
        ]),
        ..Default::default()
    }
}

/// Benthic Criminologists — {4}{U} 4/5 Merfolk Wizard. When it enters or attacks,
/// you may sacrifice an artifact; if you do, draw a card.
pub fn benthic_criminologists() -> CardDefinition {
    let may_sac_draw = || Effect::MaySacrifice {
        description: "Sacrifice an artifact to draw a card?".into(),
        filter: R::Artifact,
        count: Value::ONE,
        then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        else_: None,
    };
    CardDefinition {
        name: "Benthic Criminologists",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![etb(may_sac_draw()), on_you_attack(may_sac_draw())],
        ..Default::default()
    }
}

/// Agency Coroner — {4}{B} 3/6 Ogre Cleric. {2}{B}, sacrifice another creature:
/// draw a card. (The extra card for a suspected sacrifice is omitted — the engine
/// has no suspect tracker yet.)
pub fn agency_coroner() -> CardDefinition {
    CardDefinition {
        name: "Agency Coroner",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 6,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Call a Surprise Witness — {1}{W} Sorcery. Return a target creature card with
/// mana value 3 or less from your graveyard to the battlefield with a flying
/// counter. (The added Spirit type is omitted.)
pub fn call_a_surprise_witness() -> CardDefinition {
    CardDefinition {
        name: "Call a Surprise Witness",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    R::Creature.and(R::InGraveyard).and(R::ManaValueAtMost(3)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddKeywordCounter {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}
