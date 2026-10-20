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
