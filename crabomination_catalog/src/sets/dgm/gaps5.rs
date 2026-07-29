//! Dragon's Maze (DGM) gap cards, wave 5 — the last cards that were each
//! blocked on a single primitive. Tests in `classic_sets/dgm`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, SplitCard, SplitHalf, StaticAbility, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Predicate, Selector, StaticEffect};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Melek, Izzet Paragon — {4}{U}{R} 2/4 legendary Weird Wizard. Plays with the
/// top of your library revealed, casts instants and sorceries from there, and
/// copies each one cast from the library.
pub fn melek_izzet_paragon() -> CardDefinition {
    let instant_or_sorcery = R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Melek, Izzet Paragon",
        cost: cost(&[generic(4), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Weird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "Play with the top card of your library revealed.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may cast instant and sorcery spells from the top of your \
                              library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: instant_or_sorcery.clone() },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::CastSpellMatches(instant_or_sorcery),
                    Predicate::CastSpellFromLibrary,
                ])),
            effect: Effect::CopySpellMayChooseTargets {
                what: Selector::TriggerSource,
                count: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Plasm Capture — {G}{G}{U}{U} Instant. Counter target spell; at your next
/// main phase add mana of any colors equal to that spell's mana value.
pub fn plasm_capture() -> CardDefinition {
    CardDefinition {
        name: "Plasm Capture",
        cost: cost(&[g(), g(), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::AddManaAtNextMainPhase {
                amount: Value::CounteredSpellManaValue,
                any_color: true,
            },
        ]),
        ..Default::default()
    }
}

/// Goblin Test Pilot — {1}{U}{R} 0/2 Goblin Pilot Wizard with flying.
/// {T}: 2 damage to any target chosen at random.
pub fn goblin_test_pilot() -> CardDefinition {
    CardDefinition {
        name: "Goblin Test Pilot",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Goblin,
                CreatureType::Pilot,
                CreatureType::Wizard,
            ],
            ..Default::default()
        },
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: Selector::RandomAmong(
                    R::Creature.or(R::Player).or(R::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Catch // Release — {1}{U}{R} // {4}{R}{W} Sorcery split with Fuse. Catch
/// steals a creature for the turn; Release is a five-type edict on each player.
pub fn catch_release() -> CardDefinition {
    CardDefinition {
        name: "Catch // Release",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(4), r(), w()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(
                    [R::Artifact, R::Creature, R::Enchantment, R::Land, R::Planeswalker]
                        .into_iter()
                        .map(|filter| Effect::Sacrifice {
                            who: Selector::Player(crate::effect::PlayerRef::EachPlayer),
                            count: Value::ONE,
                            filter,
                        })
                        .collect(),
                ),
            },
            fuse: true,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Lairwatch Giant — {5}{W} 4/4 Giant Warrior (LRW). Blocks an additional
/// creature each combat; when it blocks two or more creatures it gains first
/// strike (CR 509.3e). Filed with the multi-block cluster.
pub fn lairwatch_giant() -> CardDefinition {
    CardDefinition {
        name: "Lairwatch Giant",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::CanBlockAdditional(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BlocksNOrMore(2), EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Flesh // Blood — {3}{B}{G} // {R}{G} Sorcery // Sorcery, Fuse. Flesh exiles
/// a graveyard creature and grows a creature by its power; Blood makes a
/// creature you control deal its power to any target.
pub fn flesh_blood() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Flesh // Blood",
        cost: cost(&[generic(3), b(), g()]),
        card_types: vec![CardType::Sorcery],
        // Counters first, so `Value::PowerOf` reads the card while it is still
        // in the graveyard.
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
            Effect::Exile { what: target_filtered(R::Creature.and(R::InGraveyard)) },
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[r(), g()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::DealDamageEqualToPower {
                    source: target_filtered(R::Creature.and(R::ControlledByYou)),
                    target: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.or(R::Player).or(R::Planeswalker),
                    },
                },
            },
            fuse: true,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Legion's Initiative — {R}{W} Enchantment. Red creatures get +1/+0, white
/// ones +0/+1; {R}{W}, exile this: blink all your creatures until the next
/// combat, where they return with haste.
pub fn legions_initiative() -> CardDefinition {
    use crate::effect::{DelayedTriggerKind, PlayerRef, ZoneDest};
    use crate::mana::Color;
    let anthem = |color, power, toughness| StaticAbility {
        description: "Creatures you control of a color get a bonus.",
        effect: StaticEffect::PumpPT {
            applies_to: Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::HasColor(color)),
            ),
            power,
            toughness,
        },
    };
    CardDefinition {
        name: "Legion's Initiative",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![anthem(Color::Red, 1, 0), anthem(Color::White, 0, 1)],
        // The self-exile runs in the effect rather than as a cost (an
        // `exile_self_cost` activation is graveyard-only today).
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), w()]),
            effect: Effect::Seq(vec![
                Effect::ExileLinked {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                },
                Effect::ExileSource,
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextCombat,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: Selector::CardExiledWithSource,
                            to: ZoneDest::Battlefield {
                                controller: PlayerRef::OwnerOfMoved,
                                tapped: false,
                            },
                        },
                        Effect::GrantKeyword {
                            what: Selector::LastMoved,
                            keyword: Keyword::Haste,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Reap Intellect — {X}{2}{U}{B} Sorcery. Target opponent reveals their hand;
/// exile up to X nonland cards from it and every same-named card they own.
pub fn reap_intellect() -> CardDefinition {
    use crate::effect::PlayerRef;
    use crate::mana::{b, x};
    CardDefinition {
        name: "Reap Intellect",
        cost: cost(&[x(), generic(2), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileChosenFromHand {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::XFromCost,
                filter: R::Nonland,
                link_to_source: true,
                face_down: false,
            },
            Effect::ExileSameNameAsTarget { what: Selector::CardExiledWithSource },
        ]),
        ..Default::default()
    }
}
