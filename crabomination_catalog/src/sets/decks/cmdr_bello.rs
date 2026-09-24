//! Commander: the cards the **Animated Army** precon (BLC, Bello, Bard of the
//! Brambles) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_bello.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, mint_treasures, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, r};
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

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn token(name: &str, color: Color, creature_type: CreatureType) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: vec![creature_type], ..Default::default() },
        ..Default::default()
    })
}

fn make(n: i32, def: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: def }
}

fn tapped_treasures(count: Value) -> Effect {
    let mut t = crabomination_base::tokens::treasure_token();
    t.tapped = true;
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(t) }
}

fn on_combat_damage_to_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect,
    }
}

/// "Whenever you cast a spell, if mana from a Treasure was spent to cast it."
fn cast_with_treasure() -> Predicate {
    Predicate::CastWithTreasureMana { what: Selector::TriggerSource }
}

fn cascade_for_trigger_source() -> Effect {
    Effect::Cascade { max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)) }
}

/// Bello, Bard of the Brambles — on your turn, your non-Equipment artifacts
/// and non-Aura enchantments of mana value 4 or more are 4/4 indestructible,
/// hasty Elementals that draw when they connect.
pub fn bello_bard_of_the_brambles() -> CardDefinition {
    let big = || {
        R::Artifact
            .and(R::Not(Box::new(R::HasArtifactSubtype(ArtifactSubtype::Equipment))))
            .or(R::Enchantment.and(R::Not(Box::new(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)))))
            .and(R::Not(Box::new(R::ManaValueAtMost(3))))
            .and(R::ControlledByYou)
    };
    let draw = || {
        on_combat_damage_to_player(Effect::Draw { who: Selector::You, amount: Value::ONE })
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "During your turn, your artifacts and enchantments of mana value 4+ are 4/4 Elementals with indestructible and haste.",
                effect: StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::MatchingLandsAreCreatures {
                        filter: big(),
                        power: 4,
                        toughness: 4,
                        keywords: vec![Keyword::Indestructible, Keyword::Haste],
                        creature_types: vec![CreatureType::Elemental],
                        colors: vec![],
                    }),
                },
            },
            StaticAbility {
                description: "…and have \"Whenever this creature deals combat damage to a player, draw a card.\"",
                effect: StaticEffect::WhileYourTurn {
                    inner: Box::new(StaticEffect::GrantTriggeredAbility {
                        filter: big(),
                        ability: Box::new(draw()),
                    }),
                },
            },
        ],
        ..creature(
            "Bello, Bard of the Brambles",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Raccoon, CreatureType::Bard],
            3,
            3,
        )
    }
}

/// Alchemist's Talent — two tapped Treasures; L2: Treasures tap for two of
/// one color; L3: a spell cast with Treasure mana deals its mana value to
/// each opponent (CR 716 Class levels).
pub fn alchemists_talent() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        triggered_abilities: vec![
            etb(tapped_treasures(Value::Const(2))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(
                    vec![Predicate::SourceClassLevelAtLeast(3), cast_with_treasure()],
                )),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                },
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Level 2 — Treasures you control have \"{T}, Sacrifice this artifact: Add two mana of any one color.\"",
            effect: StaticEffect::WhileClassLevelAtLeast {
                n: 2,
                inner: Box::new(StaticEffect::GrantActivatedAbility {
                    applies_to: yours(R::HasArtifactSubtype(ArtifactSubtype::Treasure)),
                    ability: ActivatedAbility {
                        tap_cost: true,
                        sac_cost: true,
                        effect: Effect::AddMana {
                            who: PlayerRef::You,
                            pool: ManaPayload::AnyOneColor(Value::Const(2)),
                        },
                        ..Default::default()
                    },
                    condition: None,
                }),
            },
        }],
        activated_abilities: vec![
            level_up(cost(&[generic(1), r()]), 1),
            level_up(cost(&[generic(4), r()]), 2),
        ],
        ..enchantment("Alchemist's Talent", cost(&[generic(3), r()]))
    }
}

/// Berserkers' Onslaught — your attackers have double strike.
pub fn berserkers_onslaught() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures you control have double strike.",
            effect: StaticEffect::GrantKeywordToAttackers { keyword: Keyword::DoubleStrike },
        }],
        ..enchantment("Berserkers' Onslaught", cost(&[generic(3), r(), r()]))
    }
}

/// Bootleggers' Stash — your lands tap for a Treasure.
pub fn bootleggers_stash() -> CardDefinition {
    CardDefinition {
        name: "Bootleggers' Stash",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Lands you control have \"{T}: Create a Treasure token.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: yours(R::Land),
                ability: ActivatedAbility { tap_cost: true, effect: mint_treasures(1), ..Default::default() },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Brightcap Badger // Fungus Frolic — your Fungi and Saprolings tap for
/// {G}; a Saproling each end step. Adventure: two Saprolings.
pub fn brightcap_badger() -> CardDefinition {
    let saproling = || token("Saproling", Color::Green, CreatureType::Saproling);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each Fungus and Saproling you control has \"{T}: Add {G}.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: yours(
                    R::HasCreatureType(CreatureType::Fungus).or(R::HasCreatureType(CreatureType::Saproling)),
                ),
                ability: super::super::tap_add(Color::Green),
                condition: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: make(1, saproling()),
        }],
        adventure: Some(Box::new(Adventure {
            name: "Fungus Frolic",
            cost: cost(&[generic(2), g()]),
            card_types: vec![CardType::Instant],
            effect: make(2, saproling()),
        })),
        ..creature(
            "Brightcap Badger",
            cost(&[generic(3), g()]),
            vec![CreatureType::Badger, CreatureType::Druid],
            3,
            4,
        )
    }
}

/// Evercoat Ursine — hideaway 3 twice; connecting plays one of them free.
/// ⚠ A land among them can't be played this way (the free path casts).
pub fn evercoat_ursine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Hideaway { count: Value::Const(3) },
                Effect::Hideaway { count: Value::Const(3) },
            ])),
            TriggeredAbility {
                // CR 603.4 — "if there are cards exiled with it".
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                    .with_filter(Predicate::ValueAtLeast(Value::CardsExiledWithSourceCount, Value::ONE)),
                effect: Effect::CastWithoutPayingImmediate {
                    what: Selector::one_of(Selector::CardExiledWithSource),
                    source_zone: Zone::Exile,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
            },
        ],
        ..creature(
            "Evercoat Ursine",
            cost(&[generic(4), g()]),
            vec![CreatureType::Elemental, CreatureType::Bear],
            6,
            5,
        )
    }
}

/// Grothama, All-Devouring — any other attacker may fight it; when it
/// leaves, each player draws the damage their sources dealt it this turn.
/// ⚠ The fight offer is Grothama's trigger asking the attacker's controller
/// (`MayDoBy`), not an ability granted to each creature, so its controller
/// is Grothama's for APNAP ordering.
pub fn grothama_all_devouring() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource },
                ),
                effect: Effect::MayDoBy {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    description: "Have the attacking creature fight Grothama?".into(),
                    body: Box::new(Effect::Fight { attacker: Selector::TriggerSource, defender: Selector::This }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::EachPlayerDrawsDamageTheyDealtToSource,
            },
        ],
        ..creature(
            "Grothama, All-Devouring",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Wurm],
            10,
            8,
        )
    }
}

/// Grumgully, the Generous — your other non-Humans enter with a +1/+1
/// counter.
pub fn grumgully_the_generous() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Each other non-Human creature you control enters with an additional +1/+1 counter on it.",
            effect: StaticEffect::MatchingEntersWithExtraCounters {
                filter: R::Creature
                    .and(R::ControlledByYou)
                    .and(R::Not(Box::new(R::IsSource)))
                    .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Human)))),
                kind: CounterType::PlusOnePlusOne,
                amount: 1,
            },
        }],
        ..creature(
            "Grumgully, the Generous",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Goblin, CreatureType::Shaman],
            3,
            3,
        )
    }
}

/// Kodama of the East Tree — another permanent of yours entering (not by
/// this ability) lets you put a permanent card of equal or lesser mana value
/// from your hand onto the battlefield. Partner.
pub fn kodama_of_the_east_tree() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Reach, Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            // CR 603.4 — the intervening "if it wasn't put onto the
            // battlefield with this ability".
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::PutOntoBattlefieldBySource,
                })),
            ),
            effect: Effect::WithX {
                x: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                body: Box::new(Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::PermanentCard.and(R::ManaValueAtMostXFromCost),
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                }),
            },
        }],
        ..creature(
            "Kodama of the East Tree",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Spirit],
            6,
            6,
        )
    }
}

/// Prosperous Bandit — offspring {1}; connecting makes that many tapped
/// Treasures.
pub fn prosperous_bandit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Offspring(cost(&[generic(1)]))],
        triggered_abilities: vec![on_combat_damage_to_player(tapped_treasures(Value::TriggerEventAmount))],
        ..creature(
            "Prosperous Bandit",
            cost(&[generic(2), r()]),
            vec![CreatureType::Raccoon, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Pyreswipe Hawk — attacking, +X/+0 for your biggest artifact; expend 6
/// steals up to one artifact while you control it.
pub fn pyreswipe_hawk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ManaValueOf(Box::new(Selector::GreatestManaValueControlledMatching {
                        who: PlayerRef::You,
                        filter: R::Artifact,
                    })),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                    .with_filter(Predicate::ExpendReached(6)),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::GainControlWhileYouControlSource {
                        what: target_filtered(R::Artifact),
                    }),
                },
            },
        ],
        ..creature(
            "Pyreswipe Hawk",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Bird],
            4,
            4,
        )
    }
}

/// Rain of Riches — two Treasures; the first spell each turn cast with
/// Treasure mana has cascade.
pub fn rain_of_riches() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(mint_treasures(2)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(cast_with_treasure())
                    .once_per_turn(),
                effect: cascade_for_trigger_source(),
            },
        ],
        ..enchantment("Rain of Riches", cost(&[generic(3), r(), r()]))
    }
}

/// Rolling Hamsphere — +1/+1 per Hamster; attacking makes three Hamsters,
/// then deals damage equal to your Hamsters to any target. Crew 3.
pub fn rolling_hamsphere() -> CardDefinition {
    let hamster = R::HasCreatureType(CreatureType::Hamster);
    CardDefinition {
        name: "Rolling Hamsphere",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Crew(3)],
        static_abilities: vec![StaticAbility {
            description: "This Vehicle gets +1/+1 for each Hamster you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: hamster.clone(),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                make(3, token("Hamster", Color::Red, CreatureType::Hamster)),
                Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::PermanentCountControlledByMatching(PlayerRef::You, hamster),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Sunbird's Invocation — a spell cast from hand reveals that many cards and
/// offers one of that mana value or less free.
pub fn sunbirds_invocation() -> CardDefinition {
    let mv = || Value::ManaValueOf(Box::new(Selector::TriggerSource));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::CastFromHand),
            effect: Effect::RevealTopMayCastOneFree { count: mv(), max_mv: mv() },
        }],
        ..enchantment("Sunbird's Invocation", cost(&[generic(5), r()]))
    }
}

/// Tendershoot Dryad — ascend; a Saproling every upkeep; Saprolings are +2/+2
/// with the city's blessing. ⚠ Ascend is checked as it enters and at each
/// upkeep, like the other ascend permanents here (CR 702.131b checks
/// continuously).
pub fn tendershoot_dryad() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Ascend { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
                effect: Effect::Seq(vec![
                    Effect::Ascend { who: PlayerRef::You },
                    make(1, token("Saproling", Color::Green, CreatureType::Saproling)),
                ]),
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Saprolings you control get +2/+2 as long as you have the city's blessing.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::HasCityBlessing { who: PlayerRef::You },
                inner: Box::new(StaticEffect::PumpPT {
                    applies_to: yours(R::HasCreatureType(CreatureType::Saproling)),
                    power: 2,
                    toughness: 2,
                }),
            },
        }],
        ..creature("Tendershoot Dryad", cost(&[generic(4), g()]), vec![CreatureType::Dryad], 2, 2)
    }
}

/// Trailtracker Scout — taps for any color; expend 8 returns up to one
/// permanent card from your graveyard to hand.
pub fn trailtracker_scout() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![super::super::tap_add_any_color()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Expend, EventScope::YourControl).with_filter(Predicate::ExpendReached(8)),
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Move {
                    what: target_filtered(R::PermanentCard.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..creature(
            "Trailtracker Scout",
            cost(&[generic(1), g()]),
            vec![CreatureType::Raccoon, CreatureType::Scout],
            1,
            3,
        )
    }
}

/// Wildsear, Scouring Maw — enchantment spells cast from your hand cascade.
pub fn wildsear_scouring_maw() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastFromHand,
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Enchantment },
            ])),
            effect: cascade_for_trigger_source(),
        }],
        ..creature(
            "Wildsear, Scouring Maw",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Elemental, CreatureType::Wolf],
            6,
            6,
        )
    }
}
