//! Champions of Kamigawa (CHK) gap batch 1 — the Myojin cycle, the
//! don't-untap dual lands, the legends and the Spirit/Arcane payoffs. Tests in
//! `classic_sets/chk_gaps`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    Selector, SelectionRequirement as R, SpellSubtype, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

fn creature(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, power, toughness, types, keywords)
    }
}

/// An Aura with the printed enchant filter and continuous grant.
fn aura(name: &'static str, mana: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, mana)
    }
}

/// The CHK "painless dual": `{T}: Add {C}` plus a two-colour tap that costs the
/// land its next untap.
fn slow_dual(name: &'static str, a: Color, b_color: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::OfColors(vec![a, b_color], Value::ONE),
                    },
                    Effect::SkipNextUntap { what: Selector::This },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// A Myojin: enters with a divinity counter if cast, is indestructible while it
/// has one, and cashes the counter in for `payoff`.
fn myojin(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    payoff: Effect,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::TriggerSourceEnteredByCast,
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Divinity,
                amount: Value::ONE,
            }),
            else_: Box::new(Effect::Noop),
        })],
        static_abilities: vec![StaticAbility {
            description: "Indestructible while it has a divinity counter",
            effect: StaticEffect::SelfHasKeywordWhileCountersAtLeast {
                kind: CounterType::Divinity,
                n: 1,
                keyword: Keyword::Indestructible,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Divinity, 1)),
            effect: payoff,
            ..Default::default()
        }],
        ..legend(name, mana, power, toughness, vec![CreatureType::Spirit], vec![])
    }
}

/// "Whenever you cast a Spirit or Arcane spell, [effect]."
fn on_spirit_or_arcane(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::CastSpellMatches(
                R::HasCreatureType(CreatureType::Spirit)
                    .or(R::HasSpellSubtype(SpellSubtype::Arcane)),
            ),
        ),
        effect,
    }
}

// ── Lands ──

/// Cloudcrest Lake — {W}/{U} at the cost of an untap.
pub fn cloudcrest_lake() -> CardDefinition {
    slow_dual("Cloudcrest Lake", Color::White, Color::Blue)
}

/// Lantern-Lit Graveyard — the {B}/{R} slow dual.
pub fn lantern_lit_graveyard() -> CardDefinition {
    slow_dual("Lantern-Lit Graveyard", Color::Black, Color::Red)
}

/// Pinecrest Ridge — the {R}/{G} slow dual.
pub fn pinecrest_ridge() -> CardDefinition {
    slow_dual("Pinecrest Ridge", Color::Red, Color::Green)
}

/// Tranquil Garden — the {G}/{W} slow dual.
pub fn tranquil_garden() -> CardDefinition {
    slow_dual("Tranquil Garden", Color::Green, Color::White)
}

/// Waterveil Cavern — the {U}/{B} slow dual.
pub fn waterveil_cavern() -> CardDefinition {
    slow_dual("Waterveil Cavern", Color::Blue, Color::Black)
}

/// Forbidden Orchard — any colour, but it hands them a Spirit.
pub fn forbidden_orchard() -> CardDefinition {
    CardDefinition {
        name: "Forbidden Orchard",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                Effect::CreateToken {
                    who: PlayerRef::Target(0),
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Spirit".into(),
                        card_types: vec![CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Spirit],
                            ..Default::default()
                        },
                        power: 1,
                        toughness: 1,
                        ..Default::default()
                    },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hall of the Bandit Lord — three life for a hasty {C}.
pub fn hall_of_the_bandit_lord() -> CardDefinition {
    CardDefinition {
        name: "Hall of the Bandit Lord",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Enters tapped",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 3,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Untaidake, the Cloud Keeper — two life for two legendary-only {C}.
pub fn untaidake_the_cloud_keeper() -> CardDefinition {
    CardDefinition {
        name: "Untaidake, the Cloud Keeper",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Enters tapped",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 2,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::Const(2))),
                    crate::mana::SpendRestriction::LegendarySpell,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── The Myojin cycle ──

/// Myojin of Cleansing Fire — a one-shot wrath off its divinity counter.
pub fn myojin_of_cleansing_fire() -> CardDefinition {
    myojin(
        "Myojin of Cleansing Fire",
        cost(&[generic(5), w(), w(), w()]),
        4,
        6,
        Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
        },
    )
}

/// Myojin of Infinite Rage — Armageddon off its divinity counter.
pub fn myojin_of_infinite_rage() -> CardDefinition {
    myojin(
        "Myojin of Infinite Rage",
        cost(&[generic(7), r(), r(), r()]),
        7,
        4,
        Effect::Destroy { what: Selector::EachPermanent(R::Land) },
    )
}

/// Myojin of Life's Web — dumps your hand's creatures onto the table.
pub fn myojin_of_lifes_web() -> CardDefinition {
    myojin(
        "Myojin of Life's Web",
        cost(&[generic(6), g(), g(), g()]),
        8,
        8,
        Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: R::Creature,
            count: Value::Const(99),
            tapped: false,
            haste: false,
            sacrifice_eot: false,
        },
    )
}

/// Myojin of Seeing Winds — a card per permanent you control.
pub fn myojin_of_seeing_winds() -> CardDefinition {
    myojin(
        "Myojin of Seeing Winds",
        cost(&[generic(7), u(), u(), u()]),
        3,
        3,
        Effect::Draw {
            who: Selector::You,
            amount: Value::CountOf(Box::new(Selector::EachPermanent(R::ControlledByYou))),
        },
    )
}

// ── Legends ──

/// Azami, Lady of Scrolls — every Wizard taps for a card.
pub fn azami_lady_of_scrolls() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature.and(R::HasCreatureType(CreatureType::Wizard))),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..legend(
            "Azami, Lady of Scrolls",
            cost(&[generic(2), u(), u(), u()]),
            0,
            2,
            vec![CreatureType::Human, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Dosan the Falling Leaf — nobody gets to act on anyone else's turn.
pub fn dosan_the_falling_leaf() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players can cast spells only during their own turns",
            effect: StaticEffect::OpponentsCantCastDuringYourTurn,
        }],
        ..legend(
            "Dosan the Falling Leaf",
            cost(&[generic(1), g(), g()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Monk],
            vec![],
        )
    }
}

/// Iname, Death Aspect — stocks the graveyard with Spirits.
pub fn iname_death_aspect() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::HasCreatureType(CreatureType::Spirit),
            count: Value::Const(99),
            to: ZoneDest::Graveyard,
        })],
        ..legend(
            "Iname, Death Aspect",
            cost(&[generic(4), b(), b()]),
            4,
            4,
            vec![CreatureType::Spirit],
            vec![],
        )
    }
}

/// Iname, Life Aspect — cashes itself in for the Spirits it left behind.
pub fn iname_life_aspect() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::MayDo {
            description: "Exile Iname to return Spirit cards from your graveyard?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::ExileSource,
                Effect::ReturnGraveyardCardsToHand {
                    filter: R::HasCreatureType(CreatureType::Spirit),
                    max: Value::Const(99),
                },
            ])),
        })],
        ..legend(
            "Iname, Life Aspect",
            cost(&[generic(4), g(), g()]),
            4,
            4,
            vec![CreatureType::Spirit],
            vec![],
        )
    }
}

/// Sachi, Daughter of Seshiro — Snakes get tougher, Shamans make mana.
pub fn sachi_daughter_of_seshiro() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Other Snakes you control get +0/+1",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasCreatureType(CreatureType::Snake))
                        .and(R::OtherThanSource),
                    power: 0,
                    toughness: 1,
                    keywords: vec![],
                    opponents: false,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
            StaticAbility {
                description: "Shamans you control have \"{T}: Add {G}{G}\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::EachPermanent(
                        R::Creature
                            .and(R::ControlledByYou)
                            .and(R::HasCreatureType(CreatureType::Shaman)),
                    ),
                    ability: ActivatedAbility {
                        tap_cost: true,
                        effect: Effect::AddMana {
                            who: PlayerRef::You,
                            pool: ManaPayload::OfColor(Color::Green, Value::Const(2)),
                        },
                        ..Default::default()
                    },
                    condition: None,
                },
            },
        ],
        ..legend(
            "Sachi, Daughter of Seshiro",
            cost(&[generic(2), g(), g()]),
            1,
            3,
            vec![CreatureType::Snake, CreatureType::Shaman],
            vec![],
        )
    }
}

/// Shisato, Whispering Hunter — eats a Snake a turn, locks their untap step.
pub fn shisato_whispering_hunter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::SelfSource,
                )
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Snake)),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::SelfSource,
                ),
                effect: Effect::SkipPlayerUntapStep { player: PlayerRef::DefendingPlayer },
            },
        ],
        ..legend(
            "Shisato, Whispering Hunter",
            cost(&[generic(3), g()]),
            2,
            2,
            vec![CreatureType::Snake, CreatureType::Warrior],
            vec![],
        )
    }
}

/// The Unspeakable — a huge flier that rebuys your Arcane spells.
pub fn the_unspeakable() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Return an Arcane card from your graveyard?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::InYourGraveyard.and(R::HasSpellSubtype(SpellSubtype::Arcane)),
                    },
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..legend(
            "The Unspeakable",
            cost(&[generic(6), u(), u(), u()]),
            6,
            7,
            vec![CreatureType::Spirit],
            vec![Keyword::Flying, Keyword::Trample],
        )
    }
}

// ── Creatures ──

/// Ore Gorger — every Spirit or Arcane spell threatens a nonbasic land.
pub fn ore_gorger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_spirit_or_arcane(Effect::MayDo {
            description: "Destroy target nonbasic land?".into(),
            body: Box::new(Effect::Destroy {
                what: target_filtered(R::Land.and(R::IsNonbasicLand)),
            }),
        })],
        ..creature(
            "Ore Gorger",
            cost(&[generic(3), r(), r()]),
            3,
            1,
            vec![CreatureType::Spirit],
            vec![],
        )
    }
}

/// Rootrunner — buries a land, then hands a Spirit back.
pub fn rootrunner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g()]),
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Land),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: crate::effect::LibraryPosition::Top,
                },
            },
            ..Default::default()
        }],
        triggered_abilities: vec![crate::effect::shortcut::soulshift(3)],
        ..creature(
            "Rootrunner",
            cost(&[generic(2), g(), g()]),
            3,
            3,
            vec![CreatureType::Spirit],
            vec![],
        )
    }
}

/// Pious Kitsune — a devotion counter a turn, cashed in for life.
pub fn pious_kitsune() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Divinity,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Divinity, 1)),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Pious Kitsune",
            cost(&[generic(2), w()]),
            1,
            2,
            vec![CreatureType::Fox, CreatureType::Cleric],
            vec![],
        )
    }
}

// ── Spells ──

/// Thoughtbind — a counter for the cheap half of their deck.
pub fn thoughtbind() -> CardDefinition {
    CardDefinition {
        name: "Thoughtbind",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::IsSpellOnStack.and(R::ManaValueAtMost(4)),
            },
        },
        ..Default::default()
    }
}

/// Hisoka's Defiance — the Kamigawa-flavoured counterspell.
pub fn hisokas_defiance() -> CardDefinition {
    CardDefinition {
        name: "Hisoka's Defiance",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpell {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::IsSpellOnStack.and(
                    R::HasCreatureType(CreatureType::Spirit)
                        .or(R::HasSpellSubtype(SpellSubtype::Arcane)),
                ),
            },
        },
        ..Default::default()
    }
}

/// Cranial Extraction — names a card and strips it from everywhere.
pub fn cranial_extraction() -> CardDefinition {
    CardDefinition {
        name: "Cranial Extraction",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Arcane],
            ..Default::default()
        },
        effect: Effect::NameCardExileMatchingAllZones,
        ..Default::default()
    }
}

/// Mana Seism — trades your lands for a pile of {C}.
pub fn mana_seism() -> CardDefinition {
    CardDefinition {
        name: "Mana Seism",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificeAnyNumber {
            filter: R::Land.and(R::ControlledByYou),
        }],
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colorless(Value::SacrificedCount),
        },
        ..Default::default()
    }
}

/// Devouring Rage — every Spirit you feed it is +3/+0.
pub fn devouring_rage() -> CardDefinition {
    CardDefinition {
        name: "Devouring Rage",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Arcane],
            ..Default::default()
        },
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificeAnyNumber {
            filter: R::Creature
                .and(R::ControlledByYou)
                .and(R::HasCreatureType(CreatureType::Spirit)),
        }],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Sum(vec![
                Value::Const(3),
                Value::Times(Box::new(Value::Const(3)), Box::new(Value::SacrificedCount)),
            ]),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Sideswipe — repoints an Arcane spell.
pub fn sideswipe() -> CardDefinition {
    CardDefinition {
        name: "Sideswipe",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseNewTargetsForSpell {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::IsSpellOnStack.and(R::HasSpellSubtype(SpellSubtype::Arcane)),
            },
        },
        ..Default::default()
    }
}

// ── Enchantments ──

/// Night of Souls' Betrayal — the whole board shrinks.
pub fn night_of_souls_betrayal() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control get -1/-1",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature.and(R::ControlledByYou),
                    power: -1,
                    toughness: -1,
                    keywords: vec![],
                    opponents: false,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
            StaticAbility {
                description: "Creatures your opponents control get -1/-1",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Creature,
                    power: -1,
                    toughness: -1,
                    keywords: vec![],
                    opponents: true,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
        ],
        ..enchantment("Night of Souls' Betrayal", cost(&[generic(2), b(), b()]))
    }
}

/// Blood Rites — a repeatable sacrifice outlet with reach.
pub fn blood_rites() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..enchantment("Blood Rites", cost(&[generic(3), r(), r()]))
    }
}

/// Nature's Will — connecting locks their lands down.
pub fn natures_will() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ControllerDealtCombatDamage, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::EachPermanent(R::Land.and(R::ControlledByOpponent)),
                },
                Effect::Untap {
                    what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                    up_to: None,
                },
            ]),
        }],
        ..enchantment("Nature's Will", cost(&[generic(2), g(), g()]))
    }
}

/// Midnight Covenant — a firebreathing Aura in black.
pub fn midnight_covenant() -> CardDefinition {
    aura(
        "Midnight Covenant",
        cost(&[generic(1), b()]),
        EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Oni Possession — a big trampler that eats a creature every upkeep.
pub fn oni_possession() -> CardDefinition {
    aura(
        "Oni Possession",
        cost(&[generic(2), b()]),
        EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Trample],
            set_creature_types: Some(vec![CreatureType::Demon, CreatureType::Spirit]),
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::SelfSource,
                )
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature,
                },
            }],
            ..Default::default()
        },
    )
}

/// Ragged Veins — every point of damage the host takes bleeds its controller.
pub fn ragged_veins() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..aura(
            "Ragged Veins",
            cost(&[generic(1), b()]),
            EquipBonus {
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                    effect: Effect::LoseLife {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::This))),
                        amount: Value::TriggerEventAmount,
                    },
                }],
                ..Default::default()
            },
        )
    }
}

// ── Artifacts ──

/// Imi Statue — one artifact untap per player per turn.
pub fn imi_statue() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players can't untap more than one artifact each untap step",
            effect: StaticEffect::MaxOneArtifactUntap,
        }],
        ..artifact("Imi Statue", cost(&[generic(3)]))
    }
}

/// Hair-Strung Koto — taps your team to mill theirs.
pub fn hair_strung_koto() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature),
            effect: Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
            ..Default::default()
        }],
        ..artifact("Hair-Strung Koto", cost(&[generic(6)]))
    }
}

/// Honor-Worn Shaku — legends untap it for extra mana.
pub fn honor_worn_shaku() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_other_filter: Some(R::Permanent.and(R::HasSupertype(Supertype::Legendary))),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
        ],
        ..artifact("Honor-Worn Shaku", cost(&[generic(3)]))
    }
}

/// Orochi Hatchery — banks {X} and hatches a Snake per counter.
pub fn orochi_hatchery() -> CardDefinition {
    CardDefinition {
        cost: cost(&[crate::mana::x(), crate::mana::x()]),
        enters_with_counters: Some((CounterType::Charge, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                },
                definition: TokenDefinition {
                    name: "Snake".into(),
                    colors: vec![Color::Green],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Snake],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..artifact("Orochi Hatchery", ManaCost::default())
    }
}

/// Tenza, Godo's Maul — bigger on a legend, tramples in red.
pub fn tenza_godos_maul() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            conditional: vec![
                crate::card::ConditionalEquipBonus {
                    host_filter: R::HasSupertype(Supertype::Legendary),
                    power: 2,
                    toughness: 2,
                    keywords: vec![],
                    condition: None,
                },
                crate::card::ConditionalEquipBonus {
                    host_filter: R::HasColor(Color::Red),
                    power: 0,
                    toughness: 0,
                    keywords: vec![Keyword::Trample],
                    condition: None,
                },
            ],
            ..Default::default()
        }),
        ..artifact("Tenza, Godo's Maul", cost(&[generic(3)]))
    }
}

/// Hankyu — charge up an arrow, then loose it.
pub fn hankyu() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![
                ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddCounter {
                        what: Selector::AttachedToMe(Box::new(Selector::This)),
                        kind: CounterType::Charge,
                        amount: Value::ONE,
                    },
                    ..Default::default()
                },
                ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::DealDamage {
                        to: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                        amount: Value::CountersOn {
                            what: Box::new(Selector::AttachedToMe(Box::new(Selector::This))),
                            kind: CounterType::Charge,
                        },
                    },
                    ..Default::default()
                },
            ],
            ..Default::default()
        }),
        ..artifact("Hankyu", cost(&[generic(1)]))
    }
}
