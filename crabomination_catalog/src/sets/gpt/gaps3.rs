//! Guildpact (GPT) third gap wave: two more haunt payoffs, a Leyline, and a
//! Bloodthirst attacker-anthem. Tests in `classic_sets/gpt`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::{bloodthirst, etb, on_dies, target_any, target_filtered};
use crate::effect::{Duration, Effect, OpeningHandEffect, PlayerRef, Selector};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// 1/1 white Spirit token with flying.
fn spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
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
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Gargoyle],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(flatten.clone()),
            on_dies(Effect::HauntCreature {
                body: Box::new(flatten),
            }),
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
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(spirit_token()),
        },
    ]);
    CardDefinition {
        name: "Seize the Soul",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            body.clone(),
            Effect::HauntCreature {
                body: Box::new(body),
            },
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
        opening_hand: Some(OpeningHandEffect::StartInPlay {
            tapped: false,
            extra: Effect::Noop,
        }),
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
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Shaman],
            ..Default::default()
        },
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

/// Borborygmos — {3}{R}{R}{G}{G} legendary 6/7 Cyclops with trample. Whenever it
/// deals combat damage to a player, put a +1/+1 counter on each creature you
/// control.
pub fn borborygmos() -> CardDefinition {
    CardDefinition {
        name: "Borborygmos",
        cost: cost(&[generic(3), r(), r(), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cyclops],
            ..Default::default()
        },
        power: 6,
        toughness: 7,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Skarrgan Skybreaker — {4}{R}{R}{G} 3/3 Giant Shaman with bloodthirst 3. {1},
/// Sacrifice this creature: it deals damage equal to its power to any target.
pub fn skarrgan_skybreaker() -> CardDefinition {
    CardDefinition {
        name: "Skarrgan Skybreaker",
        cost: cost(&[generic(4), r(), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![bloodthirst(3)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::DealDamageEqualToPower {
                source: Selector::This,
                target: target_any(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// 1/1 colorless Sand creature token.
fn sand_token() -> TokenDefinition {
    TokenDefinition {
        name: "Sand".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sand],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Dune-Brood Nephilim — {B}{R}{G}{W} 3/3 Nephilim. Whenever it deals combat
/// damage to a player, create a 1/1 colorless Sand token for each land you
/// control.
pub fn dune_brood_nephilim() -> CardDefinition {
    CardDefinition {
        name: "Dune-Brood Nephilim",
        cost: cost(&[b(), r(), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nephilim],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::count(Selector::EachPermanent(R::Land.and(R::ControlledByYou))),
                definition: Box::new(sand_token()),
            },
        }],
        ..Default::default()
    }
}

/// Glint-Eye Nephilim — {U}{B}{R}{G} 2/2 Nephilim. Whenever it deals combat
/// damage to a player, draw that many cards. {1}, Discard a card: +1/+1 until
/// end of turn.
pub fn glint_eye_nephilim() -> CardDefinition {
    CardDefinition {
        name: "Glint-Eye Nephilim",
        cost: cost(&[u(), b(), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nephilim],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::TriggerEventAmount,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
