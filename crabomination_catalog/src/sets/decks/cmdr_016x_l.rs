//! Commander: 2026-10-19 (`016Xh16H`) — Syrix, Carrier of the Flame's
//! recursive Phoenixes. Tests in `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, TriggeredAbility,
};
use crate::effect::shortcut::etb;
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, r};

fn phoenix(name: &'static str, mana: ManaCost, keywords: Vec<Keyword>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::Phoenix],
            ..Default::default()
        },
        power: p,
        toughness: t,
        keywords,
        ..Default::default()
    }
}

fn to_battlefield(tapped: bool) -> Effect {
    Effect::Move { what: Selector::This, to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped } }
}

fn to_battlefield_attacking() -> Effect {
    Effect::Seq(vec![
        to_battlefield(true),
        Effect::JoinCombatAttackingChosen {
            what: Selector::This,
            cleanup: crate::effect::AttackingTokenCleanup::None,
        },
    ])
}

fn may_pay(description: &str, mana: ManaCost, body: Effect) -> Effect {
    Effect::MayPay { description: description.into(), mana_cost: mana, body: Box::new(body), else_: None }
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

/// Bloodfeather Phoenix — {1}{R} 2/2 flying, can't block. Whenever an
/// instant or sorcery spell you control deals damage to an opponent, you may
/// pay {R} to return it from your graveyard to the battlefield with haste
/// until end of turn.
pub fn bloodfeather_phoenix() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDealtNoncombatDamage, EventScope::FromYourGraveyardSourceDamagedOpponent)
                .dealt_by(instant_or_sorcery()),
            effect: may_pay(
                "Pay {R} to return Bloodfeather Phoenix to the battlefield?",
                cost(&[r()]),
                Effect::Seq(vec![
                    to_battlefield(false),
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
            ),
        }],
        ..phoenix("Bloodfeather Phoenix", cost(&[generic(1), r()]), vec![Keyword::Flying, Keyword::CantBlock], 2, 2)
    }
}

/// Chandra's Phoenix — {1}{R}{R} 2/2 flying, haste. Whenever an opponent is
/// dealt damage by a red instant or sorcery spell or a red planeswalker you
/// control, return it from your graveyard to your hand.
pub fn chandras_phoenix() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDealtNoncombatDamage, EventScope::FromYourGraveyardSourceDamagedOpponent)
                .dealt_by(R::HasColor(Color::Red).and(instant_or_sorcery().or(R::Planeswalker))),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
        }],
        ..phoenix("Chandra's Phoenix", cost(&[generic(1), r(), r()]), vec![Keyword::Flying, Keyword::Haste], 2, 2)
    }
}

/// Kuldotha Phoenix — {2}{R}{R}{R} 4/4 flying, haste. Metalcraft — {4}:
/// return it from your graveyard to the battlefield; only during your upkeep
/// with three or more artifacts.
pub fn kuldotha_phoenix() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            from_graveyard: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::CurrentStepIs(TurnStep::Upkeep),
                Predicate::MetalcraftActive { who: PlayerRef::You },
            ])),
            effect: to_battlefield(false),
            ..Default::default()
        }],
        ..phoenix("Kuldotha Phoenix", cost(&[generic(2), r(), r(), r()]), vec![Keyword::Flying, Keyword::Haste], 4, 4)
    }
}

/// Lightning Phoenix — {2}{R} 2/2 flying, haste, can't block. At the
/// beginning of your end step, if an opponent was dealt 3 or more damage this
/// turn, you may pay {R} to return it from your graveyard to the battlefield.
pub fn lightning_phoenix() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::FromYourGraveyard).with_filter(
                Predicate::ValueAtLeast(Value::MostDamageTakenThisTurn(PlayerRef::EachOpponent), Value::Const(3)),
            ),
            effect: may_pay("Pay {R} to return Lightning Phoenix to the battlefield?", cost(&[r()]), to_battlefield(false)),
        }],
        ..phoenix(
            "Lightning Phoenix",
            cost(&[generic(2), r()]),
            vec![Keyword::Flying, Keyword::Haste, Keyword::CantBlock],
            2,
            2,
        )
    }
}

/// Magma Phoenix — {3}{R}{R} 3/3 flying. Dying, it deals 3 damage to each
/// creature and each player; {3}{R}{R}: return it from your graveyard to
/// your hand.
pub fn magma_phoenix() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(3) },
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(3) },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r(), r()]),
            from_graveyard: true,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..phoenix("Magma Phoenix", cost(&[generic(3), r(), r()]), vec![Keyword::Flying], 3, 3)
    }
}

/// Nemesis Phoenix — {3}{R}{R} 4/2 flying. {2}{R}: return it from your
/// graveyard to the battlefield tapped and attacking; only during the declare
/// attackers step while you're attacking two or more opponents.
pub fn nemesis_phoenix() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            from_graveyard: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::CurrentStepIs(TurnStep::DeclareAttackers),
                Predicate::ValueAtLeast(Value::OpponentsAttackedThisCombat, Value::Const(2)),
            ])),
            effect: to_battlefield_attacking(),
            ..Default::default()
        }],
        ..phoenix("Nemesis Phoenix", cost(&[generic(3), r(), r()]), vec![Keyword::Flying], 4, 2)
    }
}

/// Sunstreak Phoenix — {2}{R}{R} 4/2 flying. Entering with neither day nor
/// night, it becomes day; whenever day and night change, you may pay {1}{R}
/// to return it from your graveyard to the battlefield tapped.
pub fn sunstreak_phoenix() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::Not(Box::new(Predicate::Any(vec![Predicate::IsDay, Predicate::IsNight]))),
                then: Box::new(Effect::BecomeDay),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DayNightChanged, EventScope::FromYourGraveyardAnyPlayer),
                effect: may_pay(
                    "Pay {1}{R} to return Sunstreak Phoenix to the battlefield tapped?",
                    cost(&[generic(1), r()]),
                    to_battlefield(true),
                ),
            },
        ],
        ..phoenix("Sunstreak Phoenix", cost(&[generic(2), r(), r()]), vec![Keyword::Flying], 4, 2)
    }
}

/// Warcry Phoenix — {3}{R} 2/2 flying, haste. Whenever you attack with three
/// or more creatures, you may pay {2}{R} to return it from your graveyard to
/// the battlefield tapped and attacking.
pub fn warcry_phoenix() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::FromYourGraveyard)
                .with_filter(Predicate::AttackedWithCountAtLeast { who: PlayerRef::You, at_least: 3 }),
            effect: may_pay(
                "Pay {2}{R} to return Warcry Phoenix tapped and attacking?",
                cost(&[generic(2), r()]),
                to_battlefield_attacking(),
            ),
        }],
        ..phoenix("Warcry Phoenix", cost(&[generic(3), r()]), vec![Keyword::Flying, Keyword::Haste], 2, 2)
    }
}
