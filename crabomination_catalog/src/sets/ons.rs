//! Onslaught (ONS) — 2002, the tribal block's opener: Clerics, Wizards,
//! Soldiers, Goblins, Elves and Beasts, plus Morph and Cycling. Tests in
//! `classic_sets/ons`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Selector,
    StaticEffect, Value, ZoneDest,
    shortcut::{draw, etb, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, c, types, p, t) }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// A land whose only printed mana ability is `{T}: Add {C}`.
fn colorless_land(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The number of `ty` creatures on the battlefield, any controller.
fn tribe_count(ty: CreatureType) -> Value {
    Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::HasCreatureType(ty))),
        filter: R::Any,
    }
}

fn insect_token() -> TokenDefinition {
    TokenDefinition {
        name: "Insect".into(),
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

// ── White ───────────────────────────────────────────────────────────────────

/// Aven Brigadier — a Bird lord that is also a Soldier lord.
pub fn aven_brigadier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Other Bird creatures get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Bird).and(R::OtherThanSource),
                    ),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Other Soldier creatures get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Soldier).and(R::OtherThanSource),
                    ),
                    power: 1,
                    toughness: 1,
                },
            },
        ],
        ..creature(
            "Aven Brigadier",
            cost(&[generic(3), w(), w(), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            3,
            5,
        )
    }
}

/// Crowd Favorites — a big Soldier that taps blockers or braces itself.
pub fn crowd_favorites() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), w()]),
                effect: Effect::Tap { what: target_filtered(R::Creature) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), w()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(0),
                    toughness: Value::Const(5),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Crowd Favorites",
            cost(&[generic(6), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Defensive Maneuvers — a tribal fog for the type you name.
pub fn defensive_maneuvers() -> CardDefinition {
    instant(
        "Defensive Maneuvers",
        cost(&[generic(3), w()]),
        Effect::Seq(vec![
            Effect::NameCreatureType { what: Selector::This },
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsSourceChosenCreatureType)),
                power: Value::Const(0),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Gustcloak Runner — slips out of every block it walks into.
pub fn gustcloak_runner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Untap this creature and remove it from combat?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Untap { what: Selector::This, up_to: None },
                    Effect::RemoveFromCombat { what: Selector::This },
                ])),
            },
        }],
        ..creature(
            "Gustcloak Runner",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Inspirit — a combat trick that also unlocks a tapped blocker.
pub fn inspirit() -> CardDefinition {
    instant(
        "Inspirit",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Mobilization — vigilant Soldiers and a token factory.
pub fn mobilization() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Soldier creatures have vigilance.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Soldier)),
                keyword: Keyword::Vigilance,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                definition: TokenDefinition {
                    name: "Soldier".into(),
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Soldier],
                        ..Default::default()
                    },
                    card_types: vec![CardType::Creature],
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                },
                count: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Mobilization", cost(&[generic(2), w()]))
    }
}

/// Nova Cleric — a 1/2 body stapled to a Disenchant for the whole board.
pub fn nova_cleric() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy { what: Selector::EachPermanent(R::Enchantment) },
            ..Default::default()
        }],
        ..creature(
            "Nova Cleric",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Righteous Cause — every attack anywhere pays you a life.
pub fn righteous_cause() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..enchantment("Righteous Cause", cost(&[generic(3), w(), w()]))
    }
}

/// True Believer — you can't be targeted at all.
pub fn true_believer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You have shroud.",
            effect: StaticEffect::ControllerHasShroud,
        }],
        ..creature(
            "True Believer",
            cost(&[w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Daru Encampment — a colorless land that pumps a Soldier.
pub fn daru_encampment() -> CardDefinition {
    CardDefinition {
        activated_abilities: {
            let mut abs = colorless_land("Daru Encampment").activated_abilities;
            abs.push(ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                effect: Effect::PumpPT {
                    what: target_filtered(R::HasCreatureType(CreatureType::Soldier)),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            });
            abs
        },
        ..colorless_land("Daru Encampment")
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Airborne Aid — a Bird count in cards.
pub fn airborne_aid() -> CardDefinition {
    sorcery(
        "Airborne Aid",
        cost(&[generic(3), u()]),
        Effect::Draw { who: Selector::You, amount: tribe_count(CreatureType::Bird) },
    )
}

/// Blatant Thievery — one permanent from every opponent.
pub fn blatant_thievery() -> CardDefinition {
    sorcery(
        "Blatant Thievery",
        cost(&[generic(4), u(), u(), u()]),
        Effect::GainControl { what: Selector::Target(0), to: None, duration: Duration::Permanent },
    )
}

/// Crafty Pathmage — small creatures walk right past blockers.
pub fn crafty_pathmage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Crafty Pathmage",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Dispersing Orb — a repeatable bounce paid in permanents.
pub fn dispersing_orb() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            sac_other_filter: Some((R::Permanent, 1)),
            effect: Effect::Move {
                what: target_filtered(R::Permanent),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..enchantment("Dispersing Orb", cost(&[generic(3), u(), u()]))
    }
}

/// Reminisce — recycle a graveyard back into its library.
pub fn reminisce() -> CardDefinition {
    sorcery(
        "Reminisce",
        cost(&[generic(2), u()]),
        Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::Target(0) },
    )
}

/// Riptide Laboratory — the Wizard's escape hatch.
pub fn riptide_laboratory() -> CardDefinition {
    CardDefinition {
        activated_abilities: {
            let mut abs = colorless_land("Riptide Laboratory").activated_abilities;
            abs.push(ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(
                        R::HasCreatureType(CreatureType::Wizard).and(R::ControlledByYou),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                ..Default::default()
            });
            abs
        },
        ..colorless_land("Riptide Laboratory")
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Accursed Centaur — a 2/2 for {B} that eats a creature on the way in.
pub fn accursed_centaur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::You,
            filter: R::Creature,
            count: Value::ONE,
        })],
        ..creature(
            "Accursed Centaur",
            cost(&[b()]),
            vec![CreatureType::Zombie, CreatureType::Centaur],
            2,
            2,
        )
    }
}

/// Aphetto Dredging — buy back a tribe from the graveyard.
pub fn aphetto_dredging() -> CardDefinition {
    sorcery(
        "Aphetto Dredging",
        cost(&[generic(3), b()]),
        Effect::Seq(vec![
            Effect::NameCreatureType { what: Selector::This },
            Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::Creature.and(R::IsSourceChosenCreatureType),
                    }),
                    count: Box::new(Value::Const(3)),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
    )
}

/// Cabal Archon — Clerics as a drain engine.
pub fn cabal_archon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Cleric), 1)),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::Target(0)),
                to: Selector::You,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..creature(
            "Cabal Archon",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Cabal Slaver — every Goblin connection strips a card.
pub fn cabal_slaver() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer)
                .dealt_by(R::HasCreatureType(CreatureType::Goblin)),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..creature(
            "Cabal Slaver",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            1,
        )
    }
}

/// Disciple of Malice — a white-proof Cleric that cycles when it's dead weight.
pub fn disciple_of_malice() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Protection(Color::White),
            Keyword::Cycling(cost(&[generic(2)])),
        ],
        ..creature(
            "Disciple of Malice",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Infest — a two-point sweeper.
pub fn infest() -> CardDefinition {
    sorcery(
        "Infest",
        cost(&[generic(1), b(), b()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Screeching Buzzard — a flier that takes a card with it.
pub fn screeching_buzzard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..creature("Screeching Buzzard", cost(&[generic(3), b()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Visara the Dreadful — a repeatable, regeneration-proof kill.
pub fn visara_the_dreadful() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DestroyNoRegen { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..legend(
            "Visara the Dreadful",
            cost(&[generic(3), b(), b(), b()]),
            vec![CreatureType::Gorgon],
            5,
            5,
        )
    }
}

/// Wretched Anurid — a {1}{B} 3/3 that bleeds you for every arrival.
pub fn wretched_anurid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
            effect: Effect::LoseLife { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Wretched Anurid",
            cost(&[generic(1), b()]),
            vec![CreatureType::Zombie, CreatureType::Frog, CreatureType::Beast],
            3,
            3,
        )
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Aggravated Assault — buy an extra combat every turn.
pub fn aggravated_assault() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r(), r()]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Untap {
                    what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                    up_to: None,
                },
                Effect::AdditionalCombatPhaseAfterMain { count: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..enchantment("Aggravated Assault", cost(&[generic(2), r()]))
    }
}

/// Brightstone Ritual — one red per Goblin in play.
pub fn brightstone_ritual() -> CardDefinition {
    instant(
        "Brightstone Ritual",
        cost(&[r()]),
        Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColor(Color::Red, tribe_count(CreatureType::Goblin)),
        },
    )
}

/// Grand Melee — nobody sits a combat out.
pub fn grand_melee() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "All creatures attack each combat if able.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature),
                    keyword: Keyword::MustAttack,
                },
            },
            StaticAbility {
                description: "All creatures block each combat if able.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature),
                    keyword: Keyword::MustBlock,
                },
            },
        ],
        ..enchantment("Grand Melee", cost(&[generic(3), r()]))
    }
}

/// Rorix Bladewing — six hasty flying power.
pub fn rorix_bladewing() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        ..legend(
            "Rorix Bladewing",
            cost(&[generic(3), r(), r(), r()]),
            vec![CreatureType::Dragon],
            6,
            5,
        )
    }
}

/// Searing Flesh — seven to the face.
pub fn searing_flesh() -> CardDefinition {
    sorcery(
        "Searing Flesh",
        cost(&[generic(6), r()]),
        Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(7) },
    )
}

/// Wave of Indifference — X creatures stay home.
pub fn wave_of_indifference() -> CardDefinition {
    sorcery(
        "Wave of Indifference",
        cost(&[x(), r()]),
        Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                }),
            }),
        },
    )
}

/// Goblin Burrows — a colorless land that swings a Goblin.
pub fn goblin_burrows() -> CardDefinition {
    CardDefinition {
        activated_abilities: {
            let mut abs = colorless_land("Goblin Burrows").activated_abilities;
            abs.push(ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                tap_cost: true,
                effect: Effect::PumpPT {
                    what: target_filtered(R::HasCreatureType(CreatureType::Goblin)),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            });
            abs
        },
        ..colorless_land("Goblin Burrows")
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Barkhide Mauler — a 4/4 that cycles when you'd rather have a card.
pub fn barkhide_mauler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..creature("Barkhide Mauler", cost(&[generic(4), g()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Centaur Glade — a repeatable 3/3.
pub fn centaur_glade() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), g()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                definition: TokenDefinition {
                    name: "Centaur".into(),
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Centaur],
                        ..Default::default()
                    },
                    card_types: vec![CardType::Creature],
                    power: 3,
                    toughness: 3,
                    ..Default::default()
                },
                count: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Centaur Glade", cost(&[generic(3), g(), g()]))
    }
}

/// Elvish Scrapper — an Elf that eats an artifact.
pub fn elvish_scrapper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Artifact) },
            ..Default::default()
        }],
        ..creature("Elvish Scrapper", cost(&[g()]), vec![CreatureType::Elf], 1, 1)
    }
}

/// Krosan Groundshaker — a 6/6 that hands trample around the Beast tribe.
pub fn krosan_groundshaker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::HasCreatureType(CreatureType::Beast)),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Krosan Groundshaker",
            cost(&[generic(4), g(), g(), g()]),
            vec![CreatureType::Beast],
            6,
            6,
        )
    }
}

/// Mythic Proportions — +8/+8 and trample.
pub fn mythic_proportions() -> CardDefinition {
    CardDefinition {
        name: "Mythic Proportions",
        cost: cost(&[generic(4), g(), g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 8,
            toughness: 8,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Overwhelming Instinct — a card for every wide attack.
pub fn overwhelming_instinct() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(
                Predicate::ValueAtLeast(
                    Value::CountMatching {
                        sel: Box::new(Selector::ControlledBy {
                            who: PlayerRef::You,
                            filter: R::Creature.and(R::IsAttacking),
                        }),
                        filter: R::Any,
                    },
                    Value::Const(3),
                ),
            ),
            effect: draw(1),
        }],
        ..enchantment("Overwhelming Instinct", cost(&[generic(2), g()]))
    }
}

/// Silklash Spider — a wall that shoots down the sky.
pub fn silklash_spider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), g(), g()]),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(R::HasKeyword(Keyword::Flying)),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::XFromCost,
                }),
            },
            ..Default::default()
        }],
        ..creature("Silklash Spider", cost(&[generic(3), g(), g()]), vec![CreatureType::Spider], 2, 7)
    }
}

/// Silvos, Rogue Elemental — an 8/5 trampler that shrugs off removal.
pub fn silvos_rogue_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..legend(
            "Silvos, Rogue Elemental",
            cost(&[generic(3), g(), g(), g()]),
            vec![CreatureType::Elemental],
            8,
            5,
        )
    }
}

/// Symbiotic Beast — four Insects when it dies.
pub fn symbiotic_beast() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                definition: insect_token(),
                count: Value::Const(4),
            },
        }],
        ..creature(
            "Symbiotic Beast",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Insect, CreatureType::Beast],
            4,
            4,
        )
    }
}

/// Symbiotic Wurm — seven Insects when it dies.
pub fn symbiotic_wurm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                definition: insect_token(),
                count: Value::Const(7),
            },
        }],
        ..creature(
            "Symbiotic Wurm",
            cost(&[generic(5), g(), g(), g()]),
            vec![CreatureType::Wurm],
            7,
            7,
        )
    }
}

/// Tribal Unity — +X/+X to the type you name.
pub fn tribal_unity() -> CardDefinition {
    instant(
        "Tribal Unity",
        cost(&[x(), generic(2), g()]),
        Effect::Seq(vec![
            Effect::NameCreatureType { what: Selector::This },
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsSourceChosenCreatureType)),
                power: Value::XFromCost,
                toughness: Value::XFromCost,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Wall of Mulch — a 0/4 that trades Walls for cards.
pub fn wall_of_mulch() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Wall), 1)),
            effect: draw(1),
            ..Default::default()
        }],
        ..creature("Wall of Mulch", cost(&[generic(1), g()]), vec![CreatureType::Wall], 0, 4)
    }
}

/// Wirewood Elf — an Elf that taps for green.
pub fn wirewood_elf() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Green]),
            },
            ..Default::default()
        }],
        ..creature(
            "Wirewood Elf",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            1,
            2,
        )
    }
}

/// Wirewood Savage — a card for every Beast that shows up.
pub fn wirewood_savage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Beast),
                }),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(draw(1)),
            },
        }],
        ..creature("Wirewood Savage", cost(&[generic(2), g()]), vec![CreatureType::Elf], 2, 2)
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Slate of Ancestry — trade your hand for a card per creature.
pub fn slate_of_ancestry() -> CardDefinition {
    CardDefinition {
        name: "Slate of Ancestry",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            discard_hand_cost: true,
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::CountMatching {
                    sel: Box::new(Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::Creature,
                    }),
                    filter: R::Any,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
