//! FDN/BLB/DSK gap batch on existing primitives: Exemplar of Light (lifegain
//! → +1/+1 counters → a once-per-turn draw), Ashroot Animist (on-attack team
//! trample + power pump), Arahbo, the First Fang (Cat lord + nontoken-Cat ETB
//! tokens), Bumbleflower's Sharepot (Food + sac-to-destroy), and Celestial
//! Armor (flash Equipment granting hexproof/indestructible on entry). Second
//! wave: Strix Lookout (looter), Vanguard Seraph (first-lifegain surveil),
//! Vampire Soulcaller (gy-return flier), Turn Inside Out (trick + death-manifest),
//! Huskburster Swarm (graveyard-affinity fatty). Tests in
//! `crabomination/src/tests/recent177.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

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

/// Strix Lookout — {1}{U} 1/2 Bird with flying and vigilance. {1}{U},{T}: draw
/// a card, then discard a card.
pub fn strix_lookout() -> CardDefinition {
    CardDefinition {
        name: "Strix Lookout",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vanguard Seraph — {3}{W} 3/3 Angel Warrior with flying. Whenever you gain
/// life for the first time each turn, surveil 1.
pub fn vanguard_seraph() -> CardDefinition {
    CardDefinition {
        name: "Vanguard Seraph",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            // once_per_turn models "for the first time each turn".
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl).once_per_turn(),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Vampire Soulcaller — {4}{B} 3/2 Vampire Warlock with flying that can't block.
/// When it enters, return target creature card from your graveyard to your hand.
pub fn vampire_soulcaller() -> CardDefinition {
    CardDefinition {
        name: "Vampire Soulcaller",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::CantBlock],
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::InYourGraveyard) },
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Turn Inside Out — {R} Instant. Target creature gets +3/+0 until end of turn.
/// When it dies this turn, manifest dread.
pub fn turn_inside_out() -> CardDefinition {
    CardDefinition {
        name: "Turn Inside Out",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature },
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::WhenTargetDiesThisTurn {
                slot: 0,
                body: Box::new(Effect::ManifestDread { who: PlayerRef::You }),
                filter: None,
            },
        ]),
        ..Default::default()
    }
}

/// Huskburster Swarm — {7}{B} 6/6 Elemental Insect with menace and deathtouch.
/// Costs {1} less per creature card in your graveyard. (The printed rider also
/// counts exiled creature cards you own — that half is approximated.)
pub fn huskburster_swarm() -> CardDefinition {
    CardDefinition {
        name: "Huskburster Swarm",
        cost: cost(&[generic(7), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Insect],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Menace, Keyword::Deathtouch],
        affinity_graveyard_filter: Some(R::Creature),
        ..Default::default()
    }
}
