//! Commander: two-card gaps of the 2026-10-15 full-list EDHREC scan — the
//! cards several unseated average decks share. Tests in
//! `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
    Value, WardCost,
};
use crate::effect::shortcut::{choose_one_then, chosen_one, etb, on_dies, sneak, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic, u, w};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn lesson(name: &'static str, mana: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { spell_subtypes: vec![crate::card::SpellSubtype::Lesson], ..Default::default() },
        ..spell(name, mana, CardType::Instant, effect)
    }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn clue() -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(crabomination_base::tokens::clue_token()) }
}

/// CR 702.188 — web-slinging: pay `mana` and return a tapped creature you
/// control to its owner's hand.
fn web_slinging(mana: ManaCost) -> AlternativeCost {
    AlternativeCost { mana_cost: mana, return_to_hand: Some((R::Creature.and(R::Tapped), 1)), ..Default::default() }
}

/// "Until end of turn, Reptil becomes a Dinosaur Hero with base power and
/// toughness P/T and gains [keywords]."
fn dino_form(mana: ManaCost, p: i32, t: i32, keywords: &[Keyword]) -> ActivatedAbility {
    let mut steps = vec![
        Effect::BecomeCreatureType {
            what: Selector::This,
            creature_types: vec![CreatureType::Dinosaur, CreatureType::Hero],
            duration: Duration::EndOfTurn,
        },
        Effect::SetBasePT { what: Selector::This, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn },
    ];
    steps.extend(keywords.iter().map(|k| Effect::GrantKeyword { what: Selector::This, keyword: k.clone(), duration: Duration::EndOfTurn }));
    ActivatedAbility { mana_cost: mana, effect: Effect::Seq(steps), ..Default::default() }
}

/// Armaggon, Future Shark — {6}{B}{B} legendary 9/6 Shark Horror Mutant;
/// flash. ETB: destroy up to three target creatures. (Armaggon, Songbird,
/// The Falcon, Bebop & Rocksteady, Shark Shredder.)
pub fn armaggon_future_shark() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        })],
        ..legend(
            "Armaggon, Future Shark",
            cost(&[generic(6), b(), b()]),
            vec![CreatureType::Shark, CreatureType::Horror, CreatureType::Mutant],
            9,
            6,
        )
    }
}

/// Shark Shredder, Killer Clone — {2}{B}{B} legendary 4/4 Shark Octopus
/// Ninja; sneak {3}{B}{B}; first strike. Combat damage to a player: up to one
/// target creature card from that player's graveyard enters under your
/// control tapped and attacking that player.
pub fn shark_shredder_killer_clone() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        alternative_cost: Some(sneak(cost(&[generic(3), b(), b()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::InOpponentGraveyard).and(R::ControlledByTriggerPlayer),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    },
                    Effect::JoinCombatAttacking { what: Selector::LastMoved },
                ])),
            },
        }],
        ..legend(
            "Shark Shredder, Killer Clone",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Shark, CreatureType::Octopus, CreatureType::Ninja],
            4,
            4,
        )
    }
}

/// Songbird, Sonic Screamer — {1}{B} legendary 2/1 Human Villain; lifelink.
/// Discard a card: flying until end of turn.
pub fn songbird_sonic_screamer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..legend("Songbird, Sonic Screamer", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Villain], 2, 1)
    }
}

/// Unesh, Criosphinx Sovereign — {4}{U}{U} legendary 4/4 Sphinx; flying.
/// Sphinx spells cost {2} less. Whenever it or another Sphinx you control
/// enters, the Fact or Fiction split of your top four.
pub fn unesh_criosphinx_sovereign() -> CardDefinition {
    let sphinx = || R::HasCreatureType(CreatureType::Sphinx);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Sphinx spells you cast cost {2} less to cast.",
            effect: StaticEffect::CostReduction { filter: sphinx(), amount: 2 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: sphinx() }),
            effect: Effect::FactOrFiction { count: Value::Const(4), to_bottom: false },
        }],
        ..legend("Unesh, Criosphinx Sovereign", cost(&[generic(4), u(), u()]), vec![CreatureType::Sphinx], 4, 4)
    }
}

/// Teferi, Timebender — {4}{W}{U} planeswalker, loyalty 5. +2: untap up to
/// one target artifact or creature. −3: gain 2 life, draw two. −9: an extra
/// turn after this one. (Medomai the Ageless.)
pub fn teferi_timebender() -> CardDefinition {
    CardDefinition {
        name: "Teferi, Timebender",
        cost: cost(&[generic(4), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Teferi], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Untap {
                        what: target_filtered(R::HasCardType(CardType::Artifact).or(R::Creature)),
                        up_to: None,
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Seq(vec![Effect::GainLife { who: Selector::You, amount: Value::Const(2) }, draw(2)]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -9,
                effect: Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cartouche of Knowledge — {1}{U} Aura Cartouche on a creature you control.
/// ETB draw; enchanted creature gets +1/+1 and has flying. (Ms. Marvel.)
pub fn cartouche_of_knowledge() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Cartouche of Knowledge",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Cartouche],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: host(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Enchanted creature has flying.",
                effect: StaticEffect::GrantKeyword { applies_to: host(), keyword: Keyword::Flying },
            },
        ],
        triggered_abilities: vec![etb(draw(1))],
        ..Default::default()
    }
}

/// Cloak of Feathers — {U} sorcery: target creature gains flying until end of
/// turn; draw a card. (Ms. Marvel.)
pub fn cloak_of_feathers() -> CardDefinition {
    spell(
        "Cloak of Feathers",
        cost(&[u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            draw(1),
        ]),
    )
}

/// Spider-Rex, Daring Dino — {4}{G}{G} legendary 6/6 Spider Dinosaur Hero;
/// reach, trample, ward {2}. (Reptil, Moon-Boy.)
pub fn spider_rex_daring_dino() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        ..legend(
            "Spider-Rex, Daring Dino",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Spider, CreatureType::Dinosaur, CreatureType::Hero],
            6,
            6,
        )
    }
}

/// Reptil, Dinomorpher — {G} legendary 1/2 Human Hero. {3}: until end of
/// turn a 3/5 Dinosaur Hero with vigilance and reach. {6}: a 6/6 Dinosaur
/// Hero with trample.
pub fn reptil_dinomorpher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            dino_form(cost(&[generic(3)]), 3, 5, &[Keyword::Vigilance, Keyword::Reach]),
            dino_form(cost(&[generic(6)]), 6, 6, &[Keyword::Trample]),
        ],
        ..legend("Reptil, Dinomorpher", cost(&[g()]), vec![CreatureType::Human, CreatureType::Hero], 1, 2)
    }
}

/// Roaring Primadox — {3}{G} 4/4 Beast. At the beginning of your upkeep,
/// return a creature you control to its owner's hand. (Lizard, Connors's
/// Curse.)
pub fn roaring_primadox() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: choose_one_then(
                Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                PlayerRef::You,
                Effect::Move { what: chosen_one(), to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(chosen_one()))) },
            ),
        }],
        ..creature("Roaring Primadox", cost(&[generic(3), g()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Spider-Man, Brooklyn Visionary — {4}{G} legendary 4/3 Spider Human Hero;
/// web-slinging {2}{G}. ETB: a basic land onto the battlefield tapped.
/// (Lizard, Connors's Curse.)
pub fn spider_man_brooklyn_visionary() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(web_slinging(cost(&[generic(2), g()]))),
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        })],
        ..legend(
            "Spider-Man, Brooklyn Visionary",
            cost(&[generic(4), g()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            4,
            3,
        )
    }
}

/// Sokka's Sword Training — {1}{W} instant Lesson: target creature gets
/// +2/+2 until end of turn; create a Clue. (Aang, the Last Airbender.)
pub fn sokkas_sword_training() -> CardDefinition {
    lesson(
        "Sokka's Sword Training",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            clue(),
        ]),
    )
}

/// That's Rough Buddy — {1}{W} instant Lesson: a +1/+1 counter on target
/// creature (two if a creature left the battlefield under your control this
/// turn); draw a card. (Aang, the Last Airbender.)
pub fn thats_rough_buddy() -> CardDefinition {
    lesson(
        "That's Rough Buddy",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::IfPred {
                    pred: Box::new(Predicate::CreatureLeftBattlefieldThisTurn { who: PlayerRef::You }),
                    then: Box::new(Value::Const(2)),
                    else_: Box::new(Value::ONE),
                },
            },
            draw(1),
        ]),
    )
}

/// Paramecia Coloniex — {1}{B} 2/2 Zombie Worm. ETB mill three. When it
/// dies, you may exile it; when you do, put target creature card from your
/// graveyard on top of your library. (The Falcon, Airship Restored.)
pub fn paramecia_coloniex() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Mill { who: Selector::You, amount: Value::Const(3) }),
            on_dies(Effect::MayDo {
                description: "Exile Paramecia Coloniex to put a creature card from your graveyard on top of your library?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                    Effect::ReflexiveTrigger {
                        body: Box::new(Effect::Move {
                            what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                        }),
                    },
                ])),
            }),
        ],
        ..creature("Paramecia Coloniex", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Worm], 2, 2)
    }
}
