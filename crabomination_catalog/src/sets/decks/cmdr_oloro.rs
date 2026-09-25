//! Commander: the cards the **Eternal Bargain** precon (C13, Oloro, Ageless
//! Ascetic — `sets::cmdr`) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_oloro.rs`.
//!
//! Residuals (each also on its card):
//! - **Order of Succession** — the direction and every player's pick are the
//!   engine's (the next player's most valuable creature), not prompts.
//! - **Lim-Dûl's Vault** — how far to dig is the engine's pick, and the final
//!   five keep their order.
//! - **Springjack Pasture** — no bot path picks an X for its Goat sacrifice.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{blocks, etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, ManaSymbol, b, cost, generic, hybrid, u, w};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn gain(n: i32) -> Effect {
    Effect::GainLife { who: Selector::You, amount: Value::Const(n) }
}

fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl), effect }
}

fn artifact_or_enchantment() -> Selector {
    target_filtered(R::Artifact.or(R::Enchantment))
}

/// Act of Authority — enters: you may exile target artifact or enchantment;
/// your upkeep: you may exile one, and if you do its controller gains control
/// of this enchantment.
pub fn act_of_authority() -> CardDefinition {
    CardDefinition {
        name: "Act of Authority",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Exile target artifact or enchantment?".into(),
                body: Box::new(Effect::Exile { what: artifact_or_enchantment() }),
            }),
            your_upkeep(Effect::MayDo {
                description: "Exile target artifact or enchantment (its controller takes this)?".into(),
                // Control passes to the exiled permanent's controller — read
                // while it is still on the battlefield.
                body: Box::new(Effect::Seq(vec![
                    Effect::GainControl {
                        what: Selector::This,
                        to: Some(PlayerRef::ControllerOf(Box::new(artifact_or_enchantment()))),
                        duration: Duration::Permanent,
                    },
                    Effect::Exile { what: artifact_or_enchantment() },
                ])),
            }),
        ],
        ..Default::default()
    }
}

/// Augury Adept — combat damage to a player: the top card goes to your hand
/// and you gain life equal to its mana value.
pub fn augury_adept() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                // Read the mana value before the card moves.
                Effect::GainLife { who: Selector::You, amount: Value::ManaValueOf(Box::new(top())) },
                Effect::Move { what: top(), to: ZoneDest::Hand(PlayerRef::You) },
            ]),
        }],
        ..creature(
            "Augury Adept",
            ManaCost::new(vec![
                ManaSymbol::Generic(1),
                hybrid(Color::White, Color::Blue),
                hybrid(Color::White, Color::Blue),
            ]),
            vec![CreatureType::Kithkin, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Cradle of Vitality — whenever you gain life, you may pay {1}{W} for that
/// many +1/+1 counters on target creature.
pub fn cradle_of_vitality() -> CardDefinition {
    CardDefinition {
        name: "Cradle of Vitality",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1}{W} for a +1/+1 counter per life gained?".into(),
                mana_cost: cost(&[generic(1), w()]),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::TriggerEventAmount,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Disciple of Griselbrand — {1}, sacrifice a creature: gain life equal to
/// its toughness.
pub fn disciple_of_griselbrand() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            sac_other_may_be_source: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::SacrificedToughness },
            ..Default::default()
        }],
        ..creature(
            "Disciple of Griselbrand",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Divinity of Pride — flying, lifelink; +4/+4 while you have 25 or more life.
pub fn divinity_of_pride() -> CardDefinition {
    let wb = || hybrid(Color::White, Color::Black);
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +4/+4 as long as you have 25 or more life.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::PlayerLifeAtLeast { who: PlayerRef::You, life: 25 },
                power: 4,
                toughness: 4,
                keywords: vec![],
            },
        }],
        ..creature(
            "Divinity of Pride",
            ManaCost::new(vec![wb(), wb(), wb(), wb(), wb()]),
            vec![CreatureType::Spirit, CreatureType::Avatar],
            4,
            4,
        )
    }
}

/// Esper Panorama — {T}: {C}; {1}, {T}, sacrifice it: a basic Plains, Island
/// or Swamp onto the battlefield tapped.
pub fn esper_panorama() -> CardDefinition {
    CardDefinition {
        name: "Esper Panorama",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasSupertype(Supertype::Basic).and(
                        R::HasLandType(LandType::Plains)
                            .or(R::HasLandType(LandType::Island))
                            .or(R::HasLandType(LandType::Swamp)),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Famine — 3 damage to each creature and each player.
pub fn famine() -> CardDefinition {
    spell(
        "Famine",
        cost(&[generic(3), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(3) },
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(3) },
        ]),
    )
}

/// Kongming, "Sleeping Dragon" — other creatures you control get +1/+1.
pub fn kongming_sleeping_dragon() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        ..legendary(creature(
            "Kongming, \"Sleeping Dragon\"",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            2,
            2,
        ))
    }
}

/// Lim-Dûl's Vault — dig five at a time for 1 life each, then shuffle under
/// the last five. Residual: the digging is the engine's pick.
pub fn lim_duls_vault() -> CardDefinition {
    spell("Lim-Dûl's Vault", cost(&[u(), b()]), CardType::Instant, Effect::LookTopFiveDigForLife)
}

/// Marrow Bats — flying; pay 4 life: regenerate.
pub fn marrow_bats() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 4,
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Marrow Bats",
            cost(&[generic(4), b()]),
            vec![CreatureType::Bat, CreatureType::Skeleton],
            4,
            1,
        )
    }
}

/// Obelisk of Esper — {T}: {W}, {U} or {B}.
pub fn obelisk_of_esper() -> CardDefinition {
    CardDefinition {
        name: "Obelisk of Esper",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::White, Color::Blue, Color::Black], Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Order of Succession — choose left or right; each player takes a creature
/// of the next player in that direction. Residual: the engine picks.
pub fn order_of_succession() -> CardDefinition {
    spell("Order of Succession", cost(&[generic(3), u()]), CardType::Sorcery, Effect::EachPlayerTakesCreatureOfNext)
}

/// Razor Hippogriff — flying; enters: return target artifact card from your
/// graveyard to hand and gain life equal to its mana value.
pub fn razor_hippogriff() -> CardDefinition {
    let card = || target_filtered(R::Artifact.and(R::InYourGraveyard));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::ManaValueOf(Box::new(Selector::Target(0))) },
            Effect::Move { what: card(), to: ZoneDest::Hand(PlayerRef::You) },
        ]))],
        ..creature("Razor Hippogriff", cost(&[generic(3), w(), w()]), vec![CreatureType::Hippogriff], 3, 3)
    }
}

/// Serene Master — whenever it blocks, exchange its power with the power of
/// target creature it's blocking until end of combat (CR 701.10g).
pub fn serene_master() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![blocks(Effect::ExchangePower {
            a: Selector::This,
            b: target_filtered(R::Creature.and(R::IsAttacking).and(R::BlockingOrBlockedBySource)),
            duration: Duration::EndOfCombat,
        })],
        ..creature("Serene Master", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Monk], 0, 2)
    }
}

/// Springjack Pasture — {T}: {C}; {4}, {T}: a 0/1 Goat; {T}, sacrifice X
/// Goats: X mana of any one color and X life. Residual: no bot path picks X.
pub fn springjack_pasture() -> CardDefinition {
    let goat = Arc::new(TokenDefinition {
        name: "Goat".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goat], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        name: "Springjack Pasture",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: goat },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((R::HasCreatureType(CreatureType::Goat), 0)),
                sac_other_x: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::XFromCost) },
                    Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Survival Cache — gain 2, then draw if you have more life than an opponent.
/// Rebound.
pub fn survival_cache() -> CardDefinition {
    let ahead = Predicate::ForAnyPlayer {
        who: PlayerRef::EachOpponent,
        pred: Box::new(Predicate::ValueAtLeast(
            Value::LifeOf(PlayerRef::You),
            Value::Sum(vec![Value::LifeOf(PlayerRef::Triggerer), Value::ONE]),
        )),
    };
    CardDefinition {
        keywords: vec![Keyword::Rebound],
        ..spell(
            "Survival Cache",
            cost(&[generic(2), w()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                gain(2),
                Effect::If {
                    cond: ahead,
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Tempt with Immortality — tempting offer: return a creature card from your
/// graveyard; each opponent may too, and you return one more per taker.
pub fn tempt_with_immortality() -> CardDefinition {
    spell(
        "Tempt with Immortality",
        cost(&[generic(4), b()]),
        CardType::Sorcery,
        Effect::TemptingOffer {
            body: Box::new(Effect::MoveChosen {
                from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature },
                filter: None,
                count: Value::ONE,
                up_to: false,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        },
    )
}

/// Tidal Force — at the beginning of each upkeep, you may tap or untap target
/// permanent.
pub fn tidal_force() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::MayDo {
                description: "Tap or untap target permanent?".into(),
                body: Box::new(Effect::TapOrUntap { what: target_filtered(R::Permanent) }),
            },
        }],
        ..creature("Tidal Force", cost(&[generic(5), u(), u(), u()]), vec![CreatureType::Elemental], 7, 7)
    }
}

/// Tidehollow Strix — flying, deathtouch.
pub fn tidehollow_strix() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        ..creature("Tidehollow Strix", cost(&[u(), b()]), vec![CreatureType::Bird], 2, 1)
    }
}

/// Tower Gargoyle — flying.
pub fn tower_gargoyle() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        ..creature("Tower Gargoyle", cost(&[generic(1), w(), u(), b()]), vec![CreatureType::Gargoyle], 4, 4)
    }
}
