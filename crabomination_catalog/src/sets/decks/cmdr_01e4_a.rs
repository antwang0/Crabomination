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
                // "Another": the Hauler is in the graveyard by then (sacrificed as the cost).
                what: target_filtered(R::Creature.and(R::OtherThanSource).from_your_graveyard()),
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

/// Draconian Gate-Bot — {5} 4/2 artifact Robot. When it enters, choose one:
/// open an Attraction (CR 717.4); or destroy target Attraction (to its
/// owner's junkyard, CR 717.6).
pub fn draconian_gate_bot() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::OpenAnAttraction,
            Effect::Destroy { what: target_filtered(R::HasArtifactSubtype(ArtifactSubtype::Attraction)) },
        ]))],
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
            ActivatedAbility {
                craft_or_more: true,
                ..crate::effect::shortcut::craft(
                    cost(&[generic(3), r(), r()]),
                    R::HasColor(Color::Red).and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                    4,
                )
            },
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
/// total becomes your starting life total. Exile it. X targets, their
/// different mana values the cross-slot `ManaValueDiffersFromOtherTargets`.
pub fn long_rest() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Long Rest",
            cost(&[x(), g(), g(), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::TargetsExactlyX {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 20,
                        min_targets: 0,
                        filter: R::Any.from_your_graveyard().and(R::ManaValueDiffersFromOtherTargets),
                        effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
                    }),
                },
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

/// Barroom Brawl — {1}{G} sorcery. Target creature you control fights target
/// creature the opponent to your left controls. Then that player may copy this
/// spell and may choose new targets for the copy.
pub fn barroom_brawl() -> CardDefinition {
    spell(
        "Barroom Brawl",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Fight {
                attacker: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponentToYourLeft),
                },
            },
            Effect::MayCopyThisSpell {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(1))),
                cost: crate::effect::ChainCopyCost::Free,
            },
        ]),
    )
}

// ── Batch twenty (two-gap pair halves) ──────────────────────────────────────

/// Oona's Prowler — {1}{B} 3/1 flying Faerie Rogue. Discard a card: it gets
/// -2/-0 until end of turn. Any player may activate this ability.
pub fn oonas_prowler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            any_player: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(-2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Oona's Prowler", cost(&[generic(1), b()]), vec![CreatureType::Faerie, CreatureType::Rogue], 3, 1)
    }
}

/// Nahiri's Warcrafting — {1}{R}{R} sorcery. 5 damage to target creature,
/// planeswalker, or battle; look at the top X cards (X = the excess damage),
/// exile one, bottom the rest randomly; you may play the exiled card this turn.
pub fn nahiris_warcrafting() -> CardDefinition {
    spell(
        "Nahiri's Warcrafting",
        cost(&[generic(1), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Planeswalker).or(R::HasCardType(CardType::Battle))),
                amount: Value::Const(5),
            },
            Effect::LookTopExileOneMayPlay {
                count: Value::ExcessDamageDealtThisResolution,
                who: PlayerRef::You,
                grant: crate::effect::LookExileGrant::PlayThisTurn,
            },
        ]),
    )
}

/// Kolaghan Stormsinger — {R} 1/1 Human Shaman. Haste. Megamorph {R}. When it
/// is turned face up, target creature gains haste until end of turn.
pub fn kolaghan_stormsinger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Megamorph(cost(&[r()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Kolaghan Stormsinger", cost(&[r()]), vec![CreatureType::Human, CreatureType::Shaman], 1, 1)
    }
}

/// Interdisciplinary Mascot — {6}{U}{U} 5/5 Elemental Fractal. Convoke, ward
/// {3}. When it enters, look at the top four cards; put one into your hand and
/// the rest on the bottom in a random order.
pub fn interdisciplinary_mascot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke, Keyword::Ward(WardCost::generic(3))],
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(crate::effect::LookPick {
            rest_bottom_random: true,
            who: PlayerRef::You,
            count: Value::Const(4),
            take: Some(Value::ONE),
            ..Default::default()
        })))],
        ..creature(
            "Interdisciplinary Mascot",
            cost(&[generic(6), u(), u()]),
            vec![CreatureType::Elemental, CreatureType::Fractal],
            5,
            5,
        )
    }
}

/// Banish into Fable — {4}{W}{U} instant. When you cast it from your hand,
/// copy it if you control an artifact, then copy it if you control an
/// enchantment. Return target nonland permanent to its owner's hand; you
/// create a 2/2 white Knight with vigilance.
pub fn banish_into_fable() -> CardDefinition {
    let control = |t: CardType| {
        Predicate::ValueAtLeast(
            Value::CountOf(Box::new(Selector::EachPermanent(R::HasCardType(t).and(R::ControlledByYou)))),
            Value::ONE,
        )
    };
    let copy = || Effect::CopySpellMayChooseTargets { what: Selector::This, count: Value::ONE };
    let knight = TokenDefinition {
        keywords: vec![Keyword::Vigilance],
        ..token("Knight", 2, 2, vec![Color::White], vec![CreatureType::Knight])
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource).with_filter(Predicate::CastFromHand),
            effect: Effect::Seq(vec![
                Effect::If { cond: control(CardType::Artifact), then: Box::new(copy()), else_: Box::new(Effect::Noop) },
                Effect::If { cond: control(CardType::Enchantment), then: Box::new(copy()), else_: Box::new(Effect::Noop) },
            ]),
        }],
        ..spell(
            "Banish into Fable",
            cost(&[generic(4), w(), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Permanent.and(R::Land.negate())),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                make(1, knight),
            ]),
        )
    }
}

/// Gonti's Machinations — {B} enchantment. Whenever you lose life for the
/// first time each turn, you get {E}. Pay {E}{E}, sacrifice it: each opponent
/// loses 3 life; you gain life equal to the life lost this way.
pub fn gontis_machinations() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::YourControl)
                .with_filter(Predicate::FirstBatchThisTurn {
                    who: PlayerRef::You,
                    kind: crate::effect::FirstBatchKind::LifeLost,
                })
                .once_per_turn(),
            effect: Effect::AddEnergy(Value::ONE),
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 2,
            sac_cost: true,
            effect: Effect::DrainLifeLost {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..spell("Gonti's Machinations", cost(&[b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Die Young — {1}{B} sorcery. Choose target creature. You get {E}{E}, then
/// you may pay any amount of {E}; the creature gets -1/-1 until end of turn
/// for each {E} paid.
pub fn die_young() -> CardDefinition {
    let paid = || Value::Times(Box::new(Value::Const(-1)), Box::new(Value::EnergyPaidThisEffect));
    spell(
        "Die Young",
        cost(&[generic(1), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::AddEnergy(Value::Const(2)),
            Effect::PayAnyEnergy {
                then: Box::new(Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: paid(),
                    toughness: paid(),
                    duration: Duration::EndOfTurn,
                }),
            },
        ]),
    )
}

/// Quicksmith Spy — {3}{U} 2/3 Human Artificer. When it enters, target
/// artifact you control gains "{T}: Draw a card" for as long as you control
/// this creature (chosen as it enters, held while it stays — INCOMPLETE_CARDS).
pub fn quicksmith_spy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChoosePermanentForSource { filter: R::Artifact.and(R::ControlledByYou) })],
        static_abilities: vec![StaticAbility {
            description: "The chosen artifact has \"{T}: Draw a card.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::ChosenPermanentOfSource,
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ..Default::default()
                },
                // "For as long as you control this creature" (CR 611.2b): off once
                // another player takes it.
                condition: Some(Predicate::EntityMatches {
                    what: Selector::ChosenPermanentOfSource,
                    filter: R::ControlledByYou,
                }),
            },
        }],
        ..creature("Quicksmith Spy", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 3)
    }
}

/// Shape Anew — {3}{U} sorcery. The controller of target artifact sacrifices
/// it, then reveals until an artifact card, puts it onto the battlefield, and
/// shuffles the rest into their library.
pub fn shape_anew() -> CardDefinition {
    let controller = || PlayerRef::ControllerOf(Box::new(Selector::Target(0)));
    spell(
        "Shape Anew",
        cost(&[generic(3), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::SacrificePermanent { what: target_filtered(R::Artifact) },
            Effect::RevealUntilFind {
                who: controller(),
                find: R::Artifact,
                to: ZoneDest::Battlefield { controller: controller(), tapped: false },
                cap: Value::Const(200),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::ShuffleIntoLibrary,
            },
        ]),
    )
}

// ── Batch twenty-one (pair-deck closers) ────────────────────────────────────

/// Sharpened Pitchfork — {2} Equipment. Equipped creature has first strike,
/// and gets +1/+1 while it's a Human. Equip {1}.
pub fn sharpened_pitchfork() -> CardDefinition {
    equipment(
        "Sharpened Pitchfork",
        cost(&[generic(2)]),
        cost(&[generic(1)]),
        EquipBonus {
            keywords: vec![Keyword::FirstStrike],
            conditional_pt: Some((
                1,
                1,
                Predicate::EntityMatches {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    filter: R::HasCreatureType(CreatureType::Human),
                },
            )),
            ..Default::default()
        },
    )
}

/// Eyes of the Beholder — {4}{B}{B} instant. Target creature gets -11/-11
/// until end of turn.
pub fn eyes_of_the_beholder() -> CardDefinition {
    spell(
        "Eyes of the Beholder",
        cost(&[generic(4), b(), b()]),
        CardType::Instant,
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-11),
            toughness: Value::Const(-11),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Eldritch Pact — {6}{B} sorcery. Target player draws X cards and loses X
/// life, X = the cards in their graveyard.
pub fn eldritch_pact() -> CardDefinition {
    let x = || Value::GraveyardSizeOf(PlayerRef::Target(0));
    spell(
        "Eldritch Pact",
        cost(&[generic(6), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw { who: target_filtered(R::Player), amount: x() },
            Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: x() },
        ]),
    )
}

/// Sorcerer's Strongbox — {4} artifact. {2}, {T}: flip a coin; on a win,
/// sacrifice it and draw three cards.
pub fn sorcerers_strongbox() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Seq(vec![
                    Effect::SacrificeSource,
                    Effect::Draw { who: Selector::You, amount: Value::Const(3) },
                ])),
                on_tails: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..spell("Sorcerer's Strongbox", cost(&[generic(4)]), CardType::Artifact, Effect::Noop)
    }
}

/// Vessel of Ephemera — {1}{W} enchantment. {2}{W}, sacrifice it: create two
/// 1/1 white Spirit tokens with flying.
pub fn vessel_of_ephemera() -> CardDefinition {
    let spirit = TokenDefinition {
        keywords: vec![Keyword::Flying],
        ..token("Spirit", 1, 1, vec![Color::White], vec![CreatureType::Spirit])
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            sac_cost: true,
            effect: make(2, spirit),
            ..Default::default()
        }],
        ..spell("Vessel of Ephemera", cost(&[generic(1), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Ritual of Rejuvenation — {2}{W} instant. You gain 4 life. Draw a card.
pub fn ritual_of_rejuvenation() -> CardDefinition {
    spell(
        "Ritual of Rejuvenation",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Feaster of Fools — {4}{B}{B} 3/3 Demon. Convoke, flying, devour 2.
pub fn feaster_of_fools() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke, Keyword::Flying, Keyword::Devour(2)],
        ..creature("Feaster of Fools", cost(&[generic(4), b(), b()]), vec![CreatureType::Demon], 3, 3)
    }
}

/// Fetid Imp — {1}{B} 1/2 flying Imp. {B}: it gains deathtouch until end of
/// turn.
pub fn fetid_imp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..creature("Fetid Imp", cost(&[generic(1), b()]), vec![CreatureType::Imp], 1, 2)
    }
}

/// Spirit Away — {5}{U}{U} Aura. You control enchanted creature; it gets
/// +2/+2 and has flying.
pub fn spirit_away() -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains { what: enchanted() })],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets +2/+2.",
                effect: StaticEffect::PumpPT { applies_to: enchanted(), power: 2, toughness: 2 },
            },
            StaticAbility {
                description: "Enchanted creature has flying.",
                effect: StaticEffect::GrantKeyword { applies_to: enchanted(), keyword: Keyword::Flying },
            },
        ],
        ..aura("Spirit Away", cost(&[generic(5), u(), u()]), None)
    }
}

/// Quicksmith Rebel — {3}{R} 3/2 Human Artificer. When it enters, target
/// artifact you control gains "{T}: This artifact deals 2 damage to any
/// target" while you control this creature (chosen as it enters, like
/// Quicksmith Spy — INCOMPLETE_CARDS).
pub fn quicksmith_rebel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChoosePermanentForSource { filter: R::Artifact.and(R::ControlledByYou) })],
        static_abilities: vec![StaticAbility {
            description: "The chosen artifact has \"{T}: This artifact deals 2 damage to any target.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::ChosenPermanentOfSource,
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(2) },
                    ..Default::default()
                },
                // "For as long as you control this creature" (CR 611.2b): off once
                // another player takes it.
                condition: Some(Predicate::EntityMatches {
                    what: Selector::ChosenPermanentOfSource,
                    filter: R::ControlledByYou,
                }),
            },
        }],
        ..creature("Quicksmith Rebel", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Artificer], 3, 2)
    }
}

/// Seasinger — {1}{U}{U} 0/1 Merfolk. When you control no Islands, sacrifice
/// it. You may choose not to untap it. {T}: gain control of target creature
/// whose controller controls an Island for as long as you control this and it
/// remains tapped.
pub fn seasinger() -> CardDefinition {
    let islands = |who: R| Selector::EachPermanent(R::HasLandType(LandType::Island).and(who));
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        state_trigger: Some(crate::card::StateTriggeredAbility {
            condition: Predicate::Not(Box::new(Predicate::SelectorExists(islands(R::ControlledByYou)))),
            effect: Effect::SacrificeSource,
        }),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainControlWhileYouControlSourceTapped {
                what: target_filtered(R::Creature.and(R::ControllerControlsLandType(LandType::Island))),
            },
            ..Default::default()
        }],
        ..creature("Seasinger", cost(&[generic(1), u(), u()]), vec![CreatureType::Merfolk], 0, 1)
    }
}

/// Chandra, Pyrogenius — {4}{R}{R} planeswalker, loyalty 5. +2: 2 damage to
/// each opponent. −3: 4 damage to target creature. −10: 6 damage to target
/// player or planeswalker and each creature that player or that
/// planeswalker's controller controls.
pub fn chandra_pyrogenius() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype};
    let loyalty = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { loyalty_cost, effect, ..Default::default() };
    let victim = || PlayerRef::ControllerOf(Box::new(Selector::Target(0)));
    CardDefinition {
        name: "Chandra, Pyrogenius",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Chandra], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            loyalty(2, Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) }),
            loyalty(-3, Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) }),
            loyalty(
                -10,
                Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::Const(6) },
                    Effect::DealDamage {
                        to: Selector::ControlledBy { who: victim(), filter: R::Creature },
                        amount: Value::Const(6),
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Liberating Combustion — {4}{R} sorcery. 6 damage to target creature; you
/// may search your library and/or graveyard for Chandra, Pyrogenius and put
/// it into your hand.
pub fn liberating_combustion() -> CardDefinition {
    spell(
        "Liberating Combustion",
        cost(&[generic(4), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(6) },
            fetch_named_to_hand("Chandra, Pyrogenius"),
        ]),
    )
}

/// Hunter's Mark — {3}{G} instant; {3} less if it targets a blue permanent you
/// don't control; can't be countered. Target creature you control gets +1/+1,
/// then deals damage equal to its power to target creature or planeswalker
/// you don't control.
pub fn hunters_mark() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        self_cost_reduction_cost_if_target: Some((R::HasColor(Color::Blue).and(R::ControlledByYou.negate()), cost(&[generic(3)]))),
        ..spell(
            "Hunter's Mark",
            cost(&[generic(3), g()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::DealDamageFrom {
                    source: Selector::Target(0),
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.or(R::Planeswalker).and(R::ControlledByYou.negate()),
                    },
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                },
            ]),
        )
    }
}

/// Aggressive Instinct — {1}{G} sorcery. Target creature you control deals
/// damage equal to its power to target creature you don't control.
pub fn aggressive_instinct() -> CardDefinition {
    spell(
        "Aggressive Instinct",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::DealDamageFrom {
            source: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
            to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou.negate()) },
            amount: Value::PowerOf(Box::new(Selector::Target(0))),
        },
    )
}

/// Graceful Takedown — {1}{G} sorcery. Any number of target enchanted
/// creatures you control and up to one other target creature you control each
/// deal damage equal to their power to target creature you don't control.
/// (Every enchanted creature you control deals its damage untargeted —
/// INCOMPLETE_CARDS.)
pub fn graceful_takedown() -> CardDefinition {
    let enchanted = R::Creature.and(R::ControlledByYou).and(R::IsEnchanted);
    CardDefinition {
        effect: Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::ForEach {
                    selector: Selector::EachPermanent(enchanted),
                    body: Box::new(Effect::DealDamageFrom {
                        source: Selector::TriggerSource,
                        to: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou.negate()) },
                        amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                    }),
                },
                Effect::DealDamageFrom {
                    source: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::ControlledByYou).and(R::IsEnchanted.negate()),
                    },
                    to: Selector::Target(0),
                    amount: Value::PowerOf(Box::new(Selector::Target(1))),
                },
            ])),
        },
        ..spell("Graceful Takedown", cost(&[generic(1), g()]), CardType::Sorcery, Effect::Noop)
    }
}

// ── Batch twenty-two (three- and four-gap pair closers) ─────────────────────

/// Explosive Apparatus — {1} artifact. {3}, {T}, sacrifice it: 2 damage to
/// any target.
pub fn explosive_apparatus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..spell("Explosive Apparatus", cost(&[generic(1)]), CardType::Artifact, Effect::Noop)
    }
}

/// Stern Constable — {W} 1/1 Human Soldier. {T}, discard a card: tap target
/// creature.
pub fn stern_constable() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Stern Constable", cost(&[w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Expose Evil — {1}{W} instant. Tap up to two target creatures. Investigate.
pub fn expose_evil() -> CardDefinition {
    spell(
        "Expose Evil",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::Tap { what: Selector::TargetFiltered { slot: 0, filter: R::Creature } },
                    Effect::Tap { what: Selector::TargetFiltered { slot: 1, filter: R::Creature } },
                ])),
            },
            crate::effect::shortcut::investigate(1),
        ]),
    )
}

/// Make a Wish — {3}{G} sorcery. Return two cards at random from your
/// graveyard to your hand.
pub fn make_a_wish() -> CardDefinition {
    spell(
        "Make a Wish",
        cost(&[generic(3), g()]),
        CardType::Sorcery,
        Effect::ReturnRandomFromGraveyard { who: PlayerRef::You, filter: R::Any, count: Value::Const(2) },
    )
}

/// Wailing Ghoul — {1}{B} 1/3 Zombie. When it enters, mill two cards.
pub fn wailing_ghoul() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Mill { who: Selector::You, amount: Value::Const(2) })],
        ..creature("Wailing Ghoul", cost(&[generic(1), b()]), vec![CreatureType::Zombie], 1, 3)
    }
}

/// Farrel's Mantle — {2}{W} Aura. Whenever enchanted creature attacks and
/// isn't blocked, its controller may have it deal damage equal to its power
/// plus 2 to another target creature; if so, it assigns no combat damage this
/// turn.
pub fn farrels_mantle() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::EnchantedBySource),
            effect: Effect::MayDo {
                description: "Have it deal its power plus 2 to another creature (no combat damage)?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::DealDamageFrom {
                        source: host(),
                        to: target_filtered(R::Creature.and(R::IsAttacking.negate())),
                        amount: Value::Sum(vec![Value::PowerOf(Box::new(host())), Value::Const(2)]),
                    },
                    Effect::PreventAllCombatDamageInvolving { target: host() },
                ])),
            },
        }],
        ..aura("Farrel's Mantle", cost(&[generic(2), w()]), None)
    }
}

/// Cauldron's Gift — {4}{B} sorcery. Adamant — with three black mana spent,
/// mill four cards. You may return a creature card from your graveyard to the
/// battlefield with an additional +1/+1 counter.
pub fn cauldrons_gift() -> CardDefinition {
    spell(
        "Cauldron's Gift",
        cost(&[generic(4), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast { color: Color::Black, at_least: 3 },
                then: Box::new(Effect::Mill { who: Selector::You, amount: Value::Const(4) }),
                else_: Box::new(Effect::Noop),
            },
            Effect::ClearLastMoved,
            Effect::MoveChosen {
                from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature },
                filter: None,
                count: Value::ONE,
                up_to: true,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        ]),
    )
}

/// Rakshasa's Secret — {2}{B} sorcery. Target opponent discards two cards. You
/// mill two cards.
pub fn rakshasas_secret() -> CardDefinition {
    spell(
        "Rakshasa's Secret",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Discard { who: target_filtered(R::OpponentPlayer), amount: Value::Const(2), random: false },
            Effect::Mill { who: Selector::You, amount: Value::Const(2) },
        ]),
    )
}

/// Belligerent Brontodon — {5}{G}{W} 4/6 Dinosaur. Each creature you control
/// assigns combat damage equal to its toughness rather than its power.
pub fn belligerent_brontodon() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature you control assigns combat damage equal to its toughness.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::AssignsCombatDamageByToughness,
            },
        }],
        ..creature("Belligerent Brontodon", cost(&[generic(5), g(), w()]), vec![CreatureType::Dinosaur], 4, 6)
    }
}

/// Midnight Duelist — {W} 1/2 Human Soldier. Protection from Vampires.
pub fn midnight_duelist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ProtectionFromCreatureType(CreatureType::Vampire)],
        ..creature("Midnight Duelist", cost(&[w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 2)
    }
}

/// Warden of the Woods — {4}{G}{G} 5/7 Treefolk. Vigilance. Whenever it
/// becomes the target of a spell or ability an opponent controls, you may draw
/// two cards.
pub fn warden_of_the_woods() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::OpponentControl),
            effect: Effect::MayDo {
                description: "Draw two cards?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
            },
        }],
        ..creature("Warden of the Woods", cost(&[generic(4), g(), g()]), vec![CreatureType::Treefolk], 5, 7)
    }
}

/// Savage Hunger — {2}{G} Aura. Enchanted creature gets +1/+0 and has
/// trample. Cycling {2}.
pub fn savage_hunger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..aura(
            "Savage Hunger",
            cost(&[generic(2), g()]),
            Some(EquipBonus { power: 1, keywords: vec![Keyword::Trample], ..Default::default() }),
        )
    }
}

/// Wolfhunter's Quiver — {1} Equipment. Equipped creature has "{T}: 1 damage
/// to any target" and "{T}: 3 damage to target Werewolf creature." Equip {5}.
pub fn wolfhunters_quiver() -> CardDefinition {
    let ping = |amount: i32, to: Selector| ActivatedAbility {
        tap_cost: true,
        effect: Effect::DealDamage { to, amount: Value::Const(amount) },
        ..Default::default()
    };
    equipment(
        "Wolfhunter's Quiver",
        cost(&[generic(1)]),
        cost(&[generic(5)]),
        EquipBonus {
            activated_abilities: vec![
                ping(1, crate::effect::shortcut::target_any()),
                ping(3, target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Werewolf)))),
            ],
            ..Default::default()
        },
    )
}

/// Shimmering Grotto — land. {T}: add {C}. {1}, {T}: add one mana of any
/// color.
pub fn shimmering_grotto() -> CardDefinition {
    CardDefinition {
        name: "Shimmering Grotto",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility { mana_cost: cost(&[generic(1)]), ..crate::sets::tap_add_any_color() },
        ],
        ..Default::default()
    }
}

/// Nature's Embrace — {2}{G} Aura (creature or land). As a creature it gets
/// +2/+2; as a land it has "{T}: Add two mana of any one color."
pub fn natures_embrace() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    let host_is = |r: R| Predicate::EntityMatches { what: host(), filter: r };
    CardDefinition {
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.or(R::Land)) },
        static_abilities: vec![StaticAbility {
            description: "As long as enchanted permanent is a land, it has \"{T}: Add two mana of any one color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: host(),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(2)) },
                    ..Default::default()
                },
                condition: Some(host_is(R::Land)),
            },
        }],
        ..aura(
            "Nature's Embrace",
            cost(&[generic(2), g()]),
            Some(EquipBonus { conditional_pt: Some((2, 2, host_is(R::Creature))), ..Default::default() }),
        )
    }
}

/// Feast of Succession — {4}{B}{B} sorcery. All creatures get -4/-4 until end
/// of turn. You become the monarch.
pub fn feast_of_succession() -> CardDefinition {
    spell(
        "Feast of Succession",
        cost(&[generic(4), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: Value::Const(-4),
                toughness: Value::Const(-4),
                duration: Duration::EndOfTurn,
            },
            Effect::BecomeMonarch { who: PlayerRef::You },
        ]),
    )
}

/// Rusted Relic — {4} artifact. Metalcraft — it's a 5/5 Golem artifact
/// creature while you control three or more artifacts.
pub fn rusted_relic() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Metalcraft — 5/5 Golem artifact creature with three or more artifacts.",
                effect: StaticEffect::SelfIsCreatureIf {
                    condition: Predicate::MetalcraftActive { who: PlayerRef::You },
                    creature_types: vec![CreatureType::Golem],
                },
            },
            // A noncreature definition carries no base P/T (`base_power`).
            StaticAbility {
                description: "It's 5/5.",
                effect: StaticEffect::SelfBasePtFromValue { power: Value::Const(5), toughness: Value::Const(5) },
            },
        ],
        ..spell("Rusted Relic", cost(&[generic(4)]), CardType::Artifact, Effect::Noop)
    }
}

/// Invasion of Muraganda // Primordial Plasm — {4}{G} Siege, defense 6. When
/// it enters, put a +1/+1 counter on target creature you control, then it
/// fights up to one target creature you don't control. Back: a 4/4 Ooze; at
/// the beginning of combat on your turn, another target creature gets +2/+2
/// and loses all abilities until end of turn.
pub fn invasion_of_muraganda() -> CardDefinition {
    let plasm = CardDefinition {
        name: "Primordial Plasm",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ooze], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::LoseAllAbilities { what: Selector::Target(0), duration: Duration::EndOfTurn },
            ]),
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Muraganda",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![crate::card::BattleSubtype::Siege], ..Default::default() },
        defense: 6,
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::Fight {
                    attacker: Selector::Target(0),
                    defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou.negate()) },
                },
            ])),
        })],
        back_face: Some(Box::new(plasm)),
        ..Default::default()
    }
}

/// Dispense Justice — {2}{W} instant. Target player sacrifices an attacking
/// creature; metalcraft — two instead.
pub fn dispense_justice() -> CardDefinition {
    spell(
        "Dispense Justice",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Sacrifice {
            who: target_filtered(R::Player),
            count: Value::IfPred {
                pred: Box::new(Predicate::MetalcraftActive { who: PlayerRef::You }),
                then: Box::new(Value::Const(2)),
                else_: Box::new(Value::ONE),
            },
            filter: R::Creature.and(R::IsAttacking),
        },
    )
}

/// Turtle Blimp — {5} 3/4 flying artifact Vehicle. When it enters, create a
/// 2/2 red Mutant. Crew 2.
pub fn turtle_blimp() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![etb(make(1, token("Mutant", 2, 2, vec![Color::Red], vec![CreatureType::Mutant])))],
        ..spell("Turtle Blimp", cost(&[generic(5)]), CardType::Artifact, Effect::Noop)
    }
}

/// Magma Pummeler — {X}{R}{R} 0/0 Elemental. Enters with X +1/+1 counters.
/// Damage to it while it has one is prevented and removes that many counters;
/// when counters are removed this way, it deals that much damage to any
/// target. (Fires on any +1/+1 counter removal — INCOMPLETE_CARDS.)
pub fn magma_pummeler() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to this while it has a +1/+1 counter, prevent it and remove that many +1/+1 counters.",
            effect: StaticEffect::PreventDamageByRemovingCounters {
                kind: CounterType::PlusOnePlusOne,
                single: false,
                even_without: false,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterRemoved(CounterType::PlusOnePlusOne), EventScope::SelfSource),
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::TriggerEventAmount },
        }],
        ..creature("Magma Pummeler", cost(&[x(), r(), r()]), vec![CreatureType::Elemental], 0, 0)
    }
}

/// Turtle Tracks — {2}{G} sorcery. Any number of target players may each
/// search for a basic land and put it onto the battlefield. (Only you search:
/// the caster targets themselves — INCOMPLETE_CARDS.)
pub fn turtle_tracks() -> CardDefinition {
    spell(
        "Turtle Tracks",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

// ── Batch twenty-three (assist and its decks) ───────────────────────────────

/// Pause for Reflection — {2}{G} instant. Convoke. Prevent all combat damage
/// that would be dealt this turn.
pub fn pause_for_reflection() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        ..spell("Pause for Reflection", cost(&[generic(2), g()]), CardType::Instant, Effect::PreventAllCombatDamageThisTurn)
    }
}

/// The Crowd Goes Wild — {X}{G} sorcery. Assist. Support X (a +1/+1 counter on
/// each of up to X target creatures). Each creature with a +1/+1 counter on
/// it gains trample until end of turn.
pub fn the_crowd_goes_wild() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Assist],
        ..spell(
            "The Crowd Goes Wild",
            cost(&[x(), g()]),
            CardType::Sorcery,
            Effect::CapTargetsAtX {
                body: Box::new(Effect::Seq(vec![
                    Effect::SupportCounters { max_targets: 8, filter: R::Creature },
                    Effect::GrantKeyword {
                        what: Selector::EachPermanent(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        )
    }
}

/// Spellweaver Duo — {6}{U} 4/4 Human Wizard. Assist. When it enters, you may
/// return target tapped creature to its owner's hand.
pub fn spellweaver_duo() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Assist],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::Tapped)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..creature("Spellweaver Duo", cost(&[generic(6), u()]), vec![CreatureType::Human, CreatureType::Wizard], 4, 4)
    }
}

/// Dredging Claw — {2} Equipment. Equipped creature gets +1/+0 and has menace.
/// Whenever a creature enters from your graveyard, you may attach it to that
/// creature. Equip {1}{B}.
pub fn dredging_claw() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                Predicate::TriggerSourceEnteredFromGraveyard,
            ])),
            effect: Effect::MayDo {
                description: "Attach Dredging Claw to it?".into(),
                body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
            },
        }],
        ..equipment(
            "Dredging Claw",
            cost(&[generic(2)]),
            cost(&[generic(1), b()]),
            EquipBonus { power: 1, keywords: vec![Keyword::Menace], ..Default::default() },
        )
    }
}

/// Jade Seedstones // Jadeheart Attendant — {3}{G} artifact. When it enters,
/// distribute three +1/+1 counters among one, two, or three target creatures
/// you control. Craft with creature {5}{G}{G}. Back: a 7/7 Golem artifact
/// creature; when it enters, you gain life equal to the crafted card's mana
/// value.
pub fn jade_seedstones() -> CardDefinition {
    let attendant = CardDefinition {
        name: "Jadeheart Attendant",
        card_types: vec![CardType::Artifact, CardType::Creature],
        color_indicator: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 7,
        toughness: 7,
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::TotalManaValueOf(Box::new(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Exile,
                filter: R::ExiledWithSource,
            })),
        })],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DistributeCounters {
            total: Value::Const(3),
            counter: CounterType::PlusOnePlusOne,
            filter: R::Creature.and(R::ControlledByYou),
            max_targets: 3,
        })],
        activated_abilities: vec![crate::effect::shortcut::craft(cost(&[generic(5), g(), g()]), R::Creature, 1)],
        back_face: Some(Box::new(attendant)),
        ..spell("Jade Seedstones", cost(&[generic(3), g()]), CardType::Artifact, Effect::Noop)
    }
}

/// Memorial to Unity — land. Enters tapped. {T}: add {G}. {2}{G}, {T},
/// sacrifice it: look at the top five; you may put a creature card from among
/// them into your hand; the rest go to the bottom in a random order.
pub fn memorial_to_unity() -> CardDefinition {
    CardDefinition {
        name: "Memorial to Unity",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Green),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                    rest_bottom_random: true,
                    who: PlayerRef::You,
                    count: Value::Const(5),
                    take: Some(Value::ONE),
                    pick_filter: Some(R::Creature),
                    optional: true,
                    ..Default::default()
                })),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Sacellum Godspeaker — {2}{G} 2/2 Elf Druid. {T}: reveal any number of
/// creature cards with power 5 or greater from your hand; add {G} for each
/// (it always reveals them all).
pub fn sacellum_godspeaker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Green,
                    Value::CountOf(Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Hand,
                        filter: R::Creature.and(R::PowerAtLeast(5)),
                    })),
                ),
            },
            ..Default::default()
        }],
        ..creature("Sacellum Godspeaker", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Druid], 2, 2)
    }
}

// ── Batch twenty-four (five-gap pair decks) ─────────────────────────────────

fn second_draw() -> EventSpec {
    EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
        .with_filter(Predicate::ValueEquals(Value::CardsDrawnThisTurn(PlayerRef::You), Value::Const(2)))
        .once_per_turn()
}

fn on_attack(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect }
}

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

/// Centaur Peacemaker — {1}{G}{W} 3/3 Centaur Cleric. When it enters, each
/// player gains 4 life.
pub fn centaur_peacemaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(4) })],
        ..creature("Centaur Peacemaker", cost(&[generic(1), g(), w()]), vec![CreatureType::Centaur, CreatureType::Cleric], 3, 3)
    }
}

/// Golden-Scale Aeronaut — {4}{W} 2/3 flying Dwarf Pilot. Backup 1.
pub fn golden_scale_aeronaut() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::backup(1, vec![Keyword::Flying])],
        ..creature("Golden-Scale Aeronaut", cost(&[generic(4), w()]), vec![CreatureType::Dwarf, CreatureType::Pilot], 2, 3)
    }
}

/// Oakshade Stalker // Moonlit Ambusher — {2}{G} 3/3 Human Ranger Werewolf;
/// castable with flash for {2} more. Daybound. Back: a 6/3 nightbound
/// Werewolf.
pub fn oakshade_stalker() -> CardDefinition {
    let ambusher = CardDefinition {
        name: "Moonlit Ambusher",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Werewolf], ..Default::default() },
        power: 6,
        toughness: 3,
        keywords: vec![Keyword::Nightbound],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Daybound],
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(4), g()]),
            flash: true,
            ..Default::default()
        }),
        back_face: Some(Box::new(ambusher)),
        ..creature(
            "Oakshade Stalker",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Ranger, CreatureType::Werewolf],
            3,
            3,
        )
    }
}

/// Rosemane Centaur — {3}{G}{W} 4/4 Centaur Soldier. Convoke, vigilance.
pub fn rosemane_centaur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke, Keyword::Vigilance],
        ..creature("Rosemane Centaur", cost(&[generic(3), g(), w()]), vec![CreatureType::Centaur, CreatureType::Soldier], 4, 4)
    }
}

/// Vernadi Shieldmate — {1}{G/W} 2/2 Human Soldier. Vigilance.
pub fn vernadi_shieldmate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..creature(
            "Vernadi Shieldmate",
            cost(&[generic(1), hybrid(Color::Green, Color::White)]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Paladin's Shield — {1}{W} Equipment with flash. When it enters, attach it
/// to target creature you control. Equipped creature gets +0/+2. Equip {3}.
pub fn paladins_shield() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(3)]))],
        ..equipment(
            "Paladin's Shield",
            cost(&[generic(1), w()]),
            cost(&[generic(3)]),
            EquipBonus { toughness: 2, ..Default::default() },
        )
    }
}

/// Charging Paladin — {2}{W} 2/2 Human Knight. Whenever it attacks, it gets
/// +0/+3 until end of turn.
pub fn charging_paladin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(pump(Selector::This, 0, 3))],
        ..creature("Charging Paladin", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

/// Ledev Guardian — {3}{W} 2/4 Human Knight. Convoke.
pub fn ledev_guardian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        ..creature("Ledev Guardian", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 4)
    }
}

/// Blessed Alliance — {1}{W} instant, escalate {2}. Choose one or more: target
/// player gains 4 life; untap up to two target creatures (one here —
/// INCOMPLETE_CARDS); target opponent sacrifices an attacking creature.
pub fn blessed_alliance() -> CardDefinition {
    CardDefinition {
        escalate: Some(crate::card::Escalate::Mana(cost(&[generic(2)]))),
        ..spell(
            "Blessed Alliance",
            cost(&[generic(1), w()]),
            CardType::Instant,
            Effect::ChooseModesCast {
                min: 1,
                max: 3,
                allow_repeats: false,
                modes: vec![
                    Effect::GainLife { who: target_filtered(R::Player), amount: Value::Const(4) },
                    Effect::Untap { what: target_filtered(R::Creature), up_to: None },
                    Effect::Sacrifice {
                        who: target_filtered(R::OpponentPlayer),
                        count: Value::ONE,
                        filter: R::Creature.and(R::IsAttacking),
                    },
                ],
            },
        )
    }
}

/// Reknit — {1}{G/W} instant. Regenerate target permanent.
pub fn reknit() -> CardDefinition {
    spell(
        "Reknit",
        cost(&[generic(1), hybrid(Color::Green, Color::White)]),
        CardType::Instant,
        Effect::Regenerate { what: target_filtered(R::Permanent) },
    )
}

/// Staff of the Flame Magus — {3} artifact. Whenever you cast a red spell or a
/// Mountain you control enters, you gain 1 life.
pub fn staff_of_the_flame_magus() -> CardDefinition {
    let gain = || Effect::GainLife { who: Selector::You, amount: Value::ONE };
    let source_is = |kind: EventKind, filter: R| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect: gain(),
    };
    CardDefinition {
        triggered_abilities: vec![
            source_is(EventKind::SpellCast, R::HasColor(Color::Red)),
            source_is(EventKind::EntersBattlefield, R::HasLandType(LandType::Mountain)),
        ],
        ..spell("Staff of the Flame Magus", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Dawning Angel — {4}{W} 3/2 flying Angel. When it enters, you gain 4 life.
pub fn dawning_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(4) })],
        ..creature("Dawning Angel", cost(&[generic(4), w()]), vec![CreatureType::Angel], 3, 2)
    }
}

/// Tavern Ruffian // Tavern Smasher — {3}{R} 2/5 Human Warrior Werewolf,
/// daybound; back a 6/5 nightbound Werewolf.
pub fn tavern_ruffian() -> CardDefinition {
    let smasher = CardDefinition {
        name: "Tavern Smasher",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Werewolf], ..Default::default() },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Nightbound],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Daybound],
        back_face: Some(Box::new(smasher)),
        ..creature(
            "Tavern Ruffian",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Werewolf],
            2,
            5,
        )
    }
}

/// Divine Favor — {1}{W} Aura. When it enters, you gain 3 life. Enchanted
/// creature gets +1/+3.
pub fn divine_favor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) })],
        ..aura("Divine Favor", cost(&[generic(1), w()]), Some(EquipBonus { power: 1, toughness: 3, ..Default::default() }))
    }
}

/// Ainok Guide — {1}{G} 1/1 Dog Scout. When it enters, choose one: a +1/+1
/// counter on it; or search for a basic land and put it on top.
pub fn ainok_guide() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            },
        ]))],
        ..creature("Ainok Guide", cost(&[generic(1), g()]), vec![CreatureType::Dog, CreatureType::Scout], 1, 1)
    }
}

/// Anointer of Champions — {W} 1/1 Human Cleric. {T}: target attacking
/// creature gets +1/+1 until end of turn.
pub fn anointer_of_champions() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: pump(target_filtered(R::Creature.and(R::IsAttacking)), 1, 1),
            ..Default::default()
        }],
        ..creature("Anointer of Champions", cost(&[w()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 1)
    }
}

/// Orchard Spirit — {2}{G} 2/2 Spirit. Can't be blocked except by creatures
/// with flying or reach.
pub fn orchard_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(
            R::HasKeyword(Keyword::Flying).or(R::HasKeyword(Keyword::Reach)),
        ))],
        ..creature("Orchard Spirit", cost(&[generic(2), g()]), vec![CreatureType::Spirit], 2, 2)
    }
}

/// Glorious Charge — {1}{W} instant. Creatures you control get +1/+1 until
/// end of turn.
pub fn glorious_charge() -> CardDefinition {
    spell(
        "Glorious Charge",
        cost(&[generic(1), w()]),
        CardType::Instant,
        pump(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)), 1, 1),
    )
}

/// Seize the Initiative — {W} instant. Target creature gets +1/+1 and gains
/// first strike until end of turn.
pub fn seize_the_initiative() -> CardDefinition {
    spell(
        "Seize the Initiative",
        cost(&[w()]),
        CardType::Instant,
        Effect::Seq(vec![
            pump(target_filtered(R::Creature), 1, 1),
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Academy Drake — {2}{U} 2/2 flying Drake. Kicker {4}; kicked, it enters
/// with two +1/+1 counters.
pub fn academy_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[generic(4)]))],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SpellWasKicked),
                then: Box::new(Value::Const(2)),
                else_: Box::new(Value::Const(0)),
            },
        )),
        ..creature("Academy Drake", cost(&[generic(2), u()]), vec![CreatureType::Drake], 2, 2)
    }
}

/// Drownyard Explorers — {3}{U} 2/4 Human Wizard. When it enters, investigate.
pub fn drownyard_explorers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(crate::effect::shortcut::investigate(1))],
        ..creature("Drownyard Explorers", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 4)
    }
}

/// Halo Scarab — {2} 2/1 artifact creature Insect. {2}, exile it from your
/// graveyard: create a Treasure token.
pub fn halo_scarab() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: make(1, crabomination_base::tokens::treasure_token()),
            ..Default::default()
        }],
        ..creature("Halo Scarab", cost(&[generic(2)]), vec![CreatureType::Insect], 2, 1)
    }
}

/// Sea Hag // Aquatic Ingress — {4}{U} 3/5 Hag. When it enters, creatures
/// your opponents control get -4/-0 until end of turn. Adventure {2}{U}
/// instant: up to two target creatures each get +1/+0 and can't be blocked
/// this turn.
pub fn sea_hag() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(pump(Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)), -4, 0))],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Aquatic Ingress",
            cost: cost(&[generic(2), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    pump(Selector::Target(0), 1, 0),
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
                ])),
            },
        })),
        ..creature("Sea Hag", cost(&[generic(4), u()]), vec![CreatureType::Hag], 3, 5)
    }
}

/// Steelgaze Griffin — {4}{U} 2/4 flying Griffin. Whenever you draw your
/// second card each turn, it gets +2/+0 until end of turn.
pub fn steelgaze_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility { event: second_draw(), effect: pump(Selector::This, 2, 0) }],
        ..creature("Steelgaze Griffin", cost(&[generic(4), u()]), vec![CreatureType::Griffin], 2, 4)
    }
}

/// Arcane Encyclopedia — {3} artifact. {3}, {T}: draw a card.
pub fn arcane_encyclopedia() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Book], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..spell("Arcane Encyclopedia", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Briber's Purse — {X} artifact. Enters with X gem counters. {1}, {T}, remove
/// a gem counter: target creature can't attack or block this turn.
pub fn bribers_purse() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Gem, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Gem, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::CantAttack, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..spell("Briber's Purse", cost(&[x()]), CardType::Artifact, Effect::Noop)
    }
}

/// Flight Spellbomb — {1} artifact. {T}, sacrifice it: target creature gains
/// flying until end of turn. When it is put into a graveyard from the
/// battlefield, you may pay {U}; if you do, draw a card.
pub fn flight_spellbomb() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {U} to draw a card?".into(),
                mana_cost: cost(&[u()]),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: None,
            },
        }],
        ..spell("Flight Spellbomb", cost(&[generic(1)]), CardType::Artifact, Effect::Noop)
    }
}

/// Fountain of Ichor — {3} artifact. {T}: add one mana of any color. {3}: it
/// becomes a 3/3 Dinosaur artifact creature until end of turn.
pub fn fountain_of_ichor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            crate::sets::tap_add_any_color(),
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![CreatureType::Dinosaur],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..spell("Fountain of Ichor", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Cradle of Safety — {1}{U} Aura with flash, enchant creature you control.
/// When it enters, enchanted creature gains hexproof until end of turn.
/// Enchanted creature gets +1/+1.
pub fn cradle_of_safety() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: Selector::AttachedTo(Box::new(Selector::This)),
            keyword: Keyword::Hexproof,
            duration: Duration::EndOfTurn,
        })],
        ..aura("Cradle of Safety", cost(&[generic(1), u()]), Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }))
    }
}

/// Ent-Draught Basin — {2} artifact. {X}, {T}: put a +1/+1 counter on target
/// creature with power X. Activate only as a sorcery.
pub fn ent_draught_basin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::PowerExactlyXFromCost)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..spell("Ent-Draught Basin", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Gatekeeper Gargoyle — {6} 3/3 flying artifact creature Gargoyle. It enters
/// with a +1/+1 counter for each Gate you control.
pub fn gatekeeper_gargoyle() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CountOf(Box::new(Selector::EachPermanent(R::HasLandType(LandType::Gate).and(R::ControlledByYou)))),
        )),
        ..creature("Gatekeeper Gargoyle", cost(&[generic(6)]), vec![CreatureType::Gargoyle], 3, 3)
    }
}

/// Lat-Nam Adept — {3}{U} 3/3 Human Wizard. Whenever you draw your second
/// card each turn, put a +1/+1 counter on it.
pub fn lat_nam_adept() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: second_draw(),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Lat-Nam Adept", cost(&[generic(3), u()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 3)
    }
}

/// Tel-Jilad Defiance — {1}{G} instant. Target creature gains protection from
/// artifacts until end of turn. Draw a card.
pub fn tel_jilad_defiance() -> CardDefinition {
    spell(
        "Tel-Jilad Defiance",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::ProtectionFromMatching(Box::new(R::Artifact)),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Wing Puncture — {G} instant. Target creature you control deals damage
/// equal to its power to target creature with flying.
pub fn wing_puncture() -> CardDefinition {
    spell(
        "Wing Puncture",
        cost(&[g()]),
        CardType::Instant,
        Effect::DealDamageFrom {
            source: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
            to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::HasKeyword(Keyword::Flying)) },
            amount: Value::PowerOf(Box::new(Selector::Target(0))),
        },
    )
}

// ── Batch twenty-five (five- and six-gap pair decks) ────────────────────────

fn artifact_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
        effect,
    }
}

fn from_gy_return_tapped(mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        from_graveyard: true,
        discard_cost: Some((R::Any, 2)),
        effect: Effect::Move {
            what: Selector::This,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        ..Default::default()
    }
}

/// Lifesmith — {1}{G} 2/1 Human Artificer. Whenever you cast an artifact
/// spell, you may pay {1}; if you do, you gain 3 life.
pub fn lifesmith() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![artifact_cast(Effect::MayPay {
            description: "Pay {1} to gain 3 life?".into(),
            mana_cost: cost(&[generic(1)]),
            body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
            else_: None,
        })],
        ..creature("Lifesmith", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 1)
    }
}

/// Painsmith — {1}{B} 2/1 Human Artificer. Whenever you cast an artifact
/// spell, you may have target creature get +2/+0 and gain deathtouch until
/// end of turn.
pub fn painsmith() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![artifact_cast(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                pump(target_filtered(R::Creature), 2, 0),
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            ])),
        })],
        ..creature("Painsmith", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 1)
    }
}

/// Renegade's Getaway — {2}{B} instant. Target permanent gains indestructible
/// until end of turn. Create a 1/1 colorless Servo artifact creature token.
pub fn renegades_getaway() -> CardDefinition {
    let servo = TokenDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..token("Servo", 1, 1, vec![], vec![CreatureType::Servo])
    };
    spell(
        "Renegade's Getaway",
        cost(&[generic(2), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GrantKeyword { what: target_filtered(R::Permanent), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
            make(1, servo),
        ]),
    )
}

/// Tidy Conclusion — {3}{B}{B} instant. Destroy target creature. You gain 1
/// life for each artifact you control.
pub fn tidy_conclusion() -> CardDefinition {
    spell(
        "Tidy Conclusion",
        cost(&[generic(3), b(), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)))),
            },
        ]),
    )
}

/// Resourceful Return — {1}{B} sorcery. Return target creature card from your
/// graveyard to your hand. If you control an artifact, draw a card.
pub fn resourceful_return() -> CardDefinition {
    spell(
        "Resourceful Return",
        cost(&[generic(1), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Creature.and(R::InYourGraveyard)), to: ZoneDest::Hand(PlayerRef::You) },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Alabaster Kirin — {3}{W} 2/3 Kirin. Flying, vigilance.
pub fn alabaster_kirin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..creature("Alabaster Kirin", cost(&[generic(3), w()]), vec![CreatureType::Kirin], 2, 3)
    }
}

/// Angelheart Protector — {2}{W} 3/2 Human Cleric. When it enters, target
/// creature you control gains indestructible until end of turn.
pub fn angelheart_protector() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            keyword: Keyword::Indestructible,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Angelheart Protector", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Cleric], 3, 2)
    }
}

/// Beloved Princess — {W} 1/1 Human Noble. Lifelink. Can't be blocked by
/// creatures with power 3 or greater.
pub fn beloved_princess() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink, Keyword::CantBeBlockedBy(Box::new(R::PowerAtLeast(3)))],
        ..creature("Beloved Princess", cost(&[w()]), vec![CreatureType::Human, CreatureType::Noble], 1, 1)
    }
}

/// Cloudseeder — {1}{U} 1/1 flying Faerie Spellshaper. {U}, {T}, discard a
/// card: create a 1/1 blue flying Faerie named Cloud Sprite that can block
/// only creatures with flying.
pub fn cloudseeder() -> CardDefinition {
    let sprite = TokenDefinition {
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        ..token("Cloud Sprite", 1, 1, vec![Color::Blue], vec![CreatureType::Faerie])
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: make(1, sprite),
            ..Default::default()
        }],
        ..creature("Cloudseeder", cost(&[generic(1), u()]), vec![CreatureType::Faerie, CreatureType::Spellshaper], 1, 1)
    }
}

/// Eternal Isolation — {1}{W} sorcery. Put target creature with power 4 or
/// greater on the bottom of its owner's library.
pub fn eternal_isolation() -> CardDefinition {
    spell(
        "Eternal Isolation",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::Creature.and(R::PowerAtLeast(4))),
            to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom },
        },
    )
}

/// False Defeat — {3}{W} sorcery. Return target creature card from your
/// graveyard to the battlefield.
pub fn false_defeat() -> CardDefinition {
    spell(
        "False Defeat",
        cost(&[generic(3), w()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::Creature.and(R::InYourGraveyard)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Darling of the Masses — {2}{G}{W} 2/4 Elf Citizen. Other Citizens you
/// control get +1/+0. Whenever it attacks, create a 1/1 green and white
/// Citizen.
pub fn darling_of_the_masses() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Citizens you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Citizen).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![on_attack(make(
            1,
            token("Citizen", 1, 1, vec![Color::Green, Color::White], vec![CreatureType::Citizen]),
        ))],
        ..creature("Darling of the Masses", cost(&[generic(2), g(), w()]), vec![CreatureType::Elf, CreatureType::Citizen], 2, 4)
    }
}

/// Elderleaf Mentor — {3}{G} 3/2 Elf Warrior. When it enters, create a 1/1
/// green Elf Warrior.
pub fn elderleaf_mentor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(make(
            1,
            token("Elf Warrior", 1, 1, vec![Color::Green], vec![CreatureType::Elf, CreatureType::Warrior]),
        ))],
        ..creature("Elderleaf Mentor", cost(&[generic(3), g()]), vec![CreatureType::Elf, CreatureType::Warrior], 3, 2)
    }
}

/// Elturgard Ranger — {4}{G} 4/1 Human Elf Ranger. Reach. When it enters,
/// create a 2/2 green Wolf.
pub fn elturgard_ranger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(make(1, token("Wolf", 2, 2, vec![Color::Green], vec![CreatureType::Wolf])))],
        ..creature(
            "Elturgard Ranger",
            cost(&[generic(4), g()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Ranger],
            4,
            1,
        )
    }
}

/// Ledev Champion — {1}{G}{W} 2/2 Elf Knight. Whenever it attacks, you may tap
/// any number of untapped creatures you control; it gets +1/+1 for each.
/// {3}{G}{W}: create a 1/1 white Soldier with lifelink.
pub fn ledev_champion() -> CardDefinition {
    let soldier = TokenDefinition {
        keywords: vec![Keyword::Lifelink],
        ..token("Soldier", 1, 1, vec![Color::White], vec![CreatureType::Soldier])
    };
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::TapAnyNumberThenPumpPerTapped {
            filter: R::Creature.and(R::ControlledByYou).and(R::Untapped).and(R::OtherThanSource),
            power: 1,
            toughness: 1,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g(), w()]),
            effect: make(1, soldier),
            ..Default::default()
        }],
        ..creature("Ledev Champion", cost(&[generic(1), g(), w()]), vec![CreatureType::Elf, CreatureType::Knight], 2, 2)
    }
}

/// Oakhame Ranger // Bring Back — {G/W}x4 2/2 Elf Knight Ranger. {T}:
/// creatures you control get +1/+1 until end of turn. Adventure {G/W}x4
/// sorcery: create two 1/1 white Humans.
pub fn oakhame_ranger() -> CardDefinition {
    let gw = || hybrid(Color::Green, Color::White);
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: pump(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)), 1, 1),
            ..Default::default()
        }],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Bring Back",
            cost: cost(&[gw(), gw(), gw(), gw()]),
            card_types: vec![CardType::Sorcery],
            effect: make(2, token("Human", 1, 1, vec![Color::White], vec![CreatureType::Human])),
        })),
        ..creature(
            "Oakhame Ranger",
            cost(&[gw(), gw(), gw(), gw()]),
            vec![CreatureType::Elf, CreatureType::Knight, CreatureType::Ranger],
            2,
            2,
        )
    }
}

/// Gilt-Leaf Ambush — {2}{G} Kindred instant — Elf. Create two 1/1 green Elf
/// Warriors. Clash with an opponent; if you win, those creatures gain
/// deathtouch until end of turn.
pub fn gilt_leaf_ambush() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        ..spell(
            "Gilt-Leaf Ambush",
            cost(&[generic(2), g()]),
            CardType::Instant,
            Effect::Seq(vec![
                make(2, token("Elf Warrior", 1, 1, vec![Color::Green], vec![CreatureType::Elf, CreatureType::Warrior])),
                Effect::ClashWithOpponent {
                    on_win: Box::new(Effect::GrantKeyword {
                        what: Selector::LastCreatedTokens,
                        keyword: Keyword::Deathtouch,
                        duration: Duration::EndOfTurn,
                    }),
                    on_lose: None,
                    with: None,
                },
            ]),
        )
    }
}

/// Brightwood Tracker — {3}{G} 2/4 Elf Scout. {5}{G}, {T}: look at the top
/// four; you may put a creature card from among them into your hand; the rest
/// go to the bottom in a random order.
pub fn brightwood_tracker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g()]),
            tap_cost: true,
            effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                rest_bottom_random: true,
                who: PlayerRef::You,
                count: Value::Const(4),
                take: Some(Value::ONE),
                pick_filter: Some(R::Creature),
                optional: true,
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..creature("Brightwood Tracker", cost(&[generic(3), g()]), vec![CreatureType::Elf, CreatureType::Scout], 2, 4)
    }
}

/// Safewright Cavalry — {3}{G} 4/4 Elf Warrior. Can't be blocked by more than
/// one creature. {5}: target Elf you control gets +2/+2 until end of turn.
pub fn safewright_cavalry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: pump(target_filtered(R::HasCreatureType(CreatureType::Elf).and(R::ControlledByYou)), 2, 2),
            ..Default::default()
        }],
        ..creature("Safewright Cavalry", cost(&[generic(3), g()]), vec![CreatureType::Elf, CreatureType::Warrior], 4, 4)
    }
}

/// Stoic Grove-Guide — {4}{B/G} 5/4 Elf Druid. {1}{B/G}, exile it from your
/// graveyard: create a 2/2 black and green Elf. Activate only as a sorcery.
pub fn stoic_grove_guide() -> CardDefinition {
    let bg = || hybrid(Color::Black, Color::Green);
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), bg()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: make(1, token("Elf", 2, 2, vec![Color::Black, Color::Green], vec![CreatureType::Elf])),
            ..Default::default()
        }],
        ..creature("Stoic Grove-Guide", cost(&[generic(4), bg()]), vec![CreatureType::Elf, CreatureType::Druid], 5, 4)
    }
}

/// Requiting Hex — {B} instant. As an additional cost you may blight 1.
/// Destroy target creature with mana value 2 or less; if the cost was paid,
/// you gain 2 life.
pub fn requiting_hex() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(crate::card::AdditionalCastCost::Blight { n: 1 }),
        ..spell(
            "Requiting Hex",
            cost(&[b()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature.and(R::ManaValueAtMost(2))) },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Unforgiving Aim — {2}{G} instant. Choose one: destroy target creature with
/// flying; destroy target enchantment; or create a 2/2 black and green Elf.
pub fn unforgiving_aim() -> CardDefinition {
    spell(
        "Unforgiving Aim",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))) },
            Effect::Destroy { what: target_filtered(R::Enchantment) },
            make(1, token("Elf", 2, 2, vec![Color::Black, Color::Green], vec![CreatureType::Elf])),
        ]),
    )
}

/// Incorrigible Youths — {3}{R}{R} 4/3 Vampire. Haste. Madness {2}{R}.
pub fn incorrigible_youths() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Madness(cost(&[generic(2), r()]))],
        ..creature("Incorrigible Youths", cost(&[generic(3), r(), r()]), vec![CreatureType::Vampire], 4, 3)
    }
}

/// Insatiable Gorgers — {2}{R}{R} 5/3 Vampire Berserker. Attacks each combat
/// if able. Madness {3}{R}.
pub fn insatiable_gorgers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustAttack, Keyword::Madness(cost(&[generic(3), r()]))],
        ..creature("Insatiable Gorgers", cost(&[generic(2), r(), r()]), vec![CreatureType::Vampire, CreatureType::Berserker], 5, 3)
    }
}

/// Abandon Reason — {2}{R} instant. Up to two target creatures each get +1/+0
/// and gain first strike until end of turn. Madness {1}{R}.
pub fn abandon_reason() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Madness(cost(&[generic(1), r()]))],
        ..spell(
            "Abandon Reason",
            cost(&[generic(2), r()]),
            CardType::Instant,
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Seq(vec![
                    pump(Selector::Target(0), 1, 0),
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
                ])),
            },
        )
    }
}

/// Distemper of the Blood — {1}{R} sorcery. Target creature gets +2/+2 and
/// gains trample until end of turn. Madness {R}.
pub fn distemper_of_the_blood() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Madness(cost(&[r()]))],
        ..spell(
            "Distemper of the Blood",
            cost(&[generic(1), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                pump(target_filtered(R::Creature), 2, 2),
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ]),
        )
    }
}

/// Advanced Stitchwing — {3}{U}{U} 3/4 flying Zombie Horror. {2}{U}, discard
/// two cards: return it from your graveyard to the battlefield tapped.
pub fn advanced_stitchwing() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![from_gy_return_tapped(cost(&[generic(2), u()]))],
        ..creature("Advanced Stitchwing", cost(&[generic(3), u(), u()]), vec![CreatureType::Zombie, CreatureType::Horror], 3, 4)
    }
}

/// Stitchwing Skaab — {3}{U} 3/1 flying Zombie Horror. {1}{U}, discard two
/// cards: return it from your graveyard to the battlefield tapped.
pub fn stitchwing_skaab() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![from_gy_return_tapped(cost(&[generic(1), u()]))],
        ..creature("Stitchwing Skaab", cost(&[generic(3), u()]), vec![CreatureType::Zombie, CreatureType::Horror], 3, 1)
    }
}

/// Diver Skaab — {3}{U}{U} 3/5 Zombie. Exploit. When it exploits a creature,
/// target creature's owner puts it on the top or bottom of their library.
pub fn diver_skaab() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::exploit(Effect::Move {
            what: target_filtered(R::Creature),
            to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::OwnerChoice },
        })],
        ..creature("Diver Skaab", cost(&[generic(3), u(), u()]), vec![CreatureType::Zombie], 3, 5)
    }
}

/// Lich Lord of Unx — {1}{U}{B} 2/2 Zombie Wizard. {U}{B}, {T}: create a 1/1
/// blue and black Zombie Wizard. {U}{U}{B}{B}: target player loses X life and
/// mills X cards (X = Zombies you control).
pub fn lich_lord_of_unx() -> CardDefinition {
    let zombies = || Value::CountOf(Box::new(Selector::EachPermanent(R::HasCreatureType(CreatureType::Zombie).and(R::ControlledByYou))));
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u(), b()]),
                tap_cost: true,
                effect: make(1, token("Zombie Wizard", 1, 1, vec![Color::Blue, Color::Black], vec![CreatureType::Zombie, CreatureType::Wizard])),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u(), u(), b(), b()]),
                effect: Effect::Seq(vec![
                    Effect::LoseLife { who: target_filtered(R::Player), amount: zombies() },
                    Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: zombies() },
                ]),
                ..Default::default()
            },
        ],
        ..creature("Lich Lord of Unx", cost(&[generic(1), u(), b()]), vec![CreatureType::Zombie, CreatureType::Wizard], 2, 2)
    }
}

/// Stitched Drake — {1}{U}{U} 3/4 flying Zombie Drake. As an additional cost,
/// exile a creature card from your graveyard.
pub fn stitched_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::ExileFromGraveyard { filter: R::Creature, count: 1 }],
        ..creature("Stitched Drake", cost(&[generic(1), u(), u()]), vec![CreatureType::Zombie, CreatureType::Drake], 3, 4)
    }
}

/// Wight — {1}{B} 3/2 Zombie Soldier. Enters tapped. Life Drain — whenever a
/// creature dealt damage by it this turn dies, create a tapped 2/2 black
/// Zombie and exile that card.
pub fn wight() -> CardDefinition {
    let zombie = TokenDefinition { tapped: true, ..token("Zombie", 2, 2, vec![Color::Black], vec![CreatureType::Zombie]) };
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::DamagedBySourceThisTurn,
            }),
            effect: Effect::Seq(vec![make(1, zombie), Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile }]),
        }],
        ..creature("Wight", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Soldier], 3, 2)
    }
}

// ── Batch twenty-six (seven- and eight-gap pair decks) ──────────────────────

fn mine(r: R) -> Selector {
    Selector::EachPermanent(r.and(R::ControlledByYou))
}

fn count_mine(r: R) -> Value {
    Value::CountOf(Box::new(mine(r)))
}

/// Arborback Stomper — {3}{G}{G} 5/4 trampling Beast. When it enters, you
/// gain 5 life.
pub fn arborback_stomper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(5) })],
        ..creature("Arborback Stomper", cost(&[generic(3), g(), g()]), vec![CreatureType::Beast], 5, 4)
    }
}

/// High-Rise Sawjack — {2}{G} 2/3 Elf Citizen with reach. Whenever it blocks a
/// creature with flying, it gets +2/+0 until end of turn.
pub fn high_rise_sawjack() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource).with_filter(Predicate::EntityMatches {
                what: Selector::BlockedAttacker,
                filter: R::HasKeyword(Keyword::Flying),
            }),
            effect: pump(Selector::This, 2, 0),
        }],
        ..creature("High-Rise Sawjack", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Citizen], 2, 3)
    }
}

/// Kujar Seedsculptor — {1}{G} 1/2 Elf Druid. When it enters, put a +1/+1
/// counter on target creature you control.
pub fn kujar_seedsculptor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature("Kujar Seedsculptor", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Druid], 1, 2)
    }
}

/// Scaled Behemoth — {4}{G}{G} 6/7 Crocodile. Hexproof.
pub fn scaled_behemoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        ..creature("Scaled Behemoth", cost(&[generic(4), g(), g()]), vec![CreatureType::Crocodile], 6, 7)
    }
}

/// Tajuru Snarecaster — {2}{G} 1/4 Elf Rogue. Reach.
pub fn tajuru_snarecaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        ..creature("Tajuru Snarecaster", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Rogue], 1, 4)
    }
}

/// For the Family — {G} instant. Target creature gets +2/+2 until end of turn,
/// or +4/+4 if you control four or more creatures.
pub fn for_the_family() -> CardDefinition {
    let big = || Value::IfPred {
        pred: Box::new(Predicate::ValueAtLeast(count_mine(R::Creature), Value::Const(4))),
        then: Box::new(Value::Const(4)),
        else_: Box::new(Value::Const(2)),
    };
    spell(
        "For the Family",
        cost(&[g()]),
        CardType::Instant,
        Effect::PumpPT { what: target_filtered(R::Creature), power: big(), toughness: big(), duration: Duration::EndOfTurn },
    )
}

/// Woodcutter's Grit — {2}{G} instant. Target creature you control gets +3/+3
/// and gains hexproof until end of turn.
pub fn woodcutters_grit() -> CardDefinition {
    spell(
        "Woodcutter's Grit",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            pump(target_filtered(R::Creature.and(R::ControlledByYou)), 3, 3),
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Raven Wings — {2} Equipment. Equipped creature gets +1/+0, has flying, and
/// is a Bird in addition to its other types. Equip {2}.
pub fn raven_wings() -> CardDefinition {
    equipment(
        "Raven Wings",
        cost(&[generic(2)]),
        cost(&[generic(2)]),
        EquipBonus {
            power: 1,
            keywords: vec![Keyword::Flying],
            add_creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
    )
}

/// Graven Abomination — {3} 3/1 artifact creature Horror. Whenever it
/// attacks, exile target card from defending player's graveyard.
pub fn graven_abomination() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![on_attack(Effect::Move {
            what: target_filtered(R::InOpponentGraveyard.and(R::OwnedByDefendingPlayer)),
            to: ZoneDest::Exile,
        })],
        ..creature("Graven Abomination", cost(&[generic(3)]), vec![CreatureType::Horror], 3, 1)
    }
}

/// Maraleaf Rider — {1}{G} 3/1 Elf Knight. Sacrifice a Food: target creature
/// blocks it this turn if able.
pub fn maraleaf_rider() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Food), 1)),
            effect: Effect::MustBlockSource { what: target_filtered(R::Creature), chooser: None },
            ..Default::default()
        }],
        ..creature("Maraleaf Rider", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Knight], 3, 1)
    }
}

/// Taunting Arbormage — {2}{G} 2/3 Elf Wizard. Kicker {3}. When it enters, if
/// it was kicked, all creatures able to block target creature this turn do so.
pub fn taunting_arbormage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::AllMustBlock,
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Taunting Arbormage", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Wizard], 2, 3)
    }
}

/// Wildwood Tracker — {G} 1/1 Elf Warrior. Whenever it attacks or blocks, if
/// you control another non-Human creature, it gets +1/+1 until end of turn.
pub fn wildwood_tracker() -> CardDefinition {
    let gated = || Effect::If {
        cond: Predicate::SelectorExists(mine(
            R::Creature.and(R::HasCreatureType(CreatureType::Human).negate()).and(R::OtherThanSource),
        )),
        then: Box::new(pump(Selector::This, 1, 1)),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        triggered_abilities: vec![
            on_attack(gated()),
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: gated() },
        ],
        ..creature("Wildwood Tracker", cost(&[g()]), vec![CreatureType::Elf, CreatureType::Warrior], 1, 1)
    }
}

/// Spell Snuff — {1}{U}{U} instant. Counter target spell. Fateful hour — with
/// 5 or less life, draw a card.
pub fn spell_snuff() -> CardDefinition {
    spell(
        "Spell Snuff",
        cost(&[generic(1), u(), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::If {
                cond: Predicate::PlayerLifeAtMost { who: PlayerRef::You, life: 5 },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Jeskai Banner — {3} artifact. {T}: add {U}, {R}, or {W}. {U}{R}{W}, {T},
/// sacrifice it: draw a card.
pub fn jeskai_banner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            crate::sets::tap_add(Color::Blue),
            crate::sets::tap_add(Color::Red),
            crate::sets::tap_add(Color::White),
            ActivatedAbility {
                mana_cost: cost(&[u(), r(), w()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..spell("Jeskai Banner", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Adherent of Hope — {1}{W} 2/1 Human Soldier. At the beginning of combat on
/// your turn, if you control a Basri planeswalker, put a +1/+1 counter on it.
pub fn adherent_of_hope() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl).with_filter(
                Predicate::SelectorExists(mine(R::Planeswalker.and(R::HasPlaneswalkerType(PlaneswalkerSubtype::Basri)))),
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Adherent of Hope", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 1)
    }
}

/// Air Nomad Student — {3}{W} 2/2 flying Human Monk. At the beginning of your
/// end step, if it didn't attack this turn, put a +1/+1 counter on it.
pub fn air_nomad_student() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::This, filter: R::AttackedThisTurn.negate() },
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Air Nomad Student", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Monk], 2, 2)
    }
}

/// Dutiful Servants — {3}{W} 2/5 Zombie.
pub fn dutiful_servants() -> CardDefinition {
    creature("Dutiful Servants", cost(&[generic(3), w()]), vec![CreatureType::Zombie], 2, 5)
}

/// Jeskai Student — {1}{W} 1/3 Human Monk. Prowess.
pub fn jeskai_student() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![crate::effect::shortcut::prowess()],
        ..creature("Jeskai Student", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Monk], 1, 3)
    }
}

/// Jeskai Runemark — {2}{U} Aura. Enchanted creature gets +2/+2, and has
/// flying while you control a red or white permanent.
pub fn jeskai_runemark() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has flying as long as you control a red or white permanent.",
            effect: StaticEffect::PumpTeamIf {
                condition: Predicate::SelectorExists(mine(R::Permanent.and(R::HasColor(Color::Red).or(R::HasColor(Color::White))))),
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..aura("Jeskai Runemark", cost(&[generic(2), u()]), Some(EquipBonus { power: 2, toughness: 2, ..Default::default() }))
    }
}

/// Liminal Hold — {3}{W} enchantment. When it enters, exile up to one target
/// nonland permanent an opponent controls until it leaves the battlefield.
/// You gain 2 life.
pub fn liminal_hold() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::ExileUntilSourceLeaves {
                    what: target_filtered(R::Permanent.and(R::ControlledByOpponent).and(R::Land.negate())),
                    return_to: crate::card::ExileReturnZone::Battlefield,
                }),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..spell("Liminal Hold", cost(&[generic(3), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Dazzling Lights — {U} instant. Target creature gets -3/-0 until end of
/// turn. Surveil 2.
pub fn dazzling_lights() -> CardDefinition {
    spell(
        "Dazzling Lights",
        cost(&[u()]),
        CardType::Instant,
        Effect::Seq(vec![
            pump(target_filtered(R::Creature), -3, 0),
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
    )
}

/// Invasion of Belenon // Belenon War Anthem — {2}{W} Siege, defense 5. When
/// it enters, create a 2/2 white and blue Knight with vigilance. Back: an
/// enchantment; creatures you control get +1/+1.
pub fn invasion_of_belenon() -> CardDefinition {
    let anthem = CardDefinition {
        name: "Belenon War Anthem",
        card_types: vec![CardType::Enchantment],
        color_indicator: vec![Color::White],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: mine(R::Creature), power: 1, toughness: 1 },
        }],
        ..Default::default()
    };
    let knight = TokenDefinition {
        keywords: vec![Keyword::Vigilance],
        ..token("Knight", 2, 2, vec![Color::White, Color::Blue], vec![CreatureType::Knight])
    };
    CardDefinition {
        name: "Invasion of Belenon",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![crate::card::BattleSubtype::Siege], ..Default::default() },
        defense: 5,
        triggered_abilities: vec![etb(make(1, knight))],
        back_face: Some(Box::new(anthem)),
        ..Default::default()
    }
}

/// Invasion of Tolvada // The Broken Sky — {3}{W}{B} Siege, defense 5. When it
/// enters, return target nonbattle permanent card from your graveyard to the
/// battlefield. Back: creature tokens you control get +1/+0 and have
/// lifelink; at the beginning of your end step, create a 1/1 white and black
/// flying Spirit.
pub fn invasion_of_tolvada() -> CardDefinition {
    let tokens = || mine(R::Creature.and(R::IsToken));
    let spirit = TokenDefinition {
        keywords: vec![Keyword::Flying],
        ..token("Spirit", 1, 1, vec![Color::White, Color::Black], vec![CreatureType::Spirit])
    };
    let sky = CardDefinition {
        name: "The Broken Sky",
        card_types: vec![CardType::Enchantment],
        color_indicator: vec![Color::White, Color::Black],
        static_abilities: vec![
            StaticAbility {
                description: "Creature tokens you control get +1/+0.",
                effect: StaticEffect::PumpPT { applies_to: tokens(), power: 1, toughness: 0 },
            },
            StaticAbility {
                description: "Creature tokens you control have lifelink.",
                effect: StaticEffect::GrantKeyword { applies_to: tokens(), keyword: Keyword::Lifelink },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: make(1, spirit),
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Tolvada",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![crate::card::BattleSubtype::Siege], ..Default::default() },
        defense: 5,
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Permanent.and(R::HasCardType(CardType::Battle).negate()).and(R::InYourGraveyard)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        back_face: Some(Box::new(sky)),
        ..Default::default()
    }
}

/// Even the Odds — {2}{W} instant. Cast only if you control fewer creatures
/// than each opponent. Create three 1/1 white Soldiers.
pub fn even_the_odds() -> CardDefinition {
    let theirs = Value::CountOf(Box::new(Selector::ControlledBy { who: PlayerRef::Triggerer, filter: R::Creature }));
    let fewer = Predicate::ValueAtLeast(theirs, Value::Sum(vec![count_mine(R::Creature), Value::ONE]));
    CardDefinition {
        cast_condition: Some(Predicate::Not(Box::new(Predicate::ForAnyPlayer {
            who: PlayerRef::EachOpponent,
            pred: Box::new(Predicate::Not(Box::new(fewer))),
        }))),
        ..spell(
            "Even the Odds",
            cost(&[generic(2), w()]),
            CardType::Instant,
            make(3, token("Soldier", 1, 1, vec![Color::White], vec![CreatureType::Soldier])),
        )
    }
}

/// Harsh Sustenance — {1}{W}{B} instant. X damage to any target and you gain X
/// life, X = the creatures you control.
pub fn harsh_sustenance() -> CardDefinition {
    spell(
        "Harsh Sustenance",
        cost(&[generic(1), w(), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: count_mine(R::Creature) },
            Effect::GainLife { who: Selector::You, amount: count_mine(R::Creature) },
        ]),
    )
}

/// Start // Finish — {2}{W} instant: create two 1/1 white Warriors. Aftermath
/// {2}{B} sorcery: as an additional cost, sacrifice a creature; destroy target
/// creature. (Finish sacrifices on resolution, not as a cost —
/// INCOMPLETE_CARDS.)
pub fn start_finish() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(crate::card::SplitCard {
            right: crate::card::SplitHalf {
                cost: cost(&[generic(2), b()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature },
                    Effect::Destroy { what: target_filtered(R::Creature) },
                ]),
            },
            fuse: false,
            aftermath: true,
        })),
        ..spell(
            "Start // Finish",
            cost(&[generic(2), w()]),
            CardType::Instant,
            make(2, TokenDefinition { keywords: vec![Keyword::Vigilance], ..token("Warrior", 1, 1, vec![Color::White], vec![CreatureType::Warrior]) }),
        )
    }
}

/// Take Up Arms — {4}{W} instant. Create three 1/1 white Warriors.
pub fn take_up_arms() -> CardDefinition {
    spell(
        "Take Up Arms",
        cost(&[generic(4), w()]),
        CardType::Instant,
        make(3, token("Warrior", 1, 1, vec![Color::White], vec![CreatureType::Warrior])),
    )
}

/// Icatian Town — {5}{W} sorcery. Create four 1/1 white Citizens.
pub fn icatian_town() -> CardDefinition {
    spell(
        "Icatian Town",
        cost(&[generic(5), w()]),
        CardType::Sorcery,
        make(4, token("Citizen", 1, 1, vec![Color::White], vec![CreatureType::Citizen])),
    )
}

// ── Batch twenty-seven (pair decks: artifacts, moons, Tezzeret) ─────────────

fn aura_on(name: &'static str, mana: ManaCost, on: R, bonus: Option<EquipBonus>) -> CardDefinition {
    CardDefinition { effect: Effect::Attach { what: Selector::This, to: target_filtered(on) }, ..aura(name, mana, bonus) }
}

fn dont_untap() -> StaticAbility {
    StaticAbility {
        description: "Enchanted permanent doesn't untap during its controller's untap step.",
        effect: StaticEffect::PreventUntap { applies_to: Selector::AttachedTo(Box::new(Selector::This)) },
    }
}

/// Academy Journeymage — {4}{U} 3/2 Human Wizard; {1} less with a Wizard. When
/// it enters, return target creature an opponent controls to its owner's hand.
pub fn academy_journeymage() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if_control: vec![(R::HasCreatureType(CreatureType::Wizard).and(R::ControlledByYou), 1)],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature("Academy Journeymage", cost(&[generic(4), u()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 2)
    }
}

/// Embersmith — {1}{R} 2/1 Human Artificer. Whenever you cast an artifact
/// spell, you may pay {1}; if you do, it deals 1 damage to any target.
pub fn embersmith() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![artifact_cast(Effect::MayPay {
            description: "Pay {1} to deal 1 damage?".into(),
            mana_cost: cost(&[generic(1)]),
            body: Box::new(Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::ONE }),
            else_: None,
        })],
        ..creature("Embersmith", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 1)
    }
}

/// Filigree Crawler — {4} 2/2 artifact creature Insect. When it dies, create
/// a 1/1 colorless Thopter artifact creature token with flying.
pub fn filigree_crawler() -> CardDefinition {
    let thopter = TokenDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        ..token("Thopter", 1, 1, vec![], vec![CreatureType::Thopter])
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: make(1, thopter),
        }],
        ..creature("Filigree Crawler", cost(&[generic(4)]), vec![CreatureType::Insect], 2, 2)
    }
}

/// Sliver Construct — {3} 2/2 artifact creature Sliver Construct.
pub fn sliver_construct() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature("Sliver Construct", cost(&[generic(3)]), vec![CreatureType::Sliver, CreatureType::Construct], 2, 2)
    }
}

/// Vedalken Certarch — {U} 1/1 Vedalken Wizard. Metalcraft — {T}: tap target
/// artifact, creature, or land; activate only with three or more artifacts.
pub fn vedalken_certarch() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::MetalcraftActive { who: PlayerRef::You }),
            effect: Effect::Tap { what: target_filtered(R::Artifact.or(R::Creature).or(R::Land)) },
            ..Default::default()
        }],
        ..creature("Vedalken Certarch", cost(&[u()]), vec![CreatureType::Vedalken, CreatureType::Wizard], 1, 1)
    }
}

/// Bonds of Quicksilver — {3}{U} Aura with flash. Enchanted creature doesn't
/// untap during its controller's untap step.
pub fn bonds_of_quicksilver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![dont_untap()],
        ..aura("Bonds of Quicksilver", cost(&[generic(3), u()]), None)
    }
}

/// Chant of the Skifsang — {2}{U} Aura. Enchanted creature gets -13/-0.
pub fn chant_of_the_skifsang() -> CardDefinition {
    aura("Chant of the Skifsang", cost(&[generic(2), u()]), Some(EquipBonus { power: -13, ..Default::default() }))
}

/// Coma Veil — {4}{U} Aura (artifact or creature). Enchanted permanent doesn't
/// untap during its controller's untap step.
pub fn coma_veil() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![dont_untap()],
        ..aura_on("Coma Veil", cost(&[generic(4), u()]), R::Artifact.or(R::Creature), None)
    }
}

/// Sol Grail — {3} artifact. As it enters, choose a color. {T}: add one mana
/// of the chosen color.
pub fn sol_grail() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::ChosenColorOfSource },
            ..Default::default()
        }],
        ..spell("Sol Grail", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Moon Heron — {3}{U} 3/2 flying Spirit Bird.
pub fn moon_heron() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature("Moon Heron", cost(&[generic(3), u()]), vec![CreatureType::Spirit, CreatureType::Bird], 3, 2)
    }
}

/// Moon-Eating Dog — {3}{U} 3/3 Dog. Flying while you control a Yanling
/// planeswalker.
pub fn moon_eating_dog() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you control a Yanling planeswalker, this creature has flying.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Flying,
                condition: Predicate::SelectorExists(mine(R::HasPlaneswalkerType(PlaneswalkerSubtype::Yanling))),
            },
        }],
        ..creature("Moon-Eating Dog", cost(&[generic(3), u()]), vec![CreatureType::Dog], 3, 3)
    }
}

/// Moonlit Scavengers — {5}{U} 4/5 Merfolk Rogue. When it enters, if you
/// control an artifact or enchantment, return target creature an opponent
/// controls to its owner's hand.
pub fn moonlit_scavengers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SelectorExists(mine(R::Artifact.or(R::Enchantment)))),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..creature("Moonlit Scavengers", cost(&[generic(5), u()]), vec![CreatureType::Merfolk, CreatureType::Rogue], 4, 5)
    }
}

/// Nephalia Moondrakes — {5}{U}{U} 5/5 flying Drake. When it enters, target
/// creature gains flying until end of turn. {4}{U}{U}, exile it from your
/// graveyard: creatures you control gain flying until end of turn.
pub fn nephalia_moondrakes() -> CardDefinition {
    let fly = |what| Effect::GrantKeyword { what, keyword: Keyword::Flying, duration: Duration::EndOfTurn };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(fly(target_filtered(R::Creature)))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), u(), u()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: fly(mine(R::Creature)),
            ..Default::default()
        }],
        ..creature("Nephalia Moondrakes", cost(&[generic(5), u(), u()]), vec![CreatureType::Drake], 5, 5)
    }
}

/// Suncleanser — {1}{W} 1/4 Human Cleric. When it enters, choose one: remove
/// all counters from target creature, which can't have counters put on it
/// while Suncleanser remains; or target opponent loses all counters (their
/// "can't get counters" lock is not modelled — INCOMPLETE_CARDS).
pub fn suncleanser() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::RemoveAllCounters { what: target_filtered(R::Creature) },
                Effect::RememberPermanentOnSource { what: Selector::Target(0) },
            ]),
            Effect::RemoveAllPlayerCounters { who: PlayerRef::Target(0) },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "The chosen creature can't have counters put on it.",
            effect: StaticEffect::CantHaveCountersPutOn { applies_to: Selector::ChosenPermanentOfSource },
        }],
        ..creature("Suncleanser", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 4)
    }
}

/// Bound by Moonsilver — {2}{W} Aura. Enchanted creature can't attack, block,
/// or transform. Sacrifice another permanent: attach it to target creature;
/// sorcery speed, once each turn.
pub fn bound_by_moonsilver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature can't transform.",
            effect: StaticEffect::CantTransform { filter: R::IsHostOfSource },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Permanent.and(R::OtherThanSource), 1)),
            sorcery_speed: true,
            once_per_turn: true,
            effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..aura(
            "Bound by Moonsilver",
            cost(&[generic(2), w()]),
            Some(EquipBonus { keywords: vec![Keyword::CantAttack, Keyword::CantBlock], ..Default::default() }),
        )
    }
}

/// Moonlace — {U} instant. Target permanent becomes colorless (spells aren't
/// offered as targets — INCOMPLETE_CARDS).
pub fn moonlace() -> CardDefinition {
    spell(
        "Moonlace",
        cost(&[u()]),
        CardType::Instant,
        Effect::BecomeColor { what: target_filtered(R::Permanent), colors: vec![], duration: Duration::Permanent, additive: false },
    )
}

/// Ominous Parcel — {1} artifact. {2}, {T}, sacrifice it: search for a basic
/// land card, put it into your hand. {5}, {T}, sacrifice it: 4 damage to
/// target creature.
pub fn ominous_parcel() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search { who: PlayerRef::You, filter: R::IsBasicLand, to: ZoneDest::Hand(PlayerRef::You) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
                ..Default::default()
            },
        ],
        ..spell("Ominous Parcel", cost(&[generic(1)]), CardType::Artifact, Effect::Noop)
    }
}

/// Restoration Gearsmith — {2}{W}{B} 3/3 Human Artificer. When it enters,
/// return target artifact or creature card from your graveyard to your hand.
pub fn restoration_gearsmith() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Artifact.or(R::Creature).and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature("Restoration Gearsmith", cost(&[generic(2), w(), b()]), vec![CreatureType::Human, CreatureType::Artificer], 3, 3)
    }
}

/// Seraph of New Capenna // Seraph of New Phyrexia — {2}{W} 2/2 flying Angel
/// Soldier. {4}{B/P}: transform it; sorcery speed. Back: a 3/3 flying
/// Phyrexian Angel; whenever it attacks, you may sacrifice another creature
/// or artifact for +2/+1 until end of turn.
pub fn seraph_of_new_capenna() -> CardDefinition {
    let phyrexia = CardDefinition {
        name: "Seraph of New Phyrexia",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::White, Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian, CreatureType::Angel], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description: "Sacrifice another creature or artifact".into(),
            filter: R::Creature.or(R::Artifact).and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(pump(Selector::This, 2, 1)),
            else_: None,
        })],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), crate::mana::phyrexian(Color::Black)]),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(phyrexia)),
        ..creature("Seraph of New Capenna", cost(&[generic(2), w()]), vec![CreatureType::Angel, CreatureType::Soldier], 2, 2)
    }
}

/// Profane Procession // Tomb of the Dusk Rose — {1}{W}{B} legendary
/// enchantment. {3}{W}{B}: exile target creature; then with three or more
/// cards exiled with it, transform it. Back: a legendary land; {T}: any color;
/// {2}{W}{B}, {T}: put a creature card exiled with it onto the battlefield
/// under your control.
pub fn profane_procession() -> CardDefinition {
    use crate::card::Supertype;
    let tomb = CardDefinition {
        name: "Tomb of the Dusk Rose",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_any_color(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w(), b()]),
                tap_cost: true,
                effect: Effect::MoveChosen {
                    from: Selector::CardsInZone { who: PlayerRef::EachPlayer, zone: Zone::Exile, filter: R::Creature.and(R::ExiledWithSource) },
                    filter: None,
                    count: Value::ONE,
                    up_to: false,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), b()]),
            effect: Effect::Seq(vec![
                Effect::ExileLinked { what: target_filtered(R::Creature) },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::CardsExiledWithSourceCount, Value::Const(3)),
                    then: Box::new(Effect::Transform { what: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        back_face: Some(Box::new(tomb)),
        ..spell("Profane Procession", cost(&[generic(1), w(), b()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Etherium Astrolabe — {2}{U} artifact with flash. {B}, {T}, sacrifice an
/// artifact: draw a card.
pub fn etherium_astrolabe() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            sac_other_may_be_source: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..spell("Etherium Astrolabe", cost(&[generic(2), u()]), CardType::Artifact, Effect::Noop)
    }
}

/// Tezzeret's Gatebreaker — {4} artifact. When it enters, look at the top five;
/// you may put a blue or artifact card from among them into your hand; the
/// rest go to the bottom in a random order. {5}{U}, {T}, sacrifice it:
/// creatures you control can't be blocked this turn.
pub fn tezzerets_gatebreaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(crate::effect::LookPick {
            rest_bottom_random: true,
            who: PlayerRef::You,
            count: Value::Const(5),
            take: Some(Value::ONE),
            pick_filter: Some(R::HasColor(Color::Blue).or(R::Artifact)),
            optional: true,
            ..Default::default()
        })))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GrantKeyword { what: mine(R::Creature), keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..spell("Tezzeret's Gatebreaker", cost(&[generic(4)]), CardType::Artifact, Effect::Noop)
    }
}

/// Salvage Slasher — {1}{B} 1/1 artifact creature Human Rogue. It gets +1/+0
/// for each artifact card in your graveyard.
pub fn salvage_slasher() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+0 for each artifact card in your graveyard.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Artifact },
                per_power: 1,
                per_toughness: 0,
            },
        }],
        ..creature("Salvage Slasher", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Rogue], 1, 1)
    }
}

/// Tezzeret's Touch — {1}{U}{B} Aura (artifact). Enchanted artifact is a
/// creature with base power and toughness 5/5 in addition to its other types.
/// When enchanted artifact is put into a graveyard, return it to its owner's
/// hand.
pub fn tezzerets_touch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::EnchantedBySource),
            effect: Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
        }],
        ..aura_on(
            "Tezzeret's Touch",
            cost(&[generic(1), u(), b()]),
            R::Artifact,
            Some(EquipBonus { set_base_pt: Some((5, 5)), add_card_types: vec![CardType::Creature], ..Default::default() }),
        )
    }
}

/// Tezzeret, Master of Metal — {4}{U}{B} planeswalker, loyalty 5. +1: reveal
/// until an artifact card, put it into your hand, the rest on the bottom in a
/// random order. −3: target opponent loses life equal to the artifacts you
/// control. −8: gain control of all artifacts and creatures target opponent
/// controls.
pub fn tezzeret_master_of_metal() -> CardDefinition {
    use crate::card::{LoyaltyAbility, Supertype};
    let loyalty = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { loyalty_cost, effect, ..Default::default() };
    CardDefinition {
        name: "Tezzeret, Master of Metal",
        cost: cost(&[generic(4), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Tezzeret], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            loyalty(
                1,
                Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: R::Artifact,
                    to: ZoneDest::Hand(PlayerRef::You),
                    cap: Value::Const(200),
                    life_per_revealed: 0,
                    miss_dest: crate::effect::RevealMissDest::BottomRandom,
                },
            ),
            loyalty(-3, Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: count_mine(R::Artifact) }),
            loyalty(
                -8,
                Effect::GainControl {
                    what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Artifact.or(R::Creature) },
                    to: None,
                    duration: Duration::Permanent,
                },
            ),
        ],
        ..Default::default()
    }
}

/// Tezzeret's Ambition — {3}{U}{U} sorcery. Draw three cards. If you control
/// no artifacts, discard a card.
pub fn tezzerets_ambition() -> CardDefinition {
    spell(
        "Tezzeret's Ambition",
        cost(&[generic(3), u(), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::If {
                cond: Predicate::Not(Box::new(Predicate::SelectorExists(mine(R::Artifact)))),
                then: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Tezzeret's Betrayal — {3}{U}{B} sorcery. Destroy target creature; you may
/// search your library and/or graveyard for Tezzeret, Master of Metal and put
/// it into your hand.
pub fn tezzerets_betrayal() -> CardDefinition {
    spell(
        "Tezzeret's Betrayal",
        cost(&[generic(3), u(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            fetch_named_to_hand("Tezzeret, Master of Metal"),
        ]),
    )
}

/// Nimbus Champion — {5}{U} 4/4 flying Avatar Warrior. Whenever it attacks,
/// you may return target creature to its owner's hand if its power is at most
/// the number of Warriors your team controls (you control, outside team play).
pub fn nimbus_champion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::If {
                cond: Predicate::ValueAtLeast(
                    count_mine(R::HasCreatureType(CreatureType::Warrior)),
                    Value::PowerOf(Box::new(Selector::Target(0))),
                ),
                then: Box::new(Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
                else_: Box::new(Effect::Noop),
            }),
        })],
        ..creature("Nimbus Champion", cost(&[generic(5), u()]), vec![CreatureType::Avatar, CreatureType::Warrior], 4, 4)
    }
}

/// Frostfist Strider — {3}{U}{U} 4/4 Elemental Giant. Ward {2}. When it enters,
/// tap target creature an opponent controls and put a stun counter on it.
pub fn frostfist_strider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
        ]))],
        ..creature("Frostfist Strider", cost(&[generic(3), u(), u()]), vec![CreatureType::Elemental, CreatureType::Giant], 4, 4)
    }
}

/// Destructive Tampering — {2}{R} sorcery. Choose one: destroy target
/// artifact; or creatures without flying can't block this turn.
pub fn destructive_tampering() -> CardDefinition {
    spell(
        "Destructive Tampering",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying).negate())),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Sacred Boon — {1}{W} instant. Prevent the next 3 damage that would be dealt
/// to target creature this turn; it gets a +0/+1 counter for each 1 damage
/// prevented (placed as it's prevented, not at the next end step —
/// INCOMPLETE_CARDS).
pub fn sacred_boon() -> CardDefinition {
    spell(
        "Sacred Boon",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::PreventNextDamageWithCounters {
            target: target_filtered(R::Creature),
            amount: Value::Const(3),
            counter: Some(CounterType::PlusZeroPlusOne),
        },
    )
}

/// Merchant's Dockhand — {1} 1/2 artifact creature Construct. {3}{U}, {T}, tap
/// X untapped artifacts you control: look at the top X cards; put one into
/// your hand and the rest on the bottom.
pub fn merchants_dockhand() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            tap_cost: true,
            tap_n_filter: Some((R::Artifact.and(R::OtherThanSource), 0)),
            tap_n_x: true,
            effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                who: PlayerRef::You,
                count: Value::XFromCost,
                take: Some(Value::ONE),
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..creature("Merchant's Dockhand", cost(&[generic(1)]), vec![CreatureType::Construct], 1, 2)
    }
}

// ── Batch twenty-eight (Alharu + Bruse Tarl) ────────────────────────────────

/// Idol of Endurance — {2}{W} artifact. When it enters, exile all creature
/// cards with mana value 3 or less from your graveyard until it leaves the
/// battlefield. {1}{W}, {T}: until end of turn, you may cast a creature spell
/// from among the cards exiled with it without paying its mana cost.
pub fn idol_of_endurance() -> CardDefinition {
    let exiled = || Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Exile, filter: R::Creature.and(R::ExiledWithSource) };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::Creature.and(R::ManaValueAtMost(3)),
            },
            return_to: crate::card::ExileReturnZone::Graveyard,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantMayPlay {
                    what: exiled(),
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: false,
                    any_color: false,
                },
                Effect::OneCastAmongGranted { what: exiled() },
            ]),
            ..Default::default()
        }],
        ..spell("Idol of Endurance", cost(&[generic(2), w()]), CardType::Artifact, Effect::Noop)
    }
}

/// Sanctifier of Souls — {3}{W} 2/3 Human Cleric. Whenever another creature
/// you control enters, it gets +1/+1 until end of turn. {2}{W}, exile a
/// creature card from your graveyard: create a 1/1 white flying Spirit.
pub fn sanctifier_of_souls() -> CardDefinition {
    let spirit = TokenDefinition {
        keywords: vec![Keyword::Flying],
        ..token("Spirit", 1, 1, vec![Color::White], vec![CreatureType::Spirit])
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::OtherThanSource) },
            ),
            effect: pump(Selector::This, 1, 1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            exile_other_filter: Some((R::Creature, 1)),
            effect: make(1, spirit),
            ..Default::default()
        }],
        ..creature("Sanctifier of Souls", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Cleric], 2, 3)
    }
}

/// Thriving Ibex — {3}{W} 2/4 Goat. When it enters, you get {E}{E}. Whenever
/// it attacks, you may pay {E}{E}; if you do, put a +1/+1 counter on it.
pub fn thriving_ibex() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(2))),
            on_attack(Effect::PayEnergyValue {
                amount: Value::Const(2),
                then: Box::new(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
            }),
        ],
        ..creature("Thriving Ibex", cost(&[generic(3), w()]), vec![CreatureType::Goat], 2, 4)
    }
}

/// Zodiac Dog — {2}{R} 2/2 Dog. Mountainwalk.
pub fn zodiac_dog() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        ..creature("Zodiac Dog", cost(&[generic(2), r()]), vec![CreatureType::Dog], 2, 2)
    }
}

/// Zodiac Goat — {R} 1/1 Goat. Mountainwalk.
pub fn zodiac_goat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        ..creature("Zodiac Goat", cost(&[r()]), vec![CreatureType::Goat], 1, 1)
    }
}

/// Farmstead — {W}{W}{W} Aura (land). Enchanted land has "At the beginning of
/// your upkeep, you may pay {W}{W}; if you do, you gain 1 life." (The trigger
/// rides the Aura, so it is the Aura's controller's upkeep — INCOMPLETE_CARDS.)
pub fn farmstead() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {W}{W} to gain 1 life?".into(),
                mana_cost: cost(&[w(), w()]),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
                else_: None,
            },
        }],
        ..aura_on("Farmstead", cost(&[w(), w(), w()]), R::Land, None)
    }
}

/// Astral Confrontation — {4}{W} instant; {1} less for each opponent you're
/// attacking. Exile target creature.
pub fn astral_confrontation() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each opponent you're attacking.",
            effect: StaticEffect::SelfCostReducedByValue { amount: Value::OpponentsAttackedThisCombat },
        }],
        ..spell(
            "Astral Confrontation",
            cost(&[generic(4), w()]),
            CardType::Instant,
            Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::Exile },
        )
    }
}

/// Gaze of Justice — {W} sorcery. As an additional cost, tap three untapped
/// white creatures you control. Exile target creature. Flashback {5}{W}.
pub fn gaze_of_justice() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), w()]))],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::TapPermanents {
            filter: R::Creature.and(R::HasColor(Color::White)),
            count: 3,
        }],
        ..spell("Gaze of Justice", cost(&[w()]), CardType::Sorcery, Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::Exile })
    }
}

// ── Batch twenty-nine (Abdel Adrian + Criminal Past) ────────────────────────

fn enchanted_kw(kw: Keyword) -> StaticAbility {
    StaticAbility {
        description: "Enchanted creature keyword.",
        effect: StaticEffect::GrantKeyword { applies_to: Selector::AttachedTo(Box::new(Selector::This)), keyword: kw },
    }
}

fn exile_enchanted() -> Effect {
    Effect::Exile { what: Selector::AttachedTo(Box::new(Selector::This)) }
}

/// Candletrap — {W} Aura. Enchanted creature has defender; combat damage it
/// would deal is prevented. Coven — {2}{W}, sacrifice it: exile enchanted
/// creature; only with three or more creatures of different powers.
pub fn candletrap() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![enchanted_kw(Keyword::Defender), enchanted_kw(Keyword::DealsNoCombatDamage)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            sac_cost: true,
            condition: Some(Predicate::CovenActive { who: PlayerRef::You }),
            effect: exile_enchanted(),
            ..Default::default()
        }],
        ..aura("Candletrap", cost(&[w()]), None)
    }
}

/// Choking Restraints — {2}{W} Aura. Enchanted creature can't attack or
/// block. {3}{W}{W}, sacrifice it: exile enchanted creature.
pub fn choking_restraints() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![enchanted_kw(Keyword::CantAttack), enchanted_kw(Keyword::CantBlock)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), w()]),
            sac_cost: true,
            effect: exile_enchanted(),
            ..Default::default()
        }],
        ..aura("Choking Restraints", cost(&[generic(2), w()]), None)
    }
}

/// Citizen's Arrest — {1}{W}{W} enchantment. When it enters, exile target
/// creature or planeswalker an opponent controls until it leaves the
/// battlefield.
pub fn citizens_arrest() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent)),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        ..spell("Citizen's Arrest", cost(&[generic(1), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Soaring Hope — {4}{W} Aura. When it enters, you gain 3 life. Enchanted
/// creature has flying. {W}: put it on top of its owner's library.
pub fn soaring_hope() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) })],
        static_abilities: vec![enchanted_kw(Keyword::Flying)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Top },
            },
            ..Default::default()
        }],
        ..aura("Soaring Hope", cost(&[generic(4), w()]), None)
    }
}

/// Weight of Conscience — {1}{W} Aura. Enchanted creature can't attack. Tap
/// two untapped creatures you control that share a creature type: exile
/// enchanted creature.
pub fn weight_of_conscience() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![enchanted_kw(Keyword::CantAttack)],
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature, 2)),
            tap_n_share_creature_type: true,
            effect: exile_enchanted(),
            ..Default::default()
        }],
        ..aura("Weight of Conscience", cost(&[generic(1), w()]), None)
    }
}

/// Angel's Tomb — {3} Artifact. Whenever a creature you control enters, you
/// may have it become a 3/3 white Angel artifact creature with flying until
/// end of turn.
pub fn angels_tomb() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::ControlledByYou) },
            ),
            effect: Effect::MayDo {
                description: "Make Angel's Tomb a 3/3 white Angel with flying?".into(),
                body: Box::new(crate::effect::shortcut::colored_animation(
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        creature_types: vec![CreatureType::Angel],
                        keywords: vec![Keyword::Flying],
                        duration: Duration::EndOfTurn,
                    },
                    &[Color::White],
                )),
            },
        }],
        ..spell("Angel's Tomb", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Scroll of Avacyn — {1} Artifact. {1}, sacrifice it: draw a card. If you
/// control an Angel, you gain 5 life.
pub fn scroll_of_avacyn() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::SelectorExists(mine(R::HasCreatureType(CreatureType::Angel))),
                    then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(5) }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..spell("Scroll of Avacyn", cost(&[generic(1)]), CardType::Artifact, Effect::Noop)
    }
}

/// Puncturing Light — {1}{W} Instant. Destroy target attacking or blocking
/// creature with power 3 or less.
pub fn puncturing_light() -> CardDefinition {
    spell(
        "Puncturing Light",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Destroy {
            what: target_filtered(R::Creature.and(R::PowerAtMost(3)).and(R::IsAttacking.or(R::IsBlocking))),
        },
    )
}

/// Paragon of Modernity — {4} 2/2 flying Angel Warrior artifact creature.
/// {3}: +1/+1 until end of turn; if exactly three colors of mana were spent
/// to activate it, a +1/+1 counter instead.
pub fn paragon_of_modernity() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::If {
                cond: Predicate::ValueEquals(Value::DistinctColorsSpent, Value::Const(3)),
                then: Box::new(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
                else_: Box::new(pump(Selector::This, 1, 1)),
            },
            ..Default::default()
        }],
        ..creature("Paragon of Modernity", cost(&[generic(4)]), vec![CreatureType::Angel, CreatureType::Warrior], 2, 2)
    }
}

/// Guardian Angel — {X}{W} Instant. Prevent the next X damage that would be
/// dealt to any target this turn. (The "until end of turn, you may pay {1}
/// any time … to prevent the next 1 damage to it" rider is 🟡 in
/// INCOMPLETE_CARDS.md: no player-hosted activated ability.)
pub fn guardian_angel() -> CardDefinition {
    spell(
        "Guardian Angel",
        cost(&[x(), w()]),
        CardType::Instant,
        Effect::PreventNextDamage { target: target_filtered(R::any_target()), amount: Value::XFromCost },
    )
}

/// Count Nefaria — {6}{B} legendary 6/6 flying Human Villain. Costs {3} less
/// if you've sacrificed a permanent this turn.
pub fn count_nefaria() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        self_cost_reduction_if: Some((
            Predicate::PermanentsSacrificedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
            3,
        )),
        ..creature("Count Nefaria", cost(&[generic(6), b()]), vec![CreatureType::Human, CreatureType::Villain], 6, 6)
    }
}

/// Nettlevine Blight — {4}{B}{B} Aura, enchant creature or land. Enchanted
/// permanent has "At the beginning of your end step, sacrifice this permanent
/// and attach Nettlevine Blight to a creature or land you control." The
/// trigger is printed on the Aura and keyed on the host's controller's end
/// step (as Instill Furor); that player picks the new host.
pub fn nettlevine_blight() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                filter: Some(Predicate::EntityMatches { what: host(), filter: R::ControlledByActivePlayer }),
                ..EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
            },
            effect: Effect::Seq(vec![
                Effect::SacrificePermanent { what: host() },
                Effect::ChooseOneAmong {
                    what: Selector::ControlledBy { who: PlayerRef::ActivePlayer, filter: R::Creature.or(R::Land) },
                    chooser: PlayerRef::ActivePlayer,
                    chosen: Box::new(Effect::Attach { what: Selector::This, to: Selector::SeparatedPile { chosen: true } }),
                    other: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..aura_on("Nettlevine Blight", cost(&[generic(4), b(), b()]), R::Creature.or(R::Land), None)
    }
}

/// Stench of Decay — {1}{B}{B} Instant. Nonartifact creatures get -1/-1 until
/// end of turn.
pub fn stench_of_decay() -> CardDefinition {
    spell(
        "Stench of Decay",
        cost(&[generic(1), b(), b()]),
        CardType::Instant,
        pump(Selector::EachPermanent(R::Creature.and(R::Artifact.negate())), -1, -1),
    )
}

/// Mephitic Vapors — {2}{B} Sorcery. All creatures get -1/-1 until end of
/// turn. Surveil 2.
pub fn mephitic_vapors() -> CardDefinition {
    spell(
        "Mephitic Vapors",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            pump(Selector::EachPermanent(R::Creature), -1, -1),
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
    )
}

/// The named-card form of [`crate::sets::search_library_and_or_graveyard`]
/// (Niambi, Teferi's Wavecaster, Tezzeret's Betrayal, Liberating Combustion).
fn fetch_named_to_hand(name: &'static str) -> Effect {
    crate::sets::search_library_and_or_graveyard(R::HasName(name.into()))
}

fn you_control_a_teferi() -> Predicate {
    Predicate::SelectorExists(mine(R::Planeswalker.and(R::HasPlaneswalkerType(PlaneswalkerSubtype::Teferi))))
}

/// Niambi, Faithful Healer — {1}{W}{U} legendary 2/2 Human Cleric. When it
/// enters, you may fetch Teferi, Timebender from your library and/or
/// graveyard.
pub fn niambi_faithful_healer() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(fetch_named_to_hand("Teferi, Timebender"))],
        ..creature("Niambi, Faithful Healer", cost(&[generic(1), w(), u()]), vec![CreatureType::Human, CreatureType::Cleric], 2, 2)
    }
}

/// Historian of Zhalfir — {2}{U}{U} 3/3 Human Wizard. Whenever it attacks, if
/// you control a Teferi planeswalker, draw a card (CR 603.4 intervening-if).
pub fn historian_of_zhalfir() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(you_control_a_teferi()),
            effect: Effect::If {
                cond: you_control_a_teferi(),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Historian of Zhalfir", cost(&[generic(2), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 3)
    }
}

/// Teferi's Protege — {2}{U} 2/3 Human Wizard. {1}{U}, {T}: draw a card, then
/// discard a card.
pub fn teferis_protege() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        ..creature("Teferi's Protege", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 3)
    }
}

/// Teferi's Sentinel — {5} 2/6 Golem artifact creature; +4/+0 as long as you
/// control a Teferi planeswalker.
pub fn teferis_sentinel() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "As long as you control a Teferi planeswalker, this creature gets +4/+0.",
            effect: StaticEffect::PumpSelfIf { condition: you_control_a_teferi(), power: 4, toughness: 0, keywords: vec![] },
        }],
        ..creature("Teferi's Sentinel", cost(&[generic(5)]), vec![CreatureType::Golem], 2, 6)
    }
}

/// Teferi's Wavecaster — {3}{U}{U} 3/3 Merfolk Wizard with flash. When it
/// enters, you may fetch Teferi, Timeless Voyager from your library and/or
/// graveyard.
pub fn teferis_wavecaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(fetch_named_to_hand("Teferi, Timeless Voyager"))],
        ..creature("Teferi's Wavecaster", cost(&[generic(3), u(), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 3, 3)
    }
}

/// Teferi, Timeless Voyager — {4}{U}{U} Teferi, loyalty 4. +1: draw a card.
/// −3: put target creature on top of its owner's library. −8: each creature
/// target opponent controls phases out; until the end of your next turn,
/// they can't phase in.
pub fn teferi_timeless_voyager() -> CardDefinition {
    use crate::card::{LoyaltyAbility, Supertype};
    let loyalty = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { loyalty_cost, effect, ..Default::default() };
    CardDefinition {
        name: "Teferi, Timeless Voyager",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Teferi], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            loyalty(1, Effect::Draw { who: Selector::You, amount: Value::ONE }),
            loyalty(
                -3,
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        pos: LibraryPosition::Top,
                    },
                },
            ),
            loyalty(
                -8,
                Effect::TargetPlayerThen {
                    filter: R::OpponentPlayer,
                    then: Box::new(Effect::PhaseOutUntilYourNextTurnEnds {
                        what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Leonardo, Worldly Warrior — {7}{W} legendary 5/5 Mutant Ninja Turtle with
/// affinity for creatures and double strike.
pub fn leonardo_worldly_warrior() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::DoubleStrike],
        affinity_filter: Some(R::Creature.and(R::ControlledByYou)),
        ..creature(
            "Leonardo, Worldly Warrior",
            cost(&[generic(7), w()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            5,
            5,
        )
    }
}

/// Wooden Cane — {3}{W} Equipment. When it enters, create a 2/2 red Mutant
/// token and attach it to that. Equipped creature gets +2/+1. Equip {3}.
pub fn wooden_cane() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(token("Mutant", 2, 2, vec![Color::Red], vec![CreatureType::Mutant])),
            },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        ..equipment(
            "Wooden Cane",
            cost(&[generic(3), w()]),
            cost(&[generic(3)]),
            EquipBonus { power: 2, toughness: 1, ..Default::default() },
        )
    }
}

/// East Wind Avatar — {3}{W} 2/4 flying, vigilant Bird Spirit Avatar.
/// Alliance — whenever another creature you control enters, it gets +1/+0
/// until end of turn.
pub fn east_wind_avatar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: pump(Selector::This, 1, 0),
        }],
        ..creature(
            "East Wind Avatar",
            cost(&[generic(3), w()]),
            vec![CreatureType::Bird, CreatureType::Spirit, CreatureType::Avatar],
            2,
            4,
        )
    }
}

/// Triceraton Regenta — {2}{W} 2/3 vigilant Dinosaur Noble. Whenever it
/// attacks, untap up to one other target creature.
pub fn triceraton_regenta() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_attack(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature.and(R::OtherThanSource),
            effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
        })],
        ..creature("Triceraton Regenta", cost(&[generic(2), w()]), vec![CreatureType::Dinosaur, CreatureType::Noble], 2, 3)
    }
}

/// Dimensional Exile — {1}{W} Aura, enchant basic land you control. When it
/// enters, exile target creature an opponent controls until it leaves.
pub fn dimensional_exile() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        ..aura_on(
            "Dimensional Exile",
            cost(&[generic(1), w()]),
            R::Land.and(R::HasSupertype(Supertype::Basic)).and(R::ControlledByYou),
            None,
        )
    }
}

/// Grounded for Life — {4}{W} Instant. Costs {3} less if it targets a tapped
/// creature. Destroy target creature.
pub fn grounded_for_life() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if_target: Some((R::Creature.and(R::Tapped), 3)),
        ..spell(
            "Grounded for Life",
            cost(&[generic(4), w()]),
            CardType::Instant,
            Effect::Destroy { what: target_filtered(R::Creature) },
        )
    }
}

/// Leo's Guidance — {2}{W} Instant. Put a +1/+1 counter on each of up to
/// three target creatures. Untap them.
pub fn leos_guidance() -> CardDefinition {
    spell(
        "Leo's Guidance",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::Untap { what: Selector::Target(0), up_to: None },
            ])),
        },
    )
}

/// Quicksilver, Pietro Maximoff — {2}{R} legendary 3/2 Mutant Hero with haste.
pub fn quicksilver_pietro_maximoff() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        ..creature("Quicksilver, Pietro Maximoff", cost(&[generic(2), r()]), vec![CreatureType::Mutant, CreatureType::Hero], 3, 2)
    }
}

/// Javelin of Lightning — {1}{R} Equipment with flash. When it enters, attach
/// it to target creature you control. During your turn, equipped creature gets
/// +2/+0 and has first strike. Equip {4}.
pub fn javelin_of_lightning() -> CardDefinition {
    let mut javelin = equipment(
        "Javelin of Lightning",
        cost(&[generic(1), r()]),
        cost(&[generic(4)]),
        EquipBonus {
            conditional: vec![crate::card::ConditionalEquipBonus {
                host_filter: R::Creature,
                power: 2,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
                condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
                ..Default::default()
            }],
            ..Default::default()
        },
    );
    javelin.keywords.push(Keyword::Flash);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        ..javelin
    }
}

/// Guildmages' Forum — Land. {T}: {C}. {1}, {T}: one mana of any color; spent
/// on a multicolored creature spell, that creature enters with an additional
/// +1/+1 counter.
pub fn guildmages_forum() -> CardDefinition {
    CardDefinition {
        name: "Guildmages' Forum",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        crate::mana::SpendRestriction::MulticoloredCreatureCastCounter,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Coal Stoker — {3}{R} 3/3 Elemental. When it enters, if you cast it from
/// your hand, add {R}{R}{R}.
pub fn coal_stoker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SourceCastFromOwnersHand,
            then: Box::new(Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, Value::Const(3)) }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Coal Stoker", cost(&[generic(3), r()]), vec![CreatureType::Elemental], 3, 3)
    }
}

/// Havengul Vampire — {3}{R} 2/2 Vampire. A +1/+1 counter whenever it deals
/// combat damage to a player and whenever another creature dies.
pub fn havengul_vampire() -> CardDefinition {
    let grow = || Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: grow(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource }),
                effect: grow(),
            },
        ],
        ..creature("Havengul Vampire", cost(&[generic(3), r()]), vec![CreatureType::Vampire], 2, 2)
    }
}

/// Toxic Iguanar — {R} 1/1 Lizard; deathtouch as long as you control a green
/// permanent.
pub fn toxic_iguanar() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has deathtouch as long as you control a green permanent.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(mine(R::Permanent.and(R::HasColor(Color::Green)))),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Deathtouch],
            },
        }],
        ..creature("Toxic Iguanar", cost(&[r()]), vec![CreatureType::Lizard], 1, 1)
    }
}

/// Rockslide Ambush — {1}{R} Sorcery. It deals damage to target creature equal
/// to the number of Mountains you control.
pub fn rockslide_ambush() -> CardDefinition {
    spell(
        "Rockslide Ambush",
        cost(&[generic(1), r()]),
        CardType::Sorcery,
        Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: count_mine(R::HasLandType(LandType::Mountain)),
        },
    )
}

/// Relic Amulet — {2} Artifact. Whenever you cast an instant, sorcery, or
/// Wizard spell, put a charge counter on it. {2}, {T}, remove all charge
/// counters: it deals that much damage to target creature.
pub fn relic_amulet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .or(R::HasCreatureType(CreatureType::Wizard)),
            }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            remove_all_counters_cost: Some(CounterType::Charge),
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::CountersRemovedAsCost },
            ..Default::default()
        }],
        ..spell("Relic Amulet", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// "{N}, {T}, Sacrifice this artifact: it deals `dmg` damage to target
/// creature" (Scalding Cauldron, Vial of Dragonfire).
fn sac_pinger(name: &'static str, mana: ManaCost, activate: u32, dmg: i32) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(activate)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(dmg) },
            ..Default::default()
        }],
        ..spell(name, mana, CardType::Artifact, Effect::Noop)
    }
}

/// Scalding Cauldron — {1} Artifact. {3}, {T}, sacrifice it: 3 damage to
/// target creature.
pub fn scalding_cauldron() -> CardDefinition {
    sac_pinger("Scalding Cauldron", cost(&[generic(1)]), 3, 3)
}

/// Vial of Dragonfire — {2} Artifact. {2}, {T}, sacrifice it: 2 damage to
/// target creature.
pub fn vial_of_dragonfire() -> CardDefinition {
    sac_pinger("Vial of Dragonfire", cost(&[generic(2)]), 2, 2)
}

/// Tumble Magnet — {3} Artifact; enters with three charge counters. {T},
/// remove a charge counter: tap target artifact or creature.
pub fn tumble_magnet() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Charge, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Charge, 1)),
            effect: Effect::Tap { what: target_filtered(R::Artifact.or(R::Creature)) },
            ..Default::default()
        }],
        ..spell("Tumble Magnet", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Weapon Rack — {4} Artifact; enters with three +1/+1 counters. {T}: move a
/// +1/+1 counter from it onto target creature (sorcery speed).
pub fn weapon_rack() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::MoveCounters {
                from: Selector::This,
                to: target_filtered(R::Creature),
                counter: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..spell("Weapon Rack", cost(&[generic(4)]), CardType::Artifact, Effect::Noop)
    }
}

/// Grazing Kelpie — {3}{G/U} 2/3 Beast with persist. {G/U}, sacrifice it:
/// put target card from a graveyard on the bottom of its owner's library.
pub fn grazing_kelpie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Persist],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[hybrid(Color::Green, Color::Blue)]),
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::InGraveyard),
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom },
            },
            ..Default::default()
        }],
        ..creature("Grazing Kelpie", cost(&[generic(3), hybrid(Color::Green, Color::Blue)]), vec![CreatureType::Beast], 2, 3)
    }
}

/// Nephalia Seakite — {3}{U} 2/3 Bird with flash and flying.
pub fn nephalia_seakite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        ..creature("Nephalia Seakite", cost(&[generic(3), u()]), vec![CreatureType::Bird], 2, 3)
    }
}

/// Primal Plasma — {3}{U} */* Elemental Shapeshifter. As it enters, it
/// becomes a 3/3, a 2/2 with flying, or a 1/6 with defender.
pub fn primal_plasma() -> CardDefinition {
    use crate::card::EntersChoiceMode;
    let mode = |power, toughness, keywords| EntersChoiceMode { power, toughness, keywords };
    CardDefinition {
        enters_as_choice: Some(vec![
            mode(3, 3, vec![]),
            mode(2, 2, vec![Keyword::Flying]),
            mode(1, 6, vec![Keyword::Defender]),
        ]),
        ..creature("Primal Plasma", cost(&[generic(3), u()]), vec![CreatureType::Elemental, CreatureType::Shapeshifter], 0, 0)
    }
}

/// Ethereal Ambush — {3}{G}{U} Instant. Manifest the top two cards of your
/// library.
pub fn ethereal_ambush() -> CardDefinition {
    spell(
        "Ethereal Ambush",
        cost(&[generic(3), g(), u()]),
        CardType::Instant,
        Effect::Manifest { who: PlayerRef::You, amount: Value::Const(2) },
    )
}

/// Bronze Cudgels — {1} Equipment. {2}: until end of turn, equipped creature
/// gets +X/+0, X the number of times this ability has resolved this turn
/// (read as activations: this one included, so the first gives +1). Equip {1}.
pub fn bronze_cudgels() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            max_activations_per_turn: Some(u32::MAX),
            effect: Effect::PumpPT {
                what: Selector::attached_to(Selector::This),
                power: Value::SourceActivationsThisTurn,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..equipment("Bronze Cudgels", cost(&[generic(1)]), cost(&[generic(1)]), EquipBonus::default())
    }
}

/// Honed Khopesh — {1} Equipment. Equipped creature gets +1/+1. Equip {1}.
pub fn honed_khopesh() -> CardDefinition {
    equipment(
        "Honed Khopesh",
        cost(&[generic(1)]),
        cost(&[generic(1)]),
        EquipBonus { power: 1, toughness: 1, ..Default::default() },
    )
}

/// An Equipment that attaches itself to target creature you control as it
/// enters (Scavenged Blade, Utility Knife).
fn self_attaching(def: CardDefinition) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        ..def
    }
}

/// Scavenged Blade — {1}{R} Equipment. When it enters, attach it to target
/// creature you control. +2/+0. Equip {2}{R}.
pub fn scavenged_blade() -> CardDefinition {
    self_attaching(equipment(
        "Scavenged Blade",
        cost(&[generic(1), r()]),
        cost(&[generic(2), r()]),
        EquipBonus { power: 2, ..Default::default() },
    ))
}

/// Utility Knife — {1} Equipment. When it enters, attach it to target
/// creature you control. +1/+1. Equip {3}.
pub fn utility_knife() -> CardDefinition {
    self_attaching(equipment(
        "Utility Knife",
        cost(&[generic(1)]),
        cost(&[generic(3)]),
        EquipBonus { power: 1, toughness: 1, ..Default::default() },
    ))
}

/// Thirsting Axe — {3} Equipment. +4/+0. At the beginning of your end step,
/// if equipped creature didn't deal combat damage to a creature this turn,
/// sacrifice it. Equip {2}. The Axe remembers the damage, not the creature:
/// one that hit a creature unequipped and was equipped after still loses it.
pub fn thirsting_axe() -> CardDefinition {
    let host = || Selector::attached_to(Selector::This);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(Predicate::All(
                vec![
                    Predicate::SelectorExists(host()),
                    Predicate::Not(Box::new(Predicate::SourceMarkedThisTurn)),
                ],
            )),
            effect: Effect::SacrificePermanent { what: host() },
        }],
        ..equipment(
            "Thirsting Axe",
            cost(&[generic(3)]),
            cost(&[generic(2)]),
            EquipBonus {
                power: 4,
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
                    effect: Effect::MarkSourceThisTurn,
                }],
                triggers_on_equipment: true,
                ..Default::default()
            },
        )
    }
}

/// Dwarven Hold — land; enters tapped; may stay tapped (CR 502.3). Upkeep, if
/// tapped: a storage counter. {T}, remove any number of storage counters: {R}
/// for each.
pub fn dwarven_hold() -> CardDefinition {
    CardDefinition {
        name: "Dwarven Hold",
        card_types: vec![CardType::Land],
        keywords: vec![Keyword::MayChooseNotToUntap],
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::Tapped }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Storage, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_x: Some(CounterType::Storage),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![Color::Red], Value::XFromCost) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Armory Veteran — {1}{R} 2/2 Orc Warrior; menace while equipped.
pub fn armory_veteran() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is equipped, it has menace.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::IsEquipped },
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Menace],
            },
        }],
        ..creature("Armory Veteran", cost(&[generic(1), r()]), vec![CreatureType::Orc, CreatureType::Warrior], 2, 2)
    }
}

/// Fire Shrine Keeper — {R} 1/1 Elemental with menace. {7}{R}, {T}, sacrifice
/// it: 3 damage to each of up to two target creatures.
pub fn fire_shrine_keeper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(7), r()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) }),
            },
            ..Default::default()
        }],
        ..creature("Fire Shrine Keeper", cost(&[r()]), vec![CreatureType::Elemental], 1, 1)
    }
}

/// Flowstone Infusion — {R} Instant. Target creature gets +2/-2 until end of
/// turn.
pub fn flowstone_infusion() -> CardDefinition {
    spell("Flowstone Infusion", cost(&[r()]), CardType::Instant, pump(target_filtered(R::Creature), 2, -2))
}

fn snow_land() -> R {
    R::Land.and(R::IsSnow)
}

/// Arcum's Sleigh — {1} Artifact. {2}, {T}: target creature gains vigilance
/// until end of turn; only during combat and only if defending player
/// controls a snow land.
pub fn arcums_sleigh() -> CardDefinition {
    let combat = Predicate::Any(
        [
            TurnStep::BeginCombat,
            TurnStep::DeclareAttackers,
            TurnStep::DeclareBlockers,
            TurnStep::FirstStrikeDamage,
            TurnStep::CombatDamage,
            TurnStep::EndCombat,
        ]
        .into_iter()
        .map(Predicate::CurrentStepIs)
        .collect(),
    );
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            condition: Some(Predicate::All(vec![
                combat,
                Predicate::SelectorExists(Selector::ControlledBy { who: PlayerRef::DefendingPlayer, filter: snow_land() }),
            ])),
            effect: Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..spell("Arcum's Sleigh", cost(&[generic(1)]), CardType::Artifact, Effect::Noop)
    }
}

/// Arcum's Weathervane — {2} Artifact. {2}, {T}: target snow land is no longer
/// snow. {2}, {T}: target nonsnow basic land becomes snow.
pub fn arcums_weathervane() -> CardDefinition {
    use crate::card::Supertype;
    let vane = |filter: R, gain: bool| ActivatedAbility {
        mana_cost: cost(&[generic(2)]),
        tap_cost: true,
        effect: Effect::ChangeSupertype { what: target_filtered(filter), supertype: Supertype::Snow, gain, duration: Duration::Permanent },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            vane(snow_land(), false),
            vane(R::Land.and(R::HasSupertype(Supertype::Basic)).and(R::IsSnow.negate()), true),
        ],
        ..spell("Arcum's Weathervane", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Sunstone — {3} Artifact. {2}, sacrifice a snow land: prevent all combat
/// damage that would be dealt this turn.
pub fn sunstone() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((snow_land().and(R::ControlledByYou), 1)),
            effect: Effect::PreventAllCombatDamageThisTurn,
            ..Default::default()
        }],
        ..spell("Sunstone", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Balduvian Conjurer — {1}{U} 0/2 Human Wizard. {T}: target snow land
/// becomes a 2/2 creature until end of turn (still a land).
pub fn balduvian_conjurer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeCreature {
                what: target_filtered(snow_land()),
                power: Value::Const(2),
                toughness: Value::Const(2),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Balduvian Conjurer", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Wizard], 0, 2)
    }
}

/// Balduvian Frostwaker — {2}{U} 1/1 Human Wizard. {U}, {T}: target snow land
/// becomes a 2/2 blue Elemental creature with flying (still a land), for good.
pub fn balduvian_frostwaker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: crate::effect::shortcut::colored_animation(
                Effect::BecomeCreature {
                    what: target_filtered(snow_land()),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    creature_types: vec![CreatureType::Elemental],
                    keywords: vec![Keyword::Flying],
                    duration: Duration::Permanent,
                },
                &[Color::Blue],
            ),
            ..Default::default()
        }],
        ..creature("Balduvian Frostwaker", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 1)
    }
}

/// Phyrexian Ironfoot — {3} 3/4 snow Phyrexian Construct artifact creature. It
/// doesn't untap during your untap step. {1}{S}: untap it.
pub fn phyrexian_ironfoot() -> CardDefinition {
    use crate::card::Supertype;
    let mut untap_cost = cost(&[generic(1)]);
    untap_cost.symbols.push(crate::mana::ManaSymbol::Snow);
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: untap_cost,
            effect: Effect::Untap { what: Selector::This, up_to: None },
            ..Default::default()
        }],
        ..creature("Phyrexian Ironfoot", cost(&[generic(3)]), vec![CreatureType::Phyrexian, CreatureType::Construct], 3, 4)
    }
}

/// Storm Elemental — {5}{U} 3/4 flying Elemental. {U}, exile the top card of
/// your library: tap target creature with flying. {U}, exile the top card of
/// your library: if it's a snow land, +1/+1 until end of turn.
pub fn storm_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                exile_top_cost: 1,
                effect: Effect::Tap { what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                exile_top_cost: 1,
                effect: Effect::If {
                    cond: Predicate::SelectorExists(Selector::ExiledForCost),
                    then: Box::new(Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::ExiledForCost, filter: snow_land() },
                        then: Box::new(pump(Selector::This, 1, 1)),
                        else_: Box::new(Effect::Noop),
                    }),
                    else_: Box::new(Effect::Noop),
                },
                ..Default::default()
            },
        ],
        ..creature("Storm Elemental", cost(&[generic(5), u()]), vec![CreatureType::Elemental], 3, 4)
    }
}

/// Snowblind — {3}{G} Aura. Enchanted creature gets -X/-Y: X is the number of
/// snow lands defending player controls if it's attacking, else the number its
/// controller controls; Y is X or its toughness minus 1, whichever is smaller
/// (its printed toughness — 🟡 in INCOMPLETE_CARDS).
pub fn snowblind() -> CardDefinition {
    let host = || Selector::attached_to(Selector::This);
    let snow_of = |who: PlayerRef| Value::CountOf(Box::new(Selector::ControlledBy { who, filter: snow_land() }));
    let x = || Value::IfAtLeast {
        value: Box::new(Value::CountOf(Box::new(Selector::EachPermanent(R::AttachedToSource.and(R::IsAttacking))))),
        threshold: 1,
        then: Box::new(snow_of(PlayerRef::DefendingPlayerOf(Box::new(host())))),
        else_: Box::new(snow_of(PlayerRef::ControllerOf(Box::new(host())))),
    };
    let y = Value::Min(
        Box::new(x()),
        Box::new(Value::Diff(Box::new(Value::BaseToughnessOf(Box::new(host()))), Box::new(Value::ONE))),
    );
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets -X/-Y (snow lands).",
            effect: StaticEffect::PumpPTByValue {
                applies_to: host(),
                power: Value::Negate(Box::new(x())),
                toughness: Value::Negate(Box::new(Value::Max(Box::new(y), Box::new(Value::ZERO)))),
            },
        }],
        ..aura("Snowblind", cost(&[generic(3), g()]), None)
    }
}
