//! Gatecrash (GTC) wave 4: counter-payoff lords, team keyword statics, ETB and
//! death triggers, and removal on existing primitives. Tests in
//! `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, StaticAbility, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, PlayerRef, Predicate, Selector, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}

/// "Creatures you control matching `req`" as a static-ability scope.
fn yours_matching(req: R) -> Selector {
    Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(req))
}

// ── +1/+1-counter payoff lords ──────────────────────────────────────────────

/// Sapphire Drake — {5}{U} 4/4 Drake with flying. Each creature you control with
/// a +1/+1 counter on it has flying.
pub fn sapphire_drake() -> CardDefinition {
    CardDefinition {
        name: "Sapphire Drake",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Drake]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Your +1/+1-countered creatures have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours_matching(R::WithCounter(CounterType::PlusOnePlusOne)),
                keyword: Keyword::Flying,
            },
        }],
        ..Default::default()
    }
}

/// Crowned Ceratok — {3}{G} 4/3 Rhino with trample. Each creature you control
/// with a +1/+1 counter on it has trample.
pub fn crowned_ceratok() -> CardDefinition {
    CardDefinition {
        name: "Crowned Ceratok",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Rhino]),
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Your +1/+1-countered creatures have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours_matching(R::WithCounter(CounterType::PlusOnePlusOne)),
                keyword: Keyword::Trample,
            },
        }],
        ..Default::default()
    }
}

// ── Team keyword statics ────────────────────────────────────────────────────

/// Hellraiser Goblin — {2}{R} 2/2 Goblin Berserker. Creatures you control have
/// haste and attack each combat if able.
pub fn hellraiser_goblin() -> CardDefinition {
    let all = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Hellraiser Goblin",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Goblin, CreatureType::Berserker]),
        power: 2,
        toughness: 2,
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control have haste.",
                effect: StaticEffect::GrantKeyword { applies_to: all(), keyword: Keyword::Haste },
            },
            StaticAbility {
                description: "Creatures you control attack each combat if able.",
                effect: StaticEffect::GrantKeyword { applies_to: all(), keyword: Keyword::MustAttack },
            },
        ],
        ..Default::default()
    }
}

// ── Death / ETB triggers ────────────────────────────────────────────────────

/// Ogre Slumlord — {3}{B}{B} 3/3 Ogre Rogue. Another nontoken creature dies →
/// you may make a 1/1 Rat. Rats you control have deathtouch.
pub fn ogre_slumlord() -> CardDefinition {
    CardDefinition {
        name: "Ogre Slumlord",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Ogre, CreatureType::Rogue]),
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken.and(R::OtherThanSource),
                },
            ),
            effect: Effect::MayDo {
                description: "Create a 1/1 black Rat?".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Rat".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Black],
                        subtypes: creatures(vec![CreatureType::Rat]),
                        ..Default::default()
                    },
                }),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Rats you control have deathtouch.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours_matching(R::HasCreatureType(CreatureType::Rat)),
                keyword: Keyword::Deathtouch,
            },
        }],
        ..Default::default()
    }
}

/// Court Street Denizen — {2}{W} 2/2 Human Soldier. Another white creature you
/// control enters → tap target creature an opponent controls.
pub fn court_street_denizen() -> CardDefinition {
    CardDefinition {
        name: "Court Street Denizen",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Soldier]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasColor(Color::White) },
            ),
            effect: Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
        }],
        ..Default::default()
    }
}

/// Sage's Row Denizen — {2}{U} 2/3 Vedalken Wizard. Another blue creature you
/// control enters → target player mills two cards.
pub fn sages_row_denizen() -> CardDefinition {
    CardDefinition {
        name: "Sage's Row Denizen",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Vedalken, CreatureType::Wizard]),
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasColor(Color::Blue) },
            ),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// High Priest of Penance — {W}{B} 1/1 Human Cleric. Whenever it's dealt damage,
/// you may destroy target nonland permanent.
pub fn high_priest_of_penance() -> CardDefinition {
    CardDefinition {
        name: "High Priest of Penance",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Cleric]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Destroy target nonland permanent?".into(),
                body: Box::new(Effect::Destroy { what: target_filtered(R::Permanent.and(R::Land.negate())) }),
            },
        }],
        ..Default::default()
    }
}

/// Frilled Oculus — {1}{U} 1/3 Homunculus. {1}{G}: +2/+2 until end of turn.
/// Activate only once each turn.
pub fn frilled_oculus() -> CardDefinition {
    CardDefinition {
        name: "Frilled Oculus",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Homunculus]),
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dinrova Horror — {4}{U}{B} 4/4 Horror. ETB: return target permanent to its
/// owner's hand, then that player discards a card.
pub fn dinrova_horror() -> CardDefinition {
    CardDefinition {
        name: "Dinrova Horror",
        cost: cost(&[generic(4), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Horror]),
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Permanent), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            Effect::Discard {
                who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                amount: Value::ONE,
                random: false,
            },
        ]))],
        ..Default::default()
    }
}

// ── Removal ─────────────────────────────────────────────────────────────────

/// Grisly Spectacle — {2}{B}{B} Instant. Destroy target nonartifact creature.
/// Its controller mills cards equal to that creature's power.
pub fn grisly_spectacle() -> CardDefinition {
    CardDefinition {
        name: "Grisly Spectacle",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(target_filtered(
                    R::Creature.and(R::Artifact.negate()),
                )))),
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
            Effect::Destroy { what: Selector::Target(0) },
        ]),
        ..Default::default()
    }
}

/// Crackling Perimeter — {1}{R} Enchantment. Tap an untapped Gate you control:
/// this deals 1 damage to each opponent.
pub fn crackling_perimeter() -> CardDefinition {
    CardDefinition {
        name: "Crackling Perimeter",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::HasLandType(crate::card::LandType::Gate)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
