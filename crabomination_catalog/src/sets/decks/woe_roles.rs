//! Wilds of Eldraine (WOE) Role Aura tokens (CR 113 / 208). Shared by every
//! `recent12x`/`recent13x` WOE wave so the six printed Roles have one
//! authoritative definition each. A creature holds at most one Role you own;
//! the engine drops the older duplicate as a state-based action.

use crate::card::{
    CardType, CounterType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec,
    Keyword, Predicate, Selector, Subtypes, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::{Effect, PlayerRef};
use crate::mana::{Color, cost, generic};

fn role_aura(name: &'static str, color: Color, bonus: EquipBonus) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        card_types: vec![CardType::Enchantment],
        colors: vec![color],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Role],
            ..Default::default()
        },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// Cursed Role: enchanted creature is 1/1.
pub(crate) fn cursed_role() -> TokenDefinition {
    role_aura(
        "Cursed",
        Color::Black,
        EquipBonus {
            set_base_pt: Some((1, 1)),
            ..Default::default()
        },
    )
}

/// Monster Role: enchanted creature gets +1/+1 and has trample.
pub(crate) fn monster_role() -> TokenDefinition {
    role_aura(
        "Monster",
        Color::Red,
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        },
    )
}

/// Royal Role: enchanted creature gets +1/+1 and has ward {1}.
pub(crate) fn royal_role() -> TokenDefinition {
    role_aura(
        "Royal",
        Color::White,
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
            ..Default::default()
        },
    )
}

/// Sorcerer Role: enchanted creature gets +1/+1 and has "Whenever this
/// creature attacks, scry 1."
pub(crate) fn sorcerer_role() -> TokenDefinition {
    role_aura(
        "Sorcerer",
        Color::White,
        EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        },
    )
}

/// Young Hero Role: enchanted creature has "Whenever this creature attacks, if
/// its toughness is 3 or less, put a +1/+1 counter on it."
pub(crate) fn young_hero_role() -> TokenDefinition {
    role_aura(
        "Young Hero",
        Color::White,
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                // CR 603.4 intervening-if — the +1/+1 only lands while the
                // attacker's toughness is 3 or less (TriggerSource = attacker).
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                    Predicate::ValueAtMost(
                        Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                        Value::Const(3),
                    ),
                ),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        },
    )
}

/// Wicked Role: enchanted creature gets +1/+1 and has "When this creature dies,
/// each opponent loses 1 life." Modeled as a SelfSource death trigger on the
/// Aura token (it hits the graveyard with the creature it enchants).
pub(crate) fn wicked_role() -> TokenDefinition {
    let mut t = role_aura(
        "Wicked",
        Color::Black,
        EquipBonus {
            power: 1,
            toughness: 1,
            ..Default::default()
        },
    );
    t.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
        effect: Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
        },
    }];
    t
}
