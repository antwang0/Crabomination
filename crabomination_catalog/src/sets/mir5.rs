//! Mirage (MIR), fifth wave — the closing rares: prevention shields, upkeep
//! engines, control auras and the graveyard tricks. Tests in `classic_sets/mir`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{on_unblocked, target_filtered};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, RevealMissDest, Selector, StaticEffect,
    Value, ZoneDest,
};
use crate::game::TurnStep;
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
    CardDefinition { card_types: vec![CardType::Sorcery], ..instant(name, c, effect) }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, enchant: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
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

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

fn cumulative_upkeep_1() -> Keyword {
    Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))
}

/// The Aura's host, as a selector.
fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Asmira, Holy Avenger — {2}{G}{W} 2/3 flier that fattens on each end step
/// for every creature that hit your graveyard this turn.
pub fn asmira_holy_avenger() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ControllerCreaturesDiedThisTurn,
            },
        }],
        ..creature(
            "Asmira, Holy Avenger",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            3,
        )
    }
}

/// Coral Fighters — {1}{U} 1/1 that peeks at the defender's top card when it
/// slips through, and may bury it.
pub fn coral_fighters() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_unblocked(Effect::LookTopMayBottomAllElse {
            who: Some(PlayerRef::DefendingPlayer),
            count: Value::Const(1),
            then: Box::new(Effect::Noop),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Coral Fighters",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Kukemssa Pirates — {3}{U} 2/2 that trades its combat damage for an artifact
/// whenever the defender lets it through.
pub fn kukemssa_pirates() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_unblocked(Effect::MayDo {
            description: "Gain control of target artifact defending player controls?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(R::Artifact.and(R::OwnedByDefendingPlayer)),
                    to: None,
                    duration: Duration::Permanent,
                },
                Effect::AssignsNoCombatDamageThisTurn { what: Selector::This },
            ])),
        })],
        ..creature(
            "Kukemssa Pirates",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            2,
            2,
        )
    }
}

/// Shauku, Endbringer — {5}{B}{B} 5/5 flier that eats a creature a turn, can
/// only swing when the board is otherwise empty, and bleeds you for it.
pub fn shauku_endbringer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Shauku can't attack if there's another creature on the battlefield.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Creature),
                    n: Value::Const(2),
                },
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::This,
                    keyword: Keyword::CantAttack,
                }),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Creature) },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Shauku, Endbringer",
            cost(&[generic(5), b(), b()]),
            vec![CreatureType::Vampire],
            5,
            5,
        )
    }
}

/// Phyrexian Dreadnought — {1} 12/12 trample that demands twelve power of
/// creatures on the way in.
pub fn phyrexian_dreadnought() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![crate::effect::shortcut::etb(
            Effect::SacrificeSourceUnlessSacrificeTotalPower {
                filter: R::Creature,
                total_power: Value::Const(12),
            },
        )],
        ..creature(
            "Phyrexian Dreadnought",
            cost(&[generic(1)]),
            vec![CreatureType::Phyrexian, CreatureType::Dreadnought],
            12,
            12,
        )
    }
}

/// Tainted Specter — {3}{B} 2/2 flier whose tax is a discard the victim can
/// buy off by stacking their own library; a real discard sprays the board.
pub fn tainted_specter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), b()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::DiscardUnlessPutCardOnTop {
                who: PlayerRef::Target(0),
                then: Box::new(Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::EachPermanent(R::Creature),
                        amount: Value::Const(1),
                    },
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::Const(1),
                    },
                ])),
            },
            ..Default::default()
        }],
        ..creature("Tainted Specter", cost(&[generic(3), b()]), vec![CreatureType::Specter], 2, 2)
    }
}

/// Hakim, Loreweaver — {3}{U}{U} 2/4 flier that dresses itself in Auras from
/// the graveyard and can shrug them all off again.
pub fn hakim_loreweaver() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u(), u()]),
                condition: Some(Predicate::All(vec![
                    Predicate::CurrentStepIs(TurnStep::Upkeep),
                    Predicate::IsTurnOf(PlayerRef::You),
                    Predicate::Not(Box::new(Predicate::SelectorCountAtLeast {
                        sel: Selector::MatchingAmong {
                            inner: Box::new(Selector::This),
                            filter: R::IsEnchanted,
                        },
                        n: Value::Const(1),
                    })),
                ])),
                effect: Effect::AttachAuraFromGraveyardTo {
                    aura: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                            .and(R::InGraveyard),
                    },
                    host: Selector::This,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u(), u()]),
                tap_cost: true,
                effect: Effect::Destroy {
                    what: Selector::MatchingAmong {
                        inner: Box::new(Selector::AttachedToMe(Box::new(Selector::This))),
                        filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura),
                    },
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Hakim, Loreweaver",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            4,
        )
    }
}

/// Hivis of the Scale — {3}{R}{R} 3/4 that keeps a Dragon for as long as it
/// stays tapped.
pub fn hivis_of_the_scale() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainControlWhileSourceTapped {
                what: target_filtered(R::HasCreatureType(CreatureType::Dragon)),
            },
            ..Default::default()
        }],
        ..creature(
            "Hivis of the Scale",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Lizard, CreatureType::Shaman],
            3,
            4,
        )
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Amber Prison — {4} artifact that locks a permanent down for as long as it
/// stays tapped itself.
pub fn amber_prison() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        ..artifact(
            "Amber Prison",
            cost(&[generic(4)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::TapAndUntapLock {
                    what: target_filtered(R::Artifact.or(R::Creature).or(R::Land)),
                },
                ..Default::default()
            }],
        )
    }
}

/// Bone Mask — {4} artifact that eats one damage event off the top of your
/// library.
pub fn bone_mask() -> CardDefinition {
    artifact(
        "Bone Mask",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
                exile_top_per_prevented: true,
            },
            ..Default::default()
        }],
    )
}

/// Ventifact Bottle — {3} artifact that banks charge counters and dumps them
/// as colorless mana on your first main phase.
pub fn ventifact_bottle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::YourControl,
            )
            .with_filter(Predicate::ValueAtLeast(
                Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
                Value::Const(1),
            )),
            effect: Effect::Seq(vec![
                Effect::Tap { what: Selector::This },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Charge,
                    }),
                },
                Effect::RemoveAllCounters { what: Selector::This },
            ]),
        }],
        ..artifact(
            "Ventifact Bottle",
            cost(&[generic(3)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[x(), generic(1)]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::XFromCost,
                },
                ..Default::default()
            }],
        )
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Benevolent Unicorn — {1}{W} 1/2 that shaves a point off every spell's
/// damage, wherever it lands.
pub fn benevolent_unicorn() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells deal 1 less damage.",
            effect: StaticEffect::ReduceSpellDamageBy { amount: 1 },
        }],
        ..creature("Benevolent Unicorn", cost(&[generic(1), w()]), vec![CreatureType::Unicorn], 1, 2)
    }
}

/// Prismatic Circle — {2}{W} Circle of Protection for a colour you name, on an
/// escalating lease.
pub fn prismatic_circle() -> CardDefinition {
    CardDefinition {
        keywords: vec![cumulative_upkeep_1()],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::ChooseColorForSelf)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::HasChosenColorOfSource,
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
                exile_top_per_prevented: false,
            },
            ..Default::default()
        }],
        ..enchantment("Prismatic Circle", cost(&[generic(2), w()]))
    }
}

/// Roots of Life — {1}{G}{G} drips life every time an opponent taps a land of
/// the named type.
pub fn roots_of_life() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::ChooseBasicLandTypeForSource),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::OpponentControl)
                    .with_filter(Predicate::EntityMatchesAny {
                        what: Selector::TriggerSource,
                        filter: R::HasChosenLandTypeOfSource,
                    }),
                effect: crate::effect::shortcut::gain_life(1),
            },
        ],
        ..enchantment("Roots of Life", cost(&[generic(1), g(), g()]))
    }
}

/// Purgatory — {2}{W}{B} pockets your dead and rents them back.
pub fn purgatory() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatchesAny { what: Selector::TriggerSource, filter: R::NotToken }),
                effect: Effect::ExileTaggedWithSource { what: Selector::TriggerSource },
            },
            TriggeredAbility {
                event: your_upkeep(),
                effect: Effect::MayPay {
                    description: "Pay {4} and 2 life to return an exiled creature?".into(),
                    mana_cost: cost(&[generic(4)]),
                    body: Box::new(Effect::Seq(vec![
                        Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                        Effect::ReturnExiledBySourceToBattlefield {
                            decayed: false,
                            count: Some(Value::Const(1)),
                        },
                    ])),
                    else_: None,
                },
            },
        ],
        ..enchantment("Purgatory", cost(&[generic(2), w(), b()]))
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Mind Harness — {U} Aura that steals a red or green creature on an
/// escalating lease.
pub fn mind_harness() -> CardDefinition {
    CardDefinition {
        keywords: vec![cumulative_upkeep_1()],
        triggered_abilities: vec![crate::effect::shortcut::etb(
            Effect::GainControlWhileSourceAttached,
        )],
        ..aura(
            "Mind Harness",
            cost(&[u()]),
            R::Creature.and(R::HasColor(Color::Red).or(R::HasColor(Color::Green))),
            EquipBonus::default(),
        )
    }
}

/// Consuming Ferocity — {1}{R} Aura that pumps its host until it bursts.
pub fn consuming_ferocity() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: host(),
                    kind: CounterType::PlusOnePlusZero,
                    amount: Value::Const(1),
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn { what: Box::new(host()), kind: CounterType::PlusOnePlusZero },
                        Value::Const(3),
                    ),
                    then: Box::new(Effect::Seq(vec![
                        Effect::DealDamage {
                            to: Selector::Player(PlayerRef::ControllerOf(Box::new(host()))),
                            amount: Value::PowerOf(Box::new(host())),
                        },
                        Effect::DestroyNoRegen { what: host() },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..aura(
            "Consuming Ferocity",
            cost(&[generic(1), r()]),
            R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Wall)))),
            EquipBonus { power: 1, ..Default::default() },
        )
    }
}

/// Wellspring — {1}{G}{W} Aura that borrows the enchanted land every upkeep.
pub fn wellspring() -> CardDefinition {
    let seize = Effect::GainControl { what: host(), to: None, duration: Duration::EndOfTurn };
    CardDefinition {
        triggered_abilities: vec![
            crate::effect::shortcut::etb(seize.clone()),
            TriggeredAbility {
                event: your_upkeep(),
                effect: Effect::Seq(vec![Effect::Untap { what: host(), up_to: None }, seize]),
            },
        ],
        ..aura("Wellspring", cost(&[generic(1), g(), w()]), R::Land, EquipBonus::default())
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Dream Cache — {2}{U} draws three and puts two back, all on one end.
pub fn dream_cache() -> CardDefinition {
    sorcery(
        "Dream Cache",
        cost(&[generic(2), u()]),
        Effect::Seq(vec![
            crate::effect::shortcut::draw(3),
            Effect::ChooseMode(vec![
                Effect::ChooseFromHandToTopOfLibrary { who: PlayerRef::You, count: Value::Const(2) },
                Effect::PutCardsFromHandOnBottom { who: Selector::You, count: Value::Const(2) },
            ]),
        ]),
    )
}

/// Sealed Fate — {X}{U}{B} plucks one card out of the top X of an opponent's
/// library.
pub fn sealed_fate() -> CardDefinition {
    CardDefinition {
        ..sorcery(
            "Sealed Fate",
            cost(&[x(), u(), b()]),
            Effect::LookTopExileOneOfN { who: PlayerRef::Target(0), count: Value::XFromCost },
        )
    }
}

/// Shallow Grave — {1}{B} borrows the freshest corpse for one swing.
pub fn shallow_grave() -> CardDefinition {
    instant(
        "Shallow Grave",
        cost(&[generic(1), b()]),
        Effect::Seq(vec![
            Effect::ReturnTopCreatureFromGraveyard { who: PlayerRef::You },
            Effect::GrantKeyword {
                what: Selector::LastMoved,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::ExileAtNextEndStep { what: Selector::LastMoved },
        ]),
    )
}

/// Polymorph — {3}{U} kills a creature and digs its controller a new one.
pub fn polymorph() -> CardDefinition {
    CardDefinition {
        ..sorcery(
            "Polymorph",
            cost(&[generic(3), u()]),
            Effect::Seq(vec![
                Effect::DestroyNoRegen { what: target_filtered(R::Creature) },
                Effect::RevealUntilFind {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    find: R::Creature,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                        tapped: false,
                    },
                    cap: Value::Const(500),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::ShuffleIntoLibrary,
                },
            ]),
        )
    }
}

/// Aleatory — {1}{R} post-blockers coin flip with a cantrip chaser.
pub fn aleatory() -> CardDefinition {
    CardDefinition {
        cast_only_after_blockers: true,
        ..instant(
            "Aleatory",
            cost(&[generic(1), r()]),
            Effect::Seq(vec![
                Effect::FlipCoin {
                    count: Value::Const(1),
                    on_heads: Box::new(crate::effect::shortcut::pump_target(1, 1)),
                    on_tails: Box::new(Effect::Noop),
                },
                Effect::AtNextTurnsUpkeep { body: Box::new(crate::effect::shortcut::draw(1)) },
            ]),
        )
    }
}

/// Reflect Damage — {3}{R}{W} bounces one damage event back at its own source's
/// controller.
pub fn reflect_damage() -> CardDefinition {
    instant(
        "Reflect Damage",
        cost(&[generic(3), r(), w()]),
        Effect::PreventNextEventFromChosenSourceAnywhere { what: None, reflect: true },
    )
}

// ── Wave 6 ──────────────────────────────────────────────────────────────────

/// Haunting Apparition — {1}{U}{B} flier sized by the green creatures rotting
/// in a chosen opponent's graveyard.
pub fn haunting_apparition() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        as_enters_effect: Some(Effect::RememberPlayerOnSource { who: PlayerRef::EachOpponent }),
        dynamic_pt: Some(crate::card::DynamicPt::ChosenPlayerGraveyardMatching {
            base_p: 1,
            base_t: 2,
            filter: R::Creature.and(R::HasColor(Color::Green)),
        }),
        ..creature(
            "Haunting Apparition",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Spirit],
            1,
            2,
        )
    }
}

/// Basalt Golem — {5} 2/4 that trades whatever blocks it for a Wall.
pub fn basalt_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::Artifact))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificePermanent {
                        what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
                    },
                    Effect::CreateToken {
                        who: PlayerRef::DefendingPlayer,
                        count: Value::ONE,
                        definition: TokenDefinition {
                            name: "Wall".into(),
                            power: 0,
                            toughness: 2,
                            keywords: vec![Keyword::Defender],
                            card_types: vec![CardType::Artifact, CardType::Creature],
                            subtypes: Subtypes {
                                creature_types: vec![CreatureType::Wall],
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    },
                ])),
            },
        }],
        ..creature("Basalt Golem", cost(&[generic(5)]), vec![CreatureType::Golem], 2, 4)
    }
}

/// Shimmer — {2}{U}{U} gives every land of the named type phasing.
pub fn shimmer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(
            Effect::ChooseBasicLandTypeForSource,
        )],
        static_abilities: vec![StaticAbility {
            description: "Each land of the chosen type has phasing.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Land.and(R::HasChosenLandTypeOfSource)),
                keyword: Keyword::Phasing,
            },
        }],
        ..enchantment("Shimmer", cost(&[generic(2), u(), u()]))
    }
}

/// Spatial Binding — {U}{B} pins a permanent in phase for a life a shot.
pub fn spatial_binding() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Permanent),
                keyword: Keyword::CantPhaseOut,
                duration: Duration::UntilYourNextUpkeep,
            },
            ..Default::default()
        }],
        ..enchantment("Spatial Binding", cost(&[u(), b()]))
    }
}

/// Ward of Lights — {W}{W} Aura granting protection from a named colour,
/// keeping itself attached.
pub fn ward_of_lights() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::ChooseColorForSelf)],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has protection from the chosen color.",
            effect: StaticEffect::GrantProtectionFromChosenColor {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..aura(
            "Ward of Lights",
            cost(&[w(), w()]),
            R::Creature,
            // CR 702.16k — "This effect doesn't remove this Aura."
            EquipBonus { protection_keeps_self: true, ..Default::default() },
        )
    }
}

/// Malignant Growth — {3}{G}{U} feeds your opponents cards and bills them for
/// each one.
pub fn malignant_growth() -> CardDefinition {
    let growth = Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Growth };
    CardDefinition {
        keywords: vec![cumulative_upkeep_1()],
        triggered_abilities: vec![
            TriggeredAbility {
                event: your_upkeep(),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Growth,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Draw),
                    EventScope::OpponentControl,
                ),
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::Player(PlayerRef::ActivePlayer),
                        amount: growth.clone(),
                    },
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::ActivePlayer),
                        amount: growth,
                    },
                ]),
            },
        ],
        ..enchantment("Malignant Growth", cost(&[generic(3), g(), u()]))
    }
}

/// Preferred Selection — {2}{G}{G} upkeep dig: buy the card outright, or
/// settle for burying one.
pub fn preferred_selection() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::MayPay {
                description: "Sacrifice Preferred Selection and pay {2}{G}{G}?".into(),
                mana_cost: cost(&[generic(2), g(), g()]),
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificeSource,
                    Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                        who: PlayerRef::You,
                        count: Value::Const(2),
                        ..Default::default()
                    })),
                ])),
                else_: Some(Box::new(Effect::LookTopPutOneOnBottom { count: Value::Const(2) })),
            },
        }],
        ..enchantment("Preferred Selection", cost(&[generic(2), g(), g()]))
    }
}

/// Natural Balance — {2}{G}{G} trims every big mana base to five lands and
/// tops the small ones up.
pub fn natural_balance() -> CardDefinition {
    sorcery(
        "Natural Balance",
        cost(&[generic(2), g(), g()]),
        Effect::Seq(vec![
            Effect::EachPlayerKeepsNSacrificesRest {
                keep: Value::Const(5),
                filter: Some(R::Land),
            },
            Effect::CatchUpBasicLands { target: Some(Value::Const(5)), tapped: false },
        ]),
    )
}

/// Delirium — {1}{B}{R} taps a creature on its controller's own turn and turns
/// it on them.
pub fn delirium() -> CardDefinition {
    CardDefinition {
        cast_condition: Some(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
        ..instant(
            "Delirium",
            cost(&[generic(1), b(), r()]),
            Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::DealDamageEqualToPower {
                    source: Selector::Target(0),
                    target: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                },
                Effect::PreventCombatDamageToTargetThisTurn { target: Selector::Target(0) },
                Effect::PreventCombatDamageByTargetThisTurn { target: Selector::Target(0) },
            ]),
        )
    }
}

/// Emberwilde Djinn — {2}{R}{R} 5/4 flier that changes hands for {R}{R} or
/// two life at each upkeep.
pub fn emberwilde_djinn() -> CardDefinition {
    let seize = Effect::GainControl {
        what: Selector::This,
        to: None,
        duration: Duration::Permanent,
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::MayPayBy {
                who: PlayerRef::ActivePlayer,
                description: "Pay {R}{R} to gain control of Emberwilde Djinn?".into(),
                mana_cost: cost(&[r(), r()]),
                body: Box::new(seize.clone()),
                else_: Some(Box::new(Effect::MayPayLife {
                    description: "Pay 2 life to gain control of Emberwilde Djinn?".into(),
                    amount: Value::Const(2),
                    body: Box::new(seize),
                    else_: None,
                })),
            },
        }],
        ..creature("Emberwilde Djinn", cost(&[generic(2), r(), r()]), vec![CreatureType::Djinn], 5, 4)
    }
}
