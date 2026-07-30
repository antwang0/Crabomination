//! BFZ gap wave 2 — the Retreat cycle, the landfall rares, and the
//! planeswalkers.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, Keyword,
    LandType, LoyaltyAbility, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, Value,
};
use crate::effect::shortcut::{
    cast_is_instant_or_sorcery, drain, each_your_creature, etb, gain_life, landfall, pump_target,
    rally, target_filtered,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};
use crabomination_base::tokens::eldrazi_scion_token;

/// "Whenever you cast an instant or sorcery spell, …" — the spell is the
/// trigger source.
fn on_your_is_cast(effect: Effect) -> crate::card::TriggeredAbility {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(cast_is_instant_or_sorcery()),
        effect,
    }
}

fn enchantment(name: &'static str, c: crate::mana::ManaCost, t: Vec<crate::card::TriggeredAbility>)
-> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        triggered_abilities: t,
        ..Default::default()
    }
}

// ── The Retreat cycle — modal landfall ──────────────────────────────────────

/// Retreat to Coralhelm — {2}{U}. Landfall: tap or untap a creature, or scry 1.
pub fn retreat_to_coralhelm() -> CardDefinition {
    enchantment(
        "Retreat to Coralhelm",
        cost(&[generic(2), u()]),
        vec![landfall(Effect::ChooseMode(vec![
            Effect::TapOrUntap { what: target_filtered(R::Creature) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]))],
    )
}

/// Retreat to Emeria — {3}{W}. Landfall: a 1/1 Kor Ally, or your creatures get
/// +1/+1 until end of turn.
pub fn retreat_to_emeria() -> CardDefinition {
    let kor = TokenDefinition {
        name: "Kor Ally".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Ally],
            ..Default::default()
        },
        ..Default::default()
    };
    enchantment(
        "Retreat to Emeria",
        cost(&[generic(3), w()]),
        vec![landfall(Effect::ChooseMode(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: kor },
            Effect::PumpPT {
                what: each_your_creature(),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        ]))],
    )
}

/// Retreat to Hagra — {2}{B}. Landfall: +1/+0 and deathtouch on a creature, or
/// drain 1.
pub fn retreat_to_hagra() -> CardDefinition {
    enchantment(
        "Retreat to Hagra",
        cost(&[generic(2), b()]),
        vec![landfall(Effect::ChooseMode(vec![
            Effect::Seq(vec![
                pump_target(1, 0),
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
            ]),
            drain(1),
        ]))],
    )
}

/// Retreat to Kazandu — {2}{G}. Landfall: a +1/+1 counter on a creature, or gain
/// 2 life.
pub fn retreat_to_kazandu() -> CardDefinition {
    enchantment(
        "Retreat to Kazandu",
        cost(&[generic(2), g()]),
        vec![landfall(Effect::ChooseMode(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            gain_life(2),
        ]))],
    )
}

/// Retreat to Valakut — {2}{R}. Landfall: +2/+0 on a creature, or a creature
/// can't block this turn.
pub fn retreat_to_valakut() -> CardDefinition {
    enchantment(
        "Retreat to Valakut",
        cost(&[generic(2), r()]),
        vec![landfall(Effect::ChooseMode(vec![
            pump_target(2, 0),
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
        ]))],
    )
}

// ── Landfall rares ──────────────────────────────────────────────────────────

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

/// Guardian of Tazeem — {3}{U}{U} 4/5 Sphinx with flying. Landfall: tap an
/// opponent's creature; an Island also stops it untapping.
pub fn guardian_of_tazeem() -> CardDefinition {
    let tap = Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![landfall(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasLandType(LandType::Island),
            },
            then: Box::new(Effect::Seq(vec![
                tap.clone(),
                Effect::SkipNextUntap { what: Selector::Target(0) },
            ])),
            else_: Box::new(tap),
        })],
        ..creature(
            "Guardian of Tazeem",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Sphinx],
            4,
            5,
        )
    }
}

/// Guul Draz Overseer — {4}{B}{B} 3/4 Vampire with flying. Landfall: your other
/// creatures get +1/+0, or +2/+0 off a Swamp.
pub fn guul_draz_overseer() -> CardDefinition {
    let others = Selector::EachPermanent(
        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
    );
    let pump = |n| Effect::PumpPT {
        what: others.clone(),
        power: Value::Const(n),
        toughness: Value::Const(0),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![landfall(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasLandType(LandType::Swamp),
            },
            then: Box::new(pump(2)),
            else_: Box::new(pump(1)),
        })],
        ..creature(
            "Guul Draz Overseer",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Vampire],
            3,
            4,
        )
    }
}

/// Emeria Shepherd — {5}{W}{W} 4/4 Angel with flying. Landfall: return a
/// nonland permanent card from your graveyard to hand — to the battlefield off
/// a Plains.
pub fn emeria_shepherd() -> CardDefinition {
    let target = || Selector::TargetFiltered {
        slot: 0,
        filter: R::InYourGraveyard.and(R::PermanentCard).and(R::Nonland),
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![landfall(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasLandType(LandType::Plains),
            },
            then: Box::new(Effect::Move {
                what: target(),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            else_: Box::new(Effect::Move {
                what: target(),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature(
            "Emeria Shepherd",
            cost(&[generic(5), w(), w()]),
            vec![CreatureType::Angel],
            4,
            4,
        )
    }
}

/// Sire of Stagnation — {4}{U}{B} 5/7 Eldrazi. Devoid. Whenever a land an
/// opponent controls enters, they exile their top two and you draw two.
pub fn sire_of_stagnation() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::OpponentControl),
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                    link_to_source: false,
                    face_down: false,
                },
                crate::effect::shortcut::draw(2),
            ]),
        }],
        ..creature(
            "Sire of Stagnation",
            cost(&[generic(4), u(), b()]),
            vec![CreatureType::Eldrazi],
            5,
            7,
        )
    }
}

// ── Allies & utility ────────────────────────────────────────────────────────

/// Hagra Sharpshooter — {2}{B} 2/2 Human Assassin Ally. {4}{B}: a creature gets
/// -1/-1 until end of turn.
pub fn hagra_sharpshooter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Hagra Sharpshooter",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Ally],
            2,
            2,
        )
    }
}

/// Halimar Tidecaller — {2}{U} 2/3 Human Wizard Ally. ETB: return an awaken
/// card from your graveyard; your land creatures have flying.
pub fn halimar_tidecaller() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::InYourGraveyard.and(R::HasAwaken),
            },
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        static_abilities: vec![StaticAbility {
            description: "Land creatures you control have flying.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Land.and(R::Creature),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Flying],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..creature(
            "Halimar Tidecaller",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Ally],
            2,
            3,
        )
    }
}

/// Herald of Kozilek — {1}{U}{R} 2/4 Eldrazi Drone. Devoid; colorless spells you
/// cast cost {1} less.
pub fn herald_of_kozilek() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Devoid],
        static_abilities: vec![StaticAbility {
            description: "Colorless spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Colorless, amount: 1 },
        }],
        ..creature(
            "Herald of Kozilek",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Eldrazi, CreatureType::Drone],
            2,
            4,
        )
    }
}

/// Munda, Ambush Leader — {2}{R}{W} 3/4 Kor Ally with haste. Rally: look at the
/// top four and stack any Allies on top.
pub fn munda_ambush_leader() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![rally(Effect::LookTopKeepMatchingOnTop {
            who: PlayerRef::You,
            count: Value::Const(4),
            take: Value::Const(4),
            filter: R::HasCreatureType(CreatureType::Ally),
        })],
        ..creature(
            "Munda, Ambush Leader",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Kor, CreatureType::Ally],
            3,
            4,
        )
    }
}

/// Hedron Blade — {1} Equipment. +1/+1, and deathtouch when the equipped
/// creature is blocked by a colorless creature. Equip {2}.
pub fn hedron_blade() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Hedron Blade",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Colorless,
                    }),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Adverse Conditions — {3}{U} Instant. Devoid. Tap up to two creatures, which
/// don't untap next untap step; create an Eldrazi Scion.
pub fn adverse_conditions() -> CardDefinition {
    CardDefinition {
        name: "Adverse Conditions",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    Effect::Tap { what: Selector::Target(0) },
                    Effect::SkipNextUntap { what: Selector::Target(0) },
                ])),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: eldrazi_scion_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Prism Array — {4}{U} Enchantment. Converge: enters with a charge counter per
/// color spent. Remove one: tap a creature. {W}{U}{B}{R}{G}: scry 3.
pub fn prism_array() -> CardDefinition {
    CardDefinition {
        name: "Prism Array",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        enters_with_counters: Some((CounterType::Charge, Value::ConvergedValue)),
        activated_abilities: vec![
            ActivatedAbility {
                remove_counter_cost: Some((CounterType::Charge, 1)),
                effect: Effect::Tap { what: target_filtered(R::Creature) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w(), u(), b(), r(), g()]),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Noyan Dar, Roil Shaper — {3}{W}{U} 4/4 Merfolk Ally. Casting an instant or
/// sorcery may awaken a land you control with three counters.
pub fn noyan_dar_roil_shaper() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_your_is_cast(Effect::MayDo {
            description: "Put three +1/+1 counters on a land you control?".into(),
            body: Box::new(crate::effect::shortcut::animate_land(0, 3)),
        })],
        ..creature(
            "Noyan Dar, Roil Shaper",
            cost(&[generic(3), w(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Ally],
            4,
            4,
        )
    }
}

// ── Planeswalkers ───────────────────────────────────────────────────────────

/// Gideon, Ally of Zendikar — {2}{W}{W} loyalty 4.
pub fn gideon_ally_of_zendikar() -> CardDefinition {
    use crate::card::PlaneswalkerSubtype;
    let knight = TokenDefinition {
        name: "Knight Ally".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Knight, CreatureType::Ally],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Gideon, Ally of Zendikar",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Gideon],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(5),
                        toughness: Value::Const(5),
                        creature_types: vec![
                            CreatureType::Human,
                            CreatureType::Soldier,
                            CreatureType::Ally,
                        ],
                        keywords: vec![Keyword::Indestructible],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::PreventAllDamageThisTurn { target: Selector::This },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: knight,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -4,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Creatures you control get +1/+1.".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "Creatures you control get +1/+1.",
                        effect: StaticEffect::AnthemForFilter {
                            filter: R::Creature,
                            power: 1,
                            toughness: 1,
                            keywords: vec![],
                            opponents: false,
                            all_players: false,
                            only_your_turn: false,
                            scale_by_counters_on_self: None,
                        },
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ob Nixilis Reignited — {3}{B}{B} loyalty 5.
pub fn ob_nixilis_reignited() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, PlaneswalkerSubtype, TriggeredAbility};
    CardDefinition {
        name: "Ob Nixilis Reignited",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Nixilis],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    crate::effect::shortcut::draw(1),
                    crate::effect::shortcut::lose_life(1, Selector::You),
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Destroy { what: target_filtered(R::Creature) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::Target(0),
                    name: "Whenever a player draws a card, you lose 2 life.".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::CardDrawn, EventScope::AnyPlayer),
                        effect: crate::effect::shortcut::lose_life(2, Selector::You),
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ondu Rising — {1}{W} Sorcery. Attackers gain lifelink this turn.
/// Awaken 4—{4}{W}.
pub fn ondu_rising() -> CardDefinition {
    let body = Effect::OnMatchingAttacksThisTurn {
        filter: R::Creature,
        body: Box::new(Effect::GrantKeyword {
            what: Selector::TriggerSource,
            keyword: Keyword::Lifelink,
            duration: Duration::EndOfTurn,
        }),
    };
    CardDefinition {
        name: "Ondu Rising",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: body.clone(),
        alternative_cost: Some(crate::effect::shortcut::awaken(
            4,
            cost(&[generic(4), w()]),
            0,
            body,
        )),
        ..Default::default()
    }
}

/// Brutal Expulsion — {2}{U}{R} Instant. Devoid. Choose one or both: bounce a
/// spell or creature; deal 2 damage to a creature or planeswalker, exiling it
/// if it would die.
pub fn brutal_expulsion() -> CardDefinition {
    CardDefinition {
        name: "Brutal Expulsion",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Devoid],
        effect: Effect::ChooseModesCast {
            min: 1,
            max: 2,
            allow_repeats: false,
            modes: vec![
                Effect::Move {
                    what: target_filtered(R::IsSpellOnStack.or(R::Creature)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                Effect::Seq(vec![
                    Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
                    Effect::DealDamage {
                        to: target_filtered(R::Creature.or(R::Planeswalker)),
                        amount: Value::Const(2),
                    },
                ]),
            ],
        },
        ..Default::default()
    }
}

/// March from the Tomb — {3}{W}{B} Sorcery. Reanimate Ally creature cards from
/// your graveyard with total mana value 8 or less.
pub fn march_from_the_tomb() -> CardDefinition {
    CardDefinition {
        name: "March from the Tomb",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::MoveWithinTotalManaValue {
            from: Selector::EachMatching {
                zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                filter: R::Creature.and(R::HasCreatureType(CreatureType::Ally)),
            },
            filter: R::Creature.and(R::HasCreatureType(CreatureType::Ally)),
            cap: Value::Const(8),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Kiora, Master of the Depths — {2}{G}{U} loyalty 4.
pub fn kiora_master_of_the_depths() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, PlaneswalkerSubtype, TriggeredAbility};
    let octopus = TokenDefinition {
        name: "Octopus".into(),
        power: 8,
        toughness: 8,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Octopus],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Kiora, Master of the Depths",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Kiora],
            ..Default::default()
        },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Untap { what: target_filtered(R::Creature), up_to: None },
                    Effect::Untap {
                        what: Selector::TargetFiltered { slot: 1, filter: R::Land },
                        up_to: None,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::LookPickToHand {
                    who: PlayerRef::You,
                    count: Value::Const(4),
                    rest_to_graveyard: true,
                    pick_filter: Some(R::Creature.or(R::Land)),
                    take: Some(Value::Const(2)),
                    to_battlefield: false,
                    gain_life_if_pick: None,
                    gain_life_greatest_power_rest: false,
                    optional: true,
                    picked_lands_to_battlefield: false,
                    rest_bottom_random: false,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::Seq(vec![
                    Effect::CreateEmblem {
                        who: PlayerRef::You,
                        name: "Whenever a creature you control enters, you may have it fight \
                               target creature."
                            .into(),
                        triggered: vec![TriggeredAbility {
                            event: EventSpec::new(
                                EventKind::EntersBattlefield,
                                EventScope::YourControl,
                            )
                            .with_filter(Predicate::EntityMatches {
                                what: Selector::TriggerSource,
                                filter: R::Creature,
                            }),
                            effect: Effect::Fight {
                                attacker: Selector::TriggerSource,
                                defender: target_filtered(R::Creature),
                            },
                        }],
                        statics: vec![],
                    },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(3),
                        definition: octopus,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Zada, Hedron Grinder — {3}{R} 3/3 Goblin Ally. An instant or sorcery that
/// targets only Zada is copied for each other creature you control it could
/// target, one copy each.
pub fn zada_hedron_grinder() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_your_is_cast(Effect::CopyForEachOtherTargetableCreature)],
        ..creature(
            "Zada, Hedron Grinder",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Ally],
            3,
            3,
        )
    }
}

/// Gruesome Slaughter — {6} Sorcery. Your colorless creatures gain "{T}: deal
/// damage equal to this creature's power to target creature" this turn.
pub fn gruesome_slaughter() -> CardDefinition {
    CardDefinition {
        name: "Gruesome Slaughter",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GainActivatedAbility {
            what: Selector::EachPermanent(
                R::Creature.and(R::Colorless).and(R::ControlledByYou),
            ),
            ability: Box::new(ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamageEqualToPower {
                    source: Selector::This,
                    target: target_filtered(R::Creature),
                },
                ..Default::default()
            }),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}
