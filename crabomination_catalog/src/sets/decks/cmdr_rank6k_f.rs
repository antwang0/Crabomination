//! Commander: one- and two-card gaps of EDHREC's commanders ranked 5501-6500
//! (the 2026-10-14 scan), sixth file. Tests in `tests/recent_b/cmdr_rank6k_f.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EntersAsCopy, EventKind,
    EventScope, EventSpec, Keyword, MayPlayDuration, Predicate, SelectionRequirement as R,
    Subtypes, Supertype, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{add_mana, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, cost, g, generic, r, u, w};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

/// "Whenever you gain life, put two +1/+1 counters on this."
fn two_counters_on_life_gain() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
    }
}

/// Starnheim Memento — {3} artifact. {T}: add {W}. {1}{W}, {T}: target
/// creature gets +1/+1 and gains flying until end of turn, sorcery speed.
pub fn starnheim_memento() -> CardDefinition {
    CardDefinition {
        name: "Starnheim Memento",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: add_mana(vec![Color::White]), ..Default::default() },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w()]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Flying,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Trusty Packbeast — {2}{W} 2/3 Beast. ETB: return target artifact card
/// from your graveyard to your hand.
pub fn trusty_packbeast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature("Trusty Packbeast", cost(&[generic(2), w()]), vec![CreatureType::Beast], 2, 3)
    }
}

/// Kirk, Enterprising Captain — {3}{R} 4/3 legendary Human Officer, menace.
/// Whenever it attacks while creatures you control have total power 8 or
/// greater, exile your top card; you may play it this turn.
pub fn kirk_enterprising_captain() -> CardDefinition {
    let mut attack = on_attack(Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::ONE,
        duration: MayPlayDuration::EndOfThisTurn,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost: true,
        uncast_penalty: None,
    });
    attack.event = attack
        .event
        .with_filter(Predicate::ValueAtLeast(Value::TotalPowerControlled, Value::Const(8)));
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![attack],
        ..legend(
            "Kirk, Enterprising Captain",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Officer],
            4,
            3,
        )
    }
}

/// Christine Chapel, Combat Medic — {3}{G} 2/2 legendary Human Doctor. ETB:
/// gain life equal to the number of differently named lands you control.
/// Whenever you gain life, two +1/+1 counters on it.
pub fn christine_chapel_combat_medic() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::GainLife { who: Selector::You, amount: Value::DistinctNamesControlledMatching(R::Land) }),
            two_counters_on_life_gain(),
        ],
        ..legend(
            "Christine Chapel, Combat Medic",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Doctor],
            2,
            2,
        )
    }
}

/// Titanbones, Towering Heart — {3}{G} 4/3 legendary Skeleton Druid, reach.
/// Whenever you gain life, two +1/+1 counters on it. When you discard it,
/// gain 3 life (CR 603.6c look-back trigger from the graveyard).
pub fn titanbones_towering_heart() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![
            two_counters_on_life_gain(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::FromYourGraveyard)
                    .with_filter(Predicate::TriggerSourceIsSelf),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            },
        ],
        ..legend(
            "Titanbones, Towering Heart",
            cost(&[generic(3), g()]),
            vec![CreatureType::Skeleton, CreatureType::Druid],
            4,
            3,
        )
    }
}

/// Trelane, Squire of Gothos — {5}{U} 4/5 legendary Q, vigilance, ward {2},
/// can't be blocked. Residual: the engine has no `Q` creature type, so it
/// has no creature type.
pub fn trelane_squire_of_gothos() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))), Keyword::Unblockable],
        ..legend("Trelane, Squire of Gothos", cost(&[generic(5), u()]), vec![], 4, 5)
    }
}

/// Quicksilver Gargantuan — {5}{U}{U} 7/7 Shapeshifter. You may have it
/// enter as a copy of any creature on the battlefield, except it's 7/7
/// (CR 707.9b).
pub fn quicksilver_gargantuan() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy { filter: R::Creature, override_pt: Some((7, 7)), ..Default::default() }),
        ..creature(
            "Quicksilver Gargantuan",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Shapeshifter],
            7,
            7,
        )
    }
}
