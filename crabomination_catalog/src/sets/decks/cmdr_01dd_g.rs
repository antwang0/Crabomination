//! Commander: the 2026-10-20 (`01DDeUtc`) closers — Sol'Kanar the Tainted's
//! gifts. Tests in `tests/recent_b/cmdr_01dd_a.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, mint_token, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, u};

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn drain(n: i32, who: Selector) -> Effect {
    Effect::Seq(vec![
        Effect::LoseLife { who, amount: Value::Const(n) },
        Effect::GainLife { who: Selector::You, amount: Value::Const(n) },
    ])
}

/// "You discard `n` cards, lose `life` life, and sacrifice `n` creatures."
fn penance(n: i32, life: i32) -> Effect {
    Effect::Seq(vec![
        Effect::Discard { who: Selector::You, amount: Value::Const(n), random: false },
        Effect::LoseLife { who: Selector::You, amount: Value::Const(life) },
        Effect::Sacrifice { who: Selector::You, count: Value::Const(n), filter: R::Creature },
    ])
}

/// Sol'Kanar the Tainted — {2}{U}{B}{R} 5/5 legendary Elemental Demon. At
/// the beginning of your end step, choose one that hasn't been chosen: draw a
/// card; each opponent loses 2 and you gain 2; 3 damage to up to one other
/// target creature or planeswalker; or exile it and return it under an
/// opponent's control (a new object, so its record starts over).
pub fn sol_kanar_the_tainted() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::ChooseUnchosenMode {
                modes: vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    drain(2, Selector::Player(PlayerRef::EachOpponent)),
                    // "Up to one": optional, so it never hits its controller's own.
                    Effect::OptionalTargets {
                        min: 0,
                        body: Box::new(Effect::DealDamage {
                            to: target_filtered(R::Creature.or(R::Planeswalker).and(R::OtherThanSource)),
                            amount: Value::Const(3),
                        }),
                    },
                    Effect::Seq(vec![
                        Effect::ExileAndReturnToOwner { what: Selector::This },
                        Effect::GainControl {
                            what: Selector::This,
                            to: Some(PlayerRef::HostileOpponent),
                            duration: Duration::Permanent,
                        },
                    ]),
                ],
            },
        }],
        ..legend(
            "Sol'Kanar the Tainted",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::Elemental, CreatureType::Demon],
            5,
            5,
        )
    }
}

/// Blim, Comedic Genius — {2}{B}{R} 4/3 legendary Imp, flying. Whenever it
/// deals combat damage to a player, that player gains control of target
/// permanent you control; then each player loses life and discards cards
/// equal to the permanents they control but don't own.
pub fn blim_comedic_genius() -> CardDefinition {
    let not_owned = || Value::PermanentCountControlledByMatching(PlayerRef::You, R::Permanent.and(R::Not(Box::new(R::OwnedByYou))));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(R::Permanent.and(R::ControlledByYou)),
                    to: Some(PlayerRef::TriggerEventPlayer),
                    duration: Duration::Permanent,
                },
                Effect::EachPlayerDoes {
                    who: PlayerRef::EachPlayer,
                    body: Box::new(Effect::Seq(vec![
                        Effect::LoseLife { who: Selector::You, amount: not_owned() },
                        Effect::Discard { who: Selector::You, amount: not_owned(), random: false },
                    ])),
                },
            ]),
        }],
        ..legend("Blim, Comedic Genius", cost(&[generic(2), b(), r()]), vec![CreatureType::Imp], 4, 3)
    }
}

/// Greed's Gambit — {3}{B} enchantment. ETB: draw three, gain 6, three 2/1
/// black flying Bats. Your end step: discard a card, lose 2, sacrifice a
/// creature. When it leaves: discard three, lose 6, sacrifice three creatures.
pub fn greeds_gambit() -> CardDefinition {
    let bat = TokenDefinition {
        name: "Bat".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Greed's Gambit",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(6) },
                mint_token(bat, 3),
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: penance(1, 2),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: penance(3, 6),
            },
        ],
        ..Default::default()
    }
}

/// The Trickster-God's Heist — {2}{U}{B} Saga. I: you may exchange control of
/// two target creatures. II: you may exchange control of two target nonbasic,
/// noncreature permanents that share a card type. III: target player loses 3
/// life and you gain 3.
pub fn the_trickster_gods_heist() -> CardDefinition {
    let swap = |filter: R| Effect::MayDo {
        description: "Exchange control of the two targets?".into(),
        body: Box::new(Effect::ExchangeControl {
            a: Selector::TargetFiltered { slot: 0, filter: filter.clone() },
            b: Selector::TargetFiltered { slot: 1, filter: filter.and(R::SharesCardTypeWithTargetSlot(0)) },
        }),
    };
    let nonbasic_noncreature = R::Permanent.and(R::Not(Box::new(R::Creature))).and(R::Not(Box::new(R::IsBasicLand)));
    CardDefinition {
        name: "The Trickster-God's Heist",
        cost: cost(&[generic(2), u(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, swap(R::Creature)),
            (2, swap(nonbasic_noncreature)),
            (3, drain(3, target_filtered(R::Player))),
        ],
        ..Default::default()
    }
}
