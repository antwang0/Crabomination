//! Commander: the cards the **Upgrades Unleashed** precon (NEC, Chishiro, the
//! Shattered Blade) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_chishiro.rs`.
//!
//! "Modified" is CR 700.9 (`R::IsModified`): counters, Equipment, or an Aura
//! its controller controls.
//!
//! Residuals (each also on its card):
//! - **Concord with the Kami** — "choose one or more" picks every mode that
//!   can do something.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r, x};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn aura(name: &'static str, mana: ManaCost, host: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(host) },
        ..Default::default()
    }
}

fn token(name: &str, color: Option<Color>, p: i32, t: i32, kind: CreatureType, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors: color.into_iter().collect(),
        subtypes: Subtypes { creature_types: vec![kind], ..Default::default() },
        ..Default::default()
    })
}

fn colorless_spirit() -> Arc<TokenDefinition> {
    token("Spirit", None, 1, 1, CreatureType::Spirit, vec![])
}

/// CR 700.9 — a modified creature you control.
fn modified_yours() -> R {
    R::Creature.and(R::ControlledByYou).and(R::IsModified)
}

fn at_your(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

/// Akki Battle Squad — once a turn, when modified creatures you control
/// attack, untap them all and add a combat phase after this one.
pub fn akki_battle_squad() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsModified })
                .once_per_batch()
                .once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::Untap { what: Selector::EachPermanent(modified_yours()), up_to: None },
                Effect::AdditionalCombatPhase { count: Value::ONE },
            ]),
        }],
        ..creature(
            "Akki Battle Squad",
            cost(&[generic(5), r()]),
            vec![CreatureType::Goblin, CreatureType::Samurai],
            6,
            6,
        )
    }
}

/// Ascendant Acolyte — enters with a counter per +1/+1 counter among your
/// other creatures; doubles its own each upkeep.
pub fn ascendant_acolyte() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CountersOn {
                what: Box::new(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                )),
                kind: CounterType::PlusOnePlusOne,
            },
        )),
        triggered_abilities: vec![at_your(
            TurnStep::Upkeep,
            Effect::DoubleCountersOnEach { what: Selector::This, kind: CounterType::PlusOnePlusOne },
        )],
        ..creature(
            "Ascendant Acolyte",
            cost(&[generic(4), g()]),
            vec![CreatureType::Human, CreatureType::Monk],
            1,
            1,
        )
    }
}

/// Collision of Realms — each player shuffles the creatures they own into
/// their library; each who shuffled in a nontoken one reveals to a creature
/// card and puts it onto the battlefield, the rest on the bottom at random.
///
/// Per player in APNAP order: a player's shuffle touches only creatures they
/// own, so running each player's shuffle-then-reveal in turn is the
/// simultaneous result (the new creatures enter together before any
/// trigger is put on the stack).
pub fn collision_of_realms() -> CardDefinition {
    let owned = |filter: R| Selector::EachPermanent(R::Creature.and(R::OwnedByYou).and(filter));
    spell(
        "Collision of Realms",
        cost(&[generic(6), r()]),
        CardType::Sorcery,
        Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            body: Box::new(Effect::If {
                cond: Predicate::SelectorCountAtLeast { sel: owned(R::NotToken), n: Value::ONE },
                then: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: owned(R::Any),
                        to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Shuffled },
                    },
                    Effect::RevealUntilOneToBattlefieldRestBottom { filter: R::Creature, damage_controller: false },
                ])),
                else_: Box::new(Effect::Move {
                    what: owned(R::Any),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Shuffled },
                }),
            }),
        },
    )
}

/// Concord with the Kami — your end step: a counter on a creature with a
/// counter, a card with an enchanted creature, a Spirit with an equipped one.
/// ⚠ "Choose one or more" takes every mode that can do something.
pub fn concord_with_the_kami() -> CardDefinition {
    let control = |filter: R| Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(filter)),
        n: Value::ONE,
    };
    CardDefinition {
        triggered_abilities: vec![at_your(
            TurnStep::End,
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(R::Creature.and(R::WithAnyCounter)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    Effect::If {
                        cond: control(R::IsEnchanted),
                        then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                        else_: Box::new(Effect::Noop),
                    },
                    Effect::If {
                        cond: control(R::IsEquipped),
                        then: Box::new(Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: colorless_spirit(),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
        )],
        ..spell("Concord with the Kami", cost(&[generic(3), g()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Elemental Mastery — enchanted creature has "{T}: Create X 1/1 haste
/// Elementals, X its power; exile them at the next end step."
pub fn elemental_mastery() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::PowerOf(Box::new(Selector::This)),
                        definition: token(
                            "Elemental",
                            Some(Color::Red),
                            1,
                            1,
                            CreatureType::Elemental,
                            vec![Keyword::Haste],
                        ),
                    },
                    Effect::ExileLastCreatedTokensAtNextEndStep,
                ]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..aura("Elemental Mastery", cost(&[generic(3), r()]), R::Creature)
    }
}

/// Goblin Razerunners — {1}{R}, sacrifice a land: a +1/+1 counter; your end
/// step may deal its counters in damage to a player or planeswalker.
pub fn goblin_razerunners() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ..Default::default()
        }],
        triggered_abilities: vec![at_your(
            TurnStep::End,
            Effect::MayDo {
                description: "Deal damage equal to its +1/+1 counters?".into(),
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                }),
            },
        )],
        ..creature(
            "Goblin Razerunners",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Goblin, CreatureType::Warrior],
            3,
            4,
        )
    }
}

/// Kaima, the Fractured Calm — your end step goads each opposing creature
/// your Auras enchant, and grows Kaima by one per creature goaded.
pub fn kaima_the_fractured_calm() -> CardDefinition {
    let enchanted = || Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent).and(R::EnchantedByYourAura));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![at_your(
            TurnStep::End,
            Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CountOf(Box::new(enchanted())),
                },
                Effect::Goad { what: enchanted() },
            ]),
        )],
        ..creature(
            "Kaima, the Fractured Calm",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Kosei, Penitent Warlord — while enchanted, equipped and holding a counter,
/// its combat damage to an opponent draws that many and deals that much to
/// each other opponent.
pub fn kosei_penitent_warlord() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsEnchanted.and(R::IsEquipped).and(R::WithAnyCounter),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponentExceptTriggerer),
                    amount: Value::TriggerEventAmount,
                },
            ]),
        }],
        ..creature(
            "Kosei, Penitent Warlord",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Ogre, CreatureType::Samurai],
            0,
            5,
        )
    }
}

/// Mage Slayer — equipped creature attacking deals its power in damage to the
/// player or planeswalker it's attacking.
pub fn mage_slayer() -> CardDefinition {
    CardDefinition {
        name: "Mage Slayer",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![on_attack(Effect::DealDamageFrom {
                source: Selector::This,
                to: Selector::AttackedBySource,
                amount: Value::PowerOf(Box::new(Selector::This)),
            })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// One with the Kami — flash; when the enchanted creature or another modified
/// creature you control dies, a 1/1 Spirit per point of its power. Two
/// triggers: the Aura has left with its host by the time a death is
/// dispatched, so the host's half is the look-back `EnchantedBySource` scope
/// (CR 603.10a), and the watcher never sees the host.
pub fn one_with_the_kami() -> CardDefinition {
    let spirits = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::PowerOf(Box::new(Selector::TriggerSource)),
        definition: colorless_spirit(),
    };
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
                effect: spirits(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsModified }),
                effect: spirits(),
            },
        ],
        ..aura("One with the Kami", cost(&[generic(3), g()]), R::Creature.and(R::ControlledByYou))
    }
}

/// Orochi Merge-Keeper — {T}: {G}; {T}: {G}{G} while modified.
pub fn orochi_merge_keeper() -> CardDefinition {
    let green = |n: usize, condition: Option<Predicate>| ActivatedAbility {
        tap_cost: true,
        condition,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green; n]) },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            green(1, None),
            green(2, Some(Predicate::EntityMatches { what: Selector::This, filter: R::IsModified })),
        ],
        ..creature(
            "Orochi Merge-Keeper",
            cost(&[generic(1), g()]),
            vec![CreatureType::Snake, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Rampant Rejuvenator — enters with two +1/+1 counters; dying fetches up to
/// its power in basic lands onto the battlefield.
pub fn rampant_rejuvenator() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                count: Value::PowerOf(Box::new(Selector::This)),
            },
        }],
        ..creature(
            "Rampant Rejuvenator",
            cost(&[generic(3), g()]),
            vec![CreatureType::Plant, CreatureType::Hydra],
            0,
            0,
        )
    }
}

/// Silkguard — a counter on each of up to X creatures you control, then your
/// Auras, Equipment and modified creatures gain hexproof until end of turn.
pub fn silkguard() -> CardDefinition {
    spell(
        "Silkguard",
        cost(&[x(), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::CapTargetsAtX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                }),
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
                        .or(R::HasArtifactSubtype(ArtifactSubtype::Equipment))
                        .or(R::Creature.and(R::IsModified))
                        .and(R::ControlledByYou),
                ),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Smoke Blessing — the red Aura token Smoke Spirits' Aid makes: when the
/// enchanted creature dies, it deals 1 damage to its controller and you make
/// a Treasure.
fn smoke_blessing() -> TokenDefinition {
    TokenDefinition {
        name: "Smoke Blessing".into(),
        card_types: vec![CardType::Enchantment],
        colors: vec![Color::Red],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Seq(vec![
                Effect::DealDamageFrom {
                    source: Selector::TriggerSource,
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                    amount: Value::ONE,
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Smoke Spirits' Aid — a Smoke Blessing token attached to each of up to X
/// target creatures.
pub fn smoke_spirits_aid() -> CardDefinition {
    spell(
        "Smoke Spirits' Aid",
        cost(&[x(), r()]),
        CardType::Sorcery,
        Effect::CapTargetsAtX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::CreateTokenAttachedTo {
                    target: Selector::Target(0),
                    definition: Arc::new(smoke_blessing()),
                }),
            }),
        },
    )
}

/// Spearbreaker Behemoth — indestructible; {1}: a creature with power 5 or
/// greater gains indestructible until end of turn.
pub fn spearbreaker_behemoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Indestructible],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtLeast(5))),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Spearbreaker Behemoth", cost(&[generic(5), g(), g()]), vec![CreatureType::Beast], 5, 5)
    }
}

/// Tanuki Transplanter — when it or the equipped creature attacks, add {G}
/// per point of the attacker's power, kept until end of turn. Reconfigure {3}.
pub fn tanuki_transplanter() -> CardDefinition {
    let mana = || {
        on_attack(Effect::AddManaKeptThisTurnCount {
            who: PlayerRef::You,
            color: Color::Green,
            amount: Value::PowerOf(Box::new(Selector::This)),
        })
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Reconfigure(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { triggered_abilities: vec![mana()], ..Default::default() }),
        triggered_abilities: vec![mana()],
        ..creature("Tanuki Transplanter", cost(&[generic(3), g()]), vec![], 2, 4)
    }
}

/// Unquenchable Fury — enchanted creature's attacks deal the defending
/// player's hand size to them; back to your hand from the graveyard.
pub fn unquenchable_fury() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![on_attack(Effect::DealDamageFrom {
                source: Selector::This,
                to: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::HandSizeOf(PlayerRef::DefendingPlayer),
            })],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..aura("Unquenchable Fury", cost(&[generic(2), r()]), R::Creature)
    }
}

/// Vastwood Surge — kicker {4}; two basics onto the battlefield tapped, and
/// kicked, two +1/+1 counters on each creature you control.
pub fn vastwood_surge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(4)]))],
        ..spell(
            "Vastwood Surge",
            cost(&[generic(3), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    count: Value::Const(2),
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::AddCounter {
                        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}
