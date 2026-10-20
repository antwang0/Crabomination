//! Commander: the 2026-10-20 (`01DDeUtc`) closers — Fblthp, Impossibly
//! Lost's library exile and Errant, Street Artist's copies. Tests in
//! `tests/recent_b/cmdr_01dd_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, CumulativeUpkeepCost, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate, Selector, Value};
use crate::mana::{cost, generic, u};

/// Thought Lash — {2}{U}{U} enchantment. Cumulative upkeep—exile the top card
/// of your library. When a player doesn't pay it, they exile their library.
/// Exile the top card of your library: prevent the next 1 damage to you this
/// turn.
pub fn thought_lash() -> CardDefinition {
    CardDefinition {
        name: "Thought Lash",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::ExileTop(1))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CumulativeUpkeepUnpaid, EventScope::SelfSource),
            effect: Effect::ExileLibraryExceptBottom { who: PlayerRef::You, keep: Value::ZERO },
        }],
        activated_abilities: vec![ActivatedAbility {
            exile_top_cost: 1,
            effect: Effect::PreventNextDamage { target: Selector::Player(PlayerRef::You), amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mirror of Fate — {5} artifact. {T}, sacrifice it: choose up to seven
/// face-up exiled cards you own; exile your library, then put the chosen cards
/// on top of it.
pub fn mirror_of_fate() -> CardDefinition {
    CardDefinition {
        name: "Mirror of Fate",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::ExileLibraryThenTopFromExile { max: 7 },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sea Gate Stormcaller — {1}{U} 2/1 Human Wizard, kicker {4}{U}. ETB: copy
/// the next instant or sorcery spell with mana value 2 or less you cast this
/// turn (twice if kicked); new targets allowed.
pub fn sea_gate_stormcaller() -> CardDefinition {
    let copy = |n: i32| Effect::OnYourNextSpellMatchingThisTurn {
        filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::ManaValueAtMost(2)),
        body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::Const(n) }),
    };
    CardDefinition {
        name: "Sea Gate Stormcaller",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Wizard], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Kicker(cost(&[generic(4), u()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(copy(2)),
            else_: Box::new(copy(1)),
        })],
        ..Default::default()
    }
}

/// Insidious Will — {2}{U}{U} instant. Choose one: counter target spell; or
/// you may choose new targets for target spell; or copy target instant or
/// sorcery spell, new targets allowed.
pub fn insidious_will() -> CardDefinition {
    CardDefinition {
        name: "Insidious Will",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::ChooseNewTargetsForSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::CopySpellMayChooseTargets {
                what: target_filtered(
                    R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                ),
                count: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}
