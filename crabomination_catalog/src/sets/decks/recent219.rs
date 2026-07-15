//! Foundations (FDN) gap batch 9 — reprint staples that each land a small
//! engine primitive: a Ferocious graveyard-return Phoenix, a draw-land, a
//! land-enchanting mana Aura, an incubation-counter Drake payoff, and a
//! divinity-counter Myojin. Tests in `tests/recent219.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, prowess};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, ManaPayload, PlayerRef, Predicate, Selector,
    StaticAbility, StaticEffect, Value, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{cost, b, g, generic, r, Color};

/// Flamewake Phoenix — {1}{R}{R} 2/2 Phoenix. Flying, haste, attacks each combat
/// if able. Ferocious — at combat, if you control a power-4+ creature, you may
/// pay {R} to return it from your graveyard.
pub fn flamewake_phoenix() -> CardDefinition {
    CardDefinition {
        name: "Flamewake Phoenix",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phoenix], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::MustAttack],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::FromYourGraveyard)
                .with_filter(Predicate::FerociousActive { who: PlayerRef::You }),
            effect: Effect::MayPay {
                description: "Pay {R} to return Flamewake Phoenix to the battlefield?".into(),
                mana_cost: cost(&[r()]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Cryptic Caves — Land. {T}: Add {C}. {1}, {T}, Sacrifice this land: Draw a
/// card. Activate only if you control five or more lands.
pub fn cryptic_caves() -> CardDefinition {
    CardDefinition {
        name: "Cryptic Caves",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Battlefield,
                        filter: R::Land.and(R::ControlledByYou),
                    },
                    n: Value::Const(5),
                }),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// New Horizons — {2}{G} Enchantment — Aura. Enchant land; when it enters, put a
/// +1/+1 counter on target creature you control. Enchanted land has "{T}: Add
/// two mana of any one color."
pub fn new_horizons() -> CardDefinition {
    CardDefinition {
        name: "New Horizons",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: R::Land },
        },
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Add two mana of any one color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyOneColor(Value::Const(2)),
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Drake Hatcher — {1}{U} 1/3 Human Wizard. Vigilance, prowess. Combat damage to
/// a player adds that many incubation counters; remove three to make a 2/2 blue
/// flying Drake.
pub fn drake_hatcher() -> CardDefinition {
    CardDefinition {
        name: "Drake Hatcher",
        cost: cost(&[generic(1), crate::mana::u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            prowess(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Incubation,
                    amount: Value::TriggerEventAmount,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Incubation, 3)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: drake_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn drake_token() -> TokenDefinition {
    TokenDefinition {
        name: "Drake".to_string(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Drake], ..Default::default() },
        ..Default::default()
    }
}

/// Myojin of Night's Reach — {5}{B}{B}{B} 5/2 Legendary Spirit. Enters with a
/// divinity counter if cast from hand; indestructible while it has one; remove
/// the divinity counter: each opponent discards their hand.
pub fn myojin_of_nights_reach() -> CardDefinition {
    CardDefinition {
        name: "Myojin of Night's Reach",
        cost: cost(&[generic(5), b(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 5,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::TriggerSourceEnteredByCast,
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Divinity,
                amount: Value::ONE,
            }),
            else_: Box::new(Effect::Noop),
        })],
        static_abilities: vec![StaticAbility {
            description: "Has indestructible as long as it has a divinity counter on it.",
            effect: StaticEffect::SelfHasKeywordWhileCountersAtLeast {
                kind: CounterType::Divinity,
                n: 1,
                keyword: Keyword::Indestructible,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Divinity, 1)),
            effect: Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachOpponent),
                body: Box::new(Effect::Discard {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::HandSizeOf(PlayerRef::Triggerer),
                    random: false,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
