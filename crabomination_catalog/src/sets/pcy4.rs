//! Prophecy (PCY), closing wave — the untapped-lands-matter shell, the
//! sacrifice-a-land utility creatures and the remaining rares. Tests in
//! `classic_sets/pcy4`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    WardCost,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, ZoneDest,
    shortcut::{pump_target, target_any, target_filtered},
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

/// An Aura that grants the enchanted land one activated ability.
fn land_aura(name: &'static str, c: ManaCost, ability: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ability],
            ..Default::default()
        }),
        ..enchantment(name, c)
    }
}

/// "You control no untapped lands" — the PCY set-wide tap-out condition.
fn no_untapped_lands() -> Predicate {
    Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
        R::Land.and(R::Untapped).and(R::ControlledByYou),
    ))))
}

/// "Sacrifice a land: [effect]" on the source itself.
fn sac_land_ability(effect: Effect) -> ActivatedAbility {
    ActivatedAbility { sac_other_filter: Some((R::Land, 1)), effect, ..Default::default() }
}

/// Spur Grappler — {2}{R} 2/1. Bigger once you're tapped out.
pub fn spur_grappler() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +2/+1 as long as you control no untapped lands.",
            effect: StaticEffect::PumpSelfIf {
                condition: no_untapped_lands(),
                power: 2,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..creature("Spur Grappler", cost(&[generic(2), r()]), vec![CreatureType::Beast], 2, 1)
    }
}

/// Vintara Snapper — {G}{G} 2/2. Untargetable while you're tapped out.
pub fn vintara_snapper() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has shroud as long as you control no untapped lands.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Shroud,
                condition: no_untapped_lands(),
            },
        }],
        ..creature("Vintara Snapper", cost(&[g(), g()]), vec![CreatureType::Turtle], 2, 2)
    }
}

/// Vintara Elephant — {4}{G} 4/3 trample that anyone can switch off.
pub fn vintara_elephant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            any_player: true,
            effect: Effect::LoseKeyword { duration: Duration::EndOfTurn,
                what: Selector::This,
                keyword: Keyword::Trample,
            },
            ..Default::default()
        }],
        ..creature("Vintara Elephant", cost(&[generic(4), g()]), vec![CreatureType::Elephant], 4, 3)
    }
}

/// Zerapa Minotaur — {2}{R}{R} 3/3 first strike that anyone can switch off.
pub fn zerapa_minotaur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            any_player: true,
            effect: Effect::LoseKeyword { duration: Duration::EndOfTurn,
                what: Selector::This,
                keyword: Keyword::FirstStrike,
            },
            ..Default::default()
        }],
        ..creature("Zerapa Minotaur", cost(&[generic(2), r(), r()]), vec![CreatureType::Minotaur], 3, 3)
    }
}

/// Wall of Vipers — {2}{B} 2/4 defender. Anyone can trade it for what it blocks.
pub fn wall_of_vipers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            any_player: true,
            effect: Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature.and(R::InCombatWithSource)) },
                Effect::Destroy { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..creature("Wall of Vipers", cost(&[generic(2), b()]), vec![CreatureType::Snake, CreatureType::Wall], 2, 4)
    }
}

/// Veteran Brawlers — {1}{R} 4/4. Both sides have to be tapped out.
pub fn veteran_brawlers() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CantAttackIfDefenderHasUntappedLand,
            Keyword::CantBlockIfYouHaveUntappedLand,
        ],
        ..creature(
            "Veteran Brawlers",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Well of Discovery — {6}. A card each turn you end tapped out.
pub fn well_of_discovery() -> CardDefinition {
    CardDefinition {
        name: "Well of Discovery",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(no_untapped_lands()),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Well of Life — {4}. Two life each turn you end tapped out.
pub fn well_of_life() -> CardDefinition {
    CardDefinition {
        name: "Well of Life",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(no_untapped_lands()),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// Whip Sergeant — {2}{R} 2/1. Rents out haste.
pub fn whip_sergeant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Whip Sergeant",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            1,
        )
    }
}

/// Whipstitched Zombie — {1}{B} 2/2. Upkeep rent of {B}.
pub fn whipstitched_zombie() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[b()]) },
        }],
        ..creature("Whipstitched Zombie", cost(&[generic(1), b()]), vec![CreatureType::Zombie], 2, 2)
    }
}

/// Vitalizing Wind — {8}{G}. A team-wide +7/+7.
pub fn vitalizing_wind() -> CardDefinition {
    instant(
        "Vitalizing Wind",
        cost(&[generic(8), g()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::Const(7),
            toughness: Value::Const(7),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Wild Might — {1}{G}. +1/+1, and +4/+4 more unless someone buys it off.
pub fn wild_might() -> CardDefinition {
    instant(
        "Wild Might",
        cost(&[generic(1), g()]),
        Effect::Seq(vec![
            pump_target(1, 1),
            Effect::UnlessPlayerPays {
                who: PlayerRef::EachOpponent,
                cost: WardCost::Mana(cost(&[generic(2)])),
                then: Box::new(pump_target(4, 4)),
                if_paid: None,
            },
        ]),
    )
}

/// Withdraw — {U}{U}. A bounce, plus a second one they can buy off.
pub fn withdraw() -> CardDefinition {
    instant(
        "Withdraw",
        cost(&[u(), u()]),
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::UnlessPlayerPays {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(1))),
                cost: WardCost::Mana(cost(&[generic(1)])),
                then: Box::new(Effect::Move {
                    what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(1)))),
                }),
                if_paid: None,
            },
        ]),
    )
}

/// Steal Strength — {1}{B}. Moves a point of stats across the table.
pub fn steal_strength() -> CardDefinition {
    instant(
        "Steal Strength",
        cost(&[generic(1), b()]),
        Effect::Seq(vec![
            pump_target(1, 1),
            Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Stormwatch Eagle — {3}{U} 2/1 flier. A land buys it back.
pub fn stormwatch_eagle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![sac_land_ability(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
        })],
        ..creature("Stormwatch Eagle", cost(&[generic(3), u()]), vec![CreatureType::Bird], 2, 1)
    }
}

/// Trenching Steed — {3}{W} 2/3. Eats lands to survive combat.
pub fn trenching_steed() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![sac_land_ability(Effect::PumpPT {
            what: Selector::This,
            power: Value::ZERO,
            toughness: Value::Const(3),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Trenching Steed",
            cost(&[generic(3), w()]),
            vec![CreatureType::Horse, CreatureType::Rebel],
            2,
            3,
        )
    }
}

/// Troubled Healer — {2}{W} 1/2. Trades lands for prevention shields.
pub fn troubled_healer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![sac_land_ability(Effect::PreventNextDamage {
            target: target_any(),
            amount: Value::Const(2),
        })],
        ..creature(
            "Troubled Healer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Troublesome Spirit — {2}{U}{U} 3/4 flier that taps you out every turn.
pub fn troublesome_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Tap {
                what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
            },
        }],
        ..creature("Troublesome Spirit", cost(&[generic(2), u(), u()]), vec![CreatureType::Spirit], 3, 4)
    }
}

/// Sword Dancer — {1}{W} 1/2. Chips power off attackers.
pub fn sword_dancer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), w()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                power: Value::Const(-1),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Sword Dancer",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            1,
            2,
        )
    }
}

/// Squirrel Wrangler — {2}{G}{G} 2/2. Lands into Squirrels, or a Squirrel lord.
pub fn squirrel_wrangler() -> CardDefinition {
    let squirrels = ActivatedAbility {
        mana_cost: cost(&[generic(1), g()]),
        sac_other_filter: Some((R::Land, 1)),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: Box::new(TokenDefinition {
                name: "Squirrel".to_string(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![crate::mana::Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Squirrel],
                    ..Default::default()
                },
                ..Default::default()
            }),
        },
        ..Default::default()
    };
    let lord = ActivatedAbility {
        mana_cost: cost(&[generic(1), g()]),
        sac_other_filter: Some((R::Land, 1)),
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Squirrel)),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![squirrels, lord],
        ..creature(
            "Squirrel Wrangler",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// Thresher Beast — {3}{G}{G} 4/4. Blocking it costs a land.
pub fn thresher_beast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                count: Value::ONE,
                filter: R::Land,
            },
        }],
        ..creature("Thresher Beast", cost(&[generic(3), g(), g()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Thrive — {X}{G}. A +1/+1 counter on each of X creatures.
pub fn thrive() -> CardDefinition {
    sorcery(
        "Thrive",
        cost(&[x(), g()]),
        Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            }),
        },
    )
}

/// Alexi, Zephyr Mage — {3}{U}{U} 3/3 Spellshaper. Two cards bounce X creatures.
pub fn alexi_zephyr_mage() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), u()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 2)),
            effect: Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 1,
                    filter: R::Creature,
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    }),
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Alexi, Zephyr Mage",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            3,
            3,
        )
    }
}

/// Wing Storm — {2}{G}. Two damage per flier, to that flier's controller.
pub fn wing_storm() -> CardDefinition {
    sorcery(
        "Wing Storm",
        cost(&[generic(2), g()]),
        Effect::DealDamageToEachPlayerPerPermanent {
            filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
            amount: Value::Const(2),
            flat: false,
        },
    )
}

/// Wintermoon Mesa — a colorless tapland that trades itself for two taps.
pub fn wintermoon_mesa() -> CardDefinition {
    CardDefinition {
        name: "Wintermoon Mesa",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 2,
                    filter: R::Land,
                    effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Windscouter — {3}{U} 3/3 flier that goes home after every fight.
pub fn windscouter() -> CardDefinition {
    let bounce = Effect::AtEndOfCombat {
        body: Box::new(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
        }),
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: bounce.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: bounce,
            },
        ],
        ..creature(
            "Windscouter",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Scout],
            3,
            3,
        )
    }
}

/// Fickle Efreet — {3}{R} 5/2. Every fight risks handing it over.
pub fn fickle_efreet() -> CardDefinition {
    let flip = Effect::AtEndOfCombat {
        body: Box::new(Effect::FlipCoin {
            count: Value::ONE,
            on_heads: Box::new(Effect::Noop),
            on_tails: Box::new(Effect::GainControl {
                what: Selector::This,
                to: Some(PlayerRef::EachOpponent),
                duration: Duration::Permanent,
            }),
        }),
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: flip.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: flip,
            },
        ],
        ..creature("Fickle Efreet", cost(&[generic(3), r()]), vec![CreatureType::Efreet], 5, 2)
    }
}

/// Denying Wind — {7}{U}{U}. Strips seven cards out of a library.
pub fn denying_wind() -> CardDefinition {
    sorcery(
        "Denying Wind",
        cost(&[generic(7), u(), u()]),
        Effect::SearchUpToN {
            who: PlayerRef::Target(0),
            filter: R::Any,
            to: ZoneDest::Exile,
            count: Value::Const(7),
        },
    )
}

/// Forgotten Harvest — {1}{G}. Spent lands become +1/+1 counters.
pub fn forgotten_harvest() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayExileFromYourGraveyard {
                filter: R::Land,
                then: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..enchantment("Forgotten Harvest", cost(&[generic(1), g()]))
    }
}

/// Sunken Field — {1}{U}. The enchanted land taxes spells.
pub fn sunken_field() -> CardDefinition {
    land_aura(
        "Sunken Field",
        cost(&[generic(1), u()]),
        ActivatedAbility {
            tap_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(1)]),
                exile: false,
                extra_generic: None,
            },
            ..Default::default()
        },
    )
}

/// Verdant Field — {2}{G}. The enchanted land pumps.
pub fn verdant_field() -> CardDefinition {
    land_aura(
        "Verdant Field",
        cost(&[generic(2), g()]),
        ActivatedAbility { tap_cost: true, effect: pump_target(1, 1), ..Default::default() },
    )
}

/// Keldon Battlewagon — {5} 0/3 trampler that borrows power from your board.
pub fn keldon_battlewagon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::IsSource,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature.and(R::ControlledByYou)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::TappedForCostPower,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(
            "Keldon Battlewagon",
            cost(&[generic(5)]),
            vec![CreatureType::Juggernaut],
            0,
            3,
        )
    }
}

/// Brutal Suppression — {R}. Every Rebel activation now eats a land.
pub fn brutal_suppression() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Activated abilities of nontoken Rebels cost an additional \"Sacrifice a land\" to activate.",
            effect: StaticEffect::ActivationAdditionalSacrifice {
                filter: R::HasCreatureType(CreatureType::Rebel)
                    .and(R::Not(Box::new(R::IsToken))),
                sacrifice: R::Land,
            },
        }],
        ..enchantment("Brutal Suppression", cost(&[r()]))
    }
}

/// Celestial Convergence — {2}{W}{W}. Seven upkeeps, then the life leader wins.
pub fn celestial_convergence() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Omen, Value::Const(7))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Omen,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::ValueAtMost(
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Omen,
                        },
                        Value::ZERO,
                    ),
                    then: Box::new(Effect::HighestLifeWinsElseDraw),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..enchantment("Celestial Convergence", cost(&[generic(2), w(), w()]))
    }
}

/// Coffin Puppets — {3}{B}{B} 3/3. Two lands per upkeep buys it back.
pub fn coffin_puppets() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            sac_other_filter: Some((R::Land, 2)),
            condition: Some(Predicate::All(vec![
                Predicate::CurrentStepIs(TurnStep::Upkeep),
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasLandType(crate::card::LandType::Swamp).and(R::ControlledByYou),
                )),
            ])),
            effect: Effect::ReturnSelf,
            ..Default::default()
        }],
        ..creature("Coffin Puppets", cost(&[generic(3), b(), b()]), vec![CreatureType::Zombie], 3, 3)
    }
}

/// Dual Nature — {4}{G}{G}. Every creature arrives with a twin.
pub fn dual_nature() -> CardDefinition {
    let nontoken_creature = R::Creature.and(R::Not(Box::new(R::IsToken)));
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: nontoken_creature.clone(),
                    }),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::AnyPlayer,
                )
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: nontoken_creature,
                }),
                effect: Effect::ExileTokensSharingNameWith { what: Selector::TriggerSource },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::Exile { what: Selector::TokensCreatedBySource },
            },
        ],
        ..enchantment("Dual Nature", cost(&[generic(4), g(), g()]))
    }
}

/// Hollow Warrior — {4} 4/4. Every swing taps a spare creature.
pub fn hollow_warrior() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::AttackBlockCostTapAnother(Box::new(
            R::Creature.and(R::ControlledByYou),
        ))],
        ..creature(
            "Hollow Warrior",
            cost(&[generic(4)]),
            vec![CreatureType::Golem, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Infernal Genesis — {4}{B}{B}. Each upkeep mills one and pays it out in Minions.
pub fn infernal_genesis() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::ONE,
                },
                Effect::CreateToken {
                    who: PlayerRef::ActivePlayer,
                    count: Value::ManaValueOf(Box::new(Selector::LastMoved)),
                    definition: Box::new(TokenDefinition {
                        name: "Minion".to_string(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![crate::mana::Color::Black],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Minion],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
            ]),
        }],
        ..enchantment("Infernal Genesis", cost(&[generic(4), b(), b()]))
    }
}

/// Psychic Theft — {1}{U}. Borrows a spell out of their hand for a turn.
pub fn psychic_theft() -> CardDefinition {
    sorcery(
        "Psychic Theft",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::ExileChosenFromHand {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: R::Or(
                    Box::new(R::HasCardType(CardType::Instant)),
                    Box::new(R::HasCardType(CardType::Sorcery)),
                ),
                link_to_source: false,
                face_down: false,
            },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: crate::card::MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
            Effect::DelayUntilWithCapture {
                kind: crate::effect::DelayedTriggerKind::NextEndStep,
                capture: Selector::LastMoved,
                body: Box::new(Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::InExile,
                    },
                    then: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    }),
                    else_: Box::new(Effect::Noop),
                }),
            },
        ]),
    )
}

/// Search for Survivors — {2}{R}. A random graveyard card, revived or burned.
pub fn search_for_survivors() -> CardDefinition {
    sorcery(
        "Search for Survivors",
        cost(&[generic(2), r()]),
        Effect::RandomGraveyardCardToBattlefieldElse {
            who: PlayerRef::You,
            miss: ZoneDest::Exile,
        },
    )
}

/// Sheltering Prayers — {W}. Basic lands are untargetable while you're behind.
pub fn sheltering_prayers() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Basic lands each player controls have shroud as long as that player controls three or fewer lands.",
            effect: StaticEffect::GrantKeywordWhileControllerControlsAtMost {
                filter: R::Land.and(R::IsBasicLand),
                keyword: Keyword::Shroud,
                count_filter: R::Land,
                max: 3,
            },
        }],
        ..enchantment("Sheltering Prayers", cost(&[w()]))
    }
}

/// Shield Dancer — {2}{W} 1/3. Turns an attacker's swing back on itself.
pub fn shield_dancer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::RedirectNextDamageBackAtSource {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                to: Selector::This,
            },
            ..Default::default()
        }],
        ..creature(
            "Shield Dancer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            1,
            3,
        )
    }
}

/// Task Mage Assembly — {2}{R}. A ping anyone can fire, gone once the board empties.
pub fn task_mage_assembly() -> CardDefinition {
    CardDefinition {
        sacrifice_when: Some(Predicate::Not(Box::new(Predicate::SelectorExists(
            Selector::EachPermanent(R::Creature),
        )))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            any_player: true,
            sorcery_speed: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Task Mage Assembly", cost(&[generic(2), r()]))
    }
}
