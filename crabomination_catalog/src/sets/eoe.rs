//! Edge of Eternities — Exhaust (CR 702.177). "Exhaust — [Cost]: [Effect]"
//! means "[Cost]: [Effect]. Activate only once" (per game). Modeled via the
//! `ActivatedAbility.exhaust` flag + `CardInstance.exhausted_abilities`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement,
    StationBand, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{
    etb, on_attack, on_dies, station, target, target_any, target_filtered, warp,
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
