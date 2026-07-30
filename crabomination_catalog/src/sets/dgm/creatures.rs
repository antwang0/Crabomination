//! Dragon's Maze (DGM) creatures — guild commons/uncommons/rares on existing
//! primitives (keyword vanillas, ETB payoffs, activated pumps, the Gatekeeper
//! cycle). Tests in `classic_sets/dgm`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{battalion, etb, on_attack, scavenge, target_filtered, unleash};
use crate::effect::{Duration, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// Two-or-more-Gates intervening 'if' for the Gatekeeper cycle.
pub(super) fn two_gates() -> Predicate {
    Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::HasLandType(LandType::Gate).and(R::ControlledByYou)),
        n: Value::Const(2),
    }
}

fn knight_vigilance_token() -> TokenDefinition {
    TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: creatures(vec![CreatureType::Knight]),
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    }
}

/// Armored Wolf-Rider — {3}{G}{W} 4/6.
pub fn armored_wolf_rider() -> CardDefinition {
    CardDefinition {
        name: "Armored Wolf-Rider",
        cost: cost(&[generic(3), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elf, CreatureType::Knight]),
        power: 4,
        toughness: 6,
        ..Default::default()
    }
}

/// Bane Alley Blackguard — {1}{B} 1/3.
pub fn bane_alley_blackguard() -> CardDefinition {
    CardDefinition {
        name: "Bane Alley Blackguard",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Rogue]),
        power: 1,
        toughness: 3,
        ..Default::default()
    }
}

/// Murmuring Phantasm — {1}{U} 0/5 with defender.
pub fn murmuring_phantasm() -> CardDefinition {
    CardDefinition {
        name: "Murmuring Phantasm",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Spirit]),
        power: 0,
        toughness: 5,
        keywords: vec![Keyword::Defender],
        ..Default::default()
    }
}

/// Ascended Lawmage — {2}{W}{U} 3/2 with flying and hexproof.
pub fn ascended_lawmage() -> CardDefinition {
    CardDefinition {
        name: "Ascended Lawmage",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Vedalken, CreatureType::Wizard]),
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Hexproof],
        ..Default::default()
    }
}

/// Spike Jester — {B}{R} 3/1 with haste.
pub fn spike_jester() -> CardDefinition {
    CardDefinition {
        name: "Spike Jester",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Goblin, CreatureType::Warrior]),
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Riot Piker — {1}{R} 2/1 with first strike; attacks each combat if able.
pub fn riot_piker() -> CardDefinition {
    CardDefinition {
        name: "Riot Piker",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Goblin, CreatureType::Berserker]),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike, Keyword::MustAttack],
        ..Default::default()
    }
}

/// Rakdos Drake — {2}{B} 1/2 flier with unleash.
pub fn rakdos_drake() -> CardDefinition {
    CardDefinition {
        name: "Rakdos Drake",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Drake]),
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![unleash()],
        ..Default::default()
    }
}

/// Skylasher — {1}{G} 2/2 with flash, can't be countered, reach, protection
/// from blue.
pub fn skylasher() -> CardDefinition {
    CardDefinition {
        name: "Skylasher",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Insect]),
        power: 2,
        toughness: 2,
        keywords: vec![
            Keyword::Flash,
            Keyword::CantBeCountered,
            Keyword::Reach,
            Keyword::Protection(Color::Blue),
        ],
        ..Default::default()
    }
}

/// Woodlot Crawler — {U}{B} 2/1 with forestwalk and protection from green.
pub fn woodlot_crawler() -> CardDefinition {
    CardDefinition {
        name: "Woodlot Crawler",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Insect]),
        power: 2,
        toughness: 1,
        keywords: vec![
            Keyword::Landwalk(LandType::Forest),
            Keyword::Protection(Color::Green),
        ],
        ..Default::default()
    }
}

/// Boros Mastiff — {1}{W} 2/2. Battalion: gains lifelink until end of turn.
pub fn boros_mastiff() -> CardDefinition {
    CardDefinition {
        name: "Boros Mastiff",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Dog]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![battalion(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Lifelink,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Kraul Warrior — {1}{G} 2/2. {5}{G}: +3/+3 until end of turn.
pub fn kraul_warrior() -> CardDefinition {
    CardDefinition {
        name: "Kraul Warrior",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Insect, CreatureType::Warrior]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Beetleform Mage — {1}{G}{U} 2/2. {G}{U}: +2/+2 and flying until end of turn;
/// once each turn.
pub fn beetleform_mage() -> CardDefinition {
    CardDefinition {
        name: "Beetleform Mage",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![
            CreatureType::Human,
            CreatureType::Insect,
            CreatureType::Wizard,
        ]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), u()]),
            once_per_turn: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Thrashing Mossdog — {3}{G} 3/3 with reach and Scavenge {4}{G}{G}.
pub fn thrashing_mossdog() -> CardDefinition {
    CardDefinition {
        name: "Thrashing Mossdog",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Plant, CreatureType::Dog]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![scavenge(cost(&[generic(4), g(), g()]))],
        ..Default::default()
    }
}

/// Zhur-Taa Druid — {R}{G} 1/1 mana dork; tapping it for mana pings each
/// opponent for 1.
pub fn zhur_taa_druid() -> CardDefinition {
    CardDefinition {
        name: "Zhur-Taa Druid",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Druid]),
        power: 1,
        toughness: 1,
        // Zhur-Taa Druid has a single tap-for-mana ability, so the "whenever you
        // tap it for mana, it deals 1 to each opponent" trigger folds exactly
        // into that ability's resolution.
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::Green], Value::ONE),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Haunter of Nightveil — {3}{U}{B} 3/4. Creatures your opponents control get
/// -1/-0.
pub fn haunter_of_nightveil() -> CardDefinition {
    CardDefinition {
        name: "Haunter of Nightveil",
        cost: cost(&[generic(3), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Spirit]),
        power: 3,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control get -1/-0.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::ControlledByOpponent),
                power: -1,
                toughness: 0,
                keywords: vec![],
                opponents: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Jelenn Sphinx — {3}{W}{U} 1/5 with flying and vigilance; attacking pumps
/// other attackers +1/+1.
pub fn jelenn_sphinx() -> CardDefinition {
    CardDefinition {
        name: "Jelenn Sphinx",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Sphinx]),
        power: 1,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(R::IsAttacking.and(R::OtherThanSource)),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Bronzebeak Moa — {2}{G}{W} 2/2. Whenever another creature you control
/// enters, this gets +3/+3 until end of turn.
pub fn bronzebeak_moa() -> CardDefinition {
    CardDefinition {
        name: "Bronzebeak Moa",
        cost: cost(&[generic(2), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Bird]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Carnage Gladiator — {2}{B}{R} 4/2. Whenever a creature blocks, its
/// controller loses 1 life. {1}{B}{R}: Regenerate this creature.
pub fn carnage_gladiator() -> CardDefinition {
    CardDefinition {
        name: "Carnage Gladiator",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Skeleton, CreatureType::Warrior]),
        power: 4,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::AnyPlayer),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), r()]),
            effect: Effect::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Maw of the Obzedat — {3}{W}{B} 3/3. Sacrifice a creature: Creatures you
/// control get +1/+1 until end of turn.
pub fn maw_of_the_obzedat() -> CardDefinition {
    CardDefinition {
        name: "Maw of the Obzedat",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Thrull]),
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sin Collector — {1}{W}{B} 2/1. ETB: target opponent reveals their hand; you
/// exile an instant or sorcery card from it.
pub fn sin_collector() -> CardDefinition {
    CardDefinition {
        name: "Sin Collector",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Cleric]),
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::ExileChosenFromHand {
            from: Selector::Player(PlayerRef::Target(0)),
            count: Value::ONE,
            filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            link_to_source: false,
            face_down: false,
        })],
        ..Default::default()
    }
}

/// Deputy of Acquittals — {W}{U} 2/2 with flash. ETB: you may return another
/// target creature you control to its owner's hand.
pub fn deputy_of_acquittals() -> CardDefinition {
    CardDefinition {
        name: "Deputy of Acquittals",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "return another creature you control to hand?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..Default::default()
    }
}

/// Fluxcharger — {2}{U}{R} 1/5 flier. Whenever you cast an instant or sorcery,
/// you may switch its power and toughness until end of turn.
pub fn fluxcharger() -> CardDefinition {
    CardDefinition {
        name: "Fluxcharger",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Weird]),
        power: 1,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                ),
            ),
            effect: Effect::MayDo {
                description: "switch Fluxcharger's power and toughness?".into(),
                body: Box::new(Effect::SwitchPT {
                    what: Selector::This,
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Hired Torturer — {2}{B} 2/3 with defender. {3}{B}, {T}: target opponent
/// loses 2 life, then reveals a card at random from their hand.
pub fn hired_torturer() -> CardDefinition {
    CardDefinition {
        name: "Hired Torturer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Rogue]),
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3), b()]),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blood Scrivener — {1}{B} 2/1. If you'd draw a card while your hand is empty,
/// instead draw two and lose 1 life.
pub fn blood_scrivener() -> CardDefinition {
    CardDefinition {
        name: "Blood Scrivener",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Zombie, CreatureType::Wizard]),
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card while you have no cards in hand, instead draw two cards and lose 1 life.",
            effect: StaticEffect::EmptyHandDrawBonus {
                extra: 1,
                life_loss: 1,
            },
        }],
        ..Default::default()
    }
}

/// Pontiff of Blight — {4}{B}{B} 2/7. Extort; other creatures you control have
/// extort (CR 702.99 — each instance triggers separately).
pub fn pontiff_of_blight() -> CardDefinition {
    use crate::effect::shortcut::extort;
    CardDefinition {
        name: "Pontiff of Blight",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Zombie, CreatureType::Cleric]),
        power: 2,
        toughness: 7,
        triggered_abilities: vec![extort()],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have extort.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ability: Box::new(extort()),
            },
        }],
        ..Default::default()
    }
}

// ── Gatekeeper cycle (ETB, intervening 'if' two or more Gates) ───────────────

/// Sunspire Gatekeepers — {3}{W} 2/4. ETB with two+ Gates: make a 2/2 white
/// Knight with vigilance.
pub fn sunspire_gatekeepers() -> CardDefinition {
    CardDefinition {
        name: "Sunspire Gatekeepers",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Soldier]),
        power: 2,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: two_gates(),
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: knight_vigilance_token(),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Opal Lake Gatekeepers — {3}{U} 2/4. ETB with two+ Gates: you may draw a card.
pub fn opal_lake_gatekeepers() -> CardDefinition {
    CardDefinition {
        name: "Opal Lake Gatekeepers",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Vedalken, CreatureType::Soldier]),
        power: 2,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: two_gates(),
            then: Box::new(Effect::MayDo {
                description: "draw a card?".into(),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Ubul Sar Gatekeepers — {3}{B} 2/4. ETB with two+ Gates: target creature an
/// opponent controls gets -2/-2 until end of turn.
pub fn ubul_sar_gatekeepers() -> CardDefinition {
    CardDefinition {
        name: "Ubul Sar Gatekeepers",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Zombie, CreatureType::Soldier]),
        power: 2,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: two_gates(),
            then: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Saruli Gatekeepers — {3}{G} 2/4. ETB with two+ Gates: gain 7 life.
pub fn saruli_gatekeepers() -> CardDefinition {
    CardDefinition {
        name: "Saruli Gatekeepers",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elf, CreatureType::Warrior]),
        power: 2,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::If {
            cond: two_gates(),
            then: Box::new(Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(7),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Trostani's Summoner — {5}{G}{W} 1/1. ETB: make a 2/2 Knight (vigilance), a
/// 3/3 Centaur, and a 4/4 Rhino (trample).
pub fn trostanis_summoner() -> CardDefinition {
    let centaur = TokenDefinition {
        name: "Centaur".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: creatures(vec![CreatureType::Centaur]),
        ..Default::default()
    };
    let rhino = TokenDefinition {
        name: "Rhino".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: creatures(vec![CreatureType::Rhino]),
        keywords: vec![Keyword::Trample],
        ..Default::default()
    };
    CardDefinition {
        name: "Trostani's Summoner",
        cost: cost(&[generic(5), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elf, CreatureType::Shaman]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: knight_vigilance_token(),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: centaur,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: rhino,
            },
        ]))],
        ..Default::default()
    }
}

// ── Maze Elemental cycle: {5}{C} big body granting its keyword to your
// multicolored creatures ─────────────────────────────────────────────────────

fn maze_elemental(
    name: &'static str,
    mono: crate::mana::ManaCost,
    power: i32,
    toughness: i32,
    keyword: Keyword,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mono,
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elemental]),
        power,
        toughness,
        keywords: vec![keyword.clone()],
        static_abilities: vec![StaticAbility {
            description: "Multicolored creatures you control have this creature's keyword.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::ControlledByYou).and(R::Multicolored),
                power: 0,
                toughness: 0,
                keywords: vec![keyword],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Maze Sentinel — {5}{W} 3/6. Vigilance; your multicolored creatures have it.
pub fn maze_sentinel() -> CardDefinition {
    maze_elemental(
        "Maze Sentinel",
        cost(&[generic(5), w()]),
        3,
        6,
        Keyword::Vigilance,
    )
}
/// Maze Glider — {5}{U} 3/5. Flying; your multicolored creatures have it.
pub fn maze_glider() -> CardDefinition {
    maze_elemental(
        "Maze Glider",
        cost(&[generic(5), u()]),
        3,
        5,
        Keyword::Flying,
    )
}
/// Maze Abomination — {5}{B} 4/5. Deathtouch; your multicolored creatures have it.
pub fn maze_abomination() -> CardDefinition {
    maze_elemental(
        "Maze Abomination",
        cost(&[generic(5), b()]),
        4,
        5,
        Keyword::Deathtouch,
    )
}
/// Maze Rusher — {5}{R} 6/3. Haste; your multicolored creatures have it.
pub fn maze_rusher() -> CardDefinition {
    maze_elemental(
        "Maze Rusher",
        cost(&[generic(5), r()]),
        6,
        3,
        Keyword::Haste,
    )
}
/// Maze Behemoth — {5}{G} 5/4. Trample; your multicolored creatures have it.
pub fn maze_behemoth() -> CardDefinition {
    maze_elemental(
        "Maze Behemoth",
        cost(&[generic(5), g()]),
        5,
        4,
        Keyword::Trample,
    )
}

/// Korozda Gorgon — {3}{B}{G} 2/5 with deathtouch. {2}, Remove a +1/+1 counter
/// from a creature you control: target creature gets -1/-1 until end of turn.
pub fn korozda_gorgon() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Korozda Gorgon",
        cost: cost(&[generic(3), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Gorgon]),
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            remove_counter_among_filter: Some((
                Some(CounterType::PlusOnePlusOne),
                1,
                R::Creature.and(R::ControlledByYou),
            )),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Haazda Snare Squad — {2}{W} 1/4. Whenever it attacks, you may pay {W}: tap
/// target creature an opponent controls.
pub fn haazda_snare_squad() -> CardDefinition {
    CardDefinition {
        name: "Haazda Snare Squad",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Soldier]),
        power: 1,
        toughness: 4,
        triggered_abilities: vec![on_attack(Effect::MayPay {
            description: "pay {W}: tap target creature an opponent controls".into(),
            mana_cost: cost(&[w()]),
            body: Box::new(Effect::Tap {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}
