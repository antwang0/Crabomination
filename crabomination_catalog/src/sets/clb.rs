//! Commander Legends: Battle for Baldur's Gate (CLB) — the initiative
//! creatures (CR 726). Tests in `core_rules/cr_recent56`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, MayPlayDuration, Predicate, Subtypes, TriggeredAbility,
};
use crate::effect::{Effect, ManaPayload, PlayerRef, Selector, Value, shortcut::etb};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn etb_take_initiative() -> TriggeredAbility {
    etb(Effect::TakeInitiative { who: PlayerRef::You })
}

/// "If you've completed a dungeon" (CR 309.5).
fn completed_a_dungeon() -> Predicate {
    Predicate::ValueAtLeast(Value::DungeonsCompleted, Value::ONE)
}

/// Aarakocra Sneak — a flier that grabs the crown of the Undercity.
pub fn aarakocra_sneak() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb_take_initiative()],
        ..creature(
            "Aarakocra Sneak",
            cost(&[generic(3), u()]),
            vec![CreatureType::Bird, CreatureType::Rogue],
            1,
            4,
        )
    }
}

/// Passageway Seer — grows each end step you still hold the initiative.
pub fn passageway_seer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb_take_initiative(), TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::SelfSource)
                .with_filter(Predicate::HasInitiative { who: PlayerRef::You }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Passageway Seer",
            cost(&[generic(3), b()]),
            vec![CreatureType::Tiefling, CreatureType::Warlock],
            2,
            2,
        )
    }
}

/// Caves of Chaos Adventurer — impulses on attack, free once you've cleared a
/// dungeon.
pub fn caves_of_chaos_adventurer() -> CardDefinition {
    let impulse = |free: bool| Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::ONE,
        duration: MayPlayDuration::EndOfThisTurn,
        pay_own_cost: !free,
        pay_any_color: false,
        uncast_penalty: None,
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb_take_initiative(), crate::effect::shortcut::on_attack(
            Effect::If {
                cond: completed_a_dungeon(),
                then: Box::new(impulse(true)),
                else_: Box::new(impulse(false)),
            },
        )],
        ..creature(
            "Caves of Chaos Adventurer",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian],
            5,
            3,
        )
    }
}

/// Undermountain Adventurer — {G}{G}, or six once you've cleared a dungeon.
pub fn undermountain_adventurer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb_take_initiative()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::If {
                cond: completed_a_dungeon(),
                then: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Green; 6]),
                }),
                else_: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Green; 2]),
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Undermountain Adventurer",
            cost(&[generic(3), g()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            3,
            4,
        )
    }
}
