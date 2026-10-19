//! Commander: 2026-10-19 (`01DJHkWk`) — three-gap closers: the TMNT frog and
//! turtle decks, Spider-Man To the Rescue, Vulture, the Foot ninjas, Elektra
//! and MACH-1. Tests in `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn vehicle(name: &'static str, mana: ManaCost, p: i32, t: i32, crew: u32) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Crew(crew)],
        ..spell(name, mana, CardType::Artifact, Effect::Noop)
    }
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn pump(what: Selector, power: i32, toughness: i32) -> Effect {
    Effect::PumpPT {
        what,
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword {
        what,
        keyword,
        duration: Duration::EndOfTurn,
    }
}

fn mutagen() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(crabomination_base::tokens::mutagen_token()),
    }
}

fn token(name: &str, color: Color, types: Vec<CreatureType>) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: name.into(),
            power: 1,
            toughness: 1,
            card_types: vec![CardType::Creature],
            colors: vec![color],
            subtypes: Subtypes {
                creature_types: types,
                ..Default::default()
            },
            ..Default::default()
        }),
    }
}

// ── TMNT frogs and turtles ──────────────────────────────────────────────────

/// Punk Frogs — {3}{G/U}{G/U} 4/5 Frog Mutant Rebel, ward {3}.
pub fn punk_frogs() -> CardDefinition {
    let gu = || hybrid(Color::Green, Color::Blue);
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(3)])))],
        ..creature(
            "Punk Frogs",
            cost(&[generic(3), gu(), gu()]),
            vec![
                CreatureType::Frog,
                CreatureType::Mutant,
                CreatureType::Rebel,
            ],
            4,
            5,
        )
    }
}

/// Slithering Cryptid — {2}{G/U} 2/3 Fish Mutant; a Mutagen token as it
/// enters.
pub fn slithering_cryptid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(mutagen())],
        ..creature(
            "Slithering Cryptid",
            cost(&[generic(2), hybrid(Color::Green, Color::Blue)]),
            vec![CreatureType::Fish, CreatureType::Mutant],
            2,
            3,
        )
    }
}

/// Return to the Sewers — {3}{U} instant. Target creature's owner puts it on
/// top or bottom of their library; you create a Mutagen token.
pub fn return_to_the_sewers() -> CardDefinition {
    spell(
        "Return to the Sewers",
        cost(&[generic(3), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::OwnerChoice,
                },
            },
            mutagen(),
        ]),
    )
}

/// Retro-Mutation — {2}{U} flash Aura. Enchanted creature is a Turtle with
/// base power and toughness 0/1, can't attack, and loses all abilities.
pub fn retro_mutation() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((0, 1)),
            set_creature_types: Some(vec![CreatureType::Turtle]),
            remove_abilities: true,
            keywords: vec![Keyword::CantAttack],
            ..Default::default()
        }),
        ..spell(
            "Retro-Mutation",
            cost(&[generic(2), u()]),
            CardType::Enchantment,
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature),
            },
        )
    }
}

/// Turtle Power! — {2}{G} flash enchantment. Turtles you control get +2/+2.
pub fn turtle_power() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "Turtles you control get +2/+2.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    mine().and(R::HasCreatureType(CreatureType::Turtle)),
                ),
                power: 2,
                toughness: 2,
            },
        }],
        ..spell(
            "Turtle Power!",
            cost(&[generic(2), g()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

// ── Spider-Man, To the Rescue ───────────────────────────────────────────────

/// Subway Train — {2} Vehicle 3/1, crew 2. Entering, you may pay {G} to
/// search for a basic land card to hand.
pub fn subway_train() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {G} to search for a basic land?".into(),
            mana_cost: cost(&[g()]),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: None,
        })],
        ..vehicle("Subway Train", cost(&[generic(2)]), 3, 1, 2)
    }
}

/// Black Panther, Vanguard — {2}{G}{W} legendary 4/4 Human Warrior Hero.
/// Another nontoken Hero of yours entering: a 1/1 white Soldier, or your
/// creatures get +1/+1 until end of turn.
pub fn black_panther_vanguard() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature
                        .and(R::NotToken)
                        .and(R::HasCreatureType(CreatureType::Hero)),
                }),
            effect: Effect::ChooseMode(vec![
                token("Soldier", Color::White, vec![CreatureType::Soldier]),
                pump(Selector::EachPermanent(mine()), 1, 1),
            ]),
        }],
        ..creature(
            "Black Panther, Vanguard",
            cost(&[generic(2), g(), w()]),
            vec![
                CreatureType::Human,
                CreatureType::Warrior,
                CreatureType::Hero,
            ],
            4,
            4,
        )
    }
}

/// Professional Wrestler — {3}{G} 4/4 Human Warrior Performer. A Treasure as
/// it enters; can't be blocked by more than one creature.
pub fn professional_wrestler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        triggered_abilities: vec![etb(crate::effect::shortcut::mint_treasures(1))],
        ..creature(
            "Professional Wrestler",
            cost(&[generic(3), g()]),
            vec![
                CreatureType::Human,
                CreatureType::Warrior,
                CreatureType::Performer,
            ],
            4,
            4,
        )
    }
}

// ── Vulture, Scheming Scavenger ─────────────────────────────────────────────

/// Passenger Ferry — {3} Vehicle 4/3, crew 2. Attacking, you may pay {U}:
/// another target attacking creature can't be blocked this turn.
pub fn passenger_ferry() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MayPay {
            description: "Pay {U} to make another attacker unblockable?".into(),
            mana_cost: cost(&[u()]),
            body: Box::new(grant(
                target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
                Keyword::Unblockable,
            )),
            else_: None,
        })],
        ..vehicle("Passenger Ferry", cost(&[generic(3)]), 4, 3, 2)
    }
}

/// Theorix Annex — Land. Enters tapped unless you control a planeswalker.
/// {T}: Add {U} or {B}.
pub fn theorix_annex() -> CardDefinition {
    super::cmdr_oct17_g::walker_dual("Theorix Annex", Color::Blue, Color::Black)
}

/// Inner Demons Gangsters — {3}{B} 3/4 Human Rogue Villain. Discard a card:
/// +1/+0 and menace until end of turn; sorcery speed.
pub fn inner_demons_gangsters() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                pump(Selector::This, 1, 0),
                grant(Selector::This, Keyword::Menace),
            ]),
            ..Default::default()
        }],
        ..creature(
            "Inner Demons Gangsters",
            cost(&[generic(3), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Rogue,
                CreatureType::Villain,
            ],
            3,
            4,
        )
    }
}

// ── The Foot (Oroku Saki, Splinter Hamato Yoshi) ────────────────────────────

/// Shredder's Armor — {1}{B} Equipment, +2/+1. Entering, attach to target
/// creature you control. Equip—sacrifice another nonland permanent, once
/// each turn.
pub fn shredders_armor() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[]))],
        equip_sacrifice_filter: Some(R::Permanent.and(R::Nonland).and(R::OtherThanSource)),
        keyword_once_per_turn: true,
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 1,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(mine()),
        })],
        ..spell(
            "Shredder's Armor",
            cost(&[generic(1), b()]),
            CardType::Artifact,
            Effect::Noop,
        )
    }
}

/// Azra Smokeshaper — {3}{B} 3/3 Azra Ninja, ninjutsu {1}{B}. Entering, target
/// creature you control gains indestructible until end of turn.
pub fn azra_smokeshaper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), b()]))],
        triggered_abilities: vec![etb(grant(target_filtered(mine()), Keyword::Indestructible))],
        ..creature(
            "Azra Smokeshaper",
            cost(&[generic(3), b()]),
            vec![CreatureType::Azra, CreatureType::Ninja],
            3,
            3,
        )
    }
}

/// Ninja of the New Moon — {3}{B}{B} 6/3 Spirit Ninja, ninjutsu {3}{B}.
pub fn ninja_of_the_new_moon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(3), b()]))],
        ..creature(
            "Ninja of the New Moon",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Spirit, CreatureType::Ninja],
            6,
            3,
        )
    }
}

/// Foot Mystic — {3}{B} 2/4 lifelink Human Ninja Warlock. Disappear —
/// entering after a permanent left the battlefield under your control this
/// turn, a 1/1 black Ninja.
pub fn foot_mystic() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::RevoltActive {
                    who: PlayerRef::You,
                }),
            effect: token("Ninja", Color::Black, vec![CreatureType::Ninja]),
        }],
        ..creature(
            "Foot Mystic",
            cost(&[generic(3), b()]),
            vec![
                CreatureType::Human,
                CreatureType::Ninja,
                CreatureType::Warlock,
            ],
            2,
            4,
        )
    }
}

// ── Elektra, Femme Fatale ───────────────────────────────────────────────────

/// Mirror of Life Trapping — {4} artifact. Whenever a creature enters, if it
/// was cast, exile it, then return every other card exiled with this.
/// (Returning first is the same board: the newcomer is not yet among them.)
pub fn mirror_of_life_trapping() -> CardDefinition {
    let cast = R::WasCastFromHand
        .or(R::WasCastFromLibrary)
        .or(R::WasCastFromGraveyard)
        .or(R::WasCastFromExile);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(cast),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::CardExiledWithSource,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOfMoved,
                        tapped: false,
                    },
                },
                Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::ExileWithSourceStamp,
                },
            ]),
        }],
        ..spell(
            "Mirror of Life Trapping",
            cost(&[generic(4)]),
            CardType::Artifact,
            Effect::Noop,
        )
    }
}

/// Reckless Brute — {2}{R} 3/1 haste Ogre Warrior that attacks each combat if
/// able.
pub fn reckless_brute() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::MustAttack],
        ..creature(
            "Reckless Brute",
            cost(&[generic(2), r()]),
            vec![CreatureType::Ogre, CreatureType::Warrior],
            3,
            1,
        )
    }
}

/// Thresher Lizard — {2}{R} 3/2 Lizard; +1/+2 with one or fewer cards in hand.
pub fn thresher_lizard() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+2 as long as you have one or fewer cards in hand.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtMost(Value::HandSizeOf(PlayerRef::You), Value::ONE),
                power: 1,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..creature(
            "Thresher Lizard",
            cost(&[generic(2), r()]),
            vec![CreatureType::Lizard],
            3,
            2,
        )
    }
}

/// Burning Hands — {1}{R} instant. 2 damage to target creature or
/// planeswalker; 6 if it's green.
pub fn burning_hands() -> CardDefinition {
    spell(
        "Burning Hands",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::DealDamage {
            to: target_filtered(R::Creature.or(R::Planeswalker)),
            amount: Value::IfPred {
                pred: Box::new(Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasColor(Color::Green),
                }),
                then: Box::new(Value::Const(6)),
                else_: Box::new(Value::Const(2)),
            },
        },
    )
}

// ── MACH-1, Swooping Scoundrel ──────────────────────────────────────────────

/// Desperate Farmer // Depraved Harvester — {2}{B} 2/2 lifelink Human
/// Peasant; another creature of yours dying transforms it into a 4/3
/// lifelink Human Knight.
pub fn desperate_farmer() -> CardDefinition {
    let back = CardDefinition {
        keywords: vec![Keyword::Lifelink],
        color_indicator: vec![Color::Black],
        ..creature(
            "Depraved Harvester",
            ManaCost::default(),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            3,
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::Transform {
                what: Selector::This,
            },
        }],
        back_face: Some(Box::new(back)),
        ..creature(
            "Desperate Farmer",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Peasant],
            2,
            2,
        )
    }
}

/// Reverent Howl — {2}{B} instant. Choose one: target player draws two and
/// loses 2 life; or target creature gets +2/+2 and lifelink until end of turn.
pub fn reverent_howl() -> CardDefinition {
    spell(
        "Reverent Howl",
        cost(&[generic(2), b()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Draw {
                    who: target_filtered(R::Player),
                    amount: Value::Const(2),
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                },
            ]),
            Effect::Seq(vec![
                pump(target_filtered(R::Creature), 2, 2),
                grant(Selector::Target(0), Keyword::Lifelink),
            ]),
        ]),
    )
}

/// Alms of the Vein — {2}{B} sorcery, madness {B}. Target opponent loses 3
/// life and you gain 3.
pub fn alms_of_the_vein() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Madness(cost(&[b()]))],
        ..spell(
            "Alms of the Vein",
            cost(&[generic(2), b()]),
            CardType::Sorcery,
            Effect::Drain {
                from: target_filtered(R::OpponentPlayer),
                to: Selector::You,
                amount: Value::Const(3),
            },
        )
    }
}
