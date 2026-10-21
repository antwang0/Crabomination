//! Commander: the 2026-10-21 (`01KPxZ3H`) single-commander closers. Tests in
//! `tests/recent_b/cmdr_01kp_a.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{target_any, target_filtered};
use crate::effect::{Duration, Effect, LookExileGrant, PlayerRef, Predicate, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, w, x};

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

/// Captain Rex Nebula — {1}{R}{W} 2/2 legendary Human Pilot Employee. At the
/// beginning of combat on your turn, until end of turn target nonland
/// permanent you control becomes a Vehicle artifact (losing its other card
/// types and subtypes, keeping supertypes) with base P/T each its mana value,
/// crew 2 and Crash Land — "whenever this Vehicle deals damage, roll a d6; if
/// the result equals its mana value, sacrifice it, then it deals that much
/// damage to any target".
pub fn captain_rex_nebula() -> CardDefinition {
    let mv = || Value::ManaValueOf(Box::new(Selector::Target(0)));
    let crash_land = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
        // One arm over every result: `LastDieRoll` is the face only inside an
        // arm (CR 706.4), and a modified result may be 0 or above 6.
        effect: Effect::RollDie {
            sides: 6,
            count: Value::ONE,
            modifier: Value::ZERO,
            reroll_at_most: 0,
            ignore_lowest: 0,
            results: vec![(
                0,
                u8::MAX,
                Effect::If {
                    cond: Predicate::ValueEquals(Value::LastDieRoll, Value::ManaValueOf(Box::new(Selector::This))),
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificePermanent { what: Selector::This },
                        Effect::DealDamage { to: target_any(), amount: Value::LastDieRoll },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            )],
            on_doubles: None,
        },
    };
    CardDefinition {
        name: "Captain Rex Nebula",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot, CreatureType::Employee],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::SetCardTypesFor {
                    what: target_filtered(R::Permanent.and(R::ControlledByYou).and(R::Land.negate())),
                    card_types: vec![CardType::Artifact],
                    duration: Duration::EndOfTurn,
                },
                Effect::BecomeArtifactSubtypesFor {
                    what: Selector::Target(0),
                    subtypes: vec![ArtifactSubtype::Vehicle],
                    duration: Duration::EndOfTurn,
                },
                Effect::SetBasePT { what: Selector::Target(0), power: mv(), toughness: mv(), duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Crew(2), duration: Duration::EndOfTurn },
                Effect::GrantTriggeredAbility {
                    what: Selector::Target(0),
                    trigger: Box::new(crash_land),
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Mysterious Limousine — {3}{W}{W} 4/4 artifact Vehicle, crew 2. Whenever it
/// enters or attacks, exile up to one other target creature until it leaves
/// the battlefield (CR 610.3); if a creature is exiled this way, each other
/// card exiled with it returns to the battlefield under its owner's control.
pub fn mysterious_limousine() -> CardDefinition {
    let swap = || {
        Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::ExileUntilSourceLeaves {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    return_to: ExileReturnZone::Battlefield,
                }),
            },
            Effect::ReturnOtherLinkedExiles,
        ])
    };
    CardDefinition {
        name: "Mysterious Limousine",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource), effect: swap() },
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: swap() },
        ],
        ..Default::default()
    }
}
