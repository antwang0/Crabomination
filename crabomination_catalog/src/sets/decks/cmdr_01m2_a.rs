//! Commander: 2026-10-24 (`01M2Ffkh`) — Isu the Abominable's snow deck. Tests
//! in `tests/recent_b/cmdr_01m2_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, Selector, StaticEffect};
use crate::mana::{ManaCost, cost, g, generic, snow_mana, u, w};

fn snow_creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn aura(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..Default::default()
    }
}

fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn cumulative_snow() -> Keyword {
    Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[snow_mana()])))
}

/// Isu the Abominable — {3}{U}{U} legendary snow 5/5 Yeti. Look at the top
/// card of your library any time; play snow lands and cast snow spells from
/// it. Another snow permanent of yours entering lets you pay {G}, {W} or {U}
/// for a +1/+1 counter (the colour is a mode, then the may-pay).
pub fn isu_the_abominable() -> CardDefinition {
    let counter = || Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE };
    let pay = |pip| Effect::MayPay {
        description: "Pay for a +1/+1 counter on Isu?".into(),
        mana_cost: cost(&[pip]),
        body: Box::new(counter()),
        else_: None,
    };
    let trigger = Effect::ChooseMode(vec![pay(g()), pay(w()), pay(u())]);
    CardDefinition {
        supertypes: vec![Supertype::Legendary, Supertype::Snow],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may play snow lands and cast snow spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::IsSnow },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSnow }),
            effect: trigger,
        }],
        ..snow_creature("Isu the Abominable", cost(&[generic(3), u(), u()]), vec![CreatureType::Yeti], 5, 5)
    }
}

/// Diamond Faerie — {2}{G}{W}{U} snow 3/3 flying Faerie. {1}{S}: snow
/// creatures you control get +1/+1 until end of turn.
pub fn diamond_faerie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), snow_mana()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsSnow).and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..snow_creature("Diamond Faerie", cost(&[generic(2), g(), w(), u()]), vec![CreatureType::Faerie], 3, 3)
    }
}

/// Frostpeak Yeti — {3}{U} snow 3/3 Yeti. {1}{S}: it can't be blocked this turn.
pub fn frostpeak_yeti() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), snow_mana()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..snow_creature("Frostpeak Yeti", cost(&[generic(3), u()]), vec![CreatureType::Yeti], 3, 3)
    }
}

/// Squall Drifter — {1}{W} snow 1/1 flying Elemental. {W}, {T}: tap target creature.
pub fn squall_drifter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w()]),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..snow_creature("Squall Drifter", cost(&[generic(1), w()]), vec![CreatureType::Elemental], 1, 1)
    }
}

/// Snow Devil — {1}{U} Aura. Enchanted creature has flying, and first strike
/// as long as it's blocking and you control a snow land.
pub fn snow_devil() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has first strike as long as it's blocking and you control a snow land.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::All(vec![
                    Predicate::EntityMatches { what: host(), filter: R::IsBlocking },
                    Predicate::SelectorExists(Selector::EachPermanent(
                        R::Land.and(R::IsSnow).and(R::ControlledByYou),
                    )),
                ]),
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: host(), keyword: Keyword::FirstStrike }),
            },
        }],
        ..aura("Snow Devil", cost(&[generic(1), u()]))
    }
}

/// Glacial Plating — {2}{W}{W} snow Aura. Cumulative upkeep {S}; enchanted
/// creature gets +3/+3 for each age counter on it.
pub fn glacial_plating() -> CardDefinition {
    let x = || Value::Times(
        Box::new(Value::Const(3)),
        Box::new(Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Age }),
    );
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        keywords: vec![cumulative_snow()],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +3/+3 for each age counter on this Aura.",
            effect: StaticEffect::PumpPTByValue { applies_to: host(), power: x(), toughness: x() },
        }],
        ..aura("Glacial Plating", cost(&[generic(2), w(), w()]))
    }
}

/// Cover of Winter — {2}{W} snow enchantment. Cumulative upkeep {S}; X of
/// each creature's combat damage to you or your creatures is prevented, X its
/// age counters; {S}: another age counter.
pub fn cover_of_winter() -> CardDefinition {
    CardDefinition {
        name: "Cover of Winter",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Enchantment],
        keywords: vec![cumulative_snow()],
        static_abilities: vec![StaticAbility {
            description: "If a creature would deal combat damage to you and/or one or more creatures you control, prevent X of that damage, where X is the number of age counters on this enchantment.",
            effect: StaticEffect::ReduceCreatureCombatDamageToYouAndYoursByCounters { counter: CounterType::Age },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[snow_mana()]),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Age, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}
