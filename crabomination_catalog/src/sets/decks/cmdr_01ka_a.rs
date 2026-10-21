//! Commander: the 2026-10-21 (`01KabMxS`) Attraction-rider closers — The Most
//! Dangerous Gamer, Dee Kay and Spinnerette's EDHREC averages. Tests in
//! `tests/recent_b/cmdr_01ka_a.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Effect, KeywordAct, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, hybrid, u};
use crabomination_base::tokens::treasure_token;
use std::sync::Arc;

fn creature(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn visited_this_turn() -> Predicate {
    Predicate::ValueAtLeast(Value::AttractionsVisitedThisTurn(PlayerRef::You), Value::ONE)
}

fn treasures(n: i32) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(treasure_token()) }
}

/// "Whenever you roll a `n`" — one event per die (CR 706.2), the result exact.
fn on_roll(n: u8, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Performed(KeywordAct::DieResult), EventScope::YourControl).with_filter(
            Predicate::All(vec![
                Predicate::DieResultAtLeast(n),
                Predicate::Not(Box::new(Predicate::DieResultAtLeast(n + 1))),
            ]),
        ),
        effect,
    }
}

/// Soul Swindler — {4}{B} 5/3 Demon Employee. Indestructible as long as you've
/// visited an Attraction this turn (CR 717.5); ETB open an Attraction.
pub fn soul_swindler() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as you've visited an Attraction this turn, this creature has indestructible.",
            effect: StaticEffect::WhileCondition {
                condition: visited_this_turn(),
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Indestructible }),
            },
        }],
        triggered_abilities: vec![etb(Effect::OpenAnAttraction)],
        ..creature("Soul Swindler", cost(&[generic(4), b()]), vec![CreatureType::Demon, CreatureType::Employee], 5, 3)
    }
}

/// Squirrel Squatters — {3}{G}{G} 4/4 Squirrel. ETB open an Attraction;
/// attacking, a tapped-and-attacking 1/1 Squirrel per Attraction you've
/// visited this turn.
pub fn squirrel_squatters() -> CardDefinition {
    let squirrel = TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Squirrel], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::OpenAnAttraction),
            on_attack(Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::AttractionsVisitedThisTurn(PlayerRef::You),
                definition: Arc::new(squirrel),
                cleanup: Default::default(),
                defender: None,
            }),
        ],
        ..creature("Squirrel Squatters", cost(&[generic(3), g(), g()]), vec![CreatureType::Squirrel], 4, 4)
    }
}

/// Spinnerette, Arachnobat — {3}{B}{G} 2/4 legendary Spider Performer, reach.
/// ETB open an Attraction; with three or more Attractions, +2/+0 and menace.
pub fn spinnerette_arachnobat() -> CardDefinition {
    let three = || Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::HasArtifactSubtype(ArtifactSubtype::Attraction).and(R::ControlledByYou)),
        n: Value::Const(3),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Reach],
        static_abilities: vec![
            StaticAbility {
                description: "As long as you control three or more Attractions, Spinnerette gets +2/+0.",
                effect: StaticEffect::WhileCondition {
                    condition: three(),
                    inner: Box::new(StaticEffect::PumpPT { applies_to: Selector::This, power: 2, toughness: 0 }),
                },
            },
            StaticAbility {
                description: "As long as you control three or more Attractions, Spinnerette has menace.",
                effect: StaticEffect::WhileCondition {
                    condition: three(),
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Menace }),
                },
            },
        ],
        triggered_abilities: vec![etb(Effect::OpenAnAttraction)],
        ..creature(
            "Spinnerette, Arachnobat",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Spider, CreatureType::Performer],
            2,
            4,
        )
    }
}

/// Motion Sickness — {3}{U} Aura. ETB taps the enchanted creature, which
/// doesn't untap during its controller's untap step. Whenever you visit an
/// Attraction, you may attach it to target tapped creature (CR 701.3).
pub fn motion_sickness() -> CardDefinition {
    CardDefinition {
        name: "Motion Sickness",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![
            etb(Effect::Tap { what: Selector::AttachedTo(Box::new(Selector::This)) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::VisitedAttraction, EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Attach Motion Sickness to target tapped creature?".into(),
                    body: Box::new(Effect::AttachSourceTo { host: target_filtered(R::Creature.and(R::Tapped)) }),
                },
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::AttachedTo(Box::new(Selector::This)) },
        }],
        ..Default::default()
    }
}

/// Dee Kay, Finder of the Lost — {1}{U}{B} 1/4 legendary Zombie Employee. ETB
/// open an Attraction. Roll a 2: each opponent loses 1, you gain 1. Roll a 4:
/// you may tap or untap target artifact or creature. Roll a 6: return target
/// creature card from your graveyard to your hand.
pub fn dee_kay_finder_of_the_lost() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            etb(Effect::OpenAnAttraction),
            on_roll(
                2,
                Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ]),
            ),
            on_roll(
                4,
                Effect::MayDo {
                    description: "Tap or untap target artifact or creature?".into(),
                    body: Box::new(Effect::TapOrUntap { what: target_filtered(R::Artifact.or(R::Creature)) }),
                },
            ),
            on_roll(
                6,
                Effect::Move {
                    what: target_filtered(R::Creature.from_your_graveyard()),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
        ],
        ..creature(
            "Dee Kay, Finder of the Lost",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Zombie, CreatureType::Employee],
            1,
            4,
        )
    }
}

/// Done for the Day — {2}{G} enchantment. At your end step, if you control an
/// Employee, a Performer, or a Robot (CR 603.4), you may get {TK} or create a
/// Treasure; with all three, you may put a sticker on a nonland permanent you
/// own (name stickers only, CR 123).
pub fn done_for_the_day() -> CardDefinition {
    let control = |t| Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::HasCreatureType(t).and(R::ControlledByYou)),
        n: Value::ONE,
    };
    let kinds = || [CreatureType::Employee, CreatureType::Performer, CreatureType::Robot].map(control).to_vec();
    CardDefinition {
        name: "Done for the Day",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::Any(kinds())),
            effect: Effect::Seq(vec![
                Effect::ChooseMode(vec![Effect::GainTickets { who: PlayerRef::You, amount: 1 }, treasures(1)]),
                Effect::If {
                    cond: Predicate::All(kinds()),
                    then: Box::new(Effect::ChooseOneAmong {
                        what: Selector::EachPermanent(R::Land.negate().and(R::OwnedByYou)),
                        chooser: PlayerRef::You,
                        chosen: Box::new(Effect::PutNameSticker {
                            what: Selector::SeparatedPile { chosen: true },
                            optional: true,
                        }),
                        other: Box::new(Effect::Noop),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Hogaak, Arisen Necropolis — {5}{B/G}{B/G} 8/8 legendary Avatar, trample.
/// You can't spend mana to cast it: convoke and delve pay it all (both apply
/// after the total cost, so they pay a commander's tax too — CR 702.51a /
/// 702.66a / 903.8). You may cast it from your graveyard.
pub fn hogaak_arisen_necropolis() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Convoke, Keyword::Delve, Keyword::Trample],
        no_mana_spend: true,
        static_abilities: vec![StaticAbility {
            description: "You may cast this card from your graveyard.",
            effect: StaticEffect::CastSelfFromGraveyard,
        }],
        ..creature(
            "Hogaak, Arisen Necropolis",
            cost(&[generic(5), hybrid(Color::Black, Color::Green), hybrid(Color::Black, Color::Green)]),
            vec![CreatureType::Avatar],
            8,
            8,
        )
    }
}
