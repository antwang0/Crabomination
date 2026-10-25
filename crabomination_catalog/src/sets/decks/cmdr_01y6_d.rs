//! Commander: 2026-10-25 (`01Y6ZRFx`) — the Molly Hayes, Runaway deck's
//! closers, old-border red and artifacts. Tests in
//! `tests/recent_b/cmdr_01y6_d.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, Subtypes, Supertype,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{target_any, target_filtered};
use crate::effect::{Duration, Effect, Selector, StaticEffect};
use crate::game::types::TurnStep;
use crate::mana::{Color, cost, generic, r};

fn creature(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// Trip Noose — {2} artifact. {2}, {T}: tap target creature.
pub fn trip_noose() -> CardDefinition {
    CardDefinition {
        name: "Trip Noose",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mage-Ring Responder — {7} 7/7 Golem artifact creature. Doesn't untap
/// during your untap step; {7}: untap it. Whenever it attacks, it deals 7
/// damage to target creature defending player controls.
pub fn mage_ring_responder() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7)]),
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                amount: Value::Const(7),
            },
        }],
        ..creature("Mage-Ring Responder", cost(&[generic(7)]), vec![CreatureType::Golem], 7, 7)
    }
}

/// Ruinous Gremlin — {R} 1/1 Gremlin. {2}{R}, sacrifice it: destroy target
/// artifact.
pub fn ruinous_gremlin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Artifact) },
            ..Default::default()
        }],
        ..creature("Ruinous Gremlin", cost(&[r()]), vec![CreatureType::Gremlin], 1, 1)
    }
}

/// Squawkroaster — {3}{R} */4 Elemental, double strike. Vivid — its power is
/// the number of colors among permanents you control (CR 604.3).
pub fn squawkroaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        static_abilities: vec![StaticAbility {
            description: "Vivid — power equal to the number of colors among permanents you control.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(R::ControlledByYou))),
                toughness: Value::Const(4),
            },
        }],
        ..creature("Squawkroaster", cost(&[generic(3), r()]), vec![CreatureType::Elemental], 0, 4)
    }
}

/// Bloodshed Fever — {R} Aura. Enchanted creature attacks each combat if able.
pub fn bloodshed_fever() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::MustAttack], ..Default::default() }),
        ..enchantment("Bloodshed Fever", cost(&[r()]))
    }
}

/// Chaos Moon — {3}{R} enchantment. At the beginning of each upkeep, count
/// the permanents. Odd: until end of turn red creatures get +1/+1 and a
/// Mountain tapped for mana adds an additional {R}. Even: until end of turn
/// red creatures get -1/-1 and a Mountain tapped for mana produces colorless
/// instead.
pub fn chaos_moon() -> CardDefinition {
    let red_creatures = || Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Red)));
    let pump = |n: i32| Effect::PumpPT {
        what: red_creatures(),
        power: Value::Const(n),
        toughness: Value::Const(n),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::ValueIsOdd(Value::CountOf(Box::new(Selector::EachPermanent(R::Permanent)))),
                then: Box::new(Effect::Seq(vec![
                    pump(1),
                    Effect::ExtraManaOnLandTapThisTurn { land: LandType::Mountain, extra: Color::Red },
                ])),
                else_: Box::new(Effect::Seq(vec![
                    pump(-1),
                    Effect::ReplaceLandManaThisTurn {
                        mine_only: false,
                        nonbasic_only: false,
                        color_of_choice: false,
                        land_type: Some(LandType::Mountain),
                    },
                ])),
            },
        }],
        ..enchantment("Chaos Moon", cost(&[generic(3), r()]))
    }
}

/// Melting — {3}{R} enchantment. All lands are no longer snow.
pub fn melting() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All lands are no longer snow.",
            effect: StaticEffect::MatchingLoseSupertype { filter: R::Land, supertype: Supertype::Snow },
        }],
        ..enchantment("Melting", cost(&[generic(3), r()]))
    }
}

/// Total War — {3}{R} enchantment. Whenever a player attacks with one or more
/// creatures, destroy all untapped non-Wall creatures that player controls
/// that didn't attack, except ones they haven't controlled since the turn
/// began (read as "entered this turn").
pub fn total_war() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer),
            effect: Effect::Destroy {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::Untapped)
                        .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Wall))))
                        .and(R::ControlledByActivePlayer)
                        .and(R::Not(Box::new(R::IsAttacking)))
                        .and(R::Not(Box::new(R::EnteredThisTurn))),
                ),
            },
        }],
        ..enchantment("Total War", cost(&[generic(3), r()]))
    }
}

/// Thunderblade Charge — {1}{R}{R} sorcery, 3 damage to any target. Whenever
/// one or more creatures you control deal combat damage to a player, if it's
/// in your graveyard, you may pay {2}{R}{R}{R} to cast it free.
pub fn thunderblade_charge() -> CardDefinition {
    CardDefinition {
        name: "Thunderblade Charge",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::FromYourGraveyard),
            effect: Effect::MayPay {
                description: "Pay {2}{R}{R}{R} to cast Thunderblade Charge from your graveyard?".into(),
                mana_cost: cost(&[generic(2), r(), r(), r()]),
                body: Box::new(Effect::CastWithoutPayingImmediate {
                    what: Selector::This,
                    source_zone: Zone::Graveyard,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}
