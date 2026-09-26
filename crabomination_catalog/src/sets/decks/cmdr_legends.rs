//! Most-built commanders missing from the catalog (COMMANDER_BACKLOG §1):
//! Aragorn, the Uniter; Zur the Enchanter; Flubs, the Fool; Galadriel,
//! Light of Valinor; Arcades, the Strategist; Tiamat. Each is built from
//! primitives other cards already exercise.
//!
//! - **Galadriel** — alliance's "one that hasn't been chosen this turn" is
//!   `Effect::ChooseUnchosenModeThisTurn` (Teval's Judgment).
//! - **Arcades** — Doran's `AssignsCombatDamageByToughness` grant, narrowed
//!   to defenders, plus the defender-can-attack static.
//! - **Tiamat** — `SearchUpToN`; the "different names" rider is moot in a
//!   singleton deck (CR 903.5b), which is where Tiamat is played.

use std::sync::Arc;

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn legend(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        ..Default::default()
    }
}

fn on_cast_color(color: Color, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::HasColor(color))),
        effect,
    }
}

/// Aragorn, the Uniter — {R}{G}{W}{U} 5/5. Whenever you cast a white spell,
/// create a 1/1 white Human Soldier; blue, scry 2; red, 3 damage to target
/// opponent; green, target creature gets +4/+4 until end of turn. A
/// multicolored spell fires each of its colors (CR 105.2).
pub fn aragorn_the_uniter() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            on_cast_color(
                Color::White,
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(soldier) },
            ),
            on_cast_color(Color::Blue, Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
            on_cast_color(
                Color::Red,
                Effect::DealDamage { to: target_filtered(R::Player.and(R::OpponentPlayer)), amount: Value::Const(3) },
            ),
            on_cast_color(
                Color::Green,
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(4),
                    toughness: Value::Const(4),
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
        ..legend(
            "Aragorn, the Uniter",
            cost(&[r(), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Noble],
            5,
            5,
        )
    }
}

/// Zur the Enchanter — {1}{W}{U}{B} 1/4 flier. Whenever Zur attacks, you may
/// search your library for an enchantment card with mana value 3 or less,
/// put it onto the battlefield, then shuffle.
pub fn zur_the_enchanter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Search {
            who: PlayerRef::You,
            filter: R::Enchantment.and(R::ManaValueAtMost(3)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        ..legend(
            "Zur the Enchanter",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            4,
        )
    }
}

/// Flubs, the Fool — {G}{U}{R} 0/5. You may play an additional land on each
/// of your turns. Whenever you play a land or cast a spell, draw a card if
/// you have no cards in hand; otherwise, discard a card.
pub fn flubs_the_fool() -> CardDefinition {
    let draw_or_discard = || Effect::If {
        cond: Predicate::ValueAtLeast(Value::Const(0), Value::HandSizeOf(PlayerRef::You)),
        then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        else_: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
                effect: draw_or_discard(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                effect: draw_or_discard(),
            },
        ],
        ..legend("Flubs, the Fool", cost(&[g(), u(), r()]), vec![CreatureType::Frog, CreatureType::Scout], 0, 5)
    }
}

/// Galadriel, Light of Valinor — {2}{G}{W}{U} 3/3. Alliance — whenever
/// another creature you control enters, choose one that hasn't been chosen
/// this turn: add {G}{G}{G}; a +1/+1 counter on each creature you control;
/// or scry 2, then draw a card.
pub fn galadriel_light_of_valinor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::ChooseUnchosenModeThisTurn {
                modes: vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Colors(vec![Color::Green, Color::Green, Color::Green]),
                    },
                    Effect::AddCounter {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::Seq(vec![
                        Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ]),
                ],
            },
        }],
        ..legend(
            "Galadriel, Light of Valinor",
            cost(&[generic(2), g(), w(), u()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            3,
            3,
        )
    }
}

/// Arcades, the Strategist — {1}{G}{W}{U} 3/5 flying, vigilance. Whenever a
/// creature you control with defender enters, draw a card. Each creature
/// you control with defender assigns combat damage equal to its toughness
/// (CR 510.1c) and can attack as though it didn't have defender.
pub fn arcades_the_strategist() -> CardDefinition {
    let defender = || R::Creature.and(R::HasKeyword(Keyword::Defender));
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: defender() }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        static_abilities: vec![
            StaticAbility {
                description: "Each creature you control with defender assigns combat damage equal to its toughness rather than its power.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(defender().and(R::ControlledByYou)),
                    keyword: Keyword::AssignsCombatDamageByToughness,
                },
            },
            StaticAbility {
                description: "Creatures you control with defender can attack as though they didn't have defender.",
                effect: StaticEffect::YourCreaturesCanAttackAsThoughNoDefender,
            },
        ],
        ..legend(
            "Arcades, the Strategist",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Elder, CreatureType::Dragon],
            3,
            5,
        )
    }
}

/// Tiamat — {2}{W}{U}{B}{R}{G} 7/7 Dragon God, flying. When it enters, if
/// you cast it, search your library for up to five Dragon cards not named
/// Tiamat with different names, put them into your hand, then shuffle.
pub fn tiamat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SourceWasCast,
            then: Box::new(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::HasCreatureType(CreatureType::Dragon).and(R::HasName("Tiamat".into()).negate()),
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(5),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..legend(
            "Tiamat",
            cost(&[generic(2), w(), u(), b(), r(), g()]),
            vec![CreatureType::Dragon, CreatureType::God],
            7,
            7,
        )
    }
}
