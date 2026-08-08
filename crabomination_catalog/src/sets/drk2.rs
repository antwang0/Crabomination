//! The Dark (DRK) — closing wave. Tests in `classic_sets/drk`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility, WardCost,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::{target_any, target_filtered},
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

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

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
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

fn land(name: &'static str, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: abilities,
        ..Default::default()
    }
}

/// An Aura that enchants `what` and grants `bonus`.
fn aura(name: &'static str, c: ManaCost, what: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(what) },
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

/// The permanent this Aura is attached to.
fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// "At the beginning of the upkeep of enchanted [permanent]'s controller, …"
fn host_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
            .with_filter(Predicate::IsTurnOf(PlayerRef::ControllerOf(Box::new(host())))),
        effect,
    }
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Banshee — {X}, {T} splits its damage between any target and you.
pub fn banshee() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_any(),
                    amount: Value::HalvedRoundDown(Box::new(Value::XFromCost)),
                },
                Effect::DealDamage {
                    to: Selector::You,
                    amount: Value::HalvedRoundUp(Box::new(Value::XFromCost)),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Banshee", cost(&[generic(2), b(), b()]), vec![CreatureType::Spirit], 0, 1)
    }
}

/// Eater of the Dead — untaps itself by eating a creature card from a graveyard.
pub fn eater_of_the_dead() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            condition: Some(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::Tapped,
            }),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InGraveyard)),
                    to: ZoneDest::Exile,
                },
                Effect::Untap { what: Selector::This, up_to: None },
            ]),
            ..Default::default()
        }],
        ..creature("Eater of the Dead", cost(&[generic(4), b()]), vec![CreatureType::Horror], 3, 4)
    }
}

/// Giant Shark — needs an Island to swing, feeds on wounded blockers, and
/// dies once you control no Islands.
pub fn giant_shark() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackUnlessDefenderControlsLandType(LandType::Island)],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::BlockedAttacker,
                        filter: R::DealtDamageThisTurn,
                    }),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(2),
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeywords {
                        what: Selector::This,
                        keywords: vec![Keyword::Trample],
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::BlockingCreatures,
                        filter: R::DealtDamageThisTurn,
                    }),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(2),
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeywords {
                        what: Selector::This,
                        keywords: vec![Keyword::Trample],
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
        ],
        state_trigger: Some(crate::effect::StateTriggeredAbility {
            condition: Predicate::Not(Box::new(Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Island),
            }))),
            effect: Effect::SacrificeSource,
        }),
        ..creature("Giant Shark", cost(&[generic(5), u()]), vec![CreatureType::Shark], 4, 4)
    }
}

/// Goblin Wizard — deploys Goblins off the top of your hand and shields them
/// from white.
pub fn goblin_wizard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Goblin),
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::GrantKeywords {
                    what: target_filtered(R::HasCreatureType(CreatureType::Goblin)),
                    keywords: vec![Keyword::Protection(Color::White)],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Goblin Wizard",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Goblin, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Orc General — eats a lesser green-skin to pump the rest of the warband.
pub fn orc_general() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((
                R::HasCreatureType(CreatureType::Orc)
                    .or(R::HasCreatureType(CreatureType::Goblin)),
                1,
            )),
            effect: Effect::PumpPT {
                what: Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Orc).and(R::OtherThanSource),
                },
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Orc General",
            cost(&[generic(2), r()]),
            vec![CreatureType::Orc, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Rag Man — sorcery-speed hand attack that rips a creature at random.
pub fn rag_man() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b(), b()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::RevealHand { who: PlayerRef::Target(0) },
                Effect::DiscardMatchingAtRandom {
                    who: PlayerRef::Target(0),
                    filter: R::Creature,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Rag Man",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Human, CreatureType::Minion],
            2,
            1,
        )
    }
}

/// Scarwood Hag — hands forestwalk out, or takes it away.
pub fn scarwood_hag() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g(), g(), g(), g()]),
                tap_cost: true,
                effect: Effect::GrantKeywords {
                    what: target_filtered(R::Creature),
                    keywords: vec![Keyword::Landwalk(LandType::Forest)],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::LoseKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Landwalk(LandType::Forest),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature("Scarwood Hag", cost(&[generic(1), g()]), vec![CreatureType::Hag], 1, 1)
    }
}

/// Spitting Slug — either it strikes first, or everyone in the fight does.
pub fn spitting_slug() -> CardDefinition {
    let body = || Effect::MayPay {
        description: "Pay {1}{G} for first strike?".into(),
        mana_cost: cost(&[generic(1), g()]),
        body: Box::new(Effect::GrantKeywords {
            what: Selector::This,
            keywords: vec![Keyword::FirstStrike],
            duration: Duration::EndOfTurn,
        }),
        else_: Some(Box::new(Effect::GrantKeywords {
            what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
            keywords: vec![Keyword::FirstStrike],
            duration: Duration::EndOfTurn,
        })),
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: body(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: body(),
            },
        ],
        ..creature("Spitting Slug", cost(&[generic(1), g(), g()]), vec![CreatureType::Slug], 2, 4)
    }
}

/// Tracker — a paid fight that always trades damage both ways.
pub fn tracker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Creature),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
                Effect::DealDamage {
                    to: Selector::This,
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Tracker", cost(&[generic(2), g()]), vec![CreatureType::Human], 2, 2)
    }
}

/// Whippoorwill — marks a creature for a death nothing undoes.
pub fn whippoorwill() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::CantBeRegeneratedThisTurn { what: target_filtered(R::Creature) },
                Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..creature("Whippoorwill", cost(&[g()]), vec![CreatureType::Bird], 1, 1)
    }
}

// ── Artifacts ──────────────────────────────────────────────────────────────

/// Barl's Cage — {3} keeps a creature down through its next untap step.
pub fn barls_cage() -> CardDefinition {
    artifact(
        "Barl's Cage",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::SkipNextUntap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
    )
}

/// Living Armor — cashes itself in for toughness scaled to the target's cost.
pub fn living_armor() -> CardDefinition {
    artifact(
        "Living Armor",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusZeroPlusOne,
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
            ..Default::default()
        }],
    )
}

/// Necropolis — a Wall that grows on the corpses in your graveyard.
pub fn necropolis() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            exile_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusZeroPlusOne,
                amount: Value::TotalManaValueOf(Box::new(Selector::CostExiledCards)),
            },
            ..Default::default()
        }],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        ..CardDefinition { name: "Necropolis", cost: cost(&[generic(5)]), ..Default::default() }
    }
}

/// War Barge — lends islandwalk, and the loan is called in if the Barge goes.
pub fn war_barge() -> CardDefinition {
    artifact(
        "War Barge",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Seq(vec![
                Effect::GrantKeywords {
                    what: target_filtered(R::Creature),
                    keywords: vec![Keyword::Landwalk(LandType::Island)],
                    duration: Duration::EndOfTurn,
                },
                Effect::RememberPermanentOnSource { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
    )
}

// ── Lands ──────────────────────────────────────────────────────────────────

/// City of Shadows — banks a creature a turn into permanent colorless mana.
pub fn city_of_shadows() -> CardDefinition {
    land(
        "City of Shadows",
        vec![
            ActivatedAbility {
                tap_cost: true,
                exile_permanent_cost: Some((R::Creature.and(R::ControlledByYou), 1)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Storage,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Storage,
                    }),
                },
                ..Default::default()
            },
        ],
    )
}

/// Safe Haven — blinks your creatures out of reach and gives them all back at
/// once when it goes.
pub fn safe_haven() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![upkeep(Effect::MaySacrificeSource {
            description: "Sacrifice Safe Haven to return the exiled creatures?".into(),
            then: Box::new(Effect::ReturnExiledBySourceToBattlefield { decayed: false, count: None }),
            else_: None,
        })],
        ..land(
            "Safe Haven",
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::ExileWithSource {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                },
                ..Default::default()
            }],
        )
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Eternal Flame — every Mountain you control burns an opponent, and half of
/// it burns you.
pub fn eternal_flame() -> CardDefinition {
    let mountains = || {
        Value::count(Selector::ControlledBy {
            who: PlayerRef::You,
            filter: R::HasLandType(LandType::Mountain),
        })
    };
    sorcery(
        "Eternal Flame",
        cost(&[generic(2), r(), r()]),
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: mountains(),
            },
            Effect::DealDamage {
                to: Selector::You,
                amount: Value::HalvedRoundUp(Box::new(mountains())),
            },
        ]),
    )
}

/// Inquisition — a hand-reading that punishes white decks.
pub fn inquisition() -> CardDefinition {
    sorcery(
        "Inquisition",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::RevealHand { who: PlayerRef::Target(0) },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::Target(0)),
                amount: Value::CardsInHandMatching {
                    who: PlayerRef::Target(0),
                    filter: R::HasColor(Color::White),
                },
            },
        ]),
    )
}

/// Martyr's Cry — exiles the white board and pays everyone for the loss.
pub fn martyrs_cry() -> CardDefinition {
    sorcery(
        "Martyr's Cry",
        cost(&[w(), w()]),
        Effect::ExileEachMatchingThenControllerDraws {
            filter: R::Creature.and(R::HasColor(Color::White)),
        },
    )
}

/// Word of Binding — taps X target creatures.
pub fn word_of_binding() -> CardDefinition {
    sorcery(
        "Word of Binding",
        cost(&[x(), b(), b()]),
        Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
            }),
        },
    )
}

/// Flood — a repeatable tapper that grounded creatures can't dodge.
pub fn flood() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            effect: Effect::Tap {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasKeyword(
                    Keyword::Flying,
                ))))),
            },
            ..Default::default()
        }],
        ..enchantment("Flood", cost(&[u()]))
    }
}

/// Gaea's Touch — a free extra Forest each turn, or two green mana on the way
/// out.
pub fn gaeas_touch() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                sorcery_speed: true,
                once_per_turn: true,
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand.and(R::HasLandType(LandType::Forest)),
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Green, Color::Green]),
                },
                ..Default::default()
            },
        ],
        ..enchantment("Gaea's Touch", cost(&[g(), g()]))
    }
}

/// Venom — whatever the enchanted creature meets in combat dies with it.
pub fn venom() -> CardDefinition {
    let body = || Effect::AtEndOfCombat {
        body: Box::new(Effect::Destroy {
            what: Selector::CreaturesInCombatWith(Box::new(host())),
        }),
    };
    let not_wall = |what: Selector| {
        Predicate::Not(Box::new(Predicate::EntityMatches {
            what,
            filter: R::HasCreatureType(CreatureType::Wall),
        }))
    };
    aura(
        "Venom",
        cost(&[generic(1), g(), g()]),
        R::Creature,
        EquipBonus {
            triggered_abilities: vec![
                TriggeredAbility {
                    event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource)
                        .with_filter(not_wall(Selector::BlockedAttacker)),
                    effect: body(),
                },
                TriggeredAbility {
                    event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource)
                        .with_filter(not_wall(Selector::BlockingCreatures)),
                    effect: body(),
                },
            ],
            triggers_on_equipment: true,
            ..Default::default()
        },
    )
}

/// Curse Artifact — the enchanted artifact's controller feeds it or bleeds.
pub fn curse_artifact() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![host_upkeep(Effect::UnlessPlayerPays {
            who: PlayerRef::ControllerOf(Box::new(host())),
            cost: WardCost::SacrificeAttachedHost,
            then: Box::new(Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(host()))),
                amount: Value::Const(2),
            }),
            if_paid: None,
        })],
        ..aura("Curse Artifact", cost(&[generic(2), b(), b()]), R::Artifact, EquipBonus::default())
    }
}

/// Erosion — the enchanted land's controller pays every upkeep or loses it.
pub fn erosion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![host_upkeep(Effect::UnlessPlayerPays {
            who: PlayerRef::ControllerOf(Box::new(host())),
            cost: WardCost::ManaOrLife(cost(&[generic(1)]), 1),
            then: Box::new(Effect::Destroy { what: host() }),
            if_paid: None,
        })],
        ..aura("Erosion", cost(&[u(), u(), u()]), R::Land, EquipBonus::default())
    }
}

/// Angry Mob — during your turn it's 2 plus every Swamp your opponents hold.
pub fn angry_mob() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(crate::card::DynamicPt::OnlyDuringYourTurn {
            inner: Box::new(crate::card::DynamicPt::BasePlusOpponentsMatching {
                base_p: 2,
                base_t: 2,
                filter: Box::new(R::HasLandType(LandType::Swamp)),
            }),
            base_p: 2,
            base_t: 2,
        }),
        ..creature("Angry Mob", cost(&[generic(2), w(), w()]), vec![CreatureType::Human], 2, 2)
    }
}

/// Brainwash — the enchanted creature has to buy its way into combat.
pub fn brainwash() -> CardDefinition {
    aura(
        "Brainwash",
        cost(&[w()]),
        R::Creature,
        EquipBonus {
            keywords: vec![Keyword::CantAttackUnlessPay(3)],
            ..Default::default()
        },
    )
}

/// Lurker — spells can't touch it until it commits to combat.
pub fn lurker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeTargetedBySpellsUnlessAttackedOrBlocked],
        ..creature("Lurker", cost(&[generic(2), g()]), vec![CreatureType::Beast], 2, 3)
    }
}

/// Goblin Rock Sled — downhill only: it needs a Mountain to aim at and a turn
/// off after every run.
pub fn goblin_rock_sled() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Trample,
            Keyword::DoesntUntapIfAttackedLastTurn,
            Keyword::CantAttackUnlessDefenderControlsLandType(LandType::Mountain),
        ],
        ..creature("Goblin Rock Sled", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 3, 1)
    }
}

/// Tangle Kelp — taps the creature now, and keeps it down after every attack.
pub fn tangle_kelp() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Tap { what: host() })],
        ..aura(
            "Tangle Kelp",
            cost(&[u()]),
            R::Creature,
            EquipBonus {
                keywords: vec![Keyword::DoesntUntapIfAttackedLastTurn],
                ..Default::default()
            },
        )
    }
}

/// Goblin Caves — a Goblin toughness anthem, but only over a basic Mountain.
pub fn goblin_caves() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as enchanted land is a basic Mountain, Goblins get +0/+2.",
            effect: goblin_anthem(0, 2),
        }],
        ..aura("Goblin Caves", cost(&[generic(1), r(), r()]), R::Land, EquipBonus::default())
    }
}

/// Goblin Shrine — the power half of Goblin Caves, and it burns the tribe when
/// it goes.
pub fn goblin_shrine() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as enchanted land is a basic Mountain, Goblins get +1/+0.",
            effect: goblin_anthem(1, 0),
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Goblin)),
                amount: Value::ONE,
            },
        }],
        ..aura("Goblin Shrine", cost(&[generic(1), r(), r()]), R::Land, EquipBonus::default())
    }
}

/// `EntityMatches` is vacuously true over an empty set, so the existence
/// check has to be explicit: an unattached Aura is not "on a basic Mountain".
fn host_is_basic_mountain() -> Predicate {
    Predicate::All(vec![
        Predicate::SelectorExists(host()),
        Predicate::EntityMatches {
            what: host(),
            filter: R::IsBasicLand.and(R::HasLandType(LandType::Mountain)),
        },
    ])
}

/// "As long as enchanted land is a basic Mountain, Goblin creatures get …"
fn goblin_anthem(power: i32, toughness: i32) -> StaticEffect {
    StaticEffect::AnthemForFilterIf {
        filter: R::Creature.and(R::HasCreatureType(CreatureType::Goblin)),
        power,
        toughness,
        keywords: vec![],
        condition: host_is_basic_mountain(),
        all_players: true,
    }
}

/// Worms of the Earth — no lands, by any route, until somebody pays the toll.
pub fn worms_of_the_earth() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Players can't play lands.",
                effect: StaticEffect::PlayersCantPlayMatching { filter: R::Land },
            },
            StaticAbility {
                description: "Lands can't enter the battlefield.",
                effect: StaticEffect::LandsCantEnterTheBattlefield,
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::AnyPlayerMayAccept {
                who: PlayerRef::EachPlayer,
                prompt: "Sacrifice two lands to destroy Worms of the Earth?".into(),
                accepted: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::AcceptingPlayer),
                        count: Value::Const(2),
                        filter: R::Land,
                    },
                    Effect::SacrificeSource,
                ])),
                otherwise: Box::new(Effect::AnyPlayerMayAccept {
                    who: PlayerRef::EachPlayer,
                    prompt: "Take 5 damage to destroy Worms of the Earth?".into(),
                    accepted: Box::new(Effect::Seq(vec![
                        Effect::DealDamage {
                            to: Selector::Player(PlayerRef::AcceptingPlayer),
                            amount: Value::Const(5),
                        },
                        Effect::SacrificeSource,
                    ])),
                    otherwise: Box::new(Effect::Seq(vec![])),
                }),
            },
        }],
        ..enchantment("Worms of the Earth", cost(&[generic(2), b(), b(), b()]))
    }
}

/// Festival — an opponent's-upkeep instant that calls off the whole combat.
pub fn festival() -> CardDefinition {
    CardDefinition {
        cast_condition: Some(Predicate::All(vec![
            Predicate::CurrentStepIs(TurnStep::Upkeep),
            Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
        ])),
        ..instant(
            "Festival",
            cost(&[w()]),
            Effect::CantAttackThisTurn { what: Selector::EachPermanent(R::Creature) },
        )
    }
}

/// Deep Water — for a turn, every land you tap is an Island.
pub fn deep_water() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::YourLandsProduceColorThisTurn(Color::Blue),
            ..Default::default()
        }],
        ..enchantment("Deep Water", cost(&[u(), u()]))
    }
}

/// Mind Bomb — three cards or three damage, everyone's choice.
pub fn mind_bomb() -> CardDefinition {
    sorcery(
        "Mind Bomb",
        cost(&[u()]),
        Effect::EachPlayerMayDiscardUpToThenDamage { max: 3 },
    )
}

/// Leviathan — a 10/10 that costs two Islands to wake up and two more to swing.
pub fn leviathan() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Trample,
            Keyword::AttackCostSacrifice(Box::new(R::HasLandType(LandType::Island)), 2),
        ],
        static_abilities: vec![
            StaticAbility {
                description: "This creature enters tapped.",
                effect: StaticEffect::EntersTapped { applies_to: Selector::This },
            },
            StaticAbility {
                description: "This creature doesn't untap during your untap step.",
                effect: StaticEffect::PreventUntap { applies_to: Selector::This },
            },
        ],
        triggered_abilities: vec![upkeep(Effect::MaySacrifice {
            description: "Sacrifice two Islands to untap Leviathan?".into(),
            filter: R::HasLandType(LandType::Island),
            count: Value::Const(2),
            then: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
            else_: None,
        })],
        ..creature(
            "Leviathan",
            cost(&[generic(5), u(), u(), u(), u()]),
            vec![CreatureType::Leviathan],
            10,
            10,
        )
    }
}

/// Season of the Witch — every creature that sat out the turn dies for it.
pub fn season_of_the_witch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            upkeep(Effect::UnlessPlayerPays {
                who: PlayerRef::You,
                cost: WardCost::Life(2),
                then: Box::new(Effect::SacrificeSource),
                if_paid: None,
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::Destroy {
                    what: Selector::EachPermanent(
                        R::Creature
                            .and(R::Not(Box::new(R::Tapped)))
                            .and(R::Not(Box::new(R::AttackedThisTurn)))
                            .and(R::Not(Box::new(R::HasKeyword(Keyword::Defender)))),
                    ),
                },
            },
        ],
        ..enchantment("Season of the Witch", cost(&[b(), b(), b()]))
    }
}

/// Psychic Allergy — picks a colour and bleeds whoever plays it, at the cost of
/// two Islands a turn.
pub fn psychic_allergy() -> CardDefinition {
    CardDefinition {
        effect: Effect::ChooseColorForSelf,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::OpponentControl,
                ),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::count(Selector::ControlledBy {
                        who: PlayerRef::ActivePlayer,
                        filter: R::HasChosenColorOfSource.and(R::NotToken),
                    }),
                },
            },
            upkeep(Effect::MaySacrifice {
                description: "Sacrifice two Islands to keep Psychic Allergy?".into(),
                filter: R::HasLandType(LandType::Island),
                count: Value::Const(2),
                then: Box::new(Effect::Seq(vec![])),
                else_: Some(Box::new(Effect::Destroy { what: Selector::This })),
            }),
        ],
        ..enchantment("Psychic Allergy", cost(&[generic(3), u(), u()]))
    }
}

/// Scarwood Bandits — buys an artifact off an opponent unless they buy it back.
pub fn scarwood_bandits() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            tap_cost: true,
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::EachOpponent,
                cost: WardCost::generic(2),
                then: Box::new(Effect::GainControlWhileSourceRemains {
                    what: target_filtered(R::Artifact),
                }),
                if_paid: None,
            },
            ..Default::default()
        }],
        ..creature(
            "Scarwood Bandits",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Cleansing — every land dies unless somebody buys it back a life at a time.
pub fn cleansing() -> CardDefinition {
    sorcery(
        "Cleansing",
        cost(&[w(), w(), w()]),
        Effect::ForEach {
            selector: Selector::EachPermanent(R::Land),
            body: Box::new(Effect::AnyPlayerMayAccept {
                who: PlayerRef::EachPlayer,
                prompt: "Pay 1 life to save that land?".into(),
                accepted: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::AcceptingPlayer),
                    amount: Value::ONE,
                }),
                otherwise: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
            }),
        },
    )
}

/// Fire and Brimstone — punishes whoever swung, and singes you for it.
pub fn fire_and_brimstone() -> CardDefinition {
    instant(
        "Fire and Brimstone",
        cost(&[generic(3), w(), w()]),
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Player.and(R::PlayerAttackedThisTurn)),
                amount: Value::Const(4),
            },
            Effect::DealDamage { to: Selector::You, amount: Value::Const(4) },
        ]),
    )
}

/// The Fallen — everything it has ever bled keeps bleeding.
pub fn the_fallen() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![upkeep(Effect::DealDamage {
            to: Selector::DamagedBySourceThisGame,
            amount: Value::ONE,
        })],
        ..creature("The Fallen", cost(&[generic(1), b(), b(), b()]), vec![CreatureType::Zombie], 2, 3)
    }
}

/// Dark Sphere — halves the next hit you take, rounded in your favour.
pub fn dark_sphere() -> CardDefinition {
    artifact(
        "Dark Sphere",
        cost(&[]),
        vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::PreventNextHalfDamageToYouThisTurn,
            ..Default::default()
        }],
    )
}

/// Blood of the Martyr — for a turn, every wound your creatures take is yours.
pub fn blood_of_the_martyr() -> CardDefinition {
    instant(
        "Blood of the Martyr",
        cost(&[w(), w(), w()]),
        Effect::RedirectCreatureDamageToYouThisTurn,
    )
}

/// Wand of Ith — a random card out of their hand, paid for in life or lost.
pub fn wand_of_ith() -> CardDefinition {
    artifact(
        "Wand of Ith",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::RevealRandomFromHand { who: Selector::Player(PlayerRef::Target(0)) },
                Effect::PlayerMayPayLifeElse {
                    who: PlayerRef::Target(0),
                    life: Value::IfPred {
                        pred: Box::new(Predicate::EntityMatches {
                            what: Selector::LastRevealedCard,
                            filter: R::Land,
                        }),
                        then: Box::new(Value::ONE),
                        else_: Box::new(Value::LastRevealedManaValue),
                    },
                    else_: Box::new(Effect::Move {
                        what: Selector::LastRevealedCard,
                        to: ZoneDest::Graveyard,
                    }),
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Mana Vortex — everyone's lands go, one a turn, until there are none left.
pub fn mana_vortex() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            crate::effect::shortcut::on_cast(Effect::UnlessPlayerPays {
                who: PlayerRef::You,
                cost: crate::card::WardCost::SacrificeMatching(Box::new(R::Land)),
                then: Box::new(Effect::CounterSpell { what: Selector::This }),
                if_paid: None,
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    count: Value::ONE,
                    filter: R::Land,
                },
            },
        ],
        state_trigger: Some(crate::effect::StateTriggeredAbility {
            condition: Predicate::Not(Box::new(Predicate::SelectorExists(
                Selector::EachPermanent(R::Land),
            ))),
            effect: Effect::SacrificeSource,
        }),
        ..enchantment("Mana Vortex", cost(&[generic(1), u(), u()]))
    }
}

/// Reflecting Mirror — bounces a spell aimed at you onto another player.
pub fn reflecting_mirror() -> CardDefinition {
    artifact(
        "Reflecting Mirror",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            condition: Some(Predicate::ValueAtLeast(
                Value::XFromCost,
                Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(Value::ManaValueOf(Box::new(Selector::Target(0)))),
                ),
            )),
            effect: Effect::ChangeSpellTarget {
                what: target_filtered(
                    R::IsSpellOnStack
                        .and(R::SpellWithSingleTarget)
                        .and(R::SpellTargetsControllerOrControlled),
                ),
            },
            ..Default::default()
        }],
    )
}

/// Preacher — stays tapped, and keeps whatever it took while it is.
pub fn preacher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainControlWhileSourceTapped {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
            ..Default::default()
        }],
        ..creature(
            "Preacher",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Fasting — starve for life; any draw, or five upkeeps, ends it.
pub fn fasting() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may skip your draw step to gain 2 life.",
            effect: StaticEffect::ControllerMaySkipDrawStepForLife { life: 2 },
        }],
        triggered_abilities: vec![
            upkeep(Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Hunger,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Hunger,
                        },
                        Value::Const(5),
                    ),
                    then: Box::new(Effect::Destroy { what: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: Effect::Destroy { what: Selector::This },
            },
        ],
        ..enchantment("Fasting", cost(&[w()]))
    }
}

/// Nameless Race — enters as big as the life you pay, bounded by the white
/// your opponents have shown.
pub fn nameless_race() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(crate::card::DynamicPt::ChosenNumberAsEntered),
        as_enters_effect: Some(Effect::PayAnyAmountOfLifeCapped {
            max: Value::Sum(vec![
                Value::CountOf(Box::new(Selector::EachPermanent(
                    R::HasColor(Color::White)
                        .and(R::ControlledByOpponent)
                        .and(R::Not(Box::new(R::IsToken))),
                ))),
                Value::CardsInOpponentsGraveyardsMatching {
                    filter: R::HasColor(Color::White),
                },
            ]),
        }),
        ..creature("Nameless Race", cost(&[generic(3), b()]), vec![], 0, 0)
    }
}

/// Dance of Many — a token copy the enchantment is chained to; either one
/// leaving takes the other with it.
pub fn dance_of_many() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: target_filtered(R::Creature.and(R::Not(Box::new(R::IsToken)))),
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        extra_keywords: vec![],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                    },
                    Effect::RememberPermanentOnSource { what: Selector::LastCreatedToken },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::Exile { what: Selector::ChosenPermanentOfSource },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::AnyPlayer)
                    .with_filter(Predicate::TriggerSourceIsSourcesChosenPermanent),
                effect: Effect::SacrificeSource,
            },
            upkeep(Effect::UnlessPlayerPays {
                who: PlayerRef::You,
                cost: WardCost::Mana(cost(&[u(), u()])),
                then: Box::new(Effect::SacrificeSource),
                if_paid: None,
            }),
        ],
        ..enchantment("Dance of Many", cost(&[u(), u()]))
    }
}

/// Frankenstein's Monster — stitched together from X exiled creature cards,
/// each buying a +2/+0, +1/+1, or +0/+2 counter.
pub fn frankensteins_monster() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::EnterExilingGraveyardCreaturesForCounters {
            count: Value::XFromCost,
        }),
        ..creature(
            "Frankenstein's Monster",
            cost(&[x(), b(), b()]),
            vec![CreatureType::Zombie],
            0,
            1,
        )
    }
}

/// Runesword — a lethal edge: what the pumped attacker damages can't be
/// regenerated and is exiled, and losing the attacker costs the sword.
pub fn runesword() -> CardDefinition {
    artifact(
        "Runesword",
        cost(&[generic(6)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::IsAttacking)),
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantDamageDeniesRegenerationThisTurn { what: Selector::Target(0) },
                Effect::GrantDamageExilesVictimThisTurn { what: Selector::Target(0) },
                Effect::WhenTargetLeavesBattlefieldThisTurn {
                    what: Selector::Target(0),
                    body: Box::new(Effect::SacrificeSource),
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Sorrow's Path — swaps two of an opponent's blockers, and burns you for
/// tapping.
pub fn sorrows_path() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::You, amount: Value::Const(2) },
                Effect::DealDamage {
                    to: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..land(
            "Sorrow's Path",
            vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::SwapBlockAssignments {
                    a: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::IsBlocking).and(R::ControlledByOpponent),
                    },
                    b: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature
                            .and(R::IsBlocking)
                            .and(R::SameControllerAsTargetSlot(0)),
                    },
                },
                ..Default::default()
            }],
        )
    }
}
