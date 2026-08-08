//! Return to Ravnica (RTR) gap wave 7: the remaining common/uncommon spells —
//! exile+populate, X-burn+discard, loot/bounce/detain riders, land destruction,
//! graveyard recursion, and an Overload debuff. Tests in `classic_sets/rtr`.

use crate::card::{
    AlternativeCost, CardDefinition, CardType, CreatureType, Effect, Keyword,
    SelectionRequirement as R, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

/// Trostani's Judgment — {5}{W} Instant. Exile target creature, then populate.
pub fn trostanis_judgment() -> CardDefinition {
    CardDefinition {
        name: "Trostani's Judgment",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(R::Creature),
            },
            Effect::Populate {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Rakdos's Return — {X}{B}{R} Sorcery. Deals X damage to target opponent or
/// planeswalker; that player (or the planeswalker's controller) discards X cards.
pub fn rakdos_return() -> CardDefinition {
    CardDefinition {
        name: "Rakdos's Return",
        cost: cost(&[x(), b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Player.and(R::ControlledByOpponent).or(R::Planeswalker)),
                amount: Value::XFromCost,
            },
            Effect::Discard {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::XFromCost,
                random: false,
            },
        ]),
        ..Default::default()
    }
}

/// Thoughtflare — {3}{U}{R} Instant. Draw four cards, then discard two cards.
pub fn thoughtflare() -> CardDefinition {
    CardDefinition {
        name: "Thoughtflare",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(4),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(2),
                random: false,
            },
        ]),
        ..Default::default()
    }
}

/// Dramatic Rescue — {W}{U} Instant. Return target creature to its owner's hand;
/// you gain 2 life.
pub fn dramatic_rescue() -> CardDefinition {
    CardDefinition {
        name: "Dramatic Rescue",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Search Warrant — {W}{U} Sorcery. Target player reveals their hand; you gain
/// life equal to the number of cards in that player's hand.
pub fn search_warrant() -> CardDefinition {
    CardDefinition {
        name: "Search Warrant",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::HandSizeOf(PlayerRef::Target(0)),
        },
        ..Default::default()
    }
}

/// Survey the Wreckage — {4}{R} Sorcery. Destroy target land; create a 1/1 red
/// Goblin creature token.
pub fn survey_the_wreckage() -> CardDefinition {
    CardDefinition {
        name: "Survey the Wreckage",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Land),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Goblin".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Goblin],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Rites of Reaping — {4}{B}{G} Sorcery. Target creature gets +3/+3 until end of
/// turn; another target creature gets -3/-3 until end of turn.
pub fn rites_of_reaping() -> CardDefinition {
    CardDefinition {
        name: "Rites of Reaping",
        cost: cost(&[generic(4), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature,
                },
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature,
                },
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Inaction Injunction — {1}{U} Sorcery. Detain target creature an opponent
/// controls; draw a card.
pub fn inaction_injunction() -> CardDefinition {
    CardDefinition {
        name: "Inaction Injunction",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Detain {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Treasured Find — {B}{G} Sorcery. Return target card from your graveyard to
/// your hand. Exile Treasured Find.
pub fn treasured_find() -> CardDefinition {
    CardDefinition {
        name: "Treasured Find",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::Any.and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        exile_on_resolve: true,
        ..Default::default()
    }
}

/// Chemister's Trick — {U}{R} Instant. Target creature you don't control gets
/// -2/-0 until end of turn and attacks this turn if able. Overload {3}{U}{R}.
pub fn chemisters_trick() -> CardDefinition {
    let base = |sel: Selector| {
        Effect::Seq(vec![
            Effect::PumpPT {
                what: sel.clone(),
                power: Value::Const(-2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: sel,
                keyword: Keyword::MustAttack,
                duration: Duration::EndOfTurn,
            },
        ])
    };
    CardDefinition {
        name: "Chemister's Trick",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Instant],
        effect: base(target_filtered(R::Creature.and(R::ControlledByOpponent))),
        alternative_cost: Some(AlternativeCost {
            awaken: false,
            mana_cost: cost(&[generic(3), u(), r()]),
            effect_override: Some(base(Selector::EachPermanent(
                R::Creature.and(R::ControlledByOpponent),
            ))),
            ..Default::default()
        }),
        ..Default::default()
    }
}
