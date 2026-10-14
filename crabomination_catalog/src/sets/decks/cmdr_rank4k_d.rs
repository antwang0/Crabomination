//! Commander: gap cards of EDHREC's commanders ranked 3501-4500 (the
//! 2026-10-14 scan), fourth file. Tests in `tests/recent_b/cmdr_rank4k_d.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, LoyaltyAbility, ManaPayload, PlayerRef, Selector};
use crate::mana::{Color, ManaCost, cost, generic, r, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

// ── Patron of the Akki ───────────────────────────────────────────────────────

/// Goblin Goliath — {4}{R}{R} 5/4 Goblin Mutant. ETB: a 1/1 red Goblin per
/// opponent you have. {3}{R}, {T}: if a source you control would deal damage
/// to an opponent this turn, it deals double that damage to that player
/// instead (CR 614.1a).
pub fn goblin_goliath() -> CardDefinition {
    let goblin = TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::OpponentCount,
            definition: Arc::new(goblin),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            tap_cost: true,
            effect: Effect::DoubleYourSourcesDamageToOpponentPlayersThisTurn,
            ..Default::default()
        }],
        ..creature(
            "Goblin Goliath",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Goblin, CreatureType::Mutant],
            5,
            4,
        )
    }
}

// ── Sophina, Spearsage Deserter ──────────────────────────────────────────────

/// By Invitation Only — {3}{W}{W} sorcery. Choose a number between 0 and 13.
/// Each player sacrifices that many creatures of their choice.
pub fn by_invitation_only() -> CardDefinition {
    CardDefinition {
        name: "By Invitation Only",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseNumberEachPlayerSacrifices {
            max: 13,
            filter: R::Creature,
        },
        ..Default::default()
    }
}

// ── Myojin of Grim Betrayal ──────────────────────────────────────────────────

/// Dramatist's Puppet — {4} 2/4 Construct artifact creature. ETB: for each
/// kind of counter on target permanent, put another counter of that kind on it
/// or remove one from it.
pub fn dramatists_puppet() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::AddOrRemoveCounterOfEachKindOn {
            what: target_filtered(R::Permanent),
        })],
        ..creature(
            "Dramatist's Puppet",
            cost(&[generic(4)]),
            vec![CreatureType::Construct],
            2,
            4,
        )
    }
}

// ── Ryusei, the Falling Star / Jiwari, the Earth Aflame ──────────────────────

/// Way of the Pyromancer — {1}{R} legendary enchantment. Enters: empower Jace 2.
/// Planeswalkers you control have "[+1]: Add {R}."
pub fn way_of_the_pyromancer() -> CardDefinition {
    CardDefinition {
        name: "Way of the Pyromancer",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::EmpowerJace {
            who: PlayerRef::You,
            count: Value::Const(2),
        })],
        static_abilities: vec![StaticAbility {
            description: "Planeswalkers you control have \"[+1]: Add {R}.\"",
            effect: StaticEffect::PlaneswalkersHaveLoyaltyAbilities {
                abilities: vec![LoyaltyAbility {
                    loyalty_cost: 1,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::OfColor(Color::Red, Value::ONE),
                    },
                    ..Default::default()
                }],
            },
        }],
        ..Default::default()
    }
}

/// Room of Refuge — land, enters tapped; as it enters, choose a color. {T}:
/// one mana of the chosen color. {5}, {T}, sacrifice it: two +1/+1 counters on
/// target creature; sorcery speed.
pub fn room_of_refuge() -> CardDefinition {
    CardDefinition {
        name: "Room of Refuge",
        card_types: vec![CardType::Land],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::ChosenColorOfSource,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
