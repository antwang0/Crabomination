//! Commander: the 2026-10-20 (`01E4rMUL`) two-gap pair closers — each pair
//! of cards here completes an EDHREC partner-pair average deck.
//! Tests in `tests/recent_b/cmdr_01e4_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
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
