//! Commander: one-card gaps of EDHREC's commanders ranked 3501-4500 (the
//! 2026-10-14 scan), first file. Tests in `tests/recent_b/cmdr_rank4k_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
use crate::game::types::TurnStep;
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: n,
        definition: Arc::new(def),
    }
}

fn plus_one(what: Selector) -> Effect {
    Effect::AddCounter {
        what,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    }
}

fn dragon_5_5() -> TokenDefinition {
    TokenDefinition {
        name: "Dragon".into(),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        ..Default::default()
    }
}

// ── Pianna, Nomad Captain ────────────────────────────────────────────────────

/// Inspiring Unicorn — {2}{W}{W} 2/2. Whenever it attacks, creatures you
/// control get +1/+1 until end of turn.
pub fn inspiring_unicorn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: yours(R::Creature),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Inspiring Unicorn",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Unicorn],
            2,
            2,
        )
    }
}

// ── Tuktuk the Explorer ──────────────────────────────────────────────────────

/// Tuktuk Rubblefort — {2}{R} 0/3 Wall with defender and reach. Creatures you
/// control have haste.
pub fn tuktuk_rubblefort() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature),
                keyword: Keyword::Haste,
            },
        }],
        ..creature(
            "Tuktuk Rubblefort",
            cost(&[generic(2), r()]),
            vec![CreatureType::Wall],
            0,
            3,
        )
    }
}

// ── Killmonger, Ruthless Usurper ─────────────────────────────────────────────

/// Luke Cage, Hero for Hire — {2}{R}{R} legendary 4/4 trample. At the beginning
/// of combat on your turn, create a Treasure token.
pub fn luke_cage_hero_for_hire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: make(Value::ONE, crabomination_base::tokens::treasure_token()),
        }],
        ..legend(
            "Luke Cage, Hero for Hire",
            cost(&[generic(2), r(), r()]),
            vec![
                CreatureType::Human,
                CreatureType::Mercenary,
                CreatureType::Hero,
            ],
            4,
            4,
        )
    }
}

// ── Gallia, the Merrymaker ───────────────────────────────────────────────────

/// Machinesmith Automaton — {2}{R} 2/2 trample Robot Villain artifact creature.
/// Whenever another artifact you control enters, put a +1/+1 counter on it.
pub fn machinesmith_automaton() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                }),
            effect: plus_one(Selector::This),
        }],
        ..creature(
            "Machinesmith Automaton",
            cost(&[generic(2), r()]),
            vec![CreatureType::Robot, CreatureType::Villain],
            2,
            2,
        )
    }
}

// ── Tempestra, Dame of Games ─────────────────────────────────────────────────

/// Marwyn, the Clearcutter — {R} legendary 2/1 Elf Warrior. {2}, {T}, sacrifice
/// an artifact or land: draw a card.
pub fn marwyn_the_clearcutter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_other_filter: Some((R::Artifact.or(R::Land), 1)),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend(
            "Marwyn, the Clearcutter",
            cost(&[r()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            2,
            1,
        )
    }
}

// ── Zamriel, Seraph of Steel ─────────────────────────────────────────────────

/// Zamriel, Seraph of Steel — {2}{W}{W} legendary 3/4 flying Angel. During your
/// turn, equipped creatures you control have indestructible.
pub fn zamriel_seraph_of_steel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "During your turn, equipped creatures you control have indestructible.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: yours(R::Creature.and(R::IsEquipped)),
                    keyword: Keyword::Indestructible,
                }),
            },
        }],
        ..legend(
            "Zamriel, Seraph of Steel",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Angel],
            3,
            4,
        )
    }
}

// ── Kiora of Fire and Ashes ──────────────────────────────────────────────────

/// Kiora of Fire and Ashes — {4}{R}{R} legendary 2/2 Merfolk Noble. ETB: a 5/5
/// red flying Dragon. {8}: another one.
pub fn kiora_of_fire_and_ashes() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(make(Value::ONE, dragon_5_5()))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: make(Value::ONE, dragon_5_5()),
            ..Default::default()
        }],
        ..legend(
            "Kiora of Fire and Ashes",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Merfolk, CreatureType::Noble],
            2,
            2,
        )
    }
}

// ── Jiang Yanggu, Never Alone ────────────────────────────────────────────────

/// Jiang Yanggu, Never Alone — {3}{G} legendary 2/2 Human Druid. ETB: Mowu, a
/// legendary 3/3 green Dog token. At the beginning of your end step, untap all
/// tokens you control.
pub fn jiang_yanggu_never_alone() -> CardDefinition {
    let mowu = TokenDefinition {
        name: "Mowu".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(make(Value::ONE, mowu)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::Untap {
                    what: yours(R::IsToken),
                    up_to: None,
                },
            },
        ],
        ..legend(
            "Jiang Yanggu, Never Alone",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            2,
            2,
        )
    }
}

// ── Big Bertha ───────────────────────────────────────────────────────────────

/// Big Bertha — {X}{G} legendary 1/1 vigilance Mutant Hero. Enters with X
/// +1/+1 counters. {1}{G}, {T}: a +1/+1 counter on it.
pub fn big_bertha() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            tap_cost: true,
            effect: plus_one(Selector::This),
            ..Default::default()
        }],
        ..legend(
            "Big Bertha",
            cost(&[crate::mana::x(), g()]),
            vec![CreatureType::Mutant, CreatureType::Hero],
            1,
            1,
        )
    }
}

// ── Syr Armont, the Redeemer ─────────────────────────────────────────────────

/// Birnin Zana Plaza — land, enters tapped. ETB: gain 1 life. {T}: {G} or {W}.
pub fn birnin_zana_plaza() -> CardDefinition {
    super::super::tapland_untyped(
        "Birnin Zana Plaza",
        Color::Green,
        Color::White,
        super::super::etb_gain_one(),
    )
}

// ── Sosuke, Son of Seshiro ───────────────────────────────────────────────────

/// Decorated Champion — {1}{G} 2/2 Elf Warrior. Whenever another Warrior your
/// team controls enters, a +1/+1 counter on it. (Free-for-all: your team is
/// you, CR 808.)
pub fn decorated_champion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Warrior),
                }),
            effect: plus_one(Selector::This),
        }],
        ..creature(
            "Decorated Champion",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            2,
            2,
        )
    }
}

// ── Kasimir the Lone Wolf / Hunding Gjornersen ───────────────────────────────

/// Arashin Foremost — {1}{W}{W} 2/2 double strike Human Warrior. Whenever it
/// enters or attacks, another target Warrior creature you control gains double
/// strike until end of turn.
pub fn arashin_foremost() -> CardDefinition {
    let grant = || Effect::GrantKeyword {
        what: target_filtered(
            R::Creature
                .and(R::HasCreatureType(CreatureType::Warrior))
                .and(R::ControlledByYou)
                .and(R::OtherThanSource),
        ),
        keyword: Keyword::DoubleStrike,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![etb(grant()), on_attack(grant())],
        ..creature(
            "Arashin Foremost",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            2,
        )
    }
}

// ── Sengir, the Dark Baron ───────────────────────────────────────────────────

/// Sengir Connoisseur — {3}{B}{B} 3/3 flying Vampire. Whenever one or more
/// other creatures die, a +1/+1 counter on it; once each turn.
pub fn sengir_connoisseur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::OtherThanSource,
                })
                .once_per_turn(),
            effect: plus_one(Selector::This),
        }],
        ..creature(
            "Sengir Connoisseur",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Vampire],
            3,
            3,
        )
    }
}

// ── Nagao, Bound by Honor ────────────────────────────────────────────────────

/// Seven-Tail Mentor — {3}{W} 2/3 Fox Samurai. When it enters or dies, a
/// +1/+1 counter on target creature or Vehicle you control.
pub fn seven_tail_mentor() -> CardDefinition {
    let counter = || {
        plus_one(target_filtered(
            (R::Creature.or(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Vehicle))).and(R::ControlledByYou),
        ))
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(counter()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: counter(),
            },
        ],
        ..creature(
            "Seven-Tail Mentor",
            cost(&[generic(3), w()]),
            vec![CreatureType::Fox, CreatureType::Samurai],
            2,
            3,
        )
    }
}

// ── Kingpin, Wilson Fisk ─────────────────────────────────────────────────────

/// Hammerhead, Maggia Boss — {1}{B} legendary 2/1 Human Rogue Villain.
/// Sacrifice another creature or artifact: a +1/+1 counter on it.
pub fn hammerhead_maggia_boss() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature.or(R::Artifact), 1)),
            effect: plus_one(Selector::This),
            ..Default::default()
        }],
        ..legend(
            "Hammerhead, Maggia Boss",
            cost(&[generic(1), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Rogue,
                CreatureType::Villain,
            ],
            2,
            1,
        )
    }
}

// ── The Eighth Doctor ────────────────────────────────────────────────────────

/// Diligent Excavator — {1}{U} 1/3 Human Artificer. Whenever you cast a
/// historic spell, target player mills two cards.
pub fn diligent_excavator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::historic())),
            effect: Effect::Mill {
                who: target_filtered(R::Player),
                amount: Value::Const(2),
            },
        }],
        ..creature(
            "Diligent Excavator",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            3,
        )
    }
}

// ── The Fabulous Frog-Man ────────────────────────────────────────────────────

/// The Fabulous Frog-Man — {2}{G} legendary 3/3 reach Human Hero.
pub fn the_fabulous_frog_man() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        ..legend(
            "The Fabulous Frog-Man",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Hero],
            3,
            3,
        )
    }
}

// ── Zuo Ci, the Mocking Sage ─────────────────────────────────────────────────

/// Zuo Ci, the Mocking Sage — {1}{G}{G} legendary 1/2 hexproof Human Advisor;
/// can't be blocked by creatures with horsemanship.
pub fn zuo_ci_the_mocking_sage() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Hexproof,
            Keyword::CantBeBlockedBy(Box::new(R::HasKeyword(Keyword::Horsemanship))),
        ],
        ..legend(
            "Zuo Ci, the Mocking Sage",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            2,
        )
    }
}

// ── Ma Chao, Western Warrior ─────────────────────────────────────────────────

/// Ma Chao, Western Warrior — {3}{R}{R} legendary 3/3 horsemanship Human
/// Soldier Warrior. Whenever it attacks alone, it can't be blocked this combat.
pub fn ma_chao_western_warrior() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Horsemanship],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsAttackingAlone,
                },
            ),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend(
            "Ma Chao, Western Warrior",
            cost(&[generic(3), r(), r()]),
            vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Warrior,
            ],
            3,
            3,
        )
    }
}

// ── Gandalf, Wandering Wizard ────────────────────────────────────────────────

/// Gandalf, Wandering Wizard — {4}{U} legendary 4/5 Avatar Wizard, ward {3}.
/// {6}: his owner shuffles him into their library and draws three cards.
pub fn gandalf_wandering_wizard() -> CardDefinition {
    let owner = || PlayerRef::OwnerOf(Box::new(Selector::This));
    CardDefinition {
        keywords: vec![Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(3)])))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library {
                        who: owner(),
                        pos: LibraryPosition::Shuffled,
                    },
                },
                Effect::Draw {
                    who: Selector::Player(owner()),
                    amount: Value::Const(3),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Gandalf, Wandering Wizard",
            cost(&[generic(4), u()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            4,
            5,
        )
    }
}

// ── Joven and Chandler ───────────────────────────────────────────────────────

/// Joven and Chandler — {3}{R}{R} legendary 3/3 Human Rogue. Backup 2, haste.
/// Whenever it deals combat damage to an opponent, destroy target artifact that
/// player controls (backup grants both haste and that trigger).
pub fn joven_and_chandler() -> CardDefinition {
    let raid = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
            .with_filter(Predicate::PlayerIsOpponent {
                who: PlayerRef::TriggerEventPlayer,
            }),
        effect: Effect::Destroy {
            what: target_filtered(R::Artifact.and(R::ControlledByTriggerPlayer)),
        },
    };
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            crate::effect::shortcut::backup_with(2, vec![Keyword::Haste], vec![raid.clone()]),
            raid,
        ],
        ..legend(
            "Joven and Chandler",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            3,
        )
    }
}
