//! Commander: the 2026-10-13 rescan's gaps for Kaima, the Fractured Calm and
//! Verrak, Warped Sengir. Tests in `tests/recent_b/cmdr_edhrec48.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, w, ManaCost};
use crabomination_base::tokens::treasure_token;
use crabomination_base::turn_step::TurnStep;
use std::sync::Arc;

fn aura(name: &'static str, mana: ManaCost, bonus: EquipBonus, triggers: Vec<TriggeredAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        triggered_abilities: triggers,
        ..Default::default()
    }
}

/// "Return this card to its owner's hand" — only from a graveyard.
fn back_to_hand() -> Effect {
    Effect::If {
        cond: Predicate::EntityMatches { what: Selector::This, filter: R::InGraveyard },
        then: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
        else_: Box::new(Effect::Noop),
    }
}

fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl), effect }
}

// ── Kaima, the Fractured Calm ────────────────────────────────────────────────

/// Draconic Destiny — {1}{R}{R} Aura. Enchanted creature gets +1/+1, flying,
/// haste and "{1}: +1/+0 this turn", and is a Dragon too. When it dies, this
/// card returns to its owner's hand.
pub fn draconic_destiny() -> CardDefinition {
    aura(
        "Draconic Destiny",
        cost(&[generic(1), r(), r()]),
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying, Keyword::Haste],
            add_creature_types: vec![CreatureType::Dragon],
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        },
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: back_to_hand(),
        }],
    )
}

/// Fruit of the First Tree — {3}{G} Aura. When the enchanted creature dies,
/// you gain X life and draw X cards, X its toughness.
pub fn fruit_of_the_first_tree() -> CardDefinition {
    let x = || Value::ToughnessOf(Box::new(Selector::TriggerSource));
    aura(
        "Fruit of the First Tree",
        cost(&[generic(3), g()]),
        EquipBonus::default(),
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: x() },
                Effect::Draw { who: Selector::You, amount: x() },
            ]),
        }],
    )
}

/// Undying Rage — {2}{R} Aura. Enchanted creature gets +2/+2 and can't block.
/// Put into a graveyard from the battlefield, it returns to its owner's hand.
pub fn undying_rage() -> CardDefinition {
    aura(
        "Undying Rage",
        cost(&[generic(2), r()]),
        EquipBonus { power: 2, toughness: 2, keywords: vec![Keyword::CantBlock], ..Default::default() },
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: back_to_hand(),
        }],
    )
}

// ── Verrak, Warped Sengir ────────────────────────────────────────────────────

/// Forsworn Paladin — {B} 1/1 Human Knight, menace. {1}{B}, {T}, pay 1 life:
/// a Treasure. {2}{B}: target creature gets +2/+0 this turn, and deathtouch if
/// Treasure mana paid for it.
pub fn forsworn_paladin() -> CardDefinition {
    let target = || target_filtered(R::Creature);
    CardDefinition {
        name: "Forsworn Paladin",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Knight], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                tap_cost: true,
                life_cost: 1,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(treasure_token()),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                effect: Effect::Seq(vec![
                    Effect::PumpPT { what: target(), power: Value::Const(2), toughness: Value::ZERO, duration: Duration::EndOfTurn },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(Value::TreasureManaSpentToActivate, Value::ONE),
                        then: Box::new(Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Deathtouch,
                            duration: Duration::EndOfTurn,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Arguel's Blood Fast // Temple of Aclazotz — {1}{B} legendary enchantment.
/// {1}{B}, pay 2 life: draw. Upkeep at 5 life or less: you may transform it.
/// Temple (legendary land): {T}: {B}; {T}, sacrifice a creature: gain its
/// toughness in life.
pub fn arguels_blood_fast() -> CardDefinition {
    let temple = CardDefinition {
        name: "Temple of Aclazotz",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![crate::mana::Color::Black]) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((R::Creature.and(R::ControlledByYou), 1)),
                effect: Effect::GainLife { who: Selector::You, amount: Value::SacrificedToughness },
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    CardDefinition {
        name: "Arguel's Blood Fast",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            life_cost: 2,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::PlayerLifeAtMost { who: PlayerRef::You, life: 5 }),
            effect: Effect::MayDo {
                description: "Transform Arguel's Blood Fast?".into(),
                body: Box::new(Effect::Transform { what: Selector::This }),
            },
        }],
        back_face: Some(Box::new(temple)),
        ..Default::default()
    }
}

/// Axis of Mortality — {4}{W}{W} enchantment. Your upkeep: you may have two
/// target players exchange life totals.
pub fn axis_of_mortality() -> CardDefinition {
    CardDefinition {
        name: "Axis of Mortality",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![your_upkeep(Effect::MayDo {
            description: "Have two target players exchange life totals?".into(),
            body: Box::new(Effect::ExchangeLifeTotals {
                a: Selector::TargetFiltered { slot: 0, filter: R::Player },
                b: Selector::TargetFiltered { slot: 1, filter: R::Player },
            }),
        })],
        ..Default::default()
    }
}

