//! Commander: the 2026-10-21 (`01KPxZ3H`) single-commander closers. Tests in
//! `tests/recent_b/cmdr_01kp_a.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LookExileGrant, PlayerRef, Predicate, Selector, Value};
use crate::mana::{b, cost, r, w, x};

/// Squall, Gunblade Duelist — {R}{W}{B} 3/2 legendary Human Mercenary, first
/// strike. As it enters, choose a number. Whenever one or more creatures attack
/// one of your opponents, if any of them has power or toughness equal to the
/// chosen number, Squall deals damage equal to its power to that player
/// (an intervening-if, CR 603.4 — checked again on resolution).
pub fn squall_gunblade_duelist() -> CardDefinition {
    let hit = Predicate::AttackerOfTriggererPtEqualsSourceChosenNumber;
    CardDefinition {
        name: "Squall, Gunblade Duelist",
        cost: cost(&[r(), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Mercenary], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        as_enters_effect: Some(Effect::ChooseNumberForSource { max: 16, pays_life: false, min: 0, own_creature_pt: true }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::OpponentOfYoursAttacked).with_filter(hit.clone()),
            effect: Effect::If {
                cond: hit,
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Duelist's Flame — {X}{R}{R} instant. Until end of turn, target blocked
/// creature you control gets +X/+0 and gains trample and "whenever this
/// creature deals combat damage to a player, look at that many cards from the
/// top of your library; you may exile a nonland card from among them, put the
/// rest on the bottom in a random order, and cast the exiled card free".
pub fn duelists_flame() -> CardDefinition {
    let rider = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::LookTopExileOneMayPlay {
            count: Value::TriggerEventAmount,
            who: PlayerRef::You,
            grant: LookExileGrant::CastFreeNonland,
        },
    };
    CardDefinition {
        name: "Duelist's Flame",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::IsBlocked)),
                power: Value::XFromCost,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            Effect::GrantTriggeredAbility { what: Selector::Target(0), trigger: Box::new(rider), duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}
