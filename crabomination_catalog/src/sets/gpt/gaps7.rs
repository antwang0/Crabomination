//! Guildpact (GPT) gap wave 7: the two remaining Nephilim legends plus a
//! spread of simple creatures/enchantments on existing primitives. Tests in
//! `classic_sets/gpt`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_attack, on_dies};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, r, u, w};

/// Yore-Tiller Nephilim — {W}{U}{B}{R} 2/2. Whenever it attacks, return target
/// creature card from your graveyard to the battlefield tapped and attacking.
pub fn yore_tiller_nephilim() -> CardDefinition {
    CardDefinition {
        name: "Yore-Tiller Nephilim",
        cost: cost(&[w(), u(), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nephilim], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::InYourGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::JoinCombatAttacking { what: Selector::LastMoved },
        ]))],
        ..Default::default()
    }
}

/// Witch-Maw Nephilim — {G}{W}{U}{B} 1/1. Whenever you cast a spell, you may
/// put two +1/+1 counters on it. When it attacks, it gains trample until end
/// of turn if its power is 10 or greater.
pub fn witch_maw_nephilim() -> CardDefinition {
    CardDefinition {
        name: "Witch-Maw Nephilim",
        cost: cost(&[g(), w(), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nephilim], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Put two +1/+1 counters on Witch-Maw Nephilim".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    }),
                },
            },
            on_attack(Effect::If {
                cond: Predicate::EntityMatches { what: Selector::This, filter: R::PowerAtLeast(10) },
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            }),
        ],
        ..Default::default()
    }
}

/// Orzhov Pontiff — {1}{W}{B} 1/1 Cleric with haunt. When it enters or the
/// creature it haunts dies, choose one — your creatures get +1/+1, or creatures
/// you don't control get -1/-1, until end of turn.
pub fn orzhov_pontiff() -> CardDefinition {
    let modal = Effect::ChooseMode(vec![
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
    ]);
    CardDefinition {
        name: "Orzhov Pontiff",
        cost: cost(&[crate::mana::generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: modal.clone(),
            },
            on_dies(Effect::HauntCreature { body: Box::new(modal) }),
        ],
        ..Default::default()
    }
}
