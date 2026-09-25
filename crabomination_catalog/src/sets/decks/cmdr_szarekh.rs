//! Commander: the cards the **Necron Dynasties** Warhammer 40,000 Commander
//! deck (40K, Szarekh, the Silent King) needed beyond what the catalog had.
//! Tests in `tests/recent_b/cmdr_szarekh.rs`.
//!
//! Residuals (each also on its card):
//! - **Biotransference** — creature spells and creature cards off the
//!   battlefield aren't artifacts (only permanents are); its own cast trigger
//!   reads "artifact or creature spell", which is the same set.
//! - **Out of the Tombs** — the reanimated card is the engine's pick (greatest
//!   mana value): the draw funnel can't suspend for an ask.
//! - **Canoptek Wraith** — each fetched basic shares a name with a permanent,
//!   not necessarily the chosen land.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered, unearth};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, x};
use std::sync::Arc;

fn artifact_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn necron(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    artifact_creature(name, mana, vec![CreatureType::Necron], p, t)
}

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn vehicle(name: &'static str, mana: ManaCost, p: i32, t: i32, crew: u32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Crew(crew)],
        ..Default::default()
    }
}

/// The deck's 2/2 black Necron Warrior artifact creature token.
fn warrior(tapped: bool) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Necron Warrior".to_string(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Necron, CreatureType::Warrior], ..Default::default() },
        tapped,
        ..Default::default()
    })
}

fn warriors(n: Value, tapped: bool) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: warrior(tapped) }
}

fn mill(n: i32) -> Effect {
    Effect::Mill { who: Selector::You, amount: Value::Const(n) }
}

fn artifact_creature_req() -> R {
    R::Artifact.and(R::Creature)
}

fn vehicle_req() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Vehicle)
}

/// "When this creature enters from a graveyard, …" (CR 603.6a, the zone it
/// came from).
fn enters_from_graveyard(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
            .with_filter(Predicate::TriggerSourceEnteredFromGraveyard),
        effect,
    }
}

/// "… is put into a graveyard from the battlefield or is put into exile from
/// the battlefield": a dies trigger (`PermanentDied`, any card type) and a
/// leave trigger whose subject now sits in exile.
fn leaves_to_graveyard_or_exile(scope: EventScope, filter: R, effect: Effect) -> [TriggeredAbility; 2] {
    let on = |kind, filter: R| TriggeredAbility {
        event: EventSpec::new(kind, scope)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect: effect.clone(),
    };
    [on(EventKind::PermanentDied, filter.clone()), on(EventKind::PermanentLeavesBattlefield, filter.and(R::InExile))]
}

/// Szarekh, the Silent King — flying; attacks: mill three, may take an
/// artifact creature or Vehicle card milled this way.
pub fn szarekh_the_silent_king() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::MillThenToHand {
            amount: Value::Const(3),
            filter: artifact_creature_req().or(vehicle_req()),
            otherwise: None,
        })],
        ..necron("Szarekh, the Silent King", cost(&[generic(1), b(), b(), b()]), 3, 4)
    })
}

/// Anrakyr the Traveller — attacks: cast an artifact spell from hand or
/// graveyard by paying life equal to its mana value.
pub fn anrakyr_the_traveller() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MayCastFromHandOrGraveyardForLife { filter: R::Artifact })],
        ..necron("Anrakyr the Traveller", cost(&[generic(4), b()]), 4, 4)
    })
}

/// Biotransference — creatures you control are artifacts; casting an
/// artifact spell costs 1 life and makes a Necron Warrior.
///
/// ⚠ Residual: only permanents become artifacts (not spells or cards in other
/// zones); the cast trigger reads "artifact or creature spell", the set the
/// printed text makes it.
pub fn biotransference() -> CardDefinition {
    CardDefinition {
        name: "Biotransference",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control are artifacts in addition to their other types.",
            effect: StaticEffect::AddCardTypeToMatching {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                card_type: CardType::Artifact,
                artifact_subtype: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.or(R::Creature) },
            ),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                warriors(Value::ONE, false),
            ]),
        }],
        ..Default::default()
    }
}

/// Canoptek Scarab Swarm — flying; enters: exile target player's graveyard,
/// a 1/1 flying Insect per artifact or land card exiled.
pub fn canoptek_scarab_swarm() -> CardDefinition {
    let insect = Arc::new(TokenDefinition {
        name: "Insect".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(Effect::Seq(vec![
                Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: None },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::count(Selector::ExiledThisResolution { filter: R::Artifact.or(R::Land) }),
                    definition: insect,
                },
            ])),
        })],
        ..artifact_creature("Canoptek Scarab Swarm", cost(&[generic(4)]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Canoptek Spyder — flying; another nontoken artifact creature or Vehicle of
/// yours entering draws a card.
pub fn canoptek_spyder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken.and(artifact_creature_req().or(vehicle_req())),
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..artifact_creature("Canoptek Spyder", cost(&[generic(5)]), vec![CreatureType::Spider], 4, 4)
    }
}

/// Canoptek Tomb Sentinel — vigilance; entering from a graveyard exiles up to
/// one nonland permanent. Unearth {7}.
pub fn canoptek_tomb_sentinel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![enters_from_graveyard(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Nonland.and(R::Permanent),
            effect: Box::new(Effect::Exile { what: Selector::Target(0) }),
        })],
        activated_abilities: vec![unearth(cost(&[generic(7)]))],
        ..artifact_creature("Canoptek Tomb Sentinel", cost(&[generic(4)]), vec![CreatureType::Insect], 4, 3)
    }
}

/// Canoptek Wraith — unblockable; combat damage to a player: may pay {3} and
/// sacrifice it for two basic lands named like a land you control.
///
/// ⚠ Residual: each basic shares a name with some permanent, not necessarily
/// the one land chosen.
pub fn canoptek_wraith() -> CardDefinition {
    let fetch = Effect::Search {
        who: PlayerRef::You,
        filter: R::IsBasicLand.and(R::SameNameAsAPermanent),
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
    };
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {3} and sacrifice Canoptek Wraith to fetch two basic lands?".into(),
                mana_cost: cost(&[generic(3)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::IsSource },
                    fetch.clone(),
                    fetch,
                ])),
                else_: None,
            },
        }],
        ..artifact_creature("Canoptek Wraith", cost(&[generic(3)]), vec![CreatureType::Wraith], 2, 1)
    }
}

/// Chronomancer — flying; {1}, {T}, sacrifice another artifact: draw.
/// Unearth {2}{B}.
pub fn chronomancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_other_filter: Some((R::Artifact, 1)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            unearth(cost(&[generic(2), b()])),
        ],
        ..artifact_creature("Chronomancer", cost(&[generic(1), b()]), vec![CreatureType::Necron, CreatureType::Wizard], 1, 1)
    }
}

/// Convergence of Dominion — with your commander out, graveyard abilities
/// cost {2} less (floor one mana); {3}, {T}: mill three.
pub fn convergence_of_dominion() -> CardDefinition {
    CardDefinition {
        name: "Convergence of Dominion",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "As long as you control your commander, activated abilities of cards in your graveyard cost {2} less to activate.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::ControlsOwnCommander { who: PlayerRef::You },
                inner: Box::new(StaticEffect::GraveyardActivatedAbilitiesCostLess { amount: 2 }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: mill(3),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cryptek — {1}{B}, {T}: when another artifact creature of yours dies this
/// turn, it returns tapped.
pub fn cryptek() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: true,
            effect: Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
                slot: 0,
                filter: Some(artifact_creature_req().and(R::ControlledByYou).and(R::OtherThanSource)),
            },
            ..Default::default()
        }],
        ..artifact_creature("Cryptek", cost(&[generic(3), b()]), vec![CreatureType::Necron, CreatureType::Wizard], 3, 3)
    }
}

/// Cryptothrall — other artifact creatures you control have hexproof.
pub fn cryptothrall() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other artifact creatures you control have hexproof.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    artifact_creature_req().and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                keyword: Keyword::Hexproof,
            },
        }],
        ..artifact_creature("Cryptothrall", cost(&[generic(4)]), vec![CreatureType::Construct], 3, 3)
    }
}

/// Flayed One — lifelink; enters: mill three.
pub fn flayed_one() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(mill(3))],
        ..necron("Flayed One", cost(&[generic(2), b()]), 4, 1)
    }
}

/// Ghost Ark — flying; becoming crewed gives each artifact creature card in
/// your graveyard unearth {3} until end of turn. Crew 2.
pub fn ghost_ark() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            // The crew event's subject is the Vehicle: "this Vehicle becomes crewed".
            event: EventSpec::new(EventKind::CrewsOrSaddles, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSource }),
            effect: Effect::GraveyardCardsGainUnearthThisTurn {
                filter: artifact_creature_req(),
                cost: cost(&[generic(3)]),
            },
        }],
        ..vehicle("Ghost Ark", cost(&[generic(4)]), 3, 3, 2)
    }
}

/// Hexmark Destroyer — can't be blocked except by six or more creatures.
/// Unearth {4}{B}{B}.
pub fn hexmark_destroyer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedExceptByN(6)],
        activated_abilities: vec![unearth(cost(&[generic(4), b(), b()]))],
        ..necron("Hexmark Destroyer", cost(&[generic(4), b(), b()]), 6, 6)
    }
}

/// Illuminor Szeras — {T}, sacrifice another creature: {B} per its mana value.
pub fn illuminor_szeras() -> CardDefinition {
    legendary(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Black, Value::SacrificedManaValue),
            },
            ..Default::default()
        }],
        ..necron("Illuminor Szeras", cost(&[generic(2), b()]), 3, 3)
    })
}

/// Imotekh the Stormlord — artifact cards leaving your graveyard make two
/// Warriors; your combat: another artifact creature gets +2/+2 and menace.
pub fn imotekh_the_stormlord() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                // CR 603.2c — "whenever one or more": one fire per batch.
                event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact })
                    .once_per_batch(),
                effect: warriors(Value::Const(2), false),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(artifact_creature_req().and(R::ControlledByYou).and(R::OtherThanSource)),
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Menace, duration: Duration::EndOfTurn },
                ]),
            },
        ],
        ..necron("Imotekh the Stormlord", cost(&[generic(2), b(), b()]), 3, 3)
    })
}

/// Lokhust Heavy Destroyer — flying; enters: each player sacrifices a
/// creature. Unearth {5}{B}{B}{B}.
pub fn lokhust_heavy_destroyer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::ONE,
            filter: R::Creature,
        })],
        activated_abilities: vec![unearth(cost(&[generic(5), b(), b(), b()]))],
        ..necron("Lokhust Heavy Destroyer", cost(&[generic(1), b(), b(), b()]), 3, 2)
    }
}

/// Lychguard — {3}{B}, sacrifice it: return all legendary creature cards from
/// your graveyard to your hand.
pub fn lychguard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            sac_cost: true,
            effect: Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..necron("Lychguard", cost(&[generic(2), b()]), 2, 3)
    }
}

/// Necron Deathmark — flash; enters: destroy up to one creature, and target
/// player mills three (two triggers: one "up to one" slot can't share a
/// target list with a required player slot).
pub fn necron_deathmark() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            }),
            etb(Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(3) }),
            }),
        ],
        ..necron("Necron Deathmark", cost(&[generic(3), b(), b()]), 5, 3)
    }
}

/// Necron Monolith — flying, indestructible; attacks: mill three, a Warrior
/// per creature card milled. Crew 4.
pub fn necron_monolith() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Indestructible, Keyword::Crew(4)],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            mill(3),
            warriors(Value::CreatureCardsMilledThisEffect, false),
        ]))],
        ..vehicle("Necron Monolith", cost(&[generic(7)]), 7, 7, 4)
    }
}

/// Necron Overlord — {X}, {T}, tap X untapped artifacts: target opponent
/// loses X life.
pub fn necron_overlord() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            tap_n_filter: Some((R::Artifact.and(R::OtherThanSource), 0)),
            tap_n_x: true,
            effect: Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::XFromCost },
            ..Default::default()
        }],
        ..artifact_creature(
            "Necron Overlord",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Necron, CreatureType::Noble],
            2,
            5,
        )
    }
}

/// Night Scythe — flying; enters: a Necron Warrior. Crew 2.
pub fn night_scythe() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![etb(warriors(Value::ONE, false))],
        ..vehicle("Night Scythe", cost(&[generic(3)]), 3, 1, 2)
    }
}

/// Out of the Tombs — upkeep: two eon counters, then mill that many; an
/// empty-library draw reanimates a creature card instead (or loses).
///
/// ⚠ Residual: the returned card is the greatest-mana-value one, not asked.
pub fn out_of_the_tombs() -> CardDefinition {
    CardDefinition {
        name: "Out of the Tombs",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Eon, amount: Value::Const(2) },
                Effect::Mill { who: Selector::You, amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Eon } },
            ]),
        }],
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card while your library has no cards in it, instead return a creature card from your graveyard to the battlefield. If you can't, you lose the game.",
            effect: StaticEffect::ReanimateInsteadOfDrawFromEmpty,
        }],
        ..Default::default()
    }
}

/// Plasmancer — flying; enters: search for a basic Swamp to hand.
pub fn plasmancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand.and(R::HasName("Swamp".into())),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..artifact_creature("Plasmancer", cost(&[generic(2), b(), b()]), vec![CreatureType::Necron, CreatureType::Wizard], 3, 3)
    }
}

/// Psychomancer — flying; it or another nontoken artifact of yours going to a
/// graveyard or exile from the battlefield drains a target opponent for 1.
pub fn psychomancer() -> CardDefinition {
    let drain = Effect::Seq(vec![
        Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::ONE },
        Effect::GainLife { who: Selector::You, amount: Value::ONE },
    ]);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: leaves_to_graveyard_or_exile(EventScope::SelfSource, R::Any, drain.clone())
            .into_iter()
            .chain(leaves_to_graveyard_or_exile(EventScope::AnotherOfYours, R::Artifact.and(R::NotToken), drain))
            .collect(),
        ..artifact_creature("Psychomancer", cost(&[generic(1), b()]), vec![CreatureType::Necron, CreatureType::Wizard], 1, 1)
    }
}

/// Resurrection Orb — equipped creature has lifelink; its death returns it at
/// the next end step. Equip {4}.
pub fn resurrection_orb() -> CardDefinition {
    CardDefinition {
        name: "Resurrection Orb",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Lifelink],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::AtNextEndStep {
                    body: Box::new(Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                    }),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Royal Warden — enters: two tapped Warriors. Unearth {3}{B}.
pub fn royal_warden() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(warriors(Value::Const(2), true))],
        activated_abilities: vec![unearth(cost(&[generic(3), b()]))],
        ..necron("Royal Warden", cost(&[generic(3), b(), b()]), 3, 2)
    }
}

/// Sautekh Immortal — flash; enters with a +1/+1 counter per creature that
/// died this turn.
pub fn sautekh_immortal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::CreaturesDiedThisTurnTotal)),
        ..necron("Sautekh Immortal", cost(&[generic(2), b()]), 2, 2)
    }
}

/// Sceptre of Eternal Glory — {T}: any color; {T}: three of one color with
/// three lands sharing a name.
pub fn sceptre_of_eternal_glory() -> CardDefinition {
    legendary(CardDefinition {
        name: "Sceptre of Eternal Glory",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            crate::sets::tap_add_any_color(),
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::ControlsLandsWithSameNameAtLeast { who: PlayerRef::You, at_least: 3 }),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(3)) },
                ..Default::default()
            },
        ],
        ..Default::default()
    })
}

/// Shard of the Nightbringer — flying; enters, if cast: target opponent loses
/// half their life rounded up, and you gain that much.
pub fn shard_of_the_nightbringer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SourceWasCast,
            then: Box::new(Effect::DrainLifeLost {
                from: target_filtered(R::OpponentPlayer),
                to: Selector::You,
                amount: Value::HalfLifeRoundedUp(PlayerRef::Target(0)),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..CardDefinition {
            name: "Shard of the Nightbringer",
            cost: cost(&[generic(5), b(), b(), b()]),
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Ctan], ..Default::default() },
            power: 8,
            toughness: 8,
            ..Default::default()
        }
    }
}

/// Shard of the Void Dragon — flying; attacks: each opponent sacrifices a
/// nonland permanent; any artifact to a graveyard or exile from the
/// battlefield grows it by two +1/+1 counters.
pub fn shard_of_the_void_dragon() -> CardDefinition {
    let mut triggers = vec![on_attack(Effect::Sacrifice {
        who: Selector::Player(PlayerRef::EachOpponent),
        count: Value::ONE,
        filter: R::Nonland.and(R::Permanent),
    })];
    triggers.extend(leaves_to_graveyard_or_exile(
        EventScope::AnyPlayer,
        R::Artifact,
        Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
    ));
    CardDefinition {
        name: "Shard of the Void Dragon",
        cost: cost(&[generic(4), b(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ctan], ..Default::default() },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: triggers,
        ..Default::default()
    }
}

/// Skorpekh Destroyer — deathtouch; an artifact of yours entering gives it
/// first strike until end of turn.
pub fn skorpekh_destroyer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..necron("Skorpekh Destroyer", cost(&[generic(2), b(), b()]), 4, 2)
    }
}

/// Skorpekh Lord — menace; other artifact creatures you control get +1/+0 and
/// have menace. Unearth {2}{B}.
pub fn skorpekh_lord() -> CardDefinition {
    let others = || Selector::EachPermanent(artifact_creature_req().and(R::ControlledByYou).and(R::OtherThanSource));
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![
            StaticAbility {
                description: "Other artifact creatures you control get +1/+0.",
                effect: StaticEffect::PumpPT { applies_to: others(), power: 1, toughness: 0 },
            },
            StaticAbility {
                description: "Other artifact creatures you control have menace.",
                effect: StaticEffect::GrantKeyword { applies_to: others(), keyword: Keyword::Menace },
            },
        ],
        activated_abilities: vec![unearth(cost(&[generic(2), b()]))],
        ..artifact_creature("Skorpekh Lord", cost(&[generic(2), b()]), vec![CreatureType::Necron, CreatureType::Noble], 3, 2)
    }
}

/// Technomancer — enters: mill three, then return artifact creature cards with
/// total mana value 6 or less.
pub fn technomancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            mill(3),
            Effect::MoveWithinTotalManaValue {
                from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Any },
                filter: artifact_creature_req(),
                cap: Value::Const(6),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                max_count: None,
            },
        ]))],
        ..artifact_creature(
            "Technomancer",
            cost(&[generic(5), b(), b()]),
            vec![CreatureType::Necron, CreatureType::Wizard],
            5,
            1,
        )
    }
}

/// The War in Heaven — Saga: draw three and lose 3; mill three; return up to
/// three creature cards (total MV 8 or less) as necrodermis artifacts.
pub fn the_war_in_heaven() -> CardDefinition {
    CardDefinition {
        name: "The War in Heaven",
        cost: cost(&[generic(3), b(), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
                ]),
            ),
            (2, mill(3)),
            (
                3,
                Effect::Seq(vec![
                    Effect::MoveWithinTotalManaValue {
                        from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Any },
                        filter: R::Creature,
                        cap: Value::Const(8),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        max_count: Some(3),
                    },
                    Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Necrodermis, amount: Value::ONE },
                    Effect::AddCardTypeIndefinitely { what: Selector::LastMoved, card_type: CardType::Artifact, until_eot: false },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Their Name Is Death — destroy all nonartifact creatures.
pub fn their_name_is_death() -> CardDefinition {
    CardDefinition {
        name: "Their Name Is Death",
        cost: cost(&[generic(3), b(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::Artifact)))) },
        ..Default::default()
    }
}

/// Their Number Is Legion — X tapped Warriors, then life per artifact you
/// control; exiles itself; castable from your graveyard.
pub fn their_number_is_legion() -> CardDefinition {
    CardDefinition {
        name: "Their Number Is Legion",
        cost: cost(&[x(), b(), b(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            warriors(Value::XFromCost, true),
            Effect::GainLife {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))),
            },
        ]),
        static_abilities: vec![StaticAbility {
            description: "You may cast this card from your graveyard.",
            effect: StaticEffect::GraveyardCastWithLifeSurcharge {
                filter: R::HasName("Their Number Is Legion".into()),
                life: 0,
            },
        }],
        exile_on_resolve: true,
        ..Default::default()
    }
}

/// Tomb Blade — flying; combat damage to a player: they lose life per
/// creature they control unless they sacrifice a creature. Unearth {6}{B}{B}.
pub fn tomb_blade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Punisher {
                chooser: Selector::Player(PlayerRef::Triggerer),
                options: vec![Effect::Sacrifice { who: Selector::Player(PlayerRef::You), count: Value::ONE, filter: R::Creature }],
                otherwise: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::count(Selector::ControlledBy { who: PlayerRef::Triggerer, filter: R::Creature }),
                }),
            },
        }],
        activated_abilities: vec![unearth(cost(&[generic(6), b(), b()]))],
        ..necron("Tomb Blade", cost(&[generic(4), b(), b()]), 5, 4)
    }
}

/// Tomb Fortress — enters tapped; {T}: {B}; {2}{B}{B}{B}, {T}, exile it: mill
/// four, then return a creature card from your graveyard (sorcery speed).
pub fn tomb_fortress() -> CardDefinition {
    CardDefinition {
        name: "Tomb Fortress",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Black),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b(), b(), b()]),
                tap_cost: true,
                exile_self_cost: true,
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::Mill { who: Selector::You, amount: Value::Const(4) },
                    Effect::MoveChosen {
                        from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature },
                        filter: None,
                        count: Value::ONE,
                        up_to: false,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Trazyn the Infinite — deathtouch; has the activated abilities of every
/// artifact card in your graveyard.
pub fn trazyn_the_infinite() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        static_abilities: vec![StaticAbility {
            description: "As long as Trazyn is on the battlefield, it has all activated abilities of all artifact cards in your graveyard.",
            effect: StaticEffect::HasActivatedAbilitiesOfYourGraveyardArtifacts,
        }],
        ..necron("Trazyn the Infinite", cost(&[generic(4), b(), b()]), 4, 6)
    })
}

/// Triarch Praetorian — flying; entering from a graveyard draws two and loses
/// 2. Unearth {4}{B}.
pub fn triarch_praetorian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![enters_from_graveyard(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        activated_abilities: vec![unearth(cost(&[generic(4), b()]))],
        ..necron("Triarch Praetorian", cost(&[generic(1), b()]), 2, 1)
    }
}

/// Triarch Stalker — your combat: choose an opponent; creatures attacking the
/// last chosen player have menace.
pub fn triarch_stalker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::ChooseOpponentThen { then: Box::new(Effect::Noop) },
        }],
        static_abilities: vec![StaticAbility {
            description: "Creatures attacking the last chosen player have menace.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::IsAttackingChosenPlayerOfSource),
                keyword: Keyword::Menace,
            },
        }],
        ..necron("Triarch Stalker", cost(&[generic(3), b(), b()]), 4, 5)
    }
}
