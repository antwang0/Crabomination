//! Commander: the 2026-10-20 (`01DDeUtc`) closers — Sygg, River Guide's
//! Merfolk. Tests in `tests/recent_b/cmdr_01dd_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, Predicate, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, u, w};

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

fn merfolk() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Merfolk)).and(R::ControlledByYou)
}

/// Harpoon Sniper — {2}{W} 2/2 Merfolk Archer. {W}, {T}: X damage to target
/// attacking or blocking creature, X = Merfolk you control.
pub fn harpoon_sniper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::CountOf(Box::new(Selector::EachPermanent(merfolk()))),
            },
            ..Default::default()
        }],
        ..creature("Harpoon Sniper", cost(&[generic(2), w()]), vec![CreatureType::Merfolk, CreatureType::Archer], 2, 2)
    }
}

/// Judge of Currents — {1}{W} 1/1 Merfolk Wizard. Whenever a Merfolk you
/// control becomes tapped, you may gain 1 life.
pub fn judge_of_currents() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: merfolk() }),
            effect: Effect::MayDo {
                description: "Gain 1 life?".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..creature("Judge of Currents", cost(&[generic(1), w()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 1, 1)
    }
}

/// "At the beginning of your first main phase, you may pay {pip}. If you do,
/// transform Sygg."
fn sygg_flip(pip: ManaCost) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
        effect: Effect::MayPay {
            description: "Pay to transform Sygg?".into(),
            mana_cost: pip,
            body: Box::new(Effect::Transform { what: Selector::This }),
            else_: None,
        },
    }
}

/// Sygg, Wanderwine Wisdom // Sygg, Wanderbrine Shield — {1}{U} 2/2
/// legendary Merfolk; can't be blocked on both faces. Entering or
/// transforming into the front: target creature gains "whenever this deals
/// combat damage to a player or planeswalker, draw a card" until end of turn.
/// Transforming into the back: target creature you control gains protection
/// from each color until your next turn. Each face transforms at your first
/// main phase for {W} / {U}.
pub fn sygg_wanderwine_wisdom() -> CardDefinition {
    let draw_on_hit = |kind: EventKind| Effect::GrantTriggeredAbility {
        what: target_filtered(R::Creature),
        trigger: Box::new(TriggeredAbility {
            event: EventSpec::new(kind, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }),
        duration: Duration::EndOfTurn,
    };
    let front_text = Effect::Seq(vec![
        draw_on_hit(EventKind::DealsCombatDamageToPlayer),
        draw_on_hit(EventKind::DealsCombatDamageToPlaneswalker),
    ]);
    let back = CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource),
                effect: Effect::GrantKeywords {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keywords: Color::ALL.iter().map(|c| Keyword::Protection(*c)).collect(),
                    duration: Duration::UntilNextTurn,
                },
            },
            sygg_flip(cost(&[u()])),
        ],
        ..sygg_face("Sygg, Wanderbrine Shield", CreatureType::Rogue)
    };
    CardDefinition {
        cost: cost(&[generic(1), u()]),
        triggered_abilities: vec![
            etb(front_text.clone()),
            TriggeredAbility { event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource), effect: front_text },
            sygg_flip(cost(&[w()])),
        ],
        back_face: Some(Box::new(back)),
        ..sygg_face("Sygg, Wanderwine Wisdom", CreatureType::Wizard)
    }
}

fn sygg_face(name: &'static str, class: CreatureType) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Unblockable],
        ..creature(name, ManaCost::default(), vec![CreatureType::Merfolk, class], 2, 2)
    }
}
