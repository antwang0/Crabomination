//! Commander: one-card gaps of the 2026-10-15 rescan (`01Kvb…` session).
//! Tests in `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, BattleSubtype, CounterType, EnchantmentSubtype, EquipBonus, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{etb, on_attack};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, x};
use crate::sets::tap_add_colorless;
use std::sync::Arc;

fn legend(name: &'static str, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn siege(name: &'static str, mana: ManaCost, defense: u32, on_enter: Effect, back: CardDefinition) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense,
        triggered_abilities: vec![etb(on_enter)],
        back_face: Some(Box::new(back)),
        ..Default::default()
    }
}

/// Invasion of Fiora — {4}{B}{B} Siege (4). Enters: choose one or both —
/// destroy all legendary creatures; destroy all nonlegendary creatures.
/// Back: Marchesa, Resolute Monarch — 3/6 menace deathtouch; attacking,
/// remove all counters from up to one target permanent; at your upkeep, if
/// you haven't been dealt combat damage since your last turn, draw and lose 1.
pub fn invasion_of_fiora() -> CardDefinition {
    let legendary = || R::HasSupertype(Supertype::Legendary);
    let upkeep_if_undamaged = Predicate::Not(Box::new(Predicate::CombatDamagedSinceYourTurn { who: PlayerRef::You }));
    let marchesa = CardDefinition {
        color_indicator: vec![Color::Black],
        keywords: vec![Keyword::Menace, Keyword::Deathtouch],
        triggered_abilities: vec![
            on_attack(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Permanent,
                effect: Box::new(Effect::RemoveAllCounters { what: Selector::Target(0) }),
            }),
            // CR 603.4 — an intervening "if": checked as it triggers and again
            // on resolution.
            TriggeredAbility {
                event: EventSpec {
                    filter: Some(upkeep_if_undamaged.clone()),
                    ..EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                },
                effect: Effect::If {
                    cond: upkeep_if_undamaged,
                    then: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..legend("Marchesa, Resolute Monarch", vec![CreatureType::Human, CreatureType::Noble], 3, 6)
    };
    siege(
        "Invasion of Fiora",
        cost(&[generic(4), b(), b()]),
        4,
        Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(legendary())) },
                Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(legendary())))) },
            ],
        },
        marchesa,
    )
}

/// Invasion of Segovia — {2}{U} Siege (4). Enters: two 1/1 blue Kraken
/// tokens with trample. Back: Caetus, Sea Tyrant of Segovia — 3/3;
/// noncreature spells you cast have convoke; at your end step, untap up to
/// four target creatures.
pub fn invasion_of_segovia() -> CardDefinition {
    let kraken = TokenDefinition {
        name: "Kraken".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kraken], ..Default::default() },
        ..Default::default()
    };
    let caetus = CardDefinition {
        color_indicator: vec![Color::Blue],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells you cast have convoke.",
            effect: StaticEffect::GrantConvokeToSpells { filter: R::Noncreature },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::ApplyToTargets {
                max_targets: 4,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
            },
        }],
        ..legend("Caetus, Sea Tyrant of Segovia", vec![CreatureType::Serpent], 3, 3)
    };
    siege(
        "Invasion of Segovia",
        cost(&[generic(2), u()]),
        4,
        Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(kraken) },
        caetus,
    )
}

/// Gathering Place — Land. {T}: Add {C}. {T}: Add {G} or {W}; activate only
/// if this land entered this turn or if you control a basic land.
pub fn gathering_place() -> CardDefinition {
    let gated = |c: Color| ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![c]) },
        condition: Some(Predicate::Any(vec![
            Predicate::EntityMatches { what: Selector::This, filter: R::EnteredThisTurn },
            Predicate::SelectorExists(Selector::EachPermanent(R::IsBasicLand.and(R::ControlledByYou))),
        ])),
        ..Default::default()
    };
    CardDefinition {
        name: "Gathering Place",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), gated(Color::Green), gated(Color::White)],
        ..Default::default()
    }
}

/// In Bolas's Clutches — {4}{U}{U} Legendary Aura. Enchant permanent; you
/// control enchanted permanent, and it's legendary.
pub fn in_bolass_clutches() -> CardDefinition {
    CardDefinition {
        name: "In Bolas's Clutches",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Permanent) },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted permanent is legendary.",
            effect: StaticEffect::AttachedIsLegendary,
        }],
        ..Default::default()
    }
}

/// Roku's Mastery — {X}{R}{R} instant. X damage to target creature; if X is
/// 4 or more, scry 2.
pub fn rokus_mastery() -> CardDefinition {
    CardDefinition {
        name: "Roku's Mastery",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::XFromCost },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(4)),
                then: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Pet Avengers — {3}{G} 4/4 reach. Power-up — {6}{G}: a +1/+1 counter and a
/// 3/2 white Hero token with vigilance.
pub fn pet_avengers() -> CardDefinition {
    let hero = TokenDefinition {
        name: "Hero".into(),
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hero], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Pet Avengers",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Cat, CreatureType::Dog, CreatureType::Bird, CreatureType::Frog, CreatureType::Hero],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), g()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(hero) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pelia, Immortal Innovator — {3}{U} legendary 3/3 Lanthanite Artificer.
/// Enters: return target creature an opponent controls to its owner's hand.
pub fn pelia_immortal_innovator() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(3), u()]),
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        })],
        ..legend("Pelia, Immortal Innovator", vec![CreatureType::Lanthanite, CreatureType::Artificer], 3, 3)
    }
}

/// Happy Hogan, Bodyguard — {5}{U} legendary 3/3 vigilance. Enters: the owner
/// of target creature an opponent controls puts it into their library second
/// from the top or on the bottom.
pub fn happy_hogan_bodyguard() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(5), u()]),
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::SecondFromTopOrBottom,
            },
        })],
        ..legend("Happy Hogan, Bodyguard", vec![CreatureType::Human, CreatureType::Advisor], 3, 3)
    }
}

/// Forebear's Blade — {3} Equipment. +3/+0, vigilance and trample; whenever
/// equipped creature dies, attach this to target creature you control. Equip {3}.
pub fn forebears_blade() -> CardDefinition {
    CardDefinition {
        name: "Forebear's Blade",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            keywords: vec![Keyword::Vigilance, Keyword::Trample],
            ..Default::default()
        }),
        // The death snapshot (`auras_at_death`) records Equipment riders too.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
        }],
        ..Default::default()
    }
}
