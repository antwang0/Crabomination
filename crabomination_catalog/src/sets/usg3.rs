//! Urza's Saga (USG) gap closure, third wave. Tests in `classic_sets/usg3`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    MayPlayDuration, SelectionRequirement as R, StateTriggeredAbility, StaticAbility, StaticEffect,
    Subtypes,
    TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector,
    ZoneDest,
    shortcut::{etb, target_filtered},
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

/// "At the beginning of your upkeep, you may put a verse counter on this."
fn verse_upkeep() -> TriggeredAbility {
    your_upkeep(Effect::MayDo {
        description: "Put a verse counter on this".into(),
        body: Box::new(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Verse,
            amount: Value::ONE,
        }),
    })
}

fn verses() -> Value {
    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Verse }
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
                        if_paid: None,
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
            return_permanent_cost: Some((R::IsSource, 1)),
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
                return_eot: false,
                then: None,
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
                if_paid: None,
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

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// A Rune of Protection — {1}{W}. "{W}: The next time a [filter] source of
/// your choice would deal damage to you this turn, prevent that damage."
fn rune(name: &'static str, filter: R) -> CardDefinition {
    CardDefinition {
        keywords: vec![cycling_two()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::PreventNextDamageFromChosenSource {
                filter,
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
            },
            ..Default::default()
        }],
        ..enchantment(name, cost(&[generic(1), w()]))
    }
}

pub fn rune_of_protection_white() -> CardDefinition {
    rune("Rune of Protection: White", R::HasColor(Color::White))
}
pub fn rune_of_protection_blue() -> CardDefinition {
    rune("Rune of Protection: Blue", R::HasColor(Color::Blue))
}
pub fn rune_of_protection_green() -> CardDefinition {
    rune("Rune of Protection: Green", R::HasColor(Color::Green))
}
pub fn rune_of_protection_artifacts() -> CardDefinition {
    rune("Rune of Protection: Artifacts", R::Artifact)
}
pub fn rune_of_protection_lands() -> CardDefinition {
    rune("Rune of Protection: Lands", R::Land)
}

/// Electryte — {3}{R}{R}. Getting through means every blocker eats 3.
pub fn electryte() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::DealDamageEqualToPowerToEach {
                source: Selector::This,
                targets: Selector::BlockingCreatures,
                each_opponent: false,
            },
        }],
        ..creature(
            "Electryte",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Trilobite, CreatureType::Beast],
            3,
            3,
        )
    }
}

/// No Rest for the Wicked — {1}{B}. Sacrifice it to take back everything that
/// died this turn.
pub fn no_rest_for_the_wicked() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::ReturnGraveyardCardsToHand {
                filter: R::And(
                    Box::new(R::Creature),
                    Box::new(R::PutIntoGraveyardFromBattlefieldThisTurn),
                ),
                max: Value::Const(99),
            },
            ..Default::default()
        }],
        ..enchantment("No Rest for the Wicked", cost(&[generic(1), b()]))
    }
}

/// Argothian Wurm — {3}{G}. A 6/6 trampler anyone can bounce for a land.
pub fn argothian_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::PlayersMayAccept {
            who: PlayerRef::EachPlayer,
            description: "Sacrifice a land to put Argothian Wurm on top of its owner's library?"
                .into(),
            on_accept: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::Target(0),
                    count: Value::ONE,
                    filter: R::Land,
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::This)),
                        pos: LibraryPosition::Top,
                    },
                },
            ])),
            otherwise: Box::new(Effect::Noop),
        })],
        ..creature("Argothian Wurm", cost(&[generic(3), g()]), vec![CreatureType::Wurm], 6, 6)
    }
}

/// Lifeline — {5}. Nothing stays dead while another creature is out.
pub fn lifeline() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::SelectorExists(Selector::EachPermanent(R::Creature)),
            ),
            effect: Effect::DelayUntilWithCapture {
                kind: DelayedTriggerKind::NextEndStep,
                capture: Selector::TriggerSource,
                body: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOfMoved,
                        tapped: false,
                    },
                }),
            },
        }],
        ..artifact("Lifeline", cost(&[generic(5)]))
    }
}

/// Persecute — {2}{B}{B}. Name a color and strip it out of a hand.
pub fn persecute() -> CardDefinition {
    sorcery(
        "Persecute",
        cost(&[generic(2), b(), b()]),
        Effect::ChooseColorThenDiscardMatching { who: PlayerRef::Target(0) },
    )
}

/// Phyrexian Processor — {4}. Pay life on the way in; mint that big a Minion
/// every turn.
pub fn phyrexian_processor() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::Seq(vec![
            Effect::ChooseNumberForSource { max: 20 },
            Effect::LoseLife { who: Selector::You, amount: Value::ChosenNumberOfSource },
        ])),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Phyrexian Minion".into(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Phyrexian, CreatureType::Minion],
                        ..Default::default()
                    },
                    // The life paid as this entered, baked at mint time.
                    dynamic_pt: Some((
                        Value::ChosenNumberOfSource,
                        Value::ChosenNumberOfSource,
                    )),
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..artifact("Phyrexian Processor", cost(&[generic(4)]))
    }
}

/// Carpet of Flowers — {G}. Each of your main phases, cash in an opponent's
/// Islands for mana.
pub fn carpet_of_flowers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Add mana for each Island that opponent controls".into(),
                body: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::CountMatching {
                        sel: Box::new(Selector::ControlledBy {
                            who: PlayerRef::Target(0),
                            filter: R::Any,
                        }),
                        filter: R::HasLandType(LandType::Island),
                    }),
                }),
            },
        }],
        ..enchantment("Carpet of Flowers", cost(&[g()]))
    }
}

/// Remembrance — {3}{W}. Every creature that dies fetches its twin.
pub fn remembrance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken,
                }),
            effect: Effect::MayDo {
                description: "Search your library for a card with that name".into(),
                body: Box::new(Effect::SearchSameNameAs {
                    who: PlayerRef::You,
                    subject: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::You),
                    count: None,
                }),
            },
        }],
        ..enchantment("Remembrance", cost(&[generic(3), w()]))
    }
}

/// Sporogenesis — {3}{G}. Seed creatures with fungus counters; they bloom into
/// Saprolings when they die.
pub fn sporogenesis() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            your_upkeep(Effect::MayDo {
                description: "Put a fungus counter on target nontoken creature".into(),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::And(Box::new(R::Creature), Box::new(R::NotToken))),
                    kind: CounterType::Fungus,
                    amount: Value::ONE,
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::WithCounter(CounterType::Fungus),
                    },
                ),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::CountersOn {
                        what: Box::new(Selector::TriggerSource),
                        kind: CounterType::Fungus,
                    },
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
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::RemoveCounter {
                    what: Selector::EachPermanent(R::WithCounter(CounterType::Fungus)),
                    kind: CounterType::Fungus,
                    amount: Value::Const(99),
                },
            },
        ],
        ..enchantment("Sporogenesis", cost(&[generic(3), g()]))
    }
}

/// Serra's Hymn — {W}. Bank verse counters, then cash them in as a shield
/// split across any number of targets.
pub fn serras_hymn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![verse_upkeep()],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PreventNextDamageDivided {
                total: verses(),
                filter: R::Creature.or(R::Player).or(R::Planeswalker),
                max_targets: 4,
            },
            ..Default::default()
        }],
        ..enchantment("Serra's Hymn", cost(&[w()]))
    }
}

/// Discordant Dirge — {3}{B}{B}. Verse counters buy that many cards out of an
/// opponent's hand.
pub fn discordant_dirge() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![verse_upkeep()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::LookAtHand { who: Selector::Player(PlayerRef::Target(0)) },
                Effect::DiscardChosen {
                    from: Selector::Player(PlayerRef::Target(0)),
                    count: verses(),
                    filter: R::Any,
                },
            ]),
            ..Default::default()
        }],
        ..enchantment("Discordant Dirge", cost(&[generic(3), b(), b()]))
    }
}

/// Abundance — {2}{G}{G}. Trade every draw for the kind of card you want.
pub fn abundance() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would draw, you may instead reveal until a land/nonland.",
            effect: StaticEffect::MayReplaceDrawWithRevealUntilKind,
        }],
        ..enchantment("Abundance", cost(&[generic(2), g(), g()]))
    }
}

/// Academy Researchers — {1}{U}{U}. Brings an Aura out of hand with it.
pub fn academy_researchers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::PutAuraFromHandAttachedTo {
            host: Selector::This,
        })],
        ..creature(
            "Academy Researchers",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Defensive Formation — {W}. You, not the attacker, split their damage.
pub fn defensive_formation() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You assign the combat damage of each creature attacking you.",
            effect: StaticEffect::ControllerAssignsAttackersCombatDamage,
        }],
        ..enchantment("Defensive Formation", cost(&[w()]))
    }
}

/// Temporal Aperture — {2}. Shuffle up and cast the new top card for free.
/// The permission rides the card, so moving it off the top doesn't revoke it.
pub fn temporal_aperture() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::ShuffleLibrary { who: PlayerRef::You },
                Effect::RevealTopCard { who: PlayerRef::You },
                Effect::GrantMayPlay {
                    what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: false,
                    any_color: false,
                },
            ]),
            ..Default::default()
        }],
        ..artifact("Temporal Aperture", cost(&[generic(2)]))
    }
}

/// Diabolic Servitude — {3}{B}. Rents a creature out of your graveyard; the
/// two of them leave together.
pub fn diabolic_servitude() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::And(
                        Box::new(R::Creature),
                        Box::new(R::InYourGraveyard),
                    )),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOfMoved,
                        tapped: false,
                    },
                },
                Effect::RememberPermanentOnSource { what: Selector::LastMoved },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                    .with_filter(Predicate::TriggerSourceIsSourcesChosenPermanent),
                effect: Effect::Seq(vec![
                    Effect::Exile { what: Selector::TriggerSource },
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::Exile { what: Selector::ChosenPermanentOfSource },
            },
        ],
        ..enchantment("Diabolic Servitude", cost(&[generic(3), b()]))
    }
}
