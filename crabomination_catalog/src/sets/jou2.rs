//! Journey into Nyx (JOU) wave 2 — the Strive cycle, the constellation
//! enchantments, and the rest of the common/uncommon core. Tests in
//! `classic_sets/jou`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EquipBonus, Keyword, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, heroic, monstrosity, on_becomes_monstrous, target_filtered};
use crate::effect::{
    AttackingTokenCleanup, Duration, Effect, EventKind, EventScope, EventSpec, ExtraManaKind,
    PlayerRef, Predicate, Selector, ZoneDest, ZoneRef,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: ct,
            ..Default::default()
        },
        power: p,
        toughness: t,
        keywords: kw,
        ..Default::default()
    }
}

/// An "enchantment creature" body (the Nyx-touched JOU commons/uncommons).
fn nyx_creature(
    name: &'static str,
    mana: ManaCost,
    p: i32,
    t: i32,
    ct: Vec<CreatureType>,
    kw: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        ..creature(name, mana, p, t, ct, kw)
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

// ── Strive (CR 702.122) ──────────────────────────────────────────────────────

/// A Strive spell: "any number of target `filter`" whose per-extra-target
/// surcharge is `per` (`CardDefinition.cost_per_extra_target`).
fn strive(
    name: &'static str,
    mana: ManaCost,
    kind: CardType,
    per: ManaCost,
    filter: R,
    body: Effect,
) -> CardDefinition {
    CardDefinition {
        cost_per_extra_target: Some(per),
        ..spell(
            name,
            mana,
            kind,
            Effect::ApplyToTargets {
                max_targets: 10,
                min_targets: 0,
                filter,
                effect: Box::new(body),
            },
        )
    }
}

/// The most common Strive body: pump each target and hand it keywords.
fn strive_pump(
    name: &'static str,
    mana: ManaCost,
    kind: CardType,
    per: ManaCost,
    power: i32,
    toughness: i32,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    let mut body = vec![];
    if (power, toughness) != (0, 0) {
        body.push(Effect::PumpPT {
            what: Selector::Target(0),
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        });
    }
    if !keywords.is_empty() {
        body.push(Effect::GrantKeywords {
            what: Selector::Target(0),
            keywords,
            duration: Duration::EndOfTurn,
        });
    }
    strive(name, mana, kind, per, R::Creature, Effect::Seq(body))
}

/// Aerial Formation — {U} Instant. Strive {2}{U}. +1/+1 and flying.
pub fn aerial_formation() -> CardDefinition {
    strive_pump(
        "Aerial Formation",
        cost(&[u()]),
        CardType::Instant,
        cost(&[generic(2), u()]),
        1,
        1,
        vec![Keyword::Flying],
    )
}

/// Ajani's Presence — {W} Instant. Strive {2}{W}. +1/+1 and indestructible.
pub fn ajanis_presence() -> CardDefinition {
    strive_pump(
        "Ajani's Presence",
        cost(&[w()]),
        CardType::Instant,
        cost(&[generic(2), w()]),
        1,
        1,
        vec![Keyword::Indestructible],
    )
}

/// Cruel Feeding — {B} Instant. Strive {2}{B}. +1/+0 and lifelink.
pub fn cruel_feeding() -> CardDefinition {
    strive_pump(
        "Cruel Feeding",
        cost(&[b()]),
        CardType::Instant,
        cost(&[generic(2), b()]),
        1,
        0,
        vec![Keyword::Lifelink],
    )
}

/// Rouse the Mob — {R} Instant. Strive {2}{R}. +2/+0 and trample.
pub fn rouse_the_mob() -> CardDefinition {
    strive_pump(
        "Rouse the Mob",
        cost(&[r()]),
        CardType::Instant,
        cost(&[generic(2), r()]),
        2,
        0,
        vec![Keyword::Trample],
    )
}

/// Desperate Stand — {R}{W} Sorcery. Strive {R}{W}. +2/+0, first strike,
/// vigilance.
pub fn desperate_stand() -> CardDefinition {
    strive_pump(
        "Desperate Stand",
        cost(&[r(), w()]),
        CardType::Sorcery,
        cost(&[r(), w()]),
        2,
        0,
        vec![Keyword::FirstStrike, Keyword::Vigilance],
    )
}

/// Phalanx Formation — {2}{W} Instant. Strive {1}{W}. Double strike.
pub fn phalanx_formation() -> CardDefinition {
    strive_pump(
        "Phalanx Formation",
        cost(&[generic(2), w()]),
        CardType::Instant,
        cost(&[generic(1), w()]),
        0,
        0,
        vec![Keyword::DoubleStrike],
    )
}

/// Blinding Flare — {R} Sorcery. Strive {R}. Targets can't block this turn.
pub fn blinding_flare() -> CardDefinition {
    strive_pump(
        "Blinding Flare",
        cost(&[r()]),
        CardType::Sorcery,
        cost(&[r()]),
        0,
        0,
        vec![Keyword::CantBlock],
    )
}

/// Colossal Heroics — {2}{G} Instant. Strive {1}{G}. +2/+2 and untap.
pub fn colossal_heroics() -> CardDefinition {
    strive(
        "Colossal Heroics",
        cost(&[generic(2), g()]),
        CardType::Instant,
        cost(&[generic(1), g()]),
        R::Creature,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
        ]),
    )
}

/// Consign to Dust — {2}{G} Instant. Strive {2}{G}. Destroy any number of
/// target artifacts and/or enchantments.
pub fn consign_to_dust() -> CardDefinition {
    strive(
        "Consign to Dust",
        cost(&[generic(2), g()]),
        CardType::Instant,
        cost(&[generic(2), g()]),
        R::Artifact.or(R::Enchantment),
        Effect::Destroy {
            what: Selector::Target(0),
        },
    )
}

/// Kiora's Dismissal — {U} Instant. Strive {U}. Bounce any number of target
/// enchantments.
pub fn kioras_dismissal() -> CardDefinition {
    strive(
        "Kiora's Dismissal",
        cost(&[u()]),
        CardType::Instant,
        cost(&[u()]),
        R::Enchantment,
        Effect::Move {
            what: Selector::Target(0),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Nature's Panoply — {G} Instant. Strive {2}{G}. A +1/+1 counter on each
/// target creature.
pub fn natures_panoply() -> CardDefinition {
    strive(
        "Nature's Panoply",
        cost(&[g()]),
        CardType::Instant,
        cost(&[generic(2), g()]),
        R::Creature,
        Effect::AddCounter {
            what: Selector::Target(0),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    )
}

/// Solidarity of Heroes — {1}{G} Instant. Strive {1}{G}. Double the +1/+1
/// counters on each target creature.
pub fn solidarity_of_heroes() -> CardDefinition {
    strive(
        "Solidarity of Heroes",
        cost(&[generic(1), g()]),
        CardType::Instant,
        cost(&[generic(1), g()]),
        R::Creature,
        Effect::DoubleCountersOnEach {
            what: Selector::Target(0),
            kind: CounterType::PlusOnePlusOne,
        },
    )
}

/// Silence the Believers — {2}{B}{B} Instant. Strive {2}{B}. Exile any number
/// of target creatures. (The attached Auras ride along to the graveyard as an
/// SBA rather than being exiled with them.)
pub fn silence_the_believers() -> CardDefinition {
    strive(
        "Silence the Believers",
        cost(&[generic(2), b(), b()]),
        CardType::Instant,
        cost(&[generic(2), b()]),
        R::Creature,
        Effect::Exile {
            what: Selector::Target(0),
        },
    )
}

/// Harness by Force — {1}{R}{R} Sorcery. Strive {2}{R}. Steal any number of
/// target creatures until end of turn; untap them and give them haste.
pub fn harness_by_force() -> CardDefinition {
    strive(
        "Harness by Force",
        cost(&[generic(1), r(), r()]),
        CardType::Sorcery,
        cost(&[generic(2), r()]),
        R::Creature,
        Effect::Seq(vec![
            Effect::GainControl {
                what: Selector::Target(0),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Hour of Need — {2}{U} Instant. Strive {1}{U}. Exile any number of target
/// creatures; each controller gets a 4/4 blue flying Sphinx.
pub fn hour_of_need() -> CardDefinition {
    strive(
        "Hour of Need",
        cost(&[generic(2), u()]),
        CardType::Instant,
        cost(&[generic(1), u()]),
        R::Creature,
        Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Sphinx".into(),
                    power: 4,
                    toughness: 4,
                    colors: vec![Color::Blue],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Sphinx],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                },
            },
            Effect::Exile {
                what: Selector::Target(0),
            },
        ]),
    )
}

/// Twinflame — {1}{R} Sorcery. Strive {2}{R}. Token copy of each target
/// creature you control, with haste, exiled at the next end step.
pub fn twinflame() -> CardDefinition {
    strive(
        "Twinflame",
        cost(&[generic(1), r()]),
        CardType::Sorcery,
        cost(&[generic(2), r()]),
        R::Creature.and(R::ControlledByYou),
        Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::Const(1),
                source: Selector::Target(0),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![Keyword::Haste],
            },
            Effect::ExileLastCreatedTokensAtNextEndStep,
        ]),
    )
}

/// Polymorphous Rush — {2}{U} Instant. Strive {1}{U}. Any number of target
/// creatures you control become a copy of a chosen creature until end of turn.
pub fn polymorphous_rush() -> CardDefinition {
    strive(
        "Polymorphous Rush",
        cost(&[generic(2), u()]),
        CardType::Instant,
        cost(&[generic(1), u()]),
        R::Creature.and(R::ControlledByYou),
        Effect::Seq(vec![
            Effect::ChoosePermanentForSource {
                filter: R::Creature,
            },
            Effect::BecomeCopyOfFor {
                what: Selector::Target(0),
                source: Selector::ChosenPermanentOfSource,
                duration: Duration::EndOfTurn,
                non_legendary: false,
            },
        ]),
    )
}

/// Launch the Fleet — {W} Sorcery. Strive {1}. Until end of turn, any number
/// of target creatures gain "whenever this attacks, create a tapped and
/// attacking 1/1 white Soldier."
pub fn launch_the_fleet() -> CardDefinition {
    strive(
        "Launch the Fleet",
        cost(&[w()]),
        CardType::Sorcery,
        cost(&[generic(1)]),
        R::Creature,
        Effect::GrantTriggeredAbility {
            what: Selector::Target(0),
            trigger: Box::new(TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: soldier_token(),
                    cleanup: AttackingTokenCleanup::None,
                },
            }),
            duration: Duration::EndOfTurn,
        },
    )
}

fn soldier_token() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Setessan Tactics — {1}{G} Instant. Strive {G}. Until end of turn, any
/// number of target creatures get +1/+1 and gain "{T}: This creature fights
/// another target creature."
pub fn setessan_tactics() -> CardDefinition {
    strive(
        "Setessan Tactics",
        cost(&[generic(1), g()]),
        CardType::Instant,
        cost(&[g()]),
        R::Creature,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GainActivatedAbility {
                what: Selector::Target(0),
                ability: Box::new(ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::Fight {
                        attacker: Selector::This,
                        defender: target_filtered(R::Creature),
                    },
                    ..Default::default()
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

// ── Constellation ────────────────────────────────────────────────────────────

/// "Constellation — whenever this or another enchantment you control enters,
/// `body`."
fn constellation(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Enchantment,
            },
        ),
        effect: body,
    }
}

/// Agent of Erebos — {3}{B} 2/2 Nyx Zombie. Constellation: exile target
/// player's graveyard.
pub fn agent_of_erebos() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![constellation(Effect::ExilePlayerGraveyard {
            who: PlayerRef::Target(0),
        })],
        ..nyx_creature(
            "Agent of Erebos",
            cost(&[generic(3), b()]),
            2,
            2,
            vec![CreatureType::Zombie],
            vec![],
        )
    }
}

/// Dreadbringer Lampads — {4}{B} 4/2 Nymph. Constellation: target creature
/// gains intimidate until end of turn.
pub fn dreadbringer_lampads() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![constellation(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Intimidate,
            duration: Duration::EndOfTurn,
        })],
        ..nyx_creature(
            "Dreadbringer Lampads",
            cost(&[generic(4), b()]),
            4,
            2,
            vec![CreatureType::Nymph],
            vec![],
        )
    }
}

/// Forgeborn Oreads — {2}{R}{R} 4/2 Nymph. Constellation: 1 damage to any
/// target.
pub fn forgeborn_oreads() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![constellation(Effect::DealDamage {
            to: crate::effect::shortcut::target_any(),
            amount: Value::Const(1),
        })],
        ..nyx_creature(
            "Forgeborn Oreads",
            cost(&[generic(2), r(), r()]),
            4,
            2,
            vec![CreatureType::Nymph],
            vec![],
        )
    }
}

/// Goldenhide Ox — {5}{G} 5/4 Nyx Ox. Constellation: target creature must be
/// blocked this turn if able.
pub fn goldenhide_ox() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![constellation(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::MustBeBlocked,
            duration: Duration::EndOfTurn,
        })],
        ..nyx_creature(
            "Goldenhide Ox",
            cost(&[generic(5), g()]),
            5,
            4,
            vec![CreatureType::Ox],
            vec![],
        )
    }
}

/// Harvestguard Alseids — {2}{W} 2/3 Nymph. Constellation: prevent all damage
/// that would be dealt to target creature this turn.
pub fn harvestguard_alseids() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![constellation(Effect::PreventAllDamageThisTurn {
            target: target_filtered(R::Creature),
            redirect_to: None,
        })],
        ..nyx_creature(
            "Harvestguard Alseids",
            cost(&[generic(2), w()]),
            2,
            3,
            vec![CreatureType::Nymph],
            vec![],
        )
    }
}

/// Humbler of Mortals — {4}{G}{G} 5/5 Elemental. Constellation: creatures you
/// control gain trample until end of turn.
pub fn humbler_of_mortals() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![constellation(Effect::GrantKeyword {
            what: crate::effect::shortcut::each_your_creature(),
            keyword: Keyword::Trample,
            duration: Duration::EndOfTurn,
        })],
        ..nyx_creature(
            "Humbler of Mortals",
            cost(&[generic(4), g(), g()]),
            5,
            5,
            vec![CreatureType::Elemental],
            vec![],
        )
    }
}

/// Oakheart Dryads — {2}{G} 2/3 Nymph Dryad. Constellation: target creature
/// gets +1/+1 until end of turn.
pub fn oakheart_dryads() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![constellation(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..nyx_creature(
            "Oakheart Dryads",
            cost(&[generic(2), g()]),
            2,
            3,
            vec![CreatureType::Nymph, CreatureType::Dryad],
            vec![],
        )
    }
}

/// Thassa's Devourer — {4}{U} 2/6 Elemental. Constellation: target player
/// mills two.
pub fn thassas_devourer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![constellation(Effect::Mill {
            who: Selector::Target(0),
            amount: Value::Const(2),
        })],
        ..nyx_creature(
            "Thassa's Devourer",
            cost(&[generic(4), u()]),
            2,
            6,
            vec![CreatureType::Elemental],
            vec![],
        )
    }
}

/// Thoughtrender Lamia — {4}{B}{B} 5/3 Lamia. Constellation: each opponent
/// discards a card.
pub fn thoughtrender_lamia() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![constellation(Effect::Discard {
            who: crate::effect::shortcut::each_opponent(),
            amount: Value::Const(1),
            random: false,
        })],
        ..nyx_creature(
            "Thoughtrender Lamia",
            cost(&[generic(4), b(), b()]),
            5,
            3,
            vec![CreatureType::Lamia],
            vec![],
        )
    }
}

/// Whitewater Naiads — {3}{U}{U} 4/4 Nymph. Constellation: target creature
/// can't be blocked this turn.
pub fn whitewater_naiads() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![constellation(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Unblockable,
            duration: Duration::EndOfTurn,
        })],
        ..nyx_creature(
            "Whitewater Naiads",
            cost(&[generic(3), u(), u()]),
            4,
            4,
            vec![CreatureType::Nymph],
            vec![],
        )
    }
}

/// Strength from the Fallen — {1}{G} Enchantment. Constellation: target
/// creature gets +X/+X, X = creature cards in your graveyard.
pub fn strength_from_the_fallen() -> CardDefinition {
    CardDefinition {
        name: "Strength from the Fallen",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![constellation(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::count(Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::You),
                filter: R::Creature,
            }),
            toughness: Value::count(Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::You),
                filter: R::Creature,
            }),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Skybind — {3}{W}{W} Enchantment. Constellation: exile target nonenchantment
/// permanent; it returns at the beginning of the next end step.
pub fn skybind() -> CardDefinition {
    CardDefinition {
        name: "Skybind",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![constellation(Effect::ExileReturnNextEndStep {
            what: target_filtered(R::Permanent.and(R::Enchantment.negate())),
        })],
        ..Default::default()
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Bearer of the Heavens — {7}{R} 10/10 Giant. When it dies, destroy all
/// permanents at the beginning of the next end step.
pub fn bearer_of_the_heavens() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::AtNextEndStep {
            body: Box::new(Effect::Destroy {
                what: Selector::EachPermanent(R::Any),
            }),
        })],
        ..creature(
            "Bearer of the Heavens",
            cost(&[generic(7), r()]),
            10,
            10,
            vec![CreatureType::Giant],
            vec![],
        )
    }
}

/// Satyr Hoplite — {R} 1/1 Satyr Soldier. Heroic: a +1/+1 counter.
pub fn satyr_hoplite() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..creature(
            "Satyr Hoplite",
            cost(&[r()]),
            1,
            1,
            vec![CreatureType::Satyr, CreatureType::Soldier],
            vec![],
        )
    }
}

/// War-Wing Siren — {2}{U} 1/3 Siren Soldier. Flying; heroic: a +1/+1 counter.
pub fn war_wing_siren() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..creature(
            "War-Wing Siren",
            cost(&[generic(2), u()]),
            1,
            3,
            vec![CreatureType::Siren, CreatureType::Soldier],
            vec![Keyword::Flying],
        )
    }
}

/// Bloodcrazed Hoplite — {1}{B} 2/1 Human Soldier. Heroic: a +1/+1 counter;
/// whenever one is put on it, remove one from target opposing creature.
pub fn bloodcrazed_hoplite() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            heroic(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                    EventScope::SelfSource,
                ),
                effect: Effect::RemoveCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..creature(
            "Bloodcrazed Hoplite",
            cost(&[generic(1), b()]),
            2,
            1,
            vec![CreatureType::Human, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Tethmos High Priest — {2}{W} 2/3 Cat Cleric. Heroic: return target creature
/// card with mana value 2 or less from your graveyard to the battlefield.
pub fn tethmos_high_priest() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::Move {
            what: target_filtered(
                R::Creature
                    .and(R::InGraveyard)
                    .and(R::ManaValueAtMost(2))
                    .and(R::OwnedByYou),
            ),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        })],
        ..creature(
            "Tethmos High Priest",
            cost(&[generic(2), w()]),
            2,
            3,
            vec![CreatureType::Cat, CreatureType::Cleric],
            vec![],
        )
    }
}

/// Triton Cavalry — {3}{U} 2/4 Merfolk Soldier. Heroic: you may bounce target
/// enchantment.
pub fn triton_cavalry() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![heroic(Effect::MayDo {
            description: "Return target enchantment to its owner's hand?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(R::Enchantment),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..creature(
            "Triton Cavalry",
            cost(&[generic(3), u()]),
            2,
            4,
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            vec![],
        )
    }
}

/// Triton Shorestalker — {U} 1/1 Merfolk Rogue. Can't be blocked.
pub fn triton_shorestalker() -> CardDefinition {
    creature(
        "Triton Shorestalker",
        cost(&[u()]),
        1,
        1,
        vec![CreatureType::Merfolk, CreatureType::Rogue],
        vec![Keyword::Unblockable],
    )
}

/// Skyspear Cavalry — {3}{W}{W} 2/2 Human Soldier. Flying, double strike.
pub fn skyspear_cavalry() -> CardDefinition {
    creature(
        "Skyspear Cavalry",
        cost(&[generic(3), w(), w()]),
        2,
        2,
        vec![CreatureType::Human, CreatureType::Soldier],
        vec![Keyword::Flying, Keyword::DoubleStrike],
    )
}

/// Rotted Hulk — {3}{B} 2/5 Elemental vanilla.
pub fn rotted_hulk() -> CardDefinition {
    creature(
        "Rotted Hulk",
        cost(&[generic(3), b()]),
        2,
        5,
        vec![CreatureType::Elemental],
        vec![],
    )
}

/// Returned Reveler — {1}{B} 1/3 Zombie Satyr. When it dies, each player
/// mills three.
pub fn returned_reveler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Mill {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(3),
        })],
        ..creature(
            "Returned Reveler",
            cost(&[generic(1), b()]),
            1,
            3,
            vec![CreatureType::Zombie, CreatureType::Satyr],
            vec![],
        )
    }
}

/// Satyr Grovedancer — {1}{G} 1/1 Satyr Shaman. ETB: a +1/+1 counter on target
/// creature.
pub fn satyr_grovedancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..creature(
            "Satyr Grovedancer",
            cost(&[generic(1), g()]),
            1,
            1,
            vec![CreatureType::Satyr, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Supply-Line Cranes — {3}{W}{W} 2/4 Bird. Flying; ETB: a +1/+1 counter on
/// target creature.
pub fn supply_line_cranes() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..creature(
            "Supply-Line Cranes",
            cost(&[generic(3), w(), w()]),
            2,
            4,
            vec![CreatureType::Bird],
            vec![Keyword::Flying],
        )
    }
}

/// Sigiled Skink — {1}{R} 2/1 Lizard. Whenever it attacks, scry 1.
pub fn sigiled_skink() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..creature(
            "Sigiled Skink",
            cost(&[generic(1), r()]),
            2,
            1,
            vec![CreatureType::Lizard],
            vec![],
        )
    }
}

/// Sigiled Starfish — {1}{U} 0/3 Starfish. {T}: Scry 1.
pub fn sigiled_starfish() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..creature(
            "Sigiled Starfish",
            cost(&[generic(1), u()]),
            0,
            3,
            vec![CreatureType::Starfish],
            vec![],
        )
    }
}

/// Godhunter Octopus — {5}{U} 5/5 Octopus. Can't attack unless the defending
/// player controls an enchantment or an enchanted permanent.
pub fn godhunter_octopus() -> CardDefinition {
    creature(
        "Godhunter Octopus",
        cost(&[generic(5), u()]),
        5,
        5,
        vec![CreatureType::Octopus],
        vec![Keyword::CanAttackOnlyIfDefenderControls(Box::new(
            R::Enchantment.or(R::IsEnchanted),
        ))],
    )
}

/// Squelching Leeches — {2}{B}{B} */* Leech. P/T = Swamps you control.
pub fn squelching_leeches() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatching {
            base_p: 0,
            base_t: 0,
            filter: Box::new(R::HasLandType(crate::card::LandType::Swamp)),
        }),
        ..creature(
            "Squelching Leeches",
            cost(&[generic(2), b(), b()]),
            0,
            0,
            vec![CreatureType::Leech],
            vec![],
        )
    }
}

/// Spawn of Thraxes — {5}{R}{R} 5/5 Dragon. Flying; ETB: damage to any target
/// equal to the Mountains you control.
pub fn spawn_of_thraxes() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: crate::effect::shortcut::target_any(),
            amount: Value::count(Selector::EachMatching {
                zone: ZoneRef::Battlefield,
                filter: R::HasLandType(crate::card::LandType::Mountain).and(R::ControlledByYou),
            }),
        })],
        ..creature(
            "Spawn of Thraxes",
            cost(&[generic(5), r(), r()]),
            5,
            5,
            vec![CreatureType::Dragon],
            vec![Keyword::Flying],
        )
    }
}

/// Ravenous Leucrocota — {3}{G} 2/4 Beast. Vigilance; {6}{G}: Monstrosity 3.
pub fn ravenous_leucrocota() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(6), g()]), 3)],
        ..creature(
            "Ravenous Leucrocota",
            cost(&[generic(3), g()]),
            2,
            4,
            vec![CreatureType::Beast],
            vec![Keyword::Vigilance],
        )
    }
}

/// Wildfire Cerberus — {4}{R} 4/3 Dog. {5}{R}{R}: Monstrosity 1. When it
/// becomes monstrous, 2 damage to each opponent and each creature they control.
pub fn wildfire_cerberus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(5), r(), r()]), 1)],
        triggered_abilities: vec![on_becomes_monstrous(Effect::Seq(vec![
            Effect::DealDamage {
                to: crate::effect::shortcut::each_opponent(),
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: crate::effect::shortcut::each_opponent_creature(),
                amount: Value::Const(2),
            },
        ]))],
        ..creature(
            "Wildfire Cerberus",
            cost(&[generic(4), r()]),
            4,
            3,
            vec![CreatureType::Dog],
            vec![],
        )
    }
}

/// Swarmborn Giant — {2}{G}{G} 6/6 Giant. Sacrifice it when you're dealt
/// combat damage; {4}{G}{G}: Monstrosity 2; reach while monstrous.
pub fn swarmborn_giant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monstrosity(cost(&[generic(4), g(), g()]), 2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::ControllerDealtCombatDamage,
                EventScope::SelfSource,
            ),
            effect: Effect::SacrificeSource,
        }],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is monstrous, it has reach.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Reach,
                condition: Predicate::SourceIsMonstrous,
            },
        }],
        ..creature(
            "Swarmborn Giant",
            cost(&[generic(2), g(), g()]),
            6,
            6,
            vec![CreatureType::Giant],
            vec![],
        )
    }
}

/// Renowned Weaver — {G} 1/1 Human Shaman. {1}{G}, Sacrifice: create a 1/3
/// green Spider enchantment creature token with reach.
pub fn renowned_weaver() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Spider".into(),
                    power: 1,
                    toughness: 3,
                    colors: vec![Color::Green],
                    card_types: vec![CardType::Enchantment, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Spider],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Reach],
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..creature(
            "Renowned Weaver",
            cost(&[g()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Stonewise Fortifier — {1}{W} 2/2 Human Wizard. {4}{W}: prevent all damage
/// target creature would deal to this creature this turn.
pub fn stonewise_fortifier() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: Effect::PreventAllDamageBetweenThisTurn {
                from: target_filtered(R::Creature),
                to: Selector::This,
            },
            ..Default::default()
        }],
        ..creature(
            "Stonewise Fortifier",
            cost(&[generic(1), w()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Riptide Chimera — {2}{U} 3/4 Chimera. Flying; at the beginning of your
/// upkeep, return an enchantment you control to its owner's hand.
pub fn riptide_chimera() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MoveChosen {
                from: Selector::EachPermanent(R::Enchantment.and(R::ControlledByYou)),
                filter: None,
                count: Value::Const(1),
                up_to: false,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..nyx_creature(
            "Riptide Chimera",
            cost(&[generic(2), u()]),
            3,
            4,
            vec![CreatureType::Chimera],
            vec![Keyword::Flying],
        )
    }
}

// ── Spells ───────────────────────────────────────────────────────────────────

/// Rollick of Abandon — {3}{R}{R} Sorcery. All creatures get +2/-2.
pub fn rollick_of_abandon() -> CardDefinition {
    spell(
        "Rollick of Abandon",
        cost(&[generic(3), r(), r()]),
        CardType::Sorcery,
        Effect::PumpPT {
            what: crate::effect::shortcut::each_creature(),
            power: Value::Const(2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Spiteful Blow — {4}{B}{B} Sorcery. Destroy target creature and target land.
pub fn spiteful_blow() -> CardDefinition {
    spell(
        "Spiteful Blow",
        cost(&[generic(4), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature),
            },
            Effect::Destroy {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Land,
                },
            },
        ]),
    )
}

/// Spite of Mogis — {R} Sorcery. Damage to target creature equal to the
/// instants and sorceries in your graveyard, then scry 1.
pub fn spite_of_mogis() -> CardDefinition {
    spell(
        "Spite of Mogis",
        cost(&[r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::count(Selector::EachMatching {
                    zone: ZoneRef::Graveyard(PlayerRef::You),
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                }),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
    )
}

/// Starfall — {4}{R} Instant. 3 damage to target creature; if it's an
/// enchantment, 3 damage to its controller too.
pub fn starfall() -> CardDefinition {
    spell(
        "Starfall",
        cost(&[generic(4), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(3),
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::Enchantment,
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(3),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Revel of the Fallen God — {3}{R}{R}{G}{G} Sorcery. Four 2/2 red and green
/// Satyrs with haste.
pub fn revel_of_the_fallen_god() -> CardDefinition {
    spell(
        "Revel of the Fallen God",
        cost(&[generic(3), r(), r(), g(), g()]),
        CardType::Sorcery,
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(4),
            definition: TokenDefinition {
                name: "Satyr".into(),
                power: 2,
                toughness: 2,
                colors: vec![Color::Red, Color::Green],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Satyr],
                    ..Default::default()
                },
                keywords: vec![Keyword::Haste],
                ..Default::default()
            },
        },
    )
}

/// Rise of Eagles — {4}{U}{U} Sorcery. Two 2/2 blue Bird enchantment creature
/// tokens with flying, then scry 1.
pub fn rise_of_eagles() -> CardDefinition {
    spell(
        "Rise of Eagles",
        cost(&[generic(4), u(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: TokenDefinition {
                    name: "Bird".into(),
                    power: 2,
                    toughness: 2,
                    colors: vec![Color::Blue],
                    card_types: vec![CardType::Enchantment, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Bird],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                },
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
    )
}

/// Reviving Melody — {2}{G} Sorcery. Choose one or both: return target
/// creature card and/or target enchantment card from your graveyard to hand.
pub fn reviving_melody() -> CardDefinition {
    spell(
        "Reviving Melody",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InGraveyard).and(R::OwnedByYou)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Move {
                    what: target_filtered(R::Enchantment.and(R::InGraveyard).and(R::OwnedByYou)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}

/// Kruphix's Insight — {2}{G} Sorcery. Reveal the top six; up to three
/// enchantment cards to hand, the rest to the graveyard.
pub fn kruphixs_insight() -> CardDefinition {
    spell(
        "Kruphix's Insight",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(6),
            rest_to_graveyard: true,
            pick_filter: Some(R::Enchantment),
            take: Some(Value::Const(3)),
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: true,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
            rest_to_exile: false,
        },
    )
}

/// Tormented Thoughts — {2}{B} Sorcery. Sacrifice a creature; target player
/// discards cards equal to its power.
pub fn tormented_thoughts() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..spell(
            "Tormented Thoughts",
            cost(&[generic(2), b()]),
            CardType::Sorcery,
            Effect::Discard {
                who: target_filtered(R::Player),
                amount: Value::SacrificedPower,
                random: false,
            },
        )
    }
}

/// Ritual of the Returned — {3}{B} Instant. Exile target creature card from
/// your graveyard; create a black Zombie with its power and toughness.
pub fn ritual_of_the_returned() -> CardDefinition {
    spell(
        "Ritual of the Returned",
        cost(&[generic(3), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(R::Creature.and(R::InGraveyard).and(R::OwnedByYou)),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Zombie".into(),
                    colors: vec![Color::Black],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Zombie],
                        ..Default::default()
                    },
                    dynamic_pt: Some((
                        Value::PowerOf(Box::new(Selector::Target(0))),
                        Value::ToughnessOf(Box::new(Selector::Target(0))),
                    )),
                    ..Default::default()
                },
            },
        ]),
    )
}

/// Pull from the Deep — {2}{U}{U} Sorcery. Return up to one target instant and
/// up to one target sorcery card from your graveyard to your hand; exile this.
pub fn pull_from_the_deep() -> CardDefinition {
    let gy = |t: CardType| R::HasCardType(t).and(R::InGraveyard).and(R::OwnedByYou);
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Pull from the Deep",
            cost(&[generic(2), u(), u()]),
            CardType::Sorcery,
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(gy(CardType::Instant)),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                    Effect::Move {
                        what: Selector::TargetFiltered {
                            slot: 1,
                            filter: gy(CardType::Sorcery),
                        },
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                ])),
            },
        )
    }
}

/// Deicide — {1}{W} Instant. Exile target enchantment; if it's a God card,
/// exile every copy from its controller's graveyard, hand, and library.
pub fn deicide() -> CardDefinition {
    spell(
        "Deicide",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ExileSameNameAsTarget {
                what: target_filtered(R::Enchantment),
            },
            Effect::Exile {
                what: Selector::Target(0),
            },
        ]),
    )
}

// ── Auras and enchantments ───────────────────────────────────────────────────

/// Oppressive Rays — {W} Aura. Enchanted creature can't attack or block unless
/// its controller pays {3}, and its activated abilities cost {3} more.
pub fn oppressive_rays() -> CardDefinition {
    CardDefinition {
        name: "Oppressive Rays",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantAttackOrBlockUnlessPay(3)],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Activated abilities of enchanted creature cost {3} more to activate.",
            effect: StaticEffect::AttachedActivationTax { amount: 3 },
        }],
        ..Default::default()
    }
}

/// Market Festival — {3}{G} Aura on a land. Whenever the land is tapped for
/// mana, its controller adds two mana in any combination of colors.
pub fn market_festival() -> CardDefinition {
    CardDefinition {
        name: "Market Festival",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Land),
        },
        static_abilities: vec![StaticAbility {
            description: "Whenever enchanted land is tapped for mana, its controller adds two \
                          mana in any combination of colors.",
            effect: StaticEffect::ExtraManaOnLandTap {
                enchanted_only: true,
                filter: R::Any,
                extra: ExtraManaKind::AnyColors(2),
                while_monarch: false,
            },
        }],
        ..Default::default()
    }
}

/// Thassa's Ire — {U} Enchantment. {3}{U}: You may tap or untap target
/// creature.
pub fn thassas_ire() -> CardDefinition {
    CardDefinition {
        name: "Thassa's Ire",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::TapOrUntap {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Knowledge and Power — {4}{R} Enchantment. Whenever you scry, you may pay
/// {2}; if you do, 2 damage to any target.
pub fn knowledge_and_power() -> CardDefinition {
    CardDefinition {
        name: "Knowledge and Power",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ScriedOrSurveiled, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {2} to deal 2 damage?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::DealDamage {
                    to: crate::effect::shortcut::target_any(),
                    amount: Value::Const(2),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}
