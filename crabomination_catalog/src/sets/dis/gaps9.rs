//! Dissension (DIS) closure — the Aethermage and the three split cards.
//! Tests in `classic_sets/dis`.

use crate::card::{
    CardDefinition, CardType, CreatureType, SelectionRequirement as R, SplitCard, SplitHalf,
    Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, Selector, TriggeredAbility,
};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn types(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// Azorius Aethermage — {1}{W}{U} 1/1. Whenever a permanent is returned to
/// your hand, you may pay {1} to draw a card.
pub fn azorius_aethermage() -> CardDefinition {
    CardDefinition {
        name: "Azorius Aethermage",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentReturnedToHand, EventScope::YourControl),
            effect: Effect::MayPay {
                mana_cost: cost(&[generic(1)]),
                description: "Pay 1 to draw a card?".into(),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Bound // Determined — {3}{B}{G} // {G}{U}. Bound: sacrifice a creature and
/// take back that many cards (one per colour it was). Determined: your other
/// spells can't be countered this turn, and draw.
pub fn bound_determined() -> CardDefinition {
    CardDefinition {
        name: "Bound // Determined",
        cost: cost(&[generic(3), b(), g()]),
        card_types: vec![CardType::Instant],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::Creature,
            },
            Effect::ReturnGraveyardCardsToHand {
                filter: R::Any,
                max: Value::ColorCountOf(Box::new(Selector::SacrificedCard)),
            },
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[g(), u()]),
                card_types: vec![CardType::Instant],
                effect: Effect::Seq(vec![
                    Effect::GrantSpellsUncounterableThisTurn { who: Selector::You },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Odds // Ends — {U}{R} // {3}{R}{W}. Odds: flip a coin to counter or copy a
/// spell. Ends: target player sacrifices two attacking creatures.
pub fn odds_ends() -> CardDefinition {
    CardDefinition {
        name: "Odds // Ends",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::FlipCoin {
            count: Value::ONE,
            on_heads: Box::new(Effect::CounterSpell {
                what: target_filtered(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                ),
            }),
            on_tails: Box::new(Effect::CopySpellMayChooseTargets {
                what: Selector::Target(0),
                count: Value::ONE,
            }),
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(3), r(), w()]),
                card_types: vec![CardType::Instant],
                effect: Effect::Sacrifice {
                    who: target_filtered(R::Player),
                    count: Value::Const(2),
                    filter: R::Creature.and(R::IsAttacking),
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Research // Development — {G}{U} // {3}{U}{R}. Research: shuffle up to four
/// cards from outside the game into your library. Development: three tokens,
/// each of which an opponent may swap for a card you draw.
pub fn research_development() -> CardDefinition {
    CardDefinition {
        name: "Research // Development",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::WishToLibrary {
            filter: R::Any,
            max: Value::Const(4),
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(3), u(), r()]),
                card_types: vec![CardType::Instant],
                effect: Effect::TokenUnlessOpponentLetsYouDraw {
                    token: TokenDefinition {
                        name: "Elemental".into(),
                        power: 3,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Red],
                        subtypes: types(vec![CreatureType::Elemental]),
                        ..Default::default()
                    },
                    times: 3,
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Experiment Kraj — {2}{G}{G}{U}{U} 4/6. It has every activated ability of
/// each other creature with a +1/+1 counter, and grows them itself.
pub fn experiment_kraj() -> CardDefinition {
    CardDefinition {
        name: "Experiment Kraj",
        cost: cost(&[generic(2), g(), g(), u(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: types(vec![CreatureType::Ooze, CreatureType::Mutant]),
        power: 4,
        toughness: 6,
        static_abilities: vec![crate::card::StaticAbility {
            description: "Has all activated abilities of other +1/+1-countered creatures.",
            effect: crate::effect::StaticEffect::HasActivatedAbilitiesOfCounteredCreatures,
        }],
        activated_abilities: vec![crate::card::ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Swerve — {U}{R} Instant. Change the target of target spell with a single
/// target (CR 115.7a/b).
pub fn swerve() -> CardDefinition {
    CardDefinition {
        name: "Swerve",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChangeSpellTarget { what: target_filtered(R::IsSpellOnStack) },
        ..Default::default()
    }
}
