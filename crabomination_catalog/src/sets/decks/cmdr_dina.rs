//! Commander: the cards the **Witherbloom Pestilence** precon (SOC, Dina,
//! Essence Brewer) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_dina.rs`.
//!
//! Residuals (each also on its card):
//! - **Gorma, the Gullet** — the extra counters reach creatures you *cast*;
//!   a nontoken creature put onto the battlefield another way enters without
//!   them.
//! - **Stensian Sanguinist** — "this combat" is read as "this turn".

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{devour, etb, on_you_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, x, Color, ManaCost};

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

fn sorcery(name: &'static str, mana: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn pest() -> Arc<TokenDefinition> {
    Arc::new(crabomination_base::tokens::stx_pest_token())
}

fn pests(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: pest() }
}

fn bg() -> crate::mana::ManaSymbol {
    hybrid(Color::Black, Color::Green)
}

fn if_gained_life() -> Predicate {
    Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE }
}

fn your_end_step() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
}

/// "This creature becomes prepared" — one `Prepared` counter, never two.
fn become_prepared() -> Effect {
    Effect::AddCounterCapped {
        what: Selector::This,
        kind: CounterType::Prepared,
        amount: Value::ONE,
        cap: Value::ONE,
    }
}

fn attacking_pests() -> Selector {
    Selector::EachPermanent(R::HasCreatureType(CreatureType::Pest).and(R::IsAttacking).and(R::ControlledByYou))
}

/// Dina, Essence Brewer — sacrificing a creature draws a card, once each turn;
/// {2}, {T}, sacrifice another creature: gain its power in life and put that
/// many +1/+1 counters on target creature you control.
pub fn dina_essence_brewer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl).once_per_turn(),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::SacrificedPower },
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::SacrificedPower,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Dina, Essence Brewer",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Dryad, CreatureType::Druid],
            2,
            3,
        )
    }
}

/// Creakwood Liege — your other black creatures and your other green creatures
/// each get +1/+1; at your upkeep you may create a 1/1 Worm.
pub fn creakwood_liege() -> CardDefinition {
    let lord = |c: Color| StaticAbility {
        description: "Other creatures of a color you control get +1/+1.",
        effect: StaticEffect::PumpPT {
            applies_to: Selector::EachPermanent(
                R::Creature.and(R::HasColor(c)).and(R::ControlledByYou).and(R::OtherThanSource),
            ),
            power: 1,
            toughness: 1,
        },
    };
    let worm = TokenDefinition {
        name: "Worm".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Worm], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![lord(Color::Black), lord(Color::Green)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::MayDo {
                description: "Create a 1/1 Worm?".into(),
                body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(worm) }),
            },
        }],
        ..creature("Creakwood Liege", cost(&[generic(1), bg(), bg(), bg()]), vec![CreatureType::Horror], 2, 2)
    }
}

/// Defiling Daemogoth — menace; your creatures' combat damage to a player
/// gains you 1 each; your end step drains each opponent for the life you
/// gained this turn.
pub fn defiling_daemogoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: your_end_step(),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::LifeGainedThisTurn(PlayerRef::You),
                },
            },
        ],
        ..creature("Defiling Daemogoth", cost(&[generic(3), b(), b()]), vec![CreatureType::Demon], 5, 4)
    }
}

/// Eccentric Pestfinder // Turn Stones — trample; each end step after you
/// gained life, it becomes prepared. Turn Stones: a Pest per opponent.
pub fn eccentric_pestfinder() -> CardDefinition {
    let turn_stones = sorcery("Turn Stones", cost(&[b(), g()]), pests(Value::OpponentCount));
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(if_gained_life()),
            effect: become_prepared(),
        }],
        prepare_spell: Some(Arc::new(turn_stones)),
        ..creature(
            "Eccentric Pestfinder",
            cost(&[generic(2), b(), g()]),
            vec![CreatureType::Troll, CreatureType::Druid],
            5,
            5,
        )
    }
}

/// Feral Appetite — attacking Pests get +1/+0 and deathtouch; {1}{G}: exile a
/// card from a graveyard, a Pest if it was a creature card.
pub fn feral_appetite() -> CardDefinition {
    CardDefinition {
        name: "Feral Appetite",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Attacking Pests you control get +1/+0.",
                effect: StaticEffect::PumpPT { applies_to: attacking_pests(), power: 1, toughness: 0 },
            },
            StaticAbility {
                description: "Attacking Pests you control have deathtouch.",
                effect: StaticEffect::GrantKeyword { applies_to: attacking_pests(), keyword: Keyword::Deathtouch },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: target_filtered(R::InGraveyard),
                        filter: R::Creature,
                    },
                    then: Box::new(pests(Value::ONE)),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Exile { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gorma, the Gullet — lifelink; another creature of yours dying grows it;
/// nontoken creatures you cast enter with a +1/+1 counter per creature that
/// died under your control this turn. Residual: cast creatures only.
pub fn gorma_the_gullet() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Nontoken creatures you control enter with an additional +1/+1 counter for each creature that died under your control this turn.",
            effect: StaticEffect::ExtraEtbCountersForCreatureCasts {
                kind: CounterType::PlusOnePlusOne,
                value: Value::ControllerCreaturesDiedThisTurn,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature(
            "Gorma, the Gullet",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Pest, CreatureType::Frog],
            1,
            1,
        )
    }
}

/// Immoral Bargain — as an additional cost, sacrifice X creatures; destroy X
/// target nonland permanents.
pub fn immoral_bargain() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificeAnyNumber { filter: R::Creature }],
        ..sorcery(
            "Immoral Bargain",
            cost(&[generic(1), b(), g()]),
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Nonland,
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                }),
            },
        )
    }
}

/// Merchant of Venom — menace; enters: each player sacrifices a creature; any
/// player sacrificing a permanent grows it.
pub fn merchant_of_venom() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            etb(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::ONE,
                filter: R::Creature,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::AnyPlayer),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature(
            "Merchant of Venom",
            cost(&[generic(3), b()]),
            vec![CreatureType::Cat, CreatureType::Warlock],
            1,
            1,
        )
    }
}

/// Nether Traitor — haste, shadow; another creature going to your graveyard
/// from the battlefield lets you pay {B} to return it from your graveyard.
pub fn nether_traitor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Shadow],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::FromYourGraveyard).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OwnedByYou).and(R::OtherThanSource),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {B} to return Nether Traitor?".into(),
                mana_cost: cost(&[b()]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                else_: None,
            },
        }],
        ..creature("Nether Traitor", cost(&[b(), b()]), vec![CreatureType::Spirit], 1, 1)
    }
}

/// Pest Rescuer — each upkeep, a Pest if you have no Pest token; your life
/// gains are one bigger.
pub fn pest_rescuer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would gain life, you gain that much life plus 1 instead.",
            effect: StaticEffect::LifeGainBonus { target: crate::effect::PlayerStaticTarget::Controller, amount: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer).with_filter(
                Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::IsToken).and(R::HasCreatureType(CreatureType::Pest)).and(R::ControlledByYou),
                )))),
            ),
            effect: pests(Value::ONE),
        }],
        ..creature("Pest Rescuer", cost(&[generic(2), g()]), vec![CreatureType::Dryad, CreatureType::Druid], 2, 2)
    }
}

/// Ribtruss Roaster — devour 1; your end step makes a Pest per +1/+1 counter
/// on it.
pub fn ribtruss_roaster() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(devour(1)),
        triggered_abilities: vec![TriggeredAbility {
            event: your_end_step(),
            effect: pests(Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne }),
        }],
        ..creature("Ribtruss Roaster", cost(&[generic(4), g()]), vec![CreatureType::Troll, CreatureType::Druid], 3, 3)
    }
}

/// Stensian Sanguinist // Exsanguinate — whenever you attack, target creature
/// gains deathtouch, and its combat damage to a player prepares this.
/// Exsanguinate drains each opponent for X. Residual: "this combat" is read
/// as "this turn".
pub fn stensian_sanguinist() -> CardDefinition {
    let exsanguinate = sorcery(
        "Exsanguinate",
        cost(&[x(), b(), b()]),
        Effect::DrainLifeLost {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::XFromCost,
        },
    );
    CardDefinition {
        triggered_abilities: vec![on_you_attack(Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::WhenTargetDealsCombatDamageToPlayerThisTurn { slot: 0, body: Box::new(become_prepared()) },
        ]))],
        prepare_spell: Some(Arc::new(exsanguinate)),
        ..creature(
            "Stensian Sanguinist",
            cost(&[generic(1), b()]),
            vec![CreatureType::Vampire, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Turbulent Fen — Swamp Forest; enters tapped unless your opponents control
/// eight or more lands.
pub fn turbulent_fen() -> CardDefinition {
    CardDefinition {
        name: "Turbulent Fen",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Swamp, LandType::Forest], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless your opponents control eight or more lands.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Land.and(R::ControlledByOpponent)),
                    n: Value::Const(8),
                },
            },
        }],
        activated_abilities: vec![crate::sets::tap_add(Color::Black), crate::sets::tap_add(Color::Green)],
        ..Default::default()
    }
}

/// Witch of the Moors — deathtouch; your end step, if you gained life, each
/// opponent sacrifices a creature and up to one creature card returns from
/// your graveyard to your hand.
pub fn witch_of_the_moors() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: your_end_step().with_filter(if_gained_life()),
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: R::Creature,
                },
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Move {
                        what: target_filtered(R::Creature.from_your_graveyard()),
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                },
            ]),
        }],
        ..creature(
            "Witch of the Moors",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            4,
            4,
        )
    }
}
