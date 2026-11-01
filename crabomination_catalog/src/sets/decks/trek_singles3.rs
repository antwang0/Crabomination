//! Star Trek (TRK/TRC) — the third batch of EDHREC Star Trek closers: each
//! card here was the last one missing from a single-commander average deck
//! (scan of 2026-11-01). Tests in `tests/recent_b/trek_singles3.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, EnchantmentSubtype, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility, StationBand, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect, ZoneDest, ZoneRef};
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, r, u, w};

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

fn your_creature() -> R {
    R::Creature.and(R::ControlledByYou)
}

/// Syndicate Liquidators — {4}{G} 4/4 Orion Pirate, trample. Landfall: two
/// +1/+1 counters on target creature you control.
pub fn syndicate_liquidators() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::AddCounter {
                what: target_filtered(your_creature()),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..creature(
            "Syndicate Liquidators",
            cost(&[generic(4), g()]),
            vec![CreatureType::Orion, CreatureType::Pirate],
            4,
            4,
        )
    }
}

/// Artificial Anomaly — land. {T}: {C}. {3}{B}, {T}, sacrifice it: each
/// opponent loses 2 life, you gain 2, and a tapped New Planet land token
/// ("{T}: one mana of any color") enters.
pub fn artificial_anomaly() -> CardDefinition {
    let planet = Arc::new(TokenDefinition {
        name: "New Planet".into(),
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Planet], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        tapped: true,
        ..Default::default()
    });
    CardDefinition {
        name: "Artificial Anomaly",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(3), b()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Drain {
                        from: Selector::Player(PlayerRef::EachOpponent),
                        to: Selector::You,
                        amount: Value::Const(2),
                    },
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: planet },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Planet Killer — {8} 9/9 Construct artifact creature, flying,
/// indestructible, attacks each combat. Combat damage to a player: a +1/+1
/// counter per land that player controls.
pub fn planet_killer() -> CardDefinition {
    let mut c = creature("Planet Killer", cost(&[generic(8)]), vec![CreatureType::Construct], 9, 9);
    c.card_types.insert(0, CardType::Artifact);
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Indestructible, Keyword::MustAttack],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::PermanentCountControlledByMatching(PlayerRef::Triggerer, R::Land),
            },
        }],
        ..c
    }
}

/// Lwaxana Troi, Charming Guest — {1}{R} legendary 2/2 Betazoid Advisor. You
/// may choose not to untap her. {2}, {T}: for as long as she stays tapped,
/// another target creature you control's triggered abilities trigger an
/// additional time.
pub fn lwaxana_troi_charming_guest() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::DoubleTriggersWhileSourceTapped {
                what: target_filtered(your_creature().and(R::OtherThanSource)),
            },
            ..Default::default()
        }],
        ..legend(
            "Lwaxana Troi, Charming Guest",
            cost(&[generic(1), r()]),
            vec![CreatureType::Betazoid, CreatureType::Advisor],
            2,
            2,
        )
    }
}

/// Kira Nerys, Firebrand — {1}{R}{W} legendary 4/3 Bajoran Soldier, trample.
/// While you control an attacking creature, your creatures enter tapped and
/// attacking; another creature of yours entering gets +1/+1 until end of turn.
pub fn kira_nerys_firebrand() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "If a creature you control would enter while you control an attacking creature, instead it \
                          enters tapped and attacking.",
            effect: StaticEffect::YourCreaturesEnterAttacking,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend(
            "Kira Nerys, Firebrand",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Bajoran, CreatureType::Soldier],
            4,
            3,
        )
    }
}

/// The tapped "New Planet" Planet land token ({T}: one mana of any color).
fn new_planet() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "New Planet".into(),
            card_types: vec![CardType::Land],
            subtypes: Subtypes { land_types: vec![LandType::Planet], ..Default::default() },
            activated_abilities: vec![crate::sets::tap_add_any_color()],
            tapped: true,
            ..Default::default()
        }),
    }
}

/// David and Carol Marcus, Creators — {2}{G} legendary 1/3 Human Scientist.
/// Entering makes a tapped New Planet.
pub fn david_and_carol_marcus_creators() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(new_planet())],
        ..legend(
            "David and Carol Marcus, Creators",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            1,
            3,
        )
    }
}

/// Stable Wormhole — {2}{G} enchantment. Whenever you attack, a tapped New
/// Planet.
pub fn stable_wormhole() -> CardDefinition {
    CardDefinition {
        name: "Stable Wormhole",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: new_planet(),
        }],
        ..Default::default()
    }
}

/// La Forge, Perceptive Engineer — {1}{U} legendary 1/4 Human Artificer,
/// vigilance. {T}: {U}, spent only on an artifact spell.
pub fn la_forge_perceptive_engineer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Blue])),
                    SpendRestriction::ArtifactSpellsOnly,
                ),
            },
            ..Default::default()
        }],
        ..legend(
            "La Forge, Perceptive Engineer",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            4,
        )
    }
}

/// Urza's Tome — {2} Book artifact. {3}, {T}: draw, then discard unless you
/// exile a historic card from your graveyard.
pub fn urzas_tome() -> CardDefinition {
    let discard = || Effect::Discard { who: Selector::You, amount: Value::ONE, random: false };
    let historic_in_gy = || Selector::EachMatching { zone: ZoneRef::Graveyard(PlayerRef::You), filter: R::historic() };
    CardDefinition {
        name: "Urza's Tome",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Book], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::SelectorCountAtLeast { sel: historic_in_gy(), n: Value::ONE },
                    then: Box::new(Effect::MayDoElse {
                        description: "Exile a historic card from your graveyard instead of discarding?".into(),
                        body: Box::new(Effect::Move { what: Selector::one_of(historic_in_gy()), to: ZoneDest::Exile }),
                        else_: Box::new(discard()),
                    }),
                    else_: Box::new(discard()),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Test of Conviction — {1}{W} sorcery. Choose one: a +1/+1 counter on each
/// creature you control; or target opponent draws a card and you draw two.
pub fn test_of_conviction() -> CardDefinition {
    CardDefinition {
        name: "Test of Conviction",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::AddCounter {
                what: Selector::EachPermanent(your_creature()),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::Seq(vec![
                Effect::Draw { who: target_filtered(R::Player.and(R::ControlledByOpponent)), amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ]),
        ]),
        ..Default::default()
    }
}

/// Troi, Winging It — {3}{W} legendary 3/2 Betazoid Human Advisor. An
/// opponent drawing a card during your turn makes a 1/1 blue flying Faerie.
pub fn troi_winging_it() -> CardDefinition {
    let faerie = Arc::new(TokenDefinition {
        name: "Faerie".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: faerie },
        }],
        ..legend(
            "Troi, Winging It",
            cost(&[generic(3), w()]),
            vec![CreatureType::Betazoid, CreatureType::Human, CreatureType::Advisor],
            3,
            2,
        )
    }
}

/// Beckett Mariner, Impetuous Ensign — {1}{R} legendary 2/1 Human Officer,
/// first strike. Becoming tapped adds a promotion counter; at three or more
/// she removes them and deals 2 damage to each opponent.
pub fn beckett_mariner_impetuous_ensign() -> CardDefinition {
    let promotions = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Promotion };
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Promotion, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(promotions(), Value::Const(3)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter { what: Selector::This, kind: CounterType::Promotion, amount: promotions() },
                        Effect::DealDamageFrom {
                            source: Selector::This,
                            to: Selector::Player(PlayerRef::EachOpponent),
                            amount: Value::Const(2),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legend(
            "Beckett Mariner, Impetuous Ensign",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Officer],
            2,
            1,
        )
    }
}

/// Sarcophagus, Ship of the Dead — {2}{R} legendary Spacecraft. Your other
/// creatures get +1/+0. A creature dying while it isn't a creature adds a
/// charge counter. Station; 10+: 4/4 flying.
pub fn sarcophagus_ship_of_the_dead() -> CardDefinition {
    CardDefinition {
        name: "Sarcophagus, Ship of the Dead",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(your_creature().and(R::OtherThanSource)),
                power: 1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::Creature.negate() }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
        }],
        activated_abilities: vec![crate::effect::shortcut::station()],
        station: vec![StationBand { min: 10, keywords: vec![Keyword::Flying], pt: Some((4, 4)), ..Default::default() }],
        ..Default::default()
    }
}

/// Amanda Grayson, Logic's Link — {3}{G} legendary 2/3 Human Advisor,
/// affinity for Vulcans. Landfall: two +1/+1 counters on target creature you
/// control.
pub fn amanda_grayson_logics_link() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Affinity for Vulcans.",
            effect: StaticEffect::SelfCostReducedPerPermanentMatching {
                filter: R::HasCreatureType(CreatureType::Vulcan).and(R::ControlledByYou),
                per: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::AddCounter {
                what: target_filtered(your_creature()),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..legend(
            "Amanda Grayson, Logic's Link",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            2,
            3,
        )
    }
}

/// U.S.S. Voyager, Lone Wanderer — {3}{G} legendary Spacecraft. Entering
/// reveals until two lands, which enter tapped (the rest to the bottom).
/// Station; 8+: 3/3 flying, vigilance.
pub fn u_s_s_voyager_lone_wanderer() -> CardDefinition {
    CardDefinition {
        name: "U.S.S. Voyager, Lone Wanderer",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::RevealUntilLandsToBattlefield {
            count: Value::Const(2),
            tapped: true,
        })],
        activated_abilities: vec![crate::effect::shortcut::station()],
        station: vec![StationBand {
            min: 8,
            keywords: vec![Keyword::Flying, Keyword::Vigilance],
            pt: Some((3, 3)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// "Exile up to one other target permanent you control matching `filter`;
/// return it under its owner's control at the beginning of the next end step."
fn blink_until_end_step(filter: R) -> Effect {
    Effect::ApplyToTargets {
        max_targets: 1,
        min_targets: 0,
        filter: filter.and(R::ControlledByYou).and(R::OtherThanSource),
        effect: Box::new(Effect::ExileReturnToOwnerNextEndStep { what: Selector::Target(0), tapped: false }),
    }
}

/// Chief Kyle, Transporter Tech — {2}{W}{U} legendary 3/2 Human Artificer.
/// Landfall: blink up to one other creature you control until the next end
/// step.
pub fn chief_kyle_transporter_tech() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: blink_until_end_step(R::Creature),
        }],
        ..legend(
            "Chief Kyle, Transporter Tech",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            2,
        )
    }
}

/// Balance of Terror — {2}{W} Saga. I: destroy up to one other target
/// artifact or enchantment. II, III: blink up to one other permanent you
/// control until the next end step.
pub fn balance_of_terror() -> CardDefinition {
    CardDefinition {
        name: "Balance of Terror",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Artifact.or(R::Enchantment).and(R::OtherThanSource),
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                },
            ),
            (2, blink_until_end_step(R::Any)),
            (3, blink_until_end_step(R::Any)),
        ],
        ..Default::default()
    }
}

/// Jefferies Tube — {1} artifact. {2},{T}: two charge counters on target
/// artifact. {3},{T}: target creature you control can't be blocked this turn.
/// {4},{T}: draw, then you may put a land from your hand onto the battlefield
/// tapped.
pub fn jefferies_tube() -> CardDefinition {
    let tap = |n: u32, effect: Effect| ActivatedAbility {
        mana_cost: cost(&[generic(n)]),
        tap_cost: true,
        effect,
        ..Default::default()
    };
    CardDefinition {
        name: "Jefferies Tube",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            tap(
                2,
                Effect::AddCounter { what: target_filtered(R::Artifact), kind: CounterType::Charge, amount: Value::Const(2) },
            ),
            tap(
                3,
                Effect::GrantKeyword {
                    what: target_filtered(your_creature()),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ),
            tap(
                4,
                Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::MayDo {
                        description: "Put a land card from your hand onto the battlefield tapped?".into(),
                        body: Box::new(Effect::PutFromHandOntoBattlefield {
                            who: PlayerRef::You,
                            filter: R::Land,
                            count: Value::ONE,
                            tapped: true,
                            haste: false,
                            sacrifice_eot: false,
                            return_eot: false,
                            then: None,
                        }),
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Isolinear Processor — {2} artifact. {T}: {C} for artifact spells and
/// artifact abilities. {2}, {T}: draw, then discard.
pub fn isolinear_processor() -> CardDefinition {
    CardDefinition {
        name: "Isolinear Processor",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colorless(Value::ONE)),
                        SpendRestriction::ArtifactSpellsOrAbilities,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Faerie Swarm — {3}{U} */* Faerie, flying; P/T = blue permanents you
/// control.
pub fn faerie_swarm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(crate::card::DynamicPt::PermanentsControlledMatching {
            base_p: 0,
            base_t: 0,
            filter: Box::new(R::HasColor(Color::Blue)),
        }),
        ..creature("Faerie Swarm", cost(&[generic(3), u()]), vec![CreatureType::Faerie], 0, 0)
    }
}

/// T'Pring, Never Parted — {3}{G}{U} legendary 3/3 Vulcan Citizen. Entering
/// targets another creature you control and a creature an opponent controls;
/// you may have them fight, and each that fought draws you a card if it dies
/// this turn.
pub fn tpring_never_parted() -> CardDefinition {
    let draw_if_dies = |slot| Effect::WhenTargetDiesThisTurn {
        body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        slot,
        filter: None,
    };
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::MayDo {
            description: "Have the chosen creatures fight each other?".into(),
            // The watchers go first: a fight's lethal damage can leave the
            // creature gone before a later step could look it up.
            body: Box::new(Effect::Seq(vec![
                draw_if_dies(0),
                draw_if_dies(1),
                Effect::Fight {
                    attacker: Selector::TargetFiltered { slot: 0, filter: your_creature().and(R::OtherThanSource) },
                    defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                },
            ])),
        })],
        ..legend(
            "T'Pring, Never Parted",
            cost(&[generic(3), g(), u()]),
            vec![CreatureType::Vulcan, CreatureType::Citizen],
            3,
            3,
        )
    }
}

/// Federation Resupply — {1}{W} sorcery. Search for an Aura, Equipment, or
/// Spacecraft card to hand. Beam me up {3}{W}: flashback that also returns a
/// creature you control to its owner's hand.
pub fn federation_resupply() -> CardDefinition {
    CardDefinition {
        name: "Federation Resupply",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), w()]))],
        flashback_additional_cost: vec![crate::card::AdditionalCastCost::ReturnToHand {
            filter: your_creature(),
            count: 1,
            count_x: false,
        }],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment))
                .or(R::HasArtifactSubtype(ArtifactSubtype::Spacecraft)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Dominion Saboteur — {4}{U} 3/3 Shapeshifter. It may enter as a copy of
/// any artifact or creature, except not legendary and with the copied
/// permanent's counters.
pub fn dominion_saboteur() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(crate::card::EntersAsCopy {
            filter: R::Artifact.or(R::Creature),
            non_legendary: true,
            mirror_counters: true,
            ..Default::default()
        }),
        ..creature("Dominion Saboteur", cost(&[generic(4), u()]), vec![CreatureType::Shapeshifter], 3, 3)
    }
}

/// The Guardian of Forever — {3} legendary 0/6 Construct artifact creature,
/// defender. {4}, sacrifice another nontoken creature: that card returns
/// under its owner's control at the beginning of the next end step.
pub fn the_guardian_of_forever() -> CardDefinition {
    let mut c = legend("The Guardian of Forever", cost(&[generic(3)]), vec![CreatureType::Construct], 0, 6);
    c.card_types.insert(0, CardType::Artifact);
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            sac_other_filter: Some((R::Creature.and(R::NotToken), 1)),
            effect: Effect::DelayUntilWithCapture {
                kind: crate::effect::DelayedTriggerKind::NextEndStep,
                capture: Selector::SacrificedCard,
                body: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        tapped: false,
                    },
                }),
            },
            ..Default::default()
        }],
        ..c
    }
}

/// Highly Illogical — {1}{U} instant, kicker {1}{U}. Counter target spell
/// with mana value 2 or less; kicked, any target spell.
pub fn highly_illogical() -> CardDefinition {
    CardDefinition {
        name: "Highly Illogical",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[generic(1), u()]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) }),
            else_: Box::new(Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::ManaValueAtMost(2))),
            }),
        },
        ..Default::default()
    }
}

/// Deep Space Scan — {4}{U}{U} sorcery. Strange new worlds — draw a card per
/// differently named land you control; no maximum hand size until your next
/// turn (restored to seven at your next upkeep).
pub fn deep_space_scan() -> CardDefinition {
    CardDefinition {
        name: "Deep Space Scan",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::DistinctNamesControlledMatching(R::Land) },
            Effect::SetNoMaxHandSize { who: Selector::You },
            Effect::DelayUntil {
                kind: crate::effect::DelayedTriggerKind::YourNextUpkeep,
                body: Box::new(Effect::SetMaxHandSize { who: Selector::You, size: Value::Const(7) }),
            },
        ]),
        ..Default::default()
    }
}
