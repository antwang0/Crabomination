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
        Effect::ChooseN { up_to: None,
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
            effect: StaticEffect::AttachedHasSupertype(Supertype::Legendary),
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

/// Ordinary Bear — {3}{G} 4/5 Bear.
pub fn ordinary_bear() -> CardDefinition {
    creature("Ordinary Bear", cost(&[generic(3), g()]), vec![CreatureType::Bear], 4, 5)
}

/// Quake, Agent of S.H.I.E.L.D. — {2}{W} legendary 3/3 Inhuman Spy Hero.
/// Whenever you cast a noncreature spell, tap target creature or land.
pub fn quake_agent_of_shield() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(2), w()]),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(R::Noncreature)),
            effect: Effect::Tap { what: target_filtered(R::Creature.or(R::Land)) },
        }],
        ..legend("Quake, Agent of S.H.I.E.L.D.", vec![CreatureType::Inhuman, CreatureType::Spy, CreatureType::Hero], 3, 3)
    }
}

/// Denzilore Fatehold — {1}{W}{U}{U} legendary 3/4 flash flying Elder
/// Sphinx. Whenever you scry or surveil, a +1/+1 counter on each creature
/// you control.
pub fn denzilore_fatehold() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(1), w(), u(), u()]),
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ScriedOrSurveiled, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..legend("Denzilore Fatehold", vec![CreatureType::Elder, CreatureType::Sphinx], 3, 4)
    }
}

/// Gorging Vulture — {2}{B} 2/2 flying Bird. Enters: mill four, gain 1 life
/// per creature card milled.
pub fn gorging_vulture() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(4) },
            Effect::GainLife { who: Selector::You, amount: Value::CreatureCardsMilledThisEffect },
        ]))],
        ..creature("Gorging Vulture", cost(&[generic(2), b()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Sapphire Dragon — {5}{U}{U} 5/6 flying Dragon; attacking or blocking,
/// scry 2. Adventure: Psionic Pulse {2}{U} instant — counter target
/// noncreature spell.
pub fn sapphire_dragon() -> CardDefinition {
    let scry = || Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_attack(scry()),
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: scry() },
        ],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Psionic Pulse",
            cost: cost(&[generic(2), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::Noncreature)) },
        })),
        ..creature("Sapphire Dragon", cost(&[generic(5), u(), u()]), vec![CreatureType::Dragon], 5, 6)
    }
}

/// Shortcut to Mushrooms — {1}{G} enchantment. Enters: the Ring tempts you.
/// At your end step, if a permanent you controlled left the battlefield this
/// turn, a +1/+1 counter on target creature you control.
pub fn shortcut_to_mushrooms() -> CardDefinition {
    let revolt = Predicate::RevoltActive { who: PlayerRef::You };
    CardDefinition {
        name: "Shortcut to Mushrooms",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::RingTempts { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(revolt.clone()),
                effect: Effect::If {
                    cond: revolt,
                    then: Box::new(Effect::AddCounter {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Restoration Specialist — {1}{W} 2/1 Dwarf Artificer. {W}, sacrifice it:
/// return up to one target artifact card and up to one target enchantment
/// card from your graveyard to your hand.
pub fn restoration_specialist() -> CardDefinition {
    let back = |slot: u8, kind: CardType| Effect::Move {
        what: Selector::TargetFiltered { slot, filter: R::HasCardType(kind).and(R::InYourGraveyard) },
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_cost: true,
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![back(0, CardType::Artifact), back(1, CardType::Enchantment)])),
            },
            ..Default::default()
        }],
        ..creature("Restoration Specialist", cost(&[generic(1), w()]), vec![CreatureType::Dwarf, CreatureType::Artificer], 2, 1)
    }
}

/// Meddle — {1}{U} instant. If target spell has only one target and that
/// target is a creature, change that spell's target to another creature.
pub fn meddle() -> CardDefinition {
    CardDefinition {
        name: "Meddle",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChangeSpellCreatureTarget { what: target_filtered(R::IsSpellOnStack) },
        ..Default::default()
    }
}

/// Mizzium Meddler — {2}{U} 1/4 flash Vedalken Wizard. Enters: you may change
/// a target of target spell or ability to this creature.
pub fn mizzium_meddler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Change a target of that spell or ability to Mizzium Meddler?".into(),
            body: Box::new(Effect::RedirectSpellTargetToSelf {
                what: target_filtered(R::IsSpellOnStack.or(R::HasAbilityOnStack)),
            }),
        })],
        ..creature("Mizzium Meddler", cost(&[generic(2), u()]), vec![CreatureType::Vedalken, CreatureType::Wizard], 1, 4)
    }
}

/// Dáin's Company — {R}{W} 2/2 Dwarf Warrior; lifelink while you control
/// another Dwarf. Enters: look at the top four, you may reveal a Dwarf or
/// Equipment card and put it into your hand, the rest on the bottom.
pub fn dains_company() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has lifelink as long as you control another Dwarf.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Lifelink,
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Dwarf).and(R::ControlledByYou).and(R::OtherThanSource),
                )),
            },
        }],
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(crate::effect::LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(R::HasCreatureType(CreatureType::Dwarf).or(R::HasArtifactSubtype(ArtifactSubtype::Equipment))),
            take: Some(Value::ONE),
            optional: true,
            ..Default::default()
        })))],
        ..creature("Dáin's Company", cost(&[r(), w()]), vec![CreatureType::Dwarf, CreatureType::Warrior], 2, 2)
    }
}

/// Picard, Leading by Example — {1}{W} legendary 2/2 Human Officer. {1}, {T}:
/// another target creature gets +X/+X until end of turn, X = Picard's power.
/// Sorcery speed.
pub fn picard_leading_by_example() -> CardDefinition {
    let x = || Value::PowerOf(Box::new(Selector::This));
    CardDefinition {
        cost: cost(&[generic(1), w()]),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::OtherThanSource)),
                power: x(),
                toughness: x(),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legend("Picard, Leading by Example", vec![CreatureType::Human, CreatureType::Officer], 2, 2)
    }
}

/// Ant-Man, Reformed Rogue — {2}{G/U} legendary 2/3 Human Rogue Hero. A green
/// spell: +1/+0 and trample; a blue spell: -1/-0 and unblockable, until end
/// of turn. Combat damage to a player draws a card.
pub fn ant_man_reformed_rogue() -> CardDefinition {
    let cast = |c: Color| EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(R::HasColor(c)));
    let shift = |p: i32, kw: Keyword| {
        Effect::Seq(vec![
            Effect::PumpPT { what: Selector::This, power: Value::Const(p), toughness: Value::ZERO, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::This, keyword: kw, duration: Duration::EndOfTurn },
        ])
    };
    CardDefinition {
        cost: cost(&[generic(2), hybrid(Color::Green, Color::Blue)]),
        triggered_abilities: vec![
            TriggeredAbility { event: cast(Color::Green), effect: shift(1, Keyword::Trample) },
            TriggeredAbility { event: cast(Color::Blue), effect: shift(-1, Keyword::Unblockable) },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..legend("Ant-Man, Reformed Rogue", vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Hero], 2, 3)
    }
}

/// Cosmos Charger — {3}{U} 3/3 flash flying Horse Spirit. Foretelling cards
/// from your hand costs {1} less and can be done on any player's turn.
/// Foretell {2}{U}.
pub fn cosmos_charger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        foretell_cost: Some(cost(&[generic(2), u()])),
        static_abilities: vec![StaticAbility {
            description: "Foretelling cards from your hand costs {1} less and can be done on any player's turn.",
            effect: StaticEffect::ForetellCostsLessAnyTurn,
        }],
        ..creature("Cosmos Charger", cost(&[generic(3), u()]), vec![CreatureType::Horse, CreatureType::Spirit], 3, 3)
    }
}

/// Alrund, God of the Cosmos — {3}{U}{U} legendary 1/1 God; +1/+1 for each
/// card in your hand and each foretold card you own in exile. At your end
/// step, choose a card type, reveal the top two, take the chosen type, bottom
/// the rest. Back: Hakka, Whispering Raven — {1}{U} legendary 2/3 flying
/// Bird; combat damage to a player returns it to its owner's hand, then scry 2.
pub fn alrund_god_of_the_cosmos() -> CardDefinition {
    let hakka = CardDefinition {
        cost: cost(&[generic(1), u()]),
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            ]),
        }],
        ..legend("Hakka, Whispering Raven", vec![CreatureType::Bird], 2, 3)
    };
    CardDefinition {
        cost: cost(&[generic(3), u(), u()]),
        static_abilities: vec![StaticAbility {
            description: "Alrund gets +1/+1 for each card in your hand and each foretold card you own in exile.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::Sum(vec![Value::HandSizeOf(PlayerRef::You), Value::ForetoldCardsOwnedInExile(PlayerRef::You)]),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::ChooseCardTypeForSource,
                Effect::RevealTopTakeMatchingToHand {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    filter: R::IsSourceChosenCardType,
                    distinct_powers: false,
                },
            ]),
        }],
        back_face: Some(Box::new(hakka)),
        ..legend("Alrund, God of the Cosmos", vec![CreatureType::God], 1, 1)
    }
}

/// Campus Renovation — {3}{R}{W} sorcery. Return up to one target artifact or
/// enchantment card from your graveyard to the battlefield. Exile the top two
/// cards of your library; until the end of your next turn, you may play them.
pub fn campus_renovation() -> CardDefinition {
    CardDefinition {
        name: "Campus Renovation",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Artifact.or(R::HasCardType(CardType::Enchantment)).and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            ])),
        },
        ..Default::default()
    }
}

/// Repair and Recharge — {3}{W}{W} sorcery. Return target artifact,
/// enchantment, or planeswalker card from your graveyard to the battlefield.
/// Create a tapped Powerstone token.
pub fn repair_and_recharge() -> CardDefinition {
    CardDefinition {
        name: "Repair and Recharge",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    R::Artifact
                        .or(R::HasCardType(CardType::Enchantment))
                        .or(R::HasCardType(CardType::Planeswalker))
                        .and(R::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() }),
            },
        ]),
        ..Default::default()
    }
}

/// Marit Lage's Slumber — {1}{U} legendary snow enchantment. Whenever it or
/// another snow permanent you control enters, scry 1. At your upkeep, with ten
/// or more snow permanents, sacrifice it; if you do, Marit Lage (a legendary
/// 20/20 black flying indestructible Avatar).
pub fn marit_lages_slumber() -> CardDefinition {
    let marit = TokenDefinition {
        name: "Marit Lage".into(),
        power: 20,
        toughness: 20,
        keywords: vec![Keyword::Flying, Keyword::Indestructible],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Avatar], ..Default::default() },
        ..Default::default()
    };
    let ten_snow = Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::IsSnow.and(R::ControlledByYou)),
        n: Value::Const(10),
    };
    CardDefinition {
        name: "Marit Lage's Slumber",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary, Supertype::Snow],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::Any(vec![Predicate::TriggerSourceIsSelf, trigger_is(R::IsSnow)])),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(ten_snow.clone()),
                effect: Effect::If {
                    cond: ten_snow,
                    then: Box::new(Effect::Seq(vec![
                        Effect::SacrificeSource,
                        Effect::If {
                            cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                            then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(marit) }),
                            else_: Box::new(Effect::Noop),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Absorb Identity — {1}{U} instant. Return target creature to its owner's
/// hand; you may have Shapeshifters you control become copies of it until end
/// of turn (copied as it is targeted, before it leaves).
pub fn absorb_identity() -> CardDefinition {
    CardDefinition {
        name: "Absorb Identity",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::MayDo {
                description: "Have your Shapeshifters become copies of it until end of turn?".into(),
                body: Box::new(Effect::BecomeCopyOfFor {
                    what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Shapeshifter).and(R::ControlledByYou)),
                    source: Selector::Target(0),
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                }),
            },
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        ]),
        ..Default::default()
    }
}

/// Nimbleclaw Adept — {3}{U} 2/3 Dragon Wizard. Bigby's Hand — {T}: untap two
/// other target permanents; sorcery speed, once each turn.
pub fn nimbleclaw_adept() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            once_per_turn: true,
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 2,
                filter: R::Permanent.and(R::OtherThanSource),
                effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
            },
            ..Default::default()
        }],
        ..creature("Nimbleclaw Adept", cost(&[generic(3), u()]), vec![CreatureType::Dragon, CreatureType::Wizard], 2, 3)
    }
}

/// Dynaheir, Invoker Adept — {1}{U}{R}{W} legendary 4/4 haste Human Wizard.
/// Your other creatures' abilities activate as though they had haste. {T}:
/// when you next activate a non-mana ability this turn by spending four or
/// more mana, copy it.
pub fn dynaheir_invoker_adept() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(1), u(), r(), w()]),
        keywords: vec![Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "You may activate abilities of other creatures you control as though those creatures had haste.",
            effect: StaticEffect::ControllerCreatureAbilitiesAsThoughHaste,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CopyNextCostlyAbilityThisTurn { min_mana: 4 },
            ..Default::default()
        }],
        ..legend("Dynaheir, Invoker Adept", vec![CreatureType::Human, CreatureType::Wizard], 4, 4)
    }
}

/// Aisha of Sparks and Smoke — {1}{R}{R} legendary 4/2 prowess Human Warrior.
/// {R/W}: first strike until end of turn. Whenever she deals combat damage,
/// you may cast a sorcery from your hand with mana value up to that damage
/// without paying its mana cost.
pub fn aisha_of_sparks_and_smoke() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(1), r(), r()]),
        keywords: vec![Keyword::Prowess],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[hybrid(Color::Red, Color::White)]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamage, EventScope::SelfSource),
            effect: Effect::MayCastFromHandFreeMatching {
                filter: R::HasCardType(CardType::Sorcery),
                max_mv: Value::TriggerEventAmount,
                else_: Box::new(Effect::Noop),
            },
        }],
        ..legend("Aisha of Sparks and Smoke", vec![CreatureType::Human, CreatureType::Warrior], 4, 2)
    }
}

/// Howl of the Horde — {2}{R} sorcery. When you next cast an instant or
/// sorcery spell this turn, copy it (twice with raid).
pub fn howl_of_the_horde() -> CardDefinition {
    CardDefinition {
        name: "Howl of the Horde",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CopyNextInstantSorceryThisTurn {
            count: Value::Sum(vec![Value::ONE, crate::effect::shortcut::raid_value(1)]),
        },
        ..Default::default()
    }
}

/// Witch-king, Sky Scourge — {5}{B}{R} legendary 5/5 flying undying Wraith
/// Noble. Whenever you attack with one or more Wraiths, exile the top X cards
/// of your library (X = their total power); you may play them this turn.
pub fn witch_king_sky_scourge() -> CardDefinition {
    let wraiths = || Selector::EachPermanent(R::HasCreatureType(CreatureType::Wraith).and(R::IsAttacking).and(R::ControlledByYou));
    CardDefinition {
        cost: cost(&[generic(5), b(), r()]),
        keywords: vec![Keyword::Flying, Keyword::Undying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(Predicate::SelectorExists(wraiths())),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::PowerOf(Box::new(wraiths())),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..legend("Witch-king, Sky Scourge", vec![CreatureType::Wraith, CreatureType::Noble], 5, 5)
    }
}

/// Mordor on the March — {3}{B}{R} sorcery, storm. Exile a creature card from
/// your graveyard; a token copy with haste, exiled at the next end step.
/// (The card is targeted rather than chosen — see INCOMPLETE_CARDS.)
pub fn mordor_on_the_march() -> CardDefinition {
    CardDefinition {
        name: "Mordor on the March",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Storm],
        effect: Effect::Seq(vec![
            Effect::CreateTokenCopyOf {
                extra_keywords: vec![],
                who: PlayerRef::You,
                count: Value::ONE,
                source: target_filtered(R::Creature.and(R::InYourGraveyard)),
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            },
            Effect::GrantKeyword { what: Selector::LastCreatedToken, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            Effect::Exile { what: Selector::Target(0) },
            Effect::DelayUntil {
                kind: crate::effect::DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::Exile { what: Selector::LastCreatedToken }),
            },
        ]),
        ..Default::default()
    }
}

/// Stalwarts of Osgiliath — {4}{W} 4/3 Human Soldier. Enters: the Ring
/// tempts you. Your second card drawn each turn: a +1/+1 counter on it.
pub fn stalwarts_of_osgiliath() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::RingTempts { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..creature("Stalwarts of Osgiliath", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Soldier], 4, 3)
    }
}

/// Lakeshore Apothecary — {1}{U} 1/2 vigilance Human Cleric; your second card
/// drawn each turn puts a +1/+1 counter on it.
pub fn lakeshore_apothecary() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Lakeshore Apothecary", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 2)
    }
}

/// She-Hulk, Jade Defender — {3}{G} legendary 4/4 reach trample Gamma Hero.
/// Power-up — {4}{G}{G}: destroy up to one target artifact or enchantment,
/// and a +1/+1 counter on her.
pub fn she_hulk_jade_defender() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(3), g()]),
        keywords: vec![Keyword::Reach, Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), g()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Artifact.or(R::HasCardType(CardType::Enchantment)),
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..legend("She-Hulk, Jade Defender", vec![CreatureType::Gamma, CreatureType::Hero], 4, 4)
    }
}

/// Madame Hydra, Reanimated — {3}{B} legendary 2/2 menace Human Horror
/// Villain; +1/+1 per creature card in your graveyard; entering or attacking,
/// mill two.
pub fn madame_hydra_reanimated() -> CardDefinition {
    let mill = || Effect::Mill { who: Selector::You, amount: Value::Const(2) };
    CardDefinition {
        cost: cost(&[generic(3), b()]),
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Madame Hydra gets +1/+1 for each creature card in your graveyard.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::CountOf(Box::new(Selector::EachMatching {
                    zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                    filter: R::Creature,
                })),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![etb(mill()), on_attack(mill())],
        ..legend("Madame Hydra, Reanimated", vec![CreatureType::Human, CreatureType::Horror, CreatureType::Villain], 2, 2)
    }
}

/// Shake the Foundations — {2}{R} instant. 1 damage to each creature without
/// flying; draw a card.
pub fn shake_the_foundations() -> CardDefinition {
    CardDefinition {
        name: "Shake the Foundations",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))))),
                amount: Value::ONE,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Legolas, Counter of Kills — {2}{G}{U} legendary 2/3 reach Elf Archer.
/// Whenever you scry, if he's tapped, you may untap him (once each turn).
/// Whenever a creature an opponent controls dies, a +1/+1 counter on him.
pub fn legolas_counter_of_kills() -> CardDefinition {
    let tapped = Predicate::EntityMatches { what: Selector::This, filter: R::Tapped };
    CardDefinition {
        cost: cost(&[generic(2), g(), u()]),
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![
            TriggeredAbility {
                // "Do this only once each turn": spent by a taken untap.
                event: EventSpec::new(EventKind::Scried, EventScope::YourControl)
                    .with_filter(Predicate::All(vec![tapped.clone(), crate::card::Predicate::Not(Box::new(crate::card::Predicate::SourceDoneThisTurn))])),
                effect: Effect::If {
                    cond: tapped,
                    then: Box::new(Effect::MayDo {
                        description: "Untap Legolas?".into(),
                        body: Box::new(Effect::Seq(vec![
                            Effect::MarkDoneThisTurn,
                            Effect::Untap { what: Selector::This, up_to: None },
                        ])),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..legend("Legolas, Counter of Kills", vec![CreatureType::Elf, CreatureType::Archer], 2, 3)
    }
}

/// Tolarian Kraken — {4}{U}{U} 4/6 Kraken. Whenever you draw a card, you may
/// pay {1}; when you do, you may tap or untap target creature.
pub fn tolarian_kraken() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1} to tap or untap target creature?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(Effect::TapOrUntap { what: target_filtered(R::Creature) }),
                }),
                else_: None,
            },
        }],
        ..creature("Tolarian Kraken", cost(&[generic(4), u(), u()]), vec![CreatureType::Kraken], 4, 6)
    }
}

/// Ghost-Spider, Gwen Stacy — {3}{R}{R} legendary 4/4 menace Spider Human
/// Hero. Attacking, X damage to the defending player, X = attacking creatures.
pub fn ghost_spider_gwen_stacy() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(3), r(), r()]),
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::DealDamage {
            to: Selector::Player(PlayerRef::DefendingPlayer),
            amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::IsAttacking)))),
        })],
        ..legend("Ghost-Spider, Gwen Stacy", vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero], 4, 4)
    }
}

/// Web-Shooters — {1}{W} Equipment. +1/+1, reach, and "Whenever this
/// creature attacks, tap target creature an opponent controls." Equip {2}.
pub fn web_shooters() -> CardDefinition {
    CardDefinition {
        name: "Web-Shooters",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Reach],
            triggered_abilities: vec![on_attack(Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Michelangelo, On the Scene — {4}{G}{G} legendary 2/2 trample Mutant Ninja
/// Turtle; enters with a +1/+1 counter per land you control; dies: return it
/// to your hand.
pub fn michelangelo_on_the_scene() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(4), g(), g()]),
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CountOf(Box::new(Selector::EachPermanent(R::Land.and(R::ControlledByYou)))),
        )),
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..legend("Michelangelo, On the Scene", vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle], 2, 2)
    }
}

/// X-23, Deadly Weapon — {2}{G} legendary 3/3 Mutant Berserker Hero. Whenever
/// another Mutant you control enters, a +1/+1 counter on it and on X-23.
pub fn x_23_deadly_weapon() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(2), g()]),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(trigger_is(R::HasCreatureType(CreatureType::Mutant))),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::TriggerSource, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ]),
        }],
        ..legend("X-23, Deadly Weapon", vec![CreatureType::Mutant, CreatureType::Berserker, CreatureType::Hero], 3, 3)
    }
}

/// Splinter, Hamato Yoshi — {1}{B} legendary 1/3 menace Mutant Ninja Rat;
/// sneak {B}; other Ninjas you control get +1/+1.
pub fn splinter_hamato_yoshi() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(1), b()]),
        keywords: vec![Keyword::Menace],
        alternative_cost: Some(crate::effect::shortcut::sneak(cost(&[b()]))),
        static_abilities: vec![StaticAbility {
            description: "Other Ninjas you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Ninja).and(R::ControlledByYou).and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        ..legend("Splinter, Hamato Yoshi", vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Rat], 1, 3)
    }
}

/// Doom's Servo-Guards — {1}{B} 2/2 artifact Robot Villain. Enters: gain 2
/// life and mill two.
pub fn dooms_servo_guards() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Mill { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..creature("Doom's Servo-Guards", cost(&[generic(1), b()]), vec![CreatureType::Robot, CreatureType::Villain], 2, 2)
    }
}

/// Nimrodel Watcher — {1}{U} 2/1 Elf Scout. Whenever you scry, +1/+0 and
/// unblockable this turn; once each turn.
pub fn nimrodel_watcher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec { once_per_turn: true, ..EventSpec::new(EventKind::Scried, EventScope::YourControl) },
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ZERO, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("Nimrodel Watcher", cost(&[generic(1), u()]), vec![CreatureType::Elf, CreatureType::Scout], 2, 1)
    }
}

/// Little Bear — {2}{G} 3/2 flash Bear. Enters: untap another target creature
/// you control; a Bear gets a +1/+1 counter too.
pub fn little_bear() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Untap { what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)), up_to: None },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::HasCreatureType(CreatureType::Bear) },
                then: Box::new(Effect::AddCounter { what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Little Bear", cost(&[generic(2), g()]), vec![CreatureType::Bear], 3, 2)
    }
}

/// Bard, Heir of Girion — {2}{W}{U} legendary 4/4 reach vigilance Human
/// Archer; other creatures you control get +1/+1; whenever you attack, draw.
pub fn bard_heir_of_girion() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(2), w(), u()]),
        keywords: vec![Keyword::Reach, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend("Bard, Heir of Girion", vec![CreatureType::Human, CreatureType::Archer], 4, 4)
    }
}

/// Lucky the Pizza Dog — {1}{G} legendary 2/2 Dog. Casting a Cat, Dog or
/// Hero spell makes a Food; at each end step, if you gained life this turn,
/// a +1/+1 counter on Lucky.
pub fn lucky_the_pizza_dog() -> CardDefinition {
    let gained = Predicate::PlayerGainedLifeThisTurn { who: PlayerRef::You };
    CardDefinition {
        cost: cost(&[generic(1), g()]),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(
                    R::HasCreatureType(CreatureType::Cat)
                        .or(R::HasCreatureType(CreatureType::Dog))
                        .or(R::HasCreatureType(CreatureType::Hero)),
                )),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(crabomination_base::tokens::food_token()),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(gained.clone()),
                effect: Effect::If {
                    cond: gained,
                    then: Box::new(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..legend("Lucky the Pizza Dog", vec![CreatureType::Dog], 2, 2)
    }
}

/// Mind Control — {3}{U}{U} Aura. Enchant creature; you control it.
pub fn mind_control() -> CardDefinition {
    CardDefinition {
        name: "Mind Control",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        ..Default::default()
    }
}

/// Beorn, Reluctant Host — {4}{G} legendary 5/5 trample Human Bear
/// Shapeshifter. Adventure: Till and Tend {1}{G} sorcery — you may play an
/// additional land this turn.
pub fn beorn_reluctant_host() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(4), g()]),
        keywords: vec![Keyword::Trample],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Till and Tend",
            cost: cost(&[generic(1), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::ONE },
        })),
        ..legend("Beorn, Reluctant Host", vec![CreatureType::Human, CreatureType::Bear, CreatureType::Shapeshifter], 5, 5)
    }
}

/// Young Blue Dragon — {4}{U} 3/3 flying Dragon. Adventure: Sand Augury
/// {1}{U} sorcery — scry 1, then draw a card.
pub fn young_blue_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Sand Augury",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        })),
        ..creature("Young Blue Dragon", cost(&[generic(4), u()]), vec![CreatureType::Dragon], 3, 3)
    }
}

/// Venomized Cat — {2}{B} 2/3 deathtouch Symbiote Cat Villain; enters: mill two.
pub fn venomized_cat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::Mill { who: Selector::You, amount: Value::Const(2) })],
        ..creature("Venomized Cat", cost(&[generic(2), b()]), vec![CreatureType::Symbiote, CreatureType::Cat, CreatureType::Villain], 2, 3)
    }
}

/// Spider-Man, To the Rescue — {2}{G/W} legendary 3/2 flash reach vigilance
/// Spider Human Hero. Enters: you may tap him; when you do, another target
/// nonattacking creature you control gains indestructible until end of turn.
pub fn spider_man_to_the_rescue() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(2), hybrid(Color::Green, Color::White)]),
        keywords: vec![Keyword::Flash, Keyword::Reach, Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Tap Spider-Man to make another creature indestructible?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::This },
                Effect::ReflexiveTrigger {
                    body: Box::new(Effect::GrantKeyword {
                        what: target_filtered(
                            R::Creature.and(R::ControlledByYou).and(R::OtherThanSource).and(R::Not(Box::new(R::IsAttacking))),
                        ),
                        keyword: Keyword::Indestructible,
                        duration: Duration::EndOfTurn,
                    }),
                },
            ])),
        })],
        ..legend("Spider-Man, To the Rescue", vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero], 3, 2)
    }
}

/// West Wind Avatar — {5}{G}{G} 7/7 trample Cat Spirit Avatar. Entering or
/// attacking, you may sacrifice a token or a land to gain 3 life. Disappear —
/// at your end step, if a permanent left the battlefield under your control
/// this turn, draw a card.
pub fn west_wind_avatar() -> CardDefinition {
    let feed = || Effect::MaySacrifice {
        description: "Sacrifice a token or a land to gain 3 life?".into(),
        filter: R::IsToken.or(R::Land),
        count: Value::ONE,
        then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
        else_: None,
    };
    let revolt = Predicate::RevoltActive { who: PlayerRef::You };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(feed()),
            on_attack(feed()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(revolt.clone()),
                effect: Effect::If {
                    cond: revolt,
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..creature("West Wind Avatar", cost(&[generic(5), g(), g()]), vec![CreatureType::Cat, CreatureType::Spirit, CreatureType::Avatar], 7, 7)
    }
}

/// Bearded Axe — {2}{R} Equipment. Equipped creature gets +1/+1 for each
/// Dwarf, Equipment, and/or Vehicle you control. Equip {2}.
pub fn bearded_axe() -> CardDefinition {
    CardDefinition {
        name: "Bearded Axe",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            scale: Some(crate::card::EquipScale {
                filter: R::HasCreatureType(CreatureType::Dwarf)
                    .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment))
                    .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                per_power: 1,
                per_toughness: 1,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Great Gilded Boat — {2}{U} 4/4 Vehicle, crew 2. Whenever you attack,
/// recruit: draw, then discard; a nonland discard makes a 1/1 white Human
/// Soldier.
pub fn great_gilded_boat() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Great Gilded Boat",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::If {
                    cond: Predicate::DiscardedNonlandThisEffect { who: PlayerRef::You },
                    then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(soldier) }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Whtz, the Bibliophile — {1}{W}{U} legendary 1/3 Homunculus. Rulebreaker:
/// no maximum deck size. {3}, {T}: draw a card and gain 1 life. (The "{3}
/// less with a 200-card starting deck" discount is not modelled —
/// INCOMPLETE_CARDS.)
pub fn whtz_the_bibliophile() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(1), w(), u()]),
        static_abilities: vec![StaticAbility {
            description: "Rulebreaker — A deck with this commander has no maximum deck size.",
            effect: StaticEffect::RulebreakerNoMaxDeckSize,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..legend("Whtz, the Bibliophile", vec![CreatureType::Homunculus], 1, 3)
    }
}

/// Mishra, Artificer Prodigy — {1}{U}{B}{R} legendary 4/4 Human Artificer.
/// Casting an artifact spell, you may search your graveyard, hand and/or
/// library for a card with its name and put it onto the battlefield.
pub fn mishra_artificer_prodigy() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        cost: cost(&[generic(1), u(), b(), r()]),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(R::Artifact)),
            effect: Effect::MayDo {
                description: "Search for a card with the same name and put it onto the battlefield?".into(),
                body: Box::new(Effect::WithTargets {
                    what: Selector::TriggerSource,
                    body: Box::new(Effect::SearchZones {
                        who: PlayerRef::You,
                        zones: vec![Zone::Graveyard, Zone::Hand, Zone::Library],
                        filter: R::SameNameAsTarget,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                }),
            },
        }],
        ..legend("Mishra, Artificer Prodigy", vec![CreatureType::Human, CreatureType::Artificer], 4, 4)
    }
}

/// Aragorn, Company Leader — {1}{G}{W} legendary 3/3 Human Ranger. The Ring
/// tempting you with another creature chosen as Ring-bearer puts your choice
/// of a first strike, vigilance, deathtouch or lifelink counter on him; you
/// putting counters on him puts one of each of those kinds on up to one other
/// target creature.
pub fn aragorn_company_leader() -> CardDefinition {
    let counter = |keyword: Keyword| Effect::AddKeywordCounter { what: Selector::This, keyword, amount: Value::ONE };
    CardDefinition {
        cost: cost(&[generic(1), g(), w()]),
        triggered_abilities: vec![
            TriggeredAbility {
                // CR 701.54c — the chosen bearer is the event's subject.
                event: EventSpec::new(EventKind::RingTempted, EventScope::YourControl)
                    .with_filter(trigger_is(R::OtherThanSource)),
                effect: Effect::ChooseMode(vec![
                    counter(Keyword::FirstStrike),
                    counter(Keyword::Vigilance),
                    counter(Keyword::Deathtouch),
                    counter(Keyword::Lifelink),
                ]),
            },
            TriggeredAbility {
                // CR 122.6 — keyed on who put them; one trigger per kind put.
                event: EventSpec::new(EventKind::AnyCounterAdded, EventScope::YouPutCounters)
                    .with_filter(trigger_is(R::OtherThanSource.negate())),
                effect: Effect::AddCounterOfTriggerKind {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    amount: Value::ONE,
                    event_placer: false,
                },
            },
        ],
        ..legend("Aragorn, Company Leader", vec![CreatureType::Human, CreatureType::Ranger], 3, 3)
    }
}

/// Blood Sun — {2}{R} enchantment. Enters: draw a card. All lands lose all
/// abilities except mana abilities (CR 113.10b, layer 6).
pub fn blood_sun() -> CardDefinition {
    CardDefinition {
        name: "Blood Sun",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        static_abilities: vec![StaticAbility {
            description: "All lands lose all abilities except mana abilities.",
            effect: StaticEffect::LandsLoseNonManaAbilities,
        }],
        ..Default::default()
    }
}

/// Herald of Leshrac — {6}{B} 2/4 flying Avatar. Cumulative upkeep—gain
/// control of a land you don't control. +1/+1 for each land you control but
/// don't own. Leaving the battlefield, each player gains control of each land
/// they own that you control.
pub fn herald_of_leshrac() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::CumulativeUpkeep(crate::card::CumulativeUpkeepCost::GainControlOfLand),
        ],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each land you control but don't own.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::Land.and(R::ControlledByYou).and(R::OwnedByYou.negate()),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::OwnersGainControlOf { filter: R::Land.and(R::ControlledByYou) },
        }],
        ..creature("Herald of Leshrac", cost(&[generic(6), b()]), vec![CreatureType::Avatar], 2, 4)
    }
}

fn rulebreaker(description: &'static str, cards: crate::effect::RulebreakerCards) -> StaticAbility {
    StaticAbility { description, effect: StaticEffect::RulebreakerAnyColorIdentity { cards } }
}

/// Maular, the Next Evolution — {5}{G}{G} legendary 10/10 Dinosaur Mutant.
/// Rulebreaker: creature cards with mana value 7 or greater of any color
/// identity and any basic lands. A creature of yours with mana value 7 or
/// greater attacking doubles its power and toughness until end of turn.
pub fn maular_the_next_evolution() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(5), g(), g()]),
        static_abilities: vec![rulebreaker(
            "Rulebreaker — creature cards with mana value 7 or greater of any color identity, and any basic land cards.",
            crate::effect::RulebreakerCards::CreaturesWithManaValueAtLeast(7),
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(trigger_is(R::Creature.and(R::ManaValueAtLeast(7)))),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::PowerOf(Box::new(Selector::TriggerSource)),
                toughness: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend("Maular, the Next Evolution", vec![CreatureType::Dinosaur, CreatureType::Mutant], 10, 10)
    }
}

/// Seluma, Light of Aysen — {4}{W} legendary 4/4 flying Angel Warrior.
/// Rulebreaker: Angel cards of any color identity and any basic lands. Combat
/// damage to a player returns target Angel creature card from your graveyard
/// to the battlefield.
pub fn seluma_light_of_aysen() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(4), w()]),
        keywords: vec![Keyword::Flying],
        static_abilities: vec![rulebreaker(
            "Rulebreaker — Angel cards of any color identity, and any basic land cards.",
            crate::effect::RulebreakerCards::OfCreatureType(CreatureType::Angel),
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Angel)).and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..legend("Seluma, Light of Aysen", vec![CreatureType::Angel, CreatureType::Warrior], 4, 4)
    }
}

/// The Everforger — {6} legendary 4/5 artifact Construct. Rulebreaker:
/// artifact creature and Equipment cards of any color identity and any basic
/// lands. Casting an artifact creature or Equipment spell, you may copy it,
/// once each turn (spent by a copy, not by a declined trigger).
pub fn the_everforger() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![rulebreaker(
            "Rulebreaker — artifact creature and Equipment cards of any color identity, and any basic land cards.",
            crate::effect::RulebreakerCards::ArtifactCreaturesAndEquipment,
        )],
        triggered_abilities: vec![crate::effect::shortcut::may_once_each_turn(
            EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(trigger_is(
                R::Artifact.and(R::Creature).or(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
            )),
            "Copy that spell?",
            Effect::CopySpell { what: Selector::TriggerSource, count: Value::ONE },
        )],
        ..legend("The Everforger", vec![CreatureType::Construct], 4, 5)
    }
}

/// Dralnu, Lich Lord — {3}{U}{B} legendary 3/3 Zombie Wizard. Damage to him
/// is replaced by sacrificing that many permanents (CR 614). {T}: target
/// instant or sorcery card in your graveyard gains flashback until end of
/// turn, at its mana cost.
pub fn dralnu_lich_lord() -> CardDefinition {
    CardDefinition {
        cost: cost(&[generic(3), u(), b()]),
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to Dralnu, sacrifice that many permanents instead.",
            effect: StaticEffect::ReplaceDamageToSelfWithSacrifices,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantFlashbackThisTurn {
                what: target_filtered(R::InYourGraveyard.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))),
            },
            ..Default::default()
        }],
        ..legend("Dralnu, Lich Lord", vec![CreatureType::Zombie, CreatureType::Wizard], 3, 3)
    }
}

/// Ninja Teen — {2}{B} Class. A creature of yours leaving the battlefield
/// drains each opponent for 1. Level 2 ({1}{B}): creatures you control get
/// +1/+0 and have menace. Level 3 ({B}): creature cards in your graveyard have
/// sneak {3}{B}, and creature spells may be cast from your graveyard by sneak
/// (CR 702.190).
pub fn ninja_teen() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    let at = |n: u8, description: &'static str, inner: StaticEffect| StaticAbility {
        description,
        effect: StaticEffect::WhileClassLevelAtLeast { n, inner: Box::new(inner) },
    };
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Ninja Teen",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl)
                .with_filter(trigger_is(R::Creature)),
            effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        }],
        static_abilities: vec![
            at(2, "Creatures you control get +1/+0.", StaticEffect::PumpPT { applies_to: yours(), power: 1, toughness: 0 }),
            at(2, "Creatures you control have menace.", StaticEffect::GrantKeyword { applies_to: yours(), keyword: Keyword::Menace }),
            at(
                3,
                "Creature cards in your graveyard have sneak {3}{B}; you may cast creature spells from your graveyard using their sneak abilities.",
                StaticEffect::GraveyardCreaturesHaveSneak { cost: cost(&[generic(3), b()]) },
            ),
        ],
        activated_abilities: vec![level_up(cost(&[generic(1), b()]), 1), level_up(cost(&[b()]), 2)],
        ..Default::default()
    }
}
