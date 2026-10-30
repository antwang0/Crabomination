//! Commander: 2026-10-18 (`019Y6J4R`) closers, fifteenth batch. Tests in
//! `tests/recent_b/cmdr_019y_o.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, LoyaltyAbility, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, generic, hybrid, r, u};

/// "You may discard a card. If you do, [then]" — offered only with a card in
/// hand, so the rider can't ride an empty discard.
fn may_discard_then(description: &str, then: Effect) -> Effect {
    Effect::If {
        cond: Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::You), Value::ONE),
        then: Box::new(Effect::MayDo {
            description: description.into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                then,
            ])),
        }),
        else_: Box::new(Effect::Noop),
    }
}

/// Teferi's Talent — {3}{U}{U} Aura, enchant planeswalker. Enchanted
/// planeswalker has "[−12]: You get an emblem with 'You may activate loyalty
/// abilities of planeswalkers you control on any player's turn any time you
/// could cast an instant.'" Whenever you draw a card, put a loyalty counter on
/// enchanted planeswalker.
pub fn teferis_talent() -> CardDefinition {
    CardDefinition {
        name: "Teferi's Talent",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Planeswalker) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted planeswalker has \"[−12]: You get an emblem with 'You may activate loyalty abilities of planeswalkers you control on any player's turn any time you could cast an instant.'\"",
            effect: StaticEffect::AttachedPlaneswalkerHasLoyaltyAbilities {
                abilities: vec![LoyaltyAbility {
                    loyalty_cost: -12,
                    effect: Effect::CreateEmblem {
                        who: PlayerRef::You,
                        name: "Teferi's Talent".into(),
                        triggered: vec![],
                        statics: vec![StaticAbility {
                            description: "You may activate loyalty abilities of planeswalkers you control on any player's turn any time you could cast an instant.",
                            effect: StaticEffect::LoyaltyAbilitiesAtInstantSpeed,
                        }],
                    },
                    ..Default::default()
                }],
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::attached_to(Selector::This),
                kind: CounterType::Loyalty,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// The Clone Saga — {3}{U} Saga. I — surveil 3. II — when you next cast a
/// creature spell this turn, copy it, except the copy isn't legendary. III —
/// choose a card name; whenever a creature with the chosen name deals combat
/// damage to a player this turn, draw a card.
pub fn the_clone_saga() -> CardDefinition {
    CardDefinition {
        name: "The Clone Saga",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::Surveil { who: PlayerRef::You, amount: Value::Const(3) }),
            (
                2,
                Effect::OnYourNextSpellMatchingThisTurn {
                    filter: R::Creature,
                    body: Box::new(Effect::CopySpellNonLegendary { what: Selector::TriggerSource }),
                },
            ),
            (
                3,
                Effect::NameCardThenNamedCreatureCombatDamageThisTurn {
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Scarlet Spider, Kaine — {B}{R} legendary 2/1 menace Spider Human Hero. ETB:
/// you may discard a card; if you do, a +1/+1 counter on him. Mayhem {B/R}.
pub fn scarlet_spider_kaine() -> CardDefinition {
    CardDefinition {
        name: "Scarlet Spider, Kaine",
        cost: cost(&[b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace, Keyword::Mayhem(cost(&[hybrid(Color::Black, Color::Red)]))],
        triggered_abilities: vec![etb(may_discard_then(
            "Discard a card to put a +1/+1 counter on Scarlet Spider?",
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        ))],
        ..Default::default()
    }
}

/// Shriek, Treblemaker — {2}{B/R} legendary 2/3 Mutant Villain. Beginning of
/// your first main phase: you may discard a card; when you do, target
/// creature can't block this turn. Sonic Blast — whenever a creature an
/// opponent controls dies, Shriek deals 1 damage to that player.
pub fn shriek_treblemaker() -> CardDefinition {
    CardDefinition {
        name: "Shriek, Treblemaker",
        cost: cost(&[generic(2), hybrid(Color::Black, Color::Red)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mutant, CreatureType::Villain], ..Default::default() },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
                effect: may_discard_then(
                    "Discard a card so a creature can't block this turn?",
                    Effect::ReflexiveTrigger {
                        body: Box::new(Effect::GrantKeyword {
                            what: target_filtered(R::Creature),
                            keyword: Keyword::CantBlock,
                            duration: Duration::EndOfTurn,
                        }),
                    },
                ),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::TriggerEventPlayer), amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}
