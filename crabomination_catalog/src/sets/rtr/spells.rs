//! Return to Ravnica (RTR) gap wave 3: burn, pumps, and removal spells on
//! existing primitives (incl. two Overload cards). Tests in `classic_sets/rtr`.

use crate::card::{AlternativeCost, CardDefinition, CardType, Effect, Keyword, Value};
use crate::card::{CounterType, SelectionRequirement as R};
use crate::effect::shortcut::{target_any, target_filtered};
use crate::effect::{Duration, PlayerRef, Selector};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Explosive Impact — {5}{R} Instant. Deals 5 damage to any target.
pub fn explosive_impact() -> CardDefinition {
    CardDefinition {
        name: "Explosive Impact",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage { to: target_any(), amount: Value::Const(5) },
        ..Default::default()
    }
}

/// Annihilating Fire — {1}{R}{R} Instant. 3 damage to any target; if a creature
/// dealt damage this way would die this turn, exile it instead.
pub fn annihilating_fire() -> CardDefinition {
    CardDefinition {
        name: "Annihilating Fire",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
        ]),
        ..Default::default()
    }
}

/// Avenging Arrow — {2}{W} Instant. Destroy target creature that dealt damage
/// this turn.
pub fn avenging_arrow() -> CardDefinition {
    CardDefinition {
        name: "Avenging Arrow",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.and(R::DealtDamageThisTurn)),
        },
        ..Default::default()
    }
}

/// Auger Spree — {1}{B}{R} Instant. Target creature gets +4/-4 until end of turn.
pub fn auger_spree() -> CardDefinition {
    CardDefinition {
        name: "Auger Spree",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(4),
            toughness: Value::Const(-4),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Savage Surge — {1}{G} Instant. Target creature gets +2/+2 and untaps.
pub fn savage_surge() -> CardDefinition {
    CardDefinition {
        name: "Savage Surge",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}

/// Swift Justice — {W} Instant. Target creature gets +1/+0 and gains first
/// strike and lifelink until end of turn.
pub fn swift_justice() -> CardDefinition {
    CardDefinition {
        name: "Swift Justice",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Chorus of Might — {3}{G} Instant. Until end of turn, target creature gets
/// +1/+1 for each creature you control and gains trample.
pub fn chorus_of_might() -> CardDefinition {
    CardDefinition {
        name: "Chorus of Might",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
                toughness: Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Common Bond — {1}{G}{W} Instant. Put a +1/+1 counter on target creature, then
/// put a +1/+1 counter on target creature.
pub fn common_bond() -> CardDefinition {
    CardDefinition {
        name: "Common Bond",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::AddCounter {
                what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Assassin's Strike — {4}{B}{B} Sorcery. Destroy target creature; its
/// controller discards a card.
pub fn assassins_strike() -> CardDefinition {
    CardDefinition {
        name: "Assassin's Strike",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::Discard {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::ONE,
                random: false,
            },
        ]),
        ..Default::default()
    }
}

/// Skull Rend — {3}{B}{R} Sorcery. Deals 2 damage to each opponent; those
/// players each discard two cards at random.
pub fn skull_rend() -> CardDefinition {
    CardDefinition {
        name: "Skull Rend",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
                random: true,
            },
        ]),
        ..Default::default()
    }
}

/// Dynacharge — {R} Instant. Target creature you control gets +2/+0. Overload
/// {2}{R} — each creature you control instead.
pub fn dynacharge() -> CardDefinition {
    CardDefinition {
        name: "Dynacharge",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), r()]),
            effect_override: Some(Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Downsize — {U} Instant. Target creature you don't control gets -4/-0.
/// Overload {2}{U} — each creature you don't control instead.
pub fn downsize() -> CardDefinition {
    CardDefinition {
        name: "Downsize",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(-4),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), u()]),
            effect_override: Some(Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-4),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}
