//! A cross-set wave (WOE / OTJ / BLB): a Bargain dig, a reanimator with
//! finality + Flashback, graveyard-recursion drain, an X-token maker, and a
//! Flashback prowess Otter. All ride existing primitives. Tests in
//! `crabomination/src/tests/recent152.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, Selector, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::mana::{Color, b, cost, generic, r, u, w, x};

/// Rowan's Grim Search — {2}{B} Instant. Bargain. If bargained, dig the top four
/// (keep up to two; modeled as Surveil 4). You draw two and lose 2 life.
pub fn rowans_grim_search() -> CardDefinition {
    CardDefinition {
        name: "Rowan's Grim Search",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Bargain],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::SpellWasBargained,
                then: Box::new(Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::Const(4),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Rite of the Moth — {1}{W}{B}{B} Sorcery. Return a target graveyard creature
/// to the battlefield with a finality counter. Flashback {3}{W}{W}{B}.
pub fn rite_of_the_moth() -> CardDefinition {
    CardDefinition {
        name: "Rite of the Moth",
        cost: cost(&[generic(1), w(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), w(), w(), b()]))],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Finality,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Hazel's Nocturne — {3}{B} Instant. Return up to two target creature cards
/// from your graveyard to your hand; each opponent loses 2 life and you gain 2.
pub fn hazels_nocturne() -> CardDefinition {
    CardDefinition {
        name: "Hazel's Nocturne",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::ReturnGraveyardCardsToHand {
                filter: R::Creature,
                max: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// 1/1 red Mercenary token with a sorcery-speed tap-pump ability.
fn mercenary_token() -> TokenDefinition {
    TokenDefinition {
        name: "Mercenary".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mercenary],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Form a Posse — {X}{R}{W} Sorcery. Create X 1/1 red Mercenary tokens.
pub fn form_a_posse() -> CardDefinition {
    CardDefinition {
        name: "Form a Posse",
        cost: cost(&[x(), r(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::XFromCost,
            definition: Box::new(mercenary_token()),
        },
        ..Default::default()
    }
}

/// Otterball Antics — {1}{U} Sorcery. Make a 1/1 Otter with prowess; if cast
/// from anywhere other than hand, it enters with a +1/+1 counter. Flashback {3}{U}.
pub fn otterball_antics() -> CardDefinition {
    let otter = TokenDefinition {
        name: "Otter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter],
            ..Default::default()
        },
        keywords: vec![Keyword::Prowess],
        ..Default::default()
    };
    CardDefinition {
        name: "Otterball Antics",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), u()]))],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(otter),
            },
            Effect::If {
                cond: Predicate::Not(Box::new(Predicate::CastFromHand)),
                then: Box::new(Effect::AddCounter {
                    what: Selector::take(
                        Selector::EachPermanent(
                            R::HasCreatureType(CreatureType::Otter).and(R::ControlledByYou),
                        ),
                        Value::ONE,
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}
