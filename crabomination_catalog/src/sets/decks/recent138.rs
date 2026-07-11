//! A Wilds of Eldraine (WOE) wave completing cards previously deferred in
//! TODO.md: an enchanted-creatures anthem, a conditional enchantment-destroy,
//! a modal combat trick, a not-a-token self-copy, and a haste static gated on
//! an owned exiled Adventure (the new `Predicate::OwnExiledAdventureCard`).
//! Tests in `crabomination/src/tests/recent138.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Value,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect, ZoneRef};
use crate::mana::{cost, g, generic, w};

// ── White ─────────────────────────────────────────────────────────────────────

/// A Tale for the Ages — {1}{W} Enchantment. Enchanted creatures you control
/// get +2/+2 (in WOE the Roles are Auras, so this anthems your Role-bearers).
pub fn a_tale_for_the_ages() -> CardDefinition {
    CardDefinition {
        name: "A Tale for the Ages",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creatures you control get +2/+2.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::IsEnchanted),
                power: 2,
                toughness: 2,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Break the Spell — {W} Instant. Destroy target enchantment. If a permanent
/// you controlled or a token was destroyed this way, draw a card.
pub fn break_the_spell() -> CardDefinition {
    CardDefinition {
        name: "Break the Spell",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: R::ControlledByYou.or(R::IsToken),
            },
            then: Box::new(Effect::Seq(vec![
                Effect::Destroy { what: Selector::Target(0) },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ])),
            else_: Box::new(Effect::Destroy { what: target_filtered(R::Enchantment) }),
        },
        ..Default::default()
    }
}

/// Moment of Valor — {2}{W} Instant. Choose one — untap target creature; it
/// gets +1/+0 and gains indestructible until end of turn. Or: destroy target
/// creature with power 4 or greater.
pub fn moment_of_valor() -> CardDefinition {
    CardDefinition {
        name: "Moment of Valor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Untap { what: target_filtered(R::Creature), up_to: None },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
            Effect::Destroy { what: target_filtered(R::Creature.and(R::PowerAtLeast(4))) },
        ]),
        ..Default::default()
    }
}

// ── Green ─────────────────────────────────────────────────────────────────────

/// Gruff Triplets — {3}{G}{G}{G} 3/3 Satyr Warrior with trample. ETB, if it
/// isn't a token, create two token copies of it. When it dies, put +1/+1
/// counters equal to its power on each creature you control named Gruff Triplets.
pub fn gruff_triplets() -> CardDefinition {
    CardDefinition {
        name: "Gruff Triplets",
        cost: cost(&[generic(3), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::EntityMatches { what: Selector::This, filter: R::NotToken },
                then: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                }),
                else_: Box::new(Effect::Noop),
            }),
            on_dies(Effect::AddCounter {
                what: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasName("Gruff Triplets".into())),
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::PowerOf(Box::new(Selector::This)),
            }),
        ],
        ..Default::default()
    }
}

/// Howling Galefang — {2}{G}{G} 4/4 Beast with vigilance. Has haste as long as
/// you own a card in exile that has an Adventure.
pub fn howling_galefang() -> CardDefinition {
    CardDefinition {
        name: "Howling Galefang",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Howling Galefang has haste as long as you own a card in exile that has an Adventure.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Haste,
                condition: Predicate::OwnExiledAdventureCard,
            },
        }],
        ..Default::default()
    }
}
