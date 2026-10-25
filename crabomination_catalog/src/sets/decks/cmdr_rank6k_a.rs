//! Commander: one-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), first file. Tests in `tests/recent_b/cmdr_rank6k_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword,
    LandType, MayPlayDuration, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w};

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

// ── The Eleventh Doctor // Bill Potts ────────────────────────────────────────

/// Ms. Marvel, Kamala Khan — {2}{U} legendary 1/4 reach, vigilance Mutant
/// Inhuman Hero. You have no maximum hand size. Whenever you cast a spell that
/// targets a creature you control, draw a card; until end of turn her base
/// power is the number of cards in your hand, tracked as the hand changes.
pub fn ms_marvel_kamala_khan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::SpellTargetsMatching(Box::new(R::Creature.and(R::ControlledByYou))),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                // Tracks the hand as it changes (`SetBasePtLive`).
                Effect::SetBasePtLive {
                    what: Selector::This,
                    power: Value::HandSizeOf(PlayerRef::You),
                    toughness: Value::Const(4),
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..legend(
            "Ms. Marvel, Kamala Khan",
            cost(&[generic(2), u()]),
            vec![
                CreatureType::Mutant,
                CreatureType::Inhuman,
                CreatureType::Hero,
            ],
            1,
            4,
        )
    }
}

// ── She-Hulk, Jennifer Walters ───────────────────────────────────────────────

/// She-Hulk, Jennifer Walters — {3}{R} legendary 4/4 trample Gamma Berserker
/// Hero. {2}{R}, sacrifice a land: draw a card and a +1/+1 counter on her.
pub fn she_hulk_jennifer_walters() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "She-Hulk, Jennifer Walters",
            cost(&[generic(3), r()]),
            vec![
                CreatureType::Gamma,
                CreatureType::Berserker,
                CreatureType::Hero,
            ],
            4,
            4,
        )
    }
}

// ── Tri-Sentinel, Act of Vengeance ───────────────────────────────────────────

/// Vibranium Energy Daggers — {1} indestructible Equipment: +2/+2. Equip {3}.
pub fn vibranium_energy_daggers() -> CardDefinition {
    CardDefinition {
        name: "Vibranium Energy Daggers",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Indestructible, Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Brinelin // Francisco ────────────────────────────────────────────────────

/// Ice Tunnel — Snow Land — Island Swamp; enters tapped.
pub fn ice_tunnel() -> CardDefinition {
    CardDefinition {
        name: "Ice Tunnel",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Snow],
        subtypes: Subtypes {
            land_types: vec![LandType::Island, LandType::Swamp],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        activated_abilities: vec![
            crate::sets::tap_add(crate::mana::Color::Blue),
            crate::sets::tap_add(crate::mana::Color::Black),
        ],
        ..Default::default()
    }
}

// ── Red Guardian, Super-Soldier ──────────────────────────────────────────────

/// Red Guardian, Super-Soldier — {2}{W} legendary 2/2 flash Human Soldier
/// Villain. ETB: destroy target creature an opponent controls that dealt
/// damage this turn.
pub fn red_guardian_super_soldier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(
                R::Creature
                    .and(R::ControlledByOpponent)
                    .and(R::DealtDamageThisTurnAsSource),
            ),
        })],
        ..legend(
            "Red Guardian, Super-Soldier",
            cost(&[generic(2), w()]),
            vec![
                CreatureType::Human,
                CreatureType::Soldier,
                CreatureType::Villain,
            ],
            2,
            2,
        )
    }
}

// ── Victor Mancha, Runaway ───────────────────────────────────────────────────

/// Victor Mancha, Runaway — {5} legendary 4/4 Human Hero artifact creature.
/// ETB: exile target card from your graveyard; you may play it for as long as
/// you control Victor Mancha.
pub fn victor_mancha_runaway() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ClearLastMoved,
            Effect::Move {
                what: target_filtered(R::Any.from_your_graveyard()),
                to: ZoneDest::ExileWithSourceStamp,
            },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: MayPlayDuration::WhileSourceOnBattlefield {
                    source: crate::card::CardId(0),
                    one_spell_a_turn: false,
                },
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        ]))],
        ..legend(
            "Victor Mancha, Runaway",
            cost(&[generic(5)]),
            vec![CreatureType::Human, CreatureType::Hero],
            4,
            4,
        )
    }
}

// ── Yoshimaru, Scrappy Stray ─────────────────────────────────────────────────

/// Yoshimaru, Scrappy Stray — {1}{G} legendary 1/1 Dog. ETB: another target
/// creature you control fights up to one target creature an opponent
/// controls. {6}: a +1/+1 counter on target nonlegendary creature.
pub fn yoshimaru_scrappy_stray() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Fight {
                attacker: target_filtered(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            effect: Effect::AddCounter {
                what: target_filtered(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary).negate()),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend(
            "Yoshimaru, Scrappy Stray",
            cost(&[generic(1), g()]),
            vec![CreatureType::Dog],
            1,
            1,
        )
    }
}

// ── The Eleventh Doctor // Adric ─────────────────────────────────────────────

/// Skill Borrower — {2}{U} 1/3 Human Wizard artifact creature. As long as the
/// top card of your library is an artifact or creature card, it has all that
/// card's activated abilities. (Residual: the top card isn't revealed to the
/// table; you may look at it.)
pub fn skill_borrower() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "As long as the top card of your library is an artifact or creature card, this creature has all activated abilities of that card.",
                effect: StaticEffect::HasActivatedAbilitiesOfLibraryTop {
                    filter: R::Artifact.or(R::Creature),
                },
            },
        ],
        ..creature(
            "Skill Borrower",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            3,
        )
    }
}

// ── Bebop, Skull & Crossbones ────────────────────────────────────────────────

/// Cartouche of Ambition — {2}{B} Aura Cartouche, enchant creature you
/// control. ETB: you may put a -1/-1 counter on target creature. Enchanted
/// creature gets +1/+1 and has lifelink.
pub fn cartouche_of_ambition() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Cartouche of Ambition",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Cartouche],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        },
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: host(),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Enchanted creature has lifelink.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: host(),
                    keyword: Keyword::Lifelink,
                },
            },
        ],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            }),
        })],
        ..Default::default()
    }
}

// ── Jocasta, Automaton Avenger ───────────────────────────────────────────────

/// Henchbots — {4} 2/3 Robot artifact creature. ETB: exile target tapped
/// creature an opponent controls until this leaves the battlefield.
pub fn henchbots() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Creature.and(R::Tapped).and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..creature(
            "Henchbots",
            cost(&[generic(4)]),
            vec![CreatureType::Robot],
            2,
            3,
        )
    }
}

// ── Teyo, Lightshield Expert ─────────────────────────────────────────────────

/// Rasaad yn Bashir — {2}{W} legendary 0/3 Human Monk; choose a Background.
/// Each creature you control assigns combat damage equal to its toughness.
/// Whenever he attacks, if you have the initiative, double the toughness of
/// each creature you control until end of turn.
pub fn rasaad_yn_bashir() -> CardDefinition {
    let mut attack = on_attack(Effect::ForEach {
        selector: yours(R::Creature),
        body: Box::new(Effect::PumpPT {
            what: Selector::TriggerSource,
            power: Value::ZERO,
            toughness: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
            duration: Duration::EndOfTurn,
        }),
    });
    attack.event = attack.event.with_filter(Predicate::HasInitiative {
        who: PlayerRef::You,
    });
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control assigns combat damage equal to its toughness rather than its power.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature),
                keyword: Keyword::AssignsCombatDamageByToughness,
            },
        }],
        triggered_abilities: vec![attack],
        ..legend(
            "Rasaad yn Bashir",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Monk],
            0,
            3,
        )
    }
}
