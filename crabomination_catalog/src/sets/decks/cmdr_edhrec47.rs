//! Commander: the 2026-10-13 rescan's gaps for Rohgahh, Kher Keep Overlord;
//! Experiment Kraj; and Abigale, Eloquent First-Year. Tests in
//! `tests/recent_b/cmdr_edhrec47.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{battle_cry, heroic, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color, ManaCost};
use crabomination_base::turn_step::TurnStep;
use std::sync::Arc;

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

fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl), effect }
}

/// The 0/1 red Kobold token named Kobolds of Kher Keep.
fn kobolds_of_kher_keep() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "Kobolds of Kher Keep".into(),
            power: 0,
            toughness: 1,
            colors: vec![Color::Red],
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Kobold], ..Default::default() },
            ..Default::default()
        }),
    }
}

fn cast_of(kind: CreatureType) -> EventSpec {
    EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
        .with_filter(Predicate::CastSpellMatches(R::HasCreatureType(kind)))
}

/// "{G}: this gains `kw` until end of turn."
fn pay_for(mana: ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility { mana_cost: mana, effect, ..Default::default() }
}

// ── Rohgahh, Kher Keep Overlord ──────────────────────────────────────────────

/// Rohgahh, Kher Keep Overlord — {3}{B}{R} legendary 4/4 Kobold Warrior. Other
/// Kobolds you control get +2/+2. Casting a Kobold spell: you may pay {2} for
/// a 4/4 red flying Dragon. Casting a Dragon spell: a Kobolds of Kher Keep.
pub fn rohgahh_kher_keep_overlord() -> CardDefinition {
    let dragon = TokenDefinition {
        name: "Dragon".into(),
        power: 4,
        toughness: 4,
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Kobolds you control get +2/+2.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Kobold).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 2,
                toughness: 2,
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: cast_of(CreatureType::Kobold),
                effect: Effect::MayPay {
                    description: "Pay {2} for a 4/4 flying Dragon?".into(),
                    mana_cost: cost(&[generic(2)]),
                    body: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(dragon),
                    }),
                    else_: None,
                },
            },
            TriggeredAbility { event: cast_of(CreatureType::Dragon), effect: kobolds_of_kher_keep() },
        ],
        ..legend(
            "Rohgahh, Kher Keep Overlord",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Kobold, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Rosnakht, Heir of Rohgahh — {R} legendary 0/1 Kobold Warrior. Battle cry.
/// Heroic — a Kobolds of Kher Keep.
pub fn rosnakht_heir_of_rohgahh() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![battle_cry(1), heroic(kobolds_of_kher_keep())],
        ..legend("Rosnakht, Heir of Rohgahh", cost(&[r()]), vec![CreatureType::Kobold, CreatureType::Warrior], 0, 1)
    }
}

/// Venomous Changeling — {2}{B} 1/3 Shapeshifter, changeling, deathtouch.
pub fn venomous_changeling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling, Keyword::Deathtouch],
        ..creature("Venomous Changeling", cost(&[generic(2), b()]), vec![CreatureType::Shapeshifter], 1, 3)
    }
}

// ── Experiment Kraj ──────────────────────────────────────────────────────────

/// Gilder Bairn — {1}{G/U}{G/U} 1/3 Ouphe. {2}{G/U}, {Q}: double each kind of
/// counter on target permanent.
pub fn gilder_bairn() -> CardDefinition {
    let gu = || hybrid(Color::Green, Color::Blue);
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), gu()]),
            untap_self_cost: true,
            effect: Effect::DoubleAllCountersOn { what: target_filtered(R::Permanent) },
            ..Default::default()
        }],
        ..creature("Gilder Bairn", cost(&[generic(1), gu(), gu()]), vec![CreatureType::Ouphe], 1, 3)
    }
}

/// Knacksaw Clique — {3}{U} 1/4 flying Faerie Rogue. {1}{U}, {Q}: target
/// opponent exiles their top card; you may play it this turn.
pub fn knacksaw_clique() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            untap_self_cost: true,
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::Target(0),
                    count: Value::ONE,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                }),
            },
            ..Default::default()
        }],
        ..creature("Knacksaw Clique", cost(&[generic(3), u()]), vec![CreatureType::Faerie, CreatureType::Rogue], 1, 4)
    }
}

/// Thornling — {3}{G}{G} 4/4 Elemental Shapeshifter. {G}: haste, trample or
/// indestructible this turn; {1}: +1/-1 or -1/+1 this turn.
pub fn thornling() -> CardDefinition {
    let grant = |kw: Keyword| {
        pay_for(
            cost(&[g()]),
            Effect::GrantKeyword { what: Selector::This, keyword: kw, duration: Duration::EndOfTurn },
        )
    };
    let shift = |p: i32, t: i32| {
        pay_for(
            cost(&[generic(1)]),
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(p),
                toughness: Value::Const(t),
                duration: Duration::EndOfTurn,
            },
        )
    };
    CardDefinition {
        activated_abilities: vec![
            grant(Keyword::Haste),
            grant(Keyword::Trample),
            grant(Keyword::Indestructible),
            shift(1, -1),
            shift(-1, 1),
        ],
        ..creature(
            "Thornling",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Elemental, CreatureType::Shapeshifter],
            4,
            4,
        )
    }
}

// ── Abigale, Eloquent First-Year ─────────────────────────────────────────────

/// Egon, God of Death // Throne of Death — {2}{B} legendary 6/6 God,
/// deathtouch; upkeep: exile two cards from your graveyard, or if you can't,
/// sacrifice Egon and draw. Throne ({B} legendary artifact): upkeep mill one;
/// {2}{B}, {T}, exile a creature card from your graveyard: draw.
pub fn egon_god_of_death() -> CardDefinition {
    let throne = CardDefinition {
        name: "Throne of Death",
        cost: cost(&[b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![your_upkeep(Effect::Mill { who: Selector::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            exile_other_filter: Some((R::Creature, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![your_upkeep(Effect::If {
            cond: Predicate::ValueAtLeast(Value::GraveyardSizeOf(PlayerRef::You), Value::Const(2)),
            then: Box::new(Effect::ExileFromGraveyard { who: PlayerRef::You, count: Value::Const(2), filter: R::Any }),
            else_: Box::new(Effect::Seq(vec![
                Effect::SacrificeSource,
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ])),
        })],
        back_face: Some(Box::new(throne)),
        ..legend("Egon, God of Death", cost(&[generic(2), b()]), vec![CreatureType::God], 6, 6)
    }
}

/// Waning Wurm — {3}{B} 7/6 Zombie Wurm, vanishing 2.
pub fn waning_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vanishing(2)],
        ..creature("Waning Wurm", cost(&[generic(3), b()]), vec![CreatureType::Zombie, CreatureType::Wurm], 7, 6)
    }
}

/// Woolly Razorback — {2}{W}{W} 7/7 Boar Beast; enters with three ice
/// counters. With one: defender, and it deals no combat damage. Blocking
/// removes one.
pub fn woolly_razorback() -> CardDefinition {
    let iced = |keyword: Keyword, description: &'static str| StaticAbility {
        description,
        effect: StaticEffect::WhileCountersAtLeast {
            kind: CounterType::Ice,
            n: 1,
            inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword }),
        },
    };
    CardDefinition {
        enters_with_counters: Some((CounterType::Ice, Value::Const(3))),
        static_abilities: vec![
            iced(Keyword::Defender, "As long as this creature has an ice counter on it, it has defender."),
            iced(
                Keyword::DealsNoCombatDamage,
                "As long as this creature has an ice counter on it, prevent all combat damage it would deal.",
            ),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::RemoveCounter { what: Selector::This, kind: CounterType::Ice, amount: Value::ONE },
        }],
        ..creature("Woolly Razorback", cost(&[generic(2), w(), w()]), vec![CreatureType::Boar, CreatureType::Beast], 7, 7)
    }
}
