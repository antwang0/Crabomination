//! Commander: the 2026-10-20 (`01E4rMUL`) two-gap pair closers — each pair
//! of cards here completes an EDHREC partner-pair average deck.
//! Tests in `tests/recent_b/cmdr_01e4_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
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
