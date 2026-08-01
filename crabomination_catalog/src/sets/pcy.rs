//! Prophecy (PCY), first wave — the set's land-sacrifice / rhystic core.
//! Tests in `classic_sets/pcy`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value,
    WardCost,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, ZoneDest,
    shortcut::{target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w, x};

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

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
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

/// An Aura that enchants a land and grants it one activated ability.
fn land_aura(name: &'static str, c: ManaCost, ability: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        equipped_bonus: Some(EquipBonus { activated_abilities: vec![ability], ..Default::default() }),
        ..Default::default()
    }
}

/// "Sacrifice a land:" as an activation cost.
fn sac_land(mana: ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        sac_other_filter: Some((R::Land, 1)),
        effect,
        ..Default::default()
    }
}

/// Abolish — {1}{W}{W}. Naturalize you can pitch a Plains to.
pub fn abolish() -> CardDefinition {
    CardDefinition {
        name: "Abolish",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
        alternative_cost: Some(AlternativeCost {
            discard_filters: vec![(R::HasLandType(LandType::Plains), 1)],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Agent of Shauku — {1}{B} 1/1. Eats lands to push damage through.
pub fn agent_of_shauku() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![sac_land(
            cost(&[generic(1), b()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        )],
        ..creature(
            "Agent of Shauku",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            1,
            1,
        )
    }
}

/// Alexi's Cloak — {1}{U}. Flash shroud, on your creature or theirs.
pub fn alexis_cloak() -> CardDefinition {
    CardDefinition {
        name: "Alexi's Cloak",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Shroud],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Avatar of Fury — {6}{R}{R} 6/6. Cheap once they're land-flooded.
pub fn avatar_of_fury() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        self_cost_reduction_if: Some((
            Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::Land.and(R::ControlledByOpponent)),
                n: Value::Const(7),
            },
            6,
        )),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Avatar of Fury", cost(&[generic(6), r(), r()]), vec![CreatureType::Avatar], 6, 6)
    }
}

/// Avatar of Might — {6}{G}{G} 8/8. Cheap when you're being run over.
pub fn avatar_of_might() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        self_cost_reduction_if: Some((
            Predicate::ValueAtLeast(
                Value::Diff(
                    Box::new(Value::CreatureCountControlledBy(PlayerRef::EachOpponent)),
                    Box::new(Value::CreatureCountControlledBy(PlayerRef::You)),
                ),
                Value::Const(4),
            ),
            6,
        )),
        ..creature("Avatar of Might", cost(&[generic(6), g(), g()]), vec![CreatureType::Avatar], 8, 8)
    }
}

/// Avatar of Will — {6}{U}{U} 5/6. Cheap once they're empty-handed.
pub fn avatar_of_will() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        self_cost_reduction_if: Some((
            Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
                sel: Selector::CardsInZone {
                    who: PlayerRef::EachOpponent,
                    zone: crate::card::Zone::Hand,
                    filter: R::Any,
                },
                n: Value::ONE,
            })),
            6,
        )),
        ..creature("Avatar of Will", cost(&[generic(6), u(), u()]), vec![CreatureType::Avatar], 5, 6)
    }
}

/// Avatar of Woe — {6}{B}{B} 6/5. Cheap once the graveyards are full.
pub fn avatar_of_woe() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        self_cost_reduction_if: Some((
            Predicate::ValueAtLeast(
                Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::Creature,
                })),
                Value::Const(10),
            ),
            6,
        )),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::CantBeRegeneratedThisTurn { what: target_filtered(R::Creature) },
                Effect::Destroy { what: target_filtered(R::Creature) },
            ]),
            ..Default::default()
        }],
        ..creature("Avatar of Woe", cost(&[generic(6), b(), b()]), vec![CreatureType::Avatar], 6, 5)
    }
}

/// Barbed Field — {2}{R}{R}. Turns a land into a slow pinger.
pub fn barbed_field() -> CardDefinition {
    land_aura(
        "Barbed Field",
        cost(&[generic(2), r(), r()]),
        ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        },
    )
}

/// Blessed Wind — {7}{W}{W}. Nine mana to reset a life total to 20.
pub fn blessed_wind() -> CardDefinition {
    sorcery(
        "Blessed Wind",
        cost(&[generic(7), w(), w()]),
        Effect::SetLifeTotal {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(20),
        },
    )
}

/// Bog Elemental — {3}{B}{B} 5/4. A land a turn keeps it around.
pub fn bog_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(crate::mana::Color::White)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::SacrificeSourceUnlessSacrifice { filter: R::Land },
        }],
        ..creature("Bog Elemental", cost(&[generic(3), b(), b()]), vec![CreatureType::Elemental], 5, 4)
    }
}

/// Bog Glider — {2}{B} 1/1 flier. Trades a land for the next Mercenary.
pub fn bog_glider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::PermanentCard
                    .and(R::HasCreatureType(CreatureType::Mercenary))
                    .and(R::ManaValueAtMost(2)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Bog Glider",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            1,
            1,
        )
    }
}

/// Branded Brawlers — {R} 2/2. Only fights when the mana is spent.
pub fn branded_brawlers() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CantAttackIfDefenderHasUntappedLand,
            Keyword::CantBlockIfYouHaveUntappedLand,
        ],
        ..creature(
            "Branded Brawlers",
            cost(&[r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Chilling Apparition — {2}{B} 1/1. Regenerating hand attack.
pub fn chilling_apparition() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..creature("Chilling Apparition", cost(&[generic(2), b()]), vec![CreatureType::Spirit], 1, 1)
    }
}

/// Chimeric Idol — {3}. Stands up as a 3/3 by shutting off your mana.
pub fn chimeric_idol() -> CardDefinition {
    CardDefinition {
        name: "Chimeric Idol",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                },
                Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![CreatureType::Turtle],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Citadel of Pain — {2}{R}. Punishes anyone who holds up mana.
pub fn citadel_of_pain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::PermanentCountControlledByMatching(
                    PlayerRef::ActivePlayer,
                    R::Land.and(R::Untapped),
                ),
            },
        }],
        ..enchantment("Citadel of Pain", cost(&[generic(2), r()]))
    }
}

/// Coastal Hornclaw — {4}{U} 3/3. Eats a land to fly over.
pub fn coastal_hornclaw() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![sac_land(
            ManaCost::default(),
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        )],
        ..creature("Coastal Hornclaw", cost(&[generic(4), u()]), vec![CreatureType::Bird], 3, 3)
    }
}

/// Copper-Leaf Angel — {5} 2/2 flier. Turns spare lands into counters.
pub fn copper_leaf_angel() -> CardDefinition {
    CardDefinition {
        name: "Copper-Leaf Angel",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Darba — {3}{G} 5/4. Two green a turn or it walks off.
pub fn darba() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[g(), g()]) },
        }],
        ..creature("Darba", cost(&[generic(3), g()]), vec![CreatureType::Bird, CreatureType::Beast], 5, 4)
    }
}

/// Death Charmer — {2}{B} 2/2. Its bite bills the blocker's controller.
pub fn death_charmer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                cost: WardCost::Mana(cost(&[generic(2)])),
                then: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(2),
                }),
                if_paid: None,
            },
        }],
        ..creature(
            "Death Charmer",
            cost(&[generic(2), b()]),
            vec![CreatureType::Worm, CreatureType::Mercenary],
            2,
            2,
        )
    }
}

/// Despoil — {3}{B}. Land destruction with a life kicker.
pub fn despoil() -> CardDefinition {
    sorcery(
        "Despoil",
        cost(&[generic(3), b()]),
        Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(2),
            },
            Effect::Destroy { what: target_filtered(R::Land) },
        ]),
    )
}

/// Devastate — {3}{R}{R}. A land plus a board-wide singe.
pub fn devastate() -> CardDefinition {
    sorcery(
        "Devastate",
        cost(&[generic(3), r(), r()]),
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Land) },
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature),
                amount: Value::ONE,
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::ONE,
            },
        ]),
    )
}

/// Diving Griffin — {1}{W}{W} 2/2 flying vigilance.
pub fn diving_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..creature("Diving Griffin", cost(&[generic(1), w(), w()]), vec![CreatureType::Griffin], 2, 2)
    }
}

/// Endbringer's Revel — {2}{B}. A graveyard everyone gets to raid.
pub fn endbringers_revel() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            any_player: true,
            sorcery_speed: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..enchantment("Endbringer's Revel", cost(&[generic(2), b()]))
    }
}

/// Excavation — {1}{U}. Anyone can trade a land for a card.
pub fn excavation() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Land, 1)),
            any_player: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..enchantment("Excavation", cost(&[generic(1), u()]))
    }
}

/// Excise — {X}{W}. Taxes an attacker out of the game.
pub fn excise() -> CardDefinition {
    CardDefinition {
        name: "Excise",
        cost: cost(&[x(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::UnlessPlayerPays {
            who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            cost: WardCost::GenericXFromCost,
            then: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                to: ZoneDest::Exile,
            }),
            if_paid: None,
        },
        ..Default::default()
    }
}

/// Fault Riders — {2}{R} 2/2. One land, one big swing a turn.
pub fn fault_riders() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Land, 1)),
            once_per_turn: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Fault Riders",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Fen Stalker — {3}{B} 3/2. Evasive once you're tapped out.
pub fn fen_stalker() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has fear as long as you control no untapped lands.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Fear,
                condition: Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Land.and(R::Untapped).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                })),
            },
        }],
        ..creature("Fen Stalker", cost(&[generic(3), b()]), vec![CreatureType::Nightstalker], 3, 2)
    }
}

/// Flameshot — {3}{R}. Three damage spread around; pitch a Mountain instead.
pub fn flameshot() -> CardDefinition {
    CardDefinition {
        effect: Effect::DealDamageDivided {
            total: Value::Const(3),
            filter: R::Creature,
            max_targets: 3,
            retaliate_to_source: false,
        },
        alternative_cost: Some(AlternativeCost {
            discard_filters: vec![(R::HasLandType(LandType::Mountain), 1)],
            ..Default::default()
        }),
        ..sorcery("Flameshot", cost(&[generic(3), r()]), Effect::Noop)
    }
}

/// Flay — {3}{B}. One random discard, and a second unless they pay.
pub fn flay() -> CardDefinition {
    sorcery(
        "Flay",
        cost(&[generic(3), b()]),
        Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: true,
            },
            Effect::UnlessPlayerPays {
                who: PlayerRef::Target(0),
                cost: WardCost::Mana(cost(&[generic(1)])),
                then: Box::new(Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                    random: true,
                }),
                if_paid: None,
            },
        ]),
    )
}

/// Flowering Field — {1}{W}. Turns a land into a damage sponge.
pub fn flowering_field() -> CardDefinition {
    land_aura(
        "Flowering Field",
        cost(&[generic(1), w()]),
        ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage { target: target_any(), amount: Value::ONE },
            ..Default::default()
        },
    )
}

/// Foil — {2}{U}{U}. A free counterspell for two cards, one an Island.
pub fn foil() -> CardDefinition {
    CardDefinition {
        name: "Foil",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: crate::effect::shortcut::counter_target_spell(),
        alternative_cost: Some(AlternativeCost {
            discard_filters: vec![(R::HasLandType(LandType::Island), 1), (R::Any, 1)],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Glittering Lion — {2}{W} 2/2. Untouchable until someone pays {3}.
pub fn glittering_lion() -> CardDefinition {
    glittering_cat("Glittering Lion", cost(&[generic(2), w()]), 2, 2, 3)
}

/// Glittering Lynx — {W} 1/1. The one-drop half of the cycle.
pub fn glittering_lynx() -> CardDefinition {
    glittering_cat("Glittering Lynx", cost(&[w()]), 1, 1, 2)
}

fn glittering_cat(name: &'static str, c: ManaCost, p: i32, t: i32, unlock: u32) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to this creature.",
            effect: StaticEffect::PreventAllDamageToThis,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(unlock)]),
            any_player: true,
            effect: Effect::TurnOffDamagePreventionThisTurn { what: Selector::This },
            ..Default::default()
        }],
        ..creature(name, c, vec![CreatureType::Cat], p, t)
    }
}

/// Greel's Caress — {1}{B}. Flash −3/−0, enough to blank an attacker.
pub fn greels_caress() -> CardDefinition {
    CardDefinition {
        name: "Greel's Caress",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: -3, ..Default::default() }),
        ..Default::default()
    }
}

/// Gulf Squid — {3}{U} 2/2. Its ETB costs them a whole turn of mana.
pub fn gulf_squid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Land },
            },
        }],
        ..creature("Gulf Squid", cost(&[generic(3), u()]), vec![CreatureType::Squid, CreatureType::Beast], 2, 2)
    }
}

/// Elephant Resurgence — {1}{G}. Everyone gets a graveyard-sized Elephant.
pub fn elephant_resurgence() -> CardDefinition {
    use crate::card::TokenDefinition;
    let creatures_in_my_graveyard = Value::CountOf(Box::new(Selector::CardsInZone {
        who: PlayerRef::You,
        zone: crate::card::Zone::Graveyard,
        filter: R::Creature,
    }));
    sorcery(
        "Elephant Resurgence",
        cost(&[generic(1), g()]),
        Effect::CreateToken {
            who: PlayerRef::EachPlayer,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Elephant".to_string(),
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Elephant],
                    ..Default::default()
                },
                colors: vec![crate::mana::Color::Green],
                static_abilities: vec![StaticAbility {
                    description: "This token's power and toughness are each equal to the number of creature cards in its controller's graveyard.",
                    effect: StaticEffect::SelfBasePtFromValue {
                        power: creatures_in_my_graveyard.clone(),
                        toughness: creatures_in_my_graveyard,
                    },
                }],
                ..Default::default()
            },
        },
    )
}
