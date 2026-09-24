//! Commander: the cards the **Heavenly Inferno** precon (CMD, Kaalia of the
//! Vast) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_kaalia.rs`.
//!
//! Residuals (each also on its card):
//! - **Archangel of Strife** — war or peace is chosen as its ETB trigger
//!   resolves, not as it enters.
//! - **Kaalia of the Vast** — also triggers attacking an opponent's
//!   planeswalker.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, r, w};
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

/// "{R}: this gets +1/+0 until end of turn."
fn firebreathing() -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[r()]),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::ONE,
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// "When this enters, if you control two or more [land], you may [body]."
fn hedge_etb(land: LandType, body: Effect) -> TriggeredAbility {
    let mut t = etb(Effect::MayDo { description: "Use the hedge-mage trigger?".into(), body: Box::new(body) });
    t.event = t.event.with_filter(Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::HasLandType(land).and(R::ControlledByYou)),
        n: Value::Const(2),
    });
    t
}

fn dragon_5_5() -> TokenDefinition {
    TokenDefinition {
        name: "Dragon".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Akroma, Angel of Fury — uncounterable; flying, trample, protection from
/// white and blue; firebreathing; morph {3}{R}{R}{R}.
pub fn akroma_angel_of_fury() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![
            Keyword::CantBeCountered,
            Keyword::Flying,
            Keyword::Trample,
            Keyword::Protection(Color::White),
            Keyword::Protection(Color::Blue),
            Keyword::Morph(cost(&[generic(3), r(), r(), r()])),
        ],
        activated_abilities: vec![firebreathing()],
        ..creature("Akroma, Angel of Fury", cost(&[generic(5), r(), r(), r()]), vec![CreatureType::Angel], 6, 6)
    })
}

/// Archangel of Strife — flying; each player chooses war (+3/+0 to their
/// creatures) or peace (+0/+3). Residual: chosen as the ETB resolves.
pub fn archangel_of_strife() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::EachPlayerChoosesWarOrPeace)],
        static_abilities: vec![StaticAbility {
            description: "Creatures controlled by players who chose war get +3/+0. \
                          Creatures controlled by players who chose peace get +0/+3.",
            effect: StaticEffect::WarOrPeace,
        }],
        ..creature("Archangel of Strife", cost(&[generic(5), w(), w()]), vec![CreatureType::Angel], 6, 6)
    }
}

/// Armillary Sphere — {2},{T}, sacrifice: up to two basic lands to hand.
pub fn armillary_sphere() -> CardDefinition {
    let fetch = || Effect::Search { who: PlayerRef::You, filter: R::IsBasicLand, to: ZoneDest::Hand(PlayerRef::You) };
    CardDefinition {
        name: "Armillary Sphere",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![fetch(), fetch()]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Avatar of Slaughter — all creatures have double strike and attack each
/// combat if able.
pub fn avatar_of_slaughter() -> CardDefinition {
    let all = || Selector::EachPermanent(R::Creature);
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "All creatures have double strike.",
                effect: StaticEffect::GrantKeyword { applies_to: all(), keyword: Keyword::DoubleStrike },
            },
            StaticAbility {
                description: "All creatures attack each combat if able.",
                effect: StaticEffect::GrantKeyword { applies_to: all(), keyword: Keyword::MustAttack },
            },
        ],
        ..creature("Avatar of Slaughter", cost(&[generic(6), r(), r()]), vec![CreatureType::Avatar], 8, 8)
    }
}

/// Basandra, Battle Seraph — flying; no spells during combat; {R}: target
/// creature attacks this turn if able.
pub fn basandra_battle_seraph() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Players can't cast spells during combat.",
            effect: StaticEffect::PlayersCantCastDuringCombat,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::MustAttack,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Basandra, Battle Seraph", cost(&[generic(3), r(), w()]), vec![CreatureType::Angel], 4, 4)
    })
}

/// Death by Dragons — each player other than target player creates a 5/5
/// flying Dragon.
pub fn death_by_dragons() -> CardDefinition {
    spell(
        "Death by Dragons",
        cost(&[generic(4), r(), r()]),
        CardType::Sorcery,
        Effect::CreateToken {
            who: PlayerRef::EachPlayerExceptControllerOf(Box::new(target_filtered(R::Player))),
            count: Value::ONE,
            definition: Arc::new(dragon_5_5()),
        },
    )
}

/// Dragon Whelp — flying; {R}: +1/+0, and from the fourth activation this
/// turn it's sacrificed at the next end step.
pub fn dragon_whelp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            // Uncapped, but counted: the body reads the uses.
            max_activations_per_turn: Some(u32::MAX),
            effect: Effect::Seq(vec![
                firebreathing().effect,
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::SourceActivationsThisTurn, Value::Const(4)),
                    then: Box::new(Effect::SacrificeAtNextEndStep { what: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..firebreathing()
        }],
        ..creature("Dragon Whelp", cost(&[generic(2), r(), r()]), vec![CreatureType::Dragon], 2, 3)
    }
}

/// Dread Cacodemon — cast from hand: destroy every creature your opponents
/// control, then tap your other creatures.
pub fn dread_cacodemon() -> CardDefinition {
    let mut t = etb(Effect::Seq(vec![
        Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)) },
        Effect::Tap { what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)) },
    ]));
    t.event = t.event.with_filter(Predicate::SourceCastFromOwnersHand);
    CardDefinition {
        triggered_abilities: vec![t],
        ..creature("Dread Cacodemon", cost(&[generic(7), b(), b(), b()]), vec![CreatureType::Demon], 8, 8)
    }
}

/// Duergar Hedge-Mage — two Mountains: may destroy target artifact; two
/// Plains: may destroy target enchantment.
pub fn duergar_hedge_mage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            hedge_etb(LandType::Mountain, Effect::Destroy { what: target_filtered(R::Artifact) }),
            hedge_etb(LandType::Plains, Effect::Destroy { what: target_filtered(R::Enchantment) }),
        ],
        ..creature(
            "Duergar Hedge-Mage",
            cost(&[generic(2), hybrid(Color::Red, Color::White)]),
            vec![CreatureType::Dwarf, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Gwyllion Hedge-Mage — two Plains: may make a 1/1 Kithkin Soldier; two
/// Swamps: may put a −1/−1 counter on target creature.
pub fn gwyllion_hedge_mage() -> CardDefinition {
    let kithkin = TokenDefinition {
        name: "Kithkin Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kithkin, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            hedge_etb(
                LandType::Plains,
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(kithkin) },
            ),
            hedge_etb(
                LandType::Swamp,
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::ONE,
                },
            ),
        ],
        ..creature(
            "Gwyllion Hedge-Mage",
            cost(&[generic(2), hybrid(Color::White, Color::Black)]),
            vec![CreatureType::Hag, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Kaalia of the Vast — flying; attacking an opponent, you may put an Angel,
/// Demon or Dragon from your hand onto the battlefield tapped and attacking.
pub fn kaalia_of_the_vast() -> CardDefinition {
    let mut t = on_attack(Effect::DeployCreatureFromHandAttacking {
        filter: R::HasCreatureType(CreatureType::Angel)
            .or(R::HasCreatureType(CreatureType::Demon))
            .or(R::HasCreatureType(CreatureType::Dragon)),
        return_to_hand_eot: false,
    });
    t.event = t.event.with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::IsAttackingAnOpponent });
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![t],
        ..creature(
            "Kaalia of the Vast",
            cost(&[generic(1), r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    })
}

/// Malfegor — flying; ETB discard your hand, and each opponent sacrifices a
/// creature per card discarded.
pub fn malfegor() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::CountOf(Box::new(Selector::DiscardedThisResolution { filter: R::Any })),
                filter: R::Creature,
            },
        ]))],
        ..creature(
            "Malfegor",
            cost(&[generic(2), b(), b(), r(), r()]),
            vec![CreatureType::Demon, CreatureType::Dragon],
            6,
            6,
        )
    })
}

/// Oros, the Avenger — flying; combat damage to a player: you may pay
/// {2}{W} for 3 damage to each nonwhite creature.
pub fn oros_the_avenger() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {2}{W} for 3 damage to each nonwhite creature?".into(),
                mana_cost: cost(&[generic(2), w()]),
                body: Box::new(Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasColor(Color::White))))),
                    amount: Value::Const(3),
                }),
                else_: None,
            },
        }],
        ..creature("Oros, the Avenger", cost(&[generic(3), r(), w(), b()]), vec![CreatureType::Dragon], 6, 6)
    })
}

/// Stranglehold — opponents can't search libraries; an opponent's extra
/// turn is skipped.
pub fn stranglehold() -> CardDefinition {
    CardDefinition {
        name: "Stranglehold",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Your opponents can't search libraries.",
                effect: StaticEffect::OpponentsCantSearchLibraries,
            },
            StaticAbility {
                description: "If an opponent would begin an extra turn, that player skips that turn instead.",
                effect: StaticEffect::OpponentsSkipExtraTurns,
            },
        ],
        ..Default::default()
    }
}

/// Sulfurous Blast — 2 damage to each creature and each player, 3 if cast in
/// your main phase.
pub fn sulfurous_blast() -> CardDefinition {
    let blast = |n: i32| {
        Effect::Seq(vec![
            Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(n) },
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(n) },
        ])
    };
    spell(
        "Sulfurous Blast",
        cost(&[generic(2), r(), r()]),
        CardType::Instant,
        Effect::If {
            cond: Predicate::YourMainPhase,
            then: Box::new(blast(3)),
            else_: Box::new(blast(2)),
        },
    )
}

/// Tariel, Reckoner of Souls — flying, vigilance; {T}: a random creature card
/// from target opponent's graveyard onto the battlefield under your control.
pub fn tariel_reckoner_of_souls() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Move {
                what: Selector::TakeRandom {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::ControllerOf(Box::new(target_filtered(R::OpponentPlayer))),
                        zone: crate::card::Zone::Graveyard,
                        filter: R::Creature,
                    }),
                    count: Box::new(Value::ONE),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Tariel, Reckoner of Souls",
            cost(&[generic(4), r(), w(), b()]),
            vec![CreatureType::Angel],
            4,
            7,
        )
    })
}

/// Vow of Lightning — enchanted creature gets +2/+2, first strike, and can't
/// attack you or your planeswalkers.
pub fn vow_of_lightning() -> CardDefinition {
    CardDefinition {
        name: "Vow of Lightning",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::FirstStrike, Keyword::CantAttackAuraController],
            ..Default::default()
        }),
        ..Default::default()
    }
}
