//! A twenty-sixth wave — Aetherdrift (DFT) staples on existing primitives:
//! Mount/Saddle "while saddled" attack triggers, Exhaust activated abilities,
//! and vanilla bodies. Tests in `crabomination/src/tests/recent26.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, Selector, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::{Duration, PlayerRef};
use crate::game::effects::treasure_token;
use crate::mana::{cost, g, generic, r, u, w, Color};

/// "Whenever this creature attacks while saddled, [effect]."
fn attack_while_saddled(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
            .with_filter(Predicate::SourceSaddled),
        effect,
    }
}

/// Jibbirik Omnivore — {1}{G} 3/2 Beast. Vanilla.
pub fn jibbirik_omnivore() -> CardDefinition {
    CardDefinition {
        name: "Jibbirik Omnivore",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 2,
        ..Default::default()
    }
}

/// Caelorna, Coral Tyrant — {1}{U} 0/8 legendary Octopus. Vanilla wall.
pub fn caelorna_coral_tyrant() -> CardDefinition {
    CardDefinition {
        name: "Caelorna, Coral Tyrant",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Octopus], ..Default::default() },
        power: 0,
        toughness: 8,
        ..Default::default()
    }
}

/// Gilded Ghoda — {1}{R} 2/2 Horse Mount. Saddle 1. Attacks while saddled →
/// create a Treasure.
pub fn gilded_ghoda() -> CardDefinition {
    CardDefinition {
        name: "Gilded Ghoda",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horse, CreatureType::Mount],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Saddle(1)],
        triggered_abilities: vec![attack_while_saddled(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: treasure_token(),
        })],
        ..Default::default()
    }
}

/// Brightfield Mustang — {3}{W} 3/3 Horse Mount. Saddle 1. Attacks while
/// saddled → untap it and put a +1/+1 counter on it.
pub fn brightfield_mustang() -> CardDefinition {
    CardDefinition {
        name: "Brightfield Mustang",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horse, CreatureType::Mount],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Saddle(1)],
        triggered_abilities: vec![attack_while_saddled(Effect::Seq(vec![
            Effect::Untap { what: Selector::This, up_to: None },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// A 4/4 red Dinosaur Dragon with flying (Draconautics Engineer's exhaust token).
fn dino_dragon_token() -> TokenDefinition {
    TokenDefinition {
        name: "Dragon".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur, CreatureType::Dragon],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Draconautics Engineer — {1}{R} 2/2 Goblin Artificer. Exhaust {R}: other
/// creatures gain haste, put a +1/+1 counter on this. Exhaust {3}{R}: make a
/// 4/4 flying Dinosaur Dragon.
pub fn draconautics_engineer() -> CardDefinition {
    CardDefinition {
        name: "Draconautics Engineer",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                exhaust: true,
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: Selector::EachPermanent(
                            crate::card::SelectionRequirement::Creature
                                .and(crate::card::SelectionRequirement::ControlledByYou)
                                .and(crate::card::SelectionRequirement::OtherThanSource),
                        ),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), r()]),
                exhaust: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: dino_dragon_token(),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Afterburner Expert — {2}{G} 4/2 Goblin Artificer. Exhaust {2}{G}{G}: put two
/// +1/+1 counters on this creature. (The exhaust-activation reflexive trigger
/// is dropped.)
pub fn afterburner_expert() -> CardDefinition {
    CardDefinition {
        name: "Afterburner Expert",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Artificer],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), g()]),
            exhaust: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── DSK tail + simple bodies ─────────────────────────────────────────────────

/// Piranha Fly — {1}{U} 2/1 Fish Insect. Flying; enters tapped.
pub fn piranha_fly() -> CardDefinition {
    CardDefinition {
        name: "Piranha Fly",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![crate::card::StaticAbility {
            description: "This creature enters tapped.",
            effect: crate::card::StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        ..Default::default()
    }
}

/// Ripchain Razorkin — {3}{R} 5/3 Human Berserker. Reach; {2}{R}, Sacrifice a
/// land: Draw a card.
pub fn ripchain_razorkin() -> CardDefinition {
    CardDefinition {
        name: "Ripchain Razorkin",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            sac_other_filter: Some((crate::card::SelectionRequirement::Land, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Beastrider Vanguard — {1}{G} 2/2 Human Knight. {4}{G}: look at the top three;
/// you may reveal a permanent card and put it into your hand, rest on bottom.
pub fn beastrider_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Beastrider Vanguard",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: false,
                pick_filter: Some(crate::card::SelectionRequirement::Permanent),
                take: None,
                to_battlefield: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
