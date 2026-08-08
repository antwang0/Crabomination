//! Murders at Karlov Manor (MKM) — 2024. Detective set introducing the
//! Suspect (CR 701.60) and Collect Evidence (CR 701.59) keyword actions.

use crate::card::{ActivatedAbility, TokenDefinition};
use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement, Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{draw, etb, lose_life, on_attack, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate};
use crate::game::effects::clue_token;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Repeat Offender — {1}{B} 2/1 Human Assassin. "{2}{B}: If this creature is
/// suspected, put a +1/+1 counter on it. Otherwise, suspect it."
pub fn repeat_offender() -> CardDefinition {
    CardDefinition {
        name: "Repeat Offender",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::If {
                cond: Predicate::SourceIsSuspected,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Suspect {
                    what: Selector::This,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Reasonable Doubt — {1}{U} Instant. "Counter target spell unless its
/// controller pays {2}. Suspect up to one target creature." The suspect slot
/// is optional — the spell resolves (countering) with no creature supplied.
pub fn reasonable_doubt() -> CardDefinition {
    CardDefinition {
        name: "Reasonable Doubt",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                mana_cost: cost(&[generic(2)]),
                exile: false,
                extra_generic: None,
            },
            Effect::Suspect {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature,
                },
            },
        ]),
        ..Default::default()
    }
}

/// Sample Collector — {2}{G} 2/3 Troll Detective. "Whenever this attacks, you
/// may collect evidence 3. When you do, put a +1/+1 counter on target
/// creature you control."
pub fn sample_collector() -> CardDefinition {
    CardDefinition {
        name: "Sample Collector",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::CollectEvidence {
            amount: Value::Const(3),
            then: Box::new(Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..Default::default()
    }
}

/// Barbed Servitor — {3}{B} 1/1 Artifact Creature — Construct. Indestructible;
/// ETB suspect itself; combat damage to a player → draw + lose 1 life; when
/// dealt damage, each opponent loses that much life (modeled as each opponent
/// rather than a single target).
pub fn barbed_servitor() -> CardDefinition {
    CardDefinition {
        name: "Barbed Servitor",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![
            etb(Effect::Suspect {
                what: Selector::This,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![draw(1), lose_life(1, Selector::You)]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::TriggerEventAmount,
                },
            },
        ],
        ..Default::default()
    }
}

// ── Investigate (Clue tokens) ────────────────────────────────────────────────

/// Deduce — {1}{U} Instant. "Draw a card. Investigate." (Investigate mints a
/// Clue token via `clue_token()`.)
pub fn deduce() -> CardDefinition {
    CardDefinition {
        name: "Deduce",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            draw(1),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(clue_token()),
            },
        ]),
        ..Default::default()
    }
}

/// Novice Inspector — {W} 1/2 Human Detective. "When this enters, investigate."
pub fn novice_inspector() -> CardDefinition {
    CardDefinition {
        name: "Novice Inspector",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: Box::new(clue_token()),
        })],
        ..Default::default()
    }
}

/// Izoni, Center of the Web — {4}{B}{G} 5/4 Legendary Elf Detective with
/// menace. "Whenever Izoni enters or attacks, you may collect evidence 4. If
/// you do, create two 2/1 black and green Spider tokens with menace and reach."
/// (The sacrifice-four-tokens activated ability is omitted.)
pub fn izoni_center_of_the_web() -> CardDefinition {
    let spider = || TokenDefinition {
        name: "Spider".into(),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace, Keyword::Reach],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        ..Default::default()
    };
    let collect = || Effect::CollectEvidence {
        amount: Value::Const(4),
        then: Box::new(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: Box::new(spider()),
        }),
    };
    CardDefinition {
        name: "Izoni, Center of the Web",
        cost: cost(&[generic(4), b(), g()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Detective],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(collect()), on_attack(collect())],
        ..Default::default()
    }
}

// ── More MKM ─────────────────────────────────────────────────────────────────

/// A 2/2 white-and-blue Detective creature token (Person of Interest, Inside
/// Source).
fn detective_token() -> TokenDefinition {
    TokenDefinition {
        name: "Detective".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Detective],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Cold Case Cracker — {3}{U} 3/3 Spirit Detective with flying. "When this
/// dies, investigate."
pub fn cold_case_cracker() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    CardDefinition {
        name: "Cold Case Cracker",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Detective],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: Box::new(clue_token()),
        })],
        ..Default::default()
    }
}

/// Not on My Watch — {1}{W} Instant. "Exile target attacking creature."
pub fn not_on_my_watch() -> CardDefinition {
    CardDefinition {
        name: "Not on My Watch",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::IsAttacking),
            },
        },
        ..Default::default()
    }
}

/// Person of Interest — {3}{R} 2/2 Human Rogue. "When this enters, suspect it.
/// Create a 2/2 white and blue Detective creature token."
pub fn person_of_interest() -> CardDefinition {
    CardDefinition {
        name: "Person of Interest",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Suspect {
                what: Selector::This,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(detective_token()),
            },
        ]))],
        ..Default::default()
    }
}

/// Get a Leg Up — {G} Instant. "Until end of turn, target creature gets +1/+1
/// for each creature you control and gains reach."
pub fn get_a_leg_up() -> CardDefinition {
    use crate::effect::Duration;
    let count = Value::CreatureCountControlledBy(PlayerRef::You);
    CardDefinition {
        name: "Get a Leg Up",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: count.clone(),
                toughness: count,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Inside Source — {2}{W} 1/1 Human Citizen. "When this enters, create a 2/2
/// white and blue Detective creature token." (The pump-a-Detective activated
/// ability is omitted.)
pub fn inside_source() -> CardDefinition {
    CardDefinition {
        name: "Inside Source",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: Box::new(detective_token()),
        })],
        ..Default::default()
    }
}

/// Slimy Dualleech — {3}{B} 2/4 Leech. "At the beginning of combat on your
/// turn, target creature you control with power 2 or less gets +1/+0 and gains
/// deathtouch until end of turn."
pub fn slimy_dualleech() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec};
    use crate::effect::Duration;
    let target = || Selector::TargetFiltered {
        slot: 0,
        filter: SelectionRequirement::Creature
            .and(SelectionRequirement::ControlledByYou)
            .and(SelectionRequirement::PowerAtMost(2)),
    };
    CardDefinition {
        name: "Slimy Dualleech",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Leech],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target(),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target(),
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── 2026-08 gap wave ──────────────────────────────────────────────────────

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

fn legend(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        ..creature(name, c, types, p, t)
    }
}

/// Teysa, Opulent Oligarch — {1}{W}{B} 2/3 deathtouch; investigates for each
/// bloodied opponent at end step and turns spent Clues into Spirits.
pub fn teysa_opulent_oligarch() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::YourControl,
                ),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::OpponentsWhoLostLifeThisTurn,
                    definition: Box::new(clue_token()),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasName("Clue".to_string()),
                    })
                    .once_per_turn(),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(spirit_token()),
                },
            },
        ],
        ..legend(
            "Teysa, Opulent Oligarch",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            2,
            3,
        )
    }
}

/// The 1/1 white-and-black flying Spirit MKM keeps minting.
fn spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".to_string(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White, Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Soul Search — {W}{B} Sorcery. Strip a nonland card out of an opponent's
/// hand; a cheap one leaves a Spirit behind.
pub fn soul_search() -> CardDefinition {
    CardDefinition {
        name: "Soul Search",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileChosenFromHand {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: SelectionRequirement::Nonland,
                face_down: false,
                link_to_source: false,
            },
            Effect::If {
                cond: Predicate::ValueAtMost(
                    Value::ManaValueOf(Box::new(Selector::LastMoved)),
                    Value::ONE,
                ),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(spirit_token()),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Tolsimir, Midnight's Light — {2}{G}{W}{W} 3/2 lifelink that brings Voja and
/// forces a block on an attacking Wolf.
pub fn tolsimir_midnights_light() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(TokenDefinition {
                name: "Voja Fenstalker".to_string(),
                power: 5,
                toughness: 5,
                colors: vec![Color::Green, Color::White],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Wolf],
                    ..Default::default()
                },
                keywords: vec![Keyword::Trample],
                supertypes: vec![crate::card::Supertype::Legendary],
                ..Default::default()
            }),
        })],
        ..legend(
            "Tolsimir, Midnight's Light",
            cost(&[generic(2), g(), w(), w()]),
            vec![CreatureType::Elf, CreatureType::Scout],
            3,
            2,
        )
    }
}

/// Krenko's Buzzcrusher — {2}{R}{R} 4/4 flying trample; its entry blows up a
/// nonbasic land per player, each of whom may fetch a basic.
pub fn krenkos_buzzcrusher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Land
                        .and(SelectionRequirement::Not(Box::new(SelectionRequirement::IsBasicLand))),
                ),
            },
            Effect::Search {
                who: PlayerRef::EachPlayer,
                filter: SelectionRequirement::IsBasicLand,
                to: crate::effect::ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
        ]))],
        ..CardDefinition {
            card_types: vec![CardType::Artifact, CardType::Creature],
            ..creature(
                "Krenko's Buzzcrusher",
                cost(&[generic(2), r(), r()]),
                vec![CreatureType::Insect, CreatureType::Thopter],
                4,
                4,
            )
        }
    }
}

/// Repulsive Mutation — {X}{G}{U} Instant. Grow a creature, then tax a spell
/// by your biggest body.
pub fn repulsive_mutation() -> CardDefinition {
    CardDefinition {
        name: "Repulsive Mutation",
        cost: cost(&[crate::mana::x(), g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::XFromCost,
            },
            Effect::CounterUnlessPaid {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::IsSpellOnStack,
                },
                mana_cost: cost(&[]),
                exile: false,
                extra_generic: Some(Value::GreatestPowerControlled { who: PlayerRef::You }),
            },
        ]),
        ..Default::default()
    }
}

/// Lost in the Maze — {X}{U}{U} Enchantment with flash. Freezes X creatures
/// and hides your tapped ones.
pub fn lost_in_the_maze() -> CardDefinition {
    CardDefinition {
        name: "Lost in the Maze",
        cost: cost(&[crate::mana::x(), u(), u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::TapUpToValue {
                count: Value::XFromCost,
                filter: SelectionRequirement::Creature,
                skip_untap: false,
                exact: true,
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::Stun,
                amount: Value::ONE,
            },
        ]))],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Tapped creatures you control have hexproof.",
            effect: crate::effect::StaticEffect::AnthemForFilter {
                filter: SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Hexproof],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Relive the Past — {5}{G}{W} Sorcery. Reanimates an artifact, a land and an
/// enchantment as 5/5 Elementals.
pub fn relive_the_past() -> CardDefinition {
    CardDefinition {
        name: "Relive the Past",
        cost: cost(&[generic(5), g(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Artifact
                        .and(SelectionRequirement::InYourGraveyard),
                },
                to: crate::effect::ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Land
                        .and(SelectionRequirement::InYourGraveyard),
                },
                to: crate::effect::ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 2,
                    filter: SelectionRequirement::HasCardType(CardType::Enchantment)
                        .and(SelectionRequirement::InYourGraveyard)
                        .and(SelectionRequirement::Not(Box::new(
                            SelectionRequirement::HasEnchantmentSubtype(
                                crate::card::EnchantmentSubtype::Aura,
                            ),
                        ))),
                },
                to: crate::effect::ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::BecomeCreature {
                what: Selector::LastMoved,
                power: Value::Const(5),
                toughness: Value::Const(5),
                creature_types: vec![CreatureType::Elemental],
                keywords: vec![],
                duration: crate::effect::Duration::Permanent,
            },
        ]),
        ..Default::default()
    }
}

/// Delney, Streetwise Lookout — {2}{W} 2/2 that shields your small creatures
/// and doubles their triggers.
pub fn delney_streetwise_lookout() -> CardDefinition {
    let small = || SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2));
    CardDefinition {
        static_abilities: vec![
            crate::card::StaticAbility {
                description: "Creatures you control with power 2 or less can't be blocked by \
                              creatures with power 3 or greater.",
                effect: crate::effect::StaticEffect::AnthemForFilter {
                    filter: small(),
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::CantBeBlockedByPowerAtLeast(3)],
                    opponents: false,
                    all_players: false,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
            crate::card::StaticAbility {
                description: "If a triggered ability of a creature you control with power 2 or \
                              less triggers, that ability triggers an additional time.",
                effect: crate::effect::StaticEffect::DoubleControllerTriggersMatching {
                    filter: small(),
                },
            },
        ],
        ..legend(
            "Delney, Streetwise Lookout",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Scout],
            2,
            2,
        )
    }
}

/// Vannifar, Evolved Enigma — {2}{G}{U} 3/4 that either cloaks a card from
/// hand or grows your colorless team each combat.
pub fn vannifar_evolved_enigma() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::ChooseMode(vec![
                Effect::Cloak {
                    who: PlayerRef::You,
                    amount: Value::ONE,
                    from_hand: true,
                },
                Effect::AddCounter {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::Colorless),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..legend(
            "Vannifar, Evolved Enigma",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Ooze, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Tomik, Wielder of Law — {1}{W}{B} 2/4 flier that punishes a wide swing.
pub fn tomik_wielder_of_law() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        // Affinity for planeswalkers.
        self_cost_reduction_per: Some((
            Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                filter: SelectionRequirement::HasCardType(CardType::Planeswalker),
            },
            1,
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::OpponentControl)
                .with_filter(Predicate::AttackedWithCountAtLeast {
                    who: PlayerRef::Triggerer,
                    at_least: 2,
                }),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(3),
                },
                draw(1),
            ]),
        }],
        ..legend(
            "Tomik, Wielder of Law",
            cost(&[generic(1), w(), b()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            2,
            4,
        )
    }
}

/// The Pride of Hull Clade — {10}{G} 2/15 defender that gets cheap behind a
/// wall of toughness and turns a creature into a draw engine.
pub fn the_pride_of_hull_clade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        self_cost_reduction_per: Some((
            Value::TotalToughnessControlled,
            1,
        )),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), u()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: crate::effect::Duration::EndOfTurn,
                },
                Effect::GrantKeywords {
                    what: Selector::Target(0),
                    keywords: vec![Keyword::AttacksAsThoughNoDefender],
                    duration: crate::effect::Duration::EndOfTurn,
                },
                Effect::GrantTriggeredAbility {
                    what: Selector::Target(0),
                    duration: crate::effect::Duration::EndOfTurn,
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::DealsCombatDamageToPlayer,
                            EventScope::SelfSource,
                        ),
                        effect: Effect::Draw {
                            who: Selector::You,
                            amount: Value::ToughnessOf(Box::new(Selector::This)),
                        },
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "The Pride of Hull Clade",
            cost(&[generic(10), g()]),
            vec![CreatureType::Crocodile, CreatureType::Elk, CreatureType::Turtle],
            2,
            15,
        )
    }
}

/// Public Thoroughfare — a tapped any-color land you have to pay for by
/// tapping something else.
pub fn public_thoroughfare() -> CardDefinition {
    CardDefinition {
        name: "Public Thoroughfare",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::card::StaticAbility {
            description: "This land enters tapped.",
            effect: crate::effect::StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessTapMatching {
            filter: SelectionRequirement::Artifact.or(SelectionRequirement::Land),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::AnyColors(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Officious Interrogation — {W}{U} Instant. Investigate once per creature the
/// target player controls.
pub fn officious_interrogation() -> CardDefinition {
    CardDefinition {
        name: "Officious Interrogation",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::CountMatching {
                sel: Box::new(Selector::ControlledBy {
                    who: PlayerRef::Target(0),
                    filter: SelectionRequirement::Creature,
                }),
                filter: SelectionRequirement::Any,
            },
            definition: Box::new(clue_token()),
        },
        ..Default::default()
    }
}

/// Intrude on the Mind — {3}{U}{U} Instant. Split five cards; the half you
/// lose sizes the Thopter you keep.
pub fn intrude_on_the_mind() -> CardDefinition {
    CardDefinition {
        name: "Intrude on the Mind",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SeparateIntoPiles {
                what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(5) },
                splitter: PlayerRef::You,
                chooser: PlayerRef::Target(0),
                chosen: Box::new(Effect::Move {
                    what: Selector::SeparatedPile { chosen: true },
                    to: crate::effect::ZoneDest::Hand(PlayerRef::You),
                }),
                other: Box::new(Effect::Move {
                    what: Selector::SeparatedPile { chosen: false },
                    to: crate::effect::ZoneDest::Graveyard,
                }),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Thopter".to_string(),
                    power: 0,
                    toughness: 0,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Thopter],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                }),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CardsMilledThisEffectMatching {
                    filter: SelectionRequirement::Any,
                },
            },
        ]),
        ..Default::default()
    }
}

/// Fugitive Codebreaker — {1}{R} 2/1 prowess haste; unmasking it refills your
/// hand, and its disguise cost shrinks per instant/sorcery in your graveyard.
pub fn fugitive_codebreaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Prowess,
            Keyword::Haste,
            Keyword::Disguise(cost(&[generic(5), r()])),
        ],
        disguise_cost_reduction_per: Some(Value::CountMatching {
            sel: Box::new(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: SelectionRequirement::Any,
            }),
            filter: SelectionRequirement::HasCardType(CardType::Instant)
                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::CardsInHandMatching {
                        who: PlayerRef::You,
                        filter: SelectionRequirement::Any,
                    },
                    random: false,
                },
                draw(3),
            ]),
        }],
        ..creature(
            "Fugitive Codebreaker",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin, CreatureType::Rogue],
            2,
            1,
        )
    }
}

/// Unyielding Gatekeeper — {1}{W} 3/2 disguise; unmasking it blinks your own
/// permanent or exiles theirs for a Detective.
pub fn unyielding_gatekeeper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Disguise(cost(&[generic(1), w()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::ExileThenBranchByController {
                what: target_filtered(
                    SelectionRequirement::Nonland.and(SelectionRequirement::OtherThanSource),
                ),
                theirs: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(crate::game::effects::detective_token()),
                }),
            },
        }],
        ..creature(
            "Unyielding Gatekeeper",
            cost(&[generic(1), w()]),
            vec![CreatureType::Elephant, CreatureType::Cleric],
            3,
            2,
        )
    }
}
