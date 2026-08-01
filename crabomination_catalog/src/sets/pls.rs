//! Planeshift (PLS) — 2001. The Invasion block's middle set: Domain, the Kavu,
//! the Lair lands and the Dragon Charms. Tests in `classic_sets/pls`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R,
    Subtypes, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, ZoneDest,
    shortcut::{draw, etb, target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

/// Allied Strategies — {4}{U}. Domain draw for the targeted player.
pub fn allied_strategies() -> CardDefinition {
    sorcery(
        "Allied Strategies",
        cost(&[generic(4), u()]),
        Effect::Draw {
            who: Selector::TargetFiltered { slot: 0, filter: R::Player },
            amount: Value::DomainCount(PlayerRef::Target(0)),
        },
    )
}

/// Alpha Kavu — {2}{G} 2/2 that turns another Kavu into a wall.
pub fn alpha_kavu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::HasCreatureType(CreatureType::Kavu)),
                power: Value::Const(-1),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Alpha Kavu", cost(&[generic(2), g()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Amphibious Kavu — {2}{G} 2/2 that swells against blue and black.
pub fn amphibious_kavu() -> CardDefinition {
    let pump = Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(3),
        toughness: Value::Const(3),
        duration: Duration::EndOfTurn,
    };
    let blue_or_black = Predicate::SelectorExists(Selector::MatchingAmong {
        inner: Box::new(Selector::BlockingCreatures),
        filter: R::HasColor(Color::Blue).or(R::HasColor(Color::Black)),
    });
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource)
                    .with_filter(blue_or_black),
                effect: pump.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource).with_filter(
                    Predicate::SelectorExists(Selector::MatchingAmong {
                        inner: Box::new(Selector::BlockedAttacker),
                        filter: R::HasColor(Color::Blue).or(R::HasColor(Color::Black)),
                    }),
                ),
                effect: pump,
            },
        ],
        ..creature("Amphibious Kavu", cost(&[generic(2), g()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Ancient Spider — {2}{G}{W} 2/5 first strike + reach.
pub fn ancient_spider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Reach],
        ..creature(
            "Ancient Spider",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Spider],
            2,
            5,
        )
    }
}

/// Arctic Merfolk — {1}{U} 1/1. Kicker—Return a creature you control.
pub fn arctic_merfolk() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::ReturnToHand {
            filter: R::Creature,
            count: 1,
        }),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Arctic Merfolk", cost(&[generic(1), u()]), vec![CreatureType::Merfolk], 1, 1)
    }
}

/// Aura Blast — {1}{W}. Enchantment removal that replaces itself.
pub fn aura_blast() -> CardDefinition {
    instant(
        "Aura Blast",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Enchantment) },
            draw(1),
        ]),
    )
}

/// Aurora Griffin — {3}{W} 2/2 flier that whitewashes a permanent.
pub fn aurora_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::BecomeColor {
                what: target_filtered(R::Permanent),
                colors: vec![Color::White],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            ..Default::default()
        }],
        ..creature("Aurora Griffin", cost(&[generic(3), w()]), vec![CreatureType::Griffin], 2, 2)
    }
}

/// Bog Down — {2}{B}. Kicker—Sacrifice two lands for a third card.
pub fn bog_down() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 2,
        }),
        ..sorcery(
            "Bog Down",
            cost(&[generic(2), b()]),
            Effect::Discard {
                who: Selector::TargetFiltered { slot: 0, filter: R::Player },
                amount: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(3)),
                    else_: Box::new(Value::Const(2)),
                },
                random: false,
            },
        )
    }
}

/// Caldera Kavu — {2}{R} 2/2 that pumps off black and recolours off green.
pub fn caldera_kavu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::BecomeChosenColor {
                    what: Selector::This,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature("Caldera Kavu", cost(&[generic(2), r()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Cloud Cover — {2}{W}{U}. Blinks whatever they aim at.
pub fn cloud_cover() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::BecameTarget,
                EventScope::YourPermanentTargetedByOpponent,
            ),
            effect: Effect::MayDo {
                description: "Return that permanent to its owner's hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                }),
            },
        }],
        ..enchantment("Cloud Cover", cost(&[generic(2), w(), u()]))
    }
}

/// Confound — {1}{U}. Counters the removal aimed at your creature.
pub fn confound() -> CardDefinition {
    instant(
        "Confound",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::SpellTargetsCreature)),
            },
            draw(1),
        ]),
    )
}

/// Crosis's Charm — {U}{B}{R}. Bounce, kill, or shatter.
pub fn crosiss_charm() -> CardDefinition {
    instant(
        "Crosis's Charm",
        cost(&[u(), b(), r()]),
        Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::Permanent),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
            },
            Effect::Destroy { what: target_filtered(R::Artifact) },
        ]),
    )
}

/// Darigaaz's Charm — {B}{R}{G}. Regrow, burn, or pump.
pub fn darigaazs_charm() -> CardDefinition {
    instant(
        "Darigaaz's Charm",
        cost(&[b(), r(), g()]),
        Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Daring Leap — {1}{W}{U}. A combat trick with two keywords attached.
pub fn daring_leap() -> CardDefinition {
    instant(
        "Daring Leap",
        cost(&[generic(1), w(), u()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeywords {
                what: target_filtered(R::Creature),
                keywords: vec![Keyword::Flying, Keyword::FirstStrike],
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Dark Suspicions — {2}{B}{B}. Taxes every hand bigger than yours.
pub fn dark_suspicions() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::HandSizeOf(PlayerRef::ActivePlayer)),
                    Box::new(Value::HandSizeOf(PlayerRef::You)),
                ))),
            },
        }],
        ..enchantment("Dark Suspicions", cost(&[generic(2), b(), b()]))
    }
}

/// Deadapult — {2}{R}. Turns Zombies into burn.
pub fn deadapult() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Zombie), 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..enchantment("Deadapult", cost(&[generic(2), r()]))
    }
}

/// Death Bomb — {3}{B}. A creature for a creature, plus two life.
pub fn death_bomb() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..instant(
            "Death Bomb",
            cost(&[generic(3), b()]),
            Effect::Seq(vec![
                Effect::DestroyNoRegen {
                    what: target_filtered(
                        R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
                    ),
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(2),
                },
            ]),
        )
    }
}

/// Destructive Flow — {B}{R}{G}. Nonbasics stop being free.
pub fn destructive_flow() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::ActivePlayer),
                count: Value::ONE,
                filter: R::IsNonbasicLand,
            },
        }],
        ..enchantment("Destructive Flow", cost(&[b(), r(), g()]))
    }
}

/// Disciple of Kangee — {2}{W} 2/2 that lends flight and blue.
pub fn disciple_of_kangee() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::BecomeColor {
                    what: target_filtered(R::Creature),
                    colors: vec![Color::Blue],
                    duration: Duration::EndOfTurn,
                    additive: false,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Disciple of Kangee",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Dominaria's Judgment — {2}{W}. Domain protection for the whole team.
pub fn dominarias_judgment() -> CardDefinition {
    let clause = |land, color| Effect::If {
        cond: Predicate::SelectorExists(Selector::EachPermanent(
            R::HasLandType(land).and(R::ControlledByYou),
        )),
        then: Box::new(Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keyword: Keyword::Protection(color),
            duration: Duration::EndOfTurn,
        }),
        else_: Box::new(Effect::Noop),
    };
    instant(
        "Dominaria's Judgment",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            clause(LandType::Plains, Color::White),
            clause(LandType::Island, Color::Blue),
            clause(LandType::Swamp, Color::Black),
            clause(LandType::Mountain, Color::Red),
            clause(LandType::Forest, Color::Green),
        ]),
    )
}
