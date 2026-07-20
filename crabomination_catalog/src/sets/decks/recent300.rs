//! Ravnica batch 10: a tap-down Soldier and the classic MV-9 transmute
//! Leviathan. Tests in `recent_b/recent_300`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_filtered, transmute};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::mana::{cost, generic, r, u, w};

/// Thundersong Trumpeter — {R}{W} 2/1 Human Soldier. {T}: Target creature can't
/// attack or block this turn.
pub fn thundersong_trumpeter() -> CardDefinition {
    CardDefinition {
        name: "Thundersong Trumpeter",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::CantAttack,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Grozoth — {6}{U}{U}{U} 9/9 Leviathan with Defender. When it enters, you may
/// search your library for any number of mana-value-9 cards and put them into
/// your hand. {4}: This creature loses defender until end of turn. Transmute
/// {1}{U}{U}.
pub fn grozoth() -> CardDefinition {
    CardDefinition {
        name: "Grozoth",
        cost: cost(&[generic(6), u(), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Leviathan], ..Default::default() },
        power: 9,
        toughness: 9,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::ManaValueExactly(9),
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(20),
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                effect: Effect::LoseKeywordThisTurn { what: Selector::This, keyword: Keyword::Defender },
                ..Default::default()
            },
            transmute(cost(&[generic(1), u(), u()]), 9),
        ],
        ..Default::default()
    }
}
