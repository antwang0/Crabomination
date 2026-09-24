//! Commander: the cards the **World Shaper** precon (EOC, Hearthhull, the
//! Worldseed) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_hearthhull.rs`.
//!
//! Residuals (each also on its card):
//! - **Eumidian Wastewaker** — you and the defending player each discard a
//!   card; neither may sacrifice a permanent instead.
//! - **Moraug, Fury of Akoum** — +1/+0 once for a creature that has attacked
//!   this turn, however many times it attacked; the untap rides every later
//!   combat this turn.
//! - **Planetary Annihilation** — each player keeps their six lands of
//!   highest mana value (the engine's pick).
//! - **Scouring Swarm** and **Soul of Windgrace** — the token is tapped just
//!   after it is created; Windgrace's land comes from the first graveyard
//!   holding one.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, StationBand, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{encore, etb, landfall, on_attack, on_dies, station, target_any};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r};
use crate::sets::enters_tapped;
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn insect_token() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    })
}

fn you_sacrifice(filter: R) -> EventSpec {
    EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter })
}

fn lands_in_your_graveyard() -> Value {
    Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Land }
}

/// "When this land enters, sacrifice it. When you do, search for a basic A,
/// B, or C, put it onto the battlefield tapped, then gain 1 life" — the SNC
/// sacrifice lands.
fn sac_land(name: &'static str, types: [LandType; 3]) -> CardDefinition {
    let [a, b, c] = types;
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::SacrificeSource,
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand.and(R::HasLandType(a).or(R::HasLandType(b)).or(R::HasLandType(c))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]))],
        ..Default::default()
    }
}

/// Hearthhull, the Worldseed — Spacecraft, station. 2+: {1}, {T}, sacrifice
/// a land: draw two and play an additional land this turn. 8+: flying,
/// vigilance, haste (a 6/7). Whenever you sacrifice a land, each opponent
/// loses 2 life.
pub fn hearthhull_the_worldseed() -> CardDefinition {
    CardDefinition {
        name: "Hearthhull, the Worldseed",
        cost: cost(&[generic(1), b(), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        power: 6,
        toughness: 7,
        activated_abilities: vec![station()],
        station: vec![
            StationBand {
                min: 2,
                activated: vec![ActivatedAbility {
                    mana_cost: cost(&[generic(1)]),
                    tap_cost: true,
                    sac_other_filter: Some((R::Land, 1)),
                    effect: Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                        Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::ONE },
                    ]),
                    ..Default::default()
                }],
                ..Default::default()
            },
            StationBand {
                min: 8,
                keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
                pt: Some((6, 7)),
                ..Default::default()
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: you_sacrifice(R::Land),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// Cabaretti Courtyard — sacrifice on entry for a basic Mountain, Forest, or
/// Plains, tapped, and 1 life.
pub fn cabaretti_courtyard() -> CardDefinition {
    sac_land("Cabaretti Courtyard", [LandType::Mountain, LandType::Forest, LandType::Plains])
}

/// Maestros Theater — sacrifice on entry for a basic Island, Swamp, or
/// Mountain, tapped, and 1 life.
pub fn maestros_theater() -> CardDefinition {
    sac_land("Maestros Theater", [LandType::Island, LandType::Swamp, LandType::Mountain])
}

/// Eumidian Hatchery — {T}, pay 1 life: add {B} and a hatchling counter;
/// going to a graveyard from the battlefield, a 1/1 flying Insect per
/// hatchling counter (CR 603.10).
pub fn eumidian_hatchery() -> CardDefinition {
    CardDefinition {
        name: "Eumidian Hatchery",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 1,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Black, Value::ONE) },
                Effect::AddCounter { what: Selector::This, kind: CounterType::Hatchling, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Hatchling },
                definition: insect_token(),
            },
        }],
        ..Default::default()
    }
}

/// Eumidian Wastewaker — whenever it attacks, you and the defending player
/// each discard; you draw per land card discarded. Encore {6}{B}{B}.
///
/// ⚠ Residual: nobody may sacrifice a permanent instead of discarding.
pub fn eumidian_wastewaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            Effect::Discard { who: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::ONE, random: false },
            Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::DiscardedThisResolution { filter: R::Land })),
            },
        ]))],
        activated_abilities: vec![encore(cost(&[generic(6), b(), b()]))],
        ..creature(
            "Eumidian Wastewaker",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Insect, CreatureType::Cleric],
            4,
            4,
        )
    }
}

/// Festering Thicket — Swamp Forest, enters tapped, cycling {2}.
pub fn festering_thicket() -> CardDefinition {
    let mut d = crate::sets::dual_land_with(
        "Festering Thicket",
        LandType::Swamp,
        LandType::Forest,
        Color::Black,
        Color::Green,
        vec![],
    );
    d.static_abilities.push(enters_tapped());
    d.keywords.push(Keyword::Cycling(cost(&[generic(2)])));
    d
}

/// Horizon Explorer — your lands enter untapped; whenever you attack, a
/// Lander token.
pub fn horizon_explorer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Lands you control enter untapped.",
            effect: StaticEffect::LandsEnterUntapped,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::lander_token()),
            },
        }],
        ..creature(
            "Horizon Explorer",
            cost(&[generic(2), g()]),
            vec![CreatureType::Insect, CreatureType::Scout],
            2,
            4,
        )
    }
}

/// Juri, Master of the Revue — a +1/+1 counter per permanent you sacrifice;
/// dying, damage equal to its power to any target.
pub fn juri_master_of_the_revue() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: you_sacrifice(R::Permanent),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            on_dies(Effect::DealDamage { to: target_any(), amount: Value::PowerOf(Box::new(Selector::This)) }),
        ],
        ..legendary(creature(
            "Juri, Master of the Revue",
            cost(&[b(), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            1,
            1,
        ))
    }
}

/// Loamcrafter Faun — on entry you may discard land cards; then return up to
/// that many nonland permanent cards from your graveyard to hand.
///
/// ⚠ Residual: the cards are targeted as the trigger goes on the stack and
/// capped at the discard count as it resolves (Miasma Demon's shape), not
/// chosen after the discard.
pub fn loamcrafter_faun() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DiscardAnyNumber { who: Selector::You, filter: R::Land, max: None },
            Effect::CapTargetsAt {
                amount: Value::CardsDiscardedThisEffect,
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::PermanentCard.and(R::Nonland).and(R::InYourGraveyard),
                    effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
                }),
            },
        ]))],
        ..creature(
            "Loamcrafter Faun",
            cost(&[generic(2), g()]),
            vec![CreatureType::Satyr, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Moraug, Fury of Akoum — your creatures get +1/+0 for having attacked this
/// turn; landfall in your main phase adds a combat after this phase, which
/// untaps your creatures as it begins.
///
/// ⚠ Residual: +1/+0 once, however many times the creature attacked; the
/// untap comes at the beginning of every later combat this turn, not only
/// the added one.
pub fn moraug_fury_of_akoum() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature you control gets +1/+0 for each time it has attacked this turn.",
            effect: StaticEffect::PumpPT {
                applies_to: yours(R::Creature.and(R::AttackedThisTurn)),
                power: 1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![landfall(Effect::If {
            cond: Predicate::YourMainPhase,
            then: Box::new(Effect::Seq(vec![
                Effect::AdditionalCombatPhaseAfterMain { count: Value::ONE },
                Effect::AtEachCombatThisTurn {
                    body: Box::new(Effect::Untap { what: yours(R::Creature), up_to: None }),
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..legendary(creature(
            "Moraug, Fury of Akoum",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Minotaur, CreatureType::Warrior],
            6,
            6,
        ))
    }
}

/// Planetary Annihilation — each player keeps six lands and sacrifices the
/// rest; 6 damage to each creature.
///
/// ⚠ Residual: the six kept are the engine's pick.
pub fn planetary_annihilation() -> CardDefinition {
    CardDefinition {
        name: "Planetary Annihilation",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::EachPlayerKeepsNSacrificesRest { keep: Value::Const(6), filter: Some(R::Land) },
            Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(6) },
        ]),
        ..Default::default()
    }
}

/// Scouring Swarm — flying; sacrificing a land makes a tapped copy of it
/// with seven or more lands in your graveyard, else a tapped 1/1 Insect.
pub fn scouring_swarm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: you_sacrifice(R::Land),
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::ValueAtLeast(lands_in_your_graveyard(), Value::Const(7)),
                    then: Box::new(Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: Selector::This,
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: true,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![],
                    }),
                    else_: Box::new(Effect::Seq(vec![
                        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: insect_token() },
                        Effect::Tap { what: Selector::LastCreatedToken },
                    ])),
                },
            ]),
        }],
        ..creature("Scouring Swarm", cost(&[generic(1), b(), g()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Soul of Windgrace — entering or attacking, you may put a land card from a
/// graveyard onto the battlefield tapped; three discard-a-land abilities.
pub fn soul_of_windgrace() -> CardDefinition {
    let fetch = || {
        Effect::MayDo {
            description: "Put a land card from a graveyard onto the battlefield?".into(),
            body: Box::new(Effect::Move {
                what: Selector::TakeGreatestPower {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::EachPlayer,
                        zone: Zone::Graveyard,
                        filter: R::Land,
                    }),
                    count: Box::new(Value::ONE),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
        }
    };
    let discard_land = |mana: crate::mana::ManaCost, effect: Effect| ActivatedAbility {
        mana_cost: mana,
        discard_cost: Some((R::Land, 1)),
        effect,
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(fetch()), on_attack(fetch())],
        activated_abilities: vec![
            discard_land(cost(&[g()]), Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
            discard_land(cost(&[generic(1), r()]), Effect::Draw { who: Selector::You, amount: Value::ONE }),
            discard_land(
                cost(&[generic(2), b()]),
                Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Indestructible,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Tap { what: Selector::This },
                ]),
            ),
        ],
        ..legendary(creature(
            "Soul of Windgrace",
            cost(&[generic(1), b(), r(), g()]),
            vec![CreatureType::Cat, CreatureType::Avatar],
            5,
            4,
        ))
    }
}

/// Sprouting Goblin — kicker {G}: a land card with a basic land type to hand
/// on entry; {R}, {T}, sacrifice a land: draw a card.
pub fn sprouting_goblin() -> CardDefinition {
    let basic_type = [LandType::Plains, LandType::Island, LandType::Swamp, LandType::Mountain, LandType::Forest]
        .into_iter()
        .map(R::HasLandType)
        .reduce(R::or)
        .expect("five types");
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasKicked),
            effect: Effect::Search { who: PlayerRef::You, filter: R::Land.and(basic_type), to: ZoneDest::Hand(PlayerRef::You) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Sprouting Goblin",
            cost(&[generic(1), r()]),
            vec![CreatureType::Goblin, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// Szarel, Genesis Shepherd — flying; you may play lands from your
/// graveyard; sacrificing another nontoken permanent on your turn puts
/// counters equal to its power on up to one other target creature.
pub fn szarel_genesis_shepherd() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You may play lands from your graveyard.",
            effect: StaticEffect::MayPlayLandsFromGraveyard,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken.and(R::OtherThanSource) },
                Predicate::IsTurnOf(PlayerRef::You),
            ])),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::OtherThanSource),
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::PowerOf(Box::new(Selector::This)),
                }),
            },
        }],
        ..legendary(creature(
            "Szarel, Genesis Shepherd",
            cost(&[generic(2), b(), r(), g()]),
            vec![CreatureType::Insect, CreatureType::Druid],
            2,
            5,
        ))
    }
}

/// Uurg, Spawn of Turg — power equal to the lands in your graveyard (CR
/// 604.3); upkeep surveil 1; {B}{G}, sacrifice a land: gain 2 life.
pub fn uurg_spawn_of_turg() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Uurg's power is equal to the number of land cards in your graveyard.",
            effect: StaticEffect::SelfBasePtFromValue { power: lands_in_your_graveyard(), toughness: Value::Const(5) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), g()]),
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..legendary(creature(
            "Uurg, Spawn of Turg",
            cost(&[b(), b(), g()]),
            vec![CreatureType::Frog, CreatureType::Beast],
            0,
            5,
        ))
    }
}
