//! Commander: the 2026-10-20 (`01E4rMUL`) two-gap pair closers — each pair
//! of cards here completes an EDHREC partner-pair average deck.
//! Tests in `tests/recent_b/cmdr_01e4_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, PlaneswalkerSubtype, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, WardCost, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w, x};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// An "Enchant creature" Aura with `bonus` on the enchanted creature.
fn aura(name: &'static str, mana: ManaCost, bonus: Option<EquipBonus>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: bonus,
        ..Default::default()
    }
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(n: i32, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(def) }
}

fn bounce_up_to(n: u8) -> Effect {
    Effect::ApplyToTargets {
        max_targets: n,
        min_targets: 0,
        filter: R::Creature,
        effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
    }
}

fn creature_cards_in_your_graveyard() -> Selector {
    Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature }
}

// ── Batch one ───────────────────────────────────────────────────────────────

/// Dragonloft Idol — {4} 3/3 artifact creature Gargoyle. As long as you
/// control a Dragon, it gets +1/+1 and has flying and trample.
pub fn dragonloft_idol() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "As long as you control a Dragon, this gets +1/+1 and has flying and trample.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Dragon).and(R::ControlledByYou),
                )),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Flying, Keyword::Trample],
            },
        }],
        ..creature("Dragonloft Idol", cost(&[generic(4)]), vec![CreatureType::Gargoyle], 3, 3)
    }
}

/// Scion of Ugin — {6} 4/4 colorless Dragon Spirit with flying.
pub fn scion_of_ugin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature("Scion of Ugin", cost(&[generic(6)]), vec![CreatureType::Dragon, CreatureType::Spirit], 4, 4)
    }
}

/// Burden of Guilt — {W} Aura. {1}: tap enchanted creature.
pub fn burden_of_guilt() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Tap { what: Selector::attached_to(Selector::This) },
            ..Default::default()
        }],
        ..aura("Burden of Guilt", cost(&[w()]), None)
    }
}

/// Into the Void — {3}{U} sorcery. Return up to two target creatures to
/// their owners' hands.
pub fn into_the_void() -> CardDefinition {
    spell("Into the Void", cost(&[generic(3), u()]), CardType::Sorcery, bounce_up_to(2))
}

/// Sandblast — {2}{W} instant. 5 damage to target attacking or blocking
/// creature.
pub fn sandblast() -> CardDefinition {
    spell(
        "Sandblast",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            amount: Value::Const(5),
        },
    )
}

/// Ironwright's Cleansing — {2}{W} sorcery. Exile target artifact or
/// enchantment.
pub fn ironwrights_cleansing() -> CardDefinition {
    spell(
        "Ironwright's Cleansing",
        cost(&[generic(2), w()]),
        CardType::Sorcery,
        Effect::Move { what: target_filtered(R::Artifact.or(R::Enchantment)), to: ZoneDest::Exile },
    )
}

/// Hydrolash — {2}{U} instant. Attacking creatures get -2/-0 until end of
/// turn. Draw a card.
pub fn hydrolash() -> CardDefinition {
    spell(
        "Hydrolash",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                power: Value::Const(-2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Captivating Gyre — {4}{U}{U} sorcery. Return up to three target creatures
/// to their owners' hands.
pub fn captivating_gyre() -> CardDefinition {
    spell("Captivating Gyre", cost(&[generic(4), u(), u()]), CardType::Sorcery, bounce_up_to(3))
}

/// Webweaver Changeling — {3}{G}{G} 3/5 Shapeshifter. Changeling, reach.
/// When it enters, if there are three or more creature cards in your
/// graveyard, you gain 5 life (CR 603.4 intervening if).
pub fn webweaver_changeling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling, Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(
                Predicate::SelectorCountAtLeast { sel: creature_cards_in_your_graveyard(), n: Value::Const(3) },
            ),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
        }],
        ..creature("Webweaver Changeling", cost(&[generic(3), g(), g()]), vec![CreatureType::Shapeshifter], 3, 5)
    }
}

/// Phyrexian Vivisector — {1}{B} 2/2 Phyrexian Human. Whenever a creature you
/// control dies, scry 1.
pub fn phyrexian_vivisector() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..creature(
            "Phyrexian Vivisector",
            cost(&[generic(1), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Human],
            2,
            2,
        )
    }
}

/// Demon's Due — {3}{B} instant. Scry 2, then draw two cards. You lose 2 life.
pub fn demons_due() -> CardDefinition {
    spell(
        "Demon's Due",
        cost(&[generic(3), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
    )
}

/// Deadeye Tormentor — {2}{B} 2/2 Human Pirate. Raid — when it enters, if you
/// attacked this turn, target opponent discards a card.
pub fn deadeye_tormentor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
            effect: Effect::Discard {
                who: target_filtered(R::OpponentPlayer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..creature("Deadeye Tormentor", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Pirate], 2, 2)
    }
}

/// Fathom Fleet Boarder — {2}{B} 3/3 Orc Pirate. When it enters, you lose 2
/// life unless you control another Pirate.
pub fn fathom_fleet_boarder() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(
                R::HasCreatureType(CreatureType::Pirate).and(R::ControlledByYou).and(R::OtherThanSource),
            )),
            then: Box::new(Effect::Noop),
            else_: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(2) }),
        })],
        ..creature("Fathom Fleet Boarder", cost(&[generic(2), b()]), vec![CreatureType::Orc, CreatureType::Pirate], 3, 3)
    }
}

/// Moonglove Winnower — {3}{B} 2/3 Elf Rogue with deathtouch.
pub fn moonglove_winnower() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        ..creature("Moonglove Winnower", cost(&[generic(3), b()]), vec![CreatureType::Elf, CreatureType::Rogue], 2, 3)
    }
}

/// Scarred Vinebreeder — {1}{B} 1/1 Elf Shaman. {2}{B}, exile an Elf card
/// from your graveyard: +3/+3 until end of turn.
pub fn scarred_vinebreeder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            exile_other_filter: Some((R::HasCreatureType(CreatureType::Elf), 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Scarred Vinebreeder", cost(&[generic(1), b()]), vec![CreatureType::Elf, CreatureType::Shaman], 1, 1)
    }
}

/// Sleep Paralysis — {3}{U} Aura. When it enters, tap enchanted creature. It
/// doesn't untap during its controller's untap step.
pub fn sleep_paralysis() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Tap { what: Selector::attached_to(Selector::This) })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::attached_to(Selector::This) },
        }],
        ..aura("Sleep Paralysis", cost(&[generic(3), u()]), None)
    }
}

/// Commencement of Festivities — {1}{G} instant. Prevent all combat damage
/// that would be dealt to players this turn.
pub fn commencement_of_festivities() -> CardDefinition {
    spell(
        "Commencement of Festivities",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::PreventAllCombatDamageToPlayerThisTurn { who: PlayerRef::EachPlayer },
    )
}

/// Chitinous Graspling — {3}{G/U} 3/4 Shapeshifter. Changeling, reach.
pub fn chitinous_graspling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling, Keyword::Reach],
        ..creature(
            "Chitinous Graspling",
            cost(&[generic(3), hybrid(Color::Green, Color::Blue)]),
            vec![CreatureType::Shapeshifter],
            3,
            4,
        )
    }
}

/// Consuming Fervor — {R} Aura. Enchanted creature gets +3/+3 and has "At the
/// beginning of your upkeep, put a -1/-1 counter on this creature."
pub fn consuming_fervor() -> CardDefinition {
    aura(
        "Consuming Fervor",
        cost(&[r()]),
        Some(EquipBonus {
            power: 3,
            toughness: 3,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        }),
    )
}

/// Dust Corona — {R} Aura. Enchanted creature gets +2/+0 and can't be
/// blocked by creatures with flying.
pub fn dust_corona() -> CardDefinition {
    aura(
        "Dust Corona",
        cost(&[r()]),
        Some(EquipBonus {
            power: 2,
            keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::HasKeyword(Keyword::Flying)))],
            ..Default::default()
        }),
    )
}

/// Seraph of the Suns — {5}{W}{W} 4/4 Angel. Flying, indestructible.
pub fn seraph_of_the_suns() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Indestructible],
        ..creature("Seraph of the Suns", cost(&[generic(5), w(), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Archangel's Light — {7}{W} sorcery. Gain 2 life for each card in your
/// graveyard, then shuffle your graveyard into your library.
pub fn archangels_light() -> CardDefinition {
    spell(
        "Archangel's Light",
        cost(&[generic(7), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(
                    Box::new(Value::GraveyardSizeOf(PlayerRef::You)),
                    Box::new(Value::Const(2)),
                ),
            },
            Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::You },
        ]),
    )
}

/// Psychic Symbiont — {4}{U}{B} 3/3 Nightmare Horror. Flying. When it enters,
/// target opponent discards a card and you draw a card.
pub fn psychic_symbiont() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Discard { who: target_filtered(R::OpponentPlayer), amount: Value::ONE, random: false },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]))],
        ..creature(
            "Psychic Symbiont",
            cost(&[generic(4), u(), b()]),
            vec![CreatureType::Nightmare, CreatureType::Horror],
            3,
            3,
        )
    }
}

/// Abnormal Endurance — {1}{B} instant. Until end of turn, target creature
/// gets +2/+0 and gains "When this creature dies, return it to the
/// battlefield tapped under its owner's control."
pub fn abnormal_endurance() -> CardDefinition {
    let revive = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
        effect: Effect::ReturnSelfTapped,
    };
    spell(
        "Abnormal Endurance",
        cost(&[generic(1), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(revive),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Luxurious Libation — {X}{G} instant. Target creature gets +X/+X until end
/// of turn. Create a 1/1 green and white Citizen creature token.
pub fn luxurious_libation() -> CardDefinition {
    spell(
        "Luxurious Libation",
        cost(&[x(), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::XFromCost,
                toughness: Value::XFromCost,
                duration: Duration::EndOfTurn,
            },
            make(1, token("Citizen", 1, 1, vec![Color::Green, Color::White], vec![CreatureType::Citizen])),
        ]),
    )
}

/// Otherworldly Outburst — {R} instant. Target creature gets +1/+0 until end
/// of turn. When that creature dies this turn, create a 3/2 colorless
/// Eldrazi Horror creature token.
pub fn otherworldly_outburst() -> CardDefinition {
    spell(
        "Otherworldly Outburst",
        cost(&[r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::WhenTargetDiesThisTurn {
                filter: None,
                slot: 0,
                body: Box::new(make(
                    1,
                    token("Eldrazi Horror", 3, 2, vec![], vec![CreatureType::Eldrazi, CreatureType::Horror]),
                )),
            },
        ]),
    )
}

// ── Batch two ───────────────────────────────────────────────────────────────

/// Sunset Pyramid — {2} artifact. Enters with three brick counters. {2},
/// {T}, remove a brick counter: draw a card. {2}, {T}: scry 1.
pub fn sunset_pyramid() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Brick, Value::Const(3))),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Brick, 1)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..spell("Sunset Pyramid", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Unexplained Vision — {4}{U} sorcery. Draw three cards. Adamant — scry 3 if
/// at least three blue mana was spent.
pub fn unexplained_vision() -> CardDefinition {
    spell(
        "Unexplained Vision",
        cost(&[generic(4), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast { color: Color::Blue, at_least: 3 },
                then: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Sequestered Stash — land. {T}: {C}. {4}, {T}, sacrifice it: mill five,
/// then you may put an artifact card from your graveyard on top of your
/// library.
pub fn sequestered_stash() -> CardDefinition {
    CardDefinition {
        name: "Sequestered Stash",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Mill { who: Selector::You, amount: Value::Const(5) },
                    Effect::MayDo {
                        description: "Put an artifact card from your graveyard on top of your library?".into(),
                        body: Box::new(Effect::ChooseOneAmong {
                            what: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Artifact },
                            chooser: PlayerRef::You,
                            chosen: Box::new(Effect::Move {
                                what: Selector::SeparatedPile { chosen: true },
                                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                            }),
                            other: Box::new(Effect::Noop),
                        }),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Corpse Hauler — {1}{B} 2/1 Human Rogue. {2}{B}, sacrifice it: return
/// target creature card from your graveyard to your hand.
pub fn corpse_hauler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature("Corpse Hauler", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 1)
    }
}

/// Totem-Guide Hartebeest — {4}{W} 2/5 Antelope. When it enters, you may
/// search your library for an Aura card and put it into your hand.
pub fn totem_guide_hartebeest() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::HasEnchantmentSubtype(EnchantmentSubtype::Aura),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature("Totem-Guide Hartebeest", cost(&[generic(4), w()]), vec![CreatureType::Antelope], 2, 5)
    }
}

/// Crystal Carapace — {3}{G} Aura. Enchanted creature gets +3/+3 and has
/// ward {2}. Cycling {2}.
pub fn crystal_carapace() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..aura(
            "Crystal Carapace",
            cost(&[generic(3), g()]),
            Some(EquipBonus {
                power: 3,
                toughness: 3,
                keywords: vec![Keyword::Ward(WardCost::generic(2))],
                ..Default::default()
            }),
        )
    }
}

/// Dromoka Monument — {3} artifact. {T}: {G} or {W}. {4}{G}{W}: a 4/4 green
/// and white flying Dragon artifact creature until end of turn.
pub fn dromoka_monument() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::Green, Color::White], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4), g(), w()]),
                effect: Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(4),
                        toughness: Value::Const(4),
                        creature_types: vec![CreatureType::Dragon],
                        keywords: vec![Keyword::Flying],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::BecomeColor {
                        what: Selector::This,
                        colors: vec![Color::Green, Color::White],
                        duration: Duration::EndOfTurn,
                        additive: false,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..spell("Dromoka Monument", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Arashin Sovereign — {5}{G}{W} 6/6 Dragon. Flying. When it dies, you may
/// put it on your choice of the top or bottom of its owner's library.
pub fn arashin_sovereign() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::MayDo {
            description: "Put Arashin Sovereign on the top or bottom of its owner's library?".into(),
            body: Box::new(Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::OwnerChoice },
            }),
        })],
        ..creature("Arashin Sovereign", cost(&[generic(5), g(), w()]), vec![CreatureType::Dragon], 6, 6)
    }
}

/// Void Stalker — {1}{U} 2/1 Elemental. {2}{U}, {T}: put it and target
/// creature on top of their owners' libraries, then those players shuffle.
pub fn void_stalker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        pos: LibraryPosition::Top,
                    },
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                },
                Effect::ShuffleLibrary { who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))) },
                Effect::ShuffleLibrary { who: PlayerRef::You },
            ]),
            ..Default::default()
        }],
        ..creature("Void Stalker", cost(&[generic(1), u()]), vec![CreatureType::Elemental], 2, 1)
    }
}

/// Hampering Snare — {1}{U} instant. Creatures your opponents control get
/// -2/-0 until end of turn. Cycling {2}.
pub fn hampering_snare() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..spell(
            "Hampering Snare",
            cost(&[generic(1), u()]),
            CardType::Instant,
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Saurian Symbiote — {3}{G} 2/3 Fungus Dinosaur. Reach. When it enters,
/// choose one — a +1/+1 counter on it; or a 1/1 green Saproling token.
pub fn saurian_symbiote() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            make(1, token("Saproling", 1, 1, vec![Color::Green], vec![CreatureType::Saproling])),
        ]))],
        ..creature(
            "Saurian Symbiote",
            cost(&[generic(3), g()]),
            vec![CreatureType::Fungus, CreatureType::Dinosaur],
            2,
            3,
        )
    }
}

/// Squad Captain — {4}{W} 2/2 Human Soldier. Vigilance. Enters with a +1/+1
/// counter for each other creature you control.
pub fn squad_captain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource))),
        )),
        ..creature("Squad Captain", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Strength in Numbers — {1}{G} instant. Until end of turn, target creature
/// gains trample and gets +X/+X, X the number of attacking creatures.
pub fn strength_in_numbers() -> CardDefinition {
    let attackers = || Value::count(Selector::EachPermanent(R::Creature.and(R::IsAttacking)));
    spell(
        "Strength in Numbers",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: attackers(),
                toughness: attackers(),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Ballroom Brawlers — {3}{W}{W} 3/5 Human Warrior. Whenever it attacks, it
/// and up to one other target creature you control both gain your choice of
/// first strike or lifelink until end of turn.
pub fn ballroom_brawlers() -> CardDefinition {
    let both = |keyword: Keyword| {
        Effect::Seq(vec![
            Effect::GrantKeyword { what: Selector::This, keyword: keyword.clone(), duration: Duration::EndOfTurn },
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword,
                    duration: Duration::EndOfTurn,
                }),
            },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![both(Keyword::FirstStrike), both(Keyword::Lifelink)]),
        }],
        ..creature(
            "Ballroom Brawlers",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            5,
        )
    }
}

/// Campaign of Vengeance — {3}{W}{B} enchantment. Whenever a creature you
/// control attacks, defending player loses 1 life and you gain 1 life.
pub fn campaign_of_vengeance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::DefendingPlayer), amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..spell("Campaign of Vengeance", cost(&[generic(3), w(), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Ghoulcaller's Accomplice — {1}{B} 2/2 Human Rogue. {3}{B}, exile it from
/// your graveyard: a 2/2 black Zombie token. Sorcery speed.
pub fn ghoulcallers_accomplice() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: make(1, token("Zombie", 2, 2, vec![Color::Black], vec![CreatureType::Zombie])),
            ..Default::default()
        }],
        ..creature(
            "Ghoulcaller's Accomplice",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Aven Surveyor — {3}{U}{U} 2/2 Bird Scout. Flying. When it enters, choose
/// one — a +1/+1 counter on it; or return target creature to its owner's
/// hand.
pub fn aven_surveyor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
        ]))],
        ..creature("Aven Surveyor", cost(&[generic(3), u(), u()]), vec![CreatureType::Bird, CreatureType::Scout], 2, 2)
    }
}

/// Shinewend — {1}{W} 0/0 Elemental. Flying. Enters with a +1/+1 counter.
/// {1}{W}, remove a +1/+1 counter from it: destroy target enchantment.
pub fn shinewend() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::Destroy { what: target_filtered(R::Enchantment) },
            ..Default::default()
        }],
        ..creature("Shinewend", cost(&[generic(1), w()]), vec![CreatureType::Elemental], 0, 0)
    }
}

/// Barrier of Bones — {B} 0/3 Skeleton Wall. Defender. When it enters,
/// surveil 1.
pub fn barrier_of_bones() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![etb(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
        ..creature("Barrier of Bones", cost(&[b()]), vec![CreatureType::Skeleton, CreatureType::Wall], 0, 3)
    }
}

/// Mournwillow — {1}{B}{G} 3/2 Plant Skeleton. Haste. Delirium — when it
/// enters, if there are four or more card types among cards in your
/// graveyard, creatures with power 2 or less can't block this turn.
pub fn mournwillow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::DeliriumActive { who: PlayerRef::You }),
            effect: Effect::MatchingCantBlockThisTurn { filter: R::Creature.and(R::PowerAtMost(2)) },
        }],
        ..creature("Mournwillow", cost(&[generic(1), b(), g()]), vec![CreatureType::Plant, CreatureType::Skeleton], 3, 2)
    }
}

/// Basri's Acolyte — {2}{W}{W} 2/3 Cat Cleric. Lifelink. When it enters, put
/// a +1/+1 counter on each of up to two other target creatures you control.
pub fn basris_acolyte() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        })],
        ..creature("Basri's Acolyte", cost(&[generic(2), w(), w()]), vec![CreatureType::Cat, CreatureType::Cleric], 2, 3)
    }
}

/// Lurking Roper — {2}{G} 4/5 Horror. Doesn't untap during your untap step.
/// Whenever you gain life, untap it.
pub fn lurking_roper() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..creature("Lurking Roper", cost(&[generic(2), g()]), vec![CreatureType::Horror], 4, 5)
    }
}

/// Lurking Chupacabra — {3}{B} 2/3 Beast Horror. Whenever a creature you
/// control explores, target creature an opponent controls gets -2/-2 until
/// end of turn.
pub fn lurking_chupacabra() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Explored, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Lurking Chupacabra", cost(&[generic(3), b()]), vec![CreatureType::Beast, CreatureType::Horror], 2, 3)
    }
}

/// Nefarious Imp — {2}{B} 2/1 Imp. Flying. Whenever one or more permanents
/// you control leave the battlefield, scry 1 (CR 603.2c: once a batch).
pub fn nefarious_imp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl).once_per_batch(),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..creature("Nefarious Imp", cost(&[generic(2), b()]), vec![CreatureType::Imp], 2, 1)
    }
}

/// Scepter of Empires — {3} artifact. {T}: 1 damage to target player or
/// planeswalker; 3 instead if you control artifacts named Crown of Empires
/// and Throne of Empires.
pub fn scepter_of_empires() -> CardDefinition {
    let have = |name: &str| {
        Predicate::SelectorExists(Selector::EachPermanent(
            R::Artifact.and(R::ControlledByYou).and(R::HasName(name.into())),
        ))
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::IfPred {
                    pred: Box::new(Predicate::All(vec![have("Crown of Empires"), have("Throne of Empires")])),
                    then: Box::new(Value::Const(3)),
                    else_: Box::new(Value::ONE),
                },
            },
            ..Default::default()
        }],
        ..spell("Scepter of Empires", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Bristlebane Outrider — {3}{G} 3/5 Kithkin Knight. Can't be blocked by
/// creatures with power 2 or less. As long as another creature entered under
/// your control this turn, it gets +2/+0.
pub fn bristlebane_outrider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::PowerAtMost(2)))],
        static_abilities: vec![StaticAbility {
            description: "As long as another creature entered the battlefield under your control this turn, this gets +2/+0.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::AnotherCreatureEnteredThisTurn { who: PlayerRef::You },
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..creature(
            "Bristlebane Outrider",
            cost(&[generic(3), g()]),
            vec![CreatureType::Kithkin, CreatureType::Knight],
            3,
            5,
        )
    }
}

// ── Batch three ─────────────────────────────────────────────────────────────

/// Safehold Sentry — {1}{W} 2/2 Elf Warrior. {2}{W}, {Q}: +0/+2 until end of
/// turn.
pub fn safehold_sentry() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            untap_self_cost: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Safehold Sentry", cost(&[generic(1), w()]), vec![CreatureType::Elf, CreatureType::Warrior], 2, 2)
    }
}

/// Scarblade Elite — {B}{B} 2/2 Elf Assassin. {T}, exile an Assassin card
/// from your graveyard: destroy target creature.
pub fn scarblade_elite() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_other_filter: Some((R::HasCreatureType(CreatureType::Assassin), 1)),
            effect: Effect::Destroy { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Scarblade Elite", cost(&[b(), b()]), vec![CreatureType::Elf, CreatureType::Assassin], 2, 2)
    }
}

/// Bone Pit Brute — {4}{R}{R} 4/5 Cyclops. Menace. When it enters, target
/// creature gets +4/+0 until end of turn.
pub fn bone_pit_brute() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(4),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Bone Pit Brute", cost(&[generic(4), r(), r()]), vec![CreatureType::Cyclops], 4, 5)
    }
}

/// Orazca Raptor — {2}{R}{R} 3/4 Dinosaur.
pub fn orazca_raptor() -> CardDefinition {
    creature("Orazca Raptor", cost(&[generic(2), r(), r()]), vec![CreatureType::Dinosaur], 3, 4)
}

/// Thrash of Raptors — {3}{R} 3/3 Dinosaur. As long as you control another
/// Dinosaur, it gets +2/+0 and has trample.
pub fn thrash_of_raptors() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you control another Dinosaur, this gets +2/+0 and has trample.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Dinosaur).and(R::ControlledByYou).and(R::OtherThanSource),
                )),
                power: 2,
                toughness: 0,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..creature("Thrash of Raptors", cost(&[generic(3), r()]), vec![CreatureType::Dinosaur], 3, 3)
    }
}

/// Sigil Blessing — {G}{W} instant. Until end of turn, target creature you
/// control gets +3/+3 and other creatures you control get +1/+1.
pub fn sigil_blessing() -> CardDefinition {
    spell(
        "Sigil Blessing",
        cost(&[g(), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::EachPermanentExceptTargets(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Spawning Breath — {1}{R} instant. 1 damage to any target; create a 0/1
/// Eldrazi Spawn token.
pub fn spawning_breath() -> CardDefinition {
    spell(
        "Spawning Breath",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::ONE },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::eldrazi_spawn_token()),
            },
        ]),
    )
}

/// Accomplished Automaton — {7} 5/7 artifact creature Construct. Fabricate 1.
pub fn accomplished_automaton() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![crate::effect::shortcut::fabricate(1)],
        ..creature("Accomplished Automaton", cost(&[generic(7)]), vec![CreatureType::Construct], 5, 7)
    }
}

/// Ajani's Comrade — {1}{G} 2/2 Elf Soldier. Trample. At the beginning of
/// combat on your turn, if you control an Ajani planeswalker, put a +1/+1
/// counter on it.
pub fn ajanis_comrade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl).with_filter(
                Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasPlaneswalkerType(PlaneswalkerSubtype::Ajani).and(R::ControlledByYou),
                )),
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Ajani's Comrade", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Soldier], 2, 2)
    }
}

/// Aurora Champion — {2}{W} 3/2 Elf Warrior. Whenever it attacks, if your
/// team controls another Warrior, tap target creature.
pub fn aurora_champion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::SelectorExists(
                Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Warrior).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
            )),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
        }],
        ..creature("Aurora Champion", cost(&[generic(2), w()]), vec![CreatureType::Elf, CreatureType::Warrior], 3, 2)
    }
}

/// Incremental Growth — {3}{G}{G} sorcery. One, two and three +1/+1 counters
/// on three different target creatures.
pub fn incremental_growth() -> CardDefinition {
    let slot = |n: u8, filter: R| Selector::TargetFiltered { slot: n, filter };
    let grow = |what: Selector, n: i32| Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: Value::Const(n) };
    spell(
        "Incremental Growth",
        cost(&[generic(3), g(), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            grow(target_filtered(R::Creature), 1),
            grow(slot(1, R::Creature.and(R::OtherThanTargetSlot(0))), 2),
            grow(slot(2, R::Creature.and(R::OtherThanTargetSlot(0)).and(R::OtherThanTargetSlot(1))), 3),
        ]),
    )
}

/// Inspiring Roar — {3}{W} sorcery. A +1/+1 counter on each creature you
/// control.
pub fn inspiring_roar() -> CardDefinition {
    spell(
        "Inspiring Roar",
        cost(&[generic(3), w()]),
        CardType::Sorcery,
        Effect::AddCounter {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
    )
}

/// Exuberant Wolfbear — {3}{G} 4/4 Wolf Bear. Whenever it attacks, you may
/// have the base power and toughness of target Human you control become equal
/// to its power and toughness until end of turn.
pub fn exuberant_wolfbear() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Make a Human as big as Exuberant Wolfbear?".into(),
                body: Box::new(Effect::SetBasePT {
                    what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Human)).and(R::ControlledByYou)),
                    power: Value::PowerOf(Box::new(Selector::This)),
                    toughness: Value::ToughnessOf(Box::new(Selector::This)),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..creature("Exuberant Wolfbear", cost(&[generic(3), g()]), vec![CreatureType::Wolf, CreatureType::Bear], 4, 4)
    }
}

/// Razorclaw Bear — {2}{G}{G} 3/3 Bear. Whenever it becomes blocked, it gets
/// +2/+2 until end of turn.
pub fn razorclaw_bear() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Razorclaw Bear", cost(&[generic(2), g(), g()]), vec![CreatureType::Bear], 3, 3)
    }
}

/// Awaken the Bear — {2}{G} instant. Target creature gets +3/+3 and gains
/// trample until end of turn.
pub fn awaken_the_bear() -> CardDefinition {
    spell(
        "Awaken the Bear",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Golgari Locket — {3} artifact. {T}: {B} or {G}. {B/G}{B/G}{B/G}{B/G},
/// {T}, sacrifice it: draw two cards.
pub fn golgari_locket() -> CardDefinition {
    let bg = || hybrid(Color::Black, Color::Green);
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::Black, Color::Green], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[bg(), bg(), bg(), bg()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..spell("Golgari Locket", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Elvish Eulogist — {G} 1/1 Elf Shaman. Sacrifice it: gain 1 life for each
/// Elf card in your graveyard.
pub fn elvish_eulogist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::HasCreatureType(CreatureType::Elf),
                }),
            },
            ..Default::default()
        }],
        ..creature("Elvish Eulogist", cost(&[g()]), vec![CreatureType::Elf, CreatureType::Shaman], 1, 1)
    }
}

/// Shi'ar Soldier — {1}{U} 1/3 Shi'ar Soldier. Flying. {U}, {T}: return
/// another target permanent you control to its owner's hand. Activate only
/// during your turn.
pub fn shiar_soldier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Move {
                what: target_filtered(R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature("Shi'ar Soldier", cost(&[generic(1), u()]), vec![CreatureType::Shiar, CreatureType::Soldier], 1, 3)
    }
}

/// Hard Cover — {U} Aura. Enchanted creature gets +0/+2 and has "{T}: Draw a
/// card, then discard a card."
pub fn hard_cover() -> CardDefinition {
    aura(
        "Hard Cover",
        cost(&[u()]),
        Some(EquipBonus {
            toughness: 2,
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        }),
    )
}

/// School Daze — {3}{U}{U} instant. Choose one — draw three cards; or counter
/// target spell and draw a card.
pub fn school_daze() -> CardDefinition {
    spell(
        "School Daze",
        cost(&[generic(3), u(), u()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Seq(vec![
                Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        ]),
    )
}

/// Great Hall of Starnheim — land, enters tapped. {T}: {B}. {W}{W}{B}, {T},
/// sacrifice it and a creature you control: a 4/4 white flying, vigilant
/// Angel Warrior token. Sorcery speed.
pub fn great_hall_of_starnheim() -> CardDefinition {
    let angel = TokenDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..token("Angel Warrior", 4, 4, vec![Color::White], vec![CreatureType::Angel, CreatureType::Warrior])
    };
    CardDefinition {
        name: "Great Hall of Starnheim",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Black),
            ActivatedAbility {
                mana_cost: cost(&[w(), w(), b()]),
                tap_cost: true,
                sac_cost: true,
                sac_other_filter: Some((R::Creature.and(R::ControlledByYou), 1)),
                sorcery_speed: true,
                effect: make(1, angel),
                ..Default::default()
            },
        ],
        static_abilities: vec![crate::sets::enters_tapped()],
        ..Default::default()
    }
}

/// Revelation of Power — {1}{W} instant. Target creature gets +2/+2 until
/// end of turn; if it has a counter on it, it also gains flying and lifelink.
pub fn revelation_of_power() -> CardDefinition {
    let grant = |keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword, duration: Duration::EndOfTurn };
    spell(
        "Revelation of Power",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::WithAnyCounter },
                then: Box::new(Effect::Seq(vec![grant(Keyword::Flying), grant(Keyword::Lifelink)])),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Llanowar Augur — {G} 0/3 Elf Shaman. Sacrifice it: target creature gets
/// +3/+3 and gains trample until end of turn. Activate only during your
/// upkeep.
pub fn llanowar_augur() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::CurrentStepIs(TurnStep::Upkeep),
            ])),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature("Llanowar Augur", cost(&[g()]), vec![CreatureType::Elf, CreatureType::Shaman], 0, 3)
    }
}

/// Oashra Cultivator — {G} 0/3 Human Druid. {2}{G}, {T}, sacrifice it: search
/// your library for a basic land card and put it onto the battlefield tapped.
pub fn oashra_cultivator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature("Oashra Cultivator", cost(&[g()]), vec![CreatureType::Human, CreatureType::Druid], 0, 3)
    }
}

/// Sultai Flayer — {3}{G} 3/4 Snake Shaman. Whenever a creature you control
/// with toughness 4 or greater dies, you gain 4 life.
pub fn sultai_flayer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::ToughnessAtLeast(4) },
            ),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        }],
        ..creature("Sultai Flayer", cost(&[generic(3), g()]), vec![CreatureType::Snake, CreatureType::Shaman], 3, 4)
    }
}

/// Saddle of the Cavalier — {2}{G} Equipment. Equipped creature gets +3/+3
/// and can't be blocked by creatures with power 3 or less. Equip {3}.
pub fn saddle_of_the_cavalier() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::PowerAtMost(3)))],
            ..Default::default()
        }),
        ..spell("Saddle of the Cavalier", cost(&[generic(2), g()]), CardType::Artifact, Effect::Noop)
    }
}

/// Ahn-Crop Champion — {2}{G}{W} 4/4 Human Warrior. You may exert it as it
/// attacks; when you do, untap all other creatures you control.
pub fn ahn_crop_champion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Exert],
        triggered_abilities: vec![crate::effect::shortcut::on_exert(Effect::Untap {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            up_to: None,
        })],
        ..creature("Ahn-Crop Champion", cost(&[generic(2), g(), w()]), vec![CreatureType::Human, CreatureType::Warrior], 4, 4)
    }
}

/// Paladin of the Bloodstained — {3}{W} 3/2 Vampire Knight. When it enters,
/// create a 1/1 white Vampire token with lifelink.
pub fn paladin_of_the_bloodstained() -> CardDefinition {
    let vampire = TokenDefinition {
        keywords: vec![Keyword::Lifelink],
        ..token("Vampire", 1, 1, vec![Color::White], vec![CreatureType::Vampire])
    };
    CardDefinition {
        triggered_abilities: vec![etb(make(1, vampire))],
        ..creature(
            "Paladin of the Bloodstained",
            cost(&[generic(3), w()]),
            vec![CreatureType::Vampire, CreatureType::Knight],
            3,
            2,
        )
    }
}

/// Loxodon Lifechanter — {5}{W} 4/6 Elephant Cleric. When it enters, you may
/// have your life total become the total toughness of creatures you control.
/// {5}{W}: +X/+X until end of turn, X your life total.
pub fn loxodon_lifechanter() -> CardDefinition {
    let life = || Value::LifeOf(PlayerRef::You);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Have your life total become the total toughness of your creatures?".into(),
            body: Box::new(Effect::SetLifeTotal { who: Selector::You, amount: Value::TotalToughnessControlled }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), w()]),
            effect: Effect::PumpPT { what: Selector::This, power: life(), toughness: life(), duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..creature(
            "Loxodon Lifechanter",
            cost(&[generic(5), w()]),
            vec![CreatureType::Elephant, CreatureType::Cleric],
            4,
            6,
        )
    }
}

/// Sandsteppe Mastodon — {5}{G}{G} 5/5 Elephant. Reach. When it enters,
/// bolster 5.
pub fn sandsteppe_mastodon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(crate::effect::shortcut::bolster(5))],
        ..creature("Sandsteppe Mastodon", cost(&[generic(5), g(), g()]), vec![CreatureType::Elephant], 5, 5)
    }
}

/// Aquitect's Defenses — {1}{U} Aura with flash. Enchant creature you
/// control. When it enters, enchanted creature gains hexproof until end of
/// turn. Enchanted creature gets +1/+2.
pub fn aquitects_defenses() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: Selector::attached_to(Selector::This),
            keyword: Keyword::Hexproof,
            duration: Duration::EndOfTurn,
        })],
        ..aura(
            "Aquitect's Defenses",
            cost(&[generic(1), u()]),
            Some(EquipBonus { power: 1, toughness: 2, ..Default::default() }),
        )
    }
}

/// Grasp of the Hieromancer — {1}{W} Aura. Enchanted creature gets +1/+1 and
/// has "Whenever this creature attacks, tap target creature defending player
/// controls."
pub fn grasp_of_the_hieromancer() -> CardDefinition {
    aura(
        "Grasp of the Hieromancer",
        cost(&[generic(1), w()]),
        Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)) },
            }],
            ..Default::default()
        }),
    )
}

/// Park Heights Pegasus — {G}{W} 2/1 Pegasus. Flying, trample. Whenever it
/// deals combat damage to a player, draw a card if you had two or more
/// creatures enter under your control this turn.
pub fn park_heights_pegasus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(Value::CreaturesEnteredThisTurn(PlayerRef::You), Value::Const(2)),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Park Heights Pegasus", cost(&[g(), w()]), vec![CreatureType::Pegasus], 2, 1)
    }
}

/// Vine Mare — {2}{G}{G} 5/3 Elemental Horse. Hexproof. Can't be blocked by
/// black creatures.
pub fn vine_mare() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof, Keyword::CantBeBlockedBy(Box::new(R::HasColor(Color::Black)))],
        ..creature("Vine Mare", cost(&[generic(2), g(), g()]), vec![CreatureType::Elemental, CreatureType::Horse], 5, 3)
    }
}

/// Legion's Judgment — {2}{W} sorcery. Destroy target creature with power 4
/// or greater.
pub fn legions_judgment() -> CardDefinition {
    spell(
        "Legion's Judgment",
        cost(&[generic(2), w()]),
        CardType::Sorcery,
        Effect::Destroy { what: target_filtered(R::Creature.and(R::PowerAtLeast(4))) },
    )
}

/// Bloodbriar — {2}{G} 2/3 Plant Elemental. Whenever you sacrifice another
/// permanent, put a +1/+1 counter on it.
pub fn bloodbriar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource },
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Bloodbriar", cost(&[generic(2), g()]), vec![CreatureType::Plant, CreatureType::Elemental], 2, 3)
    }
}

/// Illicit Shipment — {3}{B}{B} sorcery. Casualty 3. Search your library for
/// a card and put it into your hand.
pub fn illicit_shipment() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Casualty(3)],
        ..spell(
            "Illicit Shipment",
            cost(&[generic(3), b(), b()]),
            CardType::Sorcery,
            Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) },
        )
    }
}

/// Elvish Branchbender — {2}{G} 2/2 Elf Druid. {T}: until end of turn, target
/// Forest becomes an X/X Treefolk creature in addition to its other types, X
/// the number of Elves you control.
pub fn elvish_branchbender() -> CardDefinition {
    let elves = || Value::count(Selector::EachPermanent(R::HasCreatureType(CreatureType::Elf).and(R::ControlledByYou)));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeCreature {
                what: target_filtered(R::HasLandType(LandType::Forest)),
                power: elves(),
                toughness: elves(),
                creature_types: vec![CreatureType::Treefolk],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Elvish Branchbender", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Druid], 2, 2)
    }
}

// ── Batch four ──────────────────────────────────────────────────────────────

/// Infernal Scarring — {1}{B} Aura. Enchanted creature gets +2/+0 and has
/// "When this creature dies, draw a card."
pub fn infernal_scarring() -> CardDefinition {
    aura(
        "Infernal Scarring",
        cost(&[generic(1), b()]),
        Some(EquipBonus {
            power: 2,
            triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            })],
            ..Default::default()
        }),
    )
}

/// Blood Price — {3}{B} sorcery. Look at the top four cards of your library,
/// put two into your hand and the rest on the bottom. You lose 2 life.
pub fn blood_price() -> CardDefinition {
    spell(
        "Blood Price",
        cost(&[generic(3), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                who: PlayerRef::You,
                count: Value::Const(4),
                take: Some(Value::Const(2)),
                ..Default::default()
            })),
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
    )
}

/// Soul Salvage — {2}{B} sorcery. Return up to two target creature cards
/// from your graveyard to your hand.
pub fn soul_salvage() -> CardDefinition {
    spell(
        "Soul Salvage",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::InGraveyard).and(R::OwnedByYou),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
        },
    )
}

/// Heavyweight Demolisher — {7} 8/6 artifact creature Construct. Menace. At
/// the beginning of your upkeep, tap it unless you pay {3}. Unearth {6}{R}{R}.
pub fn heavyweight_demolisher() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {3} or tap Heavyweight Demolisher?".into(),
                mana_cost: cost(&[generic(3)]),
                body: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::Tap { what: Selector::This })),
            },
        }],
        activated_abilities: vec![crate::effect::shortcut::unearth(cost(&[generic(6), r(), r()]))],
        ..creature("Heavyweight Demolisher", cost(&[generic(7)]), vec![CreatureType::Construct], 8, 6)
    }
}

/// Goremand — {4}{B}{B} 5/5 Demon. As an additional cost, sacrifice a
/// creature. Flying, trample. When it enters, each opponent sacrifices a
/// creature.
pub fn goremand() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature.and(R::ControlledByYou),
            count: 1,
        }],
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::ONE,
            filter: R::Creature,
        })],
        ..creature("Goremand", cost(&[generic(4), b(), b()]), vec![CreatureType::Demon], 5, 5)
    }
}

/// Rakshasa Gravecaller — {4}{B} 3/6 Demon. Exploit. When it exploits a
/// creature, create two 2/2 black Zombie tokens.
pub fn rakshasa_gravecaller() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::exploit(make(
            2,
            token("Zombie", 2, 2, vec![Color::Black], vec![CreatureType::Zombie]),
        ))],
        ..creature("Rakshasa Gravecaller", cost(&[generic(4), b()]), vec![CreatureType::Demon], 3, 6)
    }
}

/// Sharpshooter Elf — {2}{G} */2 Elf Ranger. Reach. Its power is the number
/// of creatures you control. When it enters, it deals damage equal to its
/// power to target creature with flying an opponent controls.
pub fn sharpshooter_elf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        dynamic_pt: Some(crate::card::DynamicPt::CreaturesYouControl { base_t: 2 }),
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying)).and(R::ControlledByOpponent)),
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        ..creature("Sharpshooter Elf", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Ranger], 0, 2)
    }
}

/// Enlarge — {3}{G}{G} sorcery. Target creature gets +7/+7 and gains trample
/// until end of turn. It must be blocked this turn if able.
pub fn enlarge() -> CardDefinition {
    let grant = |keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword, duration: Duration::EndOfTurn };
    spell(
        "Enlarge",
        cost(&[generic(3), g(), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(7),
                toughness: Value::Const(7),
                duration: Duration::EndOfTurn,
            },
            grant(Keyword::Trample),
            grant(Keyword::MustBeBlocked),
        ]),
    )
}

/// Wolfrider's Saddle — {3}{G} Equipment. When it enters, create a 2/2 green
/// Wolf token and attach this to it. Equipped creature gets +1/+1 and can't be
/// blocked by more than one creature. Equip {3}.
pub fn wolfriders_saddle() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            make(1, token("Wolf", 2, 2, vec![Color::Green], vec![CreatureType::Wolf])),
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedTokens },
        ]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
            ..Default::default()
        }),
        ..spell("Wolfrider's Saddle", cost(&[generic(3), g()]), CardType::Artifact, Effect::Noop)
    }
}

/// Briarpack Alpha — {3}{G} 3/3 Wolf with flash. When it enters, target
/// creature gets +2/+2 until end of turn.
pub fn briarpack_alpha() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Briarpack Alpha", cost(&[generic(3), g()]), vec![CreatureType::Wolf], 3, 3)
    }
}

/// Morselhoarder — {4}{R/G}{R/G} 6/4 Elemental. Enters with two -1/-1
/// counters. Remove a -1/-1 counter from it: add one mana of any color.
pub fn morselhoarder() -> CardDefinition {
    let rg = || hybrid(Color::Red, Color::Green);
    CardDefinition {
        enters_with_counters: Some((CounterType::MinusOneMinusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::MinusOneMinusOne, 1)),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..creature("Morselhoarder", cost(&[generic(4), rg(), rg()]), vec![CreatureType::Elemental], 6, 4)
    }
}

/// Bridge from Below — {B}{B}{B} enchantment that works from your graveyard:
/// a nontoken creature put into your graveyard from the battlefield makes a
/// 2/2 black Zombie; a creature put into an opponent's graveyard from the
/// battlefield exiles this card.
pub fn bridge_from_below() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::FromYourGraveyardAnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken.and(R::OwnedByYou) },
                ),
                effect: make(1, token("Zombie", 2, 2, vec![Color::Black], vec![CreatureType::Zombie])),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::FromYourGraveyardAnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Not(Box::new(R::OwnedByYou)) },
                ),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Exile },
            },
        ],
        ..spell("Bridge from Below", cost(&[b(), b(), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Sumala Rumblers — {2}{G/W}{G/W} */4 Wurm. Power equal to the number of
/// creatures you control. Myriad.
pub fn sumala_rumblers() -> CardDefinition {
    let gw = || hybrid(Color::Green, Color::White);
    CardDefinition {
        dynamic_pt: Some(crate::card::DynamicPt::CreaturesYouControl { base_t: 4 }),
        triggered_abilities: vec![crate::effect::shortcut::myriad()],
        ..creature("Sumala Rumblers", cost(&[generic(2), gw(), gw()]), vec![CreatureType::Wurm], 0, 4)
    }
}

/// Sigil of the Nayan Gods — {1}{G}{W} Aura. Enchanted creature gets +1/+1
/// for each creature you control. Cycling {G/W}.
pub fn sigil_of_the_nayan_gods() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[hybrid(Color::Green, Color::White)]))],
        ..aura(
            "Sigil of the Nayan Gods",
            cost(&[generic(1), g(), w()]),
            Some(EquipBonus {
                scale: Some(crate::card::EquipScale {
                    filter: R::Creature,
                    per_power: 1,
                    per_toughness: 1,
                    ..Default::default()
                }),
                ..Default::default()
            }),
        )
    }
}

/// Excavation Elephant — {4}{W} 3/5 Elephant. Kicker {1}{W}. When it enters,
/// if it was kicked, return target artifact card from your graveyard to your
/// hand.
pub fn excavation_elephant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), w()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasKicked),
            effect: Effect::Move {
                what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..creature("Excavation Elephant", cost(&[generic(4), w()]), vec![CreatureType::Elephant], 3, 5)
    }
}

/// Frontier Mastodon — {2}{G} 3/2 Elephant. Ferocious — enters with a +1/+1
/// counter if you control a creature with power 4 or greater.
pub fn frontier_mastodon() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)).and(R::OtherThanSource),
                ))),
                then: Box::new(Value::ONE),
                else_: Box::new(Value::Const(0)),
            },
        )),
        ..creature("Frontier Mastodon", cost(&[generic(2), g()]), vec![CreatureType::Elephant], 3, 2)
    }
}

/// Mosstodon — {4}{G} 5/3 Plant Elephant. {1}: target creature with power 5
/// or greater gains trample until end of turn.
pub fn mosstodon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtLeast(5))),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Mosstodon", cost(&[generic(4), g()]), vec![CreatureType::Plant, CreatureType::Elephant], 5, 3)
    }
}

/// Howlgeist — {5}{G} 4/2 Spirit Wolf. Creatures with power less than its
/// power can't block it. Undying.
pub fn howlgeist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerLess, Keyword::Undying],
        ..creature("Howlgeist", cost(&[generic(5), g()]), vec![CreatureType::Spirit, CreatureType::Wolf], 4, 2)
    }
}

/// Lone Wolf — {2}{G} 2/2 Wolf. It may assign its combat damage as though it
/// weren't blocked.
pub fn lone_wolf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::AssignsDamageAsThoughUnblocked],
        ..creature("Lone Wolf", cost(&[generic(2), g()]), vec![CreatureType::Wolf], 2, 2)
    }
}

/// Wandering Wolf — {1}{G} 2/1 Wolf. Creatures with power less than its power
/// can't block it.
pub fn wandering_wolf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerLess],
        ..creature("Wandering Wolf", cost(&[generic(1), g()]), vec![CreatureType::Wolf], 2, 1)
    }
}

/// Esper Sojourners — {W}{U}{B} 2/3 artifact creature Vedalken Wizard. When
/// you cycle it and when it dies, you may tap or untap target permanent.
/// Cycling {2}{U}.
pub fn esper_sojourners() -> CardDefinition {
    let tap_or_untap = || {
        Effect::ChooseMode(vec![
            Effect::Tap { what: target_filtered(R::Permanent) },
            Effect::Untap { what: target_filtered(R::Permanent), up_to: None },
        ])
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Cycling(cost(&[generic(2), u()]))],
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource), effect: tap_or_untap() },
            crate::effect::shortcut::on_dies(tap_or_untap()),
        ],
        ..creature(
            "Esper Sojourners",
            cost(&[w(), u(), b()]),
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Pale Recluse — {4}{G}{W} 4/5 Spider. Reach. Forestcycling {2},
/// plainscycling {2}.
pub fn pale_recluse() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Reach,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Forest),
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Plains),
        ],
        ..creature("Pale Recluse", cost(&[generic(4), g(), w()]), vec![CreatureType::Spider], 4, 5)
    }
}

/// Akki War Paint — {R} Aura. Enchant artifact or creature; as long as the
/// enchanted permanent is a creature, it gets +2/+1.
pub fn akki_war_paint() -> CardDefinition {
    CardDefinition {
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Artifact.or(R::Creature)) },
        ..aura("Akki War Paint", cost(&[r()]), Some(EquipBonus { power: 2, toughness: 1, ..Default::default() }))
    }
}

/// Bestial Fury — {2}{R} Aura. When it enters, draw a card at the beginning
/// of the next turn's upkeep. Whenever enchanted creature becomes blocked, it
/// gets +4/+0 and gains trample until end of turn.
pub fn bestial_fury() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AtNextTurnsUpkeep {
            body: Box::new(Effect::Draw { who: Selector::Player(PlayerRef::You), amount: Value::ONE }),
        })],
        ..aura(
            "Bestial Fury",
            cost(&[generic(2), r()]),
            Some(EquipBonus {
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                    effect: Effect::Seq(vec![
                        Effect::PumpPT {
                            what: Selector::This,
                            power: Value::Const(4),
                            toughness: Value::Const(0),
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                    ]),
                }],
                ..Default::default()
            }),
        )
    }
}

/// Imposing Visage — {R} Aura. Enchanted creature has menace.
pub fn imposing_visage() -> CardDefinition {
    aura(
        "Imposing Visage",
        cost(&[r()]),
        Some(EquipBonus { keywords: vec![Keyword::Menace], ..Default::default() }),
    )
}

// ── Batch five ──────────────────────────────────────────────────────────────

fn pump_self(p: i32, t: i32) -> Effect {
    Effect::PumpPT { what: Selector::This, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

/// Feral Ridgewolf — {2}{R} 1/2 Wolf. Trample. {1}{R}: +2/+0 until end of
/// turn.
pub fn feral_ridgewolf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: pump_self(2, 0),
            ..Default::default()
        }],
        ..creature("Feral Ridgewolf", cost(&[generic(2), r()]), vec![CreatureType::Wolf], 1, 2)
    }
}

/// Kessig Wolf — {2}{R} 3/1 Wolf. {1}{R}: first strike until end of turn.
pub fn kessig_wolf() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..creature("Kessig Wolf", cost(&[generic(2), r()]), vec![CreatureType::Wolf], 3, 1)
    }
}

/// Wolf Pack — {6}{G}{G} 7/6 Wolf. It may assign its combat damage as though
/// it weren't blocked.
pub fn wolf_pack() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::AssignsDamageAsThoughUnblocked],
        ..creature("Wolf Pack", cost(&[generic(6), g(), g()]), vec![CreatureType::Wolf], 7, 6)
    }
}

/// Artful Maneuver — {1}{W} instant. Target creature gets +2/+2 until end of
/// turn. Rebound.
pub fn artful_maneuver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Rebound],
        ..spell(
            "Artful Maneuver",
            cost(&[generic(1), w()]),
            CardType::Instant,
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Taigam's Strike — {3}{U} sorcery. Target creature gets +2/+0 until end of
/// turn and can't be blocked this turn. Rebound.
pub fn taigams_strike() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Rebound],
        ..spell(
            "Taigam's Strike",
            cost(&[generic(3), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            ]),
        )
    }
}

/// Galestrike — {2}{U} instant. Return target tapped creature to its owner's
/// hand. Draw a card.
pub fn galestrike() -> CardDefinition {
    spell(
        "Galestrike",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Creature.and(R::Tapped)), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Glacial Grasp — {2}{U} instant. Tap target creature; its controller mills
/// two; it doesn't untap during its controller's next untap step. Draw a
/// card.
pub fn glacial_grasp() -> CardDefinition {
    spell(
        "Glacial Grasp",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::Mill {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(2),
            },
            Effect::SkipNextUntap { what: Selector::Target(0) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Send to Sleep — {1}{U} instant. Tap up to two target creatures. Spell
/// mastery — they don't untap during their controllers' next untap steps.
pub fn send_to_sleep() -> CardDefinition {
    spell(
        "Send to Sleep",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::If {
                    cond: crate::effect::shortcut::spell_mastery_gate(),
                    then: Box::new(Effect::SkipNextUntap { what: Selector::Target(0) }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        },
    )
}

/// Artificer's Dragon — {6} 4/4 artifact creature Dragon. Flying. {R}:
/// artifact creatures you control get +1/+0 until end of turn. Unearth
/// {3}{R}{R}.
pub fn artificers_dragon() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(R::Artifact.and(R::Creature).and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            crate::effect::shortcut::unearth(cost(&[generic(3), r(), r()])),
        ],
        ..creature("Artificer's Dragon", cost(&[generic(6)]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Excavation Explosion — {2}{R} sorcery. 3 damage to any target. Create a
/// tapped Powerstone token.
pub fn excavation_explosion() -> CardDefinition {
    let powerstone = TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() };
    spell(
        "Excavation Explosion",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(3) },
            make(1, powerstone),
        ]),
    )
}

/// Kathari Bomber — {1}{B}{R} 2/2 Bird Shaman. Flying. When it deals combat
/// damage to a player, create two 1/1 red Goblin tokens and sacrifice it.
/// Unearth {3}{B}{R}.
pub fn kathari_bomber() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                make(2, token("Goblin", 1, 1, vec![Color::Red], vec![CreatureType::Goblin])),
                Effect::SacrificeSource,
            ]),
        }],
        activated_abilities: vec![crate::effect::shortcut::unearth(cost(&[generic(3), b(), r()]))],
        ..creature("Kathari Bomber", cost(&[generic(1), b(), r()]), vec![CreatureType::Bird, CreatureType::Shaman], 2, 2)
    }
}

/// Dose of Dawnglow — {4}{B} instant. Return target creature card from your
/// graveyard to the battlefield; then, if it isn't your main phase, blight 2.
pub fn dose_of_dawnglow() -> CardDefinition {
    spell(
        "Dose of Dawnglow",
        cost(&[generic(4), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.from_your_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::If {
                cond: Predicate::YourMainPhase,
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Blight { n: Value::Const(2) }),
            },
        ]),
    )
}

/// Lurking Crocodile — {2}{G} 2/2 Crocodile. Bloodthirst 1. Islandwalk.
pub fn lurking_crocodile() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Bloodthirst(1), Keyword::Landwalk(LandType::Island)],
        ..creature("Lurking Crocodile", cost(&[generic(2), g()]), vec![CreatureType::Crocodile], 2, 2)
    }
}

/// Lost in the Spirit World — {2}{U} sorcery. Return up to one target
/// creature to its owner's hand. Create a 1/1 colorless Spirit token that
/// can't block or be blocked by non-Spirit creatures.
pub fn lost_in_the_spirit_world() -> CardDefinition {
    let spirit = TokenDefinition {
        keywords: vec![
            Keyword::CantBlockMatching(Box::new(R::HasCreatureType(CreatureType::Spirit).negate())),
            Keyword::CantBeBlockedExceptBy(Box::new(R::HasCreatureType(CreatureType::Spirit))),
        ],
        ..token("Spirit", 1, 1, vec![], vec![CreatureType::Spirit])
    };
    spell(
        "Lost in the Spirit World",
        cost(&[generic(2), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![bounce_up_to(1), make(1, spirit)]),
    )
}

/// Deadeye Rig-Hauler — {3}{U} 3/2 Human Pirate. Raid — when it enters, if
/// you attacked this turn, you may return target creature to its owner's
/// hand.
pub fn deadeye_rig_hauler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
            effect: Effect::MayDo {
                description: "Return target creature to its owner's hand?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        }],
        ..creature("Deadeye Rig-Hauler", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Pirate], 3, 2)
    }
}

/// Embereth Skyblazer — {3}{R} 4/3 Human Knight. Flying during your turn.
/// Whenever it attacks, you may pay {2}{R}; if you do, creatures you control
/// get +X/+0 until end of turn, X the number of opponents you have.
pub fn embereth_skyblazer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has flying.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Flying,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {2}{R} to pump your creatures?".into(),
                mana_cost: cost(&[generic(2), r()]),
                body: Box::new(Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::OpponentCount,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: None,
            },
        }],
        ..creature("Embereth Skyblazer", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Knight], 4, 3)
    }
}

/// Youthful Scholar — {3}{U} 2/2 Human Wizard. When it dies, draw two cards.
pub fn youthful_scholar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..creature("Youthful Scholar", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2)
    }
}

/// Vicious Offering — {1}{B} instant. Kicker — sacrifice a creature. Target
/// creature gets -2/-2 until end of turn, -5/-5 instead if kicked.
pub fn vicious_offering() -> CardDefinition {
    let shrink = |n: i32| Effect::PumpPT {
        what: target_filtered(R::Creature),
        power: Value::Const(-n),
        toughness: Value::Const(-n),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        kicker_action_cost: Some(crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature.and(R::ControlledByYou),
            count: 1,
        }),
        ..spell(
            "Vicious Offering",
            cost(&[generic(1), b()]),
            CardType::Instant,
            Effect::If { cond: Predicate::SpellWasKicked, then: Box::new(shrink(5)), else_: Box::new(shrink(2)) },
        )
    }
}

/// Cragsmasher Yeti — {4}{R}{R} 4/2 Yeti. Mountaincycling {2}. Backup 2.
/// Trample.
pub fn cragsmasher_yeti() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Landcycling(cost(&[generic(2)]), LandType::Mountain)],
        triggered_abilities: vec![crate::effect::shortcut::backup(2, vec![Keyword::Trample])],
        ..creature("Cragsmasher Yeti", cost(&[generic(4), r(), r()]), vec![CreatureType::Yeti], 4, 2)
    }
}

/// Igneous Pouncer — {4}{B}{R} 5/1 Elemental. Haste. Swampcycling {2},
/// mountaincycling {2}.
pub fn igneous_pouncer() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Haste,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Swamp),
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Mountain),
        ],
        ..creature("Igneous Pouncer", cost(&[generic(4), b(), r()]), vec![CreatureType::Elemental], 5, 1)
    }
}

/// Blitz Leech — {5}{B} 5/2 Leech with flash. When it enters, target creature
/// an opponent controls gets -2/-2 until end of turn; remove all counters
/// from it.
pub fn blitz_leech() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::RemoveAllCounters { what: Selector::Target(0) },
        ]))],
        ..creature("Blitz Leech", cost(&[generic(5), b()]), vec![CreatureType::Leech], 5, 2)
    }
}

/// Ravenous Gigamole — {3}{B} 2/3 Mole Horror. When it enters, mill three;
/// you may put a creature card milled this way into your hand, else a +1/+1
/// counter on it.
pub fn ravenous_gigamole() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MillThenToHand {
            amount: Value::Const(3),
            filter: R::Creature,
            otherwise: Some(Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            })),
        })],
        ..creature("Ravenous Gigamole", cost(&[generic(3), b()]), vec![CreatureType::Mole, CreatureType::Horror], 2, 3)
    }
}

// ── Batch six ───────────────────────────────────────────────────────────────

/// Furystoke Giant — {3}{R}{R} 3/3 Giant Warrior. When it enters, other
/// creatures you control gain "{T}: This creature deals 2 damage to any
/// target" until end of turn. Persist.
pub fn furystoke_giant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Persist],
        triggered_abilities: vec![etb(Effect::GrantActivatedAbilityToMatching {
            filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
            ability: Box::new(ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(2) },
                ..Default::default()
            }),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Furystoke Giant", cost(&[generic(3), r(), r()]), vec![CreatureType::Giant, CreatureType::Warrior], 3, 3)
    }
}

/// Gravelgill Axeshark — {4}{U/B} 3/3 Merfolk Soldier. Persist.
pub fn gravelgill_axeshark() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Persist],
        ..creature(
            "Gravelgill Axeshark",
            cost(&[generic(4), hybrid(Color::Blue, Color::Black)]),
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Lingering Tormentor — {3}{B} 2/2 Spirit. Fear. Persist.
pub fn lingering_tormentor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear, Keyword::Persist],
        ..creature("Lingering Tormentor", cost(&[generic(3), b()]), vec![CreatureType::Spirit], 2, 2)
    }
}

/// Mistvein Borderpost — {1}{U}{B} artifact. You may pay {1} and return a
/// basic land you control to its owner's hand rather than pay its mana cost.
/// Enters tapped. {T}: {U} or {B}.
pub fn mistvein_borderpost() -> CardDefinition {
    CardDefinition {
        name: "Mistvein Borderpost",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Artifact],
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(1)]),
            return_to_hand: Some((R::IsBasicLand, 1)),
            ..Default::default()
        }),
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![crate::sets::tap_add(Color::Blue), crate::sets::tap_add(Color::Black)],
        ..Default::default()
    }
}

/// Mindwrack Demon — {2}{B}{B} 4/5 Demon. Flying, trample. When it enters,
/// mill four. Delirium — at the beginning of your upkeep, you lose 4 life
/// unless there are four or more card types among cards in your graveyard.
pub fn mindwrack_demon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![
            etb(Effect::Mill { who: Selector::You, amount: Value::Const(4) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::If {
                    cond: Predicate::DeliriumActive { who: PlayerRef::You },
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(4) }),
                },
            },
        ],
        ..creature("Mindwrack Demon", cost(&[generic(2), b(), b()]), vec![CreatureType::Demon], 4, 5)
    }
}

/// Soulstinger — {3}{B} 4/5 Scorpion Demon. When it enters, put two -1/-1
/// counters on target creature you control. When it dies, you may put a
/// -1/-1 counter on target creature for each -1/-1 counter on it.
pub fn soulstinger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::Const(2),
            }),
            crate::effect::shortcut::on_dies(Effect::MayDo {
                description: "Move Soulstinger's -1/-1 counters onto target creature?".into(),
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::MinusOneMinusOne },
                }),
            }),
        ],
        ..creature("Soulstinger", cost(&[generic(3), b()]), vec![CreatureType::Scorpion, CreatureType::Demon], 4, 5)
    }
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// Avacyn's Collar — {1} Equipment. Equipped creature gets +1/+0 and has
/// vigilance. Whenever it dies, if it was a Human, create a 1/1 white flying
/// Spirit token. Equip {2}.
pub fn avacyns_collar() -> CardDefinition {
    let spirit = TokenDefinition {
        keywords: vec![Keyword::Flying],
        ..token("Spirit", 1, 1, vec![Color::White], vec![CreatureType::Spirit])
    };
    equipment(
        "Avacyn's Collar",
        cost(&[generic(1)]),
        cost(&[generic(2)]),
        EquipBonus {
            power: 1,
            keywords: vec![Keyword::Vigilance],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource).with_filter(
                    Predicate::EntityMatches { what: Selector::This, filter: R::HasCreatureType(CreatureType::Human) },
                ),
                effect: make(1, spirit),
            }],
            ..Default::default()
        },
    )
}

/// Bladed Bracers — {1} Equipment. Equipped creature gets +1/+1; as long as
/// it's a Human or an Angel, it has vigilance. Equip {2}.
pub fn bladed_bracers() -> CardDefinition {
    equipment(
        "Bladed Bracers",
        cost(&[generic(1)]),
        cost(&[generic(2)]),
        EquipBonus {
            power: 1,
            toughness: 1,
            conditional: vec![crate::card::ConditionalEquipBonus {
                host_filter: R::HasCreatureType(CreatureType::Human).or(R::HasCreatureType(CreatureType::Angel)),
                keywords: vec![Keyword::Vigilance],
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// True-Faith Censer — {2} Equipment. Equipped creature gets +1/+1 and has
/// vigilance; a Human gets an additional +1/+0. Equip {2}.
pub fn true_faith_censer() -> CardDefinition {
    equipment(
        "True-Faith Censer",
        cost(&[generic(2)]),
        cost(&[generic(2)]),
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Vigilance],
            conditional: vec![crate::card::ConditionalEquipBonus {
                host_filter: R::HasCreatureType(CreatureType::Human),
                power: 1,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Invasion of Dominaria // Serra Faithkeeper — {2}{W} Siege, defense 5.
/// When it enters, you gain 4 life and draw a card. Back: a 4/4 white Angel
/// with flying and vigilance.
pub fn invasion_of_dominaria() -> CardDefinition {
    let faithkeeper = CardDefinition {
        color_indicator: vec![Color::White],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..creature("Serra Faithkeeper", ManaCost::default(), vec![CreatureType::Angel], 4, 4)
    };
    CardDefinition {
        name: "Invasion of Dominaria",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![crate::card::BattleSubtype::Siege], ..Default::default() },
        defense: 5,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]))],
        back_face: Some(Box::new(faithkeeper)),
        ..Default::default()
    }
}

/// Invasion of New Capenna // Holy Frazzle-Cannon — {W}{B} Siege, defense 4.
/// When it enters, you may sacrifice an artifact or creature; when you do,
/// exile target artifact or creature an opponent controls. Back: an
/// Equipment — whenever equipped creature attacks, a +1/+1 counter on it and
/// on each other creature you control that shares a creature type with it.
/// Equip {1}.
pub fn invasion_of_new_capenna() -> CardDefinition {
    let cannon = CardDefinition {
        name: "Holy Frazzle-Cannon",
        card_types: vec![CardType::Artifact],
        color_indicator: vec![Color::White, Color::Black],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::SharesCreatureTypeWithSource.or(R::IsSource)),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of New Capenna",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![crate::card::BattleSubtype::Siege], ..Default::default() },
        defense: 4,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Sacrifice an artifact or creature to exile an opponent's?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Artifact.or(R::Creature) },
                Effect::If {
                    cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                    then: Box::new(Effect::ReflexiveTrigger {
                        body: Box::new(Effect::Move {
                            what: target_filtered(R::Artifact.or(R::Creature).and(R::ControlledByOpponent)),
                            to: ZoneDest::Exile,
                        }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        back_face: Some(Box::new(cannon)),
        ..Default::default()
    }
}

/// Curse of Oblivion — {3}{B} Aura Curse. Enchant player. At the beginning
/// of enchanted player's upkeep, that player exiles two cards from their
/// graveyard.
pub fn curse_of_oblivion() -> CardDefinition {
    CardDefinition {
        name: "Curse of Oblivion",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::IsTurnOf(PlayerRef::EnchantedPlayer)),
            effect: Effect::ExileFromGraveyard { who: PlayerRef::EnchantedPlayer, count: Value::Const(2), filter: R::Any },
        }],
        ..Default::default()
    }
}

// ── Batch seven ─────────────────────────────────────────────────────────────

/// Lose Calm — {3}{R} sorcery. Gain control of target creature until end of
/// turn, untap it; it gains haste and menace until end of turn.
pub fn lose_calm() -> CardDefinition {
    let grant = |keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword, duration: Duration::EndOfTurn };
    spell(
        "Lose Calm",
        cost(&[generic(3), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::EndOfTurn },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            grant(Keyword::Haste),
            grant(Keyword::Menace),
        ]),
    )
}

/// Dragon Whisperer — {R}{R} 2/2 Human Shaman. {R}: flying until end of
/// turn. {1}{R}: +1/+0 until end of turn. Formidable — {4}{R}{R}: a 4/4 red
/// flying Dragon token; only if your creatures' total power is 8 or greater.
pub fn dragon_whisperer() -> CardDefinition {
    let dragon = TokenDefinition {
        keywords: vec![Keyword::Flying],
        ..token("Dragon", 4, 4, vec![Color::Red], vec![CreatureType::Dragon])
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Flying, duration: Duration::EndOfTurn },
                ..Default::default()
            },
            ActivatedAbility { mana_cost: cost(&[generic(1), r()]), effect: pump_self(1, 0), ..Default::default() },
            ActivatedAbility {
                mana_cost: cost(&[generic(4), r(), r()]),
                condition: Some(Predicate::FormidableActive { who: PlayerRef::You }),
                effect: make(1, dragon),
                ..Default::default()
            },
        ],
        ..creature("Dragon Whisperer", cost(&[r(), r()]), vec![CreatureType::Human, CreatureType::Shaman], 2, 2)
    }
}

/// Lapis Lazuli Talisman — {2} artifact. Whenever a player casts a blue
/// spell, you may pay {3}; if you do, untap target permanent.
pub fn lapis_lazuli_talisman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellMatches(R::HasColor(Color::Blue))),
            effect: Effect::MayPay {
                description: "Pay {3} to untap target permanent?".into(),
                mana_cost: cost(&[generic(3)]),
                body: Box::new(Effect::Untap { what: target_filtered(R::Permanent), up_to: None }),
                else_: None,
            },
        }],
        ..spell("Lapis Lazuli Talisman", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Riddleform — {1}{U} enchantment. Whenever you cast a noncreature spell,
/// you may have it become a 3/3 flying Sphinx creature in addition to its
/// other types until end of turn. {2}{U}: scry 1.
pub fn riddleform() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Not(Box::new(R::Creature)))),
            effect: Effect::MayDo {
                description: "Have Riddleform become a 3/3 flying Sphinx?".into(),
                body: Box::new(Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![CreatureType::Sphinx],
                    keywords: vec![Keyword::Flying],
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..spell("Riddleform", cost(&[generic(1), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Pyrrhic Blast — {3}{R} instant. As an additional cost, sacrifice a
/// creature. Damage equal to its power to any target. Draw a card.
pub fn pyrrhic_blast() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature.and(R::ControlledByYou),
            count: 1,
        }],
        ..spell(
            "Pyrrhic Blast",
            cost(&[generic(3), r()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::SacrificedPower },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        )
    }
}

/// Kithkin Mourncaller — {2}{G} 2/2 Kithkin Scout. Whenever an attacking
/// Kithkin or Elf is put into your graveyard from the battlefield, you may
/// draw a card.
pub fn kithkin_mourncaller() -> CardDefinition {
    let kin = R::HasCreatureType(CreatureType::Kithkin).or(R::HasCreatureType(CreatureType::Elf));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: kin.and(R::IsAttacking).and(R::OwnedByYou),
            }),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..creature("Kithkin Mourncaller", cost(&[generic(2), g()]), vec![CreatureType::Kithkin, CreatureType::Scout], 2, 2)
    }
}

/// Obelisk of Bant — {3} artifact. {T}: {G}, {W} or {U}.
pub fn obelisk_of_bant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Green, Color::White, Color::Blue], Value::ONE),
            },
            ..Default::default()
        }],
        ..spell("Obelisk of Bant", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Protomatter Powder — {2}{U} artifact. {4}{W}, {T}, sacrifice it: return
/// target artifact card from your graveyard to the battlefield.
pub fn protomatter_powder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..spell("Protomatter Powder", cost(&[generic(2), u()]), CardType::Artifact, Effect::Noop)
    }
}

/// Hold for Questioning — {3}{U} Aura. Enchant creature or planeswalker.
/// When it enters, tap enchanted permanent and investigate. It doesn't untap
/// and its activated abilities can't be activated.
pub fn hold_for_questioning() -> CardDefinition {
    CardDefinition {
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.or(R::Planeswalker)) },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: Selector::attached_to(Selector::This) },
            crate::effect::shortcut::investigate(1),
        ]))],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted permanent doesn't untap during its controller's untap step.",
                effect: StaticEffect::PreventUntap { applies_to: Selector::attached_to(Selector::This) },
            },
            StaticAbility {
                description: "Enchanted permanent's activated abilities can't be activated.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::attached_to(Selector::This),
                    keyword: Keyword::CantActivateAbilities,
                },
            },
        ],
        ..aura("Hold for Questioning", cost(&[generic(3), u()]), None)
    }
}

/// Sleek Schooner — {3} 4/3 Vehicle. Crew 1.
pub fn sleek_schooner() -> CardDefinition {
    CardDefinition {
        name: "Sleek Schooner",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Crew(1)],
        ..Default::default()
    }
}

/// Molten Blast — {2}{R} instant. Choose one — 2 damage to target creature
/// or planeswalker; or destroy target artifact.
pub fn molten_blast() -> CardDefinition {
    spell(
        "Molten Blast",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::DealDamage { to: target_filtered(R::Creature.or(R::Planeswalker)), amount: Value::Const(2) },
            Effect::Destroy { what: target_filtered(R::Artifact) },
        ]),
    )
}

/// Winterflame — {1}{U}{R} instant. Choose one or both — tap target creature;
/// 2 damage to target creature.
pub fn winterflame() -> CardDefinition {
    spell(
        "Winterflame",
        cost(&[generic(1), u(), r()]),
        CardType::Instant,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(2) },
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}

/// Throne of Empires — {4} artifact. {1}, {T}: a 1/1 white Soldier token;
/// five instead if you control artifacts named Crown of Empires and Scepter
/// of Empires.
pub fn throne_of_empires() -> CardDefinition {
    let have = |name: &str| {
        Predicate::SelectorExists(Selector::EachPermanent(
            R::Artifact.and(R::ControlledByYou).and(R::HasName(name.into())),
        ))
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::IfPred {
                    pred: Box::new(Predicate::All(vec![have("Crown of Empires"), have("Scepter of Empires")])),
                    then: Box::new(Value::Const(5)),
                    else_: Box::new(Value::ONE),
                },
                definition: Arc::new(token("Soldier", 1, 1, vec![Color::White], vec![CreatureType::Soldier])),
            },
            ..Default::default()
        }],
        ..spell("Throne of Empires", cost(&[generic(4)]), CardType::Artifact, Effect::Noop)
    }
}

/// Guardian Shield-Bearer — {1}{G} 2/1 Human Soldier. Megamorph {3}{G}. When
/// it's turned face up, put a +1/+1 counter on another target creature you
/// control.
pub fn guardian_shield_bearer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Megamorph(cost(&[generic(3), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Guardian Shield-Bearer", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 1)
    }
}

/// Infiltrator il-Kor — {4}{U} 3/1 Kor Rogue. Shadow. Suspend 2—{1}{U}.
pub fn infiltrator_il_kor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow, Keyword::Suspend(2, cost(&[generic(1), u()]))],
        ..creature("Infiltrator il-Kor", cost(&[generic(4), u()]), vec![CreatureType::Kor, CreatureType::Rogue], 3, 1)
    }
}

/// Viscerid Deepwalker — {4}{U} 2/3 Homarid Warrior. {U}: +1/+0 until end of
/// turn. Suspend 4—{U}.
pub fn viscerid_deepwalker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(4, cost(&[u()]))],
        activated_abilities: vec![ActivatedAbility { mana_cost: cost(&[u()]), effect: pump_self(1, 0), ..Default::default() }],
        ..creature("Viscerid Deepwalker", cost(&[generic(4), u()]), vec![CreatureType::Homarid, CreatureType::Warrior], 2, 3)
    }
}

/// Petrified Plating — {2}{G} Aura. Enchanted creature gets +2/+2. Suspend
/// 2—{G}.
pub fn petrified_plating() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(2, cost(&[g()]))],
        ..aura("Petrified Plating", cost(&[generic(2), g()]), Some(EquipBonus { power: 2, toughness: 2, ..Default::default() }))
    }
}

/// Aeronaut's Wings — {2} Equipment. Equipped creature gets +1/+0 and has
/// flying. Equip {2}.
pub fn aeronauts_wings() -> CardDefinition {
    equipment(
        "Aeronaut's Wings",
        cost(&[generic(2)]),
        cost(&[generic(2)]),
        EquipBonus { power: 1, keywords: vec![Keyword::Flying], ..Default::default() },
    )
}

/// Heart-Piercer Bow — {2} Equipment. Whenever equipped creature attacks,
/// this Equipment deals 1 damage to target creature defending player
/// controls. Equip {1}.
pub fn heart_piercer_bow() -> CardDefinition {
    equipment(
        "Heart-Piercer Bow",
        cost(&[generic(2)]),
        cost(&[generic(1)]),
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                    amount: Value::ONE,
                },
            }],
            triggers_on_equipment: true,
            ..Default::default()
        },
    )
}

// ── Batch eight ─────────────────────────────────────────────────────────────

/// Angelic Armaments — {3} Equipment. Equipped creature gets +2/+2, has
/// flying, and is a white Angel in addition to its other colors and types.
/// Equip {4}.
pub fn angelic_armaments() -> CardDefinition {
    equipment(
        "Angelic Armaments",
        cost(&[generic(3)]),
        cost(&[generic(4)]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying],
            add_creature_types: vec![CreatureType::Angel],
            add_colors: vec![Color::White],
            ..Default::default()
        },
    )
}

/// Spell Blast — {X}{U} instant. Counter target spell with mana value X.
pub fn spell_blast() -> CardDefinition {
    spell(
        "Spell Blast",
        cost(&[x(), u()]),
        CardType::Instant,
        Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::ManaValueExactlyXFromCost)) },
    )
}

/// Spell Burst — {X}{U} instant. Buyback {3}. Counter target spell with mana
/// value X.
pub fn spell_burst() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Buyback(cost(&[generic(3)]))],
        ..spell(
            "Spell Burst",
            cost(&[x(), u()]),
            CardType::Instant,
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::ManaValueExactlyXFromCost)) },
        )
    }
}

/// Briarberry Cohort — {1}{U} 1/1 Faerie Soldier. Flying. +1/+1 as long as
/// you control another blue creature.
pub fn briarberry_cohort() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "As long as you control another blue creature, this gets +1/+1.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::HasColor(Color::Blue)).and(R::ControlledByYou).and(R::OtherThanSource),
                )),
                power: 1,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..creature("Briarberry Cohort", cost(&[generic(1), u()]), vec![CreatureType::Faerie, CreatureType::Soldier], 1, 1)
    }
}

/// Awakener Druid — {2}{G} 1/1 Human Druid. When it enters, target Forest
/// becomes a 4/5 green Treefolk creature for as long as this creature
/// remains on the battlefield. It's still a land.
pub fn awakener_druid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::BecomeCreature {
                what: target_filtered(R::HasLandType(LandType::Forest)),
                power: Value::Const(4),
                toughness: Value::Const(5),
                creature_types: vec![CreatureType::Treefolk],
                keywords: vec![],
                duration: Duration::WhileSourceOnBattlefield,
            },
            Effect::BecomeColor {
                what: Selector::Target(0),
                colors: vec![Color::Green],
                duration: Duration::WhileSourceOnBattlefield,
                additive: false,
            },
        ]))],
        ..creature("Awakener Druid", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Druid], 1, 1)
    }
}

/// Circle of the Moon Druid — {2}{G} 2/4 Human Elf Druid. Bear Form — during
/// your turn, it is a Bear with base power and toughness 4/2 (it loses all
/// other creature types).
pub fn circle_of_the_moon_druid() -> CardDefinition {
    let your_turn = || Predicate::IsTurnOf(PlayerRef::You);
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Bear Form — during your turn, this creature has base power and toughness 4/2.",
                effect: StaticEffect::SetBasePtIf { condition: your_turn(), power: 4, toughness: 2 },
            },
            StaticAbility {
                description: "Bear Form — during your turn, this creature loses all other creature types.",
                effect: StaticEffect::WhileCondition {
                    condition: your_turn(),
                    inner: Box::new(StaticEffect::MatchingLoseAllCreatureTypes { applies_to: Selector::This }),
                },
            },
            StaticAbility {
                description: "Bear Form — during your turn, this creature is a Bear.",
                effect: StaticEffect::WhileCondition {
                    condition: your_turn(),
                    inner: Box::new(StaticEffect::AddCreatureTypeToMatching {
                        applies_to: Selector::This,
                        creature_type: CreatureType::Bear,
                    }),
                },
            },
        ],
        ..creature(
            "Circle of the Moon Druid",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Druid],
            2,
            4,
        )
    }
}

/// Alert Heedbonder — {1}{G/W}{G/W} 2/4 Human Scout. Vigilance. At the
/// beginning of your end step, gain 1 life for each creature you control
/// with vigilance.
pub fn alert_heedbonder() -> CardDefinition {
    let gw = || hybrid(Color::Green, Color::White);
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::HasKeyword(Keyword::Vigilance)),
                )),
            },
        }],
        ..creature("Alert Heedbonder", cost(&[generic(1), gw(), gw()]), vec![CreatureType::Human, CreatureType::Scout], 2, 4)
    }
}

/// Dust Elemental — {2}{W}{W} 6/6 Elemental. Flash; flying; fear. When it
/// enters, return three creatures you control to their owner's hand.
pub fn dust_elemental() -> CardDefinition {
    let one = || Effect::ReturnOneYouControl { filter: R::Creature, keep_best: false };
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Fear],
        triggered_abilities: vec![etb(Effect::Seq(vec![one(), one(), one()]))],
        ..creature("Dust Elemental", cost(&[generic(2), w(), w()]), vec![CreatureType::Elemental], 6, 6)
    }
}

/// Vivien, Arkbow Ranger — {1}{G}{G}{G} planeswalker, loyalty 4. +1:
/// distribute two +1/+1 counters among up to two target creatures; they gain
/// trample until end of turn. −3: target creature you control deals damage
/// equal to its power to target creature or planeswalker. −5: you may reveal
/// a creature card you own from outside the game and put it into your hand.
pub fn vivien_arkbow_ranger() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype};
    let loyalty = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { loyalty_cost, effect, ..Default::default() };
    let trample = |slot| Effect::GrantKeyword { what: Selector::Target(slot), keyword: Keyword::Trample, duration: Duration::EndOfTurn };
    CardDefinition {
        name: "Vivien, Arkbow Ranger",
        cost: cost(&[generic(1), g(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Vivien], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            loyalty(
                1,
                Effect::Seq(vec![
                    Effect::DistributeCounters {
                        total: Value::Const(2),
                        counter: CounterType::PlusOnePlusOne,
                        filter: R::Creature,
                        max_targets: 2,
                    },
                    trample(0),
                    trample(1),
                ]),
            ),
            loyalty(
                -3,
                Effect::DealDamageEqualToPower {
                    source: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                    target: Selector::TargetFiltered { slot: 1, filter: R::Creature.or(R::Planeswalker) },
                },
            ),
            loyalty(
                -5,
                Effect::MayDo {
                    description: "Reveal a creature card you own from outside the game?".into(),
                    body: Box::new(Effect::WishToHand { filter: R::Creature }),
                },
            ),
        ],
        ..Default::default()
    }
}

// ── Batch nine (four-gap decks) ─────────────────────────────────────────────

/// Dawnstrike Paladin — {3}{W}{W} 2/4 Human Knight. Vigilance, lifelink.
pub fn dawnstrike_paladin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        ..creature("Dawnstrike Paladin", cost(&[generic(3), w(), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 4)
    }
}

/// Tocasia's Onulet — {5} 4/4 artifact creature Construct. When it leaves the
/// battlefield, you gain 2 life. Unearth {3}{W}.
pub fn tocasias_onulet() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        activated_abilities: vec![crate::effect::shortcut::unearth(cost(&[generic(3), w()]))],
        ..creature("Tocasia's Onulet", cost(&[generic(5)]), vec![CreatureType::Construct], 4, 4)
    }
}

/// Resupply — {5}{W} instant. Gain 6 life. Draw a card.
pub fn resupply() -> CardDefinition {
    spell(
        "Resupply",
        cost(&[generic(5), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(6) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Spectral Reserves — {3}{W} sorcery. Two 1/1 white flying Spirit tokens;
/// gain 2 life.
pub fn spectral_reserves() -> CardDefinition {
    let spirit = TokenDefinition {
        keywords: vec![Keyword::Flying],
        ..token("Spirit", 1, 1, vec![Color::White], vec![CreatureType::Spirit])
    };
    spell(
        "Spectral Reserves",
        cost(&[generic(3), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![make(2, spirit), Effect::GainLife { who: Selector::You, amount: Value::Const(2) }]),
    )
}

/// Luxa River Shrine — {3} artifact. {1}, {T}: gain 1 life and put a brick
/// counter on it. {T}: gain 2 life; only with three or more brick counters.
pub fn luxa_river_shrine() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Brick, amount: Value::ONE },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::ValueAtLeast(
                    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Brick },
                    Value::Const(3),
                )),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..spell("Luxa River Shrine", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Dragon's Presence — {2}{W} instant. 5 damage to target attacking or
/// blocking creature.
pub fn dragons_presence() -> CardDefinition {
    spell(
        "Dragon's Presence",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            amount: Value::Const(5),
        },
    )
}

/// Stirring Address — {1}{W} instant. Target creature you control gets +2/+2
/// until end of turn. Overload {5}{W} (CR 702.96): each creature you control.
pub fn stirring_address() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(5), w()]),
            effect_override: Some(Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
            ..Default::default()
        }),
        ..spell(
            "Stirring Address",
            cost(&[generic(1), w()]),
            CardType::Instant,
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Cleansing Ray — {1}{W} sorcery. Choose one — destroy target Vampire; or
/// destroy target enchantment.
pub fn cleansing_ray() -> CardDefinition {
    spell(
        "Cleansing Ray",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Vampire))) },
            Effect::Destroy { what: target_filtered(R::Enchantment) },
        ]),
    )
}

/// Scarab Feast — {B} instant. Exile up to three target cards from a single
/// graveyard. Cycling {B}.
pub fn scarab_feast() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[b()]))],
        ..spell(
            "Scarab Feast",
            cost(&[b()]),
            CardType::Instant,
            Effect::ApplyToTargets {
                max_targets: 3,
                min_targets: 0,
                filter: R::InGraveyard.and(R::SameGraveyardAsTargetSlot(0)),
                effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
            },
        )
    }
}

/// Dead Drop — {9}{B} sorcery. Delve. Target player sacrifices two creatures.
pub fn dead_drop() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Delve],
        ..spell(
            "Dead Drop",
            cost(&[generic(9), b()]),
            CardType::Sorcery,
            Effect::Sacrifice { who: target_filtered(R::Player), count: Value::Const(2), filter: R::Creature },
        )
    }
}

/// Nature's Spiral — {1}{G} sorcery. Return target permanent card from your
/// graveyard to your hand.
pub fn natures_spiral() -> CardDefinition {
    spell(
        "Nature's Spiral",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::Move { what: target_filtered(R::Permanent.from_your_graveyard()), to: ZoneDest::Hand(PlayerRef::You) },
    )
}

/// Scout the Borders — {2}{G} sorcery. Reveal the top five cards; you may put
/// a creature or land card among them into your hand, the rest into your
/// graveyard.
pub fn scout_the_borders() -> CardDefinition {
    spell(
        "Scout the Borders",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::MillThenToHand { amount: Value::Const(5), filter: R::Creature.or(R::Land), otherwise: None },
    )
}

/// Silent Dart — {1} artifact. {4}, {T}, sacrifice it: 3 damage to target
/// creature.
pub fn silent_dart() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(3) },
            ..Default::default()
        }],
        ..spell("Silent Dart", cost(&[generic(1)]), CardType::Artifact, Effect::Noop)
    }
}

/// Galvanic Juggernaut — {4} 5/5 artifact creature Juggernaut. Attacks each
/// combat if able. Doesn't untap during your untap step. Whenever another
/// creature dies, untap it.
pub fn galvanic_juggernaut() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::MustAttack],
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::OtherThanSource,
            }),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..creature("Galvanic Juggernaut", cost(&[generic(4)]), vec![CreatureType::Juggernaut], 5, 5)
    }
}

/// Haunted Dead — {3}{B} 2/2 Zombie. When it enters, a 1/1 white flying
/// Spirit token. {1}{B}, discard two cards: return it from your graveyard to
/// the battlefield tapped.
pub fn haunted_dead() -> CardDefinition {
    let spirit = TokenDefinition {
        keywords: vec![Keyword::Flying],
        ..token("Spirit", 1, 1, vec![Color::White], vec![CreatureType::Spirit])
    };
    CardDefinition {
        triggered_abilities: vec![etb(make(1, spirit))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            from_graveyard: true,
            discard_cost: Some((R::Any, 2)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature("Haunted Dead", cost(&[generic(3), b()]), vec![CreatureType::Zombie], 2, 2)
    }
}

/// Salvaged Manaworker — {2} 1/3 artifact creature Construct. {1}: add one
/// mana of any color; only once each turn.
pub fn salvaged_manaworker() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            once_per_turn: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..creature("Salvaged Manaworker", cost(&[generic(2)]), vec![CreatureType::Construct], 1, 3)
    }
}

/// Castle Raptors — {4}{W} 3/3 Bird Soldier. Flying. +0/+2 while untapped.
pub fn castle_raptors() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is untapped, it gets +0/+2.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::Not(Box::new(Predicate::EntityMatches { what: Selector::This, filter: R::Tapped })),
                power: 0,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..creature("Castle Raptors", cost(&[generic(4), w()]), vec![CreatureType::Bird, CreatureType::Soldier], 3, 3)
    }
}

/// Gearsmith Guardian — {5} 3/5 artifact creature Construct. +2/+0 as long as
/// you control a blue creature.
pub fn gearsmith_guardian() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +2/+0 as long as you control a blue creature.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::HasColor(Color::Blue)).and(R::ControlledByYou),
                )),
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..creature("Gearsmith Guardian", cost(&[generic(5)]), vec![CreatureType::Construct], 3, 5)
    }
}

/// Scrapdiver Serpent — {5}{U}{U} 5/5 Serpent. Can't be blocked as long as
/// defending player controls an artifact (CR 702.14c's filtered walk).
pub fn scrapdiver_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::LandwalkFiltered(Box::new(R::Artifact))],
        ..creature("Scrapdiver Serpent", cost(&[generic(5), u(), u()]), vec![CreatureType::Serpent], 5, 5)
    }
}

/// Skilled Animator — {2}{U} 1/3 Human Artificer. When it enters, target
/// artifact you control becomes an artifact creature with base power and
/// toughness 5/5 for as long as it remains on the battlefield.
pub fn skilled_animator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::BecomeCreature {
            what: target_filtered(R::Artifact.and(R::ControlledByYou)),
            power: Value::Const(5),
            toughness: Value::Const(5),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::WhileSourceOnBattlefield,
        })],
        ..creature("Skilled Animator", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Artificer], 1, 3)
    }
}

/// Equestrian Skill — {3}{G} Aura. +3/+3; a Human has trample too.
pub fn equestrian_skill() -> CardDefinition {
    aura(
        "Equestrian Skill",
        cost(&[generic(3), g()]),
        Some(EquipBonus {
            power: 3,
            toughness: 3,
            conditional: vec![crate::card::ConditionalEquipBonus {
                host_filter: R::HasCreatureType(CreatureType::Human),
                keywords: vec![Keyword::Trample],
                ..Default::default()
            }],
            ..Default::default()
        }),
    )
}

/// Lightning Talons — {2}{R} Aura. +3/+0 and first strike.
pub fn lightning_talons() -> CardDefinition {
    aura(
        "Lightning Talons",
        cost(&[generic(2), r()]),
        Some(EquipBonus { power: 3, keywords: vec![Keyword::FirstStrike], ..Default::default() }),
    )
}

/// See Red — {1}{R} Aura. +2/+1 and first strike. At the beginning of your
/// end step, if you didn't attack with a creature this turn, sacrifice it.
pub fn see_red() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                Predicate::Not(Box::new(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You })),
            ),
            effect: Effect::SacrificeSource,
        }],
        ..aura(
            "See Red",
            cost(&[generic(1), r()]),
            Some(EquipBonus { power: 2, toughness: 1, keywords: vec![Keyword::FirstStrike], ..Default::default() }),
        )
    }
}

/// Spiteful Motives — {3}{R} Aura with flash. +3/+0 and first strike.
pub fn spiteful_motives() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..aura(
            "Spiteful Motives",
            cost(&[generic(3), r()]),
            Some(EquipBonus { power: 3, keywords: vec![Keyword::FirstStrike], ..Default::default() }),
        )
    }
}

// ── Batch ten (four-gap decks) ──────────────────────────────────────────────

/// Midnight Scavengers — {4}{B} 3/3 Human Rogue. When it enters, you may
/// return target creature card with mana value 3 or less from your graveyard
/// to your hand. (Its meld with Graf Rats is not modelled: no seat runs both.)
pub fn midnight_scavengers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return a creature card with mana value 3 or less?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::ManaValueAtMost(3)).and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature("Midnight Scavengers", cost(&[generic(4), b()]), vec![CreatureType::Human, CreatureType::Rogue], 3, 3)
    }
}

/// Ancestral Vengeance — {B}{B} Aura. When it enters, put a +1/+1 counter on
/// target creature you control. Enchanted creature gets -1/-1.
pub fn ancestral_vengeance() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..aura("Ancestral Vengeance", cost(&[b(), b()]), Some(EquipBonus { power: -1, toughness: -1, ..Default::default() }))
    }
}

/// Consecrated by Blood — {2}{B}{B} Aura. +2/+2, flying, and "Sacrifice two
/// other creatures: regenerate this creature."
pub fn consecrated_by_blood() -> CardDefinition {
    aura(
        "Consecrated by Blood",
        cost(&[generic(2), b(), b()]),
        Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Flying],
            activated_abilities: vec![ActivatedAbility {
                sac_other_filter: Some((R::Creature.and(R::ControlledByYou).and(R::OtherThanSource), 2)),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            }],
            ..Default::default()
        }),
    )
}

/// Agonizing Syphon — {3}{B} sorcery. 3 damage to any target; gain 3 life.
pub fn agonizing_syphon() -> CardDefinition {
    spell(
        "Agonizing Syphon",
        cost(&[generic(3), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(3) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]),
    )
}

/// Hemlock Vial — {1}{B} artifact. When it enters, draw a card and lose 1
/// life. {B}, {T}, sacrifice it: each equipped creature and Equipment you
/// control gains deathtouch until end of turn.
pub fn hemlock_vial() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    R::ControlledByYou.and(R::HasArtifactSubtype(ArtifactSubtype::Equipment).or(R::Creature.and(R::IsEquipped))),
                ),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..spell("Hemlock Vial", cost(&[generic(1), b()]), CardType::Artifact, Effect::Noop)
    }
}

/// Myrkul's Invoker — {2}{B} 2/3 Cat Rogue. Psychic Blades — {8}: creatures
/// you control get +2/+0 and gain menace until end of turn.
pub fn myrkuls_invoker() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: team(), power: Value::Const(2), toughness: Value::Const(0), duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: team(), keyword: Keyword::Menace, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature("Myrkul's Invoker", cost(&[generic(2), b()]), vec![CreatureType::Cat, CreatureType::Rogue], 2, 3)
    }
}

/// Final Flourish — {1}{B} instant. Kicker — sacrifice an artifact or
/// creature. Target creature gets -2/-2 until end of turn, -6/-6 if kicked.
pub fn final_flourish() -> CardDefinition {
    let shrink = |n: i32| Effect::PumpPT {
        what: target_filtered(R::Creature),
        power: Value::Const(-n),
        toughness: Value::Const(-n),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        kicker_action_cost: Some(crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Artifact.or(R::Creature).and(R::ControlledByYou),
            count: 1,
        }),
        ..spell(
            "Final Flourish",
            cost(&[generic(1), b()]),
            CardType::Instant,
            Effect::If { cond: Predicate::SpellWasKicked, then: Box::new(shrink(6)), else_: Box::new(shrink(2)) },
        )
    }
}

/// Infernal Reckoning — {B} instant. Exile target colorless creature; gain
/// life equal to its power.
pub fn infernal_reckoning() -> CardDefinition {
    spell(
        "Infernal Reckoning",
        cost(&[b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::PowerOf(Box::new(Selector::Target(0))) },
            Effect::Move { what: target_filtered(R::Creature.and(R::Colorless)), to: ZoneDest::Exile },
        ]),
    )
}

/// Ballista Charger — {5} 6/6 Vehicle. Whenever it attacks, it deals 1 damage
/// to any target. Crew 3.
pub fn ballista_charger() -> CardDefinition {
    CardDefinition {
        name: "Ballista Charger",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Crew(3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Phantasmal Dragon — {2}{U}{U} 5/5 Dragon Illusion. Flying. When it becomes
/// the target of a spell or ability, sacrifice it.
pub fn phantasmal_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::SacrificeSource,
        }],
        ..creature("Phantasmal Dragon", cost(&[generic(2), u(), u()]), vec![CreatureType::Dragon, CreatureType::Illusion], 5, 5)
    }
}

/// Pseudodragon Familiar — {2}{U} 2/1 Dragon. Flying. {2}{U}: target creature
/// gains flying until end of turn.
pub fn pseudodragon_familiar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..creature("Pseudodragon Familiar", cost(&[generic(2), u()]), vec![CreatureType::Dragon], 2, 1)
    }
}

/// Evershrike's Gift — {W} Aura. +1/+0 and flying. {1}{W}, blight 2: return
/// it from your graveyard to your hand. Sorcery speed.
pub fn evershrikes_gift() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            from_graveyard: true,
            blight_cost: 2,
            sorcery_speed: true,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..aura("Evershrike's Gift", cost(&[w()]), Some(EquipBonus { power: 1, keywords: vec![Keyword::Flying], ..Default::default() }))
    }
}

/// Dune Beetle — {1}{B} 1/4 Insect.
pub fn dune_beetle() -> CardDefinition {
    creature("Dune Beetle", cost(&[generic(1), b()]), vec![CreatureType::Insect], 1, 4)
}

/// Hinterland Logger // Timber Shredder — {1}{G} 2/1 Human Werewolf; back a
/// 4/2 trampling Werewolf (CR 701.28 werewolf transforms).
pub fn hinterland_logger() -> CardDefinition {
    use crate::effect::shortcut::{werewolf_day_transform, werewolf_night_transform};
    let shredder = CardDefinition {
        name: "Timber Shredder",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Werewolf], ..Default::default() },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![werewolf_night_transform()],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![werewolf_day_transform()],
        back_face: Some(Box::new(shredder)),
        ..creature("Hinterland Logger", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Werewolf], 2, 1)
    }
}

/// Intrepid Provisioner — {3}{G} 3/3 Human Scout. Trample. When it enters,
/// another target Human you control gets +2/+2 until end of turn.
pub fn intrepid_provisioner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                R::Creature.and(R::HasCreatureType(CreatureType::Human)).and(R::ControlledByYou).and(R::OtherThanSource),
            ),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Intrepid Provisioner", cost(&[generic(3), g()]), vec![CreatureType::Human, CreatureType::Scout], 3, 3)
    }
}

/// Grim Contest — {1}{B}{G} instant. Target creature you control and target
/// creature an opponent controls each deal damage equal to its toughness to
/// the other.
pub fn grim_contest() -> CardDefinition {
    spell(
        "Grim Contest",
        cost(&[generic(1), b(), g()]),
        CardType::Instant,
        Effect::DealDamageToEachOther {
            first: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
            second: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
            first_amount: Value::ToughnessOf(Box::new(Selector::Target(0))),
            second_amount: Value::ToughnessOf(Box::new(Selector::Target(1))),
        },
    )
}

// ── Batch eleven (two-gap decks) ────────────────────────────────────────────

/// Embodiment of Agonies — {1}{B}{B} 0/0 Demon. Flying, deathtouch. Enters
/// with a +1/+1 counter for each different mana cost among nonland cards in
/// your graveyard.
pub fn embodiment_of_agonies() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::DistinctManaCostsInGraveyard(PlayerRef::You))),
        ..creature("Embodiment of Agonies", cost(&[generic(1), b(), b()]), vec![CreatureType::Demon], 0, 0)
    }
}

/// Rakshasa Deathdealer — {B}{G} 2/2 Demon. {B}{G}: +2/+2 until end of turn.
/// {B}{G}: regenerate it.
pub fn rakshasa_deathdealer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b(), g()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b(), g()]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
        ],
        ..creature("Rakshasa Deathdealer", cost(&[b(), g()]), vec![CreatureType::Demon], 2, 2)
    }
}

/// Faithful Pikemaster — {3}{W} 3/4 Rhino Monk Soldier. When it enters, scry
/// 2. First strike during your turn.
pub fn faithful_pikemaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        static_abilities: vec![StaticAbility {
            description: "As long as it's your turn, this creature has first strike.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::FirstStrike,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        ..creature(
            "Faithful Pikemaster",
            cost(&[generic(3), w()]),
            vec![CreatureType::Rhino, CreatureType::Monk, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Judge Unworthy — {1}{W} instant. Choose target attacking or blocking
/// creature. Scry 3, then reveal the top card of your library; deal damage
/// equal to its mana value to that creature.
pub fn judge_unworthy() -> CardDefinition {
    spell(
        "Judge Unworthy",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
            Effect::RevealTopOfLibrary { who: PlayerRef::You },
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::ManaValueOf(Box::new(Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE })),
            },
        ]),
    )
}

/// Myth Realized — {W} enchantment. Whenever you cast a noncreature spell, put
/// a lore counter on it. {2}{W}: put a lore counter on it. {W}: until end of
/// turn it becomes a Monk Avatar creature with power and toughness each equal
/// to its lore counters.
pub fn myth_realized() -> CardDefinition {
    let lore = || Effect::AddCounter { what: Selector::This, kind: CounterType::Lore, amount: Value::ONE };
    let count = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Lore };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Not(Box::new(R::Creature)),
            }),
            effect: lore(),
        }],
        activated_abilities: vec![
            ActivatedAbility { mana_cost: cost(&[generic(2), w()]), effect: lore(), ..Default::default() },
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                effect: Effect::Seq(vec![
                    Effect::AnimateAsCreature { what: Selector::This, duration: Duration::EndOfTurn },
                    Effect::AddCreatureTypes {
                        what: Selector::This,
                        creature_types: vec![CreatureType::Monk, CreatureType::Avatar],
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        // The granted "power and toughness are each equal to …" only matters
        // while it is a creature, so it rides as a standing CDA.
        static_abilities: vec![StaticAbility {
            description: "Its power and toughness are each equal to the number of lore counters on it.",
            effect: StaticEffect::SelfBasePtFromValue { power: count(), toughness: count() },
        }],
        ..spell("Myth Realized", cost(&[w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Lantern Flare — {1}{W} instant. Cleave {X}{R}{W}. X damage to target
/// creature or planeswalker and you gain X life [where X is the number of
/// creatures you control].
pub fn lantern_flare() -> CardDefinition {
    let flare = |x: Value| {
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature.or(R::Planeswalker)), amount: x.clone() },
            Effect::GainLife { who: Selector::You, amount: x },
        ])
    };
    let creatures = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
        filter: R::Any,
    };
    CardDefinition {
        // CR 702.148 — the cleave cast drops the bracketed X definition.
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[x(), r(), w()]),
            effect_override: Some(flare(Value::XFromCost)),
            ..Default::default()
        }),
        ..spell("Lantern Flare", cost(&[generic(1), w()]), CardType::Instant, flare(creatures))
    }
}

/// Giant Dustwasp — {3}{G}{G} 3/3 Insect. Flying. Suspend 4—{1}{G}.
pub fn giant_dustwasp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Suspend(4, cost(&[generic(1), g()]))],
        ..creature("Giant Dustwasp", cost(&[generic(3), g(), g()]), vec![CreatureType::Insect], 3, 3)
    }
}

/// Nantuko Shaman — {2}{G} 3/2 Insect Shaman. When it enters, if you control
/// no tapped lands, draw a card. Suspend 1—{2}{G}{G}.
pub fn nantuko_shaman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(1, cost(&[generic(2), g(), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(Predicate::Not(
                Box::new(Predicate::SelectorExists(Selector::EachPermanent(
                    R::Land.and(R::Tapped).and(R::ControlledByYou),
                ))),
            )),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Nantuko Shaman", cost(&[generic(2), g()]), vec![CreatureType::Insect, CreatureType::Shaman], 3, 2)
    }
}

/// Altar of the Lost — {3} artifact. Enters tapped. {T}: add two mana in any
/// combination of colors; spend it only on spells with flashback cast from a
/// graveyard (approximated as any spell cast from your graveyard).
pub fn altar_of_the_lost() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyColors(Value::Const(2))),
                    crate::mana::SpendRestriction::SpellFromGraveyard,
                ),
            },
            ..Default::default()
        }],
        ..spell("Altar of the Lost", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Sentinel of the Pearl Trident — {4}{U} 3/3 Merfolk Soldier. Flash. When it
/// enters, you may exile target historic permanent you control; return it at
/// the beginning of the next end step.
pub fn sentinel_of_the_pearl_trident() -> CardDefinition {
    let historic = R::Artifact
        .or(R::HasSupertype(crate::card::Supertype::Legendary))
        .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga));
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile a historic permanent until the next end step?".into(),
            body: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                what: target_filtered(historic.and(R::ControlledByYou)),
                tapped: false,
            }),
        })],
        ..creature(
            "Sentinel of the Pearl Trident",
            cost(&[generic(4), u()]),
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Barricade Breaker — {7} 7/5 artifact Juggernaut. Improvise. Attacks each
/// combat if able.
pub fn barricade_breaker() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Improvise, Keyword::MustAttack],
        ..creature("Barricade Breaker", cost(&[generic(7)]), vec![CreatureType::Juggernaut], 7, 5)
    }
}

/// Surge Engine — {2} 3/2 artifact Construct. Defender. {U}: it loses
/// defender and gains "can't be blocked". {2}{U}: it becomes blue with base
/// power and toughness 5/4 (only without defender). {4}{U}{U}: draw three
/// cards (only if it's blue, and only once).
pub fn surge_engine() -> CardDefinition {
    let this_is = |f: R| Some(Predicate::EntityMatches { what: Selector::This, filter: f });
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::Seq(vec![
                    Effect::LoseKeyword { what: Selector::This, keyword: Keyword::Defender, duration: Duration::Permanent },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::Permanent },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                condition: this_is(R::Not(Box::new(R::HasKeyword(Keyword::Defender)))),
                effect: Effect::Seq(vec![
                    Effect::BecomeColor {
                        what: Selector::This,
                        colors: vec![Color::Blue],
                        duration: Duration::Permanent,
                        additive: false,
                    },
                    Effect::SetBasePT {
                        what: Selector::This,
                        power: Value::Const(5),
                        toughness: Value::Const(4),
                        duration: Duration::Permanent,
                    },
                ]),
                ..Default::default()
            },
            // "Activate only once" rides the exhaust flag (CR 702.177a).
            ActivatedAbility {
                mana_cost: cost(&[generic(4), u(), u()]),
                condition: this_is(R::HasColor(Color::Blue)),
                exhaust: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                ..Default::default()
            },
        ],
        ..creature("Surge Engine", cost(&[generic(2)]), vec![CreatureType::Construct], 3, 2)
    }
}

/// Blaring Recruiter — {3}{W} 2/2 Elf Warrior. Partner with Blaring Captain.
/// {2}{W}: create a 1/1 white Warrior token.
pub fn blaring_recruiter() -> CardDefinition {
    let warrior = TokenDefinition {
        name: "Warrior".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Warrior], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Blaring Captain".into())],
        triggered_abilities: vec![crate::effect::shortcut::partner_with_search("Blaring Captain")],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(warrior) },
            ..Default::default()
        }],
        ..creature("Blaring Recruiter", cost(&[generic(3), w()]), vec![CreatureType::Elf, CreatureType::Warrior], 2, 2)
    }
}

/// Inner-Flame Igniter — {2}{R} 2/2 Elemental Warrior. {2}{R}: creatures you
/// control get +1/+0 until end of turn; the third resolution this turn also
/// gives them first strike.
pub fn inner_flame_igniter() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: team(),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::NthResolutionThisTurn {
                    branches: vec![
                        Effect::Noop,
                        Effect::Noop,
                        Effect::GrantKeyword { what: team(), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
                    ],
                },
            ]),
            ..Default::default()
        }],
        ..creature("Inner-Flame Igniter", cost(&[generic(2), r()]), vec![CreatureType::Elemental, CreatureType::Warrior], 2, 2)
    }
}

/// Fatehold Chronologist // Peer Review — {1}{W/U} 1/2 flying Bird Wizard
/// that enters prepared. Peer Review ({2}{W/U} sorcery): a 2/2 colorless
/// Wizard Soldier token named Cadet; surveil 1.
pub fn fatehold_chronologist() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    let cadet = TokenDefinition {
        name: "Cadet".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        prepare_spell: Some(Arc::new(CardDefinition {
            name: "Peer Review",
            cost: cost(&[generic(2), wu()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(cadet) },
                Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            ]),
            ..Default::default()
        })),
        ..creature("Fatehold Chronologist", cost(&[generic(1), wu()]), vec![CreatureType::Bird, CreatureType::Wizard], 1, 2)
    }
}

/// Mystery Key — {1}{U} Equipment. When equipped creature deals combat damage
/// to a player, sacrifice this Equipment; if you do, draw three cards.
/// Equip {1}.
pub fn mystery_key() -> CardDefinition {
    equipment(
        "Mystery Key",
        cost(&[generic(1), u()]),
        cost(&[generic(1)]),
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::SourceOnBattlefield,
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSource,
                        Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            }],
            triggers_on_equipment: true,
            ..Default::default()
        },
    )
}

// ── Batch twelve (two-gap decks) ────────────────────────────────────────────

/// Ray of Frost — {1}{U} Aura. Flash. When it enters, if enchanted creature
/// is red, tap it. As long as enchanted creature is red, it loses all
/// abilities. Enchanted creature doesn't untap during its controller's untap
/// step.
pub fn ray_of_frost() -> CardDefinition {
    let host = || Selector::attached_to(Selector::This);
    let host_is_red = || Predicate::EntityMatches { what: host(), filter: R::HasColor(Color::Red) };
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::If {
            cond: host_is_red(),
            then: Box::new(Effect::Tap { what: host() }),
            else_: Box::new(Effect::Noop),
        })],
        static_abilities: vec![
            StaticAbility {
                description: "As long as enchanted creature is red, it loses all abilities.",
                effect: StaticEffect::WhileCondition {
                    condition: host_is_red(),
                    inner: Box::new(StaticEffect::MatchingLoseAllAbilities { applies_to: host() }),
                },
            },
            StaticAbility {
                description: "Enchanted creature doesn't untap during its controller's untap step.",
                effect: StaticEffect::PreventUntap { applies_to: host() },
            },
        ],
        ..aura("Ray of Frost", cost(&[generic(1), u()]), None)
    }
}

/// Tamiyo's Compleation — {3}{U} Aura. Flash. Enchant artifact, creature, or
/// planeswalker. When it enters, tap enchanted permanent; if it's an
/// Equipment, unattach it. Enchanted permanent loses all abilities and doesn't
/// untap during its controller's untap step.
pub fn tamiyos_compleation() -> CardDefinition {
    let host = || Selector::attached_to(Selector::This);
    CardDefinition {
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Artifact.or(R::Creature).or(R::Planeswalker)),
        },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: host() },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: host(),
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                },
                then: Box::new(Effect::Unattach { what: host() }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted permanent loses all abilities.",
                effect: StaticEffect::MatchingLoseAllAbilities { applies_to: host() },
            },
            StaticAbility {
                description: "Enchanted permanent doesn't untap during its controller's untap step.",
                effect: StaticEffect::PreventUntap { applies_to: host() },
            },
        ],
        ..aura("Tamiyo's Compleation", cost(&[generic(3), u()]), None)
    }
}

/// Ludevic's Test Subject // Ludevic's Abomination — {1}{U} 0/3 Lizard Egg.
/// Defender. {1}{U}: put a hatchling counter on it; then at five or more,
/// remove them and transform it. Back: a 13/13 trampling Lizard Horror.
pub fn ludevics_test_subject() -> CardDefinition {
    let hatchlings = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Hatchling };
    let abomination = CardDefinition {
        name: "Ludevic's Abomination",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Lizard, CreatureType::Horror], ..Default::default() },
        power: 13,
        toughness: 13,
        keywords: vec![Keyword::Trample],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Hatchling, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtLeast(hatchlings(), Value::Const(5)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::RemoveCounter { what: Selector::This, kind: CounterType::Hatchling, amount: hatchlings() },
                        Effect::Transform { what: Selector::This },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        back_face: Some(Box::new(abomination)),
        ..creature("Ludevic's Test Subject", cost(&[generic(1), u()]), vec![CreatureType::Lizard, CreatureType::Egg], 0, 3)
    }
}

/// Skaab Goliath — {5}{U} 6/9 Zombie Giant. As an additional cost, exile two
/// creature cards from your graveyard. Trample.
pub fn skaab_goliath() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::ExileFromGraveyard { filter: R::Creature, count: 2 }],
        ..creature("Skaab Goliath", cost(&[generic(5), u()]), vec![CreatureType::Zombie, CreatureType::Giant], 6, 9)
    }
}

/// Foreboding Statue // Forsaken Thresher — {3} 1/2 artifact Construct. {T}:
/// add one mana of any color and put an omen counter on it. At the beginning
/// of your end step, at three or more omen counters, untap it and transform
/// it. Back: a 5/5 that adds one mana of any color at the beginning of your
/// first main phase.
pub fn foreboding_statue() -> CardDefinition {
    let any_one = || Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::ONE) };
    let thresher = CardDefinition {
        name: "Forsaken Thresher",
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: any_one(),
        }],
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                any_one(),
                Effect::AddCounter { what: Selector::This, kind: CounterType::Omen, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                Predicate::ValueAtLeast(
                    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Omen },
                    Value::Const(3),
                ),
            ),
            effect: Effect::Seq(vec![Effect::Untap { what: Selector::This, up_to: None }, Effect::Transform { what: Selector::This }]),
        }],
        back_face: Some(Box::new(thresher)),
        ..creature("Foreboding Statue", cost(&[generic(3)]), vec![CreatureType::Construct], 1, 2)
    }
}

/// Profound Journey — {5}{W}{W} sorcery. Return target permanent card from
/// your graveyard to the battlefield. Rebound.
pub fn profound_journey() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Rebound],
        ..spell(
            "Profound Journey",
            cost(&[generic(5), w(), w()]),
            CardType::Sorcery,
            Effect::Move {
                what: target_filtered(R::PermanentCard.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        )
    }
}

/// Angrath, Minotaur Pirate — {4}{B}{R} planeswalker, loyalty 5. +2: 1 damage
/// to target opponent or planeswalker and each creature that player or that
/// planeswalker's controller controls. −3: return target Pirate card from your
/// graveyard to the battlefield. −11: destroy all creatures target opponent
/// controls; Angrath deals that player damage equal to their total power.
pub fn angrath_minotaur_pirate() -> CardDefinition {
    use crate::card::{LoyaltyAbility, Supertype};
    let loyalty = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { loyalty_cost, effect, ..Default::default() };
    let theirs = || Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature };
    CardDefinition {
        name: "Angrath, Minotaur Pirate",
        cost: cost(&[generic(4), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Angrath], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            loyalty(
                2,
                Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::OpponentPlayer.or(R::Planeswalker)), amount: Value::ONE },
                    Effect::DealDamage { to: theirs(), amount: Value::ONE },
                ]),
            ),
            loyalty(
                -3,
                Effect::Move {
                    what: target_filtered(R::HasCreatureType(CreatureType::Pirate).and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ),
            // The total power is read before the creatures are destroyed.
            loyalty(
                -11,
                Effect::WithX {
                    x: Value::PowerOf(Box::new(theirs())),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Destroy { what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature } },
                        Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::XFromCost },
                    ])),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Angrath's Fury — {3}{B}{R} sorcery. Destroy target creature. 3 damage to
/// target player or planeswalker. You may search your library and/or
/// graveyard for Angrath, Minotaur Pirate and put it into your hand (the
/// graveyard copy is taken first).
pub fn angraths_fury() -> CardDefinition {
    let angrath = || R::HasName("Angrath, Minotaur Pirate".into());
    let in_graveyard = || Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: angrath() };
    spell(
        "Angrath's Fury",
        cost(&[generic(3), b(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::Player.or(R::Planeswalker) },
                amount: Value::Const(3),
            },
            Effect::MayDo {
                description: "Search for Angrath, Minotaur Pirate?".into(),
                body: Box::new(Effect::If {
                    cond: Predicate::SelectorExists(in_graveyard()),
                    then: Box::new(Effect::Move { what: Selector::one_of(in_graveyard()), to: ZoneDest::Hand(PlayerRef::You) }),
                    else_: Box::new(Effect::Search { who: PlayerRef::You, filter: angrath(), to: ZoneDest::Hand(PlayerRef::You) }),
                }),
            },
        ]),
    )
}

/// Sphinx-Bone Wand — {7} artifact. Whenever you cast an instant or sorcery
/// spell, you may put a charge counter on it; if you do, it deals damage
/// equal to its charge counters to any target.
pub fn sphinx_bone_wand() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            }),
            effect: Effect::MayDo {
                description: "Put a charge counter on Sphinx-Bone Wand?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
                    Effect::DealDamage {
                        to: crate::effect::shortcut::target_any(),
                        amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
                    },
                ])),
            },
        }],
        ..spell("Sphinx-Bone Wand", cost(&[generic(7)]), CardType::Artifact, Effect::Noop)
    }
}

/// Relic Runner — {1}{U} 2/1 Human Rogue. Can't be blocked if you've cast a
/// historic spell this turn.
pub fn relic_runner() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature can't be blocked if you've cast a historic spell this turn.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Unblockable,
                condition: Predicate::ValueAtLeast(Value::HistoricSpellsCastThisTurn(PlayerRef::You), Value::ONE),
            },
        }],
        ..creature("Relic Runner", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 1)
    }
}

/// Mishra's Onslaught — {3}{R} instant. Choose one — two 1/1 colorless
/// Soldier artifact creature tokens; or creatures you control get +2/+0 until
/// end of turn.
pub fn mishras_onslaught() -> CardDefinition {
    let soldier = TokenDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..token("Soldier", 1, 1, vec![], vec![CreatureType::Soldier])
    };
    spell(
        "Mishra's Onslaught",
        cost(&[generic(3), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(soldier) },
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Rending Volley — {R} instant. Can't be countered. 4 damage to target white
/// or blue creature.
pub fn rending_volley() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        ..spell(
            "Rending Volley",
            cost(&[r()]),
            CardType::Instant,
            Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::HasColor(Color::White).or(R::HasColor(Color::Blue)))),
                amount: Value::Const(4),
            },
        )
    }
}

/// Highspire Mantis — {2}{R}{W} 3/3 Insect. Flying, trample.
pub fn highspire_mantis() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        ..creature("Highspire Mantis", cost(&[generic(2), r(), w()]), vec![CreatureType::Insect], 3, 3)
    }
}

/// Zodiac Horse — {3}{G} 3/3 Horse. Islandwalk.
pub fn zodiac_horse() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        ..creature("Zodiac Horse", cost(&[generic(3), g()]), vec![CreatureType::Horse], 3, 3)
    }
}

/// Watchwing Scarecrow — {4} 2/4 artifact Scarecrow. Vigilance while you
/// control a white creature; flying while you control a blue creature.
pub fn watchwing_scarecrow() -> CardDefinition {
    let while_you_control = |color: Color| {
        Predicate::SelectorExists(Selector::EachPermanent(R::Creature.and(R::HasColor(color)).and(R::ControlledByYou)))
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![
            StaticAbility {
                description: "This creature has vigilance as long as you control a white creature.",
                effect: StaticEffect::SelfHasKeywordWhilePredicate {
                    keyword: Keyword::Vigilance,
                    condition: while_you_control(Color::White),
                },
            },
            StaticAbility {
                description: "This creature has flying as long as you control a blue creature.",
                effect: StaticEffect::SelfHasKeywordWhilePredicate {
                    keyword: Keyword::Flying,
                    condition: while_you_control(Color::Blue),
                },
            },
        ],
        ..creature("Watchwing Scarecrow", cost(&[generic(4)]), vec![CreatureType::Scarecrow], 2, 4)
    }
}

/// Counterintelligence — {2}{U}{U} sorcery. Return one or two target
/// creatures to their owners' hands.
pub fn counterintelligence() -> CardDefinition {
    spell(
        "Counterintelligence",
        cost(&[generic(2), u(), u()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 1,
            filter: R::Creature,
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
        },
    )
}

// ── Batch thirteen (three-gap decks) ────────────────────────────────────────

/// Armor Thrull — {2}{B} 1/3 Thrull. {T}, sacrifice it: put a +1/+2 counter
/// on target creature.
pub fn armor_thrull() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusTwo,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Armor Thrull", cost(&[generic(2), b()]), vec![CreatureType::Thrull], 1, 3)
    }
}

/// Mindstab Thrull — {1}{B}{B} 2/2 Thrull. Whenever it attacks and isn't
/// blocked, you may sacrifice it; if you do, defending player discards three
/// cards.
pub fn mindstab_thrull() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_unblocked(Effect::MayDo {
            description: "Sacrifice Mindstab Thrull to make the defending player discard three?".into(),
            body: Box::new(Effect::If {
                cond: Predicate::SourceOnBattlefield,
                // The discard goes first: `DefendingPlayer` is read off the
                // attack, which ends once the Thrull is sacrificed.
                then: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::DefendingPlayer),
                        amount: Value::Const(3),
                        random: false,
                    },
                    Effect::SacrificeSource,
                ])),
                else_: Box::new(Effect::Noop),
            }),
        })],
        ..creature("Mindstab Thrull", cost(&[generic(1), b(), b()]), vec![CreatureType::Thrull], 2, 2)
    }
}

/// Thrull Champion — {4}{B} 2/2 Thrull. Thrull creatures get +1/+1. {T}: gain
/// control of target Thrull for as long as you control this creature.
pub fn thrull_champion() -> CardDefinition {
    let thrull = || R::Creature.and(R::HasCreatureType(CreatureType::Thrull));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Thrull creatures get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: Selector::EachPermanent(thrull()), power: 1, toughness: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainControlWhileYouControlSource { what: target_filtered(thrull()) },
            ..Default::default()
        }],
        ..creature("Thrull Champion", cost(&[generic(4), b()]), vec![CreatureType::Thrull], 2, 2)
    }
}

/// Contagion — {3}{B}{B} instant. You may pay 1 life and exile a black card
/// from your hand rather than pay its mana cost. Distribute two -2/-1
/// counters among one or two target creatures.
pub fn contagion() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: ManaCost::default(),
            life_cost: 1,
            exile_filter: Some(R::HasColor(Color::Black)),
            ..Default::default()
        }),
        ..spell(
            "Contagion",
            cost(&[generic(3), b(), b()]),
            CardType::Instant,
            Effect::DistributeCounters {
                total: Value::Const(2),
                counter: CounterType::MinusTwoMinusOne,
                filter: R::Creature,
                max_targets: 2,
            },
        )
    }
}

/// Changeling Sentinel — {3}{W} 3/2 Shapeshifter. Changeling, vigilance.
pub fn changeling_sentinel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling, Keyword::Vigilance],
        ..creature("Changeling Sentinel", cost(&[generic(3), w()]), vec![CreatureType::Shapeshifter], 3, 2)
    }
}

/// Solitary Camel — {2}{W} 3/2 Camel. Lifelink as long as you control a
/// Desert or there is a Desert card in your graveyard.
pub fn solitary_camel() -> CardDefinition {
    let desert = || R::HasLandType(LandType::Desert);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Lifelink while you control a Desert or one is in your graveyard.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Lifelink,
                condition: Predicate::Any(vec![
                    Predicate::SelectorExists(Selector::EachPermanent(desert().and(R::ControlledByYou))),
                    Predicate::SelectorExists(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: desert(),
                    }),
                ]),
            },
        }],
        ..creature("Solitary Camel", cost(&[generic(2), w()]), vec![CreatureType::Camel], 3, 2)
    }
}

/// Black Hole — {3}{B} sorcery. Destroy target creature and up to X other
/// target creatures, where X is the number of Attractions you've visited
/// this turn. The engine has no Attractions, so X is always 0.
pub fn black_hole() -> CardDefinition {
    spell("Black Hole", cost(&[generic(3), b()]), CardType::Sorcery, Effect::Destroy { what: target_filtered(R::Creature) })
}

/// Harvesttide Infiltrator // Harvesttide Assailant — {2}{R} 3/2 trampling
/// Human Werewolf, daybound; back a 4/4 trampling Werewolf, nightbound.
pub fn harvesttide_infiltrator() -> CardDefinition {
    let assailant = CardDefinition {
        name: "Harvesttide Assailant",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Werewolf], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Nightbound],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Daybound],
        back_face: Some(Box::new(assailant)),
        ..creature(
            "Harvesttide Infiltrator",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Werewolf],
            3,
            2,
        )
    }
}

/// Call of the Full Moon — {1}{R} Aura. Enchanted creature gets +3/+2 and has
/// trample. At the beginning of each upkeep, if a player cast two or more
/// spells last turn, sacrifice this Aura.
pub fn call_of_the_full_moon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::TwoOrMoreSpellsCastLastTurn),
            effect: Effect::SacrificeSource,
        }],
        ..aura(
            "Call of the Full Moon",
            cost(&[generic(1), r()]),
            Some(EquipBonus { power: 3, toughness: 2, keywords: vec![Keyword::Trample], ..Default::default() }),
        )
    }
}

/// Vampiric Fury — {1}{R} instant. Vampire creatures you control get +2/+0
/// and gain first strike until end of turn.
pub fn vampiric_fury() -> CardDefinition {
    let vampires =
        || Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Vampire)).and(R::ControlledByYou));
    spell(
        "Vampiric Fury",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: vampires(),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: vampires(), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Evolved Sleeper — {B} 1/1 Human. {B}: it becomes a Human Cleric with base
/// power and toughness 2/2. {1}{B}: if it's a Cleric, put a deathtouch counter
/// on it and it becomes a Phyrexian Human Cleric with base 3/3. {1}{B}{B}: if
/// it's a Phyrexian, put a +1/+1 counter on it, then draw a card and lose 1
/// life.
pub fn evolved_sleeper() -> CardDefinition {
    let become_ = |types: Vec<CreatureType>, pt: i32| {
        vec![
            Effect::BecomeCreatureType { what: Selector::This, creature_types: types, duration: Duration::Permanent },
            Effect::SetBasePT {
                what: Selector::This,
                power: Value::Const(pt),
                toughness: Value::Const(pt),
                duration: Duration::Permanent,
            },
        ]
    };
    let if_it_is = |t: CreatureType, then: Vec<Effect>| Effect::If {
        cond: Predicate::EntityMatches { what: Selector::This, filter: R::HasCreatureType(t) },
        then: Box::new(Effect::Seq(then)),
        else_: Box::new(Effect::Noop),
    };
    let mut phyrexian = vec![Effect::AddKeywordCounter {
        what: Selector::This,
        keyword: Keyword::Deathtouch,
        amount: Value::ONE,
    }];
    phyrexian.extend(become_(vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Cleric], 3));
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::Seq(become_(vec![CreatureType::Human, CreatureType::Cleric], 2)),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                effect: if_it_is(CreatureType::Cleric, phyrexian),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b(), b()]),
                effect: if_it_is(
                    CreatureType::Phyrexian,
                    vec![
                        Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                    ],
                ),
                ..Default::default()
            },
        ],
        ..creature("Evolved Sleeper", cost(&[b()]), vec![CreatureType::Human], 1, 1)
    }
}

/// Stallion of Ashmouth — {3}{B} 3/3 Nightmare Horse. Delirium — {1}{B}: +1/+1
/// until end of turn, only with four or more card types in your graveyard.
pub fn stallion_of_ashmouth() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            condition: Some(Predicate::DeliriumActive { who: PlayerRef::You }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Stallion of Ashmouth",
            cost(&[generic(3), b()]),
            vec![CreatureType::Nightmare, CreatureType::Horse],
            3,
            3,
        )
    }
}

/// Thriving Rats — {1}{B} 1/2 Rat. When it enters, you get {E}{E}. Whenever it
/// attacks, you may pay {E}{E}; if you do, put a +1/+1 counter on it.
pub fn thriving_rats() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(2))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Pay {E}{E} for a +1/+1 counter?".into(),
                    body: Box::new(Effect::PayEnergy {
                        amount: 2,
                        then: Box::new(Effect::AddCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        }),
                    }),
                },
            },
        ],
        ..creature("Thriving Rats", cost(&[generic(1), b()]), vec![CreatureType::Rat], 1, 2)
    }
}

// ── Batch fourteen (one-gap decks) ──────────────────────────────────────────

/// Dire Fleet Warmonger — {1}{B}{R} 3/3 Orc Pirate. At the beginning of combat
/// on your turn, you may sacrifice another creature; if you do, it gets +2/+2
/// and gains trample until end of turn.
pub fn dire_fleet_warmonger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::MaySacrifice {
                description: "Sacrifice another creature for +2/+2 and trample?".into(),
                filter: R::Creature.and(R::OtherThanSource),
                count: Value::ONE,
                then: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                ])),
                else_: None,
            },
        }],
        ..creature("Dire Fleet Warmonger", cost(&[generic(1), b(), r()]), vec![CreatureType::Orc, CreatureType::Pirate], 3, 3)
    }
}

/// Draconian Gate-Bot — {5} 4/2 artifact Robot. Its enters modes open an
/// Attraction or destroy one; the engine has no Attractions (no Attraction
/// deck, none on the battlefield), so both modes do nothing here.
pub fn draconian_gate_bot() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature("Draconian Gate-Bot", cost(&[generic(5)]), vec![CreatureType::Robot], 4, 2)
    }
}

// ── Batch fifteen (one- and two-gap decks) ──────────────────────────────────

/// Splitting the Powerstone — {2}{U} sorcery. As an additional cost,
/// sacrifice an artifact. Create two tapped Powerstone tokens; if the
/// sacrificed artifact was legendary, draw a card.
pub fn splitting_the_powerstone() -> CardDefinition {
    let powerstone = TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() };
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Artifact.and(R::ControlledByYou),
            count: 1,
        }],
        ..spell(
            "Splitting the Powerstone",
            cost(&[generic(2), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(powerstone) },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::SacrificedCard,
                        filter: R::HasSupertype(crate::card::Supertype::Legendary),
                    },
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// The Mana Rig — {3} legendary artifact. Whenever you cast a multicolored
/// spell, create a tapped Powerstone token. {X}{X}{X}, {T}: look at the top X
/// cards of your library; put up to two into your hand and the rest on the
/// bottom in a random order.
pub fn the_mana_rig() -> CardDefinition {
    let powerstone = TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() };
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Multicolored,
            }),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(powerstone) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), x(), x()]),
            tap_cost: true,
            effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                rest_bottom_random: true,
                who: PlayerRef::You,
                count: Value::XFromCost,
                take: Some(Value::Const(2)),
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..spell("The Mana Rig", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Shreds of Sanity — {2}{R} sorcery. Return up to one target instant card
/// and up to one target sorcery card from your graveyard to your hand, then
/// discard a card. Exile it.
pub fn shreds_of_sanity() -> CardDefinition {
    let gy_slot = |slot: u8, card_type: CardType| Selector::TargetFiltered {
        slot,
        filter: R::HasCardType(card_type).and(R::InYourGraveyard),
    };
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Shreds of Sanity",
            cost(&[generic(2), r()]),
            CardType::Sorcery,
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: gy_slot(0, CardType::Instant), to: ZoneDest::Hand(PlayerRef::You) },
                    Effect::Move { what: gy_slot(1, CardType::Sorcery), to: ZoneDest::Hand(PlayerRef::You) },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ])),
            },
        )
    }
}

/// Curious Homunculus // Voracious Reader — {1}{U} 1/1 Homunculus. {T}: add
/// {C}, spent only on instants and sorceries. At the beginning of your upkeep,
/// with three or more instant and/or sorcery cards in your graveyard,
/// transform it. Back: a 3/4 colorless Eldrazi Homunculus with prowess;
/// instant and sorcery spells you cast cost {1} less.
pub fn curious_homunculus() -> CardDefinition {
    let spells = R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    let reader = CardDefinition {
        name: "Voracious Reader",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Homunculus],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![crate::effect::shortcut::prowess()],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less.",
            effect: StaticEffect::CostReduction { filter: spells.clone(), amount: 1 },
        }],
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::ONE)),
                    crate::mana::SpendRestriction::InstantSorceryOnly,
                ),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(
                Predicate::ValueAtLeast(
                    Value::CountOf(Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: spells,
                    })),
                    Value::Const(3),
                ),
            ),
            effect: Effect::Transform { what: Selector::This },
        }],
        back_face: Some(Box::new(reader)),
        ..creature("Curious Homunculus", cost(&[generic(1), u()]), vec![CreatureType::Homunculus], 1, 1)
    }
}

// ── Batch sixteen (one-gap decks) ───────────────────────────────────────────

/// Brand of Ill Omen — {3}{R} Aura. Cumulative upkeep {R}. Enchanted
/// creature's controller can't cast creature spells.
pub fn brand_of_ill_omen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CumulativeUpkeep(crate::card::CumulativeUpkeepCost::Mana(cost(&[r()])))],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature's controller can't cast creature spells.",
            effect: StaticEffect::EnchantedControllerCantCastCreatureSpells,
        }],
        ..aura("Brand of Ill Omen", cost(&[generic(3), r()]), None)
    }
}

// ── Batch seventeen (two-gap decks) ─────────────────────────────────────────

/// Mystic Skull // Mystic Monstrosity — {2} artifact. {1}, {T}: add one mana
/// of any color. {5}, {T}: transform it. Back: a 5/6 Construct whose lands
/// tap for any color.
pub fn mystic_skull() -> CardDefinition {
    let any = || ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
        ..Default::default()
    };
    let monstrosity = CardDefinition {
        name: "Mystic Monstrosity",
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 5,
        toughness: 6,
        static_abilities: vec![StaticAbility {
            description: "Lands you control have \"{T}: Add one mana of any color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                ability: any(),
                condition: None,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility { mana_cost: cost(&[generic(1)]), ..any() },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                effect: Effect::Transform { what: Selector::This },
                ..Default::default()
            },
        ],
        back_face: Some(Box::new(monstrosity)),
        ..spell("Mystic Skull", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Harvest Pyre — {1}{R} instant. As an additional cost, exile X cards from
/// your graveyard. It deals X damage to target creature.
pub fn harvest_pyre() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::ExileFromGraveyardXFromCost {
            filter: R::InYourGraveyard,
        }],
        ..spell(
            "Harvest Pyre",
            cost(&[generic(1), r()]),
            CardType::Instant,
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::XFromCost },
        )
    }
}

/// Ore-Rich Stalactite // Cosmium Catalyst — {1}{R} artifact. {T}: add {R},
/// spent only on instants and sorceries. Craft with four or more red instant
/// and/or sorcery cards {3}{R}{R}. Back: {1}{R}, {T}: choose a card used to
/// craft it at random; you may cast it without paying its mana cost.
pub fn ore_rich_stalactite() -> CardDefinition {
    let catalyst = CardDefinition {
        name: "Cosmium Catalyst",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            tap_cost: true,
            effect: Effect::CastWithoutPayingImmediate {
                what: Selector::RandomOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Exile,
                    filter: R::ExiledWithSource,
                })),
                source_zone: Zone::Exile,
                exile_after: false,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Red])),
                        crate::mana::SpendRestriction::InstantSorceryOnly,
                    ),
                },
                ..Default::default()
            },
            crate::effect::shortcut::craft(
                cost(&[generic(3), r(), r()]),
                R::HasColor(Color::Red).and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                4,
            ),
        ],
        back_face: Some(Box::new(catalyst)),
        ..spell("Ore-Rich Stalactite", cost(&[generic(1), r()]), CardType::Artifact, Effect::Noop)
    }
}

// ── Batch eighteen (primitive-gated two-gap decks) ──────────────────────────

/// Dimir Strandcatcher — {2}{U/B}{U/B} 3/3 flying Faerie Rogue. Whenever you
/// attack, surveil X (X = opponents being attacked). At the beginning of each
/// end step, if three or more cards were put into your graveyard from
/// anywhere other than the battlefield this turn, draw a card.
pub fn dimir_strandcatcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::OpponentsAttackedThisCombat },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                    Predicate::ValueAtLeast(
                        Value::CardsPutIntoGraveyardNotFromBattlefieldThisTurn(PlayerRef::You),
                        Value::Const(3),
                    ),
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..creature(
            "Dimir Strandcatcher",
            cost(&[generic(2), hybrid(Color::Blue, Color::Black), hybrid(Color::Blue, Color::Black)]),
            vec![CreatureType::Faerie, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Memory Vessel — {3}{R}{R} artifact. {T}, exile it: each player exiles the
/// top seven cards of their library. Until your next turn, players may play
/// the cards they exiled this way, and can't play cards from their hand.
/// Activate only as a sorcery.
pub fn memory_vessel() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::Move {
                    what: Selector::TopOfLibrary { who: PlayerRef::EachPlayer, count: Value::Const(7) },
                    to: ZoneDest::Exile,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: crate::card::MayPlayDuration::UntilSeatsNextTurn { seat: 0 },
                    to_owner: true,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
                Effect::PlayersCantPlayFromHandUntilYourNextTurn { who: PlayerRef::EachPlayer },
            ]),
            ..Default::default()
        }],
        ..spell("Memory Vessel", cost(&[generic(3), r(), r()]), CardType::Artifact, Effect::Noop)
    }
}

// ── Batch nineteen (claimed one-gap closers) ────────────────────────────────

/// Invasion of New Phyrexia // Teferi Akosa of Zhalfir — {X}{W}{U} Siege,
/// defense 6. When it enters, create X 2/2 white and blue Knight tokens with
/// vigilance. Back (loyalty 4): +1 draw two, then discard two unless you
/// discard a creature card; −2 an emblem "Knights you control get +1/+0 and
/// have ward {1}"; −3 tap any number of untapped creatures you control, and
/// when you do, shuffle target nonland permanent an opponent controls with
/// mana value X or less into its owner's library (X = creatures tapped).
pub fn invasion_of_new_phyrexia() -> CardDefinition {
    use crate::card::{BattleSubtype, LoyaltyAbility, PlaneswalkerSubtype, Supertype};
    let loyalty = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { loyalty_cost, effect, ..Default::default() };
    let knights = || Selector::EachPermanent(R::HasCreatureType(CreatureType::Knight).and(R::ControlledByYou));
    let knight = TokenDefinition {
        keywords: vec![Keyword::Vigilance],
        ..token("Knight", 2, 2, vec![Color::White, Color::Blue], vec![CreatureType::Knight])
    };
    let teferi = CardDefinition {
        name: "Teferi Akosa of Zhalfir",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Teferi], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            loyalty(
                1,
                Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                    Effect::DiscardUnlessKind { who: PlayerRef::You, count: Value::Const(2), instead: R::Creature },
                ]),
            ),
            loyalty(
                -2,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Teferi Akosa of Zhalfir".into(),
                    triggered: vec![],
                    statics: vec![
                        StaticAbility {
                            description: "Knights you control get +1/+0.",
                            effect: StaticEffect::PumpPT { applies_to: knights(), power: 1, toughness: 0 },
                        },
                        StaticAbility {
                            description: "Knights you control have ward {1}.",
                            effect: StaticEffect::GrantKeyword {
                                applies_to: knights(),
                                keyword: Keyword::Ward(WardCost::generic(1)),
                            },
                        },
                    ],
                },
            ),
            loyalty(
                -3,
                Effect::TapAnyNumberThen {
                    filter: R::Creature.and(R::ControlledByYou),
                    body: Box::new(Effect::Reflexive {
                        body: Box::new(Effect::Move {
                            what: target_filtered(
                                R::Permanent
                                    .and(R::Land.negate())
                                    .and(R::ControlledByOpponent)
                                    .and(R::ManaValueAtMostXFromCost),
                            ),
                            to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Shuffled },
                        }),
                    }),
                },
            ),
        ],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of New Phyrexia",
        cost: cost(&[x(), w(), u()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense: 6,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::XFromCost,
            definition: Arc::new(knight),
        })],
        back_face: Some(Box::new(teferi)),
        ..Default::default()
    }
}

/// Long Rest — {X}{G}{G}{G} sorcery. Return X cards with different mana values
/// from your graveyard to your hand; with eight or more returned, your life
/// total becomes your starting life total. Exile it. (The X cards are picked
/// at resolution rather than targeted — INCOMPLETE_CARDS.)
pub fn long_rest() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Long Rest",
            cost(&[x(), g(), g(), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::ReturnDistinctManaValuesFromGraveyard { who: PlayerRef::You, max: Some(Value::XFromCost) },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::CountOf(Box::new(Selector::LastMoved)), Value::Const(8)),
                    then: Box::new(Effect::SetLifeTotal { who: Selector::You, amount: Value::StartingLifeTotal }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Halls of Mist — land. Cumulative upkeep {1}. Creatures that attacked during
/// their controller's last turn can't attack.
pub fn halls_of_mist() -> CardDefinition {
    CardDefinition {
        name: "Halls of Mist",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::CumulativeUpkeep(crate::card::CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        static_abilities: vec![StaticAbility {
            description: "Creatures that attacked during their controller's last turn can't attack.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::CantAttackIfAttackedLastTurn,
            },
        }],
        ..Default::default()
    }
}

/// Three Bowls of Porridge — {2} artifact Food. {2}, {T}: choose one that
/// hasn't been chosen — 2 damage to target creature; tap target creature; or
/// sacrifice it and gain 3 life. (Mode and target are picked at resolution —
/// INCOMPLETE_CARDS.)
pub fn three_bowls_of_porridge() -> CardDefinition {
    let creature = || target_filtered(R::Creature);
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Food], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::ChooseUnchosenMode {
                modes: vec![
                    Effect::Reflexive {
                        body: Box::new(Effect::DealDamage { to: creature(), amount: Value::Const(2) }),
                    },
                    Effect::Reflexive { body: Box::new(Effect::Tap { what: creature() }) },
                    Effect::Seq(vec![
                        Effect::SacrificeSource,
                        Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                    ]),
                ],
            },
            ..Default::default()
        }],
        ..spell("Three Bowls of Porridge", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}
