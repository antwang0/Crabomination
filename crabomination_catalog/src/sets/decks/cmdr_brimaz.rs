//! Commander: the cards the **Growing Threat** precon (MOC, Brimaz, Blight
//! of Oreskos) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_brimaz.rs`.
//!
//! Residuals (each also on its card):
//! - **Cataclysmic Gearhulk** — each player keeps their highest-mana-value
//!   permanent of each type rather than choosing.
//! - **Filigree Vector** — the counters go on every creature and artifact you
//!   control rather than on chosen targets.
//! - **Path of the Schemer** — the creature card is the greatest-power one
//!   among all graveyards.
//! - **Vulpine Harvester** — the target may be any artifact card in your
//!   graveyard; the mana-value check happens as the trigger resolves.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    Zone,
};
use crate::effect::shortcut::{encore, etb, on_attack, target_filtered, unearth};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, LookPick, PlayerRef, Predicate, VoteOption, VoteTally, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, generic, w};
use crate::sets::tap_add_colorless;
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

fn artifact_creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..creature(name, mana, types, p, t) }
}

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn phyrexian() -> R {
    R::HasCreatureType(CreatureType::Phyrexian)
}

fn golem_token() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Phyrexian Golem".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Golem],
            ..Default::default()
        },
        ..Default::default()
    })
}

fn create(count: Value, definition: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition }
}

fn basic_land() -> R {
    R::Land.and(R::HasSupertype(Supertype::Basic))
}

/// Brimaz, Blight of Oreskos — casting a Phyrexian creature or artifact
/// creature spell incubates its mana value; at each end step, if a Phyrexian
/// died under your control this turn, proliferate.
pub fn brimaz_blight_of_oreskos() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellMatches(R::Creature.and(phyrexian().or(R::Artifact))),
                ),
                effect: Effect::Incubate {
                    who: PlayerRef::You,
                    amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                    Predicate::CreatureDiedThisTurnMatching { filter: phyrexian().and(R::ControlledByYou) },
                ),
                effect: Effect::Proliferate,
            },
        ],
        ..legendary(creature(
            "Brimaz, Blight of Oreskos",
            cost(&[generic(2), w(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Cat],
            3,
            4,
        ))
    }
}

/// Ancient Stone Idol — flash, trample; costs {1} less per attacking
/// creature; dying leaves a 6/12 Construct with trample.
pub fn ancient_stone_idol() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Trample],
        affinity_filter: Some(R::Creature.and(R::IsAttacking)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: create(
                Value::ONE,
                Arc::new(TokenDefinition {
                    name: "Construct".into(),
                    power: 6,
                    toughness: 12,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
                    keywords: vec![Keyword::Trample],
                    ..Default::default()
                }),
            ),
        }],
        ..artifact_creature("Ancient Stone Idol", cost(&[generic(10)]), vec![CreatureType::Golem], 12, 12)
    }
}

/// Bitterthorn, Nissa's Animus — living weapon; +1/+1; whenever the equipped
/// creature attacks, you may fetch a basic land tapped. Equip {3}.
pub fn bitterthorn_nissas_animus() -> CardDefinition {
    CardDefinition {
        name: "Bitterthorn, Nissa's Animus",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            create(
                Value::ONE,
                Arc::new(TokenDefinition {
                    name: "Phyrexian Germ".into(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Phyrexian],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            ),
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![on_attack(Effect::MayDo {
                description: "Search for a basic land?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: basic_land(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Blight Titan — deathtouch; on entry and on attack, mill two then incubate
/// X, X the creature cards in your graveyard.
pub fn blight_titan() -> CardDefinition {
    let body = || {
        Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(2) },
            Effect::Incubate {
                who: PlayerRef::You,
                amount: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
            },
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(body()), on_attack(body())],
        ..creature(
            "Blight Titan",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Giant],
            6,
            6,
        )
    }
}

/// Cataclysmic Gearhulk — vigilance; on entry each player keeps an artifact,
/// a creature, an enchantment, and a planeswalker among their nonland
/// permanents and sacrifices the rest.
///
/// ⚠ Residual: each player keeps their highest-mana-value permanent of each
/// type.
pub fn cataclysmic_gearhulk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::SacrificeAllButOnePerType {
            who: Selector::Player(PlayerRef::EachPlayer),
            include_land: false,
        })],
        ..artifact_creature(
            "Cataclysmic Gearhulk",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Construct],
            4,
            5,
        )
    }
}

/// Darksteel Splicer — it or another nontoken Phyrexian of yours entering
/// makes a 3/3 Phyrexian Golem per opponent; Golems you control have
/// indestructible.
pub fn darksteel_splicer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: phyrexian().and(R::NotToken) },
            ),
            effect: create(Value::OpponentCount, golem_token()),
        }],
        static_abilities: vec![StaticAbility {
            description: "Golems you control have indestructible.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(R::HasCreatureType(CreatureType::Golem))),
                keyword: Keyword::Indestructible,
            },
        }],
        ..creature(
            "Darksteel Splicer",
            cost(&[generic(6), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Artificer],
            1,
            1,
        )
    }
}

/// Excise the Imperfect — exile target nonland permanent; its controller
/// incubates its mana value.
pub fn excise_the_imperfect() -> CardDefinition {
    CardDefinition {
        name: "Excise the Imperfect",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Permanent.and(R::Nonland)) },
            Effect::Incubate {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Filigree Vector — on entry, +1/+1 counters on creatures and charge
/// counters on artifacts; {1}, {T}, sacrifice another artifact: proliferate.
///
/// ⚠ Residual: every creature and artifact you control gets its counter
/// (not chosen targets).
pub fn filigree_vector() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter {
                what: yours(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::AddCounter { what: yours(R::Artifact), kind: CounterType::Charge, amount: Value::ONE },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::Proliferate,
            ..Default::default()
        }],
        ..artifact_creature(
            "Filigree Vector",
            cost(&[generic(3), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Construct],
            1,
            1,
        )
    }
}

/// First-Sphere Gargantua — on entry, draw a card and lose 1 life; unearth
/// {2}{B}.
pub fn first_sphere_gargantua() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
        ]))],
        activated_abilities: vec![unearth(cost(&[generic(2), b()]))],
        ..creature(
            "First-Sphere Gargantua",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            5,
            4,
        )
    }
}

/// Fractured Powerstone — {T}: add {C}; {T}: roll the planar die, as a
/// sorcery (CR 901.9).
pub fn fractured_powerstone() -> CardDefinition {
    CardDefinition {
        name: "Fractured Powerstone",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::RollPlanarDie { who: PlayerRef::You },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ichor Elixir — your planar dice roll one extra and ignore one; {T}: add
/// {C}{C}.
pub fn ichor_elixir() -> CardDefinition {
    CardDefinition {
        name: "Ichor Elixir",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "If you would roll one or more planar dice, instead roll that many plus one and ignore one.",
            effect: StaticEffect::ExtraPlanarDie,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::Colorless(Value::Const(2)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Keskit, the Flesh Sculptor — {T}, sacrifice three other artifacts and/or
/// creatures: look at the top three, two to hand, one to the graveyard.
/// Partner.
pub fn keskit_the_flesh_sculptor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact.or(R::Creature), 3)),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(3),
                take: Some(Value::Const(2)),
                rest_to_graveyard: true,
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..legendary(creature(
            "Keskit, the Flesh Sculptor",
            cost(&[generic(2), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Artificer],
            1,
            3,
        ))
    }
}

/// Moira and Teshar — flying; casting a historic spell returns a nonland
/// permanent card from your graveyard with haste, exiled at the next end step
/// or if it would leave.
pub fn moira_and_teshar() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    let historic = R::Artifact
        .or(R::HasSupertype(Supertype::Legendary))
        .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(historic)),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::PermanentCard.and(R::Nonland).and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                Effect::ExileIfLeavesBattlefield { what: Selector::Target(0) },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
                },
            ]),
        }],
        ..legendary(creature(
            "Moira and Teshar",
            cost(&[generic(3), w(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Spirit, CreatureType::Bird],
            4,
            5,
        ))
    }
}

/// Path of the Schemer — each player mills two; you put a creature card from
/// a graveyard onto the battlefield as an artifact too; then the will of the
/// planeswalkers: planeswalk, or chaos on a tie.
///
/// ⚠ Residual: the creature card is the greatest-power one among all
/// graveyards.
pub fn path_of_the_schemer() -> CardDefinition {
    CardDefinition {
        name: "Path of the Schemer",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) },
            Effect::Move {
                what: Selector::TakeGreatestPower {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::EachPlayer,
                        zone: Zone::Graveyard,
                        filter: R::Creature,
                    }),
                    count: Box::new(Value::ONE),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCardTypeIndefinitely {
                what: Selector::LastMoved,
                card_type: CardType::Artifact,
                until_eot: false,
            },
            Effect::Vote {
                options: vec![
                    VoteOption::new("planeswalk", Effect::Planeswalk { who: PlayerRef::You }),
                    VoteOption::new("chaos", Effect::ChaosEnsues { who: PlayerRef::You }),
                ],
                tally: VoteTally::Majority,
            },
        ]),
        ..Default::default()
    }
}

/// Phyrexian Triniform — dying leaves three 3/3 Phyrexian Golems; encore
/// {12}.
pub fn phyrexian_triniform() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: create(Value::Const(3), golem_token()),
        }],
        activated_abilities: vec![encore(cost(&[generic(12)]))],
        ..artifact_creature(
            "Phyrexian Triniform",
            cost(&[generic(9)]),
            vec![CreatureType::Phyrexian, CreatureType::Golem],
            9,
            9,
        )
    }
}

/// Vulpine Harvester — whenever one or more Phyrexians you control attack,
/// return target artifact card from your graveyard if its mana value is at
/// most their total power.
///
/// ⚠ Residual: any artifact card may be targeted; the mana-value check runs
/// as the trigger resolves.
pub fn vulpine_harvester() -> CardDefinition {
    let attacking_phyrexians = || yours(R::Creature.and(phyrexian()).and(R::IsAttacking));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_batch: true,
                ..EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: phyrexian() },
                )
            },
            effect: Effect::If {
                cond: Predicate::ValueAtMost(
                    Value::ManaValueOf(Box::new(Selector::Target(0))),
                    Value::PowerOf(Box::new(attacking_phyrexians())),
                ),
                then: Box::new(Effect::Move {
                    what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature(
            "Vulpine Harvester",
            cost(&[generic(3), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Fox],
            3,
            3,
        )
    }
}

/// Yawgmoth's Vile Offering — legendary sorcery: put up to one creature or
/// planeswalker card from a graveyard onto the battlefield under your
/// control, destroy up to one creature or planeswalker, exile it.
pub fn yawgmoths_vile_offering() -> CardDefinition {
    let body = R::Creature.or(R::Planeswalker);
    CardDefinition {
        name: "Yawgmoth's Vile Offering",
        cost: cost(&[generic(4), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Sorcery],
        exile_on_resolve: true,
        effect: Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TargetFiltered { slot: 0, filter: body.clone().and(R::InGraveyard) },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::Destroy { what: Selector::TargetFiltered { slot: 1, filter: body.and(R::Permanent) } },
            ])),
        },
        ..Default::default()
    }
}
