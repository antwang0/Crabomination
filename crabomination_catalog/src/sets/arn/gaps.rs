//! Arabian Nights (ARN) gap batch. Tests in `classic_sets/arn`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, Value, ZoneDest,
    shortcut::{target_any, target_filtered},
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn artifact_creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, c, types, p, t)
    }
}

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

fn land(name: &'static str, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: abilities,
        ..Default::default()
    }
}

fn aura(name: &'static str, c: ManaCost, bonus: crate::card::EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// "At the beginning of your upkeep, …"
fn upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource),
        effect,
    }
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Flying Men — {U} 1/1 flier.
pub fn flying_men() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature("Flying Men", cost(&[u()]), vec![CreatureType::Human], 1, 1)
    }
}

/// Moorish Cavalry — {2}{W}{W} 3/3 trampler.
pub fn moorish_cavalry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature(
            "Moorish Cavalry",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Stone-Throwing Devils — {B} 1/1 first striker.
pub fn stone_throwing_devils() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        ..creature("Stone-Throwing Devils", cost(&[b()]), vec![CreatureType::Devil], 1, 1)
    }
}

/// Dancing Scimitar — {4} 1/5 flying artifact creature.
pub fn dancing_scimitar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..artifact_creature(
            "Dancing Scimitar",
            cost(&[generic(4)]),
            vec![CreatureType::Spirit],
            1,
            5,
        )
    }
}

/// Repentant Blacksmith — {1}{W} 1/2 with protection from red.
pub fn repentant_blacksmith() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(crate::mana::Color::Red)],
        ..creature("Repentant Blacksmith", cost(&[generic(1), w()]), vec![CreatureType::Human], 1, 2)
    }
}

/// War Elephant — {3}{W} 2/2 with trample and banding.
pub fn war_elephant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Banding],
        ..creature("War Elephant", cost(&[generic(3), w()]), vec![CreatureType::Elephant], 2, 2)
    }
}

/// Camel — {W} 0/1 bander that shrugs off Desert pings while attacking.
pub fn camel() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Banding,
            Keyword::PreventDamageFromMatching(Box::new(R::HasLandType(LandType::Desert))),
        ],
        ..creature("Camel", cost(&[w()]), vec![CreatureType::Camel], 0, 1)
    }
}

/// Desert Nomads — {2}{R} 2/2 desertwalker, immune to Deserts.
pub fn desert_nomads() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Landwalk(LandType::Desert),
            Keyword::PreventDamageFromMatching(Box::new(R::HasLandType(LandType::Desert))),
        ],
        ..creature(
            "Desert Nomads",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            2,
            2,
        )
    }
}

/// Serendib Efreet — {2}{U} 3/4 flier that bleeds you a point each upkeep.
pub fn serendib_efreet() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![upkeep(Effect::DealDamage {
            to: Selector::You,
            amount: Value::ONE,
        })],
        ..creature("Serendib Efreet", cost(&[generic(2), u()]), vec![CreatureType::Efreet], 3, 4)
    }
}

/// Junún Efreet — {1}{B}{B} 3/3 flier with a {B}{B} upkeep rent.
pub fn junun_efreet() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![upkeep(Effect::SacrificeSourceUnlessPay {
            cost: cost(&[b(), b()]),
        })],
        ..creature("Junún Efreet", cost(&[generic(1), b(), b()]), vec![CreatureType::Efreet], 3, 3)
    }
}

/// Giant Tortoise — {1}{U} 1/1 that hunkers down to 1/4 while untapped.
pub fn giant_tortoise() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::card::StaticAbility {
            description: "This creature gets +0/+3 as long as it's untapped.",
            effect: crate::effect::StaticEffect::PumpSelfIf {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::Untapped },
                power: 0,
                toughness: 3,
                keywords: vec![],
            },
        }],
        ..creature("Giant Tortoise", cost(&[generic(1), u()]), vec![CreatureType::Turtle], 1, 1)
    }
}

/// Ali Baba — {R} 1/1 who taps Walls for a red mana.
pub fn ali_baba() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::Tap {
                what: target_filtered(R::HasCreatureType(CreatureType::Wall)),
            },
            ..Default::default()
        }],
        ..creature("Ali Baba", cost(&[r()]), vec![CreatureType::Human, CreatureType::Rogue], 1, 1)
    }
}

/// Hurr Jackal — {R} 1/1 that strips regeneration.
pub fn hurr_jackal() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CantBeRegeneratedThisTurn { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Hurr Jackal", cost(&[r()]), vec![CreatureType::Jackal], 1, 1)
    }
}

/// King Suleiman — {1}{W} 1/1 that executes Djinn and Efreet.
pub fn king_suleiman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Djinn)
                        .or(R::HasCreatureType(CreatureType::Efreet)),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "King Suleiman",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Noble],
            1,
            1,
        )
    }
}

/// El-Hajjâj — {1}{B}{B} 1/1 that drinks whatever damage it deals.
pub fn el_hajjaj() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature(
            "El-Hajjâj",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Khabál Ghoul — {2}{B} 1/1 that fattens on every end step's corpses.
pub fn khabal_ghoul() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CreaturesDiedThisTurnTotal,
            },
        }],
        ..creature("Khabál Ghoul", cost(&[generic(2), b()]), vec![CreatureType::Zombie], 1, 1)
    }
}

/// Hasran Ogress — {B}{B} 3/2 that charges you {2} to swing.
pub fn hasran_ogress() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PayManaOrElse {
                mana_cost: cost(&[generic(2)]),
                otherwise: Box::new(Effect::DealDamage {
                    to: Selector::You,
                    amount: Value::Const(3),
                }),
            },
        }],
        ..creature("Hasran Ogress", cost(&[b(), b()]), vec![CreatureType::Ogre], 3, 2)
    }
}

/// Sindbad — {1}{U} 1/1 that digs for lands and bins everything else.
pub fn sindbad() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::EntityMatches {
                        what: Selector::LastCardYouDrew,
                        filter: R::Land,
                    })),
                    then: Box::new(Effect::Move {
                        what: Selector::LastCardYouDrew,
                        to: ZoneDest::Graveyard,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Sindbad", cost(&[generic(1), u()]), vec![CreatureType::Human], 1, 1)
    }
}

/// Singing Tree — {3}{G} 0/3 that sings an attacker's power down to nothing.
pub fn singing_tree() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::SetBasePower {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                power: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Singing Tree", cost(&[generic(3), g()]), vec![CreatureType::Plant], 0, 3)
    }
}

/// Sorceress Queen — {1}{B}{B} 1/1 that shrinks anything else to 0/2.
pub fn sorceress_queen() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::SetBasePT {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::IsSource)))),
                power: Value::ZERO,
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Sorceress Queen",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Sorcerer],
            1,
            1,
        )
    }
}

/// Brass Man — {1} 1/3 that stays tapped unless you feed it {1} each upkeep.
pub fn brass_man() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::card::StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: crate::effect::StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        triggered_abilities: vec![upkeep(Effect::MayPay {
            description: "Pay {1} to untap Brass Man?".into(),
            mana_cost: cost(&[generic(1)]),
            body: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
            else_: None,
        })],
        ..artifact_creature("Brass Man", cost(&[generic(1)]), vec![CreatureType::Construct], 1, 3)
    }
}

/// Erhnam Djinn — {3}{G} 4/5 that hands an opponent's creature forestwalk.
pub fn erhnam_djinn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![upkeep(Effect::GrantKeyword {
            what: target_filtered(
                R::Creature
                    .and(R::ControlledByOpponent)
                    .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Wall)))),
            ),
            keyword: Keyword::Landwalk(LandType::Forest),
            duration: Duration::UntilYourNextUpkeep,
        })],
        ..creature("Erhnam Djinn", cost(&[generic(3), g()]), vec![CreatureType::Djinn], 4, 5)
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Desert Twister — {4}{G}{G}, destroy anything.
pub fn desert_twister() -> CardDefinition {
    sorcery(
        "Desert Twister",
        cost(&[generic(4), g(), g()]),
        Effect::Destroy { what: target_filtered(R::Permanent) },
    )
}

/// Army of Allah — {1}{W}{W}, every attacker gets +2/+0.
pub fn army_of_allah() -> CardDefinition {
    instant(
        "Army of Allah",
        cost(&[generic(1), w(), w()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
            power: Value::Const(2),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Piety — {2}{W}, every blocker gets +0/+3.
pub fn piety() -> CardDefinition {
    instant(
        "Piety",
        cost(&[generic(2), w()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::IsBlocking)),
            power: Value::ZERO,
            toughness: Value::Const(3),
            duration: Duration::EndOfTurn,
        },
    )
}

// ── Auras ──────────────────────────────────────────────────────────────────

/// Fishliver Oil — {1}{U} Aura granting islandwalk.
pub fn fishliver_oil() -> CardDefinition {
    aura(
        "Fishliver Oil",
        cost(&[generic(1), u()]),
        crate::card::EquipBonus {
            keywords: vec![Keyword::Landwalk(LandType::Island)],
            ..Default::default()
        },
    )
}

/// Unstable Mutation — {U} Aura: +3/+3 that rots a counter off each upkeep.
pub fn unstable_mutation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::AddCounter {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
        }],
        ..aura(
            "Unstable Mutation",
            cost(&[u()]),
            crate::card::EquipBonus { power: 3, toughness: 3, ..Default::default() },
        )
    }
}

// ── Artifacts ──────────────────────────────────────────────────────────────

/// Jandor's Saddlebags — {3}, {T}: untap a creature.
pub fn jandors_saddlebags() -> CardDefinition {
    artifact(
        "Jandor's Saddlebags",
        cost(&[generic(2)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            ..Default::default()
        }],
    )
}

/// Flying Carpet — {2}, {T}: a creature gains flying.
pub fn flying_carpet() -> CardDefinition {
    artifact(
        "Flying Carpet",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
    )
}

/// Bottle of Suleiman — {1}, Sacrifice: a 5/5 flying Djinn, or five to the face.
pub fn bottle_of_suleiman() -> CardDefinition {
    artifact(
        "Bottle of Suleiman",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: crate::card::TokenDefinition {
                        name: "Djinn".into(),
                        power: 5,
                        toughness: 5,
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Djinn],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Flying],
                        ..Default::default()
                    },
                }),
                on_tails: Box::new(Effect::DealDamage {
                    to: Selector::You,
                    amount: Value::Const(5),
                }),
            },
            ..Default::default()
        }],
    )
}

// ── Lands ──────────────────────────────────────────────────────────────────

/// Bazaar of Baghdad — draw two, discard three.
pub fn bazaar_of_baghdad() -> CardDefinition {
    land(
        "Bazaar of Baghdad",
        vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::Discard { who: Selector::You, amount: Value::Const(3), random: false },
            ]),
            ..Default::default()
        }],
    )
}

/// Diamond Valley — trade a creature for its toughness in life.
pub fn diamond_valley() -> CardDefinition {
    land(
        "Diamond Valley",
        vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature.and(R::ControlledByYou), 1)),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::SacrificedToughness,
            },
            ..Default::default()
        }],
    )
}

/// Elephant Graveyard — colorless mana, and a regeneration shield for Elephants.
pub fn elephant_graveyard() -> CardDefinition {
    land(
        "Elephant Graveyard",
        vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Regenerate {
                    what: target_filtered(R::HasCreatureType(CreatureType::Elephant)),
                },
                ..Default::default()
            },
        ],
    )
}

/// Library of Alexandria — a free card while your hand is exactly seven.
pub fn library_of_alexandria() -> CardDefinition {
    land(
        "Library of Alexandria",
        vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::ValueEquals(
                    Value::HandSizeOf(PlayerRef::You),
                    Value::Const(7),
                )),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
    )
}

/// Desert — colorless mana, and a ping for an attacker at end of combat.
pub fn desert() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            land_types: vec![LandType::Desert],
            ..Default::default()
        },
        ..land(
            "Desert",
            vec![
                ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                    },
                    ..Default::default()
                },
                ActivatedAbility {
                    tap_cost: true,
                    condition: Some(Predicate::CurrentStepIs(TurnStep::EndCombat)),
                    effect: Effect::DealDamage {
                        to: target_filtered(R::Creature.and(R::IsAttacking)),
                        amount: Value::ONE,
                    },
                    ..Default::default()
                },
            ],
        )
    }
}

/// Oasis — soak one point off a creature.
pub fn oasis() -> CardDefinition {
    land(
        "Oasis",
        vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Creature),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
    )
}

/// Island of Wak-Wak — grounds a flier's power for the turn.
pub fn island_of_wak_wak() -> CardDefinition {
    land(
        "Island of Wak-Wak",
        vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::SetBasePower {
                what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                power: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
    )
}

// ── Second wave ────────────────────────────────────────────────────────────

/// Abu Ja'far — 0/1 that takes its whole combat down with it.
pub fn abu_jafar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DestroyNoRegen {
                what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
            },
        }],
        ..creature("Abu Ja'far", cost(&[w()]), vec![CreatureType::Human], 0, 1)
    }
}

/// Ali from Cairo — damage can't take you below 1 life.
pub fn ali_from_cairo() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::card::StaticAbility {
            description: "Damage that would reduce your life total to less than 1 reduces it to 1.",
            effect: crate::effect::StaticEffect::DamageWontReduceControllerLifeBelowOne {
                requires_creature: false,
            },
        }],
        ..creature("Ali from Cairo", cost(&[generic(2), r(), r()]), vec![CreatureType::Human], 0, 1)
    }
}

/// Aladdin — {1}{R}{R}, {T}: borrow an artifact for as long as he lives.
pub fn aladdin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), r()]),
            tap_cost: true,
            effect: Effect::GainControlWhileSourceRemains {
                what: target_filtered(R::Artifact),
            },
            ..Default::default()
        }],
        ..creature("Aladdin", cost(&[generic(2), r(), r()]), vec![CreatureType::Human, CreatureType::Rogue], 1, 1)
    }
}

/// Old Man of the Sea — holds a creature no bigger than himself while tapped.
pub fn old_man_of_the_sea() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainControlWhileSourceTapped {
                what: target_filtered(R::Creature.and(R::PowerAtMostSourcePower)),
            },
            ..Default::default()
        }],
        ..creature("Old Man of the Sea", cost(&[generic(1), u(), u()]), vec![CreatureType::Djinn], 2, 3)
    }
}

/// Ghazbán Ogre — always defects to whoever's winning on life.
pub fn ghazban_ogre() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::GainControl {
                what: Selector::This,
                to: Some(PlayerRef::PlayerWithMostLife),
                duration: Duration::Permanent,
            },
        }],
        ..creature("Ghazbán Ogre", cost(&[g()]), vec![CreatureType::Ogre], 2, 2)
    }
}

/// Oubliette — buries a creature (and its baggage) until the enchantment goes.
pub fn oubliette() -> CardDefinition {
    CardDefinition {
        name: "Oubliette",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PhaseOut {
                what: target_filtered(R::Creature),
                until_source_leaves: true,
            },
        }],
        ..Default::default()
    }
}

/// Merchant Ship — {U} 0/2 that only sails at Islands, and pays out unblocked.
pub fn merchant_ship() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackUnlessDefenderControlsLandType(LandType::Island)],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::SelectorExists(
                        Selector::EachPermanent(
                            R::HasLandType(LandType::Island).and(R::ControlledByYou),
                        ),
                    ))),
                    then: Box::new(Effect::SacrificeSource),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..creature("Merchant Ship", cost(&[u()]), vec![CreatureType::Human], 0, 2)
    }
}

/// Island Fish Jasconius — a 6/8 that needs {U}{U}{U} a turn and an Island to swim at.
pub fn island_fish_jasconius() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackUnlessDefenderControlsLandType(LandType::Island)],
        static_abilities: vec![crate::card::StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: crate::effect::StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        triggered_abilities: vec![
            upkeep(Effect::MayPay {
                description: "Pay {U}{U}{U} to untap Island Fish Jasconius?".into(),
                mana_cost: cost(&[u(), u(), u()]),
                body: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
                else_: None,
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::SelectorExists(
                        Selector::EachPermanent(
                            R::HasLandType(LandType::Island).and(R::ControlledByYou),
                        ),
                    ))),
                    then: Box::new(Effect::SacrificeSource),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..creature(
            "Island Fish Jasconius",
            cost(&[generic(4), u(), u(), u()]),
            vec![CreatureType::Fish],
            6,
            8,
        )
    }
}

/// Serendib Djinn — a 5/6 flier that eats one of your lands every upkeep.
pub fn serendib_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            upkeep(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Land,
                },
                Effect::If {
                    cond: Predicate::EntityMatchesAny {
                        what: Selector::SacrificedCard,
                        filter: R::HasLandType(LandType::Island),
                    },
                    then: Box::new(Effect::DealDamage {
                        to: Selector::You,
                        amount: Value::Const(3),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::SelectorExists(
                        Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                    ))),
                    then: Box::new(Effect::SacrificeSource),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..creature(
            "Serendib Djinn",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Djinn],
            5,
            6,
        )
    }
}

/// Ifh-Bíff Efreet — anyone can pay {G} to rake the skies (and every player).
pub fn ifh_biff_efreet() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            any_player: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                    amount: Value::ONE,
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Ifh-Bíff Efreet", cost(&[generic(2), g(), g()]), vec![CreatureType::Efreet], 3, 3)
    }
}

/// Ebony Horse — pulls an attacker back out of the fight.
pub fn ebony_horse() -> CardDefinition {
    artifact(
        "Ebony Horse",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Untap {
                    what: target_filtered(R::Creature.and(R::IsAttacking).and(R::ControlledByYou)),
                    up_to: None,
                },
                Effect::PreventAllCombatDamageInvolving { target: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
    )
}

/// Jandor's Ring — swap the card you just drew for a fresh one.
pub fn jandors_ring() -> CardDefinition {
    artifact(
        "Jandor's Ring",
        cost(&[generic(6)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            condition: Some(Predicate::SelectorExists(Selector::LastCardYouDrew)),
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::LastCardYouDrew, to: ZoneDest::Graveyard },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
    )
}

/// Ring of Ma'rûf — buys a card from outside the game.
pub fn ring_of_maruf() -> CardDefinition {
    artifact(
        "Ring of Ma'rûf",
        cost(&[generic(5)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::WishToHand { filter: R::Any },
            ..Default::default()
        }],
    )
}

/// Pyramids — {2}: peel an Aura off a land, or shore one up against destruction.
pub fn pyramids() -> CardDefinition {
    artifact(
        "Pyramids",
        cost(&[generic(6)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::ChooseMode(vec![
                Effect::Destroy {
                    what: target_filtered(
                        R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Aura),
                    ),
                },
                Effect::Regenerate { what: target_filtered(R::Land) },
            ]),
            ..Default::default()
        }],
    )
}

/// City in a Bottle — wipes Arabian Nights off the table and keeps it off.
pub fn city_in_a_bottle() -> CardDefinition {
    CardDefinition {
        name: "City in a Bottle",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::SacrificeAllMatching {
                who: Selector::Player(PlayerRef::EachPlayer),
                filter: R::Not(Box::new(R::IsToken))
                    .and(R::Not(Box::new(R::IsSource)))
                    .and(R::OriginallyPrintedIn(crate::card::OriginalSet::ArabianNights)),
            },
        }],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Players can't cast spells or play lands originally printed in Arabian Nights.",
            effect: crate::effect::StaticEffect::PlayersCantPlayMatching {
                filter: R::OriginallyPrintedIn(crate::card::OriginalSet::ArabianNights),
            },
        }],
        ..Default::default()
    }
}

/// Cuombajj Witches — pings one target, and an opponent picks the second.
pub fn cuombajj_witches() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::ONE },
                Effect::OpponentChoosesTargetForDamage { amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Cuombajj Witches",
            cost(&[b(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Metamorphosis — a creature becomes creature-only mana, one bigger.
pub fn metamorphosis() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature.and(R::ControlledByYou),
            count: 1,
        }],
        ..sorcery(
            "Metamorphosis",
            cost(&[g()]),
            Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::Restricted(
                    Box::new(crate::effect::ManaPayload::AnyOneColor(Value::Sum(vec![
                        Value::ONE,
                        Value::SacrificedManaValue,
                    ]))),
                    crate::mana::SpendRestriction::CreatureOnly,
                ),
            },
        )
    }
}

/// Drop of Honey — culls the smallest creature every upkeep, then expires.
pub fn drop_of_honey() -> CardDefinition {
    CardDefinition {
        name: "Drop of Honey",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            upkeep(Effect::DestroyNoRegen {
                what: Selector::LeastPowerAmongAll,
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::Not(Box::new(Predicate::SelectorExists(
                        Selector::EachPermanent(R::Creature),
                    ))),
                    then: Box::new(Effect::SacrificeSource),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Cyclone — a self-feeding storm that eventually blows the board away.
pub fn cyclone() -> CardDefinition {
    CardDefinition {
        name: "Cyclone",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![upkeep(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Wind,
                amount: Value::ONE,
            },
            Effect::PayPerCounterOrSacrifice {
                kind: CounterType::Wind,
                per: cost(&[g()]),
                then: Box::new(Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::EachPermanent(R::Creature),
                        amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Wind },
                    },
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Wind },
                    },
                ])),
            },
        ]))],
        ..Default::default()
    }
}

// ── Third wave ─────────────────────────────────────────────────────────────

/// Magnetic Mountain — blue creatures stay down unless their controller pays.
pub fn magnetic_mountain() -> CardDefinition {
    CardDefinition {
        name: "Magnetic Mountain",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Blue creatures don't untap during their controllers' untap steps.",
            effect: crate::effect::StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Blue))),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::MayPayRepeatedly {
                who: PlayerRef::ActivePlayer,
                description: "Pay {4} to untap a tapped blue creature?".into(),
                mana_cost: cost(&[generic(4)]),
                body: Box::new(Effect::Untap {
                    what: Selector::EachPermanent(
                        R::Creature
                            .and(R::HasColor(Color::Blue))
                            .and(R::Tapped)
                            
                    ),
                    up_to: Some(Value::ONE),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Guardian Beast — while it stands, your noncreature artifacts are untouchable.
pub fn guardian_beast() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::card::StaticAbility {
            description: "While untapped, your noncreature artifacts are indestructible and can't be enchanted.",
            effect: crate::effect::StaticEffect::AnthemForFilterIf {
                filter: R::Artifact.and(R::Not(Box::new(R::Creature))),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Indestructible, Keyword::CantBeTargetedByAuras],
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::Untapped,
                },
                all_players: false,
            },
        }],
        ..creature("Guardian Beast", cost(&[generic(3), b()]), vec![CreatureType::Beast], 2, 4)
    }
}

/// Sandals of Abdallah — lends islandwalk, and breaks when the wearer dies.
pub fn sandals_of_abdallah() -> CardDefinition {
    artifact(
        "Sandals of Abdallah",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Landwalk(LandType::Island),
                    duration: Duration::EndOfTurn,
                },
                Effect::WhenTargetDiesThisTurn {
                    body: Box::new(Effect::Destroy { what: Selector::This }),
                    slot: 0,
                    filter: None,
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Nafs Asp — its bite costs a life at the victim's next draw step, or {1}.
pub fn nafs_asp() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::DelayUntilWithCapture {
                kind: crate::effect::DelayedTriggerKind::TargetsNextDrawStep,
                capture: Selector::Player(PlayerRef::DefendingPlayer),
                body: Box::new(Effect::MayPayBy {
                    who: PlayerRef::Target(0),
                    description: "Pay {1} to shake off the asp's venom?".into(),
                    mana_cost: cost(&[generic(1)]),
                    body: Box::new(Effect::Noop),
                    else_: Some(Box::new(Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::ONE,
                    })),
                }),
            },
        }],
        ..creature("Nafs Asp", cost(&[g()]), vec![CreatureType::Snake], 1, 1)
    }
}

/// Jihad — white creatures swell while an opponent still shows the chosen colour.
pub fn jihad() -> CardDefinition {
    let chosen_on_board = || {
        Predicate::SelectorExists(Selector::EachPermanent(
            R::HasChosenColorOfSource
                .and(R::Not(Box::new(R::IsToken)))
                .and(R::ControlledByOpponent),
        ))
    };
    CardDefinition {
        name: "Jihad",
        cost: cost(&[w(), w(), w()]),
        card_types: vec![CardType::Enchantment],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![crate::card::StaticAbility {
            description: "White creatures get +2/+1 while the chosen colour is on an opponent's board.",
            effect: crate::effect::StaticEffect::AnthemForFilterIf {
                filter: R::Creature.and(R::HasColor(Color::White)),
                power: 2,
                toughness: 1,
                keywords: vec![],
                condition: chosen_on_board(),
                all_players: true,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::If {
                cond: Predicate::Not(Box::new(chosen_on_board())),
                then: Box::new(Effect::SacrificeSource),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Eye for an Eye — the next source to hit you bleeds its controller for the
/// same.
pub fn eye_for_an_eye() -> CardDefinition {
    instant("Eye for an Eye", cost(&[w(), w()]), Effect::MirrorNextDamageToYouThisTurn)
}
