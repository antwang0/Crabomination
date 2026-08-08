//! Return to Ravnica (RTR) gap wave 5: the guild Keyrune mana-rock cycle, the
//! guildmage cycle, the populate spells, and a spread of commons/uncommons on
//! existing primitives (detain, scavenge, defender-gated Gate riders, flash).
//! Tests in `classic_sets/rtr`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus,
    EquipScale, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, scavenge, target_filtered};
use crate::effect::{Duration, Effect as E, ManaPayload, PlayerRef, Selector, StaticEffect};
use crate::mana::{Color, b, colored, cost, g, generic, hybrid, r, u, w};

fn token(
    name: &str,
    p: i32,
    t: i32,
    colors: Vec<Color>,
    ct: CreatureType,
    kw: Vec<Keyword>,
) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords: kw,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes {
            creature_types: vec![ct],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// A guild Keyrune (CR 701.35 era mana rock): {3} artifact that taps for one of
/// two guild colors and, for its guild-cost, animates into a small creature with
/// the guild body until end of turn.
fn keyrune(
    name: &'static str,
    c1: Color,
    c2: Color,
    pt: (i32, i32),
    ct: CreatureType,
    kw: Vec<Keyword>,
    extra: Vec<TriggeredAbility>,
) -> CardDefinition {
    let (p, t) = pt;
    CardDefinition {
        name,
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: E::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![c1, c2], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[colored(c1), colored(c2)]),
                effect: E::Seq(vec![
                    E::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(p),
                        toughness: Value::Const(t),
                        creature_types: vec![ct],
                        keywords: kw,
                        duration: Duration::EndOfTurn,
                    },
                    E::BecomeColor {
                        what: Selector::This,
                        colors: vec![c1, c2],
                        duration: Duration::EndOfTurn,
                        additive: false,
                    },
                ]),
                ..Default::default()
            },
        ],
        triggered_abilities: extra,
        ..Default::default()
    }
}

pub fn azorius_keyrune() -> CardDefinition {
    keyrune(
        "Azorius Keyrune",
        Color::White,
        Color::Blue,
        (2, 2),
        CreatureType::Bird,
        vec![Keyword::Flying],
        vec![],
    )
}
pub fn golgari_keyrune() -> CardDefinition {
    keyrune(
        "Golgari Keyrune",
        Color::Black,
        Color::Green,
        (2, 2),
        CreatureType::Insect,
        vec![Keyword::Deathtouch],
        vec![],
    )
}
pub fn rakdos_keyrune() -> CardDefinition {
    keyrune(
        "Rakdos Keyrune",
        Color::Black,
        Color::Red,
        (3, 1),
        CreatureType::Devil,
        vec![Keyword::FirstStrike],
        vec![],
    )
}
pub fn selesnya_keyrune() -> CardDefinition {
    keyrune(
        "Selesnya Keyrune",
        Color::Green,
        Color::White,
        (3, 3),
        CreatureType::Wolf,
        vec![],
        vec![],
    )
}
/// Izzet Keyrune animates into a 2/1 Elemental that loots on combat damage.
pub fn izzet_keyrune() -> CardDefinition {
    keyrune(
        "Izzet Keyrune",
        Color::Blue,
        Color::Red,
        (2, 1),
        CreatureType::Elemental,
        vec![],
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: E::MayDo {
                description: "Draw a card, then discard a card?".into(),
                body: Box::new(E::Seq(vec![
                    E::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    E::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                ])),
            },
        }],
    )
}

/// Trained Caracal — {W} 1/1 Cat with lifelink.
pub fn trained_caracal() -> CardDefinition {
    CardDefinition {
        name: "Trained Caracal",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    }
}

/// Fencing Ace — {1}{W} 1/1 Human Soldier with double strike.
pub fn fencing_ace() -> CardDefinition {
    CardDefinition {
        name: "Fencing Ace",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::DoubleStrike],
        ..Default::default()
    }
}

/// Azorius Arrester — {1}{W} 2/1 Human Soldier. ETB: detain target creature an
/// opponent controls (CR 701.35).
pub fn azorius_arrester() -> CardDefinition {
    CardDefinition {
        name: "Azorius Arrester",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(E::Detain {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        })],
        ..Default::default()
    }
}

/// Vassal Soul — {1}{W/U}{W/U} 2/2 Spirit with flying.
pub fn vassal_soul() -> CardDefinition {
    CardDefinition {
        name: "Vassal Soul",
        cost: cost(&[
            generic(1),
            hybrid(Color::White, Color::Blue),
            hybrid(Color::White, Color::Blue),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Armory Guard — {3}{W} 2/5 Giant Soldier. Has vigilance as long as you control
/// a Gate.
pub fn armory_guard() -> CardDefinition {
    CardDefinition {
        name: "Armory Guard",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        static_abilities: vec![StaticAbility {
            description: "This creature has vigilance as long as you control a Gate.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Vigilance,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(LandType::Gate).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
            },
        }],
        ..Default::default()
    }
}

/// Axebane Guardian — {2}{G} 0/3 Human Druid with defender. `{T}: Add X mana in
/// any combination of colors, where X is the number of creatures you control
/// with defender.`
pub fn axebane_guardian() -> CardDefinition {
    CardDefinition {
        name: "Axebane Guardian",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: E::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColors(Value::count(Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasKeyword(Keyword::Defender)),
                ))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lobber Crew — {2}{R} 0/4 Goblin Warrior with defender. `{T}: Deal 1 damage to
/// each opponent.` Untaps whenever you cast a multicolored spell.
pub fn lobber_crew() -> CardDefinition {
    CardDefinition {
        name: "Lobber Crew",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: E::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Multicolored,
                },
            ),
            effect: E::Untap {
                what: Selector::This,
                up_to: None,
            },
        }],
        ..Default::default()
    }
}

/// Drudge Beetle — {1}{G} 2/2 Insect. Scavenge {5}{G}.
pub fn drudge_beetle() -> CardDefinition {
    CardDefinition {
        name: "Drudge Beetle",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![scavenge(cost(&[generic(5), g()]))],
        ..Default::default()
    }
}

/// Bazaar Krovod — {4}{W} 2/5 Beast. Whenever it attacks, another target
/// attacking creature gets +0/+2 until end of turn and untaps.
pub fn bazaar_krovod() -> CardDefinition {
    CardDefinition {
        name: "Bazaar Krovod",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![on_attack(E::Seq(vec![
            E::PumpPT {
                what: target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
                power: Value::ZERO,
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            E::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
        ]))],
        ..Default::default()
    }
}

/// Hover Barrier — {2}{U} 0/6 Illusion Wall with defender and flying.
pub fn hover_barrier() -> CardDefinition {
    CardDefinition {
        name: "Hover Barrier",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Illusion, CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 6,
        keywords: vec![Keyword::Defender, Keyword::Flying],
        ..Default::default()
    }
}

/// Isperia's Skywatch — {5}{U} 3/3 Vedalken Knight with flying. ETB: detain
/// target creature an opponent controls.
pub fn isperias_skywatch() -> CardDefinition {
    CardDefinition {
        name: "Isperia's Skywatch",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(E::Detain {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        })],
        ..Default::default()
    }
}

/// Hussar Patrol — {2}{W}{U} 2/4 Human Knight with flash and vigilance.
pub fn hussar_patrol() -> CardDefinition {
    CardDefinition {
        name: "Hussar Patrol",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flash, Keyword::Vigilance],
        ..Default::default()
    }
}

/// Voidwielder — {4}{U} 1/4 Human Wizard. ETB: you may return target creature to
/// its owner's hand.
pub fn voidwielder() -> CardDefinition {
    CardDefinition {
        name: "Voidwielder",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![etb(E::MayDo {
            description: "Return target creature to its owner's hand?".into(),
            body: Box::new(E::Move {
                what: target_filtered(R::Creature),
                to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..Default::default()
    }
}

/// Ogre Jailbreaker — {3}{B} 4/4 Ogre Rogue with defender. Can attack as though
/// it didn't have defender as long as you control a Gate.
pub fn ogre_jailbreaker() -> CardDefinition {
    CardDefinition {
        name: "Ogre Jailbreaker",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        static_abilities: vec![StaticAbility {
            description: "Can attack as though it didn't have defender as long as you control a Gate.",
            effect: StaticEffect::CanAttackIgnoringDefenderWhile {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasLandType(LandType::Gate).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
            },
        }],
        ..Default::default()
    }
}

/// Sluiceway Scorpion — {2}{B}{G} 2/2 Scorpion with deathtouch. Scavenge {1}{B}{G}.
pub fn sluiceway_scorpion() -> CardDefinition {
    CardDefinition {
        name: "Sluiceway Scorpion",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Scorpion],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![scavenge(cost(&[generic(1), b(), g()]))],
        ..Default::default()
    }
}

/// Trestle Troll — {1}{B}{G} 1/4 Troll with defender and reach. `{1}{B}{G}:
/// Regenerate this creature.`
pub fn trestle_troll() -> CardDefinition {
    CardDefinition {
        name: "Trestle Troll",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Defender, Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), g()]),
            effect: E::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Golgari Decoy — {3}{G} 2/2 Elf Rogue. All creatures able to block it do so.
/// Scavenge {3}{G}{G}.
pub fn golgari_decoy() -> CardDefinition {
    CardDefinition {
        name: "Golgari Decoy",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::AllMustBlock],
        activated_abilities: vec![scavenge(cost(&[generic(3), g(), g()]))],
        ..Default::default()
    }
}

/// Zanikev Locust — {5}{B} 3/3 Insect with flying. Scavenge {2}{B}{B}.
pub fn zanikev_locust() -> CardDefinition {
    CardDefinition {
        name: "Zanikev Locust",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![scavenge(cost(&[generic(2), b(), b()]))],
        ..Default::default()
    }
}

/// Viashino Racketeer — {2}{R} 2/1 Lizard Rogue. ETB: you may discard a card, and
/// if you do, draw a card.
pub fn viashino_racketeer() -> CardDefinition {
    CardDefinition {
        name: "Viashino Racketeer",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(E::MayDo {
            description: "Discard a card to draw a card?".into(),
            body: Box::new(E::Seq(vec![
                E::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                E::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Judge's Familiar — {W/U} 1/1 Bird with flying. `Sacrifice this creature:
/// Counter target instant or sorcery spell unless its controller pays {1}.`
pub fn judges_familiar() -> CardDefinition {
    CardDefinition {
        name: "Judge's Familiar",
        cost: cost(&[hybrid(Color::White, Color::Blue)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: E::CounterUnlessPaid {
                what: target_filtered(
                    R::IsSpellOnStack.and(
                        R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    ),
                ),
                mana_cost: cost(&[generic(1)]),
                exile: false,
                extra_generic: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ---- Guildmage cycle ----

/// New Prahv Guildmage — {W}{U} 2/2 Human Wizard. `{W}{U}: Target creature gains
/// flying until end of turn.` `{3}{W}{U}: Detain target nonland permanent an
/// opponent controls.`
pub fn new_prahv_guildmage() -> CardDefinition {
    CardDefinition {
        name: "New Prahv Guildmage",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w(), u()]),
                effect: E::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), w(), u()]),
                effect: E::Detain {
                    what: target_filtered(
                        R::Permanent.and(R::Nonland).and(R::ControlledByOpponent),
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Vitu-Ghazi Guildmage — {G}{W} 2/2 Dryad Shaman. `{4}{G}{W}: Create a 3/3 green
/// Centaur creature token.` `{2}{G}{W}: Populate.`
pub fn vitu_ghazi_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Vitu-Ghazi Guildmage",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(4), g(), w()]),
                effect: E::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(token(
                        "Centaur",
                        3,
                        3,
                        vec![Color::Green],
                        CreatureType::Centaur,
                        vec![],
                    )),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g(), w()]),
                effect: E::Populate {
                    who: PlayerRef::You,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Nivix Guildmage — {U}{R} 2/2 Human Wizard. `{1}{U}{R}: Draw a card, then
/// discard a card.` `{2}{U}{R}: Copy target instant or sorcery spell you control.
/// You may choose new targets for the copy.`
pub fn nivix_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Nivix Guildmage",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u(), r()]),
                effect: E::Seq(vec![
                    E::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    E::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u(), r()]),
                effect: E::CopySpellMayChooseTargets {
                    what: target_filtered(R::IsSpellOnStack.and(R::ControlledByYou).and(
                        R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    )),
                    count: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Korozda Guildmage — {B}{G} 2/2 Elf Shaman. `{1}{B}{G}: Target creature gets
/// +1/+1 and gains intimidate until end of turn.` `{2}{B}{G}, Sacrifice a
/// nontoken creature: Create X 1/1 green Saproling tokens, where X is the
/// sacrificed creature's toughness.`
pub fn korozda_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Korozda Guildmage",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b(), g()]),
                effect: E::Seq(vec![
                    E::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    E::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Intimidate,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b(), g()]),
                effect: E::Seq(vec![
                    E::SacrificeAndRemember {
                        who: PlayerRef::You,
                        filter: R::Creature.and(R::NotToken),
                    },
                    E::CreateToken {
                        who: PlayerRef::You,
                        count: Value::SacrificedToughness,
                        definition: Box::new(token(
                            "Saproling",
                            1,
                            1,
                            vec![Color::Green],
                            CreatureType::Saproling,
                            vec![],
                        )),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ---- Populate spells ----

/// Rootborn Defenses — {2}{W} Instant. Populate; then creatures you control gain
/// indestructible until end of turn.
pub fn rootborn_defenses() -> CardDefinition {
    CardDefinition {
        name: "Rootborn Defenses",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: E::Seq(vec![
            E::Populate {
                who: PlayerRef::You,
            },
            E::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Eyes in the Skies — {3}{W} Instant. Create a 1/1 white Bird token with flying,
/// then populate.
pub fn eyes_in_the_skies() -> CardDefinition {
    CardDefinition {
        name: "Eyes in the Skies",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: E::Seq(vec![
            E::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(token(
                    "Bird",
                    1,
                    1,
                    vec![Color::White],
                    CreatureType::Bird,
                    vec![Keyword::Flying],
                )),
            },
            E::Populate {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Coursers' Accord — {4}{G}{W} Sorcery. Create a 3/3 green Centaur token, then
/// populate.
pub fn coursers_accord() -> CardDefinition {
    CardDefinition {
        name: "Coursers' Accord",
        cost: cost(&[generic(4), g(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: E::Seq(vec![
            E::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(token(
                    "Centaur",
                    3,
                    3,
                    vec![Color::Green],
                    CreatureType::Centaur,
                    vec![],
                )),
            },
            E::Populate {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Horncaller's Chant — {7}{G} Sorcery. Create a 4/4 green Rhino token with
/// trample, then populate.
pub fn horncallers_chant() -> CardDefinition {
    CardDefinition {
        name: "Horncaller's Chant",
        cost: cost(&[generic(7), g()]),
        card_types: vec![CardType::Sorcery],
        effect: E::Seq(vec![
            E::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(token(
                    "Rhino",
                    4,
                    4,
                    vec![Color::Green],
                    CreatureType::Rhino,
                    vec![Keyword::Trample],
                )),
            },
            E::Populate {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Druid's Deliverance — {1}{G} Instant. Prevent all combat damage that would be
/// dealt to you this turn, then populate.
pub fn druids_deliverance() -> CardDefinition {
    CardDefinition {
        name: "Druid's Deliverance",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: E::Seq(vec![
            E::PreventAllCombatDamageToPlayerThisTurn {
                who: PlayerRef::You,
            },
            E::Populate {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Civic Saber — {1} Equipment. Equipped creature gets +1/+0 for each of its
/// colors. Equip {1}.
pub fn civic_saber() -> CardDefinition {
    CardDefinition {
        name: "Civic Saber",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 0,
            toughness: 0,
            keywords: vec![],
            scale: Some(EquipScale {
                filter: R::Creature,
                per_power: 1,
                per_toughness: 0,
                count_host_colors: true,
                ..Default::default()
            }),
            triggered_abilities: vec![],
            ..Default::default()
        }),
        ..Default::default()
    }
}
