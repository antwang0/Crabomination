//! CR 407 — the ante cards. Every one carries "Remove this card from your
//! deck before playing if you're not playing for ante", modelled as
//! `CardDefinition.ante_only`, so deck legality rejects them outside an ante
//! game. Tests in `core_rules/cr_recent67`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes,
};
use crate::effect::{
    Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::{draw, target_filtered, you},
};
use crate::mana::{ManaCost, b, cost, g, generic, r};

fn ante_sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ante_only: true,
        ..Default::default()
    }
}

/// Contract from Below — {B}. Empty your hand into a fresh seven.
pub fn contract_from_below() -> CardDefinition {
    ante_sorcery(
        "Contract from Below",
        cost(&[b()]),
        Effect::Seq(vec![
            Effect::Discard {
                who: you(),
                amount: Value::HandSizeOf(PlayerRef::You),
                random: false,
            },
            Effect::AnteTopOfLibrary {
                who: PlayerRef::You,
                optional: false,
                then: None,
                else_: None,
            },
            draw(7),
        ]),
    )
}

/// Demonic Attorney — {1}{B}{B}. Everyone pays up.
pub fn demonic_attorney() -> CardDefinition {
    ante_sorcery(
        "Demonic Attorney",
        cost(&[generic(1), b(), b()]),
        Effect::AnteTopOfLibrary {
            who: PlayerRef::EachPlayer,
            optional: false,
            then: None,
            else_: None,
        },
    )
}

/// Rebirth — {3}{G}{G}{G}. Buy your way back to twenty.
pub fn rebirth() -> CardDefinition {
    ante_sorcery(
        "Rebirth",
        cost(&[generic(3), g(), g(), g()]),
        Effect::AnteTopOfLibrary {
            who: PlayerRef::EachPlayer,
            optional: true,
            then: Some(Box::new(Effect::SetLifeTotal {
                who: you(),
                amount: Value::Const(20),
            })),
            else_: None,
        },
    )
}

/// Darkpact — {B}{B}{B}. Take a card out of the ante and pay for it off the
/// top of your library.
pub fn darkpact() -> CardDefinition {
    ante_sorcery(
        "Darkpact",
        cost(&[b(), b(), b()]),
        Effect::TakeAnteCardForLibraryTop { who: PlayerRef::You },
    )
}

/// Jeweled Bird — {1}. Trade the whole ante for one card.
pub fn jeweled_bird() -> CardDefinition {
    CardDefinition {
        name: "Jeweled Bird",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        ante_only: true,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Ante { what: Selector::This },
                Effect::AnteToGraveyard { who: PlayerRef::You },
                draw(1),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Amulet of Quoz — {6}. Ante up or flip for the game.
pub fn amulet_of_quoz() -> CardDefinition {
    CardDefinition {
        name: "Amulet of Quoz",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        ante_only: true,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            condition: Some(Predicate::CurrentStepIs(crate::game::types::TurnStep::Upkeep)),
            effect: Effect::AnteTopOfLibrary {
                who: PlayerRef::Target(0),
                optional: true,
                then: None,
                else_: Some(Box::new(Effect::FlipCoin {
                    count: Value::ONE,
                    on_heads: Box::new(Effect::LoseGame { who: PlayerRef::Target(0) }),
                    on_tails: Box::new(Effect::LoseGame { who: PlayerRef::You }),
                })),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tempest Efreet — {1}{R}{R}{R} 3/3. Ten life, or a card changes hands for
/// good.
pub fn tempest_efreet() -> CardDefinition {
    CardDefinition {
        name: "Tempest Efreet",
        cost: cost(&[generic(1), r(), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Efreet], ..Default::default() },
        power: 3,
        toughness: 3,
        ante_only: true,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::PlayerMayPayLifeElse {
                who: PlayerRef::Target(0),
                life: Value::Const(10),
                else_: Box::new(Effect::Seq(vec![
                    Effect::RevealRandomFromHand { who: Selector::Player(PlayerRef::Target(0)) },
                    Effect::ExchangeOwnership {
                        a: Selector::LastRevealedCard,
                        b: Selector::This,
                        a_to: ZoneDest::Hand(PlayerRef::You),
                        b_to: ZoneDest::Graveyard,
                    },
                ])),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Timmerian Fiends — {1}{B}{B} 1/1. Their artifact or the top of their deck.
pub fn timmerian_fiends() -> CardDefinition {
    CardDefinition {
        name: "Timmerian Fiends",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horror], ..Default::default() },
        power: 1,
        toughness: 1,
        ante_only: true,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[b(), b(), b()]),
            effect: Effect::AnteTopOfLibrary {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                optional: true,
                then: None,
                else_: Some(Box::new(Effect::ExchangeOwnership {
                    a: target_filtered(R::Artifact),
                    b: Selector::This,
                    a_to: ZoneDest::Graveyard,
                    b_to: ZoneDest::Graveyard,
                })),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bronze Tablet — {6}. Ten life, or they keep the Tablet and you keep their
/// permanent. (The printed self-exile is folded into the sacrifice: the Tablet
/// leaves the battlefield either way.)
pub fn bronze_tablet() -> CardDefinition {
    CardDefinition {
        name: "Bronze Tablet",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        ante_only: true,
        static_abilities: vec![StaticAbility {
            description: "This artifact enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(4)]),
            effect: Effect::PlayerMayPayLifeElse {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                life: Value::Const(10),
                else_: Box::new(Effect::ExchangeOwnership {
                    a: target_filtered(R::Permanent.and(R::NotToken).and(R::ControlledByOpponent)),
                    b: Selector::This,
                    a_to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    b_to: ZoneDest::Graveyard,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
