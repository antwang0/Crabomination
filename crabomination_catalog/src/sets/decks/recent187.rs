//! DSK/FDN/BLB gap batch on existing primitives: Split Up (modal wrath),
//! Strongbox Raider (Raid impulse), and Fireglass Mentor (life-loss-gated
//! second-main impulse). Menagerie Liberator exercises the new Melee keyword
//! (CR 702.121). Tests in `crabomination/src/tests/recent187.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    MayPlayDuration, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Predicate, Selector};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, r, w};

/// Split Up — {1}{W}{W} Sorcery. Choose one — destroy all tapped creatures; or
/// destroy all untapped creatures.
pub fn split_up() -> CardDefinition {
    CardDefinition {
        name: "Split Up",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: Selector::EachPermanent(R::Creature.and(R::Tapped)),
            },
            Effect::Destroy {
                what: Selector::EachPermanent(R::Creature.and(R::Untapped)),
            },
        ]),
        ..Default::default()
    }
}

/// Strongbox Raider — {2}{R}{R} 5/2 Orc Pirate. Raid — when it enters, if you
/// attacked this turn, exile the top two cards of your library; until the end of
/// your next turn you may play them. (Choose-one collapses to a may-play grant on
/// both — a strictly-better impulse approximation.)
pub fn strongbox_raider() -> CardDefinition {
    CardDefinition {
        name: "Strongbox Raider",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Pirate],
            ..Default::default()
        },
        power: 5,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerAttackedThisTurn {
                who: PlayerRef::You,
            },
            then: Box::new(Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Fireglass Mentor — {B}{R} 2/1 Lizard Warlock. At the beginning of your second
/// main phase, if an opponent lost life this turn, exile the top two cards of
/// your library; until end of turn you may play one. (Grant-both impulse
/// approximation.)
pub fn fireglass_mentor() -> CardDefinition {
    CardDefinition {
        name: "Fireglass Mentor",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PostCombatMain),
                EventScope::YourControl,
            )
            .with_filter(Predicate::PlayerLostLifeThisTurn {
                who: PlayerRef::EachOpponent,
            }),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..Default::default()
    }
}

/// Menagerie Liberator — {3}{G} 3/2 Human Warrior with trample and melee (CR
/// 702.121 — +1/+1 until end of turn per opponent it attacked this combat).
pub fn menagerie_liberator() -> CardDefinition {
    CardDefinition {
        name: "Menagerie Liberator",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::Melee],
        ..Default::default()
    }
}
