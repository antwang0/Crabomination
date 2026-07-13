//! FDN/BLB/DSK gap batch on existing primitives: Exemplar of Light (lifegain
//! → +1/+1 counters → a once-per-turn draw), Ashroot Animist (on-attack team
//! trample + power pump), Arahbo, the First Fang (Cat lord + nontoken-Cat ETB
//! tokens), Bumbleflower's Sharepot (Food + sac-to-destroy), and Celestial
//! Armor (flash Equipment granting hexproof/indestructible on entry). Tests in
//! `crabomination/src/tests/recent177.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack};
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{cost, g, generic, r, w, Color};

/// Exemplar of Light — {2}{W}{W} 3/3 Angel with flying. Whenever you gain life,
/// put a +1/+1 counter on it; whenever one or more +1/+1 counters are put on it,
/// draw a card (once each turn).
pub fn exemplar_of_light() -> CardDefinition {
    CardDefinition {
        name: "Exemplar of Light",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                    EventScope::SelfSource,
                )
                .once_per_turn(),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Ashroot Animist — {2}{R}{G} 4/4 Lizard Druid with trample. Whenever it
/// attacks, another target creature you control gains trample and gets +X/+X
/// until end of turn, where X is this creature's power.
pub fn ashroot_animist() -> CardDefinition {
    CardDefinition {
        name: "Ashroot Animist",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                },
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::PowerOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// A 1/1 white Cat creature token (Arahbo).
fn cat_token() -> TokenDefinition {
    TokenDefinition {
        name: "Cat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        ..Default::default()
    }
}

/// Arahbo, the First Fang — {2}{W} 2/2 Legendary Cat Avatar. Other Cats you
/// control get +1/+1. Whenever Arahbo or another nontoken Cat you control
/// enters, create a 1/1 white Cat token.
pub fn arahbo_the_first_fang() -> CardDefinition {
    let other_cats = Selector::EachPermanent(
        R::Creature
            .and(R::HasCreatureType(CreatureType::Cat))
            .and(R::ControlledByYou)
            .and(R::OtherThanSource),
    );
    CardDefinition {
        name: "Arahbo, the First Fang",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Avatar],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Other Cats you control get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: other_cats, power: 1, toughness: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Cat)).and(R::NotToken),
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: cat_token(),
            },
        }],
        ..Default::default()
    }
}

/// Bumbleflower's Sharepot — {2} Artifact. ETB: create a Food token.
/// {5},{T},Sacrifice this: destroy target nonland permanent (sorcery speed).
pub fn bumbleflowers_sharepot() -> CardDefinition {
    CardDefinition {
        name: "Bumbleflower's Sharepot",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: crabomination_base::tokens::food_token(),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            sorcery_speed: true,
            mana_cost: cost(&[generic(5)]),
            effect: Effect::Destroy {
                what: Selector::TargetFiltered { slot: 0, filter: R::Nonland },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Celestial Armor — {2}{W} Artifact — Equipment with flash. ETB: attach to
/// target creature you control; it gains hexproof and indestructible until end
/// of turn. Equipped creature gets +2/+0 and has flying. Equip {3}{W}.
pub fn celestial_armor() -> CardDefinition {
    CardDefinition {
        name: "Celestial Armor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(3), w()]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 0,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        ..Default::default()
    }
}
