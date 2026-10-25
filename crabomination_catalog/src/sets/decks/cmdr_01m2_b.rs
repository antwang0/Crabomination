//! Commander: 2026-10-25 (`01M2Ffkh`) — the Living Lightning, Charged Up
//! deck's closers. Tests in `tests/recent_b/cmdr_01m2_b.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_any, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{cost, generic, r};

/// Contested Game Ball — {2} artifact. Whenever you're dealt combat damage,
/// the attacking player gains control of it and untaps it; {2}, {T}: draw a
/// card and put a point counter on it, then at five or more it's sacrificed
/// for a Treasure.
pub fn contested_game_ball() -> CardDefinition {
    let points = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Point };
    CardDefinition {
        name: "Contested Game Ball",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            // CR 506.2 — the attacking player is the active player; CR 603.2c,
            // one trigger however many attackers connect.
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer)
                .with_filter(Predicate::SamePlayer(PlayerRef::TriggerEventPlayer, PlayerRef::You))
                .once_per_batch(),
            effect: Effect::Seq(vec![
                Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::ActivePlayer),
                    duration: crate::effect::Duration::Permanent,
                },
                Effect::Untap { what: Selector::This, up_to: None },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::AddCounter { what: Selector::This, kind: CounterType::Point, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(points(), Value::Const(5)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificePermanent { what: Selector::This },
                        Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: std::sync::Arc::new(crate::game::effects::treasure_token()),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Jester's Cap — {4} artifact. {2}, {T}, sacrifice it: search target
/// player's library for three cards and exile them, then that player
/// shuffles. The activator picks (CR 701.23a).
pub fn jesters_cap() -> CardDefinition {
    let pick = || Effect::SearchPickedBy {
        who: PlayerRef::Target(0),
        picker: PlayerRef::You,
        filter: R::Any,
        to: ZoneDest::Exile,
    };
    CardDefinition {
        name: "Jester's Cap",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(Effect::Seq(vec![pick(), pick(), pick()])),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lightning Visionary — {1}{R} 2/1 Minotaur Shaman, prowess.
pub fn lightning_visionary() -> CardDefinition {
    CardDefinition {
        name: "Lightning Visionary",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Shaman],
            ..Default::default()
        },
        keywords: vec![Keyword::Prowess],
        power: 2,
        toughness: 1,
        ..Default::default()
    }
}

/// Lightning Prowess — {2}{R} Aura. Enchanted creature has haste and "{T}:
/// this creature deals 1 damage to any target."
pub fn lightning_prowess() -> CardDefinition {
    CardDefinition {
        name: "Lightning Prowess",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Haste],
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Lightning Storm — {1}{R}{R} instant. X damage to any target, X = 3 plus
/// its charge counters. 🟡 Its "any player may discard a land card while
/// it's on the stack" ability has no engine home (no ability activates from
/// the stack), so no counter ever lands and it deals 3.
pub fn lightning_storm() -> CardDefinition {
    CardDefinition {
        name: "Lightning Storm",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_any(),
            amount: Value::Sum(vec![
                Value::Const(3),
                Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
            ]),
        },
        ..Default::default()
    }
}

/// Radiating Lightning — {3}{R} instant. 3 damage to target player and 1 to
/// each creature that player controls.
pub fn radiating_lightning() -> CardDefinition {
    CardDefinition {
        name: "Radiating Lightning",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Player), amount: Value::Const(3) },
            Effect::DealDamage {
                to: Selector::ControlledBy {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    filter: R::Creature,
                },
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Riddle of Lightning — {3}{R}{R} instant. Choose any target; scry 3, then
/// reveal the top card of your library and deal damage equal to its mana
/// value to that target.
pub fn riddle_of_lightning() -> CardDefinition {
    CardDefinition {
        name: "Riddle of Lightning",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
            Effect::RevealTopCard { who: PlayerRef::You },
            Effect::DealDamage {
                to: target_any(),
                amount: Value::ManaValueOf(Box::new(Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE })),
            },
        ]),
        ..Default::default()
    }
}

/// Lightning Javelin — {3}{R} sorcery. 3 damage to any target, then scry 1.
pub fn lightning_javelin() -> CardDefinition {
    CardDefinition {
        name: "Lightning Javelin",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}
