//! OTJ outlaw batch — on existing primitives: Full Steam Ahead (team pump),
//! Hellspur Posse Boss (outlaw-haste lord + Mercenary tokens), Kraum (Flurry
//! payoff), and At Knifepoint (outlaw first strike + crime tokens). Tests in
//! `crabomination/src/tests/recent184.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, flurry, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{Color, b, cost, g, generic, r, u};

/// A 1/1 red Mercenary token: {T}: target creature you control gets +1/+0 until
/// end of turn (sorcery speed).
fn mercenary_token() -> TokenDefinition {
    TokenDefinition {
        name: "Mercenary".to_string(),
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mercenary],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
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

fn other_outlaws() -> Selector {
    Selector::EachPermanent(R::IsOutlaw.and(R::ControlledByYou).and(R::OtherThanSource))
}

/// Full Steam Ahead — {3}{G}{G} Sorcery. Until end of turn, each creature you
/// control gets +2/+2, gains trample, and can't be blocked by more than one
/// creature.
pub fn full_steam_ahead() -> CardDefinition {
    let team = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Full Steam Ahead",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: team(),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: team(),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: team(),
                keyword: Keyword::CantBeBlockedByMoreThanOne,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Hellspur Posse Boss — {2}{R}{R} 2/4 Lizard Rogue. Other outlaws you control
/// have haste. When it enters, create two 1/1 red Mercenary tokens.
pub fn hellspur_posse_boss() -> CardDefinition {
    CardDefinition {
        name: "Hellspur Posse Boss",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Other outlaws you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: other_outlaws(),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: Box::new(mercenary_token()),
        })],
        ..Default::default()
    }
}

/// Kraum, Violent Cacophony — {2}{U}{R} 2/3 legendary Zombie Horror with flying.
/// Whenever you cast your second spell each turn, put a +1/+1 counter on it and
/// draw a card.
pub fn kraum_violent_cacophony() -> CardDefinition {
    CardDefinition {
        name: "Kraum, Violent Cacophony",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![flurry(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]))],
        ..Default::default()
    }
}

/// At Knifepoint — {1}{B}{R} Enchantment. Outlaws you control have first strike.
/// Whenever you commit a crime, create a 1/1 red Mercenary token (once each turn).
/// (First strike modeled as always-on rather than "during your turn.")
pub fn at_knifepoint() -> CardDefinition {
    CardDefinition {
        name: "At Knifepoint",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Outlaws you control have first strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::IsOutlaw.and(R::ControlledByYou)),
                keyword: Keyword::FirstStrike,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl)
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(mercenary_token()),
            },
        }],
        ..Default::default()
    }
}
