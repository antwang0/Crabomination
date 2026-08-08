//! Antiquities (ATQ) — the artifact war. Tests in `classic_sets/atq`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::{target_any, target_filtered},
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, r, u, w, x};

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

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..enchantment(name, c)
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

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn upkeep_only() -> Predicate {
    Predicate::All(vec![
        Predicate::IsTurnOf(PlayerRef::You),
        Predicate::CurrentStepIs(TurnStep::Upkeep),
    ])
}

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource)
}

/// "Whenever [filter] becomes tapped, or its controller activates one of its
/// abilities without {T} in the cost" — the Antiquities artifact-tax shape.
fn tapped_or_activated(filter: R, effect: Effect) -> Vec<TriggeredAbility> {
    vec![
        TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: filter.clone() },
            ),
            effect: effect.clone(),
        },
        TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivated, EventScope::AnyPlayer)
                .without_tap_cost()
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter,
                }),
            effect,
        },
    ]
}

// ── Artifacts ──────────────────────────────────────────────────────────────

/// Amulet of Kroog — a repeatable one-point shield.
pub fn amulet_of_kroog() -> CardDefinition {
    artifact(
        "Amulet of Kroog",
        cost(&[generic(2)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::PreventNextDamage { target: target_any(), amount: Value::Const(1) },
            ..Default::default()
        }],
    )
}

/// Armageddon Clock — a doomsday counter anyone can wind back for {4}.
pub fn armageddon_clock() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: your_upkeep(),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Doom,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Draw),
                    EventScope::SelfSource,
                ),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Doom,
                    },
                },
            },
        ],
        ..artifact(
            "Armageddon Clock",
            cost(&[generic(6)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                any_player: true,
                condition: Some(Predicate::CurrentStepIs(TurnStep::Upkeep)),
                effect: Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Doom,
                    amount: Value::Const(1),
                },
                ..Default::default()
            }],
        )
    }
}

/// Ashnod's Battle Gear — swap toughness for power while it stays tapped.
pub fn ashnods_battle_gear() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        ..artifact(
            "Ashnod's Battle Gear",
            cost(&[generic(2)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::Const(2),
                    toughness: Value::Const(-2),
                    duration: Duration::WhileSourceTapped,
                },
                ..Default::default()
            }],
        )
    }
}

/// Ashnod's Transmogrant — grafts a counter on and makes the creature metal.
pub fn ashnods_transmogrant() -> CardDefinition {
    artifact(
        "Ashnod's Transmogrant",
        cost(&[generic(1)]),
        vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::AddCardTypeIndefinitely {
                    what: Selector::Target(0),
                    card_type: CardType::Artifact,
                    until_eot: false,
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Battering Ram — bands into combat and knocks down the Wall that stops it.
pub fn battering_ram() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::SelfSource,
                ),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Banding,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::RememberPermanentOnSource {
                        what: Selector::EachPermanent(
                            R::HasCreatureType(CreatureType::Wall).and(R::InCombatWithSource),
                        ),
                    },
                    Effect::AtEndOfCombat {
                        body: Box::new(Effect::Destroy {
                            what: Selector::ChosenPermanentOfSource,
                        }),
                    },
                ]),
            },
        ],
        ..artifact_creature(
            "Battering Ram",
            cost(&[generic(2)]),
            vec![CreatureType::Construct],
            1,
            1,
        )
    }
}

/// Clay Statue — a fragile golem that keeps standing back up.
pub fn clay_statue() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..artifact_creature("Clay Statue", cost(&[generic(4)]), vec![CreatureType::Golem], 3, 1)
    }
}

/// Colossus of Sardia — enormous, and slow to get back up.
pub fn colossus_of_sardia() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(9)]),
            condition: Some(upkeep_only()),
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        ..artifact_creature("Colossus of Sardia", cost(&[generic(9)]), vec![CreatureType::Golem], 9, 9)
    }
}

/// Coral Helm — trades cards at random for a combat trick.
pub fn coral_helm() -> CardDefinition {
    artifact(
        "Coral Helm",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            discard_cost: Some((R::Any, 1)),
            discard_cost_random: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
    )
}

/// Cursed Rack — the chosen opponent plays with a four-card hand.
pub fn cursed_rack() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::RememberPlayerOnSource { who: PlayerRef::EachOpponent },
        }],
        static_abilities: vec![StaticAbility {
            description: "The chosen player's maximum hand size is four.",
            effect: StaticEffect::ChosenPlayerMaxHandSize(4),
        }],
        ..artifact("Cursed Rack", cost(&[generic(4)]), vec![])
    }
}

/// Dragon Engine — pays mana for power.
pub fn dragon_engine() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Dragon Engine",
            cost(&[generic(3)]),
            vec![CreatureType::Construct],
            1,
            3,
        )
    }
}

/// Grapeshot Catapult — anti-air artillery.
pub fn grapeshot_catapult() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..artifact_creature(
            "Grapeshot Catapult",
            cost(&[generic(4)]),
            vec![CreatureType::Construct],
            2,
            3,
        )
    }
}

/// Jalum Tome — a slow rummage engine.
pub fn jalum_tome() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Book],
            ..Default::default()
        },
        ..artifact(
            "Jalum Tome",
            cost(&[generic(3)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                ]),
                ..Default::default()
            }],
        )
    }
}

/// Mishra's War Machine — a banding juggernaut with an upkeep appetite.
pub fn mishras_war_machine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Banding],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::MayDiscard {
                description: "Discard a card to Mishra's War Machine?".into(),
                count: Value::Const(1),
                then: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::Seq(vec![
                    Effect::DealDamage { to: Selector::You, amount: Value::Const(3) },
                    Effect::Tap { what: Selector::This },
                ]))),
            },
        }],
        ..artifact_creature(
            "Mishra's War Machine",
            cost(&[generic(7)]),
            vec![CreatureType::Juggernaut],
            5,
            5,
        )
    }
}

/// Mightstone — everyone's attackers hit harder.
pub fn mightstone() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                power: 1,
                toughness: 0,
            },
        }],
        ..artifact("Mightstone", cost(&[generic(4)]), vec![])
    }
}

/// Weakstone — Mightstone's mirror; attacking is a worse idea.
pub fn weakstone() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures get -1/-0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                power: -1,
                toughness: 0,
            },
        }],
        ..artifact("Weakstone", cost(&[generic(4)]), vec![])
    }
}

/// Mishra's Workshop — three colorless, artifacts only.
pub fn mishras_workshop() -> CardDefinition {
    CardDefinition {
        name: "Mishra's Workshop",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::Const(3))),
                    SpendRestriction::ArtifactOnly,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Obelisk of Undoing — an expensive rewind of your own permanent.
pub fn obelisk_of_undoing() -> CardDefinition {
    artifact(
        "Obelisk of Undoing",
        cost(&[generic(1)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Permanent.and(R::ControlledByYou).and(R::OwnedByYou)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
    )
}

/// Onulet — pays you back when it breaks.
pub fn onulet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..artifact_creature("Onulet", cost(&[generic(3)]), vec![CreatureType::Construct], 2, 2)
    }
}

/// Rakalite — a one-point shield that keeps coming back.
pub fn rakalite() -> CardDefinition {
    artifact(
        "Rakalite",
        cost(&[generic(6)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::PreventNextDamage { target: target_any(), amount: Value::Const(1) },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    }),
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Rocket Launcher — a slow but repeatable ping that blows itself up.
pub fn rocket_launcher() -> CardDefinition {
    artifact(
        "Rocket Launcher",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            condition: Some(Predicate::Not(Box::new(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::EnteredThisTurn,
            }))),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Destroy { what: Selector::This }),
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Staff of Zegon — blunts an attacker for a turn.
pub fn staff_of_zegon() -> CardDefinition {
    artifact(
        "Staff of Zegon",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
    )
}

/// Tablet of Epityr — a life drip off your own scrap.
pub fn tablet_of_epityr() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                }),
            effect: Effect::MayPay {
                description: "Pay {1} to gain 1 life?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(1) }),
                else_: None,
            },
        }],
        ..artifact("Tablet of Epityr", cost(&[generic(1)]), vec![])
    }
}

/// Tawnos's Wand — slips a small creature past the blockers.
pub fn tawnoss_wand() -> CardDefinition {
    artifact(
        "Tawnos's Wand",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
    )
}

/// Tawnos's Weaponry — a +1/+1 that lasts as long as the Weaponry stays down.
pub fn tawnoss_weaponry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        ..artifact(
            "Tawnos's Weaponry",
            cost(&[generic(2)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::WhileSourceTapped,
                },
                ..Default::default()
            }],
        )
    }
}

/// The Rack — the chosen opponent bleeds for every card they're missing.
pub fn the_rack() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::RememberPlayerOnSource { who: PlayerRef::EachOpponent },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::AnyPlayer,
                )
                .with_filter(Predicate::IsTurnOf(PlayerRef::ChosenPlayerOfSource)),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ChosenPlayerOfSource),
                    amount: Value::Sum(vec![
                        Value::Const(3),
                        Value::Negate(Box::new(Value::CardsInHandMatching {
                            who: PlayerRef::ChosenPlayerOfSource,
                            filter: R::Any,
                        })),
                    ]),
                },
            },
        ],
        ..artifact("The Rack", cost(&[generic(1)]), vec![])
    }
}

/// Urza's Chalice — a life drip off everyone's artifacts.
pub fn urzas_chalice() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                }),
            effect: Effect::MayPay {
                description: "Pay {1} to gain 1 life?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(1) }),
                else_: None,
            },
        }],
        ..artifact("Urza's Chalice", cost(&[generic(1)]), vec![])
    }
}

/// Urza's Miter — cards for artifacts that die on their own.
pub fn urzas_miter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                }),
            effect: Effect::MayPay {
                description: "Pay {3} to draw a card?".into(),
                mana_cost: cost(&[generic(3)]),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: None,
            },
        }],
        ..artifact("Urza's Miter", cost(&[generic(3)]), vec![])
    }
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Argivian Archaeologist — digs your artifacts back out of the graveyard.
pub fn argivian_archaeologist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), w()]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Argivian Archaeologist",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            1,
        )
    }
}

/// Argivian Blacksmith — patches up the machines.
pub fn argivian_blacksmith() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Artifact.and(R::Creature)),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Argivian Blacksmith",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            2,
        )
    }
}

/// Argothian Pixies — machines can't touch them.
pub fn argothian_pixies() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CantBeBlockedBy(Box::new(R::Artifact.and(R::Creature))),
            Keyword::PreventDamageFromMatching(Box::new(R::Artifact.and(R::Creature))),
        ],
        ..creature("Argothian Pixies", cost(&[generic(1), g()]), vec![CreatureType::Faerie], 2, 1)
    }
}

/// Argothian Treefolk — shrugs off anything made of metal.
pub fn argothian_treefolk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PreventDamageFromMatching(Box::new(R::Artifact))],
        ..creature(
            "Argothian Treefolk",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Treefolk],
            3,
            5,
        )
    }
}

/// Citanul Druid — grows off the other side's artifact deck.
pub fn citanul_druid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..creature(
            "Citanul Druid",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Dwarven Weaponsmith — melts an artifact down into a counter each upkeep.
pub fn dwarven_weaponsmith() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            condition: Some(upkeep_only()),
            effect: Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..creature(
            "Dwarven Weaponsmith",
            cost(&[generic(1), r()]),
            vec![CreatureType::Dwarf, CreatureType::Artificer],
            1,
            1,
        )
    }
}

/// Gaea's Avenger — scales with the opposing artifact count.
pub fn gaeas_avenger() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::BasePlusOpponentsMatching {
            base_p: 1,
            base_t: 1,
            filter: Box::new(R::Artifact),
        }),
        ..creature(
            "Gaea's Avenger",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Treefolk],
            1,
            1,
        )
    }
}

/// Martyrs of Korlis — soaks up every artifact's damage while it stands.
pub fn martyrs_of_korlis() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is untapped, all damage that would be dealt \
                          to you by artifacts is dealt to this creature instead.",
            effect: StaticEffect::RedirectArtifactDamageToSourceWhileUntapped,
        }],
        ..creature(
            "Martyrs of Korlis",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human],
            1,
            6,
        )
    }
}

/// Orcish Mechanics — feeds artifacts into the furnace for burn.
pub fn orcish_mechanics() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..creature("Orcish Mechanics", cost(&[generic(2), r()]), vec![CreatureType::Orc], 1, 1)
    }
}

/// Phyrexian Gremlins — pins an artifact down for as long as it stays tapped.
pub fn phyrexian_gremlins() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::TapAndHoldWhileSourceTapped {
                what: target_filtered(R::Artifact),
            },
            ..Default::default()
        }],
        ..creature(
            "Phyrexian Gremlins",
            cost(&[generic(2), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Gremlin],
            1,
            1,
        )
    }
}

/// Priest of Yawgmoth — turns scrap into black mana.
pub fn priest_of_yawgmoth() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Black, Value::SacrificedManaValue),
            },
            ..Default::default()
        }],
        ..creature(
            "Priest of Yawgmoth",
            cost(&[generic(1), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Sage of Lat-Nam — cashes artifacts in for cards.
pub fn sage_of_lat_nam() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..creature(
            "Sage of Lat-Nam",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            2,
        )
    }
}

/// Yawgmoth Demon — feed it an artifact each upkeep or take the hit.
pub fn yawgmoth_demon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::MaySacrifice {
                description: "Sacrifice an artifact to Yawgmoth Demon?".into(),
                filter: R::Artifact,
                count: Value::Const(1),
                then: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::Seq(vec![
                    Effect::Tap { what: Selector::This },
                    Effect::DealDamage { to: Selector::You, amount: Value::Const(2) },
                ]))),
            },
        }],
        ..creature(
            "Yawgmoth Demon",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Demon],
            6,
            6,
        )
    }
}

// ── Enchantments ───────────────────────────────────────────────────────────

/// Artifact Possession — the enchanted artifact bites its own controller.
pub fn artifact_possession() -> CardDefinition {
    CardDefinition {
        triggered_abilities: tapped_or_activated(
            R::IsHostOfSource,
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::AttachedTo(
                    Box::new(Selector::This),
                )))),
                amount: Value::Const(2),
            },
        ),
        ..aura("Artifact Possession", cost(&[generic(2), b()]), R::Artifact)
    }
}

/// Artifact Ward — the enchanted creature is invisible to machines.
pub fn artifact_ward() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![
                Keyword::CantBeBlockedBy(Box::new(R::Artifact.and(R::Creature))),
                Keyword::PreventDamageFromMatching(Box::new(R::Artifact)),
                Keyword::CantBeTargetedByAbilitiesFromMatching(Box::new(R::Artifact)),
            ],
            ..Default::default()
        }),
        ..aura("Artifact Ward", cost(&[w()]), R::Creature)
    }
}

/// Circle of Protection: Artifacts — the metal-flavoured Circle.
pub fn circle_of_protection_artifacts() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PreventNextDamageFromChosenSource {
                reflect: false,
                filter: R::Artifact,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
                exile_top_per_prevented: false,
            },
            ..Default::default()
        }],
        ..enchantment("Circle of Protection: Artifacts", cost(&[generic(1), w()]))
    }
}

/// Power Artifact — the enchanted artifact's abilities get two cheaper.
pub fn power_artifact() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted artifact's activated abilities cost {2} less to activate. \
                          This effect can't reduce the mana in that cost to less than one mana.",
            effect: StaticEffect::AttachedActivatedAbilitiesCostLess { amount: 2 },
        }],
        ..aura("Power Artifact", cost(&[u(), u()]), R::Artifact)
    }
}

/// Titania's Song — every noncreature artifact wakes up as a blank body.
/// (The "keeps working until end of turn if this leaves" rider is dropped.)
pub fn titanias_song() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Each noncreature artifact loses all abilities.",
                effect: StaticEffect::NoncreatureArtifactsLoseAbilities,
            },
            StaticAbility {
                description: "Each noncreature artifact becomes an artifact creature with power \
                              and toughness each equal to its mana value.",
                effect: StaticEffect::NoncreatureArtifactsAreCreatures,
            },
        ],
        ..enchantment("Titania's Song", cost(&[generic(3), g()]))
    }
}

/// Damping Field — one artifact untaps per untap step, for everybody.
pub fn damping_field() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players can't untap more than one artifact during their untap steps.",
            effect: StaticEffect::MaxOneUntapPerStep { filter: R::Artifact },
        }],
        ..enchantment("Damping Field", cost(&[generic(2), w()]))
    }
}

/// Gate to Phyrexia — a creature a turn buys an artifact's destruction.
pub fn gate_to_phyrexia() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            once_per_turn: true,
            condition: Some(upkeep_only()),
            effect: Effect::Destroy { what: target_filtered(R::Artifact) },
            ..Default::default()
        }],
        ..enchantment("Gate to Phyrexia", cost(&[b(), b()]))
    }
}

/// Haunting Wind — every artifact activation costs its controller a point.
pub fn haunting_wind() -> CardDefinition {
    CardDefinition {
        triggered_abilities: tapped_or_activated(
            R::Artifact,
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::Const(1),
            },
        ),
        ..enchantment("Haunting Wind", cost(&[generic(3), b()]))
    }
}

/// Powerleech — you gain life off your opponents' artifact activity.
pub fn powerleech() -> CardDefinition {
    CardDefinition {
        triggered_abilities: tapped_or_activated(
            R::Artifact.and(R::ControlledByOpponent),
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ),
        ..enchantment("Powerleech", cost(&[g(), g()]))
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Artifact Blast — a one-mana answer to an artifact spell.
pub fn artifact_blast() -> CardDefinition {
    instant(
        "Artifact Blast",
        cost(&[r()]),
        Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::Artifact)) },
    )
}

/// Crumble — cheap artifact removal, with a consolation prize.
pub fn crumble() -> CardDefinition {
    instant(
        "Crumble",
        cost(&[g()]),
        Effect::Seq(vec![
            Effect::DestroyNoRegen { what: target_filtered(R::Artifact) },
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
        ]),
    )
}

/// Detonate — blows up an artifact and its controller alike.
pub fn detonate() -> CardDefinition {
    sorcery(
        "Detonate",
        cost(&[x(), r()]),
        Effect::Seq(vec![
            Effect::DestroyNoRegen {
                what: target_filtered(R::Artifact.and(R::ManaValueExactlyXFromCost)),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::XFromCost,
            },
        ]),
    )
}

/// Drafna's Restoration — stacks a graveyard's artifacts back on the library.
pub fn drafnas_restoration() -> CardDefinition {
    sorcery(
        "Drafna's Restoration",
        cost(&[u()]),
        Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::Target(0),
                zone: crate::card::Zone::Graveyard,
                filter: R::Artifact,
            },
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: crate::effect::LibraryPosition::Top,
            },
        },
    )
}

/// Reconstruction — one artifact back from the yard.
pub fn reconstruction() -> CardDefinition {
    CardDefinition {
        ..sorcery(
            "Reconstruction",
            cost(&[u()]),
            Effect::Move {
                what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        )
    }
}

/// Reverse Polarity — pays you back double for everything the machines did.
pub fn reverse_polarity() -> CardDefinition {
    instant(
        "Reverse Polarity",
        cost(&[w(), w()]),
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::ArtifactDamageToPlayerThisTurn { who: PlayerRef::You }),
            ),
        },
    )
}

/// Clockwork Avian — a 0/4 flier wound up to 4/4, shedding a charge each combat
/// it fights in; your upkeep can wind it back to four.
pub fn clockwork_avian() -> CardDefinition {
    let shed = || TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::DelayUntil {
            kind: crate::effect::DelayedTriggerKind::EndOfCombat,
            body: Box::new(Effect::RemoveCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusZero,
                amount: Value::ONE,
            }),
        },
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusZero, Value::Const(4))),
        triggered_abilities: vec![
            shed(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                ..shed()
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::CurrentStepIs(TurnStep::Upkeep),
                Predicate::IsTurnOf(PlayerRef::You),
            ])),
            effect: Effect::AddCounterCapped {
                what: Selector::This,
                kind: CounterType::PlusOnePlusZero,
                amount: Value::XFromCost,
                cap: Value::Const(4),
            },
            ..Default::default()
        }],
        ..artifact_creature("Clockwork Avian", cost(&[generic(5)]), vec![CreatureType::Bird], 0, 4)
    }
}

/// Goblin Artisans — gamble for a card, and eat one of your own artifact spells
/// when the coin says no.
pub fn goblin_artisans() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                on_tails: Box::new(Effect::CounterSpell {
                    what: target_filtered(
                        R::IsSpellOnStack.and(R::Artifact).and(R::ControlledByYou),
                    ),
                }),
            },
            ..Default::default()
        }],
        ..creature("Goblin Artisans", cost(&[r()]), vec![CreatureType::Goblin, CreatureType::Artificer], 1, 1)
    }
}

/// Golgothian Sylex — the Brothers' War undone: every Antiquities card on the
/// board is sacrificed, itself included.
pub fn golgothian_sylex() -> CardDefinition {
    artifact(
        "Golgothian Sylex",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::SacrificeAllMatching {
                who: Selector::Player(PlayerRef::EachPlayer),
                filter: R::Not(Box::new(R::IsToken))
                    .and(R::OriginallyPrintedIn(crate::card::OriginalSet::Antiquities)),
            },
            ..Default::default()
        }],
    )
}

/// Primal Clay — pick a body as it enters: 3/3, 2/2 flier, or 1/6 wall.
pub fn primal_clay() -> CardDefinition {
    use crate::card::EntersChoiceMode;
    CardDefinition {
        enters_as_choice: Some(vec![
            EntersChoiceMode { power: 3, toughness: 3, keywords: vec![] },
            EntersChoiceMode { power: 2, toughness: 2, keywords: vec![Keyword::Flying] },
            EntersChoiceMode { power: 1, toughness: 6, keywords: vec![Keyword::Defender] },
        ]),
        ..artifact_creature(
            "Primal Clay",
            cost(&[generic(4)]),
            vec![CreatureType::Shapeshifter],
            0,
            0,
        )
    }
}

/// Tawnos's Coffin — a creature (and its Auras) held in stasis until the Coffin
/// untaps or leaves; it comes back tapped with the counters it went in with.
pub fn tawnoss_coffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::CoffinReturn,
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
                effect: Effect::CoffinReturn,
            },
        ],
        ..artifact(
            "Tawnos's Coffin",
            cost(&[generic(4)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::CoffinExile { what: target_filtered(R::Creature) },
                ..Default::default()
            }],
        )
    }
}

/// Tetravus — trades its +1/+1 counters for flying 1/1s at upkeep, and takes
/// them back the same way.
pub fn tetravus() -> CardDefinition {
    let tetravite = crate::card::TokenDefinition {
        name: "Tetravite".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::CantBeTargetedByAuras],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource),
                effect: Effect::RemoveCountersToCreateTokens {
                    kind: CounterType::PlusOnePlusOne,
                    definition: tetravite,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource),
                effect: Effect::ExileTokensCreatedBySourceForCounters {
                    kind: CounterType::PlusOnePlusOne,
                },
            },
        ],
        ..artifact_creature(
            "Tetravus",
            cost(&[generic(6)]),
            vec![CreatureType::Construct],
            1,
            1,
        )
    }
}

/// Transmute Artifact — trade one artifact for any other, paying the difference
/// in mana (or watching it hit the graveyard).
pub fn transmute_artifact() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Artifact.and(R::ControlledByYou),
            count: 1,
        }],
        ..sorcery("Transmute Artifact", cost(&[u(), u()]), Effect::TransmuteArtifact)
    }
}

/// Urza's Avenger — shrinks itself to buy an evasion keyword, over and over.
pub fn urzas_avenger() -> CardDefinition {
    let grant = |kw: Keyword| Effect::GrantKeyword {
        what: Selector::This,
        keyword: kw,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                Effect::ChooseMode(vec![
                    grant(Keyword::Banding),
                    grant(Keyword::Flying),
                    grant(Keyword::FirstStrike),
                    grant(Keyword::Trample),
                ]),
            ]),
            ..Default::default()
        }],
        ..artifact_creature(
            "Urza's Avenger",
            cost(&[generic(6)]),
            vec![CreatureType::Shapeshifter],
            4,
            4,
        )
    }
}

/// Xenic Poltergeist — animates an artifact at its own mana value until your
/// next upkeep.
pub fn xenic_poltergeist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeCreature {
                what: target_filtered(R::Artifact.and(R::Not(Box::new(R::Creature)))),
                power: Value::ManaValueOf(Box::new(Selector::Target(0))),
                toughness: Value::ManaValueOf(Box::new(Selector::Target(0))),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::UntilYourNextUpkeep,
            },
            ..Default::default()
        }],
        ..creature("Xenic Poltergeist", cost(&[generic(1), b(), b()]), vec![CreatureType::Spirit], 1, 1)
    }
}
