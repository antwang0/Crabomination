//! Innistrad: Midnight Hunt (MID) / Crimson Vow (VOW) commons & uncommons that
//! round out the modern-era pool. Mechanics in play: coven, decayed, exploit,
//! Blood tokens, day/night, training, flashback. Each card has at least one
//! functionality test in `crabomination/src/tests/innistrad.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EventKind, EventScope, EventSpec, Keyword, Predicate, Selector, SelectionRequirement, Subtypes,
    TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_dies, on_other_dies, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

// ── token helpers ──────────────────────────────────────────────────────────

fn white_human_token() -> TokenDefinition {
    TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        ..Default::default()
    }
}

fn flying_bat_token() -> TokenDefinition {
    TokenDefinition {
        name: "Bat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        ..Default::default()
    }
}

fn reach_spider_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spider".into(),
        power: 1,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        keywords: vec![Keyword::Reach],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        ..Default::default()
    }
}

fn boar_3_1_token() -> TokenDefinition {
    TokenDefinition {
        name: "Boar".into(),
        power: 3,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Boar], ..Default::default() },
        ..Default::default()
    }
}

fn decayed_zombie_token() -> TokenDefinition {
    TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        keywords: vec![Keyword::Decayed],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    }
}

// ── White ──────────────────────────────────────────────────────────────────

/// Unruly Mob — {1}{W} 1/1 Human. Whenever another creature you control dies,
/// put a +1/+1 counter on this creature.
pub fn unruly_mob() -> CardDefinition {
    CardDefinition {
        name: "Unruly Mob",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_other_dies(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Clarion Cathars — {3}{W} 3/3 Human Knight. ETB create a 1/1 white Human.
pub fn clarion_cathars() -> CardDefinition {
    CardDefinition {
        name: "Clarion Cathars",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: white_human_token(),
        })],
        ..Default::default()
    }
}

/// Homestead Courage — {W} Sorcery. Put a +1/+1 counter on target creature you
/// control. It gains vigilance until end of turn. Flashback {W}.
pub fn homestead_courage() -> CardDefinition {
    CardDefinition {
        name: "Homestead Courage",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[w()]))],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Flare of Faith — {1}{W} Instant. Target creature gets +2/+2; if it's a
/// Human, it gets +3/+3 and gains indestructible until end of turn instead.
pub fn flare_of_faith() -> CardDefinition {
    CardDefinition {
        name: "Flare of Faith",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: SelectionRequirement::HasCreatureType(CreatureType::Human),
            },
            then: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ])),
            else_: Box::new(Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Ritual of Hope — {1}{W} Instant. Creatures you control get +1/+1. Coven —
/// if you control three or more creatures with different powers, +2/+1 instead.
pub fn ritual_of_hope() -> CardDefinition {
    let team = Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Ritual of Hope",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::CovenActive { who: PlayerRef::You },
            then: Box::new(Effect::PumpPT {
                what: team.clone(),
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::PumpPT {
                what: team,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Sunset Revelry — {1}{W} Sorcery. Gain 4 life if an opponent has more life;
/// create two 1/1 Humans if an opponent controls more creatures; draw two cards
/// if an opponent has more cards in hand.
pub fn sunset_revelry() -> CardDefinition {
    let noop = || Box::new(Effect::Noop);
    CardDefinition {
        name: "Sunset Revelry",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::AnOpponentHasMoreLife,
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(4) }),
                else_: noop(),
            },
            Effect::If {
                cond: Predicate::AnOpponentControlsMoreCreatures,
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: white_human_token(),
                }),
                else_: noop(),
            },
            Effect::If {
                cond: Predicate::AnOpponentHasMoreCardsInHand,
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
                else_: noop(),
            },
        ]),
        ..Default::default()
    }
}

/// Valorous Stance — {1}{W} Instant. Choose one — target creature gains
/// indestructible; or destroy target creature with toughness 4 or greater.
pub fn valorous_stance() -> CardDefinition {
    CardDefinition {
        name: "Valorous Stance",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ToughnessAtLeast(4)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Sanctify — {1}{W} Sorcery. Destroy target artifact or enchantment. You gain
/// 3 life.
pub fn sanctify() -> CardDefinition {
    CardDefinition {
        name: "Sanctify",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Piercing Light — {W} Instant. Deals 2 damage to target attacking or blocking
/// creature. Scry 1.
pub fn piercing_light() -> CardDefinition {
    CardDefinition {
        name: "Piercing Light",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::IsAttacking.or(SelectionRequirement::IsBlocking),
                ),
                amount: Value::Const(2),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Traveling Minister — {W} 1/1 Human Cleric. {T}: target creature gets +1/+0
/// until end of turn and you gain 1 life. Activate only as a sorcery.
pub fn traveling_minister() -> CardDefinition {
    CardDefinition {
        name: "Traveling Minister",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Resistance Squad — {2}{W} 3/2 Human Soldier. ETB, if you control another
/// Human, draw a card.
pub fn resistance_squad() -> CardDefinition {
    CardDefinition {
        name: "Resistance Squad",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::OtherThanSource
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Human))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Loyal Gryff — {2}{W} 2/2 Hippogriff. Flash, flying. ETB you may return
/// another creature you control to its owner's hand.
pub fn loyal_gryff() -> CardDefinition {
    CardDefinition {
        name: "Loyal Gryff",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hippogriff], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return another creature you control to its owner's hand?".to_string(),
            body: Box::new(Effect::Move {
                what: Selector::take(
                    Selector::EachPermanent(
                        SelectionRequirement::OtherThanSource
                            .and(SelectionRequirement::Creature)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    Value::Const(1),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..Default::default()
    }
}

/// Heron of Hope — {3}{W} 2/3 Bird. Flying. If you would gain life, you gain
/// that much plus 1 instead. {1}{W}: this creature gains lifelink until EOT.
pub fn heron_of_hope() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::effect::PlayerStaticTarget;
    CardDefinition {
        name: "Heron of Hope",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If you would gain life, you gain that much plus 1 instead.",
            effect: StaticEffect::LifeGainBonus { target: PlayerStaticTarget::Controller, amount: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Search Party Captain — {3}{W} 2/2 Human Soldier. Costs {1} less for each
/// creature you attacked with this turn. ETB draw a card.
pub fn search_party_captain() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Search Party Captain",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Costs {1} less for each creature you attacked with this turn.",
            effect: StaticEffect::SelfCostReducedPerCreatureAttackedThisTurn { per: 1 },
        }],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(1) })],
        ..Default::default()
    }
}

// ── Blue ───────────────────────────────────────────────────────────────────

/// Larder Zombie — {U} 1/3 Zombie. Defender. Tap three untapped creatures you
/// control: Surveil 1.
pub fn larder_zombie() -> CardDefinition {
    CardDefinition {
        name: "Larder Zombie",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                3,
            )),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Startle — {1}{U} Instant. Target creature gets -2/-0 until end of turn.
/// Create a 2/2 black Zombie with decayed. Draw a card.
pub fn startle() -> CardDefinition {
    CardDefinition {
        name: "Startle",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: decayed_zombie_token(),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Organ Hoarder — {3}{U} 3/2 Zombie. ETB look at the top three cards, put one
/// into your hand and the rest into your graveyard.
pub fn organ_hoarder() -> CardDefinition {
    CardDefinition {
        name: "Organ Hoarder",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: true,
            pick_filter: None,
            take: Some(Value::Const(1)),
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Dissipate — {1}{U}{U} Instant. Counter target spell. If it's countered this
/// way, exile it instead.
pub fn dissipate() -> CardDefinition {
    use crate::effect::CounteredSpellZone;
    CardDefinition {
        name: "Dissipate",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpellToZone {
            what: Selector::Target(0),
            zone: CounteredSpellZone::Exile,
        },
        ..Default::default()
    }
}

/// Scattered Thoughts — {3}{U} Instant. Look at the top four cards. Put two into
/// your hand and the rest into your graveyard.
pub fn scattered_thoughts() -> CardDefinition {
    CardDefinition {
        name: "Scattered Thoughts",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: true,
            pick_filter: None,
            take: Some(Value::Const(2)),
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Vivisection — {3}{U} Sorcery. As an additional cost, sacrifice a creature.
/// Draw three cards.
pub fn vivisection() -> CardDefinition {
    CardDefinition {
        name: "Vivisection",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

// ── Black ──────────────────────────────────────────────────────────────────

/// Novice Occultist — {1}{B} 1/2 Human Wizard. When it dies, draw a card and
/// lose 1 life.
pub fn novice_occultist() -> CardDefinition {
    CardDefinition {
        name: "Novice Occultist",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Siege Zombie — {1}{B} 2/2 Zombie. Tap three untapped creatures you control:
/// Each opponent loses 1 life.
pub fn siege_zombie() -> CardDefinition {
    CardDefinition {
        name: "Siege Zombie",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                3,
            )),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blood Pact — {2}{B} Instant. Target player draws two cards and loses 2 life.
pub fn blood_pact() -> CardDefinition {
    CardDefinition {
        name: "Blood Pact",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::Target(0), amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::Target(0), amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Bat Whisperer — {3}{B} 4/2 Vampire. ETB, if an opponent lost life this turn,
/// create a 1/1 black Bat with flying.
pub fn bat_whisperer() -> CardDefinition {
    CardDefinition {
        name: "Bat Whisperer",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: flying_bat_token(),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Arrogant Outlaw — {2}{B} 3/2 Vampire Noble. ETB, if an opponent lost life
/// this turn, each opponent loses 2 life and you gain 2 life.
pub fn arrogant_outlaw() -> CardDefinition {
    CardDefinition {
        name: "Arrogant Outlaw",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
            then: Box::new(Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Eaten Alive — {B} Sorcery. As an additional cost, sacrifice a creature or
/// pay {3}{B}. Exile target creature or planeswalker.
pub fn eaten_alive() -> CardDefinition {
    CardDefinition {
        name: "Eaten Alive",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
            },
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Gluttonous Guest — {2}{B} 1/4 Vampire. ETB create a Blood token. Whenever
/// you sacrifice a Blood token, you gain 1 life.
pub fn gluttonous_guest() -> CardDefinition {
    CardDefinition {
        name: "Gluttonous Guest",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::blood_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Blood),
                    }),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

/// Courier Bat — {2}{B} 2/2 Bat. Flying. ETB, if you gained life this turn,
/// return up to one target creature card from your graveyard to your hand.
pub fn courier_bat() -> CardDefinition {
    CardDefinition {
        name: "Courier Bat",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bat], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::Const(1) },
            then: Box::new(Effect::Move {
                what: target_filtered(
                    SelectionRequirement::InGraveyard.and(SelectionRequirement::Creature),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

// ── Green ──────────────────────────────────────────────────────────────────

/// Timberland Guide — {1}{G} 1/1 Human Scout. ETB put a +1/+1 counter on target
/// creature.
pub fn timberland_guide() -> CardDefinition {
    CardDefinition {
        name: "Timberland Guide",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Pestilent Wolf — {1}{G} 2/2 Wolf. {2}{G}: this creature gains deathtouch
/// until end of turn.
pub fn pestilent_wolf() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Wolf",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Brood Weaver — {3}{G} 2/4 Spider. Reach. When it dies, create a 1/2 green
/// Spider with reach.
pub fn brood_weaver() -> CardDefinition {
    CardDefinition {
        name: "Brood Weaver",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: reach_spider_token(),
        })],
        ..Default::default()
    }
}

/// Toxic Scorpion — {1}{G} 1/1 Scorpion. Deathtouch. ETB another target creature
/// you control gains deathtouch until end of turn.
pub fn toxic_scorpion() -> CardDefinition {
    CardDefinition {
        name: "Toxic Scorpion",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Scorpion], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::OtherThanSource
                    .and(SelectionRequirement::Creature)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Deathtouch,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Clear Shot — {2}{G} Instant. Target creature you control gets +1/+1 until end
/// of turn. It deals damage equal to its power to target creature you don't
/// control.
pub fn clear_shot() -> CardDefinition {
    CardDefinition {
        name: "Clear Shot",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                power: Value::Const(1),
                toughness: Value::Const(1),
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

/// Might of the Old Ways — {1}{G} Instant. Target creature gets +2/+2. Coven —
/// then if you control three or more creatures with different powers, draw a card.
pub fn might_of_the_old_ways() -> CardDefinition {
    CardDefinition {
        name: "Might of the Old Ways",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::CovenActive { who: PlayerRef::You },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Crushing Canopy — {2}{G} Instant. Choose one — destroy target creature with
/// flying; or destroy target enchantment.
pub fn crushing_canopy() -> CardDefinition {
    CardDefinition {
        name: "Crushing Canopy",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                ),
            },
            Effect::Destroy { what: target_filtered(SelectionRequirement::Enchantment) },
        ]),
        ..Default::default()
    }
}

/// Rural Recruit — {3}{G} 1/1 Human Peasant. Training. ETB create a 3/1 green
/// Boar.
pub fn rural_recruit() -> CardDefinition {
    use crate::effect::shortcut::training;
    CardDefinition {
        name: "Rural Recruit",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            training(),
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: boar_3_1_token(),
            }),
        ],
        ..Default::default()
    }
}

/// Hamlet Vanguard — {2}{G} */* Human Warrior. Ward {2}. Enters with two +1/+1
/// counters for each other Human you control.
pub fn hamlet_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Hamlet Vanguard",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::count(Selector::EachPermanent(
                    SelectionRequirement::OtherThanSource
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Human))
                        .and(SelectionRequirement::ControlledByYou),
                ))),
            ),
        )),
        ..Default::default()
    }
}

/// Willow Geist — {G} 1/1 Treefolk Spirit. Trample. Whenever one or more cards
/// leave your graveyard, put a +1/+1 counter on it. Dies → gain life equal to
/// its power.
pub fn willow_geist() -> CardDefinition {
    CardDefinition {
        name: "Willow Geist",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            on_dies(Effect::GainLife {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::This)),
            }),
        ],
        ..Default::default()
    }
}

/// Packsong Pup — {1}{G} 1/1 Wolf. At the beginning of combat on your turn, if
/// you control another Wolf or Werewolf, put a +1/+1 counter on it. Dies → gain
/// life equal to its power.
pub fn packsong_pup() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Packsong Pup",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::OtherThanSource
                            .and(SelectionRequirement::ControlledByYou)
                            .and(
                                SelectionRequirement::HasCreatureType(CreatureType::Wolf)
                                    .or(SelectionRequirement::HasCreatureType(CreatureType::Werewolf)),
                            ),
                    ),
                    n: Value::Const(1),
                }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            on_dies(Effect::GainLife {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::This)),
            }),
        ],
        ..Default::default()
    }
}

/// Reclusive Taxidermist — {1}{G} 1/2 Human Druid. Gets +3/+2 while four or more
/// creature cards are in your graveyard. {T}: add one mana of any color.
pub fn reclusive_taxidermist() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::effect::shortcut::grant_tap_for_any_color;
    CardDefinition {
        name: "Reclusive Taxidermist",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        static_abilities: vec![
            StaticAbility {
                description: "Gets +3/+2 while four or more creature cards are in your graveyard.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::SelectorCountAtLeast {
                        sel: Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: crate::card::Zone::Graveyard,
                            filter: SelectionRequirement::Creature,
                        },
                        n: Value::Const(4),
                    },
                    power: 3,
                    toughness: 2,
                    keywords: vec![],
                },
            },
            grant_tap_for_any_color(SelectionRequirement::Any),
        ],
        ..Default::default()
    }
}

/// Mulch — {1}{G} Sorcery. Reveal the top four cards. Put all lands into your
/// hand and the rest into your graveyard.
pub fn mulch() -> CardDefinition {
    CardDefinition {
        name: "Mulch",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: true,
            pick_filter: Some(SelectionRequirement::Land),
            take: None,
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Tapping at the Window — {1}{G} Sorcery. Look at the top three cards. You may
/// reveal a creature card and put it into your hand. Rest to graveyard.
/// Flashback {2}{G}.
pub fn tapping_at_the_window() -> CardDefinition {
    CardDefinition {
        name: "Tapping at the Window",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), g()]))],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: true,
            pick_filter: Some(SelectionRequirement::Creature),
            take: Some(Value::Const(1)),
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Splendid Reclamation — {3}{G} Sorcery. Return all land cards from your
/// graveyard to the battlefield tapped.
pub fn splendid_reclamation() -> CardDefinition {
    CardDefinition {
        name: "Splendid Reclamation",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: SelectionRequirement::Land,
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        ..Default::default()
    }
}

// ── Red ────────────────────────────────────────────────────────────────────

/// Voldaren Stinger — {R} 1/1 Vampire Warrior. First strike while attacking.
/// {2}{R}: +2/+0 until end of turn.
pub fn voldaren_stinger() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Voldaren Stinger",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "First strike as long as this creature is attacking.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::IsAttacking,
                },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Abandon the Post — {1}{R} Sorcery. Up to two target creatures can't block
/// this turn. Flashback {3}{R}.
pub fn abandon_the_post() -> CardDefinition {
    CardDefinition {
        name: "Abandon the Post",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), r()]))],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}

/// Daybreak Combatants — {2}{R} 2/2 Human Warrior. Haste. ETB target creature
/// gets +2/+0 until end of turn.
pub fn daybreak_combatants() -> CardDefinition {
    CardDefinition {
        name: "Daybreak Combatants",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Neonate's Rush — {2}{R} Instant. Costs {1} less if you control a Vampire.
/// Deals 1 damage to target creature and 1 to its controller. Draw a card.
pub fn neonates_rush() -> CardDefinition {
    CardDefinition {
        name: "Neonate's Rush",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_control: Some((
            SelectionRequirement::HasCreatureType(CreatureType::Vampire),
            1,
        )),
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(1),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Rending Flame — {2}{R} Instant. Deals 5 damage to target creature or
/// planeswalker. If it's a Spirit, also deals 2 damage to its controller.
pub fn rending_flame() -> CardDefinition {
    CardDefinition {
        name: "Rending Flame",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(5),
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Spirit),
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// End the Festivities — {R} Sorcery. Deals 1 damage to each opponent and each
/// creature and planeswalker they control.
pub fn end_the_festivities() -> CardDefinition {
    CardDefinition {
        name: "End the Festivities",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::ControlledByOpponent.and(
                        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                    ),
                ),
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Raze the Effigy — {R} Instant. Choose one — destroy target artifact; or
/// target attacking creature gets +2/+2 until end of turn.
pub fn raze_the_effigy() -> CardDefinition {
    CardDefinition {
        name: "Raze the Effigy",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::IsAttacking),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Belligerent Guest — {2}{R} 3/2 Vampire. Trample. Combat damage to a player →
/// create a Blood token.
pub fn belligerent_guest() -> CardDefinition {
    CardDefinition {
        name: "Belligerent Guest",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::blood_token(),
            },
        }],
        ..Default::default()
    }
}

/// Frenzied Devils — {4}{R} 3/3 Devil. Haste. Whenever you cast a noncreature
/// spell, this creature gets +2/+2 until end of turn.
pub fn frenzied_devils() -> CardDefinition {
    use crate::effect::shortcut::cast_is_noncreature;
    CardDefinition {
        name: "Frenzied Devils",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Devil], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_noncreature()),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

// ── Black / Blue / Artifact (remainder) ──────────────────────────────────────

/// Undead Butler — {1}{B} 1/2 Zombie. ETB mill three. When it dies, you may
/// exile it; if you do, return target creature card from your graveyard to your
/// hand.
pub fn undead_butler() -> CardDefinition {
    CardDefinition {
        name: "Undead Butler",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::Mill { who: Selector::You, amount: Value::Const(3) }),
            on_dies(Effect::MayDo {
                description: "Exile Undead Butler to return a creature card from your graveyard?"
                    .to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Exile { what: Selector::This },
                    Effect::Move {
                        what: target_filtered(
                            SelectionRequirement::InGraveyard.and(SelectionRequirement::Creature),
                        ),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                ])),
            }),
        ],
        ..Default::default()
    }
}

/// Mindleech Ghoul — {1}{B} 2/2 Zombie. Exploit. When it exploits a creature,
/// each opponent exiles a card from their hand.
pub fn mindleech_ghoul() -> CardDefinition {
    use crate::effect::shortcut::exploit;
    CardDefinition {
        name: "Mindleech Ghoul",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![exploit(Effect::ExileFromHand {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Morkrut Behemoth — {4}{B} 7/6 Zombie Giant. Menace. (The "sacrifice a
/// creature or pay {1}{B}" additional cast cost is omitted — body only.)
pub fn morkrut_behemoth() -> CardDefinition {
    CardDefinition {
        name: "Morkrut Behemoth",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Giant],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Menace],
        ..Default::default()
    }
}

/// Demonic Bargain — {2}{B} Sorcery. Exile the top thirteen cards of your
/// library, then search your library for a card, put it into your hand, then
/// shuffle.
pub fn demonic_bargain() -> CardDefinition {
    CardDefinition {
        name: "Demonic Bargain",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileTopOfLibrary {
                who: Selector::You,
                amount: Value::Const(13),
                link_to_source: false,
                face_down: false,
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Thirst for Discovery — {2}{U} Instant. Draw three cards, then discard two
/// cards unless you discard a basic land card (modeled as discard two).
pub fn thirst_for_discovery() -> CardDefinition {
    CardDefinition {
        name: "Thirst for Discovery",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(2),
                random: false,
            },
        ]),
        ..Default::default()
    }
}

/// Blood Fountain — {B} Artifact. ETB create a Blood token. {3}{B}, {T},
/// Sacrifice this: return up to two target creature cards from your graveyard
/// to your hand.
pub fn blood_fountain() -> CardDefinition {
    CardDefinition {
        name: "Blood Fountain",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crabomination_base::tokens::blood_token(),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                filter: SelectionRequirement::InGraveyard.and(SelectionRequirement::Creature),
                effect: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
