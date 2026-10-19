//! Commander: 2026-10-19 (`01GwrDKP`) closers, second batch — Wort's
//! Boggarts, Hei Bai's Shrines. Tests in `tests/recent_b/cmdr_01gw_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color, ManaCost};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CardType>, subtypes: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: types,
        subtypes: Subtypes { creature_types: subtypes, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Mouser Mark III — {1}{U/R} 2/3 artifact creature Robot. It can't attack
/// Nobody — {1}{U/R}{U/R} 3/2 artifact creature Human Hero. ETB, return up to
/// Rock Soldiers — {3}{R} 4/3 artifact creature Elemental Soldier. ETB,
/// Hawkeye's Shot — {4}{W} sorcery, {2} less if it targets a tapped creature.
/// Umara Wizard // Umara Skyfalls — {4}{U} 4/3 Merfolk Wizard; an instant,
/// sorcery or Wizard spell you cast gives it flying until end of turn. Back: a
/// Welcome to the Fold — {2}{U}{U} sorcery, madness {X}{U}{U}. Gain control of
/// target creature if its toughness is 2 or less (X or less if the madness
/// Boggart Mob — {3}{B} 5/5 Goblin Warrior. Champion a Goblin. Whenever a
/// Goblin you control deals combat damage to a player, you may create a 1/1
/// black Goblin Rogue.
pub fn boggart_mob() -> CardDefinition {
    let rogue = TokenDefinition {
        name: "Goblin Rogue".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Rogue], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Champion { filter: R::HasCreatureType(CreatureType::Goblin) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCreatureType(CreatureType::Goblin) },
                ),
                effect: Effect::MayDo {
                    description: "Create a 1/1 Goblin Rogue?".into(),
                    body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(rogue) }),
                },
            },
        ],
        ..creature(
            "Boggart Mob",
            cost(&[generic(3), b()]),
            vec![CardType::Creature],
            vec![CreatureType::Goblin, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// Boggart Birth Rite — {B} Kindred sorcery — Goblin. Return target Goblin
/// card from your graveyard to your hand.
pub fn boggart_birth_rite() -> CardDefinition {
    CardDefinition {
        name: "Boggart Birth Rite",
        cost: cost(&[b()]),
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        effect: Effect::Move {
            what: target_filtered(R::HasCreatureType(CreatureType::Goblin).and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

fn shrine() -> R {
    R::HasEnchantmentSubtype(EnchantmentSubtype::Shrine)
}

/// Hei Bai, Forest Guardian — {3}{G} 4/4 legendary Bear Spirit. ETB, reveal
/// until a Shrine card; you may put it onto the battlefield; then shuffle.
/// {W}{U}{B}{R}{G}, {T}: a 1/1 colorless Spirit (can't block or be blocked by
/// non-Spirits) per legendary enchantment you control.
pub fn hei_bai_forest_guardian() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        keywords: vec![
            Keyword::CantBlockMatching(Box::new(R::HasCreatureType(CreatureType::Spirit).negate())),
            Keyword::CantBeBlockedExceptBy(Box::new(R::HasCreatureType(CreatureType::Spirit))),
        ],
        ..Default::default()
    };
    CardDefinition {
        name: "Hei Bai, Forest Guardian",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear, CreatureType::Spirit], ..Default::default() },
        power: 4,
        toughness: 4,
        // The misses go to the bottom, then the whole library is shuffled.
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::RevealUntilOneToBattlefieldRestBottom { filter: shrine(), damage_controller: false, may: true },
            Effect::ShuffleLibrary { who: PlayerRef::You },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::count(Selector::EachPermanent(
                    R::Enchantment.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou),
                )),
                definition: Arc::new(spirit),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sanctum of All — {W}{U}{B}{R}{G} legendary enchantment — Shrine. At your
/// upkeep, you may search your library and/or graveyard for a Shrine card and
/// put it onto the battlefield. With six or more Shrines, another Shrine's
/// triggered abilities trigger an additional time.
pub fn sanctum_of_all() -> CardDefinition {
    CardDefinition {
        name: "Sanctum of All",
        cost: cost(&[w(), u(), b(), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Shrine], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Search for a Shrine?".into(),
                body: Box::new(Effect::SearchZones {
                    who: PlayerRef::You,
                    zones: vec![Zone::Library, Zone::Graveyard],
                    filter: shrine(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "If a triggered ability of another Shrine you control triggers while you control six or more Shrines, that ability triggers an additional time.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::ValueAtLeast(
                    Value::count(Selector::EachPermanent(shrine().and(R::ControlledByYou))),
                    Value::Const(6),
                ),
                inner: Box::new(StaticEffect::DoubleControllerTriggersMatching {
                    filter: shrine().and(R::OtherThanSource),
                    include_spells: false,
                }),
            },
        }],
        ..Default::default()
    }
}
