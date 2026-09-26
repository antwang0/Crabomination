//! Commander: the cards the **Ahoy Mateys** precon (LCC, Admiral Brass,
//! Unsinkable) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_brass.rs`.
//!
//! Residuals (each also on its card):
//! - **Admiral Beckett Brass** — any damage from three Pirates this turn
//!   counts, not only combat damage.
//! - **Arm-Mounted Anchor** — equip always costs {2}; the hand-size discount
//!   isn't modelled.
//! - **Departed Deckhand** — it's sacrificed when any spell *or ability*
//!   targets it.
//! - **Gemcutter Buccaneer** — Treasures get equip {3} only, not equip
//!   Pirate {1}.
//! - **Merchant Raiders** — the lock lasts while it's on the battlefield,
//!   not while you control it.
//! - **Port Razer** — its trigger fires once a turn instead of "can't attack
//!   a player it has already attacked this turn".
//! - **Siren Stormtamer** — counters spells only, targeting you or any
//!   permanent you control.
//! - **Timestream Navigator** — it goes to the bottom as part of the effect,
//!   not as a cost.
//! - **Zara, Renegade Recruiter** — the stolen creature is the engine's pick,
//!   and you don't look at the rest of the hand.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, u};
use crate::sets::ascend;
use crabomination_base::tokens::{map_token, treasure_token};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn pirate() -> R {
    R::HasCreatureType(CreatureType::Pirate)
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn treasures(n: i32, tapped: bool) -> Effect {
    let make = Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(treasure_token()) };
    if tapped { Effect::Seq(vec![make, Effect::Tap { what: Selector::LastCreatedTokens }]) } else { make }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

/// "Whenever this creature or another Pirate you control enters".
fn pirate_enters(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: pirate() }),
        effect,
    }
}

/// "Whenever one or more Pirates you control deal combat damage to a player".
fn pirates_connect(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec {
            once_per_batch: true,
            ..EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: pirate() })
        },
        effect,
    }
}

fn on_combat_damage(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

fn pirate_token(name: &str, types: Vec<CreatureType>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.to_string(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

/// Admiral Beckett Brass — other Pirates +1/+1; at your end step, steal a
/// nonland permanent of a player three or more Pirates damaged this turn.
///
/// ⚠ Residual: any damage from the Pirates counts, not only combat damage.
pub fn admiral_beckett_brass() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Pirates you control get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: yours(pirate().and(R::OtherThanSource)), power: 1, toughness: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::GainControl {
                what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByPlayerDamagedByAtLeast {
                    filter: Box::new(pirate()),
                    n: 3,
                })),
                to: None,
                duration: Duration::Permanent,
            },
        }],
        ..creature(
            "Admiral Beckett Brass",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            3,
            3,
        )
    })
}

/// Admiral Brass, Unsinkable — entering, mill four; at the beginning of
/// combat on your turn, you may return a Pirate creature card as a 4/4 with
/// a finality counter and haste.
pub fn admiral_brass_unsinkable() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Mill { who: Selector::You, amount: Value::Const(4) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Return a Pirate creature card with a finality counter?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: target_filtered(R::Creature.and(pirate()).and(R::InYourGraveyard)),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                        Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Finality, amount: Value::ONE },
                        Effect::SetBasePT {
                            what: Selector::LastMoved,
                            power: Value::Const(4),
                            toughness: Value::Const(4),
                            duration: Duration::Permanent,
                        },
                        Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                    ])),
                },
            },
        ],
        ..creature(
            "Admiral Brass, Unsinkable",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            3,
            3,
        )
    })
}

/// Arm-Mounted Anchor — equipped creature gets +2/+2 and menace; connecting,
/// draw two, then discard two unless you discard a Pirate; equip {2}.
///
/// ⚠ Residual: equip always costs {2}; the hand-size discount isn't
/// modelled.
pub fn arm_mounted_anchor() -> CardDefinition {
    CardDefinition {
        name: "Arm-Mounted Anchor",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Menace],
            triggered_abilities: vec![on_combat_damage(Effect::Seq(vec![
                draw(2),
                Effect::DiscardUnlessKind { who: PlayerRef::You, count: Value::Const(2), instead: pirate() },
            ]))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Azure Fleet Admiral — entering makes you the monarch; the monarch's
/// creatures can't block it.
pub fn azure_fleet_admiral() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::ControlledByMonarch))],
        triggered_abilities: vec![etb(Effect::BecomeMonarch { who: PlayerRef::You })],
        ..creature(
            "Azure Fleet Admiral",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            3,
            3,
        )
    }
}

/// Blood Money — destroy all creatures; a tapped Treasure per nontoken
/// creature destroyed.
pub fn blood_money() -> CardDefinition {
    CardDefinition {
        name: "Blood Money",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::DestroyedThisResolution { filter: R::NotToken })),
                definition: Arc::new(treasure_token()),
            },
            Effect::Tap { what: Selector::LastCreatedTokens },
        ]),
        ..Default::default()
    }
}

/// Breeches, Brazen Plunderer — menace, partner; Pirates dealing combat
/// damage to an opponent exile their top card, playable this turn with any
/// mana.
pub fn breeches_brazen_plunderer() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Partner],
        triggered_abilities: vec![pirates_connect(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::TriggerEventPlayer,
            count: Value::ONE,
            duration: MayPlayDuration::EndOfThisTurn,
            pay_any_color: true,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        })],
        ..creature(
            "Breeches, Brazen Plunderer",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Pirate],
            3,
            3,
        )
    })
}

/// Coercive Recruiter — it or another Pirate entering steals a creature
/// until end of turn, untapped and hasty, and makes it a Pirate.
pub fn coercive_recruiter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![pirate_enters(Effect::Seq(vec![
            Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::EndOfTurn },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            Effect::AddCreatureTypes {
                what: Selector::Target(0),
                creature_types: vec![CreatureType::Pirate],
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature(
            "Coercive Recruiter",
            cost(&[generic(4), r()]),
            vec![CreatureType::Orc, CreatureType::Pirate],
            4,
            3,
        )
    }
}

/// Departed Deckhand — sacrificed when targeted; only Spirits block it;
/// {3}{U}: another creature of yours gains that evasion this turn.
///
/// ⚠ Residual: any spell or ability targeting it sacrifices it.
pub fn departed_deckhand() -> CardDefinition {
    let spirits_only = || Keyword::CantBeBlockedExceptBy(Box::new(R::HasCreatureType(CreatureType::Spirit)));
    CardDefinition {
        keywords: vec![spirits_only()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::SacrificeSource,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: spirits_only(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Departed Deckhand",
            cost(&[generic(1), u()]),
            vec![CreatureType::Spirit, CreatureType::Pirate],
            2,
            2,
        )
    }
}

/// Don Andres, the Renegade — creatures you control but don't own get
/// +2/+2, menace, deathtouch and become Pirates; casting a noncreature spell
/// you don't own makes two tapped Treasures.
pub fn don_andres_the_renegade() -> CardDefinition {
    let stolen = || yours(R::Creature.and(R::OwnedByYou.negate()));
    legendary(CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Each creature you control but don't own gets +2/+2.",
                effect: StaticEffect::PumpPT { applies_to: stolen(), power: 2, toughness: 2 },
            },
            StaticAbility {
                description: "Each creature you control but don't own has menace.",
                effect: StaticEffect::GrantKeyword { applies_to: stolen(), keyword: Keyword::Menace },
            },
            StaticAbility {
                description: "Each creature you control but don't own has deathtouch.",
                effect: StaticEffect::GrantKeyword { applies_to: stolen(), keyword: Keyword::Deathtouch },
            },
            StaticAbility {
                description: "Each creature you control but don't own is a Pirate.",
                effect: StaticEffect::AddCreatureTypeToMatching { applies_to: stolen(), creature_type: CreatureType::Pirate },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellNotOwnedByYou,
                Predicate::CastSpellMatches(R::Creature.negate()),
            ])),
            effect: treasures(2, true),
        }],
        ..creature(
            "Don Andres, the Renegade",
            cost(&[generic(1), u(), b(), r()]),
            vec![CreatureType::Vampire, CreatureType::Pirate],
            4,
            3,
        )
    })
}

/// Fathom Fleet Captain — menace; attacking beside another nontoken Pirate,
/// you may pay {2} for a 2/2 menace Pirate.
pub fn fathom_fleet_captain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: yours(pirate().and(R::NotToken).and(R::OtherThanSource)),
                n: Value::ONE,
            },
            then: Box::new(Effect::MayPay {
                description: "Pay {2} for a 2/2 Pirate with menace?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(TokenDefinition {
                        keywords: vec![Keyword::Menace],
                        ..(*pirate_token("Pirate", vec![CreatureType::Pirate])).clone()
                    }),
                }),
                else_: None,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Fathom Fleet Captain",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            2,
            1,
        )
    }
}

/// Francisco, Fowl Marauder — flying, can't block, partner; Pirates
/// connecting make it explore.
pub fn francisco_fowl_marauder() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CantBlock, Keyword::Partner],
        triggered_abilities: vec![pirates_connect(Effect::Explore { who: Selector::This })],
        ..creature(
            "Francisco, Fowl Marauder",
            cost(&[generic(1), b()]),
            vec![CreatureType::Bird, CreatureType::Pirate],
            0,
            1,
        )
    })
}

/// Gemcutter Buccaneer — it or another Pirate entering makes a tapped
/// Treasure; your Treasures are Equipment with +2/+0.
///
/// ⚠ Residual: Treasures get equip {3} only, not equip Pirate {1}.
pub fn gemcutter_buccaneer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Treasures you control are Equipment with \"Equipped creature gets +2/+0\" and equip {3}.",
            effect: StaticEffect::MatchingArtifactsAreEquipment {
                filter: R::HasArtifactSubtype(ArtifactSubtype::Treasure).and(R::ControlledByYou),
                equip: cost(&[generic(3)]),
                power: 2,
            },
        }],
        triggered_abilities: vec![pirate_enters(treasures(1, true))],
        ..creature(
            "Gemcutter Buccaneer",
            cost(&[generic(3), r()]),
            vec![CreatureType::Orc, CreatureType::Pirate, CreatureType::Artificer],
            1,
            3,
        )
    }
}

/// Ghost of Ramirez DePietro — toughness 3+ can't block it; partner;
/// connecting, a card discarded or milled this turn returns to its owner's
/// hand.
pub fn ghost_of_ramirez_depietro() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::ToughnessAtLeast(3))), Keyword::Partner],
        triggered_abilities: vec![on_combat_damage(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::DiscardedThisTurn.or(R::PutIntoGraveyardFromLibraryThisTurn).and(R::InGraveyard),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
        })],
        ..creature(
            "Ghost of Ramirez DePietro",
            cost(&[generic(2), u()]),
            vec![CreatureType::Spirit, CreatureType::Pirate],
            2,
            3,
        )
    })
}

/// King Narfi's Betrayal — I: each player mills four, then you may exile a
/// creature or planeswalker card from each graveyard. II, III: you may cast
/// those this turn, with any mana.
pub fn king_narfis_betrayal() -> CardDefinition {
    let cast_them = || Effect::GrantMayPlay {
        what: Selector::CardExiledWithSource,
        duration: MayPlayDuration::EndOfThisTurn,
        to_owner: false,
        exile_after: false,
        pay_own_cost: true,
        any_color: true,
    };
    CardDefinition {
        name: "King Narfi's Betrayal",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(4) },
                    Effect::ForEach {
                        selector: Selector::Player(PlayerRef::EachPlayer),
                        body: Box::new(Effect::ExileLinked {
                            what: Selector::Take {
                                inner: Box::new(Selector::CardsInZone {
                                    who: PlayerRef::Triggerer,
                                    zone: Zone::Graveyard,
                                    filter: R::Creature.or(R::Planeswalker),
                                }),
                                count: Box::new(Value::ONE),
                            },
                        }),
                    },
                ]),
            ),
            (2, cast_them()),
            (3, cast_them()),
        ],
        ..Default::default()
    }
}

/// Merchant Raiders — it or another Pirate entering taps up to one creature,
/// which stays tapped while this is around.
///
/// ⚠ Residual: the lock lasts while it's on the battlefield, not while you
/// control it.
pub fn merchant_raiders() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![pirate_enters(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::TapAndLockWhileSourcePresent { what: Selector::Target(0) }),
        })],
        ..creature(
            "Merchant Raiders",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            2,
            4,
        )
    }
}

/// Port Razer — connecting untaps your creatures and adds a combat phase.
///
/// ⚠ Residual: the trigger fires once a turn instead of "can't attack a
/// player it has already attacked this turn" (which is what ends the loop).
pub fn port_razer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource).once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::Untap { what: yours(R::Creature), up_to: None },
                Effect::AdditionalCombatPhase { count: Value::ONE },
            ]),
        }],
        ..creature(
            "Port Razer",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Orc, CreatureType::Pirate],
            4,
            4,
        )
    }
}

/// Ramirez DePietro, Pillager — entering, lose 2 life for two Treasures;
/// Pirates connecting exile that player's top card, castable while exiled.
pub fn ramirez_depietro_pillager() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![Effect::LoseLife { who: Selector::You, amount: Value::Const(2) }, treasures(2, false)])),
            pirates_connect(Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::TriggerEventPlayer,
                count: Value::ONE,
                duration: MayPlayDuration::WhileExiled,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            }),
        ],
        ..creature(
            "Ramirez DePietro, Pillager",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            4,
            3,
        )
    })
}

/// Siren Stormtamer — flying; {U}, sacrifice it: counter a spell that
/// targets you or a creature you control.
///
/// ⚠ Residual: counters spells only, targeting you or any permanent you
/// control.
pub fn siren_stormtamer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::CounterSpellOrAbility {
                what: target_filtered(R::IsSpellOnStack.and(R::SpellTargetsControllerOrControlled)),
            },
            ..Default::default()
        }],
        ..creature(
            "Siren Stormtamer",
            cost(&[u()]),
            vec![CreatureType::Siren, CreatureType::Pirate, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Skeleton Crew — other Skeletons and Pirates get +1/+1; creature cards
/// leaving your graveyard make a 2/2 Skeleton Pirate; {5}{B}: return it from
/// your graveyard tapped.
pub fn skeleton_crew() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each other creature you control that's a Skeleton or Pirate gets +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: yours(
                    R::HasCreatureType(CreatureType::Skeleton).or(pirate()).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_batch: true,
                ..EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature })
            },
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: pirate_token("Skeleton Pirate", vec![CreatureType::Skeleton, CreatureType::Pirate]),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b()]),
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature(
            "Skeleton Crew",
            cost(&[generic(3), b()]),
            vec![CreatureType::Skeleton, CreatureType::Pirate],
            3,
            3,
        )
    }
}

/// Storm Fleet Negotiator — flying; parley: a Map token per nonland card
/// revealed, then each player draws.
pub fn storm_fleet_negotiator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Parley {
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CardsRevealedThisEffect,
                definition: Arc::new(map_token()),
            }),
        })],
        ..creature(
            "Storm Fleet Negotiator",
            cost(&[generic(2), u()]),
            vec![CreatureType::Siren, CreatureType::Pirate],
            2,
            2,
        )
    }
}

/// The Grim Captain's Locker — {T}: surveil 1; {T}: your graveyard's creature
/// cards gain escape {3}{B} (exile four others) this turn.
pub fn the_grim_captains_locker() -> CardDefinition {
    CardDefinition {
        name: "The Grim Captain's Locker",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::GrantEscapeWithCostThisTurn {
                    what: Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Graveyard, filter: R::Creature },
                    cost: cost(&[generic(3), b()]),
                    exile_count: 4,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// The Indomitable — trample Vehicle, crew 3; a creature of yours connecting
/// draws a card; castable from your graveyard with three tapped Pirates
/// and/or Vehicles.
pub fn the_indomitable() -> CardDefinition {
    CardDefinition {
        name: "The Indomitable",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample, Keyword::Crew(3), Keyword::GraveyardCast],
        flashback_condition: Some(Predicate::SelectorCountAtLeast {
            sel: yours(R::Tapped.and(pirate().or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)))),
            n: Value::Const(3),
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: draw(1),
        }],
        ..Default::default()
    }
}

/// Timestream Navigator — ascend; with the city's blessing, {2}{U}{U}, {T},
/// put it on the bottom of its library: take an extra turn.
///
/// ⚠ Residual: it goes to the bottom as part of the effect, not as a cost.
pub fn timestream_navigator() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![ascend()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), u()]),
            tap_cost: true,
            condition: Some(Predicate::HasCityBlessing { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom },
                },
                Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Timestream Navigator",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Pirate, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Warkite Marauder — flying; attacking, a defending creature loses all
/// abilities and is a 0/1 until end of turn.
pub fn warkite_marauder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::LoseAllAbilities {
                what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                duration: Duration::EndOfTurn,
            },
            Effect::SetBasePT {
                what: Selector::Target(0),
                power: Value::ZERO,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature(
            "Warkite Marauder",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            2,
            1,
        )
    }
}

/// Zara, Renegade Recruiter — flying; attacking, put a creature card from
/// the defending player's hand onto the battlefield under your control,
/// tapped and attacking; it returns to its owner's hand at the next end
/// step.
///
/// ⚠ Residual: the creature is the engine's pick, and you don't look at the
/// rest of the hand.
pub fn zara_renegade_recruiter() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Put a creature from the defending player's hand onto the battlefield attacking?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::DefendingPlayer,
                            zone: Zone::Hand,
                            filter: R::Creature,
                        }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                Effect::JoinCombatAttacking { what: Selector::LastMoved },
                Effect::ReturnToOwnersHandAtNextEndStep { what: Selector::LastMoved },
            ])),
        })],
        ..creature(
            "Zara, Renegade Recruiter",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            4,
            3,
        )
    })
}
