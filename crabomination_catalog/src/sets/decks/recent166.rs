//! Edge of Eternities (EOE) and Avatar: The Last Airbender (TLA) staples that
//! were deferred for want of a primitive. The new engine piece here is the
//! impulse-until-nonland family (`Effect::ExileTopUntilNonlandMayPlay`) —
//! Territorial Bruntar's landfall and Solstice Revelations. Tests in
//! `crabomination/src/tests/recent166.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, MayPlayDuration, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, SpendRestriction, b, cost, g, generic, r, u, w};

/// A 2/2 red Soldier token with firebending 1 (Firebender Ascension).
fn firebending_soldier() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        keywords: vec![Keyword::Firebending(1)],
        ..Default::default()
    }
}

/// A 1/1 white Ally creature token.
fn ally_token() -> TokenDefinition {
    TokenDefinition {
        name: "Ally".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ally],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Territorial Bruntar — {4}{R}{R} 6/6 Reach. Landfall: exile from the top of
/// your library until you exile a nonland card; you may cast it this turn.
pub fn territorial_bruntar() -> CardDefinition {
    CardDefinition {
        name: "Territorial Bruntar",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::ExileTopUntilNonlandMayPlay {
                who: PlayerRef::You,
                duration: MayPlayDuration::EndOfThisTurn,
                free: false,
                hand_unless_mv_below: None,
                grant_to_exiling_player: false,
            },
        }],
        ..Default::default()
    }
}

/// Solstice Revelations — {2}{R} Instant — Lesson. Exile from the top of your
/// library until you exile a nonland card; you may cast it without paying its
/// mana cost if its mana value is less than the number of Mountains you control,
/// otherwise put it into your hand. Flashback {6}{R}.
pub fn solstice_revelations() -> CardDefinition {
    CardDefinition {
        name: "Solstice Revelations",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        keywords: vec![Keyword::Flashback(cost(&[generic(6), r()]))],
        effect: Effect::ExileTopUntilNonlandMayPlay {
            who: PlayerRef::You,
            duration: MayPlayDuration::EndOfThisTurn,
            free: true,
            hand_unless_mv_below: Some(Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(R::ControlledByYou)),
                filter: R::HasLandType(LandType::Mountain),
            }),
            grant_to_exiling_player: false,
        },
        ..Default::default()
    }
}

/// White Lotus Hideout — Land. `{T}: Add {C}.` `{T}: Add one mana of any color.
/// Spend only to cast a Lesson spell.` (The Shrine half is dropped.) `{1}, {T}:
/// Add one mana of any color.`
pub fn white_lotus_hideout() -> CardDefinition {
    CardDefinition {
        name: "White Lotus Hideout",
        card_types: vec![CardType::Land],
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
                        SpendRestriction::LessonSpellsOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Jasmine Dragon Tea Shop — Land. `{T}: Add {C}.` `{T}: Add one mana of any
/// color. Spend only to cast an Ally spell or activate an Ally's ability.`
/// (Approximated as "cast an Ally creature spell".) `{5}, {T}: Create a 1/1
/// white Ally creature token.`
pub fn jasmine_dragon_tea_shop() -> CardDefinition {
    CardDefinition {
        name: "Jasmine Dragon Tea Shop",
        card_types: vec![CardType::Land],
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
                        SpendRestriction::CreatureOfType(CreatureType::Ally),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(5)]),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(ally_token()),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Secret Tunnel — Land — Cave. `{T}: Add {C}.` `{4}, {T}: Target creature you
/// control can't be blocked this turn.` (The printed "two creatures that share a
/// type" is collapsed to a single target; the flavorful "this land can't be
/// blocked" is dropped — lands don't attack.)
pub fn secret_tunnel() -> CardDefinition {
    CardDefinition {
        name: "Secret Tunnel",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Cave],
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
                mana_cost: cost(&[generic(4)]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Planetarium of Wan Shi Tong — {6} Legendary Artifact. `{1}, {T}: Scry 2.`
/// Whenever you scry or surveil, exile the top card of your library; you may
/// cast it without paying its mana cost. Do this only once each turn. (Modeled
/// as an impulse-exile rather than a look-and-leave-on-top.)
pub fn planetarium_of_wan_shi_tong() -> CardDefinition {
    CardDefinition {
        name: "Planetarium of Wan Shi Tong",
        cost: cost(&[generic(6)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ScriedOrSurveiled, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: false,
                uncast_penalty: None,
            },
        }],
        ..Default::default()
    }
}

/// Phoenix Fleet Airship — {2}{B}{B} Artifact — Vehicle 4/4. Flying, Crew 1. At
/// the beginning of your end step, if you sacrificed a permanent this turn,
/// create a token that's a copy of this Vehicle. (The "8+ copies → creature"
/// clause is dropped.)
pub fn phoenix_fleet_airship() -> CardDefinition {
    CardDefinition {
        name: "Phoenix Fleet Airship",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::PermanentsSacrificedThisTurnAtLeast {
                who: PlayerRef::You,
                at_least: Value::ONE,
            }),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Firebender Ascension — {1}{R} Enchantment. When it enters, create a 2/2 red
/// Soldier creature token with firebending 1. (The quest-counter copy ability is
/// dropped — the ETB body is the played piece.)
pub fn firebender_ascension() -> CardDefinition {
    CardDefinition {
        name: "Firebender Ascension",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(firebending_soldier()),
        })],
        ..Default::default()
    }
}

/// Ragost, Deft Gastronaut — {R}{W} 2/2 legendary Lobster Citizen. `{1}, {T},
/// Sacrifice a Food: Ragost deals 3 damage to each opponent.` At the beginning
/// of each end step, if you gained life this turn, untap Ragost. (The "artifacts
/// you control are Foods with a sac-for-life ability" static is dropped.)
pub fn ragost_deft_gastronaut() -> CardDefinition {
    CardDefinition {
        name: "Ragost, Deft Gastronaut",
        cost: cost(&[r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Food), 1)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::ONE,
                }),
            effect: Effect::Untap {
                what: Selector::This,
                up_to: None,
            },
        }],
        ..Default::default()
    }
}

/// Invasion Submersible — {2}{U} Artifact — Vehicle 0/0. When it enters, return
/// up to one other target nonland permanent to its owner's hand. Exhaust — {3}:
/// This Vehicle becomes an artifact creature; put three +1/+1 counters on it.
/// (Waterbend {3} is approximated as a plain {3} cost.)
pub fn invasion_submersible() -> CardDefinition {
    CardDefinition {
        name: "Invasion Submersible",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Nonland.and(R::OtherThanSource),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCardTypeIndefinitely {
                    what: Selector::This,
                    card_type: CardType::Creature,
                    until_eot: false,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(3),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Aetherdrift (DFT) — Mounts, Speed, exhaust, graveyard value ─────────────

/// Gloryheath Lynx — {1}{W} 2/3 Cat Mount. Lifelink; Saddle 2. Whenever it
/// attacks while saddled, search your library for a basic Plains and put it into
/// your hand.
pub fn gloryheath_lynx() -> CardDefinition {
    CardDefinition {
        name: "Gloryheath Lynx",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Mount],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink, Keyword::Saddle(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::SourceSaddled),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand.and(R::HasLandType(LandType::Plains)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Guardian Sunmare — {3}{W}{W} 5/5 Horse Mount. Ward {2}; Saddle 4. Whenever it
/// attacks while saddled, search your library for a nonland permanent card with
/// mana value 3 or less and put it onto the battlefield.
pub fn guardian_sunmare() -> CardDefinition {
    CardDefinition {
        name: "Guardian Sunmare",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horse, CreatureType::Mount],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![
            Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))),
            Keyword::Saddle(4),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::SourceSaddled),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Nonland.and(R::ManaValueAtMost(3)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        }],
        ..Default::default()
    }
}

/// Guidelight Optimizer — {1}{U} 2/1 Artifact Creature — Robot. `{T}: Add {U}.
/// Spend only to cast an artifact spell or activate an ability.`
pub fn guidelight_optimizer() -> CardDefinition {
    CardDefinition {
        name: "Guidelight Optimizer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::OfColor(Color::Blue, Value::ONE)),
                    SpendRestriction::ArtifactOnly,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Grim Bauble — {B} Artifact. When it enters, target creature an opponent
/// controls gets -2/-2 until end of turn. `{2}{B}, {T}, Sacrifice: Surveil 2.`
pub fn grim_bauble() -> CardDefinition {
    CardDefinition {
        name: "Grim Bauble",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), b()]),
            sac_cost: true,
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gastal Raider — {2}{B} 2/1 Vampire Rogue. Start your engines! When it enters,
/// target opponent reveals their hand; you choose an instant or sorcery card
/// from it and that player discards it. Max speed — it gets +1/+1 and has menace.
pub fn gastal_raider() -> CardDefinition {
    CardDefinition {
        name: "Gastal Raider",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![etb(Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::Target(0)),
            count: Value::ONE,
            filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
        })],
        static_abilities: vec![StaticAbility {
            description: "Max speed — gets +1/+1 and has menace.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpeedAtLeast {
                    who: PlayerRef::You,
                    speed: 4,
                },
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Menace],
            },
        }],
        ..Default::default()
    }
}

/// Basri, Tomorrow's Champion — {W} 2/1 Human Knight. `{W}, {T}: Create a 1/1
/// white Cat creature token with lifelink.` (Exert is approximated as a plain
/// tap.) Cycling {2}{W}; when you cycle it, Cats you control gain hexproof and
/// indestructible until end of turn.
pub fn basri_tomorrows_champion() -> CardDefinition {
    let cat = || TokenDefinition {
        name: "Cat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    };
    CardDefinition {
        name: "Basri, Tomorrow's Champion",
        cost: cost(&[w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Cycling(cost(&[generic(2), w()]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(cat()),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Cat).and(R::ControlledByYou),
                    ),
                    keyword: Keyword::Hexproof,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Cat).and(R::ControlledByYou),
                    ),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Grim Javelineer — {2}{B} 3/2 Human Warrior. Whenever you attack, target
/// attacking creature gets +1/+0 until end of turn, then surveil 1. (The printed
/// surveil is gated on that creature dying; here it's unconditional.)
pub fn grim_javelineer() -> CardDefinition {
    CardDefinition {
        name: "Grim Javelineer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::IsAttacking)),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Broodheart Engine — {B}{G} Artifact. At the beginning of your upkeep,
/// surveil 1. `{2}{B}{G}, {T}, Sacrifice: Return target creature or Vehicle card
/// from your graveyard to the battlefield. Sorcery speed.`
pub fn broodheart_engine() -> CardDefinition {
    CardDefinition {
        name: "Broodheart Engine",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), b(), g()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Move {
                what: target_filtered(
                    (R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)))
                        .and(R::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Amonkhet Raceway — Land. Start your engines! `{T}: Add {C}.` Max speed —
/// `{T}: Target creature gains haste until end of turn.`
pub fn amonkhet_raceway() -> CardDefinition {
    CardDefinition {
        name: "Amonkhet Raceway",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::StartYourEngines],
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
                condition: Some(Predicate::SpeedAtLeast {
                    who: PlayerRef::You,
                    speed: 4,
                }),
                effect: Effect::GrantKeyword {
                    what: target_any(),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Fang-Druid Summoner — {3}{G} 2/4 Ape Druid. Reach. When it enters, search
/// your library for a creature card and put it into your hand. (The printed "no
/// abilities" restriction and the graveyard search half are dropped.)
pub fn fang_druid_summoner() -> CardDefinition {
    CardDefinition {
        name: "Fang-Druid Summoner",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ape, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::Creature,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

// ── Aetherdrift (DFT) legends ───────────────────────────────────────────────

/// A 1/1 green Insect creature token (Aatchik).
fn insect_token() -> TokenDefinition {
    TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Fearless Swashbuckler — {1}{U}{R} 3/3 Fish Pirate. Haste; Vehicles you control
/// have haste. Whenever you attack, draw three cards, then discard two. (The
/// printed "if a Pirate and a Vehicle attacked" gate is dropped.)
pub fn fearless_swashbuckler() -> CardDefinition {
    CardDefinition {
        name: "Fearless Swashbuckler",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish, CreatureType::Pirate],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Vehicles you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasArtifactSubtype(ArtifactSubtype::Vehicle).and(R::ControlledByYou),
                ),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(2),
                    random: false,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Caradora, Heart of Alacria — {2}{G}{W} 4/2 legendary Human Knight. ETB search
/// your library for a Mount or Vehicle card to hand. If one or more +1/+1
/// counters would be put on a creature you control, that many plus one are put
/// instead. (The Vehicle half of the counter rider is dropped.)
pub fn caradora_heart_of_alacria() -> CardDefinition {
    CardDefinition {
        name: "Caradora, Heart of Alacria",
        cost: cost(&[generic(2), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Extra +1/+1 counter whenever counters are placed on your creatures.",
            effect: StaticEffect::ExtraPlusOneCounters,
        }],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::HasCreatureType(CreatureType::Mount)
                .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Far Fortune, End Boss — {2}{B}{R} 4/5 legendary Human Mercenary. Start your
/// engines! Whenever you attack, Far Fortune deals 1 damage to each opponent.
/// (The "max speed — deal +1 damage" replacement is dropped.)
pub fn far_fortune_end_boss() -> CardDefinition {
    CardDefinition {
        name: "Far Fortune, End Boss",
        cost: cost(&[generic(2), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Hazoret, Godseeker — {1}{R} 5/3 legendary God. Indestructible, haste; Start
/// your engines! `{1}, {T}: Target creature with power 2 or less can't be
/// blocked this turn.` (The "can't attack unless max speed" restriction is
/// dropped.)
pub fn hazoret_godseeker() -> CardDefinition {
    CardDefinition {
        name: "Hazoret, Godseeker",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::God],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        keywords: vec![
            Keyword::Indestructible,
            Keyword::Haste,
            Keyword::StartYourEngines,
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Aatchik, Emerald Radian — {3}{B}{B}{G} 3/3 legendary Insect Druid. ETB create
/// a 1/1 green Insect for each artifact and/or creature card in your graveyard.
/// Whenever another Insect you control dies, put a +1/+1 counter on Aatchik and
/// each opponent loses 1 life.
pub fn aatchik_emerald_radian() -> CardDefinition {
    CardDefinition {
        name: "Aatchik, Emerald Radian",
        cost: cost(&[generic(3), b(), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::HasCardType(CardType::Creature)
                        .or(R::HasCardType(CardType::Artifact)),
                }),
                definition: Box::new(insect_token()),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Insect).and(R::OtherThanSource),
                    }),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::ONE,
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

// ── DFT Vehicles + a land ───────────────────────────────────────────────────

/// A 1/1 colorless Pilot creature token (Country Roads).
fn pilot_token() -> TokenDefinition {
    TokenDefinition {
        name: "Pilot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pilot],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Gastal Thrillroller — {2}{R} Artifact — Vehicle 4/2. Trample, haste; Crew 2.
/// When it enters, it becomes an artifact creature until end of turn. `{2}{R},
/// Discard a card: Return this from your graveyard to the battlefield with a
/// finality counter on it. Sorcery speed.`
pub fn gastal_thrillroller() -> CardDefinition {
    CardDefinition {
        name: "Gastal Thrillroller",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::Crew(2)],
        triggered_abilities: vec![etb(Effect::BecomeCreature {
            what: Selector::This,
            power: Value::Const(4),
            toughness: Value::Const(2),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            from_graveyard: true,
            sorcery_speed: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Finality,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Apocalypse Runner — {2}{B}{R} Artifact — Vehicle 6/5. Crew 3. `{T}: Target
/// creature you control with power 2 or less gains lifelink until end of turn
/// and can't be blocked this turn.`
pub fn apocalypse_runner() -> CardDefinition {
    CardDefinition {
        name: "Apocalypse Runner",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Crew(3)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(
                        R::Creature.and(R::ControlledByYou).and(R::PowerAtMost(2)),
                    ),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wingshield Agent — {2}{U} 2/3 Human Soldier. Enters with a shield counter.
/// Whenever it attacks, up to one other target creature gains flying until end
/// of turn.
pub fn wingshield_agent() -> CardDefinition {
    CardDefinition {
        name: "Wingshield Agent",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        enters_with_counters: Some((CounterType::Shield, Value::ONE)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::OtherThanSource),
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Guidelight Pathmaker — {4}{W}{U} Artifact — Vehicle 6/5. Vigilance; Crew 2.
/// When it enters, search your library for an artifact card and put it into your
/// hand. (The printed "put onto the battlefield if MV 2 or less" is collapsed to
/// always-to-hand.)
pub fn guidelight_pathmaker() -> CardDefinition {
    CardDefinition {
        name: "Guidelight Pathmaker",
        cost: cost(&[generic(4), w(), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Vigilance, Keyword::Crew(2)],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::Artifact,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Country Roads — Land. `{T}: Add {W}.` `{1}{W}, {T}, Sacrifice this land:
/// Create a 1/1 colorless Pilot creature token. Sorcery speed.` (The "enters
/// tapped unless you control a Mount or Vehicle" clause is dropped.)
pub fn country_roads() -> CardDefinition {
    CardDefinition {
        name: "Country Roads",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::White, Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1), w()]),
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(pilot_token()),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── DFT / EOE closers ───────────────────────────────────────────────────────

/// Voyager Glidecar — {W} Artifact — Vehicle 2/3. Crew 1. When it enters, scry 1.
/// `Tap three other untapped creatures you control: Until end of turn, this
/// becomes an artifact creature with flying; put a +1/+1 counter on it.`
pub fn voyager_glidecar() -> CardDefinition {
    CardDefinition {
        name: "Voyager Glidecar",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((
                R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                3,
            )),
            effect: Effect::Seq(vec![
                Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(3),
                    creature_types: vec![],
                    keywords: vec![Keyword::Flying],
                    duration: Duration::EndOfTurn,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kickoff Celebrations — {1}{R} Enchantment. Start your engines! When it enters,
/// you may discard a card; if you do, draw two. (The "max speed — sacrifice:
/// team gains haste" ability is dropped.)
pub fn kickoff_celebrations() -> CardDefinition {
    CardDefinition {
        name: "Kickoff Celebrations",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![etb(Effect::MayDiscard {
            description: "You may discard a card. If you do, draw two cards.".into(),
            count: Value::ONE,
            then: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}
