//! Edge of Eternities — Exhaust (CR 702.177). "Exhaust — [Cost]: [Effect]"
//! means "[Cost]: [Effect]. Activate only once" (per game). Modeled via the
//! `ActivatedAbility.exhaust` flag + `CardInstance.exhausted_abilities`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement,
    StationBand, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{
    etb, flurry, on_attack, on_dies, station, target, target_any, target_filtered, warp,
};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, x};
use crabomination_base::tokens::lander_token;

/// Shared exhaust ability: "Exhaust — [cost]: Put N +1/+1 counters on this."
fn exhaust_self_counters(mana: crate::mana::ManaCost, n: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        exhaust: true,
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(n),
        },
        ..Default::default()
    }
}

/// Camera Launcher — {3} Artifact Creature — Construct 2/2. "Exhaust — {3}:
/// Put a +1/+1 counter on this creature. Create a 1/1 colorless Thopter
/// artifact creature token with flying."
pub fn camera_launcher() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Camera Launcher",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: thopter },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hazard of the Dunes — {3}{G} 4/4 Wurm. Trample, reach. "Exhaust — {6}{G}:
/// Put three +1/+1 counters on this creature."
pub fn hazard_of_the_dunes() -> CardDefinition {
    CardDefinition {
        name: "Hazard of the Dunes",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Reach],
        activated_abilities: vec![exhaust_self_counters(cost(&[generic(6), g()]), 3)],
        ..Default::default()
    }
}

/// Prowcatcher Specialist — {1}{R} 2/1 Goblin Warrior. Haste. "Exhaust —
/// {3}{R}: Put two +1/+1 counters on this creature."
pub fn prowcatcher_specialist() -> CardDefinition {
    CardDefinition {
        name: "Prowcatcher Specialist",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![exhaust_self_counters(cost(&[generic(3), r()]), 2)],
        ..Default::default()
    }
}

/// Greenbelt Guardian — {1}{G} 2/2 Elf Ranger. "{G}: Target creature gains
/// trample until end of turn." plus "Exhaust — {3}{G}: Put three +1/+1
/// counters on this creature."
pub fn greenbelt_guardian() -> CardDefinition {
    CardDefinition {
        name: "Greenbelt Guardian",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Ranger],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::GrantKeyword {
                    what: target(),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            exhaust_self_counters(cost(&[generic(3), g()]), 3),
        ],
        ..Default::default()
    }
}

/// Pacesetter Paragon — {2}{R} 2/3 Human Pilot. "Exhaust — {2}{R}: Put a
/// +1/+1 counter on this creature. It gains double strike until end of turn."
pub fn pacesetter_paragon() -> CardDefinition {
    CardDefinition {
        name: "Pacesetter Paragon",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Keen Buccaneer — {2}{U} 2/3 Octopus Pirate. Vigilance. "Exhaust — {1}{U}:
/// Draw a card, then discard a card. Put a +1/+1 counter on this creature."
pub fn keen_buccaneer() -> CardDefinition {
    CardDefinition {
        name: "Keen Buccaneer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Octopus, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skystreak Engineer — {1}{U} 1/3 Human Pilot. Flying. "Exhaust — {4}{U}:
/// Put two +1/+1 counters on this creature."
pub fn skystreak_engineer() -> CardDefinition {
    CardDefinition {
        name: "Skystreak Engineer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![exhaust_self_counters(cost(&[generic(4), u()]), 2)],
        ..Default::default()
    }
}

/// Mai, Jaded Edge — {1}{R} 1/3 Legendary Human Noble. Prowess. "Exhaust —
/// {3}: Put a double strike counter on Mai."
pub fn mai_jaded_edge() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Mai, Jaded Edge",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Prowess],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            exhaust: true,
            effect: Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: Keyword::DoubleStrike,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stampeding Scurryfoot — {G} 1/1 Mouse. "Exhaust — {3}{G}: Put a +1/+1
/// counter on this creature. Create a 3/3 green Elephant creature token."
pub fn stampeding_scurryfoot() -> CardDefinition {
    use crabomination_base::mana::Color;
    let elephant = TokenDefinition {
        name: "Elephant".into(),
        power: 3,
        toughness: 3,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Stampeding Scurryfoot",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mouse], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: elephant },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mindspring Merfolk — {U} 1/1 Merfolk Wizard. "Exhaust — {X}{U}{U}, {T}:
/// Draw X cards. Put a +1/+1 counter on each Merfolk creature you control."
pub fn mindspring_merfolk() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::mana::x;
    CardDefinition {
        name: "Mindspring Merfolk",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[x(), u(), u()]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::XFromCost },
                Effect::AddCounter {
                    what: Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Merfolk)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Edge of Eternities — Warp (cast for a cheaper cost, exile at the next end
// step, recast from exile), Void (a nonland permanent left the battlefield or a
// spell was warped this turn), and Lander tokens (sac for a basic land).
// ─────────────────────────────────────────────────────────────────────────────

/// Bygone Colossus — {9} Artifact Creature — Robot Giant 9/9. Warp {3}.
pub fn bygone_colossus() -> CardDefinition {
    CardDefinition {
        name: "Bygone Colossus",
        cost: cost(&[generic(9)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Giant],
            ..Default::default()
        },
        power: 9,
        toughness: 9,
        alternative_cost: Some(warp(cost(&[generic(3)]))),
        ..Default::default()
    }
}

/// Codecracker Hound — {2}{U} Creature — Dog 2/1. ETB: look at the top two
/// cards, put one in hand and the other in your graveyard. Warp {2}{U}.
pub fn codecracker_hound() -> CardDefinition {
    CardDefinition {
        name: "Codecracker Hound",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(2),
            rest_to_graveyard: true,
            pick_filter: None,
            take: Some(Value::Const(1)),
            to_battlefield: false,
        })],
        alternative_cost: Some(warp(cost(&[generic(2), u()]))),
        ..Default::default()
    }
}

/// Nova Hellkite — {3}{R}{R} Creature — Dragon 4/5, flying, haste. ETB: deal 1
/// damage to target creature an opponent controls. Warp {2}{R}.
pub fn nova_hellkite() -> CardDefinition {
    CardDefinition {
        name: "Nova Hellkite",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![etb(Effect::DealDamage {
            amount: Value::Const(1),
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        alternative_cost: Some(warp(cost(&[generic(2), r()]))),
        ..Default::default()
    }
}

/// Drix Fatemaker — {3}{G} Creature — Drix Wizard 3/2. ETB: put a +1/+1 counter
/// on target creature. Each creature you control with a +1/+1 counter on it has
/// trample. Warp {1}{G}.
pub fn drix_fatemaker() -> CardDefinition {
    CardDefinition {
        name: "Drix Fatemaker",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drix, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target(),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a +1/+1 counter on it has trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                keyword: Keyword::Trample,
            },
        }],
        alternative_cost: Some(warp(cost(&[generic(1), g()]))),
        ..Default::default()
    }
}

/// Broodguard Elite — {X}{G}{G} Creature — Insect Knight, enters with X +1/+1
/// counters. When it leaves the battlefield, put its counters on target creature
/// you control. Warp {X}{G}.
pub fn broodguard_elite() -> CardDefinition {
    CardDefinition {
        name: "Broodguard Elite",
        cost: cost(&[x(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Knight],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::MoveAllCounters {
                from: Selector::This,
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
        }],
        alternative_cost: Some(warp(cost(&[x(), g()]))),
        ..Default::default()
    }
}

/// All-Fates Stalker — {3}{W} Creature — Drix Assassin 2/3. ETB: exile target
/// creature until this leaves the battlefield. Warp {1}{W}. (The "up to one
/// non-Assassin" rider is approximated as a plain target creature.)
pub fn all_fates_stalker() -> CardDefinition {
    CardDefinition {
        name: "All-Fates Stalker",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drix, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(SelectionRequirement::Creature),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        alternative_cost: Some(warp(cost(&[generic(1), w()]))),
        ..Default::default()
    }
}

/// Eusocial Engineering — {3}{G}{G} Enchantment. Landfall — whenever a land you
/// control enters, create a 2/2 colorless Robot artifact creature token. Warp
/// {1}{G}.
pub fn eusocial_engineering() -> CardDefinition {
    let robot = TokenDefinition {
        name: "Robot".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Eusocial Engineering",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Land,
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: robot,
            },
        }],
        alternative_cost: Some(warp(cost(&[generic(1), g()]))),
        ..Default::default()
    }
}

/// Decode Transmissions — {2}{B} Sorcery. You draw two cards and lose 2 life.
/// Void — if a nonland permanent left the battlefield this turn or a spell was
/// warped this turn, instead you draw two cards and each opponent loses 2 life.
pub fn decode_transmissions() -> CardDefinition {
    use crate::effect::shortcut::{draw, lose_life};
    CardDefinition {
        name: "Decode Transmissions",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::VoidActive { who: PlayerRef::You },
            then: Box::new(Effect::Seq(vec![
                draw(2),
                lose_life(2, Selector::Player(PlayerRef::EachOpponent)),
            ])),
            else_: Box::new(Effect::Seq(vec![draw(2), lose_life(2, Selector::You)])),
        },
        ..Default::default()
    }
}

/// Elegy Acolyte — {2}{B}{B} Creature — Human Cleric 4/4, lifelink. Whenever a
/// creature you control deals combat damage to a player, draw a card and lose 1
/// life. Void — at the beginning of your end step, if Void is active, draw a card
/// and lose 1 life. (The "one or more" batch is modeled per-creature.)
pub fn elegy_acolyte() -> CardDefinition {
    use crate::effect::shortcut::{draw, lose_life};
    let draw_lose = || Effect::Seq(vec![draw(1), lose_life(1, Selector::You)]);
    CardDefinition {
        name: "Elegy Acolyte",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
                effect: draw_lose(),
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::VoidActive { who: PlayerRef::You },
                    then: Box::new(draw_lose()),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Biomechan Engineer — {G}{U} Creature — Insect Artificer 2/2. ETB: create a
/// Lander token.
pub fn biomechan_engineer() -> CardDefinition {
    CardDefinition {
        name: "Biomechan Engineer",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Biotech Specialist — {R}{G} Creature — Insect Scientist 1/3. ETB: create a
/// Lander token.
pub fn biotech_specialist() -> CardDefinition {
    CardDefinition {
        name: "Biotech Specialist",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Scientist],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Beamsaw Prospector — {1}{B} Creature — Human Artificer 2/1. When it dies,
/// create a Lander token.
pub fn beamsaw_prospector() -> CardDefinition {
    CardDefinition {
        name: "Beamsaw Prospector",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Bioengineered Future — {1}{G}{G} Enchantment. ETB: create a Lander token.
pub fn bioengineered_future() -> CardDefinition {
    CardDefinition {
        name: "Bioengineered Future",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Blooming Stinger — {1}{G} Creature — Plant Scorpion 2/2, deathtouch. ETB:
/// another target creature you control gains deathtouch until end of turn.
pub fn blooming_stinger() -> CardDefinition {
    CardDefinition {
        name: "Blooming Stinger",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Scorpion],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            keyword: Keyword::Deathtouch,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Biosynthic Burst — {1}{G} Instant. Put a +1/+1 counter on target creature you
/// control; it gains reach, trample, and indestructible until end of turn, then
/// untap it.
pub fn biosynthic_burst() -> CardDefinition {
    CardDefinition {
        name: "Biosynthic Burst",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GrantKeyword { what: target(), keyword: Keyword::Reach, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: target(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            Effect::GrantKeyword {
                what: target(),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: target(), up_to: None },
        ]),
        ..Default::default()
    }
}

/// Cosmic Epiphany — {4}{U}{U} Sorcery. Draw cards equal to the number of instant
/// and sorcery cards in your graveyard.
pub fn cosmic_epiphany() -> CardDefinition {
    CardDefinition {
        name: "Cosmic Epiphany",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::CardsInGraveyardMatching {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            },
        },
        ..Default::default()
    }
}

/// Beyond the Quiet — {3}{W}{W} Sorcery. Exile all creatures. (Spacecraft aren't
/// modeled as a distinct type; the creature half is faithful.)
pub fn beyond_the_quiet() -> CardDefinition {
    CardDefinition {
        name: "Beyond the Quiet",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::Exile { what: Selector::TriggerSource }),
        },
        ..Default::default()
    }
}

/// Singularity Rupture — {3}{U}{B}{B} Sorcery. Destroy all creatures, then each
/// opponent mills half their library, rounded down. ("Any number of target
/// players" is approximated as each opponent.)
pub fn singularity_rupture() -> CardDefinition {
    CardDefinition {
        name: "Singularity Rupture",
        cost: cost(&[generic(3), u(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
            },
            Effect::MillHalf {
                who: Selector::Player(PlayerRef::EachOpponent),
                rounded_up: false,
            },
        ]),
        ..Default::default()
    }
}

/// Voyager Quickwelder — {2}{W} Artifact Creature — Robot Artificer 2/4. Artifact
/// spells you cast cost {1} less to cast.
pub fn voyager_quickwelder() -> CardDefinition {
    CardDefinition {
        name: "Voyager Quickwelder",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Artifact spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: SelectionRequirement::Artifact, amount: 1 },
        }],
        ..Default::default()
    }
}

/// Memory Guardian — {4}{U} Artifact Creature — Robot Artificer 3/4, flying.
/// Affinity for artifacts.
pub fn memory_guardian() -> CardDefinition {
    CardDefinition {
        name: "Memory Guardian",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        affinity_filter: Some(SelectionRequirement::Artifact),
        ..Default::default()
    }
}

/// Perimeter Patrol — {2}{G} Creature — Human Soldier 3/3. Whenever an artifact
/// you control enters, this creature gets +1/+0 until end of turn.
pub fn perimeter_patrol() -> CardDefinition {
    CardDefinition {
        name: "Perimeter Patrol",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Hulldrifter — {3}{U}{U} Artifact — Vehicle 3/2, flying. ETB: draw two cards.
/// Crew 3.
pub fn hulldrifter() -> CardDefinition {
    CardDefinition {
        name: "Hulldrifter",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Crew(3)],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(2) })],
        ..Default::default()
    }
}

/// Tidal Terror — {4}{U}{U} Creature — Octopus 5/6. Islandcycling {2}. (The
/// tap-two-to-be-unblockable attack rider is omitted.)
pub fn tidal_terror() -> CardDefinition {
    CardDefinition {
        name: "Tidal Terror",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Octopus], ..Default::default() },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Island)],
        ..Default::default()
    }
}

// ── EOE batch 2 — more Warp / Void / Lander ──────────────────────────────────

/// Germinating Wurm — {4}{G} Creature — Plant Wurm 5/5. ETB: gain 2 life. Warp
/// {1}{G}.
pub fn germinating_wurm() -> CardDefinition {
    use crate::effect::shortcut::gain_life;
    CardDefinition {
        name: "Germinating Wurm",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Wurm],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![etb(gain_life(2))],
        alternative_cost: Some(warp(cost(&[generic(1), g()]))),
        ..Default::default()
    }
}

/// Knight Luminary — {3}{W} Creature — Human Knight 3/2. ETB: make a 1/1 white
/// Human Soldier. Warp {1}{W}.
pub fn knight_luminary() -> CardDefinition {
    use crate::mana::Color;
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Knight Luminary",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: soldier,
        })],
        alternative_cost: Some(warp(cost(&[generic(1), w()]))),
        ..Default::default()
    }
}

/// Memorial Team Leader — {3}{R} Creature — Kavu Soldier 4/3. During your turn,
/// other creatures you control get +1/+0. Warp {1}{R}.
pub fn memorial_team_leader() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Memorial Team Leader",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "During your turn, other creatures you control get +1/+0.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::IsTurnOf(PlayerRef::You),
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        alternative_cost: Some(warp(cost(&[generic(1), r()]))),
        ..Default::default()
    }
}

/// Dauntless Scrapbot — {3} Artifact Creature — Robot 3/1. ETB: exile each
/// opponent's graveyard, then create a Lander token.
pub fn dauntless_scrapbot() -> CardDefinition {
    CardDefinition {
        name: "Dauntless Scrapbot",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExileAllGraveyards { filter: None, opponents_only: true },
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: lander_token() },
        ]))],
        ..Default::default()
    }
}

/// Edge Rover — {G} Artifact Creature — Robot Scout 2/2, reach. When it dies,
/// each player creates a Lander token.
pub fn edge_rover() -> CardDefinition {
    CardDefinition {
        name: "Edge Rover",
        cost: cost(&[g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::EachPlayer,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Galactic Wayfarer — {2}{G} Creature — Human Scout 3/3. ETB: create a Lander.
pub fn galactic_wayfarer() -> CardDefinition {
    CardDefinition {
        name: "Galactic Wayfarer",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Glacier Godmaw — {5}{G}{G} Creature — Leviathan 6/6, trample. ETB: create a
/// Lander.
pub fn glacier_godmaw() -> CardDefinition {
    CardDefinition {
        name: "Glacier Godmaw",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Leviathan], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Kav Landseeker — {3}{R} Creature — Kavu Soldier 4/3, menace. ETB: create a
/// Lander. (The "sacrifice it next end step" rider is dropped.)
pub fn kav_landseeker() -> CardDefinition {
    CardDefinition {
        name: "Kav Landseeker",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: lander_token(),
        })],
        ..Default::default()
    }
}

/// Emergency Eject — {2}{W} Instant. Destroy target nonland permanent; its
/// controller creates a Lander token.
pub fn emergency_eject() -> CardDefinition {
    CardDefinition {
        name: "Emergency Eject",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: lander_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Kavaron Skywarden — {4}{R} Creature — Kavu Soldier 4/5, reach. Void — at the
/// beginning of your end step, if Void is active, put a +1/+1 counter on this.
pub fn kavaron_skywarden() -> CardDefinition {
    CardDefinition {
        name: "Kavaron Skywarden",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::VoidActive { who: PlayerRef::You },
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Mechanozoa — {4}{U}{U} Artifact Creature — Robot Jellyfish 5/5. ETB: tap
/// target artifact or creature an opponent controls and put a stun counter on it.
pub fn mechanozoa() -> CardDefinition {
    CardDefinition {
        name: "Mechanozoa",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Jellyfish],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature)
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Exalted Sunborn — {3}{W}{W} Creature — Angel Wizard 4/5, flying, lifelink. If
/// one or more tokens would be created under your control, twice that many are
/// created instead. Warp {1}{W}.
pub fn exalted_sunborn() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Exalted Sunborn",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "If one or more tokens would be created under your control, twice that many are created instead.",
            effect: StaticEffect::DoubleTokens,
        }],
        alternative_cost: Some(warp(cost(&[generic(1), w()]))),
        ..Default::default()
    }
}

// ── EOE batch 3 — simple commons ─────────────────────────────────────────────

/// Bombard — {2}{R} Instant. Deal 4 damage to target creature.
pub fn bombard() -> CardDefinition {
    CardDefinition {
        name: "Bombard",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            amount: Value::Const(4),
            to: target_filtered(SelectionRequirement::Creature),
        },
        ..Default::default()
    }
}

/// Cloudsculpt Technician — {2}{U} Creature — Jellyfish Artificer 2/2, flying. As
/// long as you control an artifact, this creature gets +1/+0.
pub fn cloudsculpt_technician() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Cloudsculpt Technician",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "As long as you control an artifact, this creature gets +1/+0.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(1),
                },
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Brightspear Zealot — {2}{W} Creature — Human Soldier 2/2, vigilance. Gets
/// +2/+0 as long as you've cast two or more spells this turn.
pub fn brightspear_zealot() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Brightspear Zealot",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Gets +2/+0 as long as you've cast two or more spells this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(2),
                },
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Eumidian Terrabotanist — {1}{G} Creature — Insect Druid 2/1. Landfall —
/// whenever a land you control enters, you gain 1 life.
pub fn eumidian_terrabotanist() -> CardDefinition {
    use crate::effect::shortcut::gain_life;
    CardDefinition {
        name: "Eumidian Terrabotanist",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Land,
                }),
            effect: gain_life(1),
        }],
        ..Default::default()
    }
}

/// Dockworker Drone — {1}{W} Artifact Creature — Robot 1/1. Enters with a +1/+1
/// counter. When it dies, put its counters on target creature you control.
pub fn dockworker_drone() -> CardDefinition {
    CardDefinition {
        name: "Dockworker Drone",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 1,
        toughness: 1,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        triggered_abilities: vec![on_dies(Effect::MoveAllCounters {
            from: Selector::This,
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
        })],
        ..Default::default()
    }
}

/// Dual-Sun Adepts — {2}{W} Creature — Human Soldier 2/2, double strike. {5}:
/// Creatures you control get +1/+1 until end of turn.
pub fn dual_sun_adepts() -> CardDefinition {
    CardDefinition {
        name: "Dual-Sun Adepts",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::DoubleStrike],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dual-Sun Technique — {1}{W} Instant. Target creature you control gains double
/// strike until end of turn. If it has a +1/+1 counter on it, draw a card.
pub fn dual_sun_technique() -> CardDefinition {
    use crate::effect::shortcut::draw;
    CardDefinition {
        name: "Dual-Sun Technique",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                },
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Exosuit Savior — {2}{W} Creature — Human Soldier 2/3, flying. ETB: return
/// another target permanent you control to its owner's hand. (The "up to one"
/// optional rider is modeled as a plain target.)
pub fn exosuit_savior() -> CardDefinition {
    CardDefinition {
        name: "Exosuit Savior",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::Permanent
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        })],
        ..Default::default()
    }
}

// ── EOE batch 4 — more Void ──────────────────────────────────────────────────

/// Hymn of the Faller — {1}{B} Sorcery. Surveil 1, then draw a card and lose 1
/// life. Void — if a nonland permanent left the battlefield this turn or a spell
/// was warped this turn, draw another card.
pub fn hymn_of_the_faller() -> CardDefinition {
    use crate::effect::shortcut::{draw, lose_life};
    CardDefinition {
        name: "Hymn of the Faller",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
            draw(1),
            lose_life(1, Selector::You),
            Effect::If {
                cond: Predicate::VoidActive { who: PlayerRef::You },
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Interceptor Mechan — {2}{B}{R} Artifact Creature — Robot 2/2, flying. ETB:
/// return target artifact or creature card from your graveyard to your hand.
/// Void — at the beginning of your end step, if Void is active, put a +1/+1
/// counter on this creature.
pub fn interceptor_mechan() -> CardDefinition {
    CardDefinition {
        name: "Interceptor Mechan",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::VoidActive { who: PlayerRef::You },
                    then: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Insatiable Skittermaw — {2}{B} Creature — Insect Horror 2/2, menace. Void —
/// at the beginning of your end step, if Void is active, put a +1/+1 counter on
/// this creature.
pub fn insatiable_skittermaw() -> CardDefinition {
    CardDefinition {
        name: "Insatiable Skittermaw",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::VoidActive { who: PlayerRef::You },
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

// ── EOE Station cards (CR 721 + CR 702.184) ──────────────────────────────────
// Each Spacecraft enters as a noncreature artifact carrying the Station
// activated ability (`shortcut::station`). Once charge counters reach a band's
// `{N+}` threshold, the band's keywords + base P/T apply (it becomes a
// creature). Bands live in `CardDefinition.station`.

/// Wurmwall Sweeper — {2} Artifact — Spacecraft. ETB: surveil 2. Station;
/// {4+}: 2/2 with flying.
pub fn wurmwall_sweeper() -> CardDefinition {
    CardDefinition {
        name: "Wurmwall Sweeper",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) })],
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 4, keywords: vec![Keyword::Flying], pt: Some((2, 2)), ..Default::default() }],
        ..Default::default()
    }
}

/// Uthros Scanship — {3}{U} Artifact — Spacecraft. ETB: draw two, then discard
/// one. Station; {8+}: 4/4 with flying.
pub fn uthros_scanship() -> CardDefinition {
    CardDefinition {
        name: "Uthros Scanship",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        ]))],
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 8, keywords: vec![Keyword::Flying], pt: Some((4, 4)), ..Default::default() }],
        ..Default::default()
    }
}

/// Atmospheric Greenhouse — {4}{G} Artifact — Spacecraft. ETB: put a +1/+1
/// counter on each creature you control. Station; {8+}: 5/4 with flying,
/// trample.
pub fn atmospheric_greenhouse() -> CardDefinition {
    CardDefinition {
        name: "Atmospheric Greenhouse",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        activated_abilities: vec![station()],
        station: vec![StationBand {
            min: 8,
            keywords: vec![Keyword::Flying, Keyword::Trample],
            pt: Some((5, 4)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wedgelight Rammer — {3}{W} Artifact — Spacecraft. ETB: create a 2/2
/// colorless Robot artifact creature token. Station; {9+}: 3/4 with flying,
/// first strike.
pub fn wedgelight_rammer() -> CardDefinition {
    let robot = TokenDefinition {
        name: "Robot".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Wedgelight Rammer",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: robot,
        })],
        activated_abilities: vec![station()],
        station: vec![StationBand {
            min: 9,
            keywords: vec![Keyword::Flying, Keyword::FirstStrike],
            pt: Some((3, 4)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fell Gravship — {2}{B} Artifact — Spacecraft. ETB: mill three, then return a
/// creature or Spacecraft card from your graveyard to your hand. Station;
/// {8+}: 3/2 with flying, lifelink.
pub fn fell_gravship() -> CardDefinition {
    CardDefinition {
        name: "Fell Gravship",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::ReturnGraveyardCardsToHand {
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Spacecraft)),
                max: Value::Const(1),
            },
        ]))],
        activated_abilities: vec![station()],
        station: vec![StationBand {
            min: 8,
            keywords: vec![Keyword::Flying, Keyword::Lifelink],
            pt: Some((3, 2)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Warmaker Gunship — {2}{R} Artifact — Spacecraft. ETB: deal damage equal to
/// the number of artifacts you control to target creature an opponent
/// controls. Station; {6+}: 4/3 with flying.
pub fn warmaker_gunship() -> CardDefinition {
    CardDefinition {
        name: "Warmaker Gunship",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::DealDamage {
            amount: Value::count(Selector::EachPermanent(
                SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
            )),
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 6, keywords: vec![Keyword::Flying], pt: Some((4, 3)), ..Default::default() }],
        ..Default::default()
    }
}

/// Sledge-Class Seedship — {2}{G} Artifact — Spacecraft. Station; {7+}: 4/5
/// with flying. Whenever it attacks, you may put a creature card from your hand
/// onto the battlefield.
pub fn sledge_class_seedship() -> CardDefinition {
    CardDefinition {
        name: "Sledge-Class Seedship",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 7, keywords: vec![Keyword::Flying], pt: Some((4, 5)), ..Default::default() }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Put a creature card from your hand onto the battlefield".into(),
                body: Box::new(Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature,
                    count: Value::Const(1),
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Larval Scoutlander — {2}{G} Artifact — Spacecraft. ETB: you may sacrifice a
/// land. If you do, search your library for up to two basic lands and put them
/// onto the battlefield tapped. Station; {7+}: 3/3 with flying. (The "or
/// Lander" sacrifice option is approximated as land-only.)
pub fn larval_scoutlander() -> CardDefinition {
    CardDefinition {
        name: "Larval Scoutlander",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice a land".into(),
            filter: SelectionRequirement::Land,
            count: Value::Const(1),
            then: Box::new(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::Const(2),
            }),
            else_: None,
        })],
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 7, keywords: vec![Keyword::Flying], pt: Some((3, 3)), ..Default::default() }],
        ..Default::default()
    }
}

/// Galvanizing Sawship — {5}{R} Artifact — Spacecraft. Station; {3+}: 6/5 with
/// flying, haste.
pub fn galvanizing_sawship() -> CardDefinition {
    CardDefinition {
        name: "Galvanizing Sawship",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        activated_abilities: vec![station()],
        station: vec![StationBand {
            min: 3,
            keywords: vec![Keyword::Flying, Keyword::Haste],
            pt: Some((6, 5)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Susurian Dirgecraft — {4}{B} Artifact — Spacecraft. ETB: each opponent
/// sacrifices a nontoken creature of their choice. Station; {7+}: 4/3 with
/// flying.
pub fn susurian_dirgecraft() -> CardDefinition {
    CardDefinition {
        name: "Susurian Dirgecraft",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::Not(Box::new(SelectionRequirement::IsToken))),
        })],
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 7, keywords: vec![Keyword::Flying], pt: Some((4, 3)), ..Default::default() }],
        ..Default::default()
    }
}

/// Pinnacle Kill-Ship — {7} Artifact — Spacecraft. ETB: deals 10 damage to up
/// to one target creature. Station; {7+}: 7/7 with flying.
pub fn pinnacle_kill_ship() -> CardDefinition {
    CardDefinition {
        name: "Pinnacle Kill-Ship",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::DealDamage {
                amount: Value::Const(10),
                to: Selector::Target(0),
            }),
        })],
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 7, keywords: vec![Keyword::Flying], pt: Some((7, 7)), ..Default::default() }],
        ..Default::default()
    }
}

/// Debris Field Crusher — {4}{R} Artifact — Spacecraft. ETB: deals 3 damage to
/// any target. {1}{R}: gets +2/+0 until end of turn. Station; {8+}: 1/5 with
/// flying.
pub fn debris_field_crusher() -> CardDefinition {
    CardDefinition {
        name: "Debris Field Crusher",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::DealDamage {
            amount: Value::Const(3),
            to: target_any(),
        })],
        activated_abilities: vec![
            station(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        station: vec![StationBand { min: 8, keywords: vec![Keyword::Flying], pt: Some((1, 5)), ..Default::default() }],
        ..Default::default()
    }
}

/// Extinguisher Battleship — {8} Artifact — Spacecraft. ETB: destroy target
/// noncreature permanent, then deal 4 damage to each creature. Station; {5+}:
/// 10/10 with flying, trample.
pub fn extinguisher_battleship() -> CardDefinition {
    CardDefinition {
        name: "Extinguisher Battleship",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Noncreature),
                ),
            },
            Effect::DealDamage {
                amount: Value::Const(4),
                to: Selector::EachPermanent(SelectionRequirement::Creature),
            },
        ]))],
        activated_abilities: vec![station()],
        station: vec![StationBand {
            min: 5,
            keywords: vec![Keyword::Flying, Keyword::Trample],
            pt: Some((10, 10)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Specimen Freighter — {5}{U} Artifact — Spacecraft. ETB: return up to two
/// target non-Spacecraft creatures to their owners' hands. Whenever it attacks,
/// defending player mills four. Station; {9+}: 4/7 with flying.
pub fn specimen_freighter() -> CardDefinition {
    CardDefinition {
        name: "Specimen Freighter",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![
            etb(Effect::ApplyToTargets {
                max_targets: 2,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::Not(Box::new(SelectionRequirement::HasArtifactSubtype(
                        ArtifactSubtype::Spacecraft,
                    )))),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::DefendingPlayer),
                    amount: Value::Const(4),
                },
            },
        ],
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 9, keywords: vec![Keyword::Flying], pt: Some((4, 7)), ..Default::default() }],
        ..Default::default()
    }
}

/// Rescue Skiff — {5}{W} Artifact — Spacecraft. ETB: return target creature or
/// enchantment card from your graveyard to the battlefield. Station; {10+}: 5/6
/// with flying.
pub fn rescue_skiff() -> CardDefinition {
    CardDefinition {
        name: "Rescue Skiff",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::InYourGraveyard.and(
                    SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                ),
            ),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 10, keywords: vec![Keyword::Flying], pt: Some((5, 6)), ..Default::default() }],
        ..Default::default()
    }
}

/// Lumen-Class Frigate — {1}{W} Artifact — Spacecraft. Station; {2+}: other
/// creatures you control get +1/+1. {12+}: 3/5 with flying, lifelink.
pub fn lumen_class_frigate() -> CardDefinition {
    let others = Selector::EachPermanent(
        SelectionRequirement::Creature
            .and(SelectionRequirement::ControlledByYou)
            .and(SelectionRequirement::OtherThanSource),
    );
    CardDefinition {
        name: "Lumen-Class Frigate",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        activated_abilities: vec![station()],
        station: vec![
            StationBand {
                min: 2,
                statics: vec![StaticEffect::PumpPT { applies_to: others, power: 1, toughness: 1 }],
                ..Default::default()
            },
            StationBand {
                min: 12,
                keywords: vec![Keyword::Flying, Keyword::Lifelink],
                pt: Some((3, 5)),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Synthesizer Labship — {U} Artifact — Spacecraft. Station; {2+}: at the
/// beginning of combat on your turn, up to one other target artifact you
/// control becomes a 2/2 artifact creature with flying until end of turn.
/// {9+}: 4/4 with flying, vigilance.
pub fn synthesizer_labship() -> CardDefinition {
    CardDefinition {
        name: "Synthesizer Labship",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        activated_abilities: vec![station()],
        station: vec![
            StationBand {
                min: 2,
                triggers: vec![TriggeredAbility {
                    event: EventSpec::new(
                        EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                        EventScope::YourControl,
                    ),
                    effect: Effect::ApplyToTargets {
                        max_targets: 1,
                        filter: SelectionRequirement::Artifact
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                        effect: Box::new(Effect::BecomeCreature {
                            what: Selector::Target(0),
                            power: Value::Const(2),
                            toughness: Value::Const(2),
                            creature_types: vec![],
                            keywords: vec![Keyword::Flying],
                            duration: Duration::EndOfTurn,
                        }),
                    },
                }],
                ..Default::default()
            },
            StationBand {
                min: 9,
                keywords: vec![Keyword::Flying, Keyword::Vigilance],
                pt: Some((4, 4)),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Entropic Battlecruiser — {3}{B} Artifact — Spacecraft. Whenever it attacks,
/// each opponent discards a card. ({1+}: whenever an opponent discards a card,
/// they lose 3 life. {8+}: 3/10 with flying, deathtouch.) The attack rider's
/// "each opponent who can't loses 3 life" empty-hand branch is approximated.
pub fn entropic_battlecruiser() -> CardDefinition {
    CardDefinition {
        name: "Entropic Battlecruiser",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![on_attack(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: false,
        })],
        activated_abilities: vec![station()],
        station: vec![
            StationBand {
                min: 1,
                triggers: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl),
                    effect: Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::Const(3),
                    },
                }],
                ..Default::default()
            },
            StationBand {
                min: 8,
                keywords: vec![Keyword::Flying, Keyword::Deathtouch],
                pt: Some((3, 10)),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── EOE batch 5 — commons/uncommons on existing primitives ───────────────────

/// Shared "control two or more tapped creatures" intervening-if for the
/// end-step tap-payoff cycle (Frontline War-Rager, Dawnstrike Vanguard).
fn two_tapped_creatures() -> Predicate {
    Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::Tapped),
        ),
        n: Value::Const(2),
    }
}

/// Frontline War-Rager — {2}{R} 2/3 Kavu Soldier. At your end step, if you
/// control two or more tapped creatures, put a +1/+1 counter on this.
pub fn frontline_war_rager() -> CardDefinition {
    CardDefinition {
        name: "Frontline War-Rager",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: two_tapped_creatures(),
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Dawnstrike Vanguard — {5}{W} 4/5 Human Knight. Lifelink. At your end step,
/// if you control two or more tapped creatures, put a +1/+1 counter on each
/// other creature you control.
pub fn dawnstrike_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Dawnstrike Vanguard",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: two_tapped_creatures(),
                then: Box::new(Effect::AddCounter {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Icecave Crasher — {3}{G} 4/4 Beast. Trample. Landfall — whenever a land you
/// control enters, this creature gets +1/+0 until end of turn.
pub fn icecave_crasher() -> CardDefinition {
    CardDefinition {
        name: "Icecave Crasher",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Land,
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Illvoi Galeblade — {U} 1/1 Jellyfish Warrior. Flash, flying. {2}, Sacrifice
/// this: Draw a card.
pub fn illvoi_galeblade() -> CardDefinition {
    CardDefinition {
        name: "Illvoi Galeblade",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Intrepid Tenderfoot — {1}{G} 2/2 Insect Citizen. {3}: Put a +1/+1 counter on
/// this. Activate only as a sorcery.
pub fn intrepid_tenderfoot() -> CardDefinition {
    CardDefinition {
        name: "Intrepid Tenderfoot",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lightless Evangel — {1}{B} 2/2 Vampire Cleric. Whenever you sacrifice
/// another creature or artifact, put a +1/+1 counter on this.
pub fn lightless_evangel() -> CardDefinition {
    CardDefinition {
        name: "Lightless Evangel",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .or(SelectionRequirement::Artifact)
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Honored Knight-Captain — {1}{W} 1/1 Human Advisor Knight. ETB: create a 1/1
/// white Human Soldier token. {4}{W}{W}, Sacrifice this: Search your library
/// for an Equipment card, put it onto the battlefield, then shuffle.
pub fn honored_knight_captain() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Honored Knight-Captain",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: soldier,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w(), w()]),
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Harmonious Grovestrider — {3}{G}{G} */* Beast. Ward {2}. Its power and
/// toughness are each equal to the number of lands you control.
pub fn harmonious_grovestrider() -> CardDefinition {
    CardDefinition {
        name: "Harmonious Grovestrider",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        dynamic_pt: Some(DynamicPt::LandsControlled { base: 0 }),
        ..Default::default()
    }
}

// ── EOE batch 6 — removal + utility ──────────────────────────────────────────

/// Gravkill — {3}{B} Instant. Exile target creature or Spacecraft.
pub fn gravkill() -> CardDefinition {
    CardDefinition {
        name: "Gravkill",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Spacecraft)),
            ),
        },
        ..Default::default()
    }
}

/// Depressurize — {1}{B} Instant. Target creature gets -3/-0 until end of turn.
/// Then if that creature's power is 0 or less, destroy it.
pub fn depressurize() -> CardDefinition {
    CardDefinition {
        name: "Depressurize",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::PowerAtMost(0),
                },
                then: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Invasive Maneuvers — {1}{R} Instant. Deals 3 damage to target creature — 5
/// instead if you control a Spacecraft.
pub fn invasive_maneuvers() -> CardDefinition {
    CardDefinition {
        name: "Invasive Maneuvers",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            amount: Value::IfAtLeast {
                value: Box::new(Value::count(Selector::EachPermanent(
                    SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Spacecraft)
                        .and(SelectionRequirement::ControlledByYou),
                ))),
                threshold: 1,
                then: Box::new(Value::Const(5)),
                else_: Box::new(Value::Const(3)),
            },
            to: target_filtered(SelectionRequirement::Creature),
        },
        ..Default::default()
    }
}

/// Gravpack Monoist — {2}{B} 2/1 Human Scout. Flying. When this dies, create a
/// tapped 2/2 colorless Robot artifact creature token.
pub fn gravpack_monoist() -> CardDefinition {
    let robot = TokenDefinition {
        name: "Robot".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        tapped: true,
        ..Default::default()
    };
    CardDefinition {
        name: "Gravpack Monoist",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: robot,
        })],
        ..Default::default()
    }
}

/// Gene Pollinator — {G} 1/2 Artifact Creature — Robot Insect. {T}, Tap an
/// untapped permanent you control: Add one mana of any color.
pub fn gene_pollinator() -> CardDefinition {
    CardDefinition {
        name: "Gene Pollinator",
        cost: cost(&[g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            tap_other_filter: Some(SelectionRequirement::Permanent),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── EOE batch 7 — creatures + a Food ─────────────────────────────────────────

/// Hullcarver — {B} 1/1 Artifact Creature — Robot Assassin. Deathtouch.
pub fn hullcarver() -> CardDefinition {
    CardDefinition {
        name: "Hullcarver",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Assassin],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Kavaron Turbodrone — {2}{R} 2/3 Artifact Creature — Robot Scout. {T}: Target
/// creature you control gets +1/+1 and gains haste until end of turn. Sorcery
/// speed.
pub fn kavaron_turbodrone() -> CardDefinition {
    CardDefinition {
        name: "Kavaron Turbodrone",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Illvoi Operative — {1}{U} 2/1 Jellyfish Rogue. Whenever you cast your second
/// spell each turn, put a +1/+1 counter on this.
pub fn illvoi_operative() -> CardDefinition {
    CardDefinition {
        name: "Illvoi Operative",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Comet Crawler — {2}{B} 2/3 Insect Horror. Lifelink. Whenever this attacks,
/// you may sacrifice another creature or artifact; if you do, it gets +2/+0
/// until end of turn.
pub fn comet_crawler() -> CardDefinition {
    CardDefinition {
        name: "Comet Crawler",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MaySacrifice {
                description: "Sacrifice another creature or artifact".into(),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Artifact)
                    .and(SelectionRequirement::OtherThanSource),
                count: Value::Const(1),
                then: Box::new(Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Kavaron Harrier — {R} 2/1 Artifact Creature — Robot Soldier. Whenever this
/// attacks, you may pay {2}. If you do, create a 2/2 colorless Robot artifact
/// creature token that's tapped and attacking; sacrifice it at end of combat.
pub fn kavaron_harrier() -> CardDefinition {
    let robot = TokenDefinition {
        name: "Robot".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Kavaron Harrier",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {2} to make a Robot".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: robot,
                    cleanup: crate::effect::AttackingTokenCleanup::SacrificeAtEndOfCombat,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Dubious Delicacy — {2}{B} Artifact — Food. Flash. ETB: up to one target
/// creature gets -3/-3 until end of turn. {2},{T},Sacrifice: gain 3 life. Or
/// {2},{T},Sacrifice: target opponent loses 3 life.
pub fn dubious_delicacy() -> CardDefinition {
    let sac_ability = |effect: Effect| ActivatedAbility {
        tap_cost: true,
        sac_cost: true,
        mana_cost: cost(&[generic(2)]),
        effect,
        ..Default::default()
    };
    CardDefinition {
        name: "Dubious Delicacy",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Food], ..Default::default() },
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            }),
        })],
        activated_abilities: vec![
            sac_ability(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
            sac_ability(Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(3),
            }),
        ],
        ..Default::default()
    }
}

// ── EOE batch 8 — burn, landfall, robots ─────────────────────────────────────

/// Nebula Dragon — {6}{R} 4/4 Dragon. Flying. ETB: deals 3 damage to any target.
pub fn nebula_dragon() -> CardDefinition {
    CardDefinition {
        name: "Nebula Dragon",
        cost: cost(&[generic(6), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::DealDamage {
            amount: Value::Const(3),
            to: target_any(),
        })],
        ..Default::default()
    }
}

/// Plasma Bolt — {R} Sorcery. Deals 2 damage to any target — 3 instead if Void
/// is active (a nonland permanent left the battlefield, or a spell was warped,
/// this turn).
pub fn plasma_bolt() -> CardDefinition {
    CardDefinition {
        name: "Plasma Bolt",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { amount: Value::Const(2), to: target_any() },
            Effect::If {
                cond: Predicate::VoidActive { who: PlayerRef::You },
                then: Box::new(Effect::DealDamage {
                    amount: Value::Const(1),
                    to: Selector::Target(0),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Remnant Elemental — {1}{R} 0/4 Elemental. Reach. Landfall — whenever a land
/// you control enters, this creature gets +2/+0 until end of turn.
pub fn remnant_elemental() -> CardDefinition {
    CardDefinition {
        name: "Remnant Elemental",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Land,
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Slagdrill Scrapper — {R} 1/2 Artifact Creature — Robot Scout. {2}, {T},
/// Sacrifice another artifact or land: Draw a card.
pub fn slagdrill_scrapper() -> CardDefinition {
    CardDefinition {
        name: "Slagdrill Scrapper",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((
                SelectionRequirement::Artifact.or(SelectionRequirement::Land),
                1,
            )),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Seedship Agrarian — {3}{G} 3/3 Insect Scientist. Whenever this becomes
/// tapped, create a Lander token. Landfall — whenever a land you control
/// enters, put a +1/+1 counter on this.
pub fn seedship_agrarian() -> CardDefinition {
    CardDefinition {
        name: "Seedship Agrarian",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Scientist],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: lander_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Land,
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Nutrient Block — {1} Artifact — Food. Indestructible. {2}, {T}, Sacrifice
/// this: You gain 3 life. When it's put into a graveyard from the battlefield,
/// draw a card.
pub fn nutrient_block() -> CardDefinition {
    CardDefinition {
        name: "Nutrient Block",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Food], ..Default::default() },
        keywords: vec![Keyword::Indestructible],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Oreplate Pangolin — {1}{R} 2/2 Artifact Creature — Robot Pangolin. Whenever
/// another artifact you control enters, you may pay {1}; if you do, put a +1/+1
/// counter on this.
pub fn oreplate_pangolin() -> CardDefinition {
    CardDefinition {
        name: "Oreplate Pangolin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Pangolin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::MayPay {
                description: "Pay {1} for a +1/+1 counter".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Rayblade Trooper — {2}{W} 2/2 Human Soldier. ETB: put a +1/+1 counter on
/// target creature you control. Whenever a nontoken creature you control with a
/// +1/+1 counter on it dies, create a 1/1 white Human Soldier token. Warp
/// {1}{W}.
pub fn rayblade_trooper() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Rayblade Trooper",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)
                            .and(SelectionRequirement::Not(Box::new(SelectionRequirement::IsToken))),
                    }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: soldier,
                },
            },
        ],
        alternative_cost: Some(warp(cost(&[generic(1), w()]))),
        ..Default::default()
    }
}

// ── EOE batch 9 — new primitives (uncounterable/unpreventable statics) ────────

/// Frenzied Baloth — {G}{G} Creature — Beast 3/2, trample, haste. This spell
/// can't be countered. Creature spells you control can't be countered. Combat
/// damage can't be prevented.
pub fn frenzied_baloth() -> CardDefinition {
    CardDefinition {
        name: "Frenzied Baloth",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::CantBeCountered],
        static_abilities: vec![
            StaticAbility {
                description: "Creature spells you control can't be countered.",
                effect: StaticEffect::SpellsUncounterable { filter: SelectionRequirement::Creature },
            },
            StaticAbility {
                description: "Combat damage can't be prevented.",
                effect: StaticEffect::CombatDamageCantBePrevented,
            },
        ],
        ..Default::default()
    }
}

/// Gravblade Heavy — {3}{B} Creature — Human Soldier 3/4. As long as you control
/// an artifact, this creature gets +1/+0 and has deathtouch.
pub fn gravblade_heavy() -> CardDefinition {
    CardDefinition {
        name: "Gravblade Heavy",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "As long as you control an artifact, this gets +1/+0 and has deathtouch.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(1),
                },
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Deathtouch],
            },
        }],
        ..Default::default()
    }
}

/// Skystinger — {2}{G} Creature — Insect Warrior 3/3, reach. Whenever it blocks
/// a creature with flying, it gets +5/+0 until end of turn.
pub fn skystinger() -> CardDefinition {
    CardDefinition {
        name: "Skystinger",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::BlockedAttacker,
                    filter: SelectionRequirement::HasKeyword(Keyword::Flying),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(5),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

// ── EOE batch 10 — commons/uncommons on existing primitives ──────────────────

/// Honor — {W} Sorcery. Put a +1/+1 counter on target creature. Draw a card.
pub fn honor() -> CardDefinition {
    CardDefinition {
        name: "Honor",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Radiant Strike — {3}{W} Instant. Destroy target artifact or tapped creature.
/// You gain 3 life.
pub fn radiant_strike() -> CardDefinition {
    CardDefinition {
        name: "Radiant Strike",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature.and(SelectionRequirement::Tapped)),
                ),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Rig for War — {1}{R} Instant. Target creature gets +3/+0 and gains first
/// strike and reach until end of turn.
pub fn rig_for_war() -> CardDefinition {
    CardDefinition {
        name: "Rig for War",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Mechan Navigator — {1}{U} Artifact Creature — Robot Pilot 2/1. Whenever it
/// becomes tapped, draw a card, then discard a card.
pub fn mechan_navigator() -> CardDefinition {
    CardDefinition {
        name: "Mechan Navigator",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ]),
        }],
        ..Default::default()
    }
}

/// Monoist Sentry — {B} Artifact Creature — Robot 4/1. Defender.
pub fn monoist_sentry() -> CardDefinition {
    CardDefinition {
        name: "Monoist Sentry",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 4,
        toughness: 1,
        keywords: vec![Keyword::Defender],
        ..Default::default()
    }
}

/// Red Tiger Mechan — {3}{R} Artifact Creature — Robot Cat 3/3, haste.
/// Warp {1}{R}.
pub fn red_tiger_mechan() -> CardDefinition {
    CardDefinition {
        name: "Red Tiger Mechan",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Cat],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        alternative_cost: Some(warp(cost(&[generic(1), r()]))),
        ..Default::default()
    }
}

/// Flight-Deck Coordinator — {2}{W} Creature — Human Soldier 3/3. At your end
/// step, if you control two or more tapped creatures, you gain 2 life.
pub fn flight_deck_coordinator() -> CardDefinition {
    CardDefinition {
        name: "Flight-Deck Coordinator",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: two_tapped_creatures(),
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Luxknight Breacher — {3}{W} Creature — Human Knight 2/2. Enters with a +1/+1
/// counter for each other creature and/or artifact you control.
pub fn luxknight_breacher() -> CardDefinition {
    CardDefinition {
        name: "Luxknight Breacher",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::count(Selector::EachPermanent(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Artifact)
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            )),
        )),
        ..Default::default()
    }
}

/// Molecular Modifier — {2}{R} Creature — Kavu Artificer 2/2. At the beginning
/// of combat on your turn, target creature you control gets +1/+0 and gains
/// first strike until end of turn.
pub fn molecular_modifier() -> CardDefinition {
    CardDefinition {
        name: "Molecular Modifier",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Diplomatic Relations — {2}{G} Instant. Target creature you control gets
/// +1/+0 and gains vigilance until end of turn. It deals damage equal to its
/// power to target creature an opponent controls.
pub fn diplomatic_relations() -> CardDefinition {
    CardDefinition {
        name: "Diplomatic Relations",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Cut Propulsion — {2}{R} Instant. Target creature deals damage to itself equal
/// to its power; twice that much instead if it has flying.
pub fn cut_propulsion() -> CardDefinition {
    CardDefinition {
        name: "Cut Propulsion",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::HasKeyword(Keyword::Flying),
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Mechan Shieldmate — {1}{U} Artifact Creature — Robot Soldier 3/2, defender.
pub fn mechan_shieldmate() -> CardDefinition {
    CardDefinition {
        name: "Mechan Shieldmate",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Defender],
        ..Default::default()
    }
}

// ── EOE batch 11 — cost-reduction, Lander, and utility cards ──────────────────

/// Gigastorm Titan — {4}{U} Creature — Elemental 4/4. Costs {3} less if you've
/// cast another spell this turn.
pub fn gigastorm_titan() -> CardDefinition {
    CardDefinition {
        name: "Gigastorm Titan",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 4,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Costs {3} less if you've cast another spell this turn.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::SpellsCastThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(1) },
                amount: 3,
            },
        }],
        ..Default::default()
    }
}

/// Lashwhip Predator — {4}{G}{G} Creature — Plant Beast 5/7, reach. Costs {2}
/// less if your opponents control three or more creatures.
pub fn lashwhip_predator() -> CardDefinition {
    CardDefinition {
        name: "Lashwhip Predator",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant, CreatureType::Beast], ..Default::default() },
        power: 5,
        toughness: 7,
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Costs {2} less if your opponents control three or more creatures.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    n: Value::Const(3),
                },
                amount: 2,
            },
        }],
        ..Default::default()
    }
}

/// Cerebral Download — {4}{U} Instant. Surveil X (X = artifacts you control),
/// then draw three cards.
pub fn cerebral_download() -> CardDefinition {
    CardDefinition {
        name: "Cerebral Download",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                )),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Sami's Curiosity — {G} Sorcery. You gain 2 life. Create a Lander token.
pub fn samis_curiosity() -> CardDefinition {
    CardDefinition {
        name: "Sami's Curiosity",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: lander_token() },
        ]),
        ..Default::default()
    }
}

/// Lithobraking — {2}{R} Instant. Create a Lander token. Then you may sacrifice
/// an artifact; if you do, Lithobraking deals 2 damage to each creature.
pub fn lithobraking() -> CardDefinition {
    CardDefinition {
        name: "Lithobraking",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: lander_token() },
            Effect::MaySacrifice {
                description: "Sacrifice an artifact".into(),
                filter: SelectionRequirement::Artifact,
                count: Value::Const(1),
                then: Box::new(Effect::DealDamage {
                    to: Selector::EachPermanent(SelectionRequirement::Creature),
                    amount: Value::Const(2),
                }),
                else_: None,
            },
        ]),
        ..Default::default()
    }
}

/// Rust Harvester — {R} Artifact Creature — Robot 1/1, menace. {2}, {T}, Exile
/// an artifact card from your graveyard: Put a +1/+1 counter on this, then it
/// deals damage equal to its power to any target.
pub fn rust_harvester() -> CardDefinition {
    CardDefinition {
        name: "Rust Harvester",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            exile_other_filter: Some((SelectionRequirement::Artifact, 1)),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::DealDamage {
                    to: target_any(),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nanoform Sentinel — {2}{U} Artifact Creature — Robot 3/2. Whenever it becomes
/// tapped, untap another target permanent. Triggers only once each turn.
pub fn nanoform_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Nanoform Sentinel",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource).once_per_turn(),
            effect: Effect::Untap {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Permanent
                        .and(SelectionRequirement::OtherThanSource),
                },
                up_to: None,
            },
        }],
        ..Default::default()
    }
}

// ── EOE batch 12 — artifact-matters, modal, and sacrifice-cost cards ──────────

/// 2/2 colorless Robot artifact creature token.
fn robot_2_2() -> TokenDefinition {
    TokenDefinition {
        name: "Robot".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    }
}

/// Mechan Assembler — {4}{U} Artifact Creature — Robot Artificer 4/4. Whenever
/// another artifact you control enters, create a 2/2 Robot. Once each turn.
pub fn mechan_assembler() -> CardDefinition {
    CardDefinition {
        name: "Mechan Assembler",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Artificer],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                })
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: robot_2_2(),
            },
        }],
        ..Default::default()
    }
}

/// Mm'menon, Uthros Exile — {1}{U}{R} Legendary Creature — Jellyfish Advisor
/// 1/3, flying. Whenever an artifact you control enters, put a +1/+1 counter on
/// target creature.
pub fn mmmenon_uthros_exile() -> CardDefinition {
    CardDefinition {
        name: "Mm'menon, Uthros Exile",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish, CreatureType::Advisor],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact,
                }),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Embrace Oblivion — {B} Sorcery. Additional cost: sacrifice an artifact or
/// creature. Destroy target creature or Spacecraft.
pub fn embrace_oblivion() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    CardDefinition {
        name: "Embrace Oblivion",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            count: 1,
        }],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Spacecraft)),
            ),
        },
        ..Default::default()
    }
}

/// Scrounge for Eternity — {2}{B} Sorcery. Additional cost: sacrifice an artifact
/// or creature. Return target creature or Spacecraft card with mana value 5 or
/// less from your graveyard to the battlefield. Then create a Lander token.
pub fn scrounge_for_eternity() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    CardDefinition {
        name: "Scrounge for Eternity",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            count: 1,
        }],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::InYourGraveyard
                        .and(
                            SelectionRequirement::Creature.or(SelectionRequirement::HasArtifactSubtype(
                                ArtifactSubtype::Spacecraft,
                            )),
                        )
                        .and(SelectionRequirement::ManaValueAtMost(5)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: lander_token() },
        ]),
        ..Default::default()
    }
}

/// Ruinous Rampage — {1}{R}{R} Sorcery. Choose one — deal 3 damage to each
/// opponent; or exile all artifacts with mana value 3 or less.
pub fn ruinous_rampage() -> CardDefinition {
    CardDefinition {
        name: "Ruinous Rampage",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
            Effect::Move {
                what: Selector::EachPermanent(
                    SelectionRequirement::Artifact.and(SelectionRequirement::ManaValueAtMost(3)),
                ),
                to: ZoneDest::Exile,
            },
        ]),
        ..Default::default()
    }
}

/// Drill Too Deep — {1}{R} Instant. Choose one — put five charge counters on
/// target Spacecraft you control; or destroy target artifact.
pub fn drill_too_deep() -> CardDefinition {
    CardDefinition {
        name: "Drill Too Deep",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Spacecraft)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::Charge,
                amount: Value::Const(5),
            },
            Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
        ]),
        ..Default::default()
    }
}

/// Reroute Systems — {W} Instant. Choose one — target artifact or creature gains
/// indestructible until end of turn; or deal 2 damage to target tapped creature.
pub fn reroute_systems() -> CardDefinition {
    CardDefinition {
        name: "Reroute Systems",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                ),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
                ),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

// ── EOE batch 13 — keyword bodies + small triggers ───────────────────────────

/// Mouth of the Storm — {6}{U} Creature — Elemental 6/6, flying, ward {2}. When
/// it enters, creatures your opponents control get -3/-0 until your next turn.
pub fn mouth_of_the_storm() -> CardDefinition {
    CardDefinition {
        name: "Mouth of the Storm",
        cost: cost(&[generic(6), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-3),
            toughness: Value::Const(0),
            duration: Duration::UntilNextTurn,
        })],
        ..Default::default()
    }
}

/// Chrome Companion — {2} Artifact Creature — Dog 2/1. Whenever it becomes
/// tapped, you gain 1 life. {2}, {T}: Put target card from a graveyard on the
/// bottom of its owner's library.
pub fn chrome_companion() -> CardDefinition {
    CardDefinition {
        name: "Chrome Companion",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::InGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: crate::effect::LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Meltstrider Eulogist — {2}{G} Creature — Insect Soldier 3/3. Whenever a
/// creature you control with a +1/+1 counter on it dies, draw a card.
pub fn meltstrider_eulogist() -> CardDefinition {
    CardDefinition {
        name: "Meltstrider Eulogist",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Cryogen Relic — {1}{U} Artifact. When it enters or leaves the battlefield,
/// draw a card. {1}{U}, Sacrifice this: Put a stun counter on up to one target
/// tapped creature.
pub fn cryogen_relic() -> CardDefinition {
    CardDefinition {
        name: "Cryogen Relic",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            sac_cost: true,
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::Tapped),
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::Const(1),
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── EOE batch 14 — pump / landfall / warp riders ─────────────────────────────

/// Hemosymbic Mite — {G} Creature — Mite 1/1. Whenever it becomes tapped,
/// another target creature you control gets +X/+X until end of turn, where X is
/// this creature's power.
pub fn hemosymbic_mite() -> CardDefinition {
    CardDefinition {
        name: "Hemosymbic Mite",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mite], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::PowerOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Genemorph Imago — {G}{U} Creature — Insect Druid 1/3, flying. Landfall —
/// whenever a land you control enters, target creature has base power and
/// toughness 3/3 until end of turn. (The 6+-lands 5/5 upgrade is approximated to
/// the 3/3 set.)
/// 5 if you control six or more lands, else 3 — Genemorph Imago's landfall
/// base-P/T set.
fn six_lands_5_else_3() -> Value {
    Value::IfPred {
        pred: Box::new(Predicate::SelectorCountAtLeast {
            sel: Selector::EachPermanent(
                SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            ),
            n: Value::Const(6),
        }),
        then: Box::new(Value::Const(5)),
        else_: Box::new(Value::Const(3)),
    }
}

pub fn genemorph_imago() -> CardDefinition {
    CardDefinition {
        name: "Genemorph Imago",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Land,
                }),
            // Base 3/3, or 5/5 instead if you control six or more lands.
            effect: Effect::SetBasePT {
                what: target_filtered(SelectionRequirement::Creature),
                power: six_lands_5_else_3(),
                toughness: six_lands_5_else_3(),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Full Bore — {R} Instant. Target creature you control gets +3/+2 until end of
/// turn; if it was cast for its warp cost, it also gains trample and haste.
pub fn full_bore() -> CardDefinition {
    CardDefinition {
        name: "Full Bore",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(3),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::Warped,
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Emissary Escort — {1}{U} Artifact Creature — Robot Soldier 0/4. Gets +X/+0,
/// where X is the greatest mana value among other artifacts you control.
pub fn emissary_escort() -> CardDefinition {
    CardDefinition {
        name: "Emissary Escort",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Soldier],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        dynamic_pt: Some(DynamicPt::BasePlusGreatestOtherArtifactMv { base_p: 0, base_t: 4 }),
        ..Default::default()
    }
}

/// Fungal Colossus — {6}{G} Creature — Fungus Beast 5/5. Costs {X} less to
/// cast, where X is the number of differently named lands you control.
pub fn fungal_colossus() -> CardDefinition {
    CardDefinition {
        name: "Fungal Colossus",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fungus, CreatureType::Beast],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is the number of differently named lands you control.",
            effect: StaticEffect::SelfCostReducedByDistinctLandNames,
        }],
        ..Default::default()
    }
}

/// Dark Endurance — {1}{B} Instant. Costs {1} less if it targets a blocking
/// creature. Target creature gets +2/+0 and gains indestructible until end of
/// turn.
pub fn dark_endurance() -> CardDefinition {
    CardDefinition {
        name: "Dark Endurance",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((SelectionRequirement::IsBlocking, 1)),
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Shattered Wings — {2}{G} Sorcery. Destroy target artifact, enchantment, or
/// creature with flying. Surveil 1.
pub fn shattered_wings() -> CardDefinition {
    CardDefinition {
        name: "Shattered Wings",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Enchantment)
                        .or(SelectionRequirement::Creature
                            .and(SelectionRequirement::HasKeyword(Keyword::Flying))),
                ),
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Seam Rip — {W} Enchantment. ETB: exile target nonland permanent an opponent
/// controls with mana value 2 or less until this enchantment leaves.
pub fn seam_rip() -> CardDefinition {
    CardDefinition {
        name: "Seam Rip",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::Nonland
                    .and(SelectionRequirement::ControlledByOpponent)
                    .and(SelectionRequirement::ManaValueAtMost(2)),
            ),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Seedship Impact — {1}{G} Instant. Destroy target artifact or enchantment. If
/// its mana value was 2 or less, create a Lander token.
pub fn seedship_impact() -> CardDefinition {
    CardDefinition {
        name: "Seedship Impact",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            // Evaluate the target's mana value before destroying it.
            cond: Predicate::ValueAtMost(
                Value::ManaValueOf(Box::new(Selector::Target(0))),
                Value::Const(2),
            ),
            then: Box::new(Effect::Seq(vec![
                Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                    ),
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: lander_token() },
            ])),
            else_: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        },
        ..Default::default()
    }
}

/// Desculpting Blast — {1}{U} Instant. Return target nonland permanent to its
/// owner's hand. If it was attacking, create a 1/1 colorless Drone artifact
/// creature token with flying.
pub fn desculpting_blast() -> CardDefinition {
    let drone = TokenDefinition {
        name: "Drone".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Drone], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Desculpting Blast",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        // Check "was attacking" before the bounce removes it from combat.
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::IsAttacking,
                },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: drone,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Nonland),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        ]),
        ..Default::default()
    }
}

/// Lost in Space — {3}{U} Instant. Target artifact or creature's owner puts it
/// on their choice of the top or bottom of their library. Surveil 1.
pub fn lost_in_space() -> CardDefinition {
    CardDefinition {
        name: "Lost in Space",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                ),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: crate::effect::LibraryPosition::OwnerChoice,
                },
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Sinister Cryologist — {2}{U} Creature — Jellyfish Wizard 2/3. ETB: target
/// creature an opponent controls gets -3/-0 until end of turn. Warp {U}.
pub fn sinister_cryologist() -> CardDefinition {
    CardDefinition {
        name: "Sinister Cryologist",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-3),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        alternative_cost: Some(warp(cost(&[u()]))),
        ..Default::default()
    }
}

/// Orbital Plunge — {3}{R} Sorcery. Deals 6 damage to target creature. If excess
/// damage was dealt this way (CR 120.10), create a Lander token.
pub fn orbital_plunge() -> CardDefinition {
    CardDefinition {
        name: "Orbital Plunge",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(6),
            },
            Effect::If {
                cond: Predicate::ExcessDamageDealtThisResolution,
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: lander_token(),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Anticausal Vestige — {6} Creature — Eldrazi 7/5. When it leaves the
/// battlefield, draw a card, then you may put a permanent card with mana value
/// ≤ the number of lands you control from your hand onto the battlefield tapped.
/// Warp {4}.
pub fn anticausal_vestige() -> CardDefinition {
    CardDefinition {
        name: "Anticausal Vestige",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 7,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Permanent.and(
                        SelectionRequirement::ManaValueAtMostYourCount(Box::new(
                            SelectionRequirement::Land,
                        )),
                    ),
                    count: Value::Const(1),
                    tapped: true,
                    haste: false,
                    sacrifice_eot: false,
                },
            ]),
        }],
        alternative_cost: Some(warp(cost(&[generic(4)]))),
        ..Default::default()
    }
}

/// Faller's Faithful — {2}{B} Creature — Human Wizard 3/1. ETB: destroy up to
/// one other target creature; if it wasn't dealt damage this turn, its
/// controller draws two cards.
pub fn fallers_faithful() -> CardDefinition {
    CardDefinition {
        name: "Faller's Faithful",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            // Check the damage state before the creature is destroyed.
            Effect::If {
                cond: Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::DealtDamageThisTurn,
                })),
                then: Box::new(Effect::Draw {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                ),
            },
        ]))],
        ..Default::default()
    }
}

/// Selfcraft Mechan — {3}{U} Artifact Creature — Robot Artificer 3/4. ETB: you
/// may sacrifice an artifact; if you do, put a +1/+1 counter on target creature
/// and draw a card.
pub fn selfcraft_mechan() -> CardDefinition {
    CardDefinition {
        name: "Selfcraft Mechan",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice an artifact?".into(),
            filter: SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
            count: Value::Const(1),
            then: Box::new(Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ])),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Cosmogrand Zenith — {2}{W} Creature — Human Soldier 2/4. Whenever you cast
/// your second spell each turn, choose one — create two 1/1 white Soldiers; or
/// put a +1/+1 counter on each creature you control.
pub fn cosmogrand_zenith() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        colors: vec![crate::mana::Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Cosmogrand Zenith",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![flurry(Effect::ChooseMode(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: soldier },
            Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Seedship Broodtender — {B}{G} Creature — Insect Citizen 2/3. ETB: mill three.
/// {3}{B}{G}, Sacrifice this: return target creature or Spacecraft card from
/// your graveyard to the battlefield. Sorcery speed.
pub fn seedship_broodtender() -> CardDefinition {
    CardDefinition {
        name: "Seedship Broodtender",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Mill { who: Selector::You, amount: Value::Const(3) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b(), g()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::InYourGraveyard.and(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Spacecraft)),
                    ),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Virus Beetle — {1}{B} Artifact Creature — Insect 1/1. ETB: each opponent
/// discards a card.
pub fn virus_beetle() -> CardDefinition {
    CardDefinition {
        name: "Virus Beetle",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: false,
        })],
        ..Default::default()
    }
}

/// Tragic Trajectory — {B} Sorcery. Target creature gets -2/-2; Void — -10/-10
/// instead if a nonland permanent left or a spell was warped this turn.
pub fn tragic_trajectory() -> CardDefinition {
    CardDefinition {
        name: "Tragic Trajectory",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::VoidActive { who: PlayerRef::You },
            then: Box::new(Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-10),
                toughness: Value::Const(-10),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Sunstar Expansionist — {1}{W} Creature — Human Knight 2/3. ETB: if an opponent
/// controls more lands than you, create a Lander. Landfall: +1/+0 until EOT.
pub fn sunstar_expansionist() -> CardDefinition {
    CardDefinition {
        name: "Sunstar Expansionist",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::OpponentControlsMoreLandsThanYou,
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: lander_token(),
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Land,
                    }),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Sunstar Lightsmith — {3}{W} Creature — Human Artificer 3/3. Whenever you cast
/// your second spell each turn, put a +1/+1 counter on this and draw a card.
pub fn sunstar_lightsmith() -> CardDefinition {
    CardDefinition {
        name: "Sunstar Lightsmith",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![flurry(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Uthros Psionicist — {2}{U} Creature — Jellyfish Scientist 2/4. The second
/// spell you cast each turn costs {2} less.
pub fn uthros_psionicist() -> CardDefinition {
    CardDefinition {
        name: "Uthros Psionicist",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish, CreatureType::Scientist],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "The second spell you cast each turn costs {2} less to cast.",
            effect: StaticEffect::CostReductionNthSpell {
                filter: SelectionRequirement::Any,
                nth: 2,
                amount: 2,
            },
        }],
        ..Default::default()
    }
}

/// Zealous Display — {2}{W} Instant. Creatures you control get +2/+0 until end of
/// turn. If it's not your turn, untap those creatures.
pub fn zealous_display() -> CardDefinition {
    let my_creatures = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Zealous Display",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: my_creatures(),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
                then: Box::new(Effect::Untap { what: my_creatures(), up_to: None }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Thawbringer — {2}{G} Creature — Insect Scout 4/2. When it enters or dies,
/// surveil 1.
pub fn thawbringer() -> CardDefinition {
    let surveil = || Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) };
    CardDefinition {
        name: "Thawbringer",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Scout],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![etb(surveil()), on_dies(surveil())],
        ..Default::default()
    }
}

/// Susurian Voidborn — {2}{B} Creature — Vampire Soldier 2/2. Whenever this or
/// another creature you control dies, target opponent loses 1 life and you gain
/// 1. Warp {B}. (The printed "or artifact" branch is approximated to creatures.)
pub fn susurian_voidborn() -> CardDefinition {
    CardDefinition {
        name: "Susurian Voidborn",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            ]),
        }],
        alternative_cost: Some(warp(cost(&[b()]))),
        ..Default::default()
    }
}

/// Mental Modulation — {1}{U} Instant. Costs {1} less during your turn. Tap
/// target artifact or creature, then draw a card.
pub fn mental_modulation() -> CardDefinition {
    CardDefinition {
        name: "Mental Modulation",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast during your turn.",
            effect: StaticEffect::SelfCostReducedDuringYourTurn { amount: 1 },
        }],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                ),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Weftstalker Ardent — {2}{R} Creature — Drix Artificer 2/3. Whenever another
/// creature or artifact you control enters, deal 1 to each opponent. Warp {R}.
pub fn weftstalker_ardent() -> CardDefinition {
    CardDefinition {
        name: "Weftstalker Ardent",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drix, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        alternative_cost: Some(warp(cost(&[r()]))),
        ..Default::default()
    }
}

/// Weftblade Enhancer — {5}{W} Creature — Drix Artificer 3/4. ETB: put a +1/+1
/// counter on each of up to two target creatures. Warp {2}{W}.
pub fn weftblade_enhancer() -> CardDefinition {
    CardDefinition {
        name: "Weftblade Enhancer",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drix, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        alternative_cost: Some(warp(cost(&[generic(2), w()]))),
        ..Default::default()
    }
}

/// Swarm Culler — {3}{B} Creature — Insect Warrior 2/4, flying. Whenever it
/// becomes tapped, you may sacrifice another creature or artifact; if you do,
/// draw a card.
pub fn swarm_culler() -> CardDefinition {
    CardDefinition {
        name: "Swarm Culler",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::MaySacrifice {
                description: "Sacrifice another creature or artifact?".into(),
                filter: (SelectionRequirement::Creature.or(SelectionRequirement::Artifact))
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
                count: Value::Const(1),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Sunstar Chaplain — {1}{W} Creature — Human Cleric 3/2. At your end step, if
/// you control two or more tapped creatures, put a +1/+1 counter on target
/// creature you control. {2}, Remove a +1/+1 counter from a creature you
/// control: Tap target artifact or creature.
pub fn sunstar_chaplain() -> CardDefinition {
    CardDefinition {
        name: "Sunstar Chaplain",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: two_tapped_creatures(),
                then: Box::new(Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Zookeeper Mechan — {1}{R} Artifact Creature — Robot 1/3. {T}: Add {R}.
/// {6}{R}: Target creature you control gets +4/+0 until end of turn. Sorcery
/// speed.
pub fn zookeeper_mechan() -> CardDefinition {
    CardDefinition {
        name: "Zookeeper Mechan",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 1,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::OfColor(crate::mana::Color::Red, Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(6), r()]),
                sorcery_speed: true,
                effect: Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(4),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Vaultguard Trooper — {4}{R} Creature — Kavu Soldier 5/5. At your end step, if
/// you control two or more tapped creatures, you may discard your hand; if you
/// do, draw two cards.
pub fn vaultguard_trooper() -> CardDefinition {
    CardDefinition {
        name: "Vaultguard Trooper",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu, CreatureType::Soldier],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: two_tapped_creatures(),
                then: Box::new(Effect::MayDo {
                    description: "Discard your hand to draw two?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard {
                            who: Selector::You,
                            amount: Value::Const(99),
                            random: false,
                        },
                        Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Terrapact Intimidator — {1}{R} Creature — Kavu Scout 2/1. ETB: target opponent
/// may have you create two Lander tokens; if they don't, put two +1/+1 counters
/// on this creature.
pub fn terrapact_intimidator() -> CardDefinition {
    CardDefinition {
        name: "Terrapact Intimidator",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::VillainousChoice {
            who: Selector::Player(PlayerRef::EachOpponent),
            option_a: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: lander_token(),
            }),
            option_b: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            }),
        })],
        ..Default::default()
    }
}

/// Voidforged Titan — {4}{B} Artifact Creature — Robot Warrior 5/4. Void — At
/// your end step, if a nonland permanent left the battlefield this turn or a
/// spell was warped this turn, draw a card and lose 1 life.
pub fn voidforged_titan() -> CardDefinition {
    CardDefinition {
        name: "Voidforged Titan",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::VoidActive { who: PlayerRef::You },
                then: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Meltstrider's Gear — {G} Artifact — Equipment. ETB: attach to target creature
/// you control. Equipped creature gets +2/+1 and has reach. Equip {5}.
pub fn meltstriders_gear() -> CardDefinition {
    CardDefinition {
        name: "Meltstrider's Gear",
        cost: cost(&[g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(5)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 2,
            toughness: 1,
            keywords: vec![Keyword::Reach],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
        })],
        ..Default::default()
    }
}

/// Illvoi Light Jammer — {1}{U} Artifact — Equipment. Flash. ETB: attach to
/// target creature you control; it gains hexproof until end of turn. Equipped
/// creature gets +1/+2. Equip {3}.
pub fn illvoi_light_jammer() -> CardDefinition {
    CardDefinition {
        name: "Illvoi Light Jammer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(crate::card::EquipBonus { power: 1, toughness: 2, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Hylderblade — {B} Artifact — Equipment. Equipped creature gets +3/+1. Void —
/// at your end step, if Void is active, attach this to target creature you
/// control. Equip {4}.
pub fn hylderblade() -> CardDefinition {
    CardDefinition {
        name: "Hylderblade",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(crate::card::EquipBonus { power: 3, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::VoidActive { who: PlayerRef::You },
                then: Box::new(Effect::Attach {
                    what: Selector::This,
                    to: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Sami, Ship's Engineer — {2}{R}{W} Legendary Creature — Human Artificer 2/4.
/// At your end step, if you control two or more tapped creatures, create a
/// tapped 2/2 colorless Robot artifact creature token.
pub fn sami_ships_engineer() -> CardDefinition {
    let robot = TokenDefinition {
        name: "Robot".into(),
        power: 2,
        toughness: 2,
        tapped: true,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Sami, Ship's Engineer",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: two_tapped_creatures(),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: robot,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}



/// Starfighter Pilot — {1}{W} Creature — Human Pilot 2/2. Whenever it becomes
/// tapped, surveil 1.
pub fn starfighter_pilot() -> CardDefinition {
    CardDefinition {
        name: "Starfighter Pilot",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Starbreach Whale — {4}{U} Creature — Whale 3/5, flying. ETB: surveil 2.
/// Warp {1}{U}.
pub fn starbreach_whale() -> CardDefinition {
    CardDefinition {
        name: "Starbreach Whale",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Whale], ..Default::default() },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) })],
        alternative_cost: Some(warp(cost(&[generic(1), u()]))),
        ..Default::default()
    }
}

/// Haliya, Ascendant Cadet — {2}{G}{W}{W} Legendary Creature — Human Soldier
/// 3/3. Whenever Haliya enters or attacks, put a +1/+1 counter on target
/// creature you control. (The counter-creatures-deal-damage card-draw rider is
/// approximated away.)
pub fn haliya_ascendant_cadet() -> CardDefinition {
    let counter = || Effect::AddCounter {
        what: target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    };
    CardDefinition {
        name: "Haliya, Ascendant Cadet",
        cost: cost(&[generic(2), g(), w(), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(counter()), on_attack(counter())],
        ..Default::default()
    }
}

// ── Shared helpers ──────────────────────────────────────────────────────────

/// The EOE 2/2 colorless Robot artifact creature token.
fn robot_token() -> TokenDefinition {
    TokenDefinition {
        name: "Robot".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        ..Default::default()
    }
}

// ── Planets (CR 721 Station lands) ──────────────────────────────────────────
//
// "Land — Planet": enters tapped, taps for one color, carries Station. Each
// Planet's 12+ charge-counter activated band is dropped (12 charges is rarely
// reached and each band differs); see TODO.md.

fn eoe_planet(name: &'static str, color: crate::mana::Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Planet], ..Default::default() },
        activated_abilities: vec![super::tap_add(color), station()],
        triggered_abilities: vec![super::etb_tap()],
        ..Default::default()
    }
}

/// A Planet's `12+` Station band that adds `color` mana for each permanent you
/// control matching `filter` (Evendo — per creature; Uthros — per artifact).
fn planet_mana_band(
    mana: crate::mana::ManaCost,
    color: crate::mana::Color,
    filter: SelectionRequirement,
) -> StationBand {
    use crate::effect::ManaPayload;
    StationBand {
        min: 12,
        activated: vec![ActivatedAbility {
            mana_cost: mana,
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    color,
                    Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                        filter,
                    },
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// Adagia/Kavaron's 12+ bands (legendary token-copy / sac-a-land Robot-and-buff)
// are dropped; see TODO.md. Evendo/Uthros/Susur Secundi ride the new activated
// Station band (CR 721.2a).
pub fn adagia_windswept_bastion() -> CardDefinition {
    eoe_planet("Adagia, Windswept Bastion", crate::mana::Color::White)
}
pub fn evendo_waking_haven() -> CardDefinition {
    let mut c = eoe_planet("Evendo, Waking Haven", crate::mana::Color::Green);
    c.station = vec![planet_mana_band(cost(&[g()]), crate::mana::Color::Green, SelectionRequirement::Creature)];
    c
}
pub fn kavaron_memorial_world() -> CardDefinition {
    eoe_planet("Kavaron, Memorial World", crate::mana::Color::Red)
}
pub fn susur_secundi_void_altar() -> CardDefinition {
    let mut c = eoe_planet("Susur Secundi, Void Altar", crate::mana::Color::Black);
    // 12+ | {1}{B}, {T}, Pay 2 life, Sacrifice a creature: Draw cards equal to
    // its power. Sorcery-speed.
    c.station = vec![StationBand {
        min: 12,
        activated: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: true,
            life_cost: 2,
            sorcery_speed: true,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::SacrificedPower },
            ..Default::default()
        }],
        ..Default::default()
    }];
    c
}
pub fn uthros_titanic_godcore() -> CardDefinition {
    let mut c = eoe_planet("Uthros, Titanic Godcore", crate::mana::Color::Blue);
    c.station = vec![planet_mana_band(cost(&[u()]), crate::mana::Color::Blue, SelectionRequirement::Artifact)];
    c
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Pulsar Squadron Ace — {1}{W} 1/2 Human Pilot. ETB: look at the top five, you
/// may reveal a Spacecraft and put it into your hand, rest on the bottom. (The
/// consolation +1/+1 counter when you whiff is dropped.)
pub fn pulsar_squadron_ace() -> CardDefinition {
    CardDefinition {
        name: "Pulsar Squadron Ace",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: false,
            pick_filter: Some(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Spacecraft)),
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Umbral Collar Zealot — {1}{B} 3/2 Human Cleric. Sacrifice another creature or
/// artifact: Surveil 1.
pub fn umbral_collar_zealot() -> CardDefinition {
    CardDefinition {
        name: "Umbral Collar Zealot",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Artifact)
                    .and(SelectionRequirement::OtherThanSource),
                1,
            )),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sunset Saboteur — {1}{B} 4/1 Human Rogue. Menace, Ward—Discard a card.
/// Whenever this attacks, put a +1/+1 counter on target creature an opponent
/// controls.
pub fn sunset_saboteur() -> CardDefinition {
    CardDefinition {
        name: "Sunset Saboteur",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        keywords: vec![Keyword::Menace, Keyword::Ward(WardCost::Discard(1))],
        triggered_abilities: vec![on_attack(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Station Monitor — {W}{U} 2/2 Lizard Artificer. Whenever you cast your second
/// spell each turn, create a 1/1 colorless Drone artifact creature token with
/// flying. (The "can block only flyers" rider on the token is dropped.)
pub fn station_monitor() -> CardDefinition {
    let drone = TokenDefinition {
        name: "Drone".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Drone], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Station Monitor",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::Const(2) },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: drone,
            },
        }],
        ..Default::default()
    }
}

/// Virulent Silencer — {3} 2/3 Artifact Creature — Robot Assassin. Whenever a
/// nontoken artifact creature you control deals combat damage to a player, that
/// player gets two poison counters.
pub fn virulent_silencer() -> CardDefinition {
    CardDefinition {
        name: "Virulent Silencer",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact
                        .and(SelectionRequirement::Creature)
                        .and(SelectionRequirement::NotToken),
                }),
            effect: Effect::AddPoison { who: Selector::Target(0), amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// Steelswarm Operator — {1}{U} 1/1 Artifact Creature — Robot Soldier. Flying;
/// two artifact-restricted mana abilities. (Both modeled as `ArtifactOnly`.)
pub fn steelswarm_operator() -> CardDefinition {
    use crate::effect::ManaPayload;
    let restricted = |n: i32| ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Restricted(
                Box::new(ManaPayload::Colors(vec![crate::mana::Color::Blue; n as usize])),
                crate::mana::SpendRestriction::ArtifactOnly,
            ),
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Steelswarm Operator",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![restricted(1), restricted(2)],
        ..Default::default()
    }
}

/// Syr Vondam, Sunstar Exemplar — {W}{B} 2/2 Legendary Human Knight. Vigilance,
/// menace. Whenever another creature you control dies or is exiled, put a +1/+1
/// counter on Syr Vondam and gain 1 life. When Syr Vondam dies or is exiled with
/// power 4+, destroy up to one target nonland permanent.
pub fn syr_vondam_sunstar_exemplar() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Syr Vondam, Sunstar Exemplar",
        cost: cost(&[w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance, Keyword::Menace],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::OtherThanSource,
                    }),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::This,
                        filter: SelectionRequirement::PowerAtLeast(4),
                    },
                ),
                effect: Effect::Destroy {
                    what: target_filtered(SelectionRequirement::Nonland),
                },
            },
        ],
        ..Default::default()
    }
}

/// Starfield Shepherd — {3}{W}{W} 3/2 Angel. Flying. ETB: search for a basic
/// Plains or a creature card with mana value 1 or less and put it into your
/// hand. Warp {1}{W}.
pub fn starfield_shepherd() -> CardDefinition {
    CardDefinition {
        name: "Starfield Shepherd",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand
                .and(SelectionRequirement::HasLandType(LandType::Plains))
                .or(SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(1))),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        alternative_cost: Some(warp(cost(&[generic(1), w()]))),
        ..Default::default()
    }
}

/// Timeline Culler — {B}{B} 2/2 Drix Warlock. Haste; Warp—{B}, Pay 2 life. (The
/// "cast from graveyard via warp" clause is dropped — warp casts from hand.)
pub fn timeline_culler() -> CardDefinition {
    let mut warp_cost = warp(cost(&[b()]));
    warp_cost.life_cost = 2;
    CardDefinition {
        name: "Timeline Culler",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drix, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        alternative_cost: Some(warp_cost),
        ..Default::default()
    }
}

/// Tannuk, Memorial Ensign — {1}{R}{G} 2/4 Legendary Kavu Pilot. Landfall —
/// whenever a land you control enters, deal 1 damage to each opponent. (The
/// second-landfall-this-turn card draw is dropped — no per-source resolution
/// counter yet.)
pub fn tannuk_memorial_ensign() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Tannuk, Memorial Ensign",
        cost: cost(&[generic(1), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kavu, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Xu-Ifit, Osteoharmonist — {1}{B}{B} 2/3 Legendary Human Wizard. {T}: Return
/// target creature card from your graveyard to the battlefield. Activate only as
/// a sorcery. (The "is a Skeleton with no abilities" rider is dropped.)
pub fn xu_ifit_osteoharmonist() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Xu-Ifit, Osteoharmonist",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Monoist Circuit-Feeder — {4}{B}{B} 4/4 Artifact Creature — Nautilus. Flying.
/// ETB: until end of turn, target creature you control gets +X/+0 and target
/// creature an opponent controls gets -0/-X, where X is the number of artifacts
/// you control.
pub fn monoist_circuit_feeder() -> CardDefinition {
    let artifacts = || Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
        filter: SelectionRequirement::Artifact,
    };
    CardDefinition {
        name: "Monoist Circuit-Feeder",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nautilus], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                power: artifacts(),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
                power: Value::Const(0),
                toughness: Value::Times(Box::new(Value::Const(-1)), Box::new(artifacts())),
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Space-Time Anomaly — {2}{W}{U} Sorcery. Target player mills cards equal to
/// your life total.
pub fn space_time_anomaly() -> CardDefinition {
    CardDefinition {
        name: "Space-Time Anomaly",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Mill {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::LifeOf(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Systems Override — {2}{R} Sorcery. Gain control of target artifact or creature
/// until end of turn; untap it; it gains haste. (The Spacecraft charge rider is
/// dropped.)
pub fn systems_override() -> CardDefinition {
    CardDefinition {
        name: "Systems Override",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
                ),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Mutinous Massacre — {3}{B}{B}{R}{R} Sorcery. Choose odd or even, destroy each
/// creature with mana value of that parity, then gain control of all surviving
/// creatures until end of turn, untap them, and they gain haste.
pub fn mutinous_massacre() -> CardDefinition {
    let mass_threaten = || {
        vec![
            Effect::GainControl {
                what: Selector::EachPermanent(SelectionRequirement::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::EachPermanent(SelectionRequirement::Creature),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(SelectionRequirement::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]
    };
    let mode = |odd: bool| {
        let mut seq = vec![Effect::Destroy {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ManaValueParity { odd }),
            ),
        }];
        seq.extend(mass_threaten());
        Effect::Seq(seq)
    };
    CardDefinition {
        name: "Mutinous Massacre",
        cost: cost(&[generic(3), b(), b(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN { picks: vec![0], modes: vec![mode(true), mode(false)] },
        ..Default::default()
    }
}

/// Focus Fire — {W} Instant. Deal X damage to target attacking or blocking
/// creature, where X is 2 plus the number of creatures and/or Spacecraft you
/// control.
pub fn focus_fire() -> CardDefinition {
    CardDefinition {
        name: "Focus Fire",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::IsAttacking.or(SelectionRequirement::IsBlocking),
            ),
            amount: Value::Sum(vec![
                Value::Const(2),
                Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                    filter: SelectionRequirement::Creature
                        .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Spacecraft)),
                },
            ]),
        },
        ..Default::default()
    }
}

/// Scour for Scrap — {3}{U} Instant. Choose one or both — search for an artifact
/// card and put it into your hand; and/or return target artifact card from your
/// graveyard to your hand.
pub fn scour_for_scrap() -> CardDefinition {
    CardDefinition {
        name: "Scour for Scrap",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Artifact,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Artifact
                            .and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ],
        },
        ..Default::default()
    }
}

/// Terminal Velocity — {4}{R}{R} Sorcery. Put an artifact or creature card from
/// your hand onto the battlefield with haste; sacrifice it at your end step.
/// (The "deals MV damage to each creature when it leaves" rider is dropped.)
pub fn terminal_velocity() -> CardDefinition {
    CardDefinition {
        name: "Terminal Velocity",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            count: Value::Const(1),
            tapped: false,
            haste: true,
            sacrifice_eot: true,
        },
        ..Default::default()
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Melded Moxite — {1}{R} Artifact. ETB: you may discard a card; if you do, draw
/// two. {3}, Sacrifice this: Create a tapped 2/2 colorless Robot artifact
/// creature token.
pub fn melded_moxite() -> CardDefinition {
    let mut tapped_robot = robot_token();
    tapped_robot.tapped = true;
    CardDefinition {
        name: "Melded Moxite",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Discard a card, then draw two".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ])),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: tapped_robot,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Squire's Lightblade — {W} Artifact — Equipment. Flash. ETB: attach to target
/// creature you control; it gains first strike until end of turn. Equipped
/// creature gets +1/+0. Equip {3}.
pub fn squires_lightblade() -> CardDefinition {
    CardDefinition {
        name: "Squire's Lightblade",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(crate::card::EquipBonus { power: 1, toughness: 0, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Auxiliary Boosters — {4}{W} Artifact — Equipment. ETB: create a 2/2 Robot and
/// attach this to it. Equipped creature gets +1/+2 and has flying. Equip {3}.
pub fn auxiliary_boosters() -> CardDefinition {
    CardDefinition {
        name: "Auxiliary Boosters",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 2,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: robot_token(),
            },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        ..Default::default()
    }
}

/// Thaumaton Torpedo — {1} Artifact. {6}, {T}, Sacrifice this: Destroy target
/// nonland permanent. (The "{3} less if you attacked with a Spacecraft" discount
/// is dropped.)
pub fn thaumaton_torpedo() -> CardDefinition {
    CardDefinition {
        name: "Thaumaton Torpedo",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(SelectionRequirement::Nonland),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Terrasymbiosis — {2}{G} Enchantment. Whenever you put one or more +1/+1
/// counters on a creature you control, you may draw that many cards. Do this
/// only once each turn.
pub fn terrasymbiosis() -> CardDefinition {
    CardDefinition {
        name: "Terrasymbiosis",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            })
            .once_per_turn(),
            effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..Default::default()
    }
}

/// Weapons Manufacturing — {1}{R} Enchantment. Whenever a nontoken artifact you
/// control enters, create a colorless artifact token named Munitions with "When
/// this token leaves the battlefield, it deals 2 damage to any target."
pub fn weapons_manufacturing() -> CardDefinition {
    let munitions = TokenDefinition {
        name: "Munitions".into(),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Weapons Manufacturing",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Artifact.and(SelectionRequirement::NotToken),
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: munitions,
            },
        }],
        ..Default::default()
    }
}

// ── New EOE batch 2 (modern_decks) ──────────────────────────────────────────

/// Syr Vondam, the Lucent — {2}{W}{B}{B} 4/4 Legendary Human Knight. Deathtouch,
/// lifelink. Whenever Syr Vondam enters or attacks, other creatures you control
/// get +1/+0 and gain deathtouch until end of turn.
pub fn syr_vondam_the_lucent() -> CardDefinition {
    use crate::card::Supertype;
    let buff = || {
        let others = || Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
        );
        Effect::Seq(vec![
            Effect::PumpPT {
                what: others(),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: others(),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
        ])
    };
    CardDefinition {
        name: "Syr Vondam, the Lucent",
        cost: cost(&[generic(2), w(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![etb(buff()), on_attack(buff())],
        ..Default::default()
    }
}

/// Starwinder — {5}{U}{U} 7/7 Leviathan. Whenever a creature you control deals
/// combat damage to a player, you may draw that many cards. Warp {2}{U}{U}.
pub fn starwinder() -> CardDefinition {
    CardDefinition {
        name: "Starwinder",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Leviathan], ..Default::default() },
        power: 7,
        toughness: 7,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Draw that many cards".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount }),
            },
        }],
        alternative_cost: Some(warp(cost(&[generic(2), u(), u()]))),
        ..Default::default()
    }
}

/// Pinnacle Starcage — {1}{W}{W} Artifact. ETB: exile all artifacts and
/// creatures with mana value 2 or less until this leaves. (The `{6}{W}{W}`
/// dump-to-graveyard-and-make-Robots activated payoff is dropped.)
pub fn pinnacle_starcage() -> CardDefinition {
    CardDefinition {
        name: "Pinnacle Starcage",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: Selector::EachPermanent(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Creature)
                    .and(SelectionRequirement::ManaValueAtMost(2))
                    .and(SelectionRequirement::OtherThanSource),
            ),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Temporal Intervention — {2}{B} Sorcery. Void — costs {2} less if a nonland
/// permanent left the battlefield or a spell was warped this turn. Target
/// opponent reveals their hand; you choose a nonland card and they discard it.
/// (Modeled against each opponent — the single-target restriction is dropped.)
pub fn temporal_intervention() -> CardDefinition {
    CardDefinition {
        name: "Temporal Intervention",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "Void — costs {2} less to cast.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::VoidActive { who: PlayerRef::You },
                amount: 2,
            },
        }],
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Nonland,
        },
        ..Default::default()
    }
}

/// Vote Out — {3}{B} Sorcery. Convoke. Destroy target creature.
pub fn vote_out() -> CardDefinition {
    CardDefinition {
        name: "Vote Out",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Convoke],
        effect: Effect::Destroy { what: target_filtered(SelectionRequirement::Creature) },
        ..Default::default()
    }
}

/// Archenemy's Charm — {B}{B}{B} Instant. Choose one — exile target creature or
/// planeswalker; or return up to two creature/planeswalker cards from your
/// graveyard to your hand; or put two +1/+1 counters on a creature you control
/// and it gains lifelink. (Mode 2's per-card targeting is modeled as "up to
/// two".)
pub fn archenemys_charm() -> CardDefinition {
    CardDefinition {
        name: "Archenemy's Charm",
        cost: cost(&[b(), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![0],
            modes: vec![
                Effect::Exile {
                    what: target_filtered(
                        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                    ),
                },
                Effect::ReturnGraveyardCardsToHand {
                    filter: SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                    max: Value::Const(2),
                },
                Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        ),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Lifelink,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            ],
        },
        ..Default::default()
    }
}

/// Illvoi Infiltrator — {2}{U} 1/3 Jellyfish Rogue. Can't be blocked if you've
/// cast two or more spells this turn. Whenever it deals combat damage to a
/// player, draw a card.
pub fn illvoi_infiltrator() -> CardDefinition {
    CardDefinition {
        name: "Illvoi Infiltrator",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::CantBeBlockedIfControllerCastSpells(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Cryoshatter — {U} Aura. Enchant creature. Enchanted creature gets -5/-0; when
/// it becomes tapped or is dealt damage, destroy it.
pub fn cryoshatter() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    let destroy_host = |event: EventKind| TriggeredAbility {
        event: EventSpec::new(event, EventScope::SelfSource),
        effect: Effect::Destroy { what: Selector::This },
    };
    CardDefinition {
        name: "Cryoshatter",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: -5,
            triggered_abilities: vec![
                destroy_host(EventKind::Tapped),
                destroy_host(EventKind::DealtDamage),
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Hardlight Containment — {W} Aura. Enchant artifact you control. When it
/// enters, exile target creature an opponent controls until this Aura leaves.
pub fn hardlight_containment() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    CardDefinition {
        name: "Hardlight Containment",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                },
            },
            Effect::ExileUntilSourceLeaves {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
                return_to: crate::card::ExileReturnZone::Battlefield,
            },
        ]),
        ..Default::default()
    }
}

/// Meltstrider's Resolve — {G} Aura. Enchant creature you control. When it
/// enters, the enchanted creature fights up to one target creature an opponent
/// controls.
pub fn meltstriders_resolve() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    CardDefinition {
        name: "Meltstrider's Resolve",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                },
            },
            Effect::Fight {
                attacker: Selector::Target(0),
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Pain for All — {2}{R} Aura. Enchant creature you control. When it enters, the
/// enchanted creature deals damage equal to its power to any other target.
pub fn pain_for_all() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    CardDefinition {
        name: "Pain for All",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                },
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Any },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Tractor Beam — {2}{U}{U} Aura. Enchant creature or Spacecraft. ETB taps the
/// enchanted permanent; you control it and it doesn't untap during its
/// controller's untap step.
pub fn tractor_beam() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EnchantmentSubtype};
    CardDefinition {
        name: "Tractor Beam",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Spacecraft)),
            },
        },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: Selector::attached_to(Selector::This) },
            Effect::GainControlWhileSourceRemains { what: Selector::attached_to(Selector::This) },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Enchanted permanent doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::attached_to(Selector::This) },
        }],
        ..Default::default()
    }
}

/// Starport Security — {W} 1/1 Artifact Creature — Robot Soldier. {3}{W}, {T}:
/// Tap another target creature. (The "{2} less if you control a +1/+1-countered
/// creature" discount is dropped.)
pub fn starport_security() -> CardDefinition {
    CardDefinition {
        name: "Starport Security",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Robot, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mm'menon, the Right Hand — {3}{U}{U} 3/4 Legendary Jellyfish Advisor. Flying.
/// You may look at the top card of your library any time, and cast artifact
/// spells from the top of your library. (The artifact-restricted mana grant is
/// dropped.)
pub fn mmmenon_the_right_hand() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Mm'menon, the Right Hand",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish, CreatureType::Advisor],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Look at the top card of your library any time.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may cast artifact spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: SelectionRequirement::Artifact },
            },
        ],
        ..Default::default()
    }
}

/// Memorial Vault — {3}{R} Artifact. {T}, Sacrifice another artifact: Exile the
/// top X cards of your library, where X is one plus the sacrificed artifact's
/// mana value. You may play those cards this turn.
pub fn memorial_vault() -> CardDefinition {
    CardDefinition {
        name: "Memorial Vault",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((
                SelectionRequirement::Artifact.and(SelectionRequirement::OtherThanSource),
                1,
            )),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Sum(vec![Value::Const(1), Value::SacrificedManaValue]),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: true,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Astelli Reclaimer — {3}{W}{W} 5/4 Angel Warrior, flying. ETB: return target
/// noncreature, nonland permanent card with mana value ≤ the mana spent to cast
/// this from your graveyard to the battlefield.
pub fn astelli_reclaimer() -> CardDefinition {
    CardDefinition {
        name: "Astelli Reclaimer",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::PermanentCard
                    .and(SelectionRequirement::Noncreature)
                    .and(SelectionRequirement::Nonland)
                    .and(SelectionRequirement::InYourGraveyard)
                    .and(SelectionRequirement::ManaValueAtMostCastManaSpent),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        ..Default::default()
    }
}

/// Starfield Vocalist — {3}{U} 3/4 Human Bard. If a permanent entering causes one
/// of your permanents' triggered abilities to trigger, it triggers an additional
/// time (Panharmonicon). Warp {1}{U}.
pub fn starfield_vocalist() -> CardDefinition {
    CardDefinition {
        name: "Starfield Vocalist",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Your permanent-ETB triggered abilities trigger an additional time.",
            effect: StaticEffect::DoubleControllerEtbTriggers,
        }],
        alternative_cost: Some(warp(cost(&[generic(1), u()]))),
        ..Default::default()
    }
}

/// Perigee Beckoner — {4}{B} 4/5 Horror. ETB: another target creature you control
/// gets +2/+0 until end of turn. Warp {2}{B}. (The granted "dies → return tapped"
/// rider is dropped.)
pub fn perigee_beckoner() -> CardDefinition {
    CardDefinition {
        name: "Perigee Beckoner",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            },
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        alternative_cost: Some(warp(cost(&[generic(2), b()]))),
        ..Default::default()
    }
}

/// The Seriema — {1}{W}{W} legendary Spacecraft. ETB: search your library for a
/// legendary creature card and put it into your hand. Station; at 7+ it's a 5/5
/// with flying. (The 7+ "other tapped legendary creatures have indestructible"
/// static is dropped.)
pub fn the_seriema() -> CardDefinition {
    CardDefinition {
        name: "The Seriema",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::HasSupertype(crate::card::Supertype::Legendary)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 7, keywords: vec![Keyword::Flying], pt: Some((5, 5)), ..Default::default() }],
        ..Default::default()
    }
}

/// Survey Mechan — {4} 1/3 Robot, flying, hexproof. {10}, Sacrifice this: it
/// deals 3 damage to any target and you draw three cards. (The distinct-land-
/// name activation discount and the "target player" routing are approximated.)
pub fn survey_mechan() -> CardDefinition {
    CardDefinition {
        name: "Survey Mechan",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Hexproof],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(10)]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Loading Zone — {3}{G} Enchantment. Counters put on permanents you control are
/// doubled. Warp {G}. (Restriction to creatures/Spacecraft/Planets approximated
/// to all your counters.)
pub fn loading_zone() -> CardDefinition {
    CardDefinition {
        name: "Loading Zone",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If counters would be put on a permanent you control, double them.",
            effect: StaticEffect::DoubleCounters,
        }],
        alternative_cost: Some(warp(cost(&[g()]))),
        ..Default::default()
    }
}

/// Sami, Wildcat Captain — {4}{R}{W} 4/4 Human Artificer Rogue with double strike
/// and vigilance. Your instant/sorcery spells have affinity for artifacts. (The
/// "all spells" breadth is approximated to instants and sorceries.)
pub fn sami_wildcat_captain() -> CardDefinition {
    CardDefinition {
        name: "Sami, Wildcat Captain",
        cost: cost(&[generic(4), r(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Artificer,
                CreatureType::Rogue,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::DoubleStrike, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast have affinity for artifacts.",
            effect: StaticEffect::GrantAffinityToISSpells {
                permanent_filter: SelectionRequirement::Artifact
                    .and(SelectionRequirement::ControlledByYou),
            },
        }],
        ..Default::default()
    }
}

/// Divert Disaster — {1}{U} Instant. Counter target spell unless its controller
/// pays {2}. (The "if they pay, you create a Lander" rider is dropped.)
pub fn divert_disaster() -> CardDefinition {
    CardDefinition {
        name: "Divert Disaster",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
            exile: false,
            extra_generic: None,
        },
        ..Default::default()
    }
}

/// Blade of the Swarm — {3}{B} 3/1 Insect Assassin. ETB: choose one — put two
/// +1/+1 counters on this; or put target exiled card with warp on the bottom of
/// its owner's library.
pub fn blade_of_the_swarm() -> CardDefinition {
    CardDefinition {
        name: "Blade of the Swarm",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Warped.and(SelectionRequirement::InExile),
                ),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: crate::effect::LibraryPosition::Bottom,
                },
            },
        ]))],
        ..Default::default()
    }
}
