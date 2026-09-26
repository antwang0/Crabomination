//! Commander: the cards the **Multiverse Reforged** precon (FRC, Jace,
//! Multiverse Architect) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_jace.rs`.
//!
//! Residuals (each also on its card):
//! - **Dack Fayden, Helping Hand** — the revealed creatures go to the
//!   opponents in turn order, not by your choice.
//! - **Tamiyo, Upriser Crowned** — "one or more creatures" fires once per
//!   creature (the same taps and stun counters).

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EventKind,
    EventScope, EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, landfall, on_attack, target_filtered};
use crate::effect::{
    CounteredSpellZone, DelayedTriggerKind, Duration, Effect, LibraryPosition, ManaPayload, PlayerRef,
    PlayerStaticTarget, Predicate, RevealMissDest, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, w};
use crate::sets::tap_add;
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn empower(n: i32) -> Effect {
    Effect::EmpowerJace { who: PlayerRef::You, count: Value::Const(n) }
}

/// "This land enters tapped unless your opponents control eight or more
/// lands" (the Turbulent cycle, as Turbulent Wetlands).
fn turbulent(name: &'static str, types: [LandType; 2], colors: [Color; 2]) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: types.to_vec(), ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless your opponents control eight or more lands.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Land.and(R::ControlledByOpponent)),
                    n: Value::Const(8),
                },
            },
        }],
        activated_abilities: vec![tap_add(colors[0]), tap_add(colors[1])],
        ..Default::default()
    }
}

/// Archfiend of Despair — flying; opponents can't gain life; each end step,
/// each opponent loses life equal to the life they lost this turn.
pub fn archfiend_of_despair() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife { target: PlayerStaticTarget::EachOpponent },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::ForEachOpponent {
                body: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::LifeLostThisTurn(PlayerRef::Triggerer),
                }),
            },
        }],
        ..creature("Archfiend of Despair", cost(&[generic(6), b(), b()]), vec![CreatureType::Demon], 6, 6)
    }
}

/// Avacyn, Angel of Horror — flying, deathtouch; whenever it or another
/// nontoken creature you control dies, return that card at the beginning of
/// the next end step.
pub fn avacyn_angel_of_horror() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
            ),
            effect: Effect::DelayUntilWithCapture {
                kind: DelayedTriggerKind::NextEndStep,
                capture: Selector::TriggerSource,
                body: Box::new(Effect::Move {
                    what: Selector::TargetFiltered { slot: 0, filter: R::InGraveyard },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..legend("Avacyn, Angel of Horror", cost(&[generic(5), b(), b(), b()]), vec![CreatureType::Angel], 8, 8)
    }
}

/// Dack Fayden, Helping Hand — ETB: reveal until X creature cards (X your
/// opponents), put them onto the battlefield goaded for the rest of the
/// game, and give one to each opponent. Residual: the opponents get them in
/// turn order, not by your choice.
pub fn dack_fayden_helping_hand() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::RevealUntilMatchingToBattlefield {
                filter: R::Creature,
                count: Value::OpponentCount,
                rest_bottom: false,
            },
            Effect::GoadForTheGame { what: Selector::LastMoved },
            Effect::DistributeControlAmongOpponents { what: Selector::LastMoved },
        ]))],
        ..legend(
            "Dack Fayden, Helping Hand",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            4,
            6,
        )
    }
}

/// Darksteel Angel — flying, indestructible; you can't lose and opponents
/// can't win; your creatures can't have -1/-1 counters put on them.
pub fn darksteel_angel() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Indestructible],
        static_abilities: vec![
            StaticAbility {
                description: "You can't lose the game and your opponents can't win the game.",
                effect: StaticEffect::ControllerCantLoseGame,
            },
            StaticAbility {
                description: "Creatures you control can't have -1/-1 counters put on them.",
                effect: StaticEffect::NoMinusCountersOnYourCreatures,
            },
        ],
        ..creature("Darksteel Angel", cost(&[generic(9)]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Fatehold Charm — choose one: draw a card and empower Jace 2; return
/// target spell or creature to its owner's hand; creatures you control get
/// +1/+2 until end of turn.
pub fn fatehold_charm() -> CardDefinition {
    CardDefinition {
        name: "Fatehold Charm",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![draw(1), empower(2)]),
            Effect::MoveSpellToZone {
                what: target_filtered(R::IsSpellOnStack),
                zone: CounteredSpellZone::OwnerHand,
            },
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// A Gingerbrute token: 1/1 Food Golem artifact creature with haste, "{1}:
/// can't be blocked this turn except by creatures with haste" and "{2},
/// {T}, sacrifice it: gain 3 life".
fn gingerbrute_token() -> TokenDefinition {
    TokenDefinition {
        name: "Gingerbrute".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            artifact_subtypes: vec![ArtifactSubtype::Food],
            ..Default::default()
        },
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::CantBeBlockedExceptBy(Box::new(R::HasKeyword(Keyword::Haste))),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ginger, Queen of Sweets — ETB you become the monarch; {2},{T}, sacrifice
/// it: gain 6; each upkeep, if you're the monarch, a Gingerbrute.
pub fn ginger_queen_of_sweets() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Noble],
            artifact_subtypes: vec![ArtifactSubtype::Food],
            ..Default::default()
        },
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                    .with_filter(Predicate::IsMonarch { who: PlayerRef::You }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(gingerbrute_token()),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(6) },
            ..Default::default()
        }],
        ..legend("Ginger, Queen of Sweets", cost(&[generic(6)]), vec![], 6, 4)
    }
}

/// Jace, Multiverse Architect — each opponent's beginning of combat, they
/// may pay {2}; if they don't, their creatures can't attack your Jaces this
/// turn. +1: draw two, put a card from hand on the bottom. −3: exile another
/// target planeswalker or creature you control, then reveal until a creature
/// or planeswalker card and put it onto the battlefield. Can be your
/// commander.
pub fn jace_multiverse_architect() -> CardDefinition {
    CardDefinition {
        name: "Jace, Multiverse Architect",
        cost: cost(&[generic(1), w(), u(), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Jace], ..Default::default() },
        base_loyalty: 4,
        can_be_commander: true,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::OpponentControl),
            // The pay runs as the active player, so the protected Jaces are
            // named by the source's controller, not `You`.
            effect: Effect::MayPayBy {
                who: PlayerRef::ActivePlayer,
                description: "Pay {2}, or your creatures can't attack Jaces this turn".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::CantAttackPlaneswalkerTypeThisTurn {
                    who: PlayerRef::ActivePlayer,
                    defender: PlayerRef::ControllerOf(Box::new(Selector::This)),
                    subtype: PlaneswalkerSubtype::Jace,
                })),
            },
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    draw(2),
                    Effect::PutCardsFromHandOnBottom { who: Selector::You, count: Value::ONE },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::Exile {
                        what: target_filtered(
                            R::Planeswalker.or(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                        ),
                    },
                    Effect::RevealUntilOneToBattlefieldRestBottom {
                        filter: R::Creature.or(R::Planeswalker),
                        damage_controller: false,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Jhoira, Weatherlight Corsair — enters or attacks: target opponent reveals
/// until a historic permanent card; you put it onto the battlefield and lose
/// life equal to its mana value; the rest go to the bottom.
pub fn jhoira_weatherlight_corsair() -> CardDefinition {
    let historic = R::Artifact
        .or(R::HasSupertype(Supertype::Legendary))
        .or(R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Saga));
    let raid = || {
        Effect::Seq(vec![
            Effect::RevealUntilFind {
                who: PlayerRef::Target(0),
                find: historic.clone().and(R::PermanentCard),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                cap: Value::Const(999),
                life_per_revealed: 0,
                miss_dest: RevealMissDest::BottomRandom,
            },
            Effect::LoseLife { who: Selector::You, amount: Value::ManaValueOf(Box::new(Selector::LastMoved)) },
        ])
    };
    let mut on_etb = etb(raid());
    let mut on_atk = on_attack(raid());
    // "target opponent": declare the slot on the life clause's sibling.
    for t in [&mut on_etb, &mut on_atk] {
        t.effect = Effect::Seq(vec![
            Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::Const(0) },
            t.effect.clone(),
        ]);
    }
    CardDefinition {
        triggered_abilities: vec![on_etb, on_atk],
        ..legend(
            "Jhoira, Weatherlight Corsair",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            4,
            5,
        )
    }
}

/// Memnarch, the Warden — indestructible; ETB two 1/1 Myr; attacks: draw a
/// card per artifact you control.
pub fn memnarch_the_warden() -> CardDefinition {
    let myr = Arc::new(TokenDefinition {
        name: "Myr".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Myr], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: myr }),
            on_attack(Effect::Draw {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))),
            }),
        ],
        ..legend("Memnarch, the Warden", cost(&[generic(10)]), vec![CreatureType::Wizard], 8, 9)
    }
}

/// Nissa, Leyline Tamer — deathtouch, vigilance; landfall: draw a card, then
/// the first time each turn, reveal until a creature card and put it onto
/// the battlefield.
pub fn nissa_leyline_tamer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Vigilance],
        triggered_abilities: vec![landfall(Effect::Seq(vec![
            draw(1),
            Effect::NthResolutionThisTurn {
                branches: vec![Effect::RevealUntilOneToBattlefieldRestBottom {
                    filter: R::Creature,
                    damage_controller: false,
                }],
            },
        ]))],
        ..legend(
            "Nissa, Leyline Tamer",
            cost(&[generic(3), w(), u(), b(), r()]),
            vec![CreatureType::Elf, CreatureType::Wizard],
            5,
            5,
        )
    }
}

/// Niv-Mizzet, Ghost Counsel — flying; whenever you gain life, you may pay
/// that much life to draw that many; {T}: drain 1.
pub fn niv_mizzet_ghost_counsel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::MayPayLife {
                description: "Pay that much life to draw that many cards".into(),
                amount: Value::TriggerEventAmount,
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount }),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend(
            "Niv-Mizzet, Ghost Counsel",
            cost(&[generic(2), w(), w(), b(), b()]),
            vec![CreatureType::Spirit, CreatureType::Dragon],
            4,
            4,
        )
    }
}

/// Ob Nixilis, the Ascended — flying; ETB destroy all tapped creatures your
/// opponents control and gain 1 per creature destroyed; each end step, if
/// you gained life this turn, a 4/4 flying Angel.
pub fn ob_nixilis_the_ascended() -> CardDefinition {
    let angel = Arc::new(TokenDefinition {
        name: "Angel".into(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Destroy {
                    what: Selector::EachPermanent(R::Creature.and(R::Tapped).and(R::ControlledByOpponent)),
                },
                Effect::GainLife { who: Selector::You, amount: Value::PermanentsDestroyedThisResolution },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                    Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
                ),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: angel },
            },
        ],
        ..legend("Ob Nixilis, the Ascended", cost(&[generic(5), w(), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Omnath, Locus of the Void — +1/+1 per unspent mana you have; unspent mana
/// becomes colorless instead of emptying; landfall: add {C}{C}.
pub fn omnath_locus_of_the_void() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::BasePlusUnspentMana { base_p: 6, base_t: 6 }),
        static_abilities: vec![StaticAbility {
            description: "If you would lose unspent mana, that mana becomes colorless instead.",
            effect: StaticEffect::UnspentManaBecomesColorless,
        }],
        triggered_abilities: vec![landfall(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colorless(Value::Const(2)),
        })],
        ..legend("Omnath, Locus of the Void", cost(&[generic(7)]), vec![CreatureType::Elemental], 6, 6)
    }
}

/// Plan for All Outcomes — ETB: the owner of up to one other target nonland
/// permanent puts it on their choice of top or bottom of their library;
/// your first noncreature spell each turn empowers Jace 1.
pub fn plan_for_all_outcomes() -> CardDefinition {
    CardDefinition {
        name: "Plan for All Outcomes",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Permanent.and(R::Nonland).and(R::OtherThanSource),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        pos: LibraryPosition::OwnerChoice,
                    },
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Noncreature },
                        Predicate::ValueEquals(Value::NoncreatureSpellsCastThisTurn(PlayerRef::You), Value::ONE),
                    ]),
                ),
                effect: empower(1),
            },
        ],
        ..Default::default()
    }
}

/// Tamiyo, Upriser Crowned — flying, double strike, haste; ETB you become the
/// monarch; a creature dealing combat damage to you while you're the monarch
/// is tapped and stunned. Residual: fires once per creature rather than once
/// per batch (same outcome).
pub fn tamiyo_upriser_crowned() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::DoubleStrike, Keyword::Haste],
        triggered_abilities: vec![
            etb(Effect::BecomeMonarch { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer).with_filter(
                    Predicate::All(vec![
                        Predicate::SamePlayer(PlayerRef::TriggerEventPlayer, PlayerRef::You),
                        Predicate::IsMonarch { who: PlayerRef::You },
                    ]),
                ),
                effect: Effect::Seq(vec![
                    Effect::Tap { what: Selector::TriggerSource },
                    Effect::AddCounter { what: Selector::TriggerSource, kind: CounterType::Stun, amount: Value::ONE },
                ]),
            },
        ],
        ..legend(
            "Tamiyo, Upriser Crowned",
            cost(&[generic(4), r(), w()]),
            vec![CreatureType::Moonfolk, CreatureType::Warrior],
            3,
            5,
        )
    }
}

/// Teferi's Reproach — target opponent gains protection from everything and
/// their life total can't change until their next turn; their nonland
/// permanents phase out. Exile it.
pub fn teferis_reproach() -> CardDefinition {
    CardDefinition {
        name: "Teferi's Reproach",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::LifeLockUntilNextTurn { who: target_filtered(R::OpponentPlayer) },
            Effect::PlayerProtectionUntilNextTurn { who: PlayerRef::Target(0) },
            Effect::PhaseOut {
                what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Nonland },
                until_source_leaves: false,
            },
        ]),
        ..Default::default()
    }
}

/// The Ur-Sphinx — eminence: other Sphinx spells cost {1} less; flying;
/// whenever one or more Sphinxes you control attack, each player mills that
/// many, and you may cast one card each player milled for free.
pub fn the_ur_sphinx() -> CardDefinition {
    let sphinx = || R::HasCreatureType(CreatureType::Sphinx);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        statics_in_command_zone: true,
        static_abilities: vec![StaticAbility {
            description: "Other Sphinx spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: sphinx().and(R::HasName("The Ur-Sphinx".into()).negate()),
                amount: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: sphinx() })
                .once_per_batch(),
            effect: Effect::EachPlayerMillsYouMayCastOne {
                count: Value::count(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(sphinx()).and(R::IsAttacking),
                )),
            },
        }],
        ..legend(
            "The Ur-Sphinx",
            cost(&[generic(6), w(), u(), b()]),
            vec![CreatureType::Sphinx, CreatureType::Avatar],
            10,
            10,
        )
    }
}

/// Turbulent Crater — Swamp Mountain; enters tapped unless your opponents
/// control eight or more lands.
pub fn turbulent_crater() -> CardDefinition {
    turbulent("Turbulent Crater", [LandType::Swamp, LandType::Mountain], [Color::Black, Color::Red])
}

/// Turbulent Shore — Plains Island; enters tapped unless your opponents
/// control eight or more lands.
pub fn turbulent_shore() -> CardDefinition {
    turbulent("Turbulent Shore", [LandType::Plains, LandType::Island], [Color::White, Color::Blue])
}

/// Venser, Fervent Forger — flash; ETB choose one: copy target instant or
/// sorcery spell an opponent controls twice; or two hasty token copies of
/// target permanent an opponent controls, sacrificed at the next end step.
pub fn venser_fervent_forger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::CopySpellMayChooseTargets {
                what: target_filtered(
                    R::IsSpellOnStack
                        .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
                        .and(R::ControlledByOpponent),
                ),
                count: Value::Const(2),
            },
            Effect::CreateTokenCopiesHasteSac {
                who: PlayerRef::You,
                count: Value::Const(2),
                source: target_filtered(R::Permanent.and(R::ControlledByOpponent)),
                exile: false,
            },
        ]))],
        ..legend(
            "Venser, Fervent Forger",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Human, CreatureType::Sorcerer],
            5,
            3,
        )
    }
}
