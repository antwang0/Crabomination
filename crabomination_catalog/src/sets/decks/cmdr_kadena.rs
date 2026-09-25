//! Commander: the cards the **Faceless Menace** precon (C19, Kadena,
//! Slinking Sorcerer) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_kadena.rs`.
//!
//! Residuals (each also on its card):
//! - **Gift of Doom** — turned face up it isn't attached as it turns (the
//!   "as" replacement), so it attaches by a trigger if it survives.
//! - **Rayami, First of the Fallen** — protection isn't among the shared
//!   keywords.
//! - **Road of Return** — the entwined cast takes both modes; the choice of
//!   one is the engine's.
//! - **Vesuvan Shapeshifter** — it copies only as it enters (not as it's
//!   turned face up), and has no upkeep turn-face-down ability.
//! - **Volrath, the Shapestealer** — the copy isn't 7/5 and loses the {1}
//!   ability for the turn.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EntersAsCopy,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, u};
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

fn turned_face_up(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource), effect }
}

fn not_yours() -> R {
    R::Creature.and(R::ControlledByYou.negate())
}

/// Kadena, Slinking Sorcerer — {1}{B}{G}{U} 3/3 Snake Wizard. The first
/// face-down creature spell you cast each turn costs {3} less (CR 702.37);
/// a face-down creature entering under you draws a card.
pub fn kadena_slinking_sorcerer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        can_be_commander: true,
        static_abilities: vec![StaticAbility {
            description: "The first face-down creature spell you cast each turn costs {3} less to cast.",
            effect: StaticEffect::FirstFaceDownSpellEachTurnCostsLess { amount: 3 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::FaceDown) },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Kadena, Slinking Sorcerer",
            cost(&[generic(1), b(), g(), u()]),
            vec![CreatureType::Snake, CreatureType::Wizard, CreatureType::Sorcerer],
            3,
            3,
        )
    }
}

/// Apex Altisaur — {7}{G}{G} 10/10 Dinosaur. Entering, and whenever it's
/// dealt damage (enrage), it fights up to one creature you don't control.
pub fn apex_altisaur() -> CardDefinition {
    let fight = || Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::Fight { attacker: Selector::This, defender: target_filtered(not_yours()) }),
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(fight()),
            TriggeredAbility { event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource), effect: fight() },
        ],
        ..creature("Apex Altisaur", cost(&[generic(7), g(), g()]), vec![CreatureType::Dinosaur], 10, 10)
    }
}

/// Bounty of the Luxa — {2}{G}{U} enchantment. Your first main phase: with
/// flood counters on it, remove them and add {C}{G}{U}; without, put one on
/// it and draw.
pub fn bounty_of_the_luxa() -> CardDefinition {
    let flooded = Predicate::ValueAtLeast(
        Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Flood },
        Value::ONE,
    );
    CardDefinition {
        name: "Bounty of the Luxa",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::ActivePlayer),
            effect: Effect::If {
                cond: flooded,
                then: Box::new(Effect::Seq(vec![
                    Effect::RemoveCounter {
                        what: Selector::This,
                        kind: CounterType::Flood,
                        amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Flood },
                    },
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green, Color::Blue]) },
                ])),
                else_: Box::new(Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::This, kind: CounterType::Flood, amount: Value::ONE },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Foul Orchard — enters tapped; {T}: {B} or {G}.
pub fn foul_orchard() -> CardDefinition {
    CardDefinition {
        name: "Foul Orchard",
        card_types: vec![CardType::Land],
        activated_abilities: vec![crate::sets::tap_add(Color::Black), crate::sets::tap_add(Color::Green)],
        static_abilities: vec![crate::sets::enters_tapped()],
        ..Default::default()
    }
}

/// Gift of Doom — {4}{B} Aura: the enchanted creature has deathtouch and
/// indestructible. Morph—sacrifice another creature.
/// Residual: turned face up, it attaches by a trigger, not as it turns.
pub fn gift_of_doom() -> CardDefinition {
    let grant = |keyword: Keyword| StaticAbility {
        description: "Enchanted creature has deathtouch and indestructible.",
        effect: StaticEffect::GrantKeyword { applies_to: Selector::AttachedTo(Box::new(Selector::This)), keyword },
    };
    CardDefinition {
        name: "Gift of Doom",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::MorphCost(Box::new(WardCost::SacrificeMatchingN(
            Box::new(R::Creature.and(R::OtherThanSource)),
            1,
        )))],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![grant(Keyword::Deathtouch), grant(Keyword::Indestructible)],
        triggered_abilities: vec![turned_face_up(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::OtherThanSource)),
        })],
        ..Default::default()
    }
}

/// Grismold, the Dreadsower — {1}{B}{G} 3/3 Troll Shaman, trample. Your end
/// step gives each player a 1/1 Plant; a creature token dying grows it.
pub fn grismold_the_dreadsower() -> CardDefinition {
    let plant = TokenDefinition {
        name: "Plant".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        can_be_commander: true,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
                effect: Effect::CreateToken { who: PlayerRef::EachPlayer, count: Value::ONE, definition: Arc::new(plant) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::NotToken.negate()) },
                ),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..creature(
            "Grismold, the Dreadsower",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Troll, CreatureType::Shaman],
            3,
            3,
        )
    }
}

/// Icefeather Aven — {G}{U} 2/2 Bird Shaman, flying, morph {1}{G}{U}. Turned
/// face up, it may bounce another creature.
pub fn icefeather_aven() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(1), g(), u()]))],
        triggered_abilities: vec![turned_face_up(Effect::MayDo {
            description: "Return another target creature to its owner's hand?".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            }),
        })],
        ..creature("Icefeather Aven", cost(&[g(), u()]), vec![CreatureType::Bird, CreatureType::Shaman], 2, 2)
    }
}

/// Kadena's Silencer — {1}{U} 2/1 Snake Wizard, megamorph {1}{U}. Turned
/// face up, it counters every ability your opponents control.
pub fn kadenas_silencer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Megamorph(cost(&[generic(1), u()]))],
        triggered_abilities: vec![turned_face_up(Effect::CounterAllAbilitiesOf { who: PlayerRef::EachOpponent })],
        ..creature(
            "Kadena's Silencer",
            cost(&[generic(1), u()]),
            vec![CreatureType::Snake, CreatureType::Wizard],
            2,
            1,
        )
    }
}

/// Leadership Vacuum — {2}{U} instant. Target player returns each commander
/// they control to the command zone (CR 903.9); draw a card.
pub fn leadership_vacuum() -> CardDefinition {
    CardDefinition {
        name: "Leadership Vacuum",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            // "Target player": a zero-card draw names slot 0's filter.
            Effect::Draw { who: Selector::TargetFiltered { slot: 0, filter: R::Player }, amount: Value::Const(0) },
            Effect::ReturnCommandersToCommandZone { who: PlayerRef::Target(0) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Mire in Misery — {1}{B} sorcery. Each opponent sacrifices a creature or
/// enchantment.
pub fn mire_in_misery() -> CardDefinition {
    CardDefinition {
        name: "Mire in Misery",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::ONE,
            filter: R::Creature.or(R::HasCardType(CardType::Enchantment)),
        },
        ..Default::default()
    }
}

/// Rayami, First of the Fallen — {1}{B}{G}{U} 5/4 Vampire. A nontoken
/// creature that would die is exiled with a blood counter instead (CR 614);
/// Rayami has each listed keyword an exiled blood-countered creature card
/// has.
/// Residual: protection isn't shared.
pub fn rayami_first_of_the_fallen() -> CardDefinition {
    let shared = [
        Keyword::Flying,
        Keyword::FirstStrike,
        Keyword::DoubleStrike,
        Keyword::Deathtouch,
        Keyword::Haste,
        Keyword::Hexproof,
        Keyword::Indestructible,
        Keyword::Lifelink,
        Keyword::Menace,
        Keyword::Reach,
        Keyword::Trample,
        Keyword::Vigilance,
    ];
    let mut statics = vec![StaticAbility {
        description: "If a nontoken creature would die, exile that card with a blood counter on it instead.",
        effect: StaticEffect::ExileDyingNontokenCreaturesWithCounter { counter: CounterType::Blood },
    }];
    statics.extend(shared.into_iter().map(|keyword| StaticAbility {
        description: "Rayami has the keywords of exiled creature cards with blood counters on them.",
        effect: StaticEffect::WhileCondition {
            condition: Predicate::SelectorCountAtLeast {
                sel: Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: crate::card::Zone::Exile,
                    filter: R::Creature.and(R::WithCounter(CounterType::Blood)).and(R::HasKeyword(keyword.clone())),
                },
                n: Value::ONE,
            },
            inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword }),
        },
    }));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        can_be_commander: true,
        static_abilities: statics,
        ..creature(
            "Rayami, First of the Fallen",
            cost(&[generic(1), b(), g(), u()]),
            vec![CreatureType::Vampire],
            5,
            4,
        )
    }
}

/// Road of Return — {G}{G} sorcery, entwine {2}: return a permanent card
/// from your graveyard to hand, and/or put your commander into your hand.
/// Residual: an unentwined cast's mode is the engine's pick.
pub fn road_of_return() -> CardDefinition {
    CardDefinition {
        name: "Road of Return",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Entwine(cost(&[generic(2)]))],
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::PermanentCard.from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::CommanderToHand { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}

/// Sagu Mauler — {4}{G}{U} 6/6 Beast, trample, hexproof, morph {3}{G}{U}.
pub fn sagu_mauler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Hexproof, Keyword::Morph(cost(&[generic(3), g(), u()]))],
        ..creature("Sagu Mauler", cost(&[generic(4), g(), u()]), vec![CreatureType::Beast], 6, 6)
    }
}

/// Secret Plans — {G}{U} enchantment. Your face-down creatures get +0/+1;
/// a permanent of yours turned face up draws a card.
pub fn secret_plans() -> CardDefinition {
    CardDefinition {
        name: "Secret Plans",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Face-down creatures you control get +0/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::FaceDown).and(R::ControlledByYou)),
                power: 0,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Silumgar Assassin — {1}{B} 2/1 Human Assassin; bigger creatures can't
/// block it; megamorph {2}{B}; turned face up, it destroys an opponent's
/// creature with power 3 or less.
pub fn silumgar_assassin() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            // The unnamed text of skulk (CR 702.118b), enforced against the
            // attacker's computed power where the block-filter walker can't.
            Keyword::Skulk,
            Keyword::Megamorph(cost(&[generic(2), b()])),
        ],
        triggered_abilities: vec![turned_face_up(Effect::Destroy {
            what: target_filtered(R::Creature.and(R::PowerAtMost(3)).and(R::ControlledByOpponent)),
        })],
        ..creature(
            "Silumgar Assassin",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            2,
            1,
        )
    }
}

/// Stratus Dancer — {1}{U} 2/1 Djinn Monk, flying, megamorph {1}{U}. Turned
/// face up, it counters an instant or sorcery spell.
pub fn stratus_dancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Megamorph(cost(&[generic(1), u()]))],
        triggered_abilities: vec![turned_face_up(Effect::CounterSpell {
            what: target_filtered(
                R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            ),
        })],
        ..creature("Stratus Dancer", cost(&[generic(1), u()]), vec![CreatureType::Djinn, CreatureType::Monk], 2, 1)
    }
}

/// Sudden Substitution — {2}{U}{U} instant, split second. Exchange control
/// of a noncreature spell and a creature; the spell's new controller may
/// choose new targets.
pub fn sudden_substitution() -> CardDefinition {
    CardDefinition {
        name: "Sudden Substitution",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::SplitSecond],
        effect: Effect::ExchangeSpellAndCreatureControl {
            a: target_filtered(R::IsSpellOnStack.and(R::Noncreature)),
            b: Selector::TargetFiltered { slot: 1, filter: R::Creature },
        },
        ..Default::default()
    }
}

/// Thought Sponge — {3}{U} 1/1 Sponge, flash. It enters with +1/+1 counters
/// equal to the most cards an opponent drew this turn; dying, it draws its
/// power (last known, CR 603.10).
pub fn thought_sponge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::GreatestCardsDrawnThisTurnByAnOpponent)),
        triggered_abilities: vec![on_dies(Effect::Draw {
            who: Selector::You,
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        ..creature("Thought Sponge", cost(&[generic(3), u()]), vec![CreatureType::Sponge], 1, 1)
    }
}

/// Thousand Winds — {4}{U}{U} 5/6 Elemental, flying, morph {5}{U}{U}.
/// Turned face up, every other tapped creature returns to its owner's hand.
pub fn thousand_winds() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Morph(cost(&[generic(5), u(), u()]))],
        triggered_abilities: vec![turned_face_up(Effect::Move {
            what: Selector::EachPermanent(R::Creature.and(R::Tapped).and(R::OtherThanSource)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature("Thousand Winds", cost(&[generic(4), u(), u()]), vec![CreatureType::Elemental], 5, 6)
    }
}

/// Vesuvan Shapeshifter — {3}{U}{U} 0/0 Shapeshifter, morph {1}{U}. As it
/// enters it may become a copy of another creature (CR 707.2).
/// Residual: no copy as it's turned face up, and no upkeep turn-face-down.
pub fn vesuvan_shapeshifter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), u()]))],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.and(R::OtherThanSource),
            ..Default::default()
        }),
        ..creature(
            "Vesuvan Shapeshifter",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Shapeshifter],
            0,
            0,
        )
    }
}

/// Volrath, the Shapestealer — {2}{B}{G}{U} 7/5 Phyrexian Shapeshifter. Your
/// combat puts a -1/-1 counter on up to one creature; {1}: until your next
/// turn it becomes a copy of a creature with a counter on it.
/// Residual: the copy isn't 7/5 and doesn't keep the {1} ability.
pub fn volrath_the_shapestealer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        can_be_commander: true,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::BecomeCopyOfFor {
                what: Selector::This,
                source: target_filtered(R::Creature.and(R::WithAnyCounter).and(R::OtherThanSource)),
                duration: Duration::UntilNextTurn,
                non_legendary: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Volrath, the Shapestealer",
            cost(&[generic(2), b(), g(), u()]),
            vec![CreatureType::Phyrexian, CreatureType::Shapeshifter],
            7,
            5,
        )
    }
}
