//! Commander: one-card gaps of the 2026-10-15 rescan (`01Kvb…` session).
//! Tests in `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    LoyaltyAbility, PlaneswalkerSubtype,
    ActivatedAbility, ArtifactSubtype, BattleSubtype, CounterType, EnchantmentSubtype, EquipBonus, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w, x};
use crate::sets::tap_add_colorless;
use std::sync::Arc;

fn legend(name: &'static str, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: ManaCost::default(),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn siege(name: &'static str, mana: ManaCost, defense: u32, on_enter: Effect, back: CardDefinition) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() },
        defense,
        triggered_abilities: vec![etb(on_enter)],
        back_face: Some(Box::new(back)),
        ..Default::default()
    }
}

/// Invasion of Fiora — {4}{B}{B} Siege (4). Enters: choose one or both —
/// destroy all legendary creatures; destroy all nonlegendary creatures.
/// Back: Marchesa, Resolute Monarch — 3/6 menace deathtouch; attacking,
/// remove all counters from up to one target permanent; at your upkeep, if
/// you haven't been dealt combat damage since your last turn, draw and lose 1.
pub fn invasion_of_fiora() -> CardDefinition {
    let legendary = || R::HasSupertype(Supertype::Legendary);
    let upkeep_if_undamaged = Predicate::Not(Box::new(Predicate::CombatDamagedSinceYourTurn { who: PlayerRef::You }));
    let marchesa = CardDefinition {
        color_indicator: vec![Color::Black],
        keywords: vec![Keyword::Menace, Keyword::Deathtouch],
        triggered_abilities: vec![
            on_attack(Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Permanent,
                effect: Box::new(Effect::RemoveAllCounters { what: Selector::Target(0) }),
            }),
            // CR 603.4 — an intervening "if": checked as it triggers and again
            // on resolution.
            TriggeredAbility {
                event: EventSpec {
                    filter: Some(upkeep_if_undamaged.clone()),
                    ..EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                },
                effect: Effect::If {
                    cond: upkeep_if_undamaged,
                    then: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..legend("Marchesa, Resolute Monarch", vec![CreatureType::Human, CreatureType::Noble], 3, 6)
    };
    siege(
        "Invasion of Fiora",
        cost(&[generic(4), b(), b()]),
        4,
        Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(legendary())) },
                Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(legendary())))) },
            ],
        },
        marchesa,
    )
}

/// Invasion of Segovia — {2}{U} Siege (4). Enters: two 1/1 blue Kraken
/// tokens with trample. Back: Caetus, Sea Tyrant of Segovia — 3/3;
/// noncreature spells you cast have convoke; at your end step, untap up to
/// four target creatures.
pub fn invasion_of_segovia() -> CardDefinition {
    let kraken = TokenDefinition {
        name: "Kraken".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kraken], ..Default::default() },
        ..Default::default()
    };
    let caetus = CardDefinition {
        color_indicator: vec![Color::Blue],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells you cast have convoke.",
            effect: StaticEffect::GrantConvokeToSpells { filter: R::Noncreature },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::ApplyToTargets {
                max_targets: 4,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
            },
        }],
        ..legend("Caetus, Sea Tyrant of Segovia", vec![CreatureType::Serpent], 3, 3)
    };
    siege(
        "Invasion of Segovia",
        cost(&[generic(2), u()]),
        4,
        Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(kraken) },
        caetus,
    )
}

/// Gathering Place — Land. {T}: Add {C}. {T}: Add {G} or {W}; activate only
/// if this land entered this turn or if you control a basic land.
pub fn gathering_place() -> CardDefinition {
    let gated = |c: Color| ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![c]) },
        condition: Some(Predicate::Any(vec![
            Predicate::EntityMatches { what: Selector::This, filter: R::EnteredThisTurn },
            Predicate::SelectorExists(Selector::EachPermanent(R::IsBasicLand.and(R::ControlledByYou))),
        ])),
        ..Default::default()
    };
    CardDefinition {
        name: "Gathering Place",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add_colorless(), gated(Color::Green), gated(Color::White)],
        ..Default::default()
    }
}

/// In Bolas's Clutches — {4}{U}{U} Legendary Aura. Enchant permanent; you
/// control enchanted permanent, and it's legendary.
pub fn in_bolass_clutches() -> CardDefinition {
    CardDefinition {
        name: "In Bolas's Clutches",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Permanent) },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted permanent is legendary.",
            effect: StaticEffect::AttachedIsLegendary,
        }],
        ..Default::default()
    }
}

/// Roku's Mastery — {X}{R}{R} instant. X damage to target creature; if X is
/// 4 or more, scry 2.
pub fn rokus_mastery() -> CardDefinition {
    CardDefinition {
        name: "Roku's Mastery",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::XFromCost },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(4)),
                then: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Pet Avengers — {3}{G} 4/4 reach. Power-up — {6}{G}: a +1/+1 counter and a
/// 3/2 white Hero token with vigilance.
pub fn pet_avengers() -> CardDefinition {
    let hero = TokenDefinition {
        name: "Hero".into(),
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Hero], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Pet Avengers",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Cat, CreatureType::Dog, CreatureType::Bird, CreatureType::Frog, CreatureType::Hero],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), g()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(hero) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pelia, Immortal Innovator — {3}{U} legendary 3/3 Lanthanite Artificer.
/// Enters: return target creature an opponent controls to its owner's hand.
pub fn pelia_immortal_innovator() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(3), u()]),
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        })],
        ..legend("Pelia, Immortal Innovator", vec![CreatureType::Lanthanite, CreatureType::Artificer], 3, 3)
    }
}

/// Happy Hogan, Bodyguard — {5}{U} legendary 3/3 vigilance. Enters: the owner
/// of target creature an opponent controls puts it into their library second
/// from the top or on the bottom.
pub fn happy_hogan_bodyguard() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(5), u()]),
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::SecondFromTopOrBottom,
            },
        })],
        ..legend("Happy Hogan, Bodyguard", vec![CreatureType::Human, CreatureType::Advisor], 3, 3)
    }
}

/// Forebear's Blade — {3} Equipment. +3/+0, vigilance and trample; whenever
/// equipped creature dies, attach this to target creature you control. Equip {3}.
pub fn forebears_blade() -> CardDefinition {
    CardDefinition {
        name: "Forebear's Blade",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            keywords: vec![Keyword::Vigilance, Keyword::Trample],
            ..Default::default()
        }),
        // The death snapshot (`auras_at_death`) records Equipment riders too.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
        }],
        ..Default::default()
    }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        cost: mana,
        supertypes: vec![],
        ..legend(name, types, p, t)
    }
}

/// Rune of Flight — {1}{U} Aura Rune, enchant permanent. Enters: draw a card.
/// An enchanted creature has flying; on an Equipment, the creature it equips does.
pub fn rune_of_flight() -> CardDefinition {
    CardDefinition {
        name: "Rune of Flight",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Rune], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Permanent) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Flying],
            relays_through_equipment: true,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        ..Default::default()
    }
}

/// Zhalfirin Shapecraft — {1}{U} instant. Target creature has base P/T 4/3
/// until end of turn. Draw a card.
pub fn zhalfirin_shapecraft() -> CardDefinition {
    CardDefinition {
        name: "Zhalfirin Shapecraft",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SetBasePT {
                what: target_filtered(R::Creature),
                power: Value::Const(4),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Orchard Warden — {4}{G}{G} 4/6 Treefolk Shaman. Whenever another Treefolk
/// creature you control enters, you may gain life equal to its toughness.
pub fn orchard_warden() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(trigger_is(R::Creature.and(R::HasCreatureType(CreatureType::Treefolk)))),
            effect: Effect::MayDo {
                description: "Gain life equal to that Treefolk's toughness?".into(),
                body: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                }),
            },
        }],
        ..creature("Orchard Warden", cost(&[generic(4), g(), g()]), vec![CreatureType::Treefolk, CreatureType::Shaman], 4, 6)
    }
}

/// Ulvenwald Observer — {4}{G}{G} 6/6 Treefolk. Whenever a creature you
/// control with toughness 4 or greater dies, draw a card.
pub fn ulvenwald_observer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(trigger_is(R::ToughnessAtLeast(4))),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Ulvenwald Observer", cost(&[generic(4), g(), g()]), vec![CreatureType::Treefolk], 6, 6)
    }
}

fn changeling(name: &'static str, ability: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        activated_abilities: vec![ability],
        ..creature(name, cost(&[generic(2), b()]), vec![CreatureType::Shapeshifter], 2, 2)
    }
}

/// Ghostly Changeling — {2}{B} 2/2 changeling; {1}{B}: +1/+1 until end of turn.
pub fn ghostly_changeling() -> CardDefinition {
    changeling(
        "Ghostly Changeling",
        ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
            ..Default::default()
        },
    )
}

/// Moonglove Changeling — {2}{B} 2/2 changeling; {B}: deathtouch until end of turn.
pub fn moonglove_changeling() -> CardDefinition {
    changeling(
        "Moonglove Changeling",
        ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            ..Default::default()
        },
    )
}

/// Futurist Forge — {1}{U} artifact. Enters: draw a card. {3}{U}, sacrifice
/// it: draw two cards.
pub fn futurist_forge() -> CardDefinition {
    CardDefinition {
        name: "Futurist Forge",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// True Polymorph — {4}{U}{U} instant. Target artifact or creature becomes a
/// copy of another target artifact or creature.
pub fn true_polymorph() -> CardDefinition {
    let either = || R::Artifact.or(R::Creature);
    CardDefinition {
        name: "True Polymorph",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::BecomeCopyOf {
            what: target_filtered(either()),
            source: Selector::TargetFiltered { slot: 1, filter: either().and(R::OtherThanTargetSlot(0)) },
            extra_creature_types: vec![],
            keep_own_triggered: false,
            keep_own_activated: false,
            keep_name: false,
        },
        ..Default::default()
    }
}

/// Iceman and Firestar — {2}{U/R}{U/R} legendary 3/3 flying Mutant Hero.
/// Casting a blue spell taps up to one target creature; casting a red one,
/// you may discard a card to draw a card.
pub fn iceman_and_firestar() -> CardDefinition {
    let cast = |c: Color| EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(R::HasColor(c)));
    CardDefinition {
        cost: cost(&[generic(2), hybrid(Color::Blue, Color::Red), hybrid(Color::Blue, Color::Red)]),
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: cast(Color::Blue),
                effect: Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
                },
            },
            TriggeredAbility {
                event: cast(Color::Red),
                effect: Effect::MayDo {
                    description: "Discard a card to draw a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                    ])),
                },
            },
        ],
        ..legend("Iceman and Firestar", vec![CreatureType::Mutant, CreatureType::Hero], 3, 3)
    }
}

/// The Royal Scions — {1}{U}{R} Will Rowan, loyalty 5. +1: loot. +1: target
/// creature gets +2/+0, first strike and trample until end of turn. −8: draw
/// four; when you do, damage equal to your hand size to any target.
pub fn the_royal_scions() -> CardDefinition {
    let target = || Selector::TargetFiltered { slot: 0, filter: R::Creature };
    CardDefinition {
        name: "The Royal Scions",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Will, PlaneswalkerSubtype::Rowan], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::PumpPT { what: target(), power: Value::Const(2), toughness: Value::ZERO, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(4) },
                    // CR 603.7c — the reflexive trigger targets as it triggers.
                    Effect::ReflexiveTrigger {
                        body: Box::new(Effect::DealDamage {
                            to: crate::effect::shortcut::target_any(),
                            amount: Value::HandSizeOf(PlayerRef::You),
                        }),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Katerina of Myra's Marvels — {2}{W}{W} legendary 3/3 Human Performer,
/// partner. As it enters, choose a letter; casting a spell whose name begins
/// with it makes a 2/2 white flying Cat.
pub fn katerina_of_myras_marvels() -> CardDefinition {
    let cat = TokenDefinition {
        name: "Cat".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        cost: cost(&[generic(2), w(), w()]),
        keywords: vec![Keyword::Partner],
        as_enters_effect: Some(Effect::ChooseLetterForSource),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(trigger_is(R::NameBeginsWithChosenLetterOfSource)),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(cat) },
        }],
        ..legend("Katerina of Myra's Marvels", vec![CreatureType::Human, CreatureType::Performer], 3, 3)
    }
}

/// Safe Passage — {2}{W} instant. Prevent all damage that would be dealt to
/// you and creatures you control this turn. (The creatures are the ones you
/// control as it resolves — see INCOMPLETE_CARDS.)
pub fn safe_passage() -> CardDefinition {
    CardDefinition {
        name: "Safe Passage",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PreventAllDamageThisTurn { target: Selector::Player(PlayerRef::You), redirect_to: None },
            Effect::PreventAllDamageThisTurn {
                target: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                redirect_to: None,
            },
        ]),
        ..Default::default()
    }
}

/// Dwalin, Weaponmaster — {1}{R/W} legendary 2/1 first strike Dwarf Warrior.
/// Entering or attacking, a hone counter on each Equipment you control (each
/// grants +1/+0 to the equipped creature).
pub fn dwalin_weaponmaster() -> CardDefinition {
    let hone = || Effect::AddCounter {
        what: Selector::EachPermanent(R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou)),
        kind: CounterType::Hone,
        amount: Value::ONE,
    };
    CardDefinition {
        cost: cost(&[generic(1), hybrid(Color::Red, Color::White)]),
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![etb(hone()), on_attack(hone())],
        ..legend("Dwalin, Weaponmaster", vec![CreatureType::Dwarf, CreatureType::Warrior], 2, 1)
    }
}

/// Thorin Oakenshield — {R}{W} legendary 3/2 trample Dwarf Noble, storied.
/// With an enduring story, artifacts and creatures you control have ward {1}.
pub fn thorin_oakenshield() -> CardDefinition {
    CardDefinition {
        cost: cost(&[r(), w()]),
        keywords: vec![Keyword::Trample],
        static_abilities: vec![
            StaticAbility {
                description: "Storied (If you control three or more artifacts, legendaries, and/or Sagas, you have an enduring story for the rest of the game.)",
                effect: StaticEffect::Storied,
            },
            StaticAbility {
                description: "As long as you have an enduring story, artifacts and creatures you control have ward {1}.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::HasEnduringStory { who: PlayerRef::You },
                    inner: Box::new(StaticEffect::GrantKeyword {
                        applies_to: Selector::EachPermanent(R::Artifact.or(R::Creature).and(R::ControlledByYou)),
                        keyword: Keyword::Ward(WardCost::generic(1)),
                    }),
                },
            },
        ],
        ..legend("Thorin Oakenshield", vec![CreatureType::Dwarf, CreatureType::Noble], 3, 2)
    }
}
