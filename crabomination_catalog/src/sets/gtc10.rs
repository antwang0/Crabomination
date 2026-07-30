//! Gatecrash (GTC) wave 10: the five Primordial ETB Avatars, a Bloodrush
//! beater, a spell-count evasion creature, and two combat-payoff rares. The
//! Primordials' "for each opponent" clauses collapse to a single opponent in a
//! two-player game (the multiplayer per-opponent fan-out is the tracked
//! "target each" gap). Tests in `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, LandType, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, extort, target_filtered};
use crate::effect::{Duration, PlayerRef, Predicate, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// "Bloodrush — {cost}, Discard this card: target attacking creature gets
/// +power/+toughness and gains `extra`." (Local twin of gtc6's helper.)
fn bloodrush(
    mana: crate::mana::ManaCost,
    power: i32,
    toughness: i32,
    extra: Vec<Keyword>,
) -> ActivatedAbility {
    let mut body = vec![Effect::PumpPT {
        what: target_filtered(R::Creature.and(R::IsAttacking)),
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }];
    body.extend(extra.into_iter().map(|k| Effect::GrantKeyword {
        what: Selector::Target(0),
        keyword: k,
        duration: Duration::EndOfTurn,
    }));
    ActivatedAbility {
        mana_cost: mana,
        from_hand: true,
        discard_self_cost: true,
        effect: Effect::Seq(body),
        ..Default::default()
    }
}

/// Wrecking Ogre — {4}{R} 3/3 Ogre Warrior. Double strike; Bloodrush — {3}{R}{R}:
/// target attacking creature gets +3/+3 and gains double strike.
pub fn wrecking_ogre() -> CardDefinition {
    CardDefinition {
        name: "Wrecking Ogre",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Ogre, CreatureType::Warrior]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::DoubleStrike],
        activated_abilities: vec![bloodrush(
            cost(&[generic(3), r(), r()]),
            3,
            3,
            vec![Keyword::DoubleStrike],
        )],
        ..Default::default()
    }
}

/// Incursion Specialist — {1}{U} 1/3 Human Wizard. Whenever you cast your second
/// spell each turn, it gets +2/+0 and can't be blocked this turn.
pub fn incursion_specialist() -> CardDefinition {
    CardDefinition {
        name: "Incursion Specialist",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 1,
        toughness: 3,
        triggered_abilities: vec![crate::effect::shortcut::flurry(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Up-to-one-target ETB body, filtered to a single opponent's objects.
fn primordial_etb(filter: R, effect: Effect) -> TriggeredAbility {
    etb(Effect::ApplyToTargets {
        max_targets: 1,
        min_targets: 0,
        filter,
        effect: Box::new(effect),
    })
}

/// Molten Primordial — {5}{R}{R} 6/4 Avatar. Haste; ETB gain control of up to
/// one target creature an opponent controls until end of turn, untap it, it
/// gains haste.
pub fn molten_primordial() -> CardDefinition {
    CardDefinition {
        name: "Molten Primordial",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Avatar]),
        power: 6,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![primordial_etb(
            R::Creature.and(R::ControlledByOpponent),
            Effect::Seq(vec![
                Effect::GainControl {
                    what: Selector::Target(0),
                    to: None,
                    duration: Duration::EndOfTurn,
                },
                Effect::Untap {
                    what: Selector::Target(0),
                    up_to: None,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        )],
        ..Default::default()
    }
}

/// Sepulchral Primordial — {5}{B}{B} 5/4 Avatar. Intimidate; ETB put up to one
/// target creature card from an opponent's graveyard onto the battlefield under
/// your control.
pub fn sepulchral_primordial() -> CardDefinition {
    CardDefinition {
        name: "Sepulchral Primordial",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Avatar]),
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Intimidate],
        triggered_abilities: vec![primordial_etb(
            R::Creature.and(R::InOpponentGraveyard),
            Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        )],
        ..Default::default()
    }
}

/// Luminate Primordial — {5}{W}{W} 4/7 Avatar. Vigilance; ETB exile up to one
/// target creature an opponent controls and that player gains life equal to its
/// power.
pub fn luminate_primordial() -> CardDefinition {
    CardDefinition {
        name: "Luminate Primordial",
        cost: cost(&[generic(5), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Avatar]),
        power: 4,
        toughness: 7,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![primordial_etb(
            R::Creature.and(R::ControlledByOpponent),
            // Gain life (reads live power) before the exile removes it.
            Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                },
                Effect::Exile {
                    what: Selector::Target(0),
                },
            ]),
        )],
        ..Default::default()
    }
}

/// Sylvan Primordial — {5}{G}{G} 6/8 Avatar. Reach; ETB destroy target
/// noncreature permanent an opponent controls, then search your library for a
/// Forest card and put it onto the battlefield tapped.
pub fn sylvan_primordial() -> CardDefinition {
    CardDefinition {
        name: "Sylvan Primordial",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Avatar]),
        power: 6,
        toughness: 8,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    R::Permanent
                        .and(R::Not(Box::new(R::Creature)))
                        .and(R::ControlledByOpponent),
                ),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Forest),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
        ]))],
        ..Default::default()
    }
}

/// Treasury Thrull — {4}{W}{B} 4/4 Thrull. Extort; whenever it deals combat
/// damage to a player, return target artifact, creature, or enchantment card
/// from your graveyard to the battlefield.
pub fn treasury_thrull() -> CardDefinition {
    CardDefinition {
        name: "Treasury Thrull",
        cost: cost(&[generic(4), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Thrull]),
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            extort(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Move {
                    what: target_filtered(
                        R::InYourGraveyard.and(R::Artifact.or(R::Creature).or(R::Enchantment)),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            },
        ],
        ..Default::default()
    }
}

/// Hellkite Tyrant — {4}{R}{R} 6/5 Dragon. Flying, trample; combat damage to a
/// player steals all artifacts that player controls; at your upkeep, if you
/// control twenty or more artifacts, you win the game.
pub fn hellkite_tyrant() -> CardDefinition {
    CardDefinition {
        name: "Hellkite Tyrant",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Dragon]),
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::GainControl {
                    what: Selector::EachPermanent(R::Artifact.and(R::ControlledByOpponent)),
                    to: Some(PlayerRef::You),
                    duration: Duration::Permanent,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::If {
                    cond: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                        n: Value::Const(20),
                    },
                    then: Box::new(Effect::WinGame {
                        who: PlayerRef::You,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}
