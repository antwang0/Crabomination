//! Guildpact (GPT) third gap wave: two more haunt payoffs, a Leyline, and a
//! Bloodthirst attacker-anthem. Tests in `classic_sets/gpt`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{bloodthirst, etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, OpeningHandEffect, PlayerRef, Selector};
use crate::mana::{b, cost, generic, r, Color};

/// 1/1 white Spirit token with flying.
fn spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    }
}

/// Graven Dominator — {4}{W}{W} 4/4 Gargoyle with flying and haunt. When it
/// enters or the creature it haunts dies, each other creature has base power and
/// toughness 1/1 until end of turn.
pub fn graven_dominator() -> CardDefinition {
    let flatten = Effect::SetBasePT {
        what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
        power: Value::ONE,
        toughness: Value::ONE,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Graven Dominator",
        cost: cost(&[generic(4), crate::mana::w(), crate::mana::w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gargoyle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(flatten.clone()),
            on_dies(Effect::HauntCreature { body: Box::new(flatten) }),
        ],
        ..Default::default()
    }
}

/// Seize the Soul — {2}{B}{B} Instant with haunt. Destroy target nonwhite,
/// nonblack creature and create a 1/1 white Spirit with flying; the haunt body
/// repeats when the haunted creature dies.
pub fn seize_the_soul() -> CardDefinition {
    let body = Effect::Seq(vec![
        Effect::Destroy {
            what: target_filtered(
                R::Creature
                    .and(R::Not(Box::new(R::HasColor(Color::White))))
                    .and(R::Not(Box::new(R::HasColor(Color::Black)))),
            ),
        },
        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: spirit_token() },
    ]);
    CardDefinition {
        name: "Seize the Soul",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            body.clone(),
            Effect::HauntCreature { body: Box::new(body) },
        ]),
        ..Default::default()
    }
}

/// Leyline of Lightning — {2}{R}{R} Enchantment. If in your opening hand, you
/// may begin the game with it in play. Whenever you cast a spell, you may pay
/// {1}. If you do, deal 1 damage to target player or planeswalker.
pub fn leyline_of_lightning() -> CardDefinition {
    CardDefinition {
        name: "Leyline of Lightning",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1}: this deals 1 damage to target player or planeswalker".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::ONE,
                }),
                else_: None,
            },
        }],
        opening_hand: Some(OpeningHandEffect::StartInPlay { tapped: false, extra: Effect::Noop }),
        ..Default::default()
    }
}

/// Rabble-Rouser — {3}{R} 1/1 Goblin Shaman with bloodthirst 1. {R}, {T}:
/// Attacking creatures get +X/+0 until end of turn, where X is this creature's
/// power.
pub fn rabble_rouser() -> CardDefinition {
    CardDefinition {
        name: "Rabble-Rouser",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Shaman], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![bloodthirst(1)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
