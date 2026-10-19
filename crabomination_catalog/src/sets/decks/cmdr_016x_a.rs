//! Commander: 2026-10-19 (`016Xh16H`) — one- and two-gap closers. Tests in
//! `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{draw, embalm, etb, gain_life, target_filtered};
use crate::effect::{ActivationTrait, Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, generic, u, w};
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

// ── Valentina Allegra de Fontaine / Madame Hydra / Kang ─────────────────────

/// Cruel Alliance — {2}{B} sorcery, teamwork 2. Exile target creature with
/// mana value 3 or less; cast using teamwork, any creature instead and you
/// gain 3 life. Teamwork is announced before targets (CR 601.2b-c), so the
/// kicker-aware slot filter picks the branch that will resolve.
pub fn cruel_alliance() -> CardDefinition {
    CardDefinition {
        name: "Cruel Alliance",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        kicker_action_cost: Some(AdditionalCastCost::TapTotalPower {
            filter: R::Creature.and(R::ControlledByYou),
            power: 2,
        }),
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Creature) },
                gain_life(3),
            ])),
            else_: Box::new(Effect::Exile {
                what: target_filtered(R::Creature.and(R::ManaValueAtMost(3))),
            }),
        },
        ..Default::default()
    }
}

// ── Temmet, Vizier of Naktamun ──────────────────────────────────────────────

/// Glyph Keeper — {3}{U}{U} 5/3 flying Sphinx. Counters the first spell or
/// ability to target it each turn; embalm {5}{U}{U}.
pub fn glyph_keeper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CounterFirstTargetingEachTurn],
        activated_abilities: vec![embalm(cost(&[generic(5), u(), u()]))],
        ..creature("Glyph Keeper", cost(&[generic(3), u(), u()]), vec![CreatureType::Sphinx], 5, 3)
    }
}

/// Vizier of the Anointed — {3}{U} 2/4 Human Cleric. Its ETB may entomb a
/// creature card with eternalize or embalm; activating one of those draws.
pub fn vizier_of_the_anointed() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Search for a creature card with eternalize or embalm?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Creature.and(R::HasEmbalmOrEternalize),
                    to: ZoneDest::Graveyard,
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::AbilityActivatedWith(ActivationTrait::Embalm),
                    EventScope::YourControl,
                ),
                effect: draw(1),
            },
        ],
        ..creature(
            "Vizier of the Anointed",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            4,
        )
    }
}

// ── Splinter & Leo, Father & Son ────────────────────────────────────────────

/// Splinter & Leo, Father & Son — {2}{W} 2/2 legendary Mutant Ninja Rat
/// Turtle. ETB, choose one or both, each mode targeting a different player:
/// that player creates a 2/2 red Mutant; +1/+1 counter on each other creature
/// that player controls (CR 700.2 — the Shadrix shape, one player slot a mode).
pub fn splinter_leo_father_son() -> CardDefinition {
    let mutant = TokenDefinition {
        name: "Mutant".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mutant], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::ChooseN { up_to: None,
            picks: vec![1, 0],
            modes: vec![
                Effect::CreateToken { who: PlayerRef::Target(0), count: Value::ONE, definition: Arc::new(mutant) },
                Effect::AddCounter {
                    what: Selector::ControlledBy {
                        who: PlayerRef::Target(0),
                        filter: R::Creature.and(R::OtherThanSource),
                    },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ],
        })],
        ..creature(
            "Splinter & Leo, Father & Son",
            cost(&[generic(2), w()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Rat, CreatureType::Turtle],
            2,
            2,
        )
    }
}

// ── Tomik, Wielder of Law / Djeru, With Eyes Open / Mila ────────────────────

/// Djeru, With Eyes Open — {3}{W}{W} 4/3 legendary Human Warrior, vigilance.
/// ETB may tutor a planeswalker card; damage to your planeswalkers is
/// reduced by 1 (CR 615).
pub fn djeru_with_eyes_open() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search your library for a planeswalker card?".into(),
            body: Box::new(Effect::Search { who: PlayerRef::You, filter: R::Planeswalker, to: ZoneDest::Hand(PlayerRef::You) }),
        })],
        static_abilities: vec![StaticAbility {
            description: "If a source would deal damage to a planeswalker you control, prevent 1 of that damage.",
            effect: StaticEffect::ReduceDamageToYourMatchingCreaturesBy { filter: R::Planeswalker, amount: 1 },
        }],
        ..creature(
            "Djeru, With Eyes Open",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            4,
            3,
        )
    }
}

/// Elspeth's Talent — {2}{W}{W} Aura, enchant planeswalker. It has "[+1]:
/// three 1/1 white Soldiers"; activating one of its loyalty abilities pumps
/// your team +2/+2 and vigilance until end of turn.
pub fn elspeths_talent() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Elspeth's Talent",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Planeswalker) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted planeswalker has \"[+1]: Create three 1/1 white Soldier creature tokens.\"",
            effect: StaticEffect::AttachedPlaneswalkerHasLoyaltyAbilities {
                abilities: vec![LoyaltyAbility {
                    loyalty_cost: 1,
                    effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(3), definition: Arc::new(soldier) },
                    ..Default::default()
                }],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::AbilityActivatedWith(ActivationTrait::Loyalty),
                EventScope::EnchantedBySource,
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: team(), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: team(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            ]),
        }],
        ..Default::default()
    }
}
