//! Outlaws of Thunder Junction gap batch 1 — the plot payoffs, the Mount /
//! saddle legends, and the "Joins Up" enchantment cycle. Tests in
//! `recent_b/recent_309_310`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{LookPick, 
    CounteredSpellZone, Duration, Effect, ManaPayload, PlayerRef, Predicate, SpreeMode,
    StaticAbility, StaticEffect, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// A legendary creature body with no default abilities.
fn legend(
    name: &'static str,
    mana: crate::mana::ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}

/// A legendary enchantment shell for the "Joins Up" cycle. `ongoing: None`
/// for the member whose second ability is static rather than triggered
/// (Annie) — a filler `legend_enters(Noop)` gave it a trigger that fires on
/// every legendary creature entering and resolves to nothing.
fn joins_up(
    name: &'static str,
    mana: crate::mana::ManaCost,
    etb: Effect,
    ongoing: Option<TriggeredAbility>,
) -> CardDefinition {
    let mut triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: etb,
    }];
    triggered_abilities.extend(ongoing);
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities,
        ..Default::default()
    }
}

/// "Whenever a legendary creature you control enters" — the shared trigger of
/// the Joins Up cycle's second ability.
fn legend_enters(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
            }),
        effect,
    }
}

/// The OTJ 1/1 red Mercenary token with the shared sorcery-speed pump.
fn mercenary_token() -> TokenDefinition {
    TokenDefinition {
        name: "Mercenary".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mercenary],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// One Spree additional-cost mode (CR 702.172).
fn mode(c: crate::mana::ManaCost, effect: Effect) -> SpreeMode {
    SpreeMode { cost: c, effect }
}

// ── Plot payoffs (CR 702.170) ───────────────────────────────────────────────

/// Kellan Joins Up — {G}{W}{U} legendary enchantment. ETB: you may plot a
/// nonland card with mana value 3 or less from your hand. Whenever a legendary
/// creature you control enters, put a +1/+1 counter on each creature you
/// control.
pub fn kellan_joins_up() -> CardDefinition {
    joins_up(
        "Kellan Joins Up",
        cost(&[g(), w(), u()]),
        Effect::MoveChosen {
            from: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Hand,
                filter: R::Not(Box::new(R::Land)).and(R::ManaValueAtMost(3)),
            },
            filter: None,
            count: Value::ONE,
            up_to: true,
            to: ZoneDest::ExilePlotted,
        },
        Some(legend_enters(Effect::AddCounter {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })),
    )
}

/// Make Your Own Luck — {3}{G}{U} Sorcery. Look at the top three cards of your
/// library. You may plot a nonland card from among them; put the rest into
/// your hand.
pub fn make_your_own_luck() -> CardDefinition {
    CardDefinition {
        name: "Make Your Own Luck",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::MoveChosen {
                from: Selector::TopOfLibrary {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                },
                filter: Some(R::Not(Box::new(R::Land))),
                count: Value::ONE,
                up_to: true,
                to: ZoneDest::ExilePlotted,
            },
            Effect::Move {
                what: Selector::TopOfLibrary {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Aven Interrupter — {1}{W}{W} 2/2 Bird Rogue with flash and flying. ETB:
/// exile target spell; it becomes plotted. (The opponents'-spells-from-
/// graveyards-and-exile tax is dropped — no cast-origin spell filter yet.)
pub fn aven_interrupter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CounterSpellToZone {
                what: target_filtered(R::IsSpellOnStack),
                zone: CounteredSpellZone::ExilePlotted,
            },
        }],
        ..legend(
            "Aven Interrupter",
            cost(&[generic(1), w(), w()]),
            2,
            2,
            vec![CreatureType::Bird, CreatureType::Rogue],
            vec![Keyword::Flash, Keyword::Flying],
        )
    }
}

/// Step Between Worlds — {3}{U}{U} Sorcery with plot {4}{U}{U}. Each player
/// may shuffle their hand and graveyard into their library and draw seven.
/// Exile Step Between Worlds.
pub fn step_between_worlds() -> CardDefinition {
    CardDefinition {
        name: "Step Between Worlds",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        plot_cost: Some(cost(&[generic(4), u(), u()])),
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::ShuffleHandAndGraveyardIntoLibrary {
                who: PlayerRef::EachPlayer,
            },
            Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(7),
            },
        ]),
        ..Default::default()
    }
}

// ── The rest of the "Joins Up" cycle ────────────────────────────────────────

/// Annie Joins Up — {1}{R}{G}{W} legendary enchantment. ETB: 5 damage to
/// target creature or planeswalker an opponent controls. Triggered abilities
/// of your legendary creatures trigger an additional time.
pub fn annie_joins_up() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a triggered ability of a legendary creature you control triggers, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerLegendaryCreatureTriggers,
        }],
        ..joins_up(
            "Annie Joins Up",
            cost(&[generic(1), r(), g(), w()]),
            Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent)),
                amount: Value::Const(5),
            },
            None,
        )
    }
}

/// Rakdos Joins Up — {3}{B}{R} legendary enchantment. ETB: reanimate a
/// creature card with two extra +1/+1 counters. Whenever a legendary creature
/// you control dies, deal damage equal to its power to target opponent.
pub fn rakdos_joins_up() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(
                            R::Creature.and(R::PermanentCard).and(R::InYourGraveyard),
                        ),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                    Effect::AddCounter {
                        what: Selector::LastMoved,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasSupertype(Supertype::Legendary),
                    }),
                effect: Effect::DealDamage {
                    to: target_filtered(R::OpponentPlayer),
                    amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                },
            },
        ],
        ..CardDefinition {
            name: "Rakdos Joins Up",
            cost: cost(&[generic(3), b(), r()]),
            supertypes: vec![Supertype::Legendary],
            card_types: vec![CardType::Enchantment],
            ..Default::default()
        }
    }
}

/// Tinybones Joins Up — {B} legendary enchantment. ETB: target player discards
/// a card. Whenever a legendary creature you control enters, target player
/// mills a card and loses 1 life. ("Any number of target players" is modeled
/// as one target.)
pub fn tinybones_joins_up() -> CardDefinition {
    joins_up(
        "Tinybones Joins Up",
        cost(&[b()]),
        Effect::Discard {
            who: target_filtered(R::Player),
            amount: Value::ONE,
            random: false,
        },
        Some(legend_enters(Effect::Seq(vec![
            Effect::Mill {
                who: target_filtered(R::Player),
                amount: Value::ONE,
            },
            Effect::LoseLife {
                who: Selector::Target(0),
                amount: Value::ONE,
            },
        ]))),
    )
}

/// Vraska Joins Up — {B}{G} legendary enchantment. ETB: a deathtouch counter
/// on each creature you control. Whenever a legendary creature you control
/// deals combat damage to a player, draw a card.
pub fn vraska_joins_up() -> CardDefinition {
    joins_up(
        "Vraska Joins Up",
        cost(&[b(), g()]),
        Effect::AddKeywordCounter {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keyword: Keyword::Deathtouch,
            amount: Value::ONE,
        },
        Some(TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasSupertype(Supertype::Legendary),
            }),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }),
    )
}

// ── Mounts and saddle payoffs (CR 702.171) ──────────────────────────────────

/// Archmage's Newt — {1}{U} 2/2 Salamander Mount with saddle 3. Combat damage
/// to a player grants a graveyard instant/sorcery flashback for the turn.
/// (The "flashback {0} while saddled" discount is dropped — the granted
/// flashback always costs the card's mana cost.)
pub fn archmages_newt() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Salamander, CreatureType::Mount],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::GrantFlashbackThisTurn {
                what: target_filtered(
                    R::InYourGraveyard.and(
                        R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    ),
                ),
            },
        }],
        ..legend(
            "Archmage's Newt",
            cost(&[generic(1), u()]),
            2,
            2,
            vec![],
            vec![Keyword::Saddle(3)],
        )
    }
}

// ── Legends on existing primitives ──────────────────────────────────────────

/// Akul the Unrepentant — {B}{B}{R}{R} 5/5 Scorpion Dragon Rogue with flying
/// and trample. Sacrifice three other creatures: put a creature card from your
/// hand onto the battlefield. Sorcery speed, once each turn.
pub fn akul_the_unrepentant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 3)),
            sorcery_speed: true,
            once_per_turn: true,
            effect: Effect::MayDo {
                description: "put a creature card from your hand onto the battlefield".into(),
                body: Box::new(Effect::Move {
                    what: Selector::take(
                        Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: crate::card::Zone::Hand,
                            filter: R::Creature.and(R::PermanentCard),
                        },
                        Value::ONE,
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
            },
            ..Default::default()
        }],
        ..legend(
            "Akul the Unrepentant",
            cost(&[b(), b(), r(), r()]),
            5,
            5,
            vec![
                CreatureType::Scorpion,
                CreatureType::Dragon,
                CreatureType::Rogue,
            ],
            vec![Keyword::Flying, Keyword::Trample],
        )
    }
}

/// Obeka, Splitter of Seconds — {1}{U}{B}{R} 2/5 Ogre Warlock with menace.
/// Combat damage to a player gives that many additional upkeep steps.
pub fn obeka_splitter_of_seconds() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AdditionalUpkeepStep {
                count: Value::TriggerEventAmount,
            },
        }],
        ..legend(
            "Obeka, Splitter of Seconds",
            cost(&[generic(1), u(), b(), r()]),
            2,
            5,
            vec![CreatureType::Ogre, CreatureType::Warlock],
            vec![Keyword::Menace],
        )
    }
}

/// Geralf, the Fleshwright — {2}{U} 2/3 Human Warlock. Your second and later
/// spell each of your turns makes a 2/2 Zombie Rogue; each Zombie you control
/// enters with a +1/+1 counter per other Zombie that entered this turn.
pub fn geralf_the_fleshwright() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        Predicate::IsTurnOf(PlayerRef::You),
                        Predicate::SpellsCastThisTurnAtLeast {
                            who: PlayerRef::You,
                            at_least: Value::Const(2),
                        },
                    ]),
                ),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(TokenDefinition {
                        name: "Zombie Rogue".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Blue, Color::Black],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Zombie, CreatureType::Rogue],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Zombie),
                    }),
                effect: Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::OtherCreaturesOfTypeEnteredThisTurn(CreatureType::Zombie),
                },
            },
        ],
        ..legend(
            "Geralf, the Fleshwright",
            cost(&[generic(2), u()]),
            2,
            3,
            vec![CreatureType::Human, CreatureType::Warlock],
            vec![],
        )
    }
}

/// Selvala, Eager Trailblazer — {2}{G}{W} 4/5 Elf Scout with vigilance. Each
/// creature spell you cast makes a Mercenary; {T} adds one mana of a chosen
/// color per different power among your creatures.
pub fn selvala_eager_trailblazer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PermanentCard),
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(mercenary_token()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::DistinctPowersAmongCreaturesControlled(
                    PlayerRef::You,
                )),
            },
            ..Default::default()
        }],
        ..legend(
            "Selvala, Eager Trailblazer",
            cost(&[generic(2), g(), w()]),
            4,
            5,
            vec![CreatureType::Elf, CreatureType::Scout],
            vec![Keyword::Vigilance],
        )
    }
}

/// Ertha Jo, Frontier Mentor — {2}{R}{W} 2/4 Kor Advisor. ETB: create a
/// Mercenary. (The activated-ability copier is dropped — the engine has no
/// ability-copy primitive.)
pub fn ertha_jo_frontier_mentor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(mercenary_token()),
            },
        }],
        ..legend(
            "Ertha Jo, Frontier Mentor",
            cost(&[generic(2), r(), w()]),
            2,
            4,
            vec![CreatureType::Kor, CreatureType::Advisor],
            vec![],
        )
    }
}

/// Bonny Pall, Clearcutter — {3}{G}{U}{U} 6/5 Giant Scout with reach. ETB:
/// create Beau, a legendary Ox whose P/T is your land count. Whenever you
/// attack, draw a card, then you may deploy a land from your hand.
pub fn bonny_pall_clearcutter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(TokenDefinition {
                        name: "Beau".into(),
                        power: 0,
                        toughness: 0,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Blue],
                        supertypes: vec![Supertype::Legendary],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Ox],
                            ..Default::default()
                        },
                        dynamic_pt: Some((
                            Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou))),
                            Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou))),
                        )),
                        ..Default::default()
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    Effect::MayDo {
                        description: "put a land card from your hand onto the battlefield".into(),
                        body: Box::new(Effect::Move {
                            what: Selector::take(
                                Selector::CardsInZone {
                                    who: PlayerRef::You,
                                    zone: crate::card::Zone::Hand,
                                    filter: R::Land,
                                },
                                Value::ONE,
                            ),
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::You,
                                tapped: false,
                            },
                        }),
                    },
                ]),
            },
        ],
        ..legend(
            "Bonny Pall, Clearcutter",
            cost(&[generic(3), g(), u(), u()]),
            6,
            5,
            vec![CreatureType::Giant, CreatureType::Scout],
            vec![Keyword::Reach],
        )
    }
}

/// Satoru, the Infiltrator — {U}{B} 2/3 Human Ninja Rogue with menace.
/// Whenever a nontoken creature you control enters without being cast, draw a
/// card. (The "one or more at once" batching resolves per creature.)
pub fn satoru_the_infiltrator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::NotToken),
                    },
                    Predicate::Not(Box::new(Predicate::TriggerSourceEnteredByCast)),
                ])),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..legend(
            "Satoru, the Infiltrator",
            cost(&[u(), b()]),
            2,
            3,
            vec![
                CreatureType::Human,
                CreatureType::Ninja,
                CreatureType::Rogue,
            ],
            vec![Keyword::Menace],
        )
    }
}

// ── Lands, Auras, Equipment ─────────────────────────────────────────────────

/// Arid Archway — Desert land that enters tapped and bounces a land you
/// control; if that land was another Desert, surveil 1. {T}: add {C}{C}.
pub fn arid_archway() -> CardDefinition {
    CardDefinition {
        name: "Arid Archway",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![crate::card::LandType::Desert],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::MoveChosen {
                    from: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                    filter: None,
                    count: Value::ONE,
                    up_to: false,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::LastMoved,
                        filter: R::HasLandType(crate::card::LandType::Desert),
                    },
                    then: Box::new(Effect::Surveil {
                        who: PlayerRef::You,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(2)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stop Cold — {3}{U} Aura with flash. ETB taps the enchanted artifact or
/// creature; it loses all abilities and doesn't untap.
pub fn stop_cold() -> CardDefinition {
    CardDefinition {
        name: "Stop Cold",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.or(R::Artifact)),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        equipped_bonus: Some(crate::card::EquipBonus {
            remove_abilities: true,
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted permanent doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedToMe(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}

// ── Spree spells (CR 702.172) ───────────────────────────────────────────────

/// One Last Job — {2}{W} Sorcery with Spree: reanimate a creature, a
/// Mount/Vehicle, and/or an Aura/Equipment from your graveyard.
pub fn one_last_job() -> CardDefinition {
    CardDefinition {
        name: "One Last Job",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Spree {
            modes: vec![
                mode(
                    cost(&[generic(2)]),
                    Effect::Move {
                        what: target_filtered(
                            R::Creature.and(R::PermanentCard).and(R::InYourGraveyard),
                        ),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                ),
                mode(
                    cost(&[generic(1)]),
                    Effect::Move {
                        what: target_filtered(
                            R::InYourGraveyard.and(
                                R::HasCreatureType(CreatureType::Mount).or(R::HasArtifactSubtype(
                                    crate::card::ArtifactSubtype::Vehicle,
                                )),
                            ),
                        ),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                ),
                mode(
                    cost(&[generic(1)]),
                    Effect::Move {
                        what: target_filtered(R::InYourGraveyard.and(
                            R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Aura).or(
                                R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment),
                            ),
                        )),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                ),
            ],
        },
        ..Default::default()
    }
}

/// Shifting Grift — {U}{U} Sorcery with Spree: exchange control of two
/// creatures, two artifacts, and/or two enchantments.
pub fn shifting_grift() -> CardDefinition {
    let swap = |filter: R| Effect::ExchangeControlChoosing {
        filter,
        with: Selector::Target(0),
    };
    CardDefinition {
        name: "Shifting Grift",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Spree {
            modes: vec![
                mode(cost(&[generic(2)]), swap(R::Creature)),
                mode(cost(&[generic(1)]), swap(R::Artifact)),
                mode(cost(&[generic(1)]), swap(R::Enchantment)),
            ],
        },
        ..Default::default()
    }
}

/// Great Train Heist — {R} Instant with Spree: untap your team plus an extra
/// combat, a team pump with first strike, and/or Treasures on combat damage to
/// a chosen opponent.
pub fn great_train_heist() -> CardDefinition {
    CardDefinition {
        name: "Great Train Heist",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Spree {
            modes: vec![
                mode(
                    cost(&[generic(2), r()]),
                    Effect::Seq(vec![
                        Effect::Untap {
                            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                            up_to: None,
                        },
                        Effect::AdditionalCombatPhase { count: Value::ONE },
                    ]),
                ),
                mode(
                    cost(&[generic(2)]),
                    Effect::Seq(vec![
                        Effect::PumpPT {
                            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                            power: Value::ONE,
                            toughness: Value::Const(0),
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                            keyword: Keyword::FirstStrike,
                            duration: Duration::EndOfTurn,
                        },
                    ]),
                ),
                mode(
                    cost(&[r()]),
                    Effect::CreaturesYouControlDealingCombatDamageThisTurn {
                        body: Box::new(Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: Box::new(crabomination_base::tokens::treasure_token()),
                        }),
                    },
                ),
            ],
        },
        ..Default::default()
    }
}

/// Laughing Jasper Flint — {1}{B}{R} 4/3 Lizard Rogue. Creatures you control
/// but don't own are Mercenaries; each upkeep exile the top X of an opponent's
/// library (X = outlaws you control) and you may play them this turn.
pub fn laughing_jasper_flint() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control but don't own are Mercenaries in addition to their other types.",
            effect: StaticEffect::AddCreatureTypeToMatching {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::Not(Box::new(R::OwnedByYou))),
                ),
                creature_type: CreatureType::Mercenary,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::EachOpponent,
                count: Value::count(Selector::EachPermanent(R::IsOutlaw.and(R::ControlledByYou))),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: true,
                max_mana_value: None,
                pay_own_cost: false,
                uncast_penalty: None,
            },
        }],
        ..legend(
            "Laughing Jasper Flint",
            cost(&[generic(1), b(), r()]),
            4,
            3,
            vec![CreatureType::Lizard, CreatureType::Rogue],
            vec![],
        )
    }
}

/// Ghired, Mirror of the Wilds — {R}{G}{W} 3/3 Human Shaman with haste. Your
/// nontoken creatures can tap to copy a token you control that entered this
/// turn.
pub fn ghired_mirror_of_the_wilds() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Nontoken creatures you control have \"{T}: Create a token that's a copy of target token you control that entered this turn.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::NotToken),
                ),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::CreateTokenCopyOf {
                        source: target_filtered(
                            R::IsToken.and(R::ControlledByYou).and(R::EnteredThisTurn),
                        ),
                        count: Value::ONE,
                        who: PlayerRef::You,
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        extra_keywords: vec![],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: false,
                        legendary: false,
                        non_legendary: false,
                    },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..legend(
            "Ghired, Mirror of the Wilds",
            cost(&[r(), g(), w()]),
            3,
            3,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![Keyword::Haste],
        )
    }
}

/// Kambal, Profiteering Mayor — {1}{W}{B} 2/4 Human Advisor. Opponents' tokens
/// entering are copied (once each turn); your tokens entering drain each
/// opponent for 1.
pub fn kambal_profiteering_mayor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::TokenCreated, EventScope::OpponentControl)
                    .once_per_turn(),
                effect: Effect::CreateTokenCopyOf {
                    source: Selector::TriggerSource,
                    count: Value::ONE,
                    who: PlayerRef::You,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    extra_keywords: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: true,
                    legendary: false,
                    non_legendary: false,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::TokenCreated, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::ONE,
                    },
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                ]),
            },
        ],
        ..legend(
            "Kambal, Profiteering Mayor",
            cost(&[generic(1), w(), b()]),
            2,
            4,
            vec![CreatureType::Human, CreatureType::Advisor],
            vec![],
        )
    }
}

/// Annie Flash, the Veteran — {3}{R}{G}{W} 4/5 Human Rogue with flash. Cast-ETB
/// reanimates a mana-value-3-or-less permanent tapped; becoming tapped exiles
/// two cards you may play this turn.
pub fn annie_flash_the_veteran() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(Predicate::TriggerSourceEnteredByCast),
                effect: Effect::Move {
                    what: target_filtered(
                        R::PermanentCard
                            .and(R::InYourGraveyard)
                            .and(R::ManaValueAtMost(3)),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            },
        ],
        ..legend(
            "Annie Flash, the Veteran",
            cost(&[generic(3), r(), g(), w()]),
            4,
            5,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![Keyword::Flash],
        )
    }
}

/// Fblthp, Lost on the Range — {1}{U}{U} 1/1 Homunculus with ward {2}. You
/// play with the top card of your library revealed and may plot nonland cards
/// from the top of your library.
pub fn fblthp_lost_on_the_range() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may plot nonland cards from the top of your library.",
                effect: StaticEffect::MayPlotFromLibraryTop,
            },
        ],
        ..legend(
            "Fblthp, Lost on the Range",
            cost(&[generic(1), u(), u()]),
            1,
            1,
            vec![CreatureType::Homunculus],
            vec![Keyword::Ward(crate::card::WardCost::Mana(cost(&[
                generic(2),
            ])))],
        )
    }
}

/// Rakdos, the Muscle — {2}{B}{B}{R} 6/5 Demon Mercenary with flying and
/// trample. Sacrificing another creature impulses that many cards off a target
/// player's library; a sac ability makes Rakdos indestructible.
pub fn rakdos_the_muscle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::EachOpponent,
                count: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: true,
                max_mana_value: None,
                pay_own_cost: false,
                uncast_penalty: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            once_per_turn: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::Tap {
                    what: Selector::This,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Rakdos, the Muscle",
            cost(&[generic(2), b(), b(), r()]),
            6,
            5,
            vec![CreatureType::Demon, CreatureType::Mercenary],
            vec![Keyword::Flying, Keyword::Trample],
        )
    }
}

/// Lazav, Familiar Stranger — {1}{U}{B} 1/4 Shapeshifter. Your first crime each
/// turn grows Lazav and may exile a graveyard card; a creature card exiled this
/// way can be copied until end of turn.
pub fn lazav_familiar_stranger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::MayDo {
                    description: "exile a creature card from a graveyard and copy it".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: target_filtered(
                                R::Creature.and(R::PermanentCard).and(R::InGraveyard),
                            ),
                            to: ZoneDest::Exile,
                        },
                        Effect::BecomeCopyOf {
                            what: Selector::This,
                            source: Selector::LastMoved,
                            extra_creature_types: vec![],
                            keep_own_triggered: false,
                            keep_own_activated: false,
                        },
                    ])),
                },
            ]),
        }],
        ..legend(
            "Lazav, Familiar Stranger",
            cost(&[generic(1), u(), b()]),
            1,
            4,
            vec![CreatureType::Shapeshifter],
            vec![],
        )
    }
}

// ── Batch 2: planeswalkers, graveyard theft, the Desert ─────────────────────

/// Jace Reawakened — {U}{U} legendary planeswalker, loyalty 3. +1 loots; +1
/// plots a cheap nonland card from hand; −6 copies your spells for the turn.
/// (The "can't cast during your first three turns" restriction is dropped —
/// no turn-number cast gate.)
pub fn jace_reawakened() -> CardDefinition {
    CardDefinition {
        name: "Jace Reawakened",
        cost: cost(&[u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Jace],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            crate::card::LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                ]),
                ..Default::default()
            },
            crate::card::LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::MoveChosen {
                    from: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Hand,
                        filter: R::Not(Box::new(R::Land)).and(R::ManaValueAtMost(3)),
                    },
                    filter: None,
                    count: Value::ONE,
                    up_to: true,
                    to: ZoneDest::ExilePlotted,
                },
                ..Default::default()
            },
            crate::card::LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::OnEachSpellCastThisTurn {
                    body: Box::new(Effect::CopySpellMayChooseTargets {
                        what: Selector::TriggerSource,
                        count: Value::ONE,
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Oko, the Ringleader — {2}{G}{U} legendary planeswalker, loyalty 3. He copies
/// one of your creatures each combat; +1 draws two and discards (one after a
/// crime); −1 makes a 3/3 Elk; −5 copies your other nonland permanents.
pub fn oko_the_ringleader() -> CardDefinition {
    CardDefinition {
        name: "Oko, the Ringleader",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Oko],
            ..Default::default()
        },
        base_loyalty: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            // "Except he has hexproof" rides as a separate EOT grant.
            effect: Effect::Seq(vec![
                Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: target_filtered(R::Creature.and(R::ControlledByYou)),
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Hexproof,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        loyalty_abilities: vec![
            crate::card::LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(2),
                    },
                    Effect::If {
                        cond: Predicate::CommittedCrimeThisTurn {
                            who: PlayerRef::You,
                        },
                        then: Box::new(Effect::Discard {
                            who: Selector::You,
                            amount: Value::ONE,
                            random: false,
                        }),
                        else_: Box::new(Effect::Discard {
                            who: Selector::You,
                            amount: Value::Const(2),
                            random: false,
                        }),
                    },
                ]),
                ..Default::default()
            },
            crate::card::LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(TokenDefinition {
                        name: "Elk".into(),
                        power: 3,
                        toughness: 3,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Green],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Elk],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
                ..Default::default()
            },
            crate::card::LoyaltyAbility {
                loyalty_cost: -5,
                effect: Effect::CreateTokenCopyOf {
                    source: Selector::EachPermanent(
                        R::ControlledByYou
                            .and(R::Not(Box::new(R::Land)))
                            .and(R::OtherThanSource),
                    ),
                    count: Value::ONE,
                    who: PlayerRef::You,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    extra_keywords: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    legendary: false,
                    non_legendary: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Kaervek, the Punisher — {1}{B}{B} 3/3 Human Warlock. Each crime you commit
/// recasts a black card from your graveyard as a copy for 2 life. (The
/// original stays in the graveyard — the copy path doesn't exile it.)
pub fn kaervek_the_punisher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::CastWithoutPayingImmediate {
                    reduce_generic: 0,
                                pay_own_cost: false,
                    what: target_filtered(R::InYourGraveyard.and(R::HasColor(Color::Black))),
                    source_zone: crate::card::Zone::Graveyard,
                    exile_after: true,
                    copy: true,
                },
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..legend(
            "Kaervek, the Punisher",
            cost(&[generic(1), b(), b()]),
            3,
            3,
            vec![CreatureType::Human, CreatureType::Warlock],
            vec![],
        )
    }
}

/// Tinybones, the Pickpocket — {B} 1/1 Skeleton Rogue with deathtouch. Combat
/// damage to a player lets you cast a nonland permanent card from their
/// graveyard. (Modeled as a free cast rather than "pay its cost with any
/// mana".)
pub fn tinybones_the_pickpocket() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CastWithoutPayingImmediate {
                reduce_generic: 0,
                                pay_own_cost: false,
                what: target_filtered(
                    R::PermanentCard
                        .and(R::Not(Box::new(R::Land)))
                        .and(R::InGraveyard),
                ),
                source_zone: crate::card::Zone::Graveyard,
                exile_after: false,
                copy: false,
            },
        }],
        ..legend(
            "Tinybones, the Pickpocket",
            cost(&[b()]),
            1,
            1,
            vec![CreatureType::Skeleton, CreatureType::Rogue],
            vec![Keyword::Deathtouch],
        )
    }
}

/// The Key to the Vault — {1}{U} legendary Equipment, equip {2}{U}. Combat
/// damage from the equipped creature digs that many deep, exiles a card and
/// lets you cast it for free.
pub fn the_key_to_the_vault() -> CardDefinition {
    CardDefinition {
        name: "The Key to the Vault",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2), u()]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::LookTopExileOneMayPlay {
                    count: Value::TriggerEventAmount,
                    who: PlayerRef::You,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Bucolic Ranch — Desert land. {T}: add {C}. {T}: add one mana of any color,
/// spendable only on a Mount spell. {3},{T}: dig one deep for a Mount.
pub fn bucolic_ranch() -> CardDefinition {
    CardDefinition {
        name: "Bucolic Ranch",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![crate::card::LandType::Desert],
            ..Default::default()
        },
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
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        crate::mana::SpendRestriction::CreatureOfType(CreatureType::Mount),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    pick_filter: Some(R::HasCreatureType(CreatureType::Mount)),
                    optional: true,
    ..Default::default()
})),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
