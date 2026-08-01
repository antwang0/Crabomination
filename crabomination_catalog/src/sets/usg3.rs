//! Urza's Saga (USG) gap closure, third wave. Tests in `classic_sets/usg3`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    StateTriggeredAbility, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest, shortcut::target_filtered,
};
use crate::game::TurnStep;
use crate::mana::{Color, SpendRestriction, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: crate::mana::ManaCost,
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

fn artifact(name: &'static str, c: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, c: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn instant(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

fn cycling_two() -> Keyword {
    Keyword::Cycling(cost(&[generic(2)]))
}

/// "At the beginning of each player's upkeep, [effect]." The upkeep is always
/// the active player's, so bodies read `PlayerRef::ActivePlayer`.
fn each_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
        effect,
    }
}

fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

// ── Statics ─────────────────────────────────────────────────────────────────

/// Telepathy — {U}. Your opponents play with their hands revealed.
pub fn telepathy() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Your opponents play with their hands revealed.",
            effect: StaticEffect::OpponentsPlayWithHandsRevealed,
        }],
        ..enchantment("Telepathy", cost(&[u()]))
    }
}

/// Fluctuator — {2}. Cycling abilities you activate cost {2} less.
pub fn fluctuator() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Cycling abilities you activate cost {2} less to activate.",
            effect: StaticEffect::CyclingCostReduction(2),
        }],
        ..artifact("Fluctuator", cost(&[generic(2)]))
    }
}

/// Sulfuric Vapors — {3}{R}. Every red spell's damage comes in one heavier.
pub fn sulfuric_vapors() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a red spell would deal damage, it deals that much plus 1 instead.",
            effect: StaticEffect::AddDamageFromColorSpells { color: Color::Red, amount: 1 },
        }],
        ..enchantment("Sulfuric Vapors", cost(&[generic(3), r()]))
    }
}

/// Contamination — {3}{B}. Every land taps for {B}; feed it a creature a turn.
pub fn contamination() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![your_upkeep(Effect::SacrificeSourceUnlessSacrifice {
            filter: R::Creature,
        })],
        static_abilities: vec![StaticAbility {
            description: "If a land is tapped for mana, it produces {B} instead.",
            effect: StaticEffect::LandsProduceColorInstead(Color::Black),
        }],
        ..enchantment("Contamination", cost(&[generic(2), b()]))
    }
}

/// Energy Field — {1}{U}. Nothing an opponent controls can touch you — until
/// the first card hits your graveyard.
pub fn energy_field() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::YourControl),
            effect: Effect::SacrificeSource,
        }],
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage sources you don't control would deal to you.",
            effect: StaticEffect::PreventAllDamageToControllerFromOthersSources,
        }],
        ..enchantment("Energy Field", cost(&[generic(1), u()]))
    }
}

// ── The state-triggered half of the Hidden / Veiled cycles (CR 603.8) ───────

/// "When [condition], if this permanent is an enchantment, it becomes a P/T
/// creature." The animation replaces the enchantment type outright.
fn wakes_when(
    condition: Predicate,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> StateTriggeredAbility {
    StateTriggeredAbility {
        condition: Predicate::All(vec![
            condition,
            Predicate::EntityMatches { what: Selector::This, filter: R::Enchantment },
        ]),
        effect: Effect::BecomeCreatureLosingTypes {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            creature_types: types,
            keywords,
        },
    }
}

/// Hidden Predators — {G}. Wakes as a 4/4 Beast once an opponent fields a
/// 4-power creature.
pub fn hidden_predators() -> CardDefinition {
    CardDefinition {
        state_trigger: Some(wakes_when(
            Predicate::SelectorExists(Selector::EachPermanent(R::And(
                Box::new(R::ControlledByOpponent),
                Box::new(R::And(Box::new(R::Creature), Box::new(R::PowerAtLeast(4)))),
            ))),
            4,
            4,
            vec![CreatureType::Beast],
            vec![],
        )),
        ..enchantment("Hidden Predators", cost(&[g()]))
    }
}

/// Veiled Crocodile — {2}{U}. Wakes as a 4/4 the moment anyone empties their
/// hand.
pub fn veiled_crocodile() -> CardDefinition {
    CardDefinition {
        state_trigger: Some(wakes_when(
            Predicate::Any(vec![
                Predicate::ValueAtMost(
                    Value::HandSizeOf(PlayerRef::You),
                    Value::Const(0),
                ),
                Predicate::ValueAtMost(
                    Value::HandSizeOf(PlayerRef::EachOpponent),
                    Value::Const(0),
                ),
            ]),
            4,
            4,
            vec![CreatureType::Crocodile],
            vec![],
        )),
        ..enchantment("Veiled Crocodile", cost(&[generic(2), u()]))
    }
}

/// Veiled Apparition — {1}{U}. An opponent's spell wakes a 3/3 flier that
/// wants {1}{U} an upkeep.
pub fn veiled_apparition() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::This, filter: R::Enchantment },
            ),
            effect: Effect::Seq(vec![
                Effect::BecomeCreatureLosingTypes {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![CreatureType::Illusion],
                    keywords: vec![Keyword::Flying],
                },
                Effect::GrantTriggeredAbility {
                    what: Selector::This,
                    trigger: Box::new(your_upkeep(Effect::UnlessPlayerPays {
                        who: PlayerRef::You,
                        cost: WardCost::Mana(cost(&[generic(1), u()])),
                        then: Box::new(Effect::SacrificeSource),
                    })),
                    duration: Duration::Permanent,
                },
            ]),
        }],
        ..enchantment("Veiled Apparition", cost(&[generic(1), u()]))
    }
}

// ── Combat ──────────────────────────────────────────────────────────────────

/// Okk — {1}{R}. A 4/4 that only moves when something bigger moves with it.
pub fn okk() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CantAttackUnlessGreaterPowerAttacks,
            Keyword::CantBlockUnlessGreaterPowerBlocks,
        ],
        ..creature("Okk", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 4, 4)
    }
}

/// Outmaneuver — {X}{R}. X blocked creatures hit the player anyway.
pub fn outmaneuver() -> CardDefinition {
    instant(
        "Outmaneuver",
        cost(&[x(), r()]),
        Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                min_targets: 1,
                max_targets: 8,
                filter: R::IsBlocked,
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::AssignsDamageAsThoughUnblocked,
                    duration: Duration::EndOfTurn,
                }),
            }),
        },
    )
}

/// Waylay — {2}{W}. Three Knights for one combat; they vanish at cleanup.
pub fn waylay() -> CardDefinition {
    instant(
        "Waylay",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: TokenDefinition {
                    name: "Knight".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Knight],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            Effect::ExileLastCreatedTokensAtNextCleanup,
        ]),
    )
}

// ── Upkeep engines ──────────────────────────────────────────────────────────

/// Umbilicus — {4}. Two life an upkeep, or something goes back to hand.
pub fn umbilicus() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![each_upkeep(
            Effect::PlayerReturnsPermanentUnlessPaysLife {
                who: PlayerRef::ActivePlayer,
                life: 2,
            },
        )],
        ..artifact("Umbilicus", cost(&[generic(4)]))
    }
}

/// Noetic Scales — {4}. Creatures bigger than their controller's hand bounce.
pub fn noetic_scales() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![each_upkeep(
            Effect::ReturnCreaturesWithPowerGreaterThanHand { who: PlayerRef::ActivePlayer },
        )],
        ..artifact("Noetic Scales", cost(&[generic(4)]))
    }
}

/// Purging Scythe — {5}. Two damage an upkeep to the flimsiest creature out.
pub fn purging_scythe() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![your_upkeep(Effect::DealDamage {
            to: Selector::LeastToughnessAmongAll,
            amount: Value::Const(2),
        })],
        ..artifact("Purging Scythe", cost(&[generic(5)]))
    }
}

/// Thran Turbine — {1}. Two colorless an upkeep, abilities only.
pub fn thran_turbine() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![your_upkeep(Effect::MayDo {
            description: "Add {C}{C} (spend only on abilities)".into(),
            body: Box::new(Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::Const(2))),
                    SpendRestriction::AbilitiesOnly,
                ),
            }),
        })],
        ..artifact("Thran Turbine", cost(&[generic(1)]))
    }
}

/// Wild Dogs — {G}. A 2/1 that keeps defecting to whoever's ahead on life.
pub fn wild_dogs() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        triggered_abilities: vec![your_upkeep(Effect::If {
            cond: Predicate::PlayerHasMostLife { who: PlayerRef::HighestLife },
            then: Box::new(Effect::GainControl {
                what: Selector::This,
                to: Some(PlayerRef::HighestLife),
                duration: Duration::Permanent,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Wild Dogs", cost(&[g()]), vec![CreatureType::Dog], 2, 1)
    }
}

/// Greener Pastures — {2}{G}. The land leader gets a Saproling each upkeep.
pub fn greener_pastures() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::PlayerControlsMostOf {
                    who: PlayerRef::ActivePlayer,
                    filter: R::Land,
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::ActivePlayer,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Saproling".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Saproling],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..enchantment("Greener Pastures", cost(&[generic(2), g()]))
    }
}

/// Antagonism — {3}{R}. End your turn without drawing blood and take 2.
pub fn antagonism() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::PlayerDamagedThisTurn {
                    who: PlayerRef::EachPlayerExceptControllerOf(Box::new(Selector::Player(
                        PlayerRef::ActivePlayer,
                    ))),
                }))),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(2),
            },
        }],
        ..enchantment("Antagonism", cost(&[generic(3), r()]))
    }
}

// ── Activations ─────────────────────────────────────────────────────────────

/// Attunement — {2}{U}. Bounce it to churn four cards into the graveyard.
pub fn attunement() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            return_permanent_cost: Some(R::IsSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(4),
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..enchantment("Attunement", cost(&[generic(2), u()]))
    }
}

/// Copper Gnomes — {2}. Sacrifice it to drop an artifact from hand for free.
pub fn copper_gnomes() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            sac_cost: true,
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Artifact,
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
            },
            ..Default::default()
        }],
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature("Copper Gnomes", cost(&[generic(2)]), vec![CreatureType::Gnome], 1, 1)
    }
}

/// Viashino Sandswimmer — {2}{R}{R}. A 3/2 that gambles its way out of trouble.
pub fn viashino_sandswimmer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::ReturnSelf),
                on_tails: Box::new(Effect::SacrificeSource),
            },
            ..Default::default()
        }],
        ..creature("Viashino Sandswimmer", cost(&[generic(2), r(), r()]), vec![CreatureType::Lizard], 3, 2)
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Pendrell Flux — {1}{U}. The enchanted creature pays its own cost every
/// upkeep or dies.
pub fn pendrell_flux() -> CardDefinition {
    CardDefinition {
        name: "Pendrell Flux",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![your_upkeep(Effect::SacrificeSourceUnlessPayManaValue)],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Power Taint — {1}{U}. The enchanted enchantment taxes its controller 2 life
/// an upkeep.
pub fn power_taint() -> CardDefinition {
    CardDefinition {
        name: "Power Taint",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![cycling_two()],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Enchantment) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::IsTurnOf(PlayerRef::ControllerOf(Box::new(
                    Selector::AttachedTo(Box::new(Selector::This)),
                )))),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::ControllerOf(Box::new(Selector::AttachedTo(Box::new(
                    Selector::This,
                )))),
                cost: WardCost::Mana(cost(&[generic(2)])),
                then: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::AttachedTo(
                        Box::new(Selector::This),
                    )))),
                    amount: Value::Const(2),
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Sorceries / instants ────────────────────────────────────────────────────

/// Yawgmoth's Will — {2}{B}. Your graveyard is a second hand for a turn, and
/// everything that dies afterwards is exiled.
pub fn yawgmoths_will() -> CardDefinition {
    sorcery(
        "Yawgmoth's Will",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::PlayFromGraveyardThisTurn,
            Effect::ExileYourGraveyardBoundThisTurn,
        ]),
    )
}

/// Planar Birth — {1}{W}. Every basic land in every graveyard comes back.
pub fn planar_birth() -> CardDefinition {
    sorcery(
        "Planar Birth",
        cost(&[generic(1), w()]),
        Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::EachPlayer,
                zone: Zone::Graveyard,
                filter: R::IsBasicLand,
            },
            to: ZoneDest::Battlefield {
                controller: PlayerRef::OwnerOfMoved,
                tapped: true,
            },
        },
    )
}

/// Brand — {R}. Take back everything that was stolen from you.
pub fn brand() -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        ..instant(
            "Brand",
            cost(&[r()]),
            Effect::GainControl {
                what: Selector::EachPermanent(R::OwnedByYou),
                to: Some(PlayerRef::You),
                duration: Duration::Permanent,
            },
        )
    }
}

/// Victimize — {2}{B}. Trade one creature for two out of the graveyard.
pub fn victimize() -> CardDefinition {
    sorcery(
        "Victimize",
        cost(&[generic(2), b()]),
        Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(R::And(
                Box::new(R::Creature),
                Box::new(R::ControlledByYou),
            ))),
            then: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature,
                },
                Effect::ApplyToTargets {
                    min_targets: 2,
                    max_targets: 2,
                    filter: R::And(Box::new(R::Creature), Box::new(R::InYourGraveyard)),
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOfMoved,
                            tapped: true,
                        },
                    }),
                },
            ])),
            else_: Box::new(Effect::Noop),
        },
    )
}

/// Ill-Gotten Gains — {2}{B}{B}. Everyone pitches their hand and buys back
/// three.
pub fn ill_gotten_gains() -> CardDefinition {
    sorcery(
        "Ill-Gotten Gains",
        cost(&[generic(2), b(), b()]),
        Effect::Seq(vec![
            Effect::ExileResolvingSpell,
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::HandSizeOf(PlayerRef::EachPlayer),
                random: false,
            },
            Effect::ReturnGraveyardCardsToHand { filter: R::Any, max: Value::Const(3) },
        ]),
    )
}

/// Time Spiral — {4}{U}{U}. Everyone refuels; you untap six lands to do it
/// again.
pub fn time_spiral() -> CardDefinition {
    sorcery(
        "Time Spiral",
        cost(&[generic(4), u(), u()]),
        Effect::Seq(vec![
            Effect::ExileResolvingSpell,
            Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
            Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(7),
            },
            Effect::Untap {
                up_to: Some(Value::Const(6)),
                what: Selector::EachPermanent(R::And(
                    Box::new(R::Land),
                    Box::new(R::And(Box::new(R::ControlledByYou), Box::new(R::Tapped))),
                )),
            },
        ]),
    )
}
